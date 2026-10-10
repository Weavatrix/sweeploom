# The idea

[README](../README.md) · [All docs](../README.md#documentation)

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

## Scope: SweepLoom is not a context compressor

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

Next: [SAFETY.md](SAFETY.md) (classes and the apply gate),
[BENCHMARKS.md](BENCHMARKS.md) (with vs without), [CLI.md](CLI.md)
(transcripts).
