# CLI — bench, browser, clean, contract

[README](../README.md) · [All docs](../README.md#documentation)

Second half of the CLI transcripts. Start with [CLI.md](CLI.md) for help,
sessions, scan, projects, and AI stores. Same capture: a real Windows
workstation on 2026-09-14.

## 6. Bench — AI without SweepLoom vs with it

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

## 7. Browser — RSS is real; tabs are not guessed

```text
$ sweeploom browser
```

```text
companion=disconnected hosts=1 rss=8.2 GB
tab lastAccessed unavailable without the companion; not shown as zero
Edge     sessions=1   proc=62   rss=8.2 GB     cpu= 13.2%
```

`tabs=0` would be a lie. Install the companion if you want
`lastAccessed` (see [Browser companion](BROWSER.md)).

## 8. Clean — look, then `--apply`

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

## 9. Print the agent contract — do not serve

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

## 10. Companion (tabs)

From a clone (a `cargo install` already puts `sweeploom-companion-host`
next to `sweeploom`, so skip the build line):

```text
$ cargo build -p sweeploom --release --bins
$ sweeploom companion-install
$ sweeploom companion-install --chromium-id <id from edge://extensions>
$ sweeploom browser
```

Load `browser/chromium-extension` or `browser/firefox-extension`
unpacked. `companion-host` is what the browser launches — not a daily
command.

## 11. One-liners you will actually type

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
cargo run -p sweeploom -- sessions     # from a clone, not installed
cargo run -p sweeploom -- clean .
```
