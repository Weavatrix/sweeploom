---
title: "Inside SweepLoom: sixteen crates and no async runtime"
date: "2026-10-12"
summary: "A tool that deletes files should be boring on the inside. How SweepLoom's workspace is split into crates, which invariants every crate keeps, why cargo-deny bans tokio, hyper and reqwest, and how the desktop app scans in parallel with nothing but std threads and channels."
tags: ["architecture", "rust", "engineering", "safety"]
---

When the job of a program is to decide what can be removed from your disk, the interesting property is not cleverness. It is predictability: the same input gives the same plan, the plan is checked again before anything happens, and you can read the code that does it. SweepLoom's architecture is mostly a set of decisions that keep it that way. This post walks through them, from the crate map to the absence of an async runtime.

## The crate map

SweepLoom is one Cargo workspace with fifteen library and binary crates plus the desktop app:

```text
sweeploom-core        no OS APIs, no egui
sweeploom-platform    paths, trash, process control
sweeploom-process     sysinfo snapshots
sweeploom-session     logical grouping + forgotten score
sweeploom-network     capability-gated connections
sweeploom-storage     weavatrix-scan inventory
sweeploom-exec        plan / revalidate / receipt
sweeploom-dev         Cargo / Node / Python workspace analyzers
sweeploom-ai          inspect-first AI stores, classify, token tax
sweeploom-general     temp, logs, dumps, Downloads review
sweeploom-rules       declarative TOML cleaner rules
sweeploom-browser     companion protocol: tab heat, discard, Later
sweeploom-history     bounded in-memory observed history
sweeploom-macos       AppKit/Foundation/libc bridge for the App Store edition
sweeploom (cli)       CLI, MCP server, Rust library facade
sweeploom-gui         egui + eframe (glow)
```

The layering matters more than the count. `sweeploom-core` holds the vocabulary (candidates, safety, recommendations, redaction) and is not allowed to touch the operating system or the UI. Everything that does touch the OS sits in a thin crate of its own, so the rules can be tested without a real machine. `sweeploom-exec` owns the plan, revalidate and receipt sequence for generated cleanup, and its Trash path for user data. The CLI, the MCP server and the desktop app's Project output all apply through it; only native objects (Docker, `simctl`, Ollama, `rustup`) are handed to their own tools.

The newest crate, `sweeploom-macos`, exists for the sandboxed Mac App Store edition, which is the desktop app built with the `mas` cargo feature (coming to the Mac App Store, currently in TestFlight testing). It wraps every Objective-C and libc call that edition needs in a safe function: the folder picker that grants access to a folder, security-scoped bookmarks that keep a grant across launches, Move to Trash through `NSFileManager`, Show in Finder through `NSWorkspace`, and the real home folder, because inside the sandbox `$HOME` is the app container. On other platforms it compiles to the same API and returns `Unsupported`. The free full edition does not depend on it.

The `sweeploom` crate is the facade. It is the CLI binary, the MCP server (behind a default `mcp` feature, using the MIT-licensed `mcport` crate for stdio JSON-RPC) and the library you can depend on from Git. Because the CLI, MCP and library are the same crate, an agent calling `cleanup_candidates` and a person typing `sweeploom clean .` get the same rows.

## The license boundary

SweepLoom is MPL-2.0. It builds on Weavatrix libraries such as `weavatrix-scan` (the parallel filesystem walker) and `weavatrix-git`, which are MIT and stay MIT. When SweepLoom needs a scanning or Git capability those libraries lack, the change goes into the library and is consumed here. Nothing is vendored, forked or relicensed. That keeps the boundary visible in `Cargo.toml` rather than buried in a copied folder.

## Invariants every crate keeps

A short list, enforced by tests rather than by convention:

- **Safety and recommendation are independent axes.** "This looks forgotten" and "this is safe to remove" are different questions, computed separately.
- **A recommendation never bypasses a blocker.** Blockers are explicit: an active process, a recent write, uncommitted or untracked Git files, unknown Git state, a symlink or reparse point escaping the root, another volume, a protected or pinned path, a candidate that changed after the plan.
- **Processes are keyed by PID plus start time.** A PID the OS recycled is never mistaken for the process you reviewed.
- **Command lines are redacted** (`--token` values, credentials in URIs) before they reach the UI, logs or receipts.
- **No LLM on the destructive path**, and no telemetry by default.

The test names read like the list: `recommendation_cannot_bypass_blocker`, `blockers_force_blocked_level`, `redacts_flag_value_and_next_token`, `refuses_src_root_dot_and_escape`, `mcp_apply_without_confirm_does_not_run`.

## Why there is no async runtime

The workspace's `deny.toml` says it directly:

```toml
[bans]
deny = [
    { name = "tokio", reason = "SweepLoom MVP is std::thread + channels, not an async runtime" },
    { name = "hyper", reason = "no network client stack in the base product" },
    { name = "reqwest", reason = "local-first: no cloud HTTP client in the base product" },
]
```

`cargo deny` runs in CI, so adding any of them, even transitively, fails the build. The reasons are practical:

- **The work is blocking anyway.** Reading directories, `lstat`, process tables and `xcrun simctl` are blocking system calls. An async runtime would push them onto a blocking thread pool, which is what plain threads already are.
- **The MCP server is stdio.** One host, one pipe, requests answered in order. There is nothing to multiplex.
- **There is no network.** The CLI and MCP server make no requests, and the dependency tree carries no HTTP client. Banning the stack is the cheapest way to keep that claim true as dependencies change. (The desktop app's only network use, listing and removing Ollama models on `127.0.0.1:11434`, needs no HTTP stack either.)
- **Smaller is easier to audit.** A tool that deletes files should have a dependency tree a person can read.

## Concurrency with threads and channels

None of this means SweepLoom is single-threaded. This release made every expensive scan parallel, with two small pieces in `sweeploom-storage`:

```rust
/// Worker count for metadata-heavy walks: the CPU count, clamped so a slow
/// disk is not flooded with concurrent `stat` calls.
pub fn walk_workers() -> usize {
    std::thread::available_parallelism()
        .map_or(4, std::num::NonZeroUsize::get)
        .clamp(2, 12)
}

/// Map `items` on up to `workers` scoped threads. Results keep input order.
pub fn parallel_map<T, R, F>(items: Vec<T>, workers: usize, map: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Sync,
```

`parallel_map` uses `std::thread::scope`: no global pool, no `'static` bounds, and every worker is joined before it returns. Project discovery, the Project output rebuild and project sizes all run through it. The Explorer walk uses a folder-level work queue on the same worker count, where each folder folds into its parent once its last child finishes, so memory is bounded by open folders.

The desktop app adds `crossbeam-channel` for the other direction. Each scan runs on its own thread and sends messages back to the UI, which polls without blocking a frame:

```rust
enum ListingMsg {
    Initial(Listing),
    Measured(Item),
    Done,
    Failed(String),
}
```

`Initial` puts rows on screen as soon as they are discovered, `Measured` replaces one row when its size lands, and the table never blanks while a rescan runs. Git probes for a selected session also run off the UI thread and are cached.

## When something does go wrong

The desktop app's scan threads are wrapped in `catch_unwind`. In Cleanup, a panic becomes a `Failed` message ("Storage inventory failed; refresh to retry") instead of a dead window. That only works if release builds unwind, which is set in the workspace profile with a comment explaining why:

```toml
[profile.release]
lto = "fat"
codegen-units = 1
# Unwind so the GUI's worker `catch_unwind` contains a failed scan instead of
# aborting the whole app.
panic = "unwind"
strip = "symbols"
```

Because symbols are stripped, the panic hook also appends the message and source location to a log (`~/Library/Logs/SweepLoom/panic.log` on macOS), so a crash report can be traced to a line.

## Budgets as architecture

Two more rules keep the codebase readable. The workspace forbids `unsafe` code (`unsafe_code = "forbid"`) and warns on missing docs. Two crates opt out: `sweeploom-network`, for Windows-only TCP byte counters, and `sweeploom-macos`, which denies `unsafe` by default and allows it item by item, each next to a `SAFETY:` note. And a CI step called *Architecture budget* fails any tracked Rust, Markdown, TOML or YAML file over **300 lines** and any Rust function over **100 lines**. It is blunt, and it is why the GUI is many small files with names like `cleanup_ios_merge.rs` instead of a few large ones.

## The trade-offs

- **Threads are not free.** Each scan owns a thread, and worker pools are capped at twelve so a slow disk is not flooded. On very large machines a scan can still take minutes; it streams so you are not staring at a spinner.
- **No async means no concurrent network features.** That is a choice, not an oversight. A future web UI is a separate product and does not have to inherit these bans.
- **Budgets cost some indirection.** Splitting by line count occasionally separates code a reader would like to see together.

The crate map and invariants are in [docs/ARCHITECTURE.md](https://github.com/Weavatrix/sweeploom/blob/main/docs/ARCHITECTURE.md), and the test inventory is in [docs/TESTING.md](https://github.com/Weavatrix/sweeploom/blob/main/docs/TESTING.md).
