---
name: token-tax
description: Inspect always-on AI context token tax. History is archive. Do not disable AGENTS.md or rules.
---

# Always-on token tax

SweepLoom estimates prompt tokens from metadata. It does not open secrets or SQLite.

## Loop

1. Call `list_ai_stores`.
2. For a name, call `advise_context` with `{ "relative": "skills" }` (or `AGENTS.md`, `rules`, `plugins`).
3. Report `prompt_tokens` and `advice`. History / cache / secrets show `0` tokens.

## Rules

- History on disk is archive, not prompt tax. Do not treat `history.jsonl` as always-on context.
- `AGENTS.md`, `*.mdc`, and `rules` are Keep. SweepLoom will not park them.
- Stale `skills` / `plugins` (idle 30+ days) may show `park?`. SweepLoom does not flip `alwaysApply`. Tell the user; do not edit those files yourself unless they asked.
- Never invent token counts. Use the tool numbers.
- `sweeploom bench` is a **CLI fixture** (with vs without SweepLoom on a gold store). Do not use those 16_288 / 965_000 numbers as this machine's tax. Call `list_ai_stores` for the real disk.

## Example

```text
tools/call  list_ai_stores
tools/call  advise_context       { "relative": "AGENTS.md" }
tools/call  advise_context       { "relative": "skills" }
```
