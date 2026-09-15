# Benchmarks (re-run these)

Numbers below are **assertions in tests**, not marketing slides. Re-run on
your machine. Wall times change; the token arithmetic and policy checks
must not.

Every bench is **with SweepLoom vs without**. Without it, an agent
improvises: dump the store, bill History as prompt, substring-classify
(`AGENTS.md` → Other, `password-reset.md` → Secret), and kill idle
Cursor to free RAM. The tests fail if SweepLoom does not beat that.

```text
sweeploom bench
cargo test -p sweeploom-ai bench_ai_with_and_without_sweeploom -- --nocapture
cargo test -p sweeploom-session bench_agent_survive_with_and_without_sweeploom -- --nocapture
cargo test -p sweeploom bench_with_without_card -- --nocapture
```

Captured 2026-09-14:

```text
WITHOUT dump-store/4          238466250 tok
WITHOUT History-as-prompt     965000 tok
WITH    always-on estimate    16288 tok
apply confirm=false           refused=true deleted=0
idle 2GB agents               WITHOUT would-kill=6  WITH Keep=6
WITHOUT wrong class=8  false-secret=1  would-clean-Context=5
WITH    wrong class=0
```

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

## MCP apply gate

```text
cargo test -p sweeploom mcp_apply_without_confirm_does_not_run
```

`apply_cleanup` with `confirm: false` returns `ok=false`, `deleted=0`, and
an error containing `confirm=true`.
