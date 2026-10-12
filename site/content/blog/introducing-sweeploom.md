---
title: "Introducing SweepLoom: reclaim your workstation without losing your workspace"
date: "2026-09-25"
summary: "SweepLoom is a local-first resource manager for developer machines. What it finds, the three questions it answers, who it is for, and how its CLI, MCP server and desktop app share one engine and one rule: nothing is deleted without plan, revalidate, execute and receipt."
tags: ["announcement", "overview", "cli", "mcp", "safety"]
---

Every developer machine fills up the same way. A `target/` folder per Rust project, a `node_modules/` per JavaScript one, a DerivedData folder per Xcode project you opened once, simulators for iOS versions you no longer support, Docker images from last quarter, a few local models, and a home folder full of agent history. Then the disk warning arrives, usually in the middle of something else.

The tools we reach for at that point were not built for this machine. Storage settings show "Developer" as one alarming number. A PC cleaner sees an idle editor holding 7 GB and offers to close it. A one-liner from a forum deletes five folders at once, two of which you wanted. And lately, an AI agent asked to "free some space" will happily improvise with `rm -rf`.

SweepLoom is the tool we wanted instead. It is a **local-first, developer-aware workstation resource manager**: it maps what is on disk, in RAM and inside your coding-agent hosts, tells you what is regenerable junk and what is still earning its keep, and reclaims space through a plan you review.

## Three questions

SweepLoom answers three questions, locally, with no LLM on the path that decides or deletes:

1. **What is sitting on disk, in RAM and in the agent host?**
2. **Is it still earning its keep?** A live chat, a project contract like `AGENTS.md`, or regenerable output?
3. **How do you reclaim it without losing the workspace?**

The second question is the one most tools skip. Task Manager shows a flat list of `node.exe` and `Cursor.exe`. A coding day is a tree:

```text
Cursor
  ├─ MCP (attached — Keep)
  ├─ terminal
  └─ vite

Codex
  └─ cargo / rustc

orphan mcp-server          ← no live agent parent; Optional

~/.claude/history.jsonl    ← archive, not prompt tax
~/.claude/AGENTS.md        ← always-on, Keep
app/target                 ← SAFE generated, rebuild is cheap
```

SweepLoom works on that tree. Agents are grouped with their MCP servers, terminals and dev servers. Build output is attributed to the project that owns it, with the rebuild cost next to the size. AI tool folders are split into what the host injects every turn and what is just history on disk.

## What it finds

| What | Examples | Where |
| --- | --- | --- |
| Generated build output | `target/`, `node_modules/`, Python generated output | CLI · MCP · App |
| Live sessions | Claude Code, Codex, Cursor, OpenCode, Gemini, Grok, their MCP servers, dev servers | CLI · MCP · App |
| Always-on prompt tax | `AGENTS.md`, `*.mdc`, `rules`, `skills`, `plugins` | CLI · MCP · App |
| Browser pressure | process trees and RSS; tab activity via an optional companion | CLI · MCP · App |
| Package and tool caches | npm, pnpm, Yarn, Bun, pip, uv, Cargo, Gradle, NuGet, Go, Homebrew, SwiftPM | App |
| Xcode and iOS Simulator | DerivedData by project, device support by version, devices and runtimes | App |
| Docker | images, stopped containers, unused volumes, old build cache | App |
| AI models | Ollama, Hugging Face, LM Studio, PyTorch, Whisper, Core ML | App |
| Archives and large data | Xcode archives, device backups, large downloads, logs | App |

The last column matters. Anything that is not regenerable output, or that belongs to another tool (Docker, `simctl`, Ollama, `rustup`), is handled only in the desktop app, where a person confirms each action and user data goes to the Trash rather than being deleted.

## One engine, three ways in

**The CLI is for hands.** Every command is read-only except one flag:

```text
$ sweeploom sessions             # what is alive; agents are Keep
$ sweeploom ai                   # AI stores and always-on token tax
$ sweeploom scan .               # what is on disk in this tree
$ sweeploom projects .           # heat, Git state, artifact offers
$ sweeploom clean .              # cleanup candidates, dry run
$ sweeploom clean . --apply      # delete pre-selected SAFE rows
```

**The MCP server is for agents.** Your host launches `sweeploom mcp` over stdio. Eight tools read; one, `apply_cleanup`, deletes, and only with `confirm: true` after the person agreed to specific ids. There is no terminate tool. A Codex plugin bundles the same server with skills that make review the default.

**The desktop app is for review.** A native Rust (egui) app for Windows, macOS and Linux that shows live pressure, every cleanup source with the consequence of removing each row, and the projects and AI stores behind them.

All three share the same classification and the same apply path, so an agent and a person looking at the same folder see the same rows.

## The rule: plan, revalidate, execute, receipt

Here is what cleanup looks like in a workspace that has built:

```text
$ sweeploom clean .
[x]  1.2 GB    cargo target
[x]  812.0 MB  node_modules
[ ]  12.0 KB   something inspect-only  BLOCKED
dry-run; pass --apply to delete pre-selected SAFE rows after revalidation

$ sweeploom clean . --apply
receipt=1  deleted=2  skipped_changed=0  failed=0
```

Four steps, on every surface:

1. **Plan.** Candidates come from metadata. Only SAFE generated rows can be pre-selected; inspect-only rows are blocked.
2. **Revalidate.** Right before removal, every row is checked again. If you rebuilt `target/` after the plan, that row is skipped, not forced.
3. **Execute.** Only on consent: `--apply` on the CLI, `confirm: true` from an agent, a confirmation in the app.
4. **Receipt.** What was deleted, skipped because it changed, or failed.

`sweeploom clean .` printing `no generated candidates` is a valid answer. SweepLoom does not invent junk to look useful.

## Who it is for

- **Developers with many repositories** whose disk is mostly rebuildable output they have lost track of.
- **iOS and macOS developers**, for whom `~/Library/Developer` is a mix of cache, native objects and irreplaceable archives.
- **People running coding agents**, who want to know what their agent hosts keep on disk and what is actually sent every turn.
- **People who want an agent to help clean up without trusting it to improvise.** The MCP server gives the agent evidence and a gate instead of a shell.

## What it is not

It is **not a RAM booster.** Idle Claude Code, Codex, Cursor, OpenCode, Gemini and Grok sessions are Keep. Killing a live agent frees memory and loses the conversation you were in.

It is **not a context compressor.** It estimates the always-on prompt tax of rules, skills and plugins; it does not shrink what an agent reads. A live coding-agent run on 2026-09-15 found **no** coding-task token saving with the MCP attached (+75.5% estimated spend), and that result is published next to the fixtures.

It is **not a cloud service.** No account, no telemetry, no update checks. The CLI and MCP server make no network requests; the desktop app's only network use is a local Ollama server on `127.0.0.1`.

## Try it

SweepLoom installs from source today. Packages for crates.io and npm are prepared but not published, and there are no prebuilt releases. With Rust 1.88 or newer:

```text
$ cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom
$ sweeploom clean .
```

The desktop app needs Rust 1.95 or newer and builds from a clone. Everything is open source under MPL-2.0 and tested in CI on Windows, macOS and Linux. Read the [docs](/docs/), connect an agent with the [MCP guide](/mcp/), and [open an issue](https://github.com/Weavatrix/sweeploom/issues) when it gets something wrong.
