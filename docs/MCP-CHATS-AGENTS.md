# MCP chats — tokens, live agents, folders

[README](../README.md) · [All docs](../README.md#documentation)

MCP is **stdio JSON-RPC**. You do not type `tools/call` in a shell.
The host (Cursor, Claude Code, Codex) does. Below, `User` / `Agent` /
`Tool` is the chat. JSON is what the tool returns. Setup and the full
argument table: [MCP.md](MCP.md).

Cleanup and the refused shortcut are in [MCP-CHATS.md](MCP-CHATS.md)
(Chat A and Chat B).

## Chat C — “why is my agent so expensive?” (tokens, not RAM)

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

## Chat D — “Cursor is idle, kill it” (must refuse)

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

## Chat E — map a folder, then decide

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

Then continue as [Chat A](MCP-CHATS.md#chat-a--this-repo-is-huge-clean-it-the-only-apply-path) if they want a delete. `tabs: null` is
“unknown”, not zero.
