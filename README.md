<div align="center">

# SweepLoom

### Reclaim your workstation without losing your workspace

[![CI](https://github.com/Weavatrix/sweeploom/actions/workflows/ci.yml/badge.svg)](https://github.com/Weavatrix/sweeploom/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MPL--2.0-orange)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.88%2B-000000?logo=rust)](https://www.rust-lang.org/)

**[Website](https://sweeploom.com)** · **[Docs](#documentation)** · **[Blog](https://sweeploom.com/blog/)** · **[Terms](TERMS.md)**

A [Weavatrix](https://github.com/Weavatrix) product · [github.com/Weavatrix/sweeploom](https://github.com/Weavatrix/sweeploom)

</div>

---

SweepLoom is a local-first, **developer-aware workstation resource
manager**. It maps what is sitting on disk, in RAM, and inside your
coding-agent hosts — build output, package caches, Xcode and Docker
leftovers, AI models, forgotten MCP helpers, and the always-on prompt tax
of agent rules — and tells you what is regenerable junk and what is still
earning its keep. Live chats stay **Keep**. Nothing is removed until you
have reviewed it and confirmed, and every apply is revalidated first. It
ships as a CLI, an MCP server for coding agents, a Codex plugin, a Rust
library, and an optional desktop app, with no LLM on classify or apply
and no telemetry.

<table>
  <tr>
    <td align="center" width="50%"><img src="docs/screenshots/overview.png" alt="Overview screen: memory, per-core CPU, and disk pressure with top reclaim opportunities" width="420"><br><sub><b>Overview</b></sub></td>
    <td align="center" width="50%"><img src="docs/screenshots/sessions.png" alt="Sessions screen: coding agents, terminals, and MCP helpers grouped into logical sessions" width="420"><br><sub><b>Sessions</b></sub></td>
  </tr>
  <tr>
    <td align="center" width="50%"><img src="docs/screenshots/projects.png" alt="Projects screen: project heat, Git state, and build artifact sizes" width="420"><br><sub><b>Projects</b></sub></td>
    <td align="center" width="50%"><img src="docs/screenshots/cleanup-ios.png" alt="Cleanup screen, iOS Simulator tab: devices and runtimes with sizes" width="420"><br><sub><b>Cleanup · iOS Simulator</b></sub></td>
  </tr>
  <tr>
    <td align="center" width="50%"><img src="docs/screenshots/review.png" alt="Review screen: generated cleanup candidates before apply" width="420"><br><sub><b>Review</b></sub></td>
    <td align="center" width="50%"><img src="docs/screenshots/ai.png" alt="AI screen: Claude and Codex stores with always-on token estimates" width="420"><br><sub><b>AI</b></sub></td>
  </tr>
</table>

## What it finds

| What | Examples | Where | How it is handled |
| --- | --- | --- | --- |
| **Generated build output** | `target/`, `node_modules/`, Vite cache, `__pycache__` | CLI · MCP · App | SAFE generated: dry run, then `--apply` / `confirm: true`; revalidated first |
| **Package-manager caches** | npm/npx, pnpm, Yarn, Bun, Python, Cargo, Gradle, NuGet, Go, Homebrew, SwiftPM, Playwright/Puppeteer | App | Confirmed generated cleanup, or move to Trash |
| **Xcode and iOS** | DerivedData by project, iOS/watchOS/tvOS DeviceSupport by version, Simulator devices and runtimes | App | Native `simctl` delete/erase with running-device checks; generated cleanup for DerivedData |
| **Docker** | Images, stopped containers, unused volumes, old build cache | App | Native Docker operations; in-use objects are protected |
| **AI models** | Ollama, Hugging Face, LM Studio, PyTorch, Whisper, Core ML | App | Ollama's local API, or Trash; never assumed regenerable |
| **Archives and large data** | Xcode archives, device backups, large downloads and installers, logs, Maven artifacts, inactive Rust toolchains, Node versions | App | Trash for user data; `rustup` uninstall for inactive toolchains |
| **Live sessions, coding agents, MCP servers** | Cursor, Claude Code, Codex, OpenCode, Gemini, Grok trees; attached vs orphan MCP; dev servers | CLI · MCP · App | Live agents and attached MCP are **Keep**; CLI and MCP never terminate |
| **Browser tabs** | Browser process trees and RSS; tab `lastAccessed` via the companion | CLI · MCP · App | Unknown stays unknown, never zero; discard or focus only on request |
| **AI always-on prompt tax** | `AGENTS.md`, `*.mdc`, `rules`, `skills`, `plugins` in `~/.claude`, `~/.codex`, … | CLI · MCP · App | Capped token estimate; History counts **0**; never edits the contract |

The desktop Cleanup sources and their exact actions are in
[docs/CLEANUP.md](docs/CLEANUP.md). Why “idle” agents stay Keep, and why
History is not prompt tax: [docs/CONCEPTS.md](docs/CONCEPTS.md).

## Install

SweepLoom is installed **from source**. It is not on crates.io or npm
yet, and there are no prebuilt releases. With [Rust](https://rustup.rs/)
1.88+:

```text
cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom
```

That installs `sweeploom` (the CLI; `sweeploom mcp` is the MCP server)
and `sweeploom-companion-host` (for the optional browser companion).
Clones, updates, and every option: [docs/INSTALL.md](docs/INSTALL.md).

### Quickstart

```text
sweeploom --help
sweeploom sessions
sweeploom ai
sweeploom bench
sweeploom scan .
sweeploom clean .               # look first
sweeploom clean . --apply       # then delete SAFE generated rows
```

```text
$ sweeploom clean .
[x]	1.2 GB	cargo target
[x]	812.0 MB	node_modules
[ ]	12.0 KB	something inspect-only	BLOCKED
dry-run; pass --apply to delete pre-selected SAFE rows after revalidation
```

Full transcripts from a real workstation: [docs/CLI.md](docs/CLI.md) and
[docs/CLI-CLEAN.md](docs/CLI-CLEAN.md).

## Desktop app

```text
git clone https://github.com/Weavatrix/sweeploom.git
cd sweeploom
cargo run --release -p sweeploom-gui
```

On macOS, build the signed `.app` so folder permissions survive updates:
`python3 scripts/macos-app.py` ([docs/MACOS.md](docs/MACOS.md)).

- **Overview** — memory, per-core CPU, and disk pressure, with the top
  reclaim opportunities.
- **Sessions** — processes grouped into logical sessions (agents,
  terminals, dev servers, MCP helpers) with members and project; stopping
  one is an explicit, confirmed action.
- **History** — observed CPU and memory per session since SweepLoom
  started; never back-filled.
- **Review** — generated cleanup candidates, applied after revalidation.
- **Explorer** — folder inspector tree; collapsing never re-walks the disk.
- **Projects** — project heat, Git state, artifact sizes, group actions.
- **Cleanup** — Docker, iOS Simulator, build and package caches, app and
  browser caches, AI models, archives and large data.
- **Scan history** — saved scans and folder growth over time.
- **Browser** — browser process trees, companion tab heat, the Later shelf.
- **AI** — inspect-first Claude, Codex, Cursor, OpenCode, Gemini, and Grok
  stores with always-on token estimates.

Settings are local-only; telemetry is off. An optional tray mode runs on
Windows and macOS.

## MCP for agents

Point any MCP host at the installed binary:

```json
{
  "mcpServers": {
    "sweeploom": {
      "command": "sweeploom",
      "args": ["mcp"]
    }
  }
}
```

```text
claude mcp add -s user sweeploom -- sweeploom mcp
```

Nine tools; eight are read-only, and `apply_cleanup` deletes only with
`confirm: true` after the user agreed to the exact rows. There is no
terminate tool. Per-host setup and the argument table:
[docs/MCP.md](docs/MCP.md). Copy-paste agent chats:
[docs/MCP-CHATS.md](docs/MCP-CHATS.md). Codex plugin (MCP + skills):
[docs/PLUGIN.md](docs/PLUGIN.md).

## Safety model

Generated cleanup — CLI, MCP, and the app's Review screen — is always
**plan → revalidate → execute → receipt**:

1. **Plan** — build candidates from metadata, without reading file
   contents. Only SAFE regenerable rows can be pre-selected.
2. **Revalidate** — re-check every row at apply time. A row that changed
   since the plan is skipped (`skipped_changed`), not forced.
3. **Execute** — only on explicit consent: `--apply` on the CLI,
   `confirm: true` from an agent after your OK, or your confirmation in
   the app.
4. **Receipt** — every apply reports what was deleted, skipped, or failed.

The desktop Cleanup screen re-checks each item before acting (live paths,
running simulators, in-use Docker objects, loaded Ollama models) and moves
user data to the Trash rather than deleting it.

Idle agents are **Keep**: killing a live Claude, Codex, or Cursor drops
the chat you were trying to make cheaper. Nothing is deleted
automatically; AI stores, History, secrets, SQLite, user data, and live
agents are never pre-selected. Details and the class table:
[docs/SAFETY.md](docs/SAFETY.md).

## Benchmarks

Fixed gold store, with SweepLoom vs without (re-run with `sweeploom bench`):

| Check | Without SweepLoom | With SweepLoom |
| --- | --- | --- |
| Always-on prompt tax | 238,466,250 tok (store dump) · 965,000 (History as prompt) | **16,288 tok** |
| Classify 30 gold names | 24/30 (substring rules) | **30/30**, ~2 µs/call, no LLM |
| Idle 2 GB agents | 6 would be killed | **6 Keep** |
| `apply_cleanup` without `confirm` | deletes whatever an agent invents | **refused, 0 deleted** |

These are policy fixtures, not coding-agent spend. A live coding-agent
run found **no** token saving (+75.5% estimated spend with the MCP
attached). All numbers and commands: [docs/BENCHMARKS.md](docs/BENCHMARKS.md)
and [docs/BENCHMARKS-LIVE.md](docs/BENCHMARKS-LIVE.md).

## Documentation

| Document | What is inside |
| --- | --- |
| [INSTALL.md](docs/INSTALL.md) | Install from source, desktop app, MCP hosts, Codex plugin |
| [CONCEPTS.md](docs/CONCEPTS.md) | The idea: two kinds of waste, why live agents are Keep, scope |
| [CLI.md](docs/CLI.md) | CLI transcripts: help, sessions, scan, projects, AI stores |
| [CLI-CLEAN.md](docs/CLI-CLEAN.md) | CLI transcripts: bench, browser, clean, agent contract, companion |
| [MCP.md](docs/MCP.md) | MCP setup per host, silent vs told, every tool and its arguments |
| [MCP-CHATS.md](docs/MCP-CHATS.md) | Agent chats A–B: review then apply; the refused shortcut |
| [MCP-CHATS-AGENTS.md](docs/MCP-CHATS-AGENTS.md) | Agent chats C–E: token tax, idle Cursor, folder map |
| [PLUGIN.md](docs/PLUGIN.md) | Codex plugin: install, skills, layout |
| [LIBRARY.md](docs/LIBRARY.md) | Rust library examples |
| [SAFETY.md](docs/SAFETY.md) | Safety model, what the classes mean, MCP gate |
| [CLEANUP.md](docs/CLEANUP.md) | Desktop Cleanup sources and actions |
| [MACOS.md](docs/MACOS.md) | Signed macOS app bundle |
| [BROWSER.md](docs/BROWSER.md) | Browser companion: install, what it sends, tab actions |
| [BENCHMARKS.md](docs/BENCHMARKS.md) | With vs without: token tax, classify, live agents, MCP gate |
| [BENCHMARKS-LIVE.md](docs/BENCHMARKS-LIVE.md) | Live coding-agent check (2026-09-15) |
| [TESTING.md](docs/TESTING.md) | Test inventory and CI |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Crate map, license boundary, invariants |
| [FAQ.md](docs/FAQ.md) | Short answers |
| [product/](docs/product/README.md) | Product plan |
| [TERMS.md](TERMS.md) | Terms of Use |

## About

SweepLoom is built by Sergii Ziborov at
[Weavatrix](https://github.com/Weavatrix). Weavatrix is the **company**;
SweepLoom is the **product**.

- **Local-first.** Everything runs on your machine against local
  metadata. There is no account and no cloud service.
- **No telemetry.** No analytics, crash reports, or update checks. The
  only network use is the desktop app talking to a local Ollama server on
  `127.0.0.1` to list and remove models.
- **Deterministic.** No LLM on classify or apply.
- **Open source.** MPL-2.0, tested in CI on Windows, macOS, and Linux.

Bugs and feature requests:
[GitHub issues](https://github.com/Weavatrix/sweeploom/issues). For a
security problem, open an issue that asks for a private contact and leave
out exploit details and secrets.

## License

SweepLoom is **MPL-2.0**. [`LICENSE`](LICENSE).

`weavatrix-scan`, `weavatrix-git`, `mcport` remain **MIT**. Do not
relicense them here.

Use of the software and of [sweeploom.com](https://sweeploom.com) is also
covered by the [Terms of Use](TERMS.md).
