# MCP for agents

[README](../README.md) · [All docs](../README.md#documentation)

MCP is **stdio JSON-RPC**. You do not type `tools/call` in a shell.
The host (Cursor, Claude Code, Codex) does. In the chat transcripts,
`User` / `Agent` / `Tool` is the chat. JSON is what the tool returns.

Server id: `io.github.Weavatrix/sweeploom`.
Launch: `sweeploom mcp` — the `sweeploom` binary with the argument `mcp`,
installed from source ([INSTALL.md](INSTALL.md)). There is no separate
`sweeploom-mcp` binary in a Cargo install; that name is only the alias of
the future npm launcher. `npx -y sweeploom mcp` will work only after the
npm package is published (it is not today).

- Copy-paste chats: [MCP-CHATS.md](MCP-CHATS.md) (review, then apply;
  the refused shortcut) and [MCP-CHATS-AGENTS.md](MCP-CHATS-AGENTS.md)
  (token tax, idle agents, folder map).
- What may and may not be deleted: [SAFETY.md](SAFETY.md).

## Hook it up

This clone already ships the MCP launch files. Cursor Marketplace will
not list SweepLoom — toggle **MCP**, not Plugins.

| Host | File | Starts |
| --- | --- | --- |
| **Cursor** | [`.cursor/mcp.json`](../.cursor/mcp.json) | installed `sweeploom mcp` |
| **Claude Code** | [`.mcp.json`](../.mcp.json) | installed `sweeploom mcp` |
| **Codex** | [`plugins/sweeploom/.mcp.json`](../plugins/sweeploom/.mcp.json) | `npx -y sweeploom mcp` (needs the npm package — not published yet; see [PLUGIN.md](PLUGIN.md)) |

Prefer an installed binary in an always-on MCP configuration. Using
`cargo run` there recompiles on host restarts, emits build diagnostics
on the MCP process, and can disconnect with an OS disk-full error.
Install once, either straight from GitHub or from a clone:

```text
cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom
cargo install --path crates/sweeploom-cli --locked
sweeploom mcp --list
```

Then point the host at `sweeploom mcp`. `cargo run ... -- mcp` remains a
development command, not the recommended persistent host wiring.

If the server is connected but you never say “SweepLoom”, the model
can ignore the catalog and use Shell. Name the product (or a skill)
when you want `list_ai_stores`.

**Cursor** — this project: `.cursor/mcp.json`. Every workspace:
`~/.cursor/mcp.json` (`%USERPROFILE%\.cursor\mcp.json` on Windows):

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

GUI hosts started from the Dock, Start menu, or a launcher may not see
your shell `PATH`. If the server does not start, use the absolute path
of the installed binary (`~/.cargo/bin/sweeploom` on macOS / Linux).
Windows, explicit path:

```json
{
  "mcpServers": {
    "sweeploom": {
      "command": "C:\\Users\\you\\.cargo\\bin\\sweeploom.exe",
      "args": ["mcp"]
    }
  }
}
```

**Claude Code:**

```text
claude mcp add -s user sweeploom -- sweeploom mcp
```

From a clone without installing (development only; recompiles on host
restarts):

```text
claude mcp add -s user sweeploom -- cargo run -q --manifest-path C:/Users/you/work/sweeploom/Cargo.toml -p sweeploom -- mcp
```

This repo’s [`.mcp.json`](../.mcp.json) is the same `sweeploom mcp`
launch for a project scope.

**Claude Desktop** — `%APPDATA%\Claude\claude_desktop_config.json`
(Windows) or `~/Library/Application Support/Claude/claude_desktop_config.json`
(macOS). Same `mcpServers.sweeploom` object as Cursor.

**Codex** — prefer the [plugin](PLUGIN.md) (MCP + skills). MCP only, in
`~/.codex/config.toml`:

```toml
[mcp_servers.sweeploom]
command = "sweeploom"
args = ["mcp"]
```

The plugin's bundled entry is:

```text
# plugins/sweeploom/.mcp.json
npx -y sweeploom mcp
```

That line needs the npm package, which is not published yet.

Restart the host. Empty `root` on path tools = **user home**. Always
pass the workspace absolute path.

**Logo / mark.** No marketplace icon. In docs and the optional GUI the
accent is gold `#C48C30` on a dark field; the name is one word
**SweepLoom**. Provider marks (Claude / Codex / Cursor) are those
products’ art, not SweepLoom’s.

## Silent vs told

Same MCP connection. Different ask.

```text
# connected, not named — model may skip the catalog
Codex feels expensive. Do not delete my chats.

# connected and named — model should call list_ai_stores
Use SweepLoom to list_ai_stores. Do not park AGENTS.md.
```

Tests: `cargo test -p sweeploom silent_asks_never_name_the_product`.

## Every MCP tool (arguments)

| Tool | Args | Writes |
| --- | --- | --- |
| `list_sessions` | none | no |
| `disk_inventory` | `root?` string | no |
| `list_projects` | `root?` string | no |
| `list_ai_stores` | none | no |
| `advise_context` | `relative` string | no |
| `cleanup_candidates` | `root?` string | no |
| `explain_candidate` | `id` u64, `root?` string | no |
| `list_browser` | none | no |
| `apply_cleanup` | `confirm` bool, `root?`, `ids?` u64[] | **yes if confirm** |

There is no `terminate`, no `park_file`, no `rewrite_agents`.

The same catalog, as the CLI prints it (`sweeploom mcp --list`), is in
[CLI-CLEAN.md](CLI-CLEAN.md#9-print-the-agent-contract--do-not-serve).
