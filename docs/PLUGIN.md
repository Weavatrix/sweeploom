# Codex plugin

SweepLoom ships as a [Codex](https://developers.openai.com/codex/plugins) plugin:
bundled MCP (`sweeploom-mcp`) plus skills that force review-before-apply.

Weavatrix is the **company**. SweepLoom is the **product**. The marketplace
name in this repo is `weavatrix` so other Weavatrix products can join the
same catalog later.

A web UI is a later, separate product. The egui desktop app stays in-tree as
an optional local reviewer (`cargo run -p sweeploom-gui`). It is not the
published front-end.

## Install (Codex CLI)

From this repository (or a clone):

```text
codex plugin marketplace add .
codex plugin add sweeploom@weavatrix
```

From GitHub after the repo is public:

```text
codex plugin marketplace add Weavatrix/sweeploom
codex plugin add sweeploom@weavatrix
```

Then start a **new** Codex session. Installed skills and MCP tools load on
new sessions, not mid-chat.

```text
codex plugin list
/plugins
```

Ask, for example:

```text
Use SweepLoom to list cleanup_candidates for this repo and explain anything unclear. Do not apply yet.
```

SweepLoom is **MCP**, not a Cursor Marketplace plugin. Search there and
you will see no matches. Add a server entry.

## Install (MCP only, any host)

This clone already has:

- Cursor: `.cursor/mcp.json` (`cargo run -p sweeploom -- mcp`)
- Claude Code / generic: `.mcp.json` (same)
- Codex plugin: `plugins/sweeploom/.mcp.json` (`npx -y sweeploom mcp`)

If you do not use Codex plugins, point the host at the same server:

```json
{
  "mcpServers": {
    "sweeploom": {
      "command": "npx",
      "args": ["-y", "sweeploom", "mcp"]
    }
  }
}
```

```json
{
  "mcpServers": {
    "sweeploom": {
      "command": "sweeploom",
      "args": ["mcp"]
    }
  }
}
```

```text
claude mcp add -s user sweeploom -- npx -y sweeploom mcp
```

## Layout

```text
.cursor/mcp.json                     Cursor (cargo run -- mcp)
.mcp.json                            Claude Code / generic (same)
.agents/plugins/marketplace.json     Weavatrix catalog
plugins/sweeploom/
  .codex-plugin/plugin.json          Codex manifest
  .mcp.json                          npx -y sweeploom mcp
  skills/review-cleanup/SKILL.md
  skills/token-tax/SKILL.md
  skills/live-sessions/SKILL.md
```

## What the plugin will not do

- Terminate live agents (that drops chat context).
- Apply without `confirm=true`.
- Rewrite `AGENTS.md` / `*.mdc` / `rules`.
- Open secrets or SQLite.
