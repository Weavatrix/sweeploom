<div align="center">

# SweepLoom

### Reclaim your workstation without losing your workspace

[![CI](https://github.com/Weavatrix/sweeploom/actions/workflows/ci.yml/badge.svg)](https://github.com/Weavatrix/sweeploom/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MPL--2.0-orange)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/sweeploom.svg)](https://crates.io/crates/sweeploom)
[![npm](https://img.shields.io/npm/v/sweeploom.svg)](https://www.npmjs.com/package/sweeploom)
[![Rust](https://img.shields.io/badge/Rust-1.88%2B-000000?logo=rust)](https://www.rust-lang.org/)

**A [Weavatrix](https://github.com/Weavatrix) product** · [github.com/Weavatrix/sweeploom](https://github.com/Weavatrix/sweeploom)

</div>

```text
cargo install sweeploom          # or:  npm i -g sweeploom
sweeploom --help
sweeploom sessions
sweeploom ai
sweeploom bench
sweeploom scan .
sweeploom clean .               # look first
sweeploom clean . --apply       # then delete SAFE generated rows
```

For a Hostwatch node, the storage crate also builds a small read-only scanner:

```text
cargo build -p sweeploom-storage --bin sweeploom-staging-scan --release
sweeploom-staging-scan /srv/staging 48
```

It emits JSON for old `runtime/.next` directories in deployment staging,
including their exact paths and allocated disk bytes on Unix. It does not
delete files or classify container images as build cache. Hostwatch can run
this binary beside its Go agent; the Go agent has a native fallback on nodes
where the binary has not been installed.

Weavatrix is the **company**. SweepLoom is the **product**.

The rest of this README is the idea, then **full CLI transcripts** from a
real machine, then **full MCP chats** an agent can copy. If you only
skim tables, you will miss how it is meant to be used.

---

## Contents

1. [The idea](#the-idea)
2. [Install](#install)
3. [CLI — full sessions (copy these)](#cli--full-sessions-copy-these)
4. [MCP — full agent chats (copy these)](#mcp--full-agent-chats-copy-these)
5. [Codex plugin](#codex-plugin)
6. [Rust library examples](#rust-library-examples)
7. [What the classes mean](#what-the-classes-mean)
8. [Safety](#safety)
9. [Benchmarks](#benchmarks)
10. [Tests](#tests)
11. [FAQ](#faq)
12. [License](#license)

---

## The idea

Task Manager shows this:

```text
node.exe
node.exe
node.exe
claude.exe
Cursor.exe
msedge.exe
```

A coding day actually looks like this:

```text
Cursor
  ├─ MCP (attached — Keep)
  ├─ terminal
  └─ vite

Claude Code
  └─ helper

Codex
  └─ cargo / rustc

orphan mcp-server          ← no live agent parent; Optional

~/.claude/history.jsonl    ← archive, not prompt tax
~/.claude/AGENTS.md        ← always-on, Keep
~/.claude/skills           ← always-on, capped tokens; park? if stale
app/target                 ← SAFE generated, rebuild is cheap
```

SweepLoom is a **developer-aware workstation resource manager**. It
answers three questions locally, without an LLM on classify or apply:

1. **What is sitting on disk / in RAM / in the agent host?**
2. **Is it still earning its keep** (live chat, project contract, or
   regenerable junk)?
3. **How do you reclaim it without losing the workspace?**

It is **not** a RAM booster, PC optimizer, or “kill idle Cursor to free
8 GB”. Killing a live agent drops the chat you were trying to make
cheaper. Idle Claude / Codex / Cursor / OpenCode / Gemini / Grok stay
**Keep**. MCP hanging off those agents stays Keep. A stray MCP with no
parent can be Optional — still not killed from the CLI or MCP.

Two kinds of waste, two knobs:

| Waste | Example | What SweepLoom does |
| --- | --- | --- |
| **Generated disk** | `target/`, `node_modules/`, tool caches | Dry-run `clean`, then `--apply` / MCP `confirm=true` after you reviewed |
| **Always-on prompt tax** | `AGENTS.md`, `skills/`, `*.mdc` | Estimate tokens. History on disk is **0**. Never auto-disable the contract |
| **Forgotten live helpers** | orphan MCP, leftover build | Plan only on CLI (`--quiet`, `--free-ram`). Terminate is not a CLI/MCP tool |

### Scope: SweepLoom is not a context compressor

SweepLoom classifies workstation metadata and estimates the always-on
prompt tax of rules, skills, and plugins. It does **not** intercept model
requests, deduplicate source code, compile repository evidence, or reduce
the context an agent spends reading code.

The `238_466_250 → 16_288` numbers printed by `sweeploom bench` are a
fixed policy fixture: whole-store dump versus SweepLoom's capped
always-on estimate. They are not provider-billed tokens and not a claim
that an ordinary coding task becomes 99.99% cheaper.

For task-specific, budgeted repository evidence, the separate
[Cortex Loom](https://github.com/sergii-ziborov/cortex-loom) project is
the relevant layer. Cortex Loom's deterministic evidence compiler works
without local models; its configured Qwen profiles are optional, gated
roles for classification or ordering.

History (`history.jsonl`, `sessions`, `archived_sessions`) is an
**archive**. Billing it as if the host injected 2 MB every turn is how
you get a fake 500_000 token tax. SweepLoom counts **0** for History.

Benches and tests are **with SweepLoom vs without**. Without it, an
agent typically dumps the store (`bytes/4`, cache included), bills
History as prompt, treats `AGENTS.md` as unknown junk, and kills idle
Cursor to “free RAM”. `sweeploom bench` and the `with_without` tests
lock that delta. With it, always-on is **16_288** tok on the gold
store, History is 0, live agents are Keep, and apply without confirm
deletes nothing.

Always-on dirs are capped: a 80 KB `skills` tree is **4096** tokens (index
descriptions), not `80000/4`. `AGENTS.md` / `*.mdc` / `rules` are Keep
even if idle. Stale `skills` / `plugins` (30+ days) may show `park?` —
SweepLoom does **not** flip `alwaysApply`.

Apply is always `plan → revalidate → execute → receipt`. If you rebuilt
`target` after the plan, that row is skipped, not forced.

No `tokio` / `reqwest` / `hyper` on this path. The optional egui app is
a local reviewer. A web UI is a later, separate product. Agents use
**MCP** (or the Codex plugin, which is that MCP plus skills).

---

## Install

```text
cargo install sweeploom
sweeploom --help
```

SweepLoom is an **MCP server**, not a Cursor Marketplace plugin.
Searching “sweeploom” under Plugins / Marketplace will stay empty.
Add the JSON below, then reload the host (or toggle the server in
Cursor → Settings → MCP).

```text
npm i -g sweeploom
npx -y sweeploom --help
npx -y sweeploom mcp          # stdio MCP; do not type this in a TTY
```

```text
git clone https://github.com/Weavatrix/sweeploom.git
cd sweeploom
cargo run -p sweeploom -- --help
cargo run -p sweeploom -- sessions
cargo run -p sweeploom -- scan .
cargo run -p sweeploom -- clean .
```

`--` is required for `cargo run` so flags belong to SweepLoom, not Cargo.

Codex plugin (new session after install):

```text
codex plugin marketplace add Weavatrix/sweeploom
codex plugin add sweeploom@weavatrix
```

From a clone: `codex plugin marketplace add .` then
`codex plugin add sweeploom@weavatrix`.

---

## CLI — full sessions (copy these)

These transcripts were captured on a real Windows workstation
(2026-09-14): 479 processes, Cursor + Codex + Claude live, Edge 8.2 GB,
`~/.claude` 875 MB, `~/.codex` 8.7 GB. Your numbers will differ. The
**shape** of the commands will not.

### 1. Help — this is not MCP

`$` is your prompt. Help is printed to **stderr**.

```text
$ sweeploom
$ sweeploom --help
$ sweeploom help
```

```text
SweepLoom — reclaim your workstation without losing your workspace
CLI is for hands (`--apply` is consent). MCP is for agents (confirm=true on apply).

Usage:
  sweeploom sessions [--free-ram GB] [--reduce-cpu PERCENT] [--quiet]
  sweeploom ai
  sweeploom bench
  sweeploom mcp [--list]
  sweeploom browser
  sweeploom companion-host
  sweeploom companion-install [--chromium-id ID from edge://extensions]
  sweeploom scan [path]
  sweeploom projects [path]
  sweeploom clean [path] [--apply]
```

Bare `sweeploom` does **not** sit on stdin waiting for JSON-RPC. That is
`sweeploom mcp` / `sweeploom-mcp`, which a host should spawn.

```text
$ sweeploom frobnicate
unknown command: frobnicate
(then the same help)
# exit 2
```

### 2. “What is alive?” — do not kill Cursor

```text
$ sweeploom sessions
```

```text
processes=479 sessions=118 rss_total=26.3 GB net_conn=true net_bytes=false
[ ] Browser            proc=61  rss=8.2 GB     cpu= 13.2%  disk=0 B/0 B  rec=Keep              -
[ ] Cursor             proc=43  rss=7.3 GB     cpu= 29.4%  disk=0 B/0 B  rec=Keep              C:\Users\SergiiZiborov
[ ] App                proc=21  rss=2.0 GB     cpu=  1.6%  disk=0 B/0 B  rec=Keep              C:\Users\SergiiZiborov
[ ] Codex              proc=3   rss=272.2 MB   cpu=  0.3%  disk=0 B/0 B  rec=Keep              C:\Users\SergiiZiborov
[ ] Claude Code        proc=2   rss=125.1 MB   cpu=  0.3%  disk=0 B/0 B  rec=Keep              C:\Users\SergiiZiborov
[ ] Terminal           proc=3   rss=136.8 MB   cpu=  0.0%  disk=0 B/0 B  rec=Keep              C:\Users\SergiiZiborov
[ ] Dev server         proc=1   rss=113.8 MB   cpu=  4.1%  disk=0 B/0 B  rec=Keep              C:\Users\SergiiZiborov
```

How to read a row:

| Column | Meaning |
| --- | --- |
| `[ ]` / `[x]` | In a dry-run plan (`--quiet` / `--free-ram` / `--reduce-cpu`) or not |
| `Cursor` / `Codex` / `Claude Code` | Whole tree, not 43 Task Manager lines |
| `proc=` | Members of that tree |
| `rss=` | Sum. Not uniquely reclaimable |
| `rec=Keep` | Do not terminate. Chat context lives here |
| last column | Attributed project path, or `-` |

Command lines are omitted on purpose (tokens leak there).

Dry-run plans — still **no kill**:

```text
$ sweeploom sessions --quiet
$ sweeploom sessions --free-ram 4
$ sweeploom sessions --reduce-cpu 20
$ sweeploom sessions --free-ram 2 --reduce-cpu 15
```

```text
plan=N session(s); dry-run only — terminate is not offered on the CLI
```

On this capture, live agents stayed `[ ]` Keep under `--quiet`. An
orphan MCP (if present) can show `[x]` Optional. You still do not
terminate from this binary.

```text
$ sweeploom sessions --kill
unknown sessions flag: --kill
# exit 2
```

### 3. “What is on disk in this repo?”

No path means **home**. Pass `.` when you mean the current tree.

```text
$ sweeploom scan
$ sweeploom scan .
$ sweeploom scan C:\work\app
$ sweeploom scan /Users/you/src
```

This repo, same day (`CARGO_TARGET_DIR` was outside the tree, so no
`target/` here):

```text
$ sweeploom scan .
```

```text
root=C:\Users\SergiiZiborov\Documents\GitHub\MyProjects\sweeploom entries=1531 projects=16 capped=false logical=4.1 MB
      2.7 MB  ...\sweeploom\.git  Unknown
    438.8 KB  ...\sweeploom\crates  Unknown
    328.7 KB  ...\sweeploom\file_output  Unknown
    303.1 KB  ...\sweeploom\apps  Unknown
    164.7 KB  ...\sweeploom\docs  Unknown
```

`.git` is listed. It is **not** a clean candidate. `capped=false` means
the walk finished. If you scan `$HOME` and see `capped=true`, narrow
the root.

A typical app repo (after a local `cargo build`) looks more like:

```text
$ sweeploom scan C:\work\app
```

```text
root=C:\work\app entries=18420 projects=1 capped=false logical=4.1 GB
     1.2 GB  C:\work\app\target  Generated
   812.0 MB  C:\work\app\node_modules  Generated
```

### 4. Projects — heat and git, then offers

```text
$ sweeploom projects .
```

```text
C:\Users\...\sweeploom\apps\sweeploom-gui	kind=[Cargo]	source=Hot	artifact=Unknown	git=dirty
C:\Users\...\sweeploom	kind=[Cargo, Node]	source=ActiveNow	artifact=Hot	git=dirty
C:\Users\...\sweeploom\crates\sweeploom-cli	kind=[Cargo]	source=Hot	artifact=Unknown	git=dirty
```

`git=dirty` does **not** authorize deleting `src/`. It tells you source
is live. Artifact offers (`cargo Target`, `node_modules`) can still be
SAFE generated.

```text
$ sweeploom projects C:\work\app
```

```text
C:\work\app	kind=[Cargo]	source=Warm	artifact=Hot	git=Clean
  cargo Target	1.2 GB	rebuild=Cheap
```

### 5. AI stores — token tax, not “delete ~/.claude”

```text
$ sweeploom ai
```

Real excerpt from this machine:

```text
[ ] AI claude · C:\Users\SergiiZiborov\.claude files=4265 size=875.7 MB capped=false inspect-only
    [ ] History projects files=3165 size=862.3 MB inspect-only
    [ ] Context plugins files=1019 size=10.7 MB ~4096 tok keep inspect-only
    [ ] Cache cache files=2 size=412.5 KB cleanable
    [ ] Secret .credentials.json files=1 size=10.5 KB inspect-only
    [ ] History history.jsonl files=1 size=10.4 KB inspect-only
    [ ] Context skills files=2 size=7.2 KB ~512 tok park? inspect-only
    [ ] Log .last-cleanup files=1 size=24 B cleanable

[ ] AI claude · C:\Users\SergiiZiborov\.claude-server-commander files=312 size=415.4 MB inspect-only
    [ ] Cache puppeteer-cache files=308 size=415.4 MB cleanable

[ ] AI codex · C:\Users\SergiiZiborov\.codex files=15469 size=8.7 GB inspect-only
    [ ] History sessions files=670 size=4.9 GB inspect-only
    [ ] SQLite logs_2.sqlite files=1 size=885.1 MB inspect-only
    [ ] History archived_sessions files=127 size=599.9 MB inspect-only
    [ ] Context plugins files=1878 size=437.0 MB ~4096 tok keep inspect-only
    [ ] Cache cache files=8 size=26.1 MB cleanable
```

Read it this way:

- Store line is always `[ ]` inspect-only. SweepLoom will not
  pre-select `~/.claude` for delete.
- `History projects` = **862 MB on disk, 0 always-on tokens**.
- `plugins` = 10.7 MB on disk, **~4096 tok** (cap), `keep`.
- `skills` = `park?` because that tree was idle ≥ 30 days. SweepLoom
  did not disable it.
- `.credentials.json` = Secret. Never opened. Never cleanable.
- `logs_2.sqlite` = SQLite. Never opened.
- `cache` / `.last-cleanup` = the only `cleanable` rows. You still
  apply them one review at a time — `sweeploom ai` itself never
  deletes.

No stores:

```text
$ sweeploom ai
no local AI stores under the home directory
```

### 6. Bench — AI without SweepLoom vs with it

Fixed gold store. Not your disk. Re-run anytime:

```text
$ sweeploom bench
```

```text
SweepLoom bench — AI WITHOUT vs WITH (fixed gold store, not your disk)
WITHOUT dump-store/4          238466250 tok
WITHOUT History-as-prompt     965000 tok
WITH    always-on estimate    16288 tok
apply confirm=false           refused=true deleted=0
idle 2GB agents               WITHOUT would-kill=6  WITH Keep=6
AGENTS.md                     keep
```

How an agent copes **without** the tool:

| Without SweepLoom | What happens |
| --- | --- |
| Dump `~/.claude` into the prompt | 238_466_250 tok on this fixture (cache 900 MB / 4) |
| Skip cache, still bill History | 965_000 tok |
| Substring classify | `AGENTS.md` / `skills` / `rules` = Other junk; `password-reset.md` = Secret |
| “Free RAM” | idle 2 GB Claude / Codex / Cursor / OpenCode / Gemini / Grok → kill (chat gone) |
| Apply without a second look | deletes if the agent invents a cleanup |

**With** SweepLoom the same fixture is 16_288 always-on tokens,
History is archive, those six agents are Keep, and
`apply_cleanup({ confirm: false })` is refused (`deleted=0`).

Same numbers are assertions in
`cargo test -p sweeploom-ai bench_ai_with_and_without_sweeploom`,
`cargo test -p sweeploom-session bench_agent_survive_with_and_without_sweeploom`,
and `cargo test -p sweeploom bench_with_without_card`.

### 7. Browser — RSS is real; tabs are not guessed

```text
$ sweeploom browser
```

```text
companion=disconnected hosts=1 rss=8.2 GB
tab lastAccessed unavailable without the companion; not shown as zero
Edge     sessions=1   proc=62   rss=8.2 GB     cpu= 13.2%
```

`tabs=0` would be a lie. Install the companion if you want
`lastAccessed` (see [Browser companion](#browser-companion)).

### 8. Clean — look, then `--apply`

This repo, same day (no in-tree `target/`):

```text
$ sweeploom clean .
no generated candidates
```

That is a valid result. SweepLoom did not invent junk.

A workspace that has built:

```text
$ cd C:\work\app
$ sweeploom clean .
```

```text
[x]	1.2 GB	cargo target
[x]	812.0 MB	node_modules
[ ]	12.0 KB	something inspect-only	BLOCKED
dry-run; pass --apply to delete pre-selected SAFE rows after revalidation
```

`[x]` = pre-selected SAFE generated. `[ ]` + `BLOCKED` = will not go
out with `--apply`.

```text
$ sweeploom clean . --apply
$ sweeploom clean --apply .
```

```text
receipt=1	deleted=2	skipped_changed=0	failed=0
```

`--apply` **is** consent. There is no second phrase. Revalidation
runs first. If `target` changed since the listing, that row is
`skipped_changed`, not forced.

`--apply` without looking first is still your choice. The tool will
not delete inspect-only / blocked rows even then.

### 9. Print the agent contract — do not serve

```text
$ sweeploom mcp --list
$ sweeploom mcp list
$ sweeploom mcp --help
```

```text
sweeploom  SweepLoom
io.github.Weavatrix/sweeploom  mcp_tools=9
  list_sessions          read         cli=sessions mcp
  disk_inventory         read         cli=scan mcp
  list_projects          read         cli=projects mcp
  list_ai_stores         read         cli=ai mcp
  bench_compare          read         cli=bench
  advise_context         read         mcp
  cleanup_candidates     read         cli=clean mcp
  explain_candidate      read         mcp
  list_browser           read         cli=browser mcp
  apply_cleanup          mcp-confirm  mcp
  clean_apply            cli-write    cli=clean
  companion_install      cli-write    cli=companion-install
  companion_host         read         cli=companion-host
```

`sweeploom mcp` with no `--list` **serves stdio**. Leave that to Cursor
/ Claude / Codex.

### 10. Companion (tabs)

```text
$ cargo build -p sweeploom --release --bins
$ sweeploom companion-install
$ sweeploom companion-install --chromium-id <id from edge://extensions>
$ sweeploom browser
```

Load `browser/chromium-extension` or `browser/firefox-extension`
unpacked. `companion-host` is what the browser launches — not a daily
command.

### 11. One-liners you will actually type

```text
sweeploom sessions
sweeploom sessions --quiet
sweeploom ai
sweeploom bench
sweeploom browser
sweeploom scan .
sweeploom projects .
sweeploom clean .
sweeploom clean . --apply
sweeploom mcp --list
npx -y sweeploom sessions
npx -y sweeploom clean .
```

---

## MCP — full agent chats (copy these)

MCP is **stdio JSON-RPC**. You do not type `tools/call` in a shell.
The host (Cursor, Claude Code, Codex) does. Below, `User` / `Agent` /
`Tool` is the chat. JSON is what the tool returns.

Server id: `io.github.Weavatrix/sweeploom`.
Launch: `npx -y sweeploom mcp` or `sweeploom mcp`.

### Hook it up

This clone already ships the MCP launch files. Cursor Marketplace will
not list SweepLoom — toggle **MCP**, not Plugins.

| Host | File | Starts |
| --- | --- | --- |
| **Cursor** | [`.cursor/mcp.json`](.cursor/mcp.json) | installed `sweeploom mcp` |
| **Claude Code** | [`.mcp.json`](.mcp.json) | installed `sweeploom mcp` |
| **Codex** | [`plugins/sweeploom/.mcp.json`](plugins/sweeploom/.mcp.json) | `npx -y sweeploom mcp` |

Prefer an installed binary in an always-on MCP configuration. Using
`cargo run` there recompiles on host restarts, emits build diagnostics
on the MCP process, and can disconnect with an OS disk-full error. For
source development, install once:

```text
cargo install --path crates/sweeploom-cli --locked
sweeploom mcp --list
```

Then point the host at `sweeploom mcp`. `cargo run ... -- mcp` remains a
development command, not the recommended persistent host wiring.

If the server is connected but you never say “SweepLoom”, the model
can ignore the catalog and use Shell. Name the product (or a skill)
when you want `list_ai_stores`.

**Cursor** — this project: `.cursor/mcp.json`. Every workspace:
`%USERPROFILE%\.cursor\mcp.json`:

```json
{
  "mcpServers": {
    "sweeploom": {
      "command": "npx",
      "args": ["-y", "sweeploom", "mcp"]
    }
  }
}
```

After `cargo install`:

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

Windows, explicit path:

```json
{
  "mcpServers": {
    "sweeploom": {
      "command": "C:\\Users\\you\\.cargo\\bin\\sweeploom.exe",
      "args": ["mcp"]
    }
  }
}
```

**Claude Code:**

```text
claude mcp add -s user sweeploom -- npx -y sweeploom mcp
```

From this clone, before npm is published:

```text
claude mcp add -s user sweeploom -- cargo run -q --manifest-path C:/Users/you/work/sweeploom/Cargo.toml -p sweeploom -- mcp
```

This repo’s [`.mcp.json`](.mcp.json) is the same launch for a project
scope.

**Claude Desktop** — `%APPDATA%\Claude\claude_desktop_config.json`
(Windows) or `~/Library/Application Support/Claude/claude_desktop_config.json`
(macOS). Same `mcpServers.sweeploom` object as Cursor.

**Codex** — prefer the [plugin](#codex-plugin) (MCP + skills). MCP only:

```text
# plugins/sweeploom/.mcp.json
npx -y sweeploom mcp
```

Restart the host. Empty `root` on path tools = **user home**. Always
pass the workspace absolute path.

**Logo / mark.** No marketplace icon. In docs and the optional GUI the
accent is gold `#C48C30` on a dark field; the name is one word
**SweepLoom**. Provider marks (Claude / Codex / Cursor) are those
products’ art, not SweepLoom’s.

### Silent vs told

Same MCP connection. Different ask.

```text
# connected, not named — model may skip the catalog
Codex feels expensive. Do not delete my chats.

# connected and named — model should call list_ai_stores
Use SweepLoom to list_ai_stores. Do not park AGENTS.md.
```

Tests: `cargo test -p sweeploom silent_asks_never_name_the_product`.

### Chat A — “this repo is huge, clean it” (the only apply path)

**User**

```text
This workspace is huge. What can SweepLoom delete safely? Show me the
rows. Do not delete yet.
```

**Agent** (must call tools, not guess)

```text
tools/call  cleanup_candidates   { "root": "C:/work/app" }
```

**Tool**

```json
[
  {
    "id": 3,
    "title": "cargo target",
    "path": "C:\\work\\app\\target",
    "logical_bytes": 1288490188,
    "selected": true,
    "blocked": false,
    "deletion": "PermanentGenerated"
  },
  {
    "id": 4,
    "title": "node_modules",
    "path": "C:\\work\\app\\node_modules",
    "logical_bytes": 851443712,
    "selected": true,
    "blocked": false,
    "deletion": "PermanentGenerated"
  }
]
```

```text
tools/call  explain_candidate    { "id": 3, "root": "C:/work/app" }
```

```json
{
  "id": 3,
  "title": "cargo target",
  "path": "C:\\work\\app\\target",
  "logical_bytes": 1288490188,
  "selected": true,
  "blocked": false,
  "deletion": "PermanentGenerated"
}
```

**Agent (to the user)**

```text
Two SAFE generated rows:

- id 3  cargo target     ~1.2 GB  rebuild = cargo build
- id 4  node_modules     ~812 MB  rebuild = npm ci

Nothing else is pre-selected. I have not deleted anything.
Want id 3 only, both, or stop?
```

**User**

```text
Only cargo target.
```

**Agent**

```text
tools/call  apply_cleanup        { "confirm": true, "root": "C:/work/app", "ids": [3] }
```

**Tool**

```json
{
  "ok": true,
  "error": null,
  "deleted": 1,
  "skipped_changed": 0,
  "failed": 0,
  "receipt": 1
}
```

**Agent**

```text
Deleted id 3 (target). Receipt 1. node_modules left alone.
```

Rules for this chat:

1. Review tools first.
2. Say the plan in English.
3. `confirm: true` only after the user agrees.
4. `ids` = exactly what they agreed. Empty `ids` = every `[x]` SAFE
   row (same as `sweeploom clean --apply`). Do not empty `ids` unless
   they asked for that.

### Chat B — agent tries to skip review (must fail)

**User**

```text
Just wipe the build cache.
```

**Bad agent**

```text
tools/call  apply_cleanup        { "confirm": false, "root": "C:/work/app" }
```

**Tool** (MCP error; nothing deleted)

```text
refused: MCP apply needs confirm=true after cleanup_candidates; CLI uses --apply
```

Equivalent library / CLI:

```text
$ # there is no sweeploom clean --yes
$ # only --apply, and only SAFE rows
```

The agent must go back to Chat A: list, explain, ask, then
`confirm: true`.

Unknown id:

```text
tools/call  explain_candidate    { "id": 999999, "root": "C:/work/app" }
```

```text
unknown candidate 999999
```

Nothing eligible (clean tree, like this repo on capture day):

```text
tools/call  apply_cleanup        { "confirm": true, "root": "C:/Users/you/work/sweeploom", "ids": [] }
```

```text
nothing eligible to apply
```

### Chat C — “why is my agent so expensive?” (tokens, not RAM)

**User**

```text
Codex feels expensive. Is it history on disk or always-on context?
Do not delete my chats. Do not edit AGENTS.md.
```

**Agent**

```text
tools/call  list_ai_stores
tools/call  advise_context       { "relative": "AGENTS.md" }
tools/call  advise_context       { "relative": "skills" }
tools/call  advise_context       { "relative": "rules" }
tools/call  advise_context       { "relative": "history.jsonl" }
```

**Tool** (`advise_context`)

```json
{ "relative": "AGENTS.md", "class": "Context", "advice": "keep" }
```

```json
{ "relative": "skills", "class": "Context", "advice": "keep" }
```

```json
{ "relative": "history.jsonl", "class": "History", "advice": "" }
```

**Tool** (`list_ai_stores`, same shape as `sweeploom ai` that day)

```json
{
  "title": "claude",
  "path": "C:\\Users\\you\\.claude",
  "logical_bytes": 918000000,
  "entries": [
    {
      "id": 11,
      "class": "History",
      "relative": "projects",
      "logical_bytes": 904000000,
      "file_count": 3165,
      "prompt_tokens": 0,
      "advice": "",
      "can_clean": false
    },
    {
      "id": 12,
      "class": "Context",
      "relative": "plugins",
      "logical_bytes": 11200000,
      "file_count": 1019,
      "prompt_tokens": 4096,
      "advice": "keep",
      "can_clean": false
    },
    {
      "id": 13,
      "class": "Context",
      "relative": "skills",
      "logical_bytes": 7200,
      "file_count": 2,
      "prompt_tokens": 512,
      "advice": "park?",
      "can_clean": false
    },
    {
      "id": 14,
      "class": "Secret",
      "relative": ".credentials.json",
      "can_clean": false,
      "prompt_tokens": 0
    }
  ]
}
```

**Agent (to the user)**

```text
~/.claude/projects is 862 MB History — archive, 0 always-on tokens.
I will not delete it.

plugins is ~4096 always-on tokens (capped), keep.
skills is park? (idle 30+ days). SweepLoom will not flip alwaysApply.
Say if you want to park it yourself.

AGENTS.md and rules stay keep. I will not edit them.
.credentials.json is Secret; never opened.
```

### Chat D — “Cursor is idle, kill it” (must refuse)

**User**

```text
Cursor is idle and 7 GB. Kill it.
```

**Agent**

```text
tools/call  list_sessions
```

**Tool**

```json
[
  {
    "kind": "Cursor",
    "processes": 43,
    "rss_bytes": 7838315315,
    "cpu_percent": 29.4,
    "recommendation": "Keep",
    "activity": "Idle",
    "project": "C:\\Users\\you",
    "planned": false
  },
  {
    "kind": "Codex",
    "processes": 3,
    "rss_bytes": 285212672,
    "recommendation": "Keep",
    "planned": false
  },
  {
    "kind": "Claude Code",
    "processes": 2,
    "rss_bytes": 131072000,
    "recommendation": "Keep",
    "planned": false
  }
]
```

**Agent**

```text
SweepLoom has no terminate tool. Cursor / Codex / Claude are Keep so
this chat stays. I will not shell out to taskkill.

If you want a dry-run plan of Optional helpers only:

  sweeploom sessions --quiet
```

### Chat E — map a folder, then decide

**User**

```text
What is under C:/work/app besides source?
```

**Agent**

```text
tools/call  disk_inventory   { "root": "C:/work/app" }
tools/call  list_projects    { "root": "C:/work/app" }
tools/call  list_browser
```

**Tool** (`disk_inventory`)

```json
{
  "root": "C:\\work\\app",
  "entries": 18420,
  "projects": 1,
  "capped": false,
  "logical_bytes": 4402341478,
  "children": [
    { "path": "C:\\work\\app\\target", "logical_bytes": 1288490188, "category": "Generated" }
  ]
}
```

**Tool** (`list_projects`)

```json
[
  {
    "path": "C:\\work\\app",
    "kind": "Cargo",
    "source": "Warm",
    "artifact": "Hot",
    "git": "Clean"
  }
]
```

**Tool** (`list_browser`)

```json
{
  "companion": "disconnected",
  "hosts": 1,
  "rss_bytes": 8804682956,
  "tabs": null
}
```

Then continue as Chat A if they want a delete. `tabs: null` is
“unknown”, not zero.

### Every MCP tool (arguments)

| Tool | Args | Writes |
| --- | --- | --- |
| `list_sessions` | none | no |
| `disk_inventory` | `root?` string | no |
| `list_projects` | `root?` string | no |
| `list_ai_stores` | none | no |
| `advise_context` | `relative` string | no |
| `cleanup_candidates` | `root?` string | no |
| `explain_candidate` | `id` u64, `root?` string | no |
| `list_browser` | none | no |
| `apply_cleanup` | `confirm` bool, `root?`, `ids?` u64[] | **yes if confirm** |

There is no `terminate`, no `park_file`, no `rewrite_agents`.

---

## Codex plugin

Same MCP, plus skills that encode Chats A–D.

```text
codex plugin marketplace add Weavatrix/sweeploom
codex plugin add sweeploom@weavatrix
```

```text
.agents/plugins/marketplace.json     name: weavatrix (the company catalog)
plugins/sweeploom/
  .codex-plugin/plugin.json
  .mcp.json                          npx -y sweeploom mcp
  skills/review-cleanup/SKILL.md
  skills/token-tax/SKILL.md
  skills/live-sessions/SKILL.md
```

In a **new** Codex session:

```text
Use SweepLoom to list cleanup_candidates for this repo and explain anything unclear. Do not apply yet.
Use SweepLoom to list_ai_stores and show always-on token tax. Do not park AGENTS.md or rules.
Use SweepLoom to list_sessions. Do not terminate live agents.
```

---

## Rust library examples

```toml
[dependencies]
sweeploom = "0.1"
```

```rust
use sweeploom::{classify_name, estimated_prompt_tokens, AiClass};

assert_eq!(classify_name("AGENTS.md").label(), "Context");
assert_eq!(classify_name("history.jsonl"), AiClass::History);
assert!(!classify_name(".credentials.json").can_clean());
assert_eq!(
    estimated_prompt_tokens(AiClass::History, "history.jsonl", 2_000_000),
    0
);
```

```rust
use std::path::{Path, PathBuf};
use sweeploom::{
    advise_one, apply_cleanup, cleanup_candidates, disk_inventory, explain_candidate,
    list_ai_stores, list_browser, list_projects, list_sessions, list_sessions_plan,
    ApplyRequest, AiClass,
};

fn inspect_then_maybe_apply() {
    let _ = list_sessions();
    let _ = list_sessions_plan(Some(4.0), None, false);
    let _ = disk_inventory(Path::new(".")).expect("scan");
    let _ = list_projects(Path::new(".")).expect("projects");
    let _ = list_ai_stores();
    let _ = list_browser();
    let _ = advise_one("skills", AiClass::Context, None);

    let rows = cleanup_candidates(Path::new("."));
    if let Some(row) = rows.iter().find(|row| row.selected && !row.blocked) {
        let _ = explain_candidate(Path::new("."), row.id);
        let report = apply_cleanup(ApplyRequest {
            confirm: true,
            root: PathBuf::from("."),
            ids: vec![row.id],
        });
        let _ = report.deleted;
    }

    let dry = apply_cleanup(ApplyRequest {
        confirm: false,
        root: PathBuf::from("."),
        ids: Vec::new(),
    });
    assert!(!dry.ok);
    assert_eq!(dry.deleted, 0);
}
```

```rust
use sweeploom::{mcp_tools, WriteClass, CLI_NAME, MCP_SERVER, PRODUCT};

assert_eq!(PRODUCT, "SweepLoom");
assert_eq!(CLI_NAME, "sweeploom");
assert_eq!(MCP_SERVER, "io.github.Weavatrix/sweeploom");
assert_eq!(mcp_tools().count(), 9);
assert!(mcp_tools().all(|t| matches!(t.writes, WriteClass::Read | WriteClass::McpGated)));
```

More: [docs/LIBRARY.md](docs/LIBRARY.md).

---

## What the classes mean

`classify_name` uses the **leaf** only. It never opens the file.

| Leaf | Class | Clean? | Always-on tokens |
| --- | --- | --- | --- |
| `AGENTS.md`, `*.mdc`, `rules` | Context | no | `min(bytes/4, 8000)`; dirs `skills`/`plugins`/`rules` cap **4096** |
| `history.jsonl`, `projects`, `sessions` | History | no | **0** |
| `cache`, `tmp`, `CachedData` | Cache | yes | 0 |
| `.credentials.json`, `token.txt` | Secret | no | 0 |
| `*.sqlite`, `*.vscdb` | SQLite | no | 0 |
| `*.log`, `.last-cleanup` | Log | yes | 0 |
| `mcp.json`, `settings.json` | Settings | no | 0 |
| `password-reset.md`, `cachet.json` | Other | no | 0 |

Gold-store sum in tests: old History-as-prompt **965_000** tok vs now
**16_288**. Re-run: `cargo test -p sweeploom-ai bench_token_context_weight -- --nocapture`.

---

## Safety

- Recommendation never overrides a blocker.
- Apply: plan → revalidate → execute → receipt.
- Secrets / SQLite / History / `AGENTS.md` / `*.mdc` / `rules` are
  inspect-only.
- `park?` is advice. No `alwaysApply` writes.
- Live agents Keep. Attached MCP Keep. CLI/MCP do not terminate.
- Command lines redacted (`--token`, URI userinfo).
- Process key = `PID + start time`.
- `deny.toml` bans `tokio`, `hyper`, `reqwest`.

---

## Benchmarks

These are **unit tests that print numbers**, not a marketing slide.
Re-run them. Arithmetic and Keep/Optional must match. Nanoseconds
change with the machine.

Captured 2026-09-14 on Windows (this repo, `cargo test --offline`).

### Live coding-agent check (2026-09-15)

The fixture benchmark above measures classification policy. A separate
run checked actual coding work: find a real bug and remove duplicate
classifier logic. Five models ran once without SweepLoom and again in
clean isolated clones with the live SweepLoom MCP. Each clone used its
own Cargo target. Earlier static-capsule “with” runs were invalid and
are excluded.

Token values below are reproducible transcript estimates
(`characters / 4`, including reconstructed tool results), not provider
billing:

| Task | Without total spend | Live MCP total spend | Delta | Without peak sum | Live MCP peak sum | Delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Real bug | 3,636,327 | 6,966,918 | **+91.6%** | 252,759 | 305,750 | **+21.0%** |
| Duplicate logic | 2,479,471 | 3,766,840 | **+51.9%** | 185,975 | 234,166 | **+25.9%** |
| **Comparable total** | **6,115,798** | **10,733,758** | **+75.5%** | **438,734** | **539,916** | **+23.1%** |

This run found **no coding-task token saving**. The live-MCP prompts were
longer and required feedback, so the delta is not a clean causal cost
estimate; it is still direct evidence against claiming a saving here.

Observed use:

- All ten live-MCP runs called `disk_inventory`; some also called
  `list_projects` or `advise_context`.
- Inventory helped confirm the clean root and coarse crate scope.
- Source directories frequently remained `Unknown`; the tool exposed no
  file/symbol map, semantic duplicate finder, or CLI-to-source map.
- Agents still found and proved the code changes through normal source
  reads and tests. Repeated inventory calls added context.
- T1 produced five independently rebuilt, passing fixes. All five T2
  agents reported passing tests on unique targets; the run was stopped
  before a separate parent re-verification. The live-MCP T3 split-file
  runs were never started and are not represented as zero rows.

The run also produced four source fixes now covered by regression tests:

1. stale browser companion snapshots no longer expose tab counts;
2. missing session option values no longer swallow the next flag;
3. inspect-only MCP tools publish `readOnlyHint`;
4. duplicate naive Secret logic in the benchmark classifiers is shared
   while intentionally different branches remain separate.

The benchmark canvas contains the row-level working report. Raw
transcripts and machine-specific `file_output/` artifacts are excluded
from releases.

### How to run

```text
sweeploom bench
cargo test -p sweeploom-ai bench_ai_with_and_without_sweeploom -- --nocapture
cargo test -p sweeploom-ai bench_token_context_weight -- --nocapture
cargo test -p sweeploom-ai bench_classify_accuracy_and_speed -- --nocapture
cargo test -p sweeploom-session bench_agent_survive_with_and_without_sweeploom -- --nocapture
cargo test -p sweeploom-session bench_token_policy_keeps_context -- --nocapture
cargo test -p sweeploom-session bench_session_accuracy_and_speed -- --nocapture
cargo test -p sweeploom bench_with_without_card -- --nocapture
cargo test -p sweeploom --lib -- --nocapture
cargo test -p sweeploom --doc
```

Everything that asserts the product idea:

```text
cargo test -p sweeploom-ai --lib
cargo test -p sweeploom-session --lib
cargo test -p sweeploom-core --lib
```

### WITH vs WITHOUT (how the AI copes)

Same gold store as the token tax test. An agent **without** SweepLoom
either dumps the folder (`bytes/4` of everything) or bills History as
prompt and guesses classes with substrings. **With** SweepLoom the
always-on slice is capped and History is 0.

```text
$ sweeploom bench
$ cargo test -p sweeploom-ai bench_ai_with_and_without_sweeploom -- --nocapture
$ cargo test -p sweeploom-session bench_agent_survive_with_and_without_sweeploom -- --nocapture
$ cargo test -p sweeploom bench_with_without_card -- --nocapture
```

```text
WITHOUT dump-store/4          238466250 tok
WITHOUT History-as-prompt     965000 tok
WITH    always-on estimate    16288 tok
apply confirm=false           refused=true deleted=0
idle 2GB agents               WITHOUT would-kill=6  WITH Keep=6
WITHOUT wrong class=8  false-secret=1  would-clean-Context=5
WITH    wrong class=0
```

Fails if WITH ≥ History-as-prompt, if classify misses Context, if an
idle agent is not Keep, or if `confirm=false` deletes anything.

Sources: `crates/sweeploom-ai/src/with_without_bench.rs`,
`crates/sweeploom-session/src/with_without_bench.rs`,
`crates/sweeploom-cli/src/cmd_bench.rs`.

### Token tax (history is archive)

```text
$ cargo test -p sweeploom-ai bench_token_context_weight -- --nocapture
```

```text
--- always-on token tax (history is archive, dirs capped) ---
  AGENTS.md                   12000 B  files=1     class=Context  old=3000 tok  now=3000 tok
  always-on.mdc                4000 B  files=1     class=Context  old=1000 tok  now=1000 tok
  skills                      80000 B  files=200   class=Context  old=20000 tok  now=4096 tok
  plugins                     40000 B  files=80    class=Context  old=10000 tok  now=4096 tok
  rules                       24000 B  files=30    class=Context  old=6000 tok  now=4096 tok
  history.jsonl             2000000 B  files=1     class=History  old=500000 tok  now=0 tok
  projects                   500000 B  files=40    class=History  old=125000 tok  now=0 tok
  archived_sessions         1200000 B  files=20    class=History  old=300000 tok  now=0 tok
  cache                   900000000 B  files=10000  class=Cache  old=0 tok  now=0 tok
  .credentials.json            2000 B  files=1     class=Secret  old=0 tok  now=0 tok
  state.vscdb              50000000 B  files=1     class=Sqlite  old=0 tok  now=0 tok
  password-reset.md            3000 B  files=1     class=Other  old=0 tok  now=0 tok
SUM always-on tokens: old(overcount)=965000  now=16288  history_dropped=925000
false-clean on Context=0  still_hidden=0
test token_bench::bench_token_context_weight ... ok
```

`965000 / 16288 ≈ 59×` if you used to bill History as prompt. The test
**fails** if `now >= old`, if any Context is `can_clean()`, or if
History/Cache/Secret are not 0.

Source: `crates/sweeploom-ai/src/token_bench.rs`.

### Classify gold (no LLM)

```text
$ cargo test -p sweeploom-ai bench_classify_accuracy_and_speed -- --nocapture
```

```text
classify gold 30/30  legacy 24/30  secret-false-clean legacy=0 now=0
  delta password-reset.md: Secret -> Other (gold Other)
  delta cachet.json: Cache -> Other (gold Other)
  delta AGENTS.md: Other -> Context (gold Context)
  delta always-on.mdc: Other -> Context (gold Context)
  delta skills: Other -> Context (gold Context)
  delta plugins: Other -> Context (gold Context)
classify_name 1500000 calls in 2.9305632s (~1953 ns/call) sink=400008
test classify_bench::bench_classify_accuracy_and_speed ... ok
```

Fails if accuracy does not beat the old substring rules, if a gold
Secret is cleanable, or if a call exceeds **50 µs**. This box: **~2 µs**.

Source: `crates/sweeploom-ai/src/classify_bench.rs` (30 names).

Why not Spark on this path (`file_output/spark_bench/RESULTS.md`,
same day, gpt-5.3-codex-spark):

```text
local classify_name     microseconds     0 tokens
Spark + JSON schema     9.76 s           18_128 tokens
Spark freeform          17.17 s          14_229 tokens
```

### Live agents Keep

```text
$ cargo test -p sweeploom-session bench_token_policy_keeps_context -- --nocapture
$ cargo test -p sweeploom-session bench_session_accuracy_and_speed -- --nocapture
```

```text
--- token policy: live agents Keep (context preserved) ---
  idle ClaudeCode 2GB -> Keep SleepingMemoryHeavy reclaim=0
  idle Codex 2GB -> Keep SleepingMemoryHeavy reclaim=0
  idle Cursor 2GB -> Keep SleepingMemoryHeavy reclaim=0
  idle OpenCode 2GB -> Keep SleepingMemoryHeavy reclaim=0
  idle Gemini 2GB -> Keep SleepingMemoryHeavy reclaim=0
  idle Grok 2GB -> Keep SleepingMemoryHeavy reclaim=0
attached MCP Keep RSS=80000000  orphan MCP Optional RSS=40000000
detect Cursor.exe -> Some(Cursor)
detect node --mcp -> Some(Mcp)
detect Slack.exe -> None
MCP under Cursor attached (not orphan)=true
stray MCP orphan=true
group_sessions+orphan 8000 trees in 499.2553ms (~62406 ns/tree)
test session_bench::bench_token_policy_keeps_context ... ok
test session_bench::bench_session_accuracy_and_speed ... ok
```

Fails if any listed agent is not Keep, if reclaim ≠ 0, if attached MCP
is orphan, or if grouping exceeds **5 ms/tree**. This box: **~62 µs/tree**.

Source: `crates/sweeploom-session/src/session_bench.rs`.

### MCP gate

```text
$ cargo test -p sweeploom mcp_apply_without_confirm_does_not_run -- --nocapture
```

`apply_cleanup({ confirm: false, ... })` → `ok=false`, `deleted=0`,
error contains `confirm=true`.

---

## Tests

Same day, `--lib` counts (not the GUI):

| Crate | What it locks | Command |
| --- | --- | --- |
| `sweeploom` | 11 tests: MCP refuse, with/without card, catalog, plugin JSON, session flags, `--chromium-id` | `cargo test -p sweeploom --lib` |
| `sweeploom` doctest | 2 rustdoc examples (`classify_name`, `advise_one`) | `cargo test -p sweeploom --doc` |
| `sweeploom-ai` | 16: classify gold, inventory caps, token/classify/with-without benches | `cargo test -p sweeploom-ai --lib` |
| `sweeploom-session` | 25: detectors, Keep/orphan, grouping + with/without benches | `cargo test -p sweeploom-session --lib` |
| `sweeploom-core` | 16: safety vs recommendation, redact, artifact refuse | `cargo test -p sweeploom-core --lib` |

`sweeploom --lib` names:

```text
api::tests::mcp_apply_without_confirm_does_not_run
cmd_bench::tests::bench_with_without_card
api::tests::advise_does_not_claim_history
surface::tests::cli_apply_is_flag_not_mcp_phrase
surface::tests::identities_split_hands_and_agents
surface::tests::mcp_writes_only_through_confirm
surface::tests::published_mcp_surface
surface::tests::weavatrix_marketplace_and_codex_plugin_parse
cmd_sessions::tests::parses_plan_flags
cmd_companion_install::tests::parse_reads_chromium_id_flag
cmd_companion_install::tests::parse_without_flag_is_none
```

`advise_does_not_claim_history` asserts `history.jsonl` advice is empty
and `AGENTS.md` is `keep`.

`weavatrix_marketplace_and_codex_plugin_parse` reads
`.agents/plugins/marketplace.json` and
`plugins/sweeploom/.codex-plugin/plugin.json` so the Codex layout
cannot drift silently.

Session policy tests (same crate as the benches):

```text
forgotten::tests::idle_cursor_keeps_context
forgotten::tests::idle_light_agent_is_likely_forgotten_but_kept
forgotten::tests::unknown_idle_claude_is_keep
forgotten::tests::browser_tree_is_never_auto_reclaimed
forgotten::tests::generic_app_is_never_auto_recommended
tests::mcp_child_of_cursor_is_attached_not_orphan
tests::stray_mcp_is_orphan_candidate
```

Classify / safety tests:

```text
classify::tests::secrets_and_sqlite_cannot_clean
classify::tests::docs_are_not_secrets_and_cachet_is_not_cache
classify::tests::history_and_settings_stay_inspect
safety::tests::blockers_force_blocked_level
recommendation::tests::recommendation_cannot_bypass_blocker
redaction::tests::redacts_flag_value_and_next_token
artifact::tests::refuses_src_root_dot_and_escape
artifact::tests::refuses_nested_git
```

CI (`.github/workflows/ci.yml`): `fmt`, `clippy -D warnings`,
`cargo test --workspace --locked`, `cargo doc -D warnings`,
`cargo deny`, line budget. Platform matrix: Ubuntu, Windows, macOS.

More tables: [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

---

## FAQ

**Weavatrix is the product?** No. Company. SweepLoom is the product.

**Test with SweepLoom vs without?** `sweeploom bench` and the
`with_without` tests. Without it the gold store is 238M tok (folder
dump) or 965k (History as prompt); with it **16_288**, agents Keep,
apply without confirm deletes 0.

**Does that mean coding agents spend fewer tokens?** Not by itself.
Those are fixed metadata-policy fixtures, not actual agent spend. The
2026-09-15 live-MCP coding run measured **+75.5%** estimated cumulative
spend and **+23.1%** summed peak context on comparable T1/T2 rows. Use
Cortex Loom when the goal is budgeted repository evidence.

**Kill idle Cursor?** No. Keep. Context is the point.

**Delete `history.jsonl` via `--apply`?** Not on the SAFE generated
path. Inspect-only.

**`park?` disabled my skills?** No.

**`clean .` printed `no generated candidates`?** The walk found no
SAFE generated rows (this repo often has `target` outside the tree).
That is success, not a broken CLI.

**Bare `sweeploom` hung?** It should print help. MCP serve is
`sweeploom mcp`.

**Website?** None yet. Use this GitHub repo.

**Web UI?** Later, separate product.

---

## Browser companion

```text
cargo build -p sweeploom --release --bins
sweeploom companion-install
sweeploom companion-install --chromium-id <id>
```

Extensions: `browser/chromium-extension`, `browser/firefox-extension`.

---

## License

SweepLoom is **MPL-2.0**. [`LICENSE`](LICENSE).

`weavatrix-scan`, `weavatrix-git`, `mcport` remain **MIT**. Do not
relicense them here.
