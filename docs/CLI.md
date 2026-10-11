# CLI — full sessions (copy these)

[README](../README.md) · [All docs](../README.md#documentation)

Install first: [INSTALL.md](INSTALL.md). From a clone without installing,
prefix every command with `cargo run -p sweeploom --` instead of `sweeploom`.

These transcripts were captured on a real Windows workstation
(2026-09-14): 479 processes, Cursor + Codex + Claude live, Edge 8.2 GB,
`~/.claude` 875 MB, `~/.codex` 8.7 GB. Your numbers will differ. The
**shape** of the commands will not.

## 1. Help — this is not MCP

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
`sweeploom mcp`, which a host should spawn.

```text
$ sweeploom frobnicate
unknown command: frobnicate
(then the same help)
# exit 2
```

## 2. “What is alive?” — do not kill Cursor

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

## 3. “What is on disk in this repo?”

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

## 4. Projects — heat and git, then offers

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

## 5. AI stores — token tax, not “delete ~/.claude”

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

Continued in [CLI-CLEAN.md](CLI-CLEAN.md): bench, browser, clean,
the agent contract, the companion, and the one-liners you will type.
