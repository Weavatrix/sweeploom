---
name: review-cleanup
description: Review SweepLoom generated cleanup rows, then apply only after the user agrees. Never delete first.
---

# Review, then apply

SweepLoom deletes nothing until a human-reviewed apply. You are on MCP.

## Loop

1. Call `cleanup_candidates` with `{ "root": "<absolute workspace path>" }`.
2. Call `explain_candidate` for any row you do not understand: `{ "id": <n>, "root": "<same path>" }`.
3. Show the user a short plan: path, size, why it is SAFE generated.
4. Only after they say yes, call `apply_cleanup` with `{ "confirm": true, "root": "<same path>", "ids": [<n>, ...] }`.

## Rules

- Without `confirm: true`, apply is refused and nothing is deleted. That is correct. Do not retry with confirm unless the user agreed.
- Omit `ids` only when the user asked to apply the pre-selected SAFE rows (same as `sweeploom clean --apply`).
- Never terminate processes. `apply_cleanup` cannot do that.
- Never rewrite `AGENTS.md`, `*.mdc`, or `rules`.
- Secrets, SQLite, History, and Context rows are inspect-only. Skip them.

## Example

```text
tools/call  cleanup_candidates   { "root": "C:/work/app" }
tools/call  explain_candidate    { "id": 3, "root": "C:/work/app" }
# show the user row 3
tools/call  apply_cleanup        { "confirm": true, "root": "C:/work/app", "ids": [3] }
```
