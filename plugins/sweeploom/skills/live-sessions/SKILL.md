---
name: live-sessions
description: List live coding sessions. Keep live agents. Only orphan MCP (no parent agent) may be Optional.
---

# Live sessions

1. Call `list_sessions`. Optional dry-run plans exist only on the CLI (`--free-ram`, `--quiet`). MCP listing does not kill anything.
2. Live agents (Claude, Codex, Cursor, OpenCode, Gemini, Grok) stay **Keep**. Killing them drops chat context.
3. MCP attached to a live agent stays Keep. Stray MCP with no agent parent can be Optional.
4. Do not terminate processes from this plugin. Terminate is not an MCP tool.

## Example

```text
tools/call  list_sessions
```

If the user wants a dry-run RAM/CPU plan, tell them to run the CLI:

```text
sweeploom sessions --quiet
sweeploom sessions --free-ram 4
sweeploom sessions --reduce-cpu 20
```

Those flags print a plan. They do not kill.
