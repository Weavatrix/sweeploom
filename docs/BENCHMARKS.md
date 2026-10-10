# Benchmarks (re-run these)

[README](../README.md) · [All docs](../README.md#documentation)

Numbers below are **assertions in tests**, not marketing slides. Re-run on
your machine. Wall times change; the token arithmetic and policy checks
must not.

Every bench is **with SweepLoom vs without**. Without it, an agent
improvises: dump the store, bill History as prompt, substring-classify
(`AGENTS.md` → Other, `password-reset.md` → Secret), and kill idle
Cursor to free RAM. The tests fail if SweepLoom does not beat that.

Captured 2026-09-14 on Windows (this repo, `cargo test --offline`).

These fixtures measure classification policy, not coding-agent spend. A
live coding-agent run found **no** token saving; see
[BENCHMARKS-LIVE.md](BENCHMARKS-LIVE.md).

## How to run

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

## WITH vs WITHOUT (how the AI copes)

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

## Always-on token tax vs History-bytes/4

Command:

```text
cargo test -p sweeploom-ai bench_token_context_weight -- --nocapture
```

The gold store (see `crates/sweeploom-ai/src/token_bench.rs`):

| Leaf | Bytes | Files | Class | Old tax (bytes/4 if Context or History) | Now |
| --- | ---: | ---: | --- | ---: | ---: |
| `AGENTS.md` | 12_000 | 1 | Context | 3_000 | 3_000 |
| `always-on.mdc` | 4_000 | 1 | Context | 1_000 | 1_000 |
| `skills` | 80_000 | 200 | Context | 20_000 | **4_096** (dir cap) |
| `plugins` | 40_000 | 80 | Context | 10_000 | **4_096** |
| `rules` | 24_000 | 30 | Context | 6_000 | **4_096** |
| `history.jsonl` | 2_000_000 | 1 | History | 500_000 | **0** |
| `projects` | 500_000 | 40 | History | 125_000 | **0** |
| `archived_sessions` | 1_200_000 | 20 | History | 300_000 | **0** |
| `cache` | 900_000_000 | 10_000 | Cache | 0 | 0 |
| `.credentials.json` | 2_000 | 1 | Secret | 0 | 0 |
| `state.vscdb` | 50_000_000 | 1 | SQLite | 0 | 0 |
| `password-reset.md` | 3_000 | 1 | Other | 0 | 0 |

Sums the test prints and asserts:

```text
old(overcount) = 965_000
now            = 16_288
history_dropped = 925_000
```

`965_000 / 16_288 ≈ 59×` less “visible prompt” if you used to bill History
as if it were injected every turn. History is archive. Always-on files are
capped (`8_000` tokens/file, `4_096` for `skills`/`plugins`/`rules`).

Also asserted:

- `estimated_prompt_tokens(History, "history.jsonl", 2_000_000) == 0`
- `estimated_prompt_tokens(Cache, "cache", 900_000_000) == 0`
- `estimated_prompt_tokens(Secret, ".credentials.json", 2_000) == 0`
- no Context row is `can_clean()`
- `16_288` is between `4_000` and `8_000+1_000+4_096×3`

Raw test output:

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

## Classify accuracy (no LLM)

```text
cargo test -p sweeploom-ai bench_classify_accuracy_and_speed -- --nocapture
```

30 gold names in `crates/sweeploom-ai/src/classify_bench.rs`. Current
`classify_name` must beat the old substring rules and must never mark a
gold Secret as cleanable. Captured 2026-09-14 (Windows): **30/30**,
legacy **24/30**, `secret-false-clean now=0`, **~1953 ns/call**
(1_500_000 calls). The test fails if a call exceeds 50 µs.

Deltas vs the old substring rules include: `password-reset.md` is Other
(not Secret), `cachet.json` is Other (not Cache), `AGENTS.md` / `skills`
are Context.

Raw test output:

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

## Why not an LLM on the hot path

`file_output/spark_bench/RESULTS.md` (2026-09-14, gpt-5.3-codex-spark):

| Path | Wall | Tokens billed |
| --- | --- | --- |
| Local `classify_name` (Rust) | microseconds | 0 |
| Spark, 25 names + JSON schema | 9.76 s | 18_128 |
| Spark, 25 names freeform | 17.17 s | 14_229 |

Re-run that comparison only if you want it; it needs a Codex CLI login.
The product decision is already encoded: **no LLM on classify/apply**.

## Live agents Keep (context preserved)

```text
cargo test -p sweeploom-session bench_token_policy_keeps_context -- --nocapture
cargo test -p sweeploom-session bench_session_accuracy_and_speed -- --nocapture
```

Asserted:

- Idle Claude / Codex / Cursor / OpenCode / Gemini / Grok at 2 GB RSS →
  **Keep**, reclaimable RSS **0**
- MCP under Cursor → attached, **Keep**, 80 MB
- Stray `mcp-server` → **OrphanCandidate** / **Optional**, 40 MB
- `Cursor.exe` detects as Cursor; `node --mcp` as MCP; Slack is not an agent
- Grouping 8_000 synthetic trees stays under **5 ms/tree**
  (captured: **~62 µs/tree**)

Raw test output:

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

## MCP apply gate

```text
cargo test -p sweeploom mcp_apply_without_confirm_does_not_run -- --nocapture
```

`apply_cleanup` with `confirm: false` returns `ok=false`, `deleted=0`, and
an error containing `confirm=true`.

Test inventory and CI: [TESTING.md](TESTING.md).
