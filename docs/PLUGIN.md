# Codex plugin

[README](../README.md) · [All docs](../README.md#documentation)

SweepLoom ships as a [Codex](https://developers.openai.com/codex/plugins) plugin:
the same MCP server, plus skills that force review-before-apply. The skills
encode the copy-paste chats in [MCP-CHATS.md](MCP-CHATS.md) and
[MCP-CHATS-AGENTS.md](MCP-CHATS-AGENTS.md) (Chats A–D).

Weavatrix is the **company**. SweepLoom is the **product**. The marketplace
name in this repo is `weavatrix` so other Weavatrix products can join the
same catalog later.

A web UI is a later, separate product. The egui desktop app stays in-tree as
an optional local reviewer (`cargo run --release -p sweeploom-gui`). It is not
the published front-end.

## Install (Codex CLI)

From GitHub:

```text
codex plugin marketplace add Weavatrix/sweeploom
codex plugin add sweeploom@weavatrix
```

From a clone: `codex plugin marketplace add .` then
`codex plugin add sweeploom@weavatrix`.

Then start a **new** Codex session. Installed skills and MCP tools load on
new sessions, not mid-chat.

```text
codex plugin list
/plugins
```

### The bundled MCP entry needs npm (not published yet)

The plugin's `plugins/sweeploom/.mcp.json` starts `npx -y sweeploom mcp`.
SweepLoom is not on npm yet, so that entry cannot start on its own today.
Until it is published, install the CLI from source
([INSTALL.md](INSTALL.md)) and register the server in Codex's own MCP
configuration, `~/.codex/config.toml`:

```toml
[mcp_servers.sweeploom]
command = "sweeploom"
args = ["mcp"]
```

The plugin's skills load either way.

## Ask

In a **new** Codex session:

```text
Use SweepLoom to list cleanup_candidates for this repo and explain anything unclear. Do not apply yet.
Use SweepLoom to list_ai_stores and show always-on token tax. Do not park AGENTS.md or rules.
Use SweepLoom to list_sessions. Do not terminate live agents.
```

SweepLoom is **MCP**, not a Cursor Marketplace plugin. Search there and
you will see no matches. Add a server entry.

## Install (MCP only, any host)

This clone already has:

- Cursor: `.cursor/mcp.json` (installed `sweeploom mcp`)
- Claude Code / generic: `.mcp.json` (same)
- Codex plugin: `plugins/sweeploom/.mcp.json` (`npx -y sweeploom mcp`,
  needs the npm package)

If you do not use Codex plugins, point the host at the installed binary:

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
claude mcp add -s user sweeploom -- sweeploom mcp
```

Every host, explicit paths, and the tool table: [MCP.md](MCP.md).

## Layout

```text
.cursor/mcp.json                     Cursor (sweeploom mcp)
.mcp.json                            Claude Code / generic (same)
.agents/plugins/marketplace.json     name: weavatrix (the company catalog)
plugins/sweeploom/
  .codex-plugin/plugin.json          Codex manifest
  .mcp.json                          npx -y sweeploom mcp
  skills/review-cleanup/SKILL.md
  skills/token-tax/SKILL.md
  skills/live-sessions/SKILL.md
```

`cargo test -p sweeploom weavatrix_marketplace_and_codex_plugin_parse`
reads the marketplace and plugin manifests so this layout cannot drift
silently ([TESTING.md](TESTING.md)).

## What the plugin will not do

- Terminate live agents (that drops chat context).
- Apply without `confirm=true`.
- Rewrite `AGENTS.md` / `*.mdc` / `rules`.
- Open secrets or SQLite.
