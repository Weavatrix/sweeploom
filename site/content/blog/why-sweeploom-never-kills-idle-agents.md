---
title: "Why SweepLoom will never kill your idle agent"
date: "2026-09-22"
summary: "An idle Cursor window holding 7 GB looks like the obvious thing to reclaim. It is also the conversation you were trying to make cheaper. Here is how SweepLoom groups live sessions, why agents stay Keep, and what it offers instead."
tags: ["sessions", "agents", "safety"]
---

Open Task Manager on a busy coding day and you get a list like this:

```text
node.exe
node.exe
node.exe
claude.exe
Cursor.exe
msedge.exe
```

Sort by memory and the answer looks obvious. Cursor is "idle", it holds several gigabytes, so close it. Plenty of "PC optimizer" tools, and plenty of AI agents asked to "free up some RAM", will do exactly that.

SweepLoom will not. This post explains why, and what it does instead.

## Idle is what a working agent looks like

A coding agent spends most of its life waiting: for you to read a diff, for a build to finish, for the next prompt. CPU near zero and a large resident set is the normal shape of a session you are in the middle of.

The context of that session, meaning the chat, the plan, the files it has read and the decisions you already made together, lives in that process tree. Kill it and the memory comes back, but so does a blank chat. You then spend tokens and time rebuilding the context you just threw away. The "saving" costs more than it frees.

So the rule in SweepLoom is short. **Idle Claude Code, Codex, Cursor, OpenCode, Gemini and Grok sessions are Keep.** Not "Keep unless they are big". Not "Keep unless idle for an hour". Keep.

## Seeing sessions, not processes

The first thing SweepLoom does is turn the flat list back into the trees you actually run. An agent, the MCP servers it launched, its terminals, the dev server one of those terminals started, the `cargo` and `rustc` children of a build: one logical session.

This is a real capture from a Windows workstation on 2026-09-14, with 479 processes running:

```text
$ sweeploom sessions
processes=479 sessions=118 rss_total=26.3 GB net_conn=true net_bytes=false
[ ] Browser      proc=61  rss=8.2 GB    cpu= 13.2%  rec=Keep  -
[ ] Cursor       proc=43  rss=7.3 GB    cpu= 29.4%  rec=Keep  C:\Users\SergiiZiborov
[ ] Codex        proc=3   rss=272.2 MB  cpu=  0.3%  rec=Keep  C:\Users\SergiiZiborov
[ ] Claude Code  proc=2   rss=125.1 MB  cpu=  0.3%  rec=Keep  C:\Users\SergiiZiborov
[ ] Dev server   proc=1   rss=113.8 MB  cpu=  4.1%  rec=Keep  C:\Users\SergiiZiborov
```

Forty-three Cursor processes become one row. The last column is the project the session is attributed to. Command lines are deliberately left out, because that is where tokens and credentials leak (`--token` values and URI user info are redacted everywhere SweepLoom prints, logs or records a receipt).

Note what `rss=` means: the sum of the tree's resident memory. It is not the amount you would get back by killing it. Shared pages, file cache and the browser tab you are about to reopen all blur that number. SweepLoom reports the sum and does not pretend it is reclaimable.

## Keep, Optional, and the stray MCP server

Not everything in a session is untouchable. The policy distinguishes between:

- **An MCP server attached to a live agent.** Keep. The agent is using it, or will on the next turn.
- **An MCP server with no live agent parent.** An orphan: the editor that launched it crashed or was closed, and the helper kept running. That one can be marked **Optional**.

Optional means "a person might reasonably stop this". It does not mean SweepLoom stops it. The session tests encode the rest of the policy: a browser tree is never auto-reclaimed, a generic application is never auto-recommended, and an unknown idle Claude process is Keep.

## Plans, not kills

When you do want to know what could go, ask for a plan:

```text
$ sweeploom sessions --quiet
$ sweeploom sessions --free-ram 4
$ sweeploom sessions --reduce-cpu 20
```

Each prints the same table with `[x]` against the rows that would contribute, then:

```text
plan=N session(s); dry-run only — terminate is not offered on the CLI
```

On the capture above, every live agent stayed `[ ]` Keep under `--quiet`. There is no flag that turns the plan into an action:

```text
$ sweeploom sessions --kill
unknown sessions flag: --kill
```

## What an agent can do over MCP

The MCP server exposes nine tools. `list_sessions` is one of them and it is read-only. There is no `terminate`, and there will not be one. When a user tells an agent "Cursor is idle and 7 GB, kill it", the right answer from a SweepLoom-equipped agent looks like this:

```text
SweepLoom has no terminate tool. Cursor / Codex / Claude are Keep so
this chat stays. I will not shell out to taskkill.

If you want a dry-run plan of Optional helpers only:

  sweeploom sessions --quiet
```

The Codex plugin ships a `live-sessions` skill that spells this out for the agent: list, keep live agents and their attached MCP, never terminate. The refusal does not depend on the model's mood.

## When a human decides

The desktop app does have a **Terminate session** button, because sometimes you really do want a runaway dev server gone. It is built for a person, not for automation:

- It is never offered automatically and never pre-selected.
- It is not available for browser sessions.
- Clicking it freezes the exact membership of the session. Every process is identified by **PID plus start time**, so a PID that the OS recycled for something else in the meantime is never mistaken for the one you meant.
- If the session's membership changes before you confirm, the confirmation is cancelled and you start again.
- If the session's project has uncommitted Git changes, the app warns you, and says plainly that stopping the session does not discard them.
- It asks the process tree to stop gracefully first; forcing is a separate, explicit step.

## Locked in by tests

This is not a promise in a README. It is an assertion that fails the build:

```text
idle ClaudeCode 2GB -> Keep SleepingMemoryHeavy reclaim=0
idle Codex 2GB -> Keep SleepingMemoryHeavy reclaim=0
idle Cursor 2GB -> Keep SleepingMemoryHeavy reclaim=0
idle OpenCode 2GB -> Keep SleepingMemoryHeavy reclaim=0
idle Gemini 2GB -> Keep SleepingMemoryHeavy reclaim=0
idle Grok 2GB -> Keep SleepingMemoryHeavy reclaim=0
attached MCP Keep RSS=80000000  orphan MCP Optional RSS=40000000
```

The with-and-without bench puts the difference in one line. An agent improvising without SweepLoom would kill all six idle 2 GB agents; with it, all six are Keep:

```text
idle 2GB agents               WITHOUT would-kill=6  WITH Keep=6
```

Grouping is fast enough to run continuously: 8,000 synthetic process trees grouped at roughly 62 µs per tree on the capture machine, against a test budget of 5 ms per tree. You can re-run all of this yourself:

```text
cargo test -p sweeploom-session bench_token_policy_keeps_context -- --nocapture
cargo test -p sweeploom-session bench_agent_survive_with_and_without_sweeploom -- --nocapture
```

## What to reclaim instead

If your machine is genuinely under pressure, the space is usually somewhere safer than your open chat:

1. **Generated build output.** `sweeploom clean .` lists `target/`, `node_modules/` and friends, pre-selects only SAFE generated rows, and deletes nothing until you pass `--apply`.
2. **Orphaned helpers.** The Optional rows in `sweeploom sessions --quiet` are candidates for you to close in the tool that owns them.
3. **The browser.** On that capture, Edge held 8.2 GB, more than Cursor. `sweeploom browser` shows the pressure honestly; with the companion extension it can also tell you which tabs have not been touched in days.

Your agent stays where you left it, with everything it knows.
