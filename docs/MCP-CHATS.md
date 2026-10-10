# MCP chats — review, then apply (copy these)

[README](../README.md) · [All docs](../README.md#documentation)

MCP is **stdio JSON-RPC**. You do not type `tools/call` in a shell.
The host (Cursor, Claude Code, Codex) does. Below, `User` / `Agent` /
`Tool` is the chat. JSON is what the tool returns. Setup and the full
argument table: [MCP.md](MCP.md).

## Chat A — “this repo is huge, clean it” (the only apply path)

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

## Chat B — agent tries to skip review (must fail)

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

More chats (tokens, live agents, folders):
[MCP-CHATS-AGENTS.md](MCP-CHATS-AGENTS.md).
