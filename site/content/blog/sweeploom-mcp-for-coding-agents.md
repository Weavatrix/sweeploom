---
title: "Give your coding agent a cleanup tool it can't misuse: SweepLoom MCP"
date: "2026-09-29"
summary: "Ask an agent to free disk space and it improvises with du, guesses and rm -rf. SweepLoom's MCP server gives it eight read-only tools and one gated apply instead. Setup for Claude Code, Cursor, Claude Desktop and Codex, the confirm=true contract, and the limits worth knowing."
tags: ["mcp", "agents", "safety", "claude-code", "cursor", "codex"]
---

"My disk is almost full, can you clean up this repo?" is a reasonable thing to ask a coding agent. It is also a request with no good answer from a shell. The agent runs `du`, decides from folder names what looks disposable, and proposes an `rm -rf`. If it is being thorough, it measures `~/.claude`, divides by four and tells you your context is costing hundreds of millions of tokens. If you mention memory, it may offer to kill the editor it is running in.

None of that is malice. The agent simply has no ground truth about your machine, and nothing between its guess and your files. SweepLoom's MCP server provides both.

## What an agent does without it

SweepLoom's benchmarks are written as "with versus without", and the "without" column is a fair description of an improvising agent on a fixed fixture:

| Without SweepLoom | What happens |
| --- | --- |
| Dump `~/.claude` into the estimate | 238,466,250 tokens (cache bytes divided by 4) |
| Skip the cache, still bill History | 965,000 tokens |
| Classify by substring | `AGENTS.md`, `skills` and `rules` look like junk; `password-reset.md` looks like a secret |
| "Free RAM" | idle 2 GB Claude, Codex, Cursor, OpenCode, Gemini and Grok sessions get killed |
| Apply without a second look | deletes whatever cleanup the agent invents |

With SweepLoom the same fixture is 16,288 always-on tokens, History counts as zero, all six agents are Keep, and an apply without confirmation is refused with nothing deleted. Those numbers are assertions in tests; `sweeploom bench` prints them.

## Install and connect

The server is the CLI binary itself. Install it from source (SweepLoom is not on crates.io or npm yet):

```text
$ cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom
$ sweeploom mcp --list
```

`sweeploom mcp --list` prints the contract without serving. The host starts `sweeploom` with the argument `mcp` and speaks JSON-RPC over stdio; there is no separate `sweeploom-mcp` binary.

**Claude Code**, for every project:

```text
$ claude mcp add -s user sweeploom -- sweeploom mcp
```

**Cursor** reads `~/.cursor/mcp.json` (or `.cursor/mcp.json` in a project), and **Claude Desktop** reads `claude_desktop_config.json` (`~/Library/Application Support/Claude/` on macOS, `%APPDATA%\Claude\` on Windows). Both take the same entry:

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

**Codex** reads `~/.codex/config.toml`:

```toml
[mcp_servers.sweeploom]
command = "sweeploom"
args = ["mcp"]
```

Two practical notes. Apps started from the Dock or Start menu may not inherit your shell `PATH`; if the server does not start, use the absolute path to the binary in `~/.cargo/bin`. And SweepLoom is an MCP server, not a Cursor Marketplace plugin, so searching the Marketplace finds nothing.

## Nine tools, one that writes

| Tool | Arguments | Access |
| --- | --- | --- |
| `list_sessions` | none | read-only |
| `disk_inventory` | `root?` | read-only |
| `list_projects` | `root?` | read-only |
| `list_ai_stores` | none | read-only |
| `advise_context` | `relative` | read-only |
| `cleanup_candidates` | `root?` | read-only |
| `explain_candidate` | `id`, `root?` | read-only |
| `list_browser` | none | read-only |
| `apply_cleanup` | `confirm`, `root?`, `ids?` | **writes, only with confirm** |

The eight read-only tools publish `readOnlyHint` in `tools/list`, so a host can let them run without asking you each time. A test compares the served annotations with SweepLoom's own catalog and fails if they drift. An empty `root` means your home directory, so a good agent always passes the workspace path.

What is missing is deliberate: there is no `terminate`, no `park_file` and no `rewrite_agents`.

## The contract: list, explain, ask, apply

The only path to a deletion looks like this:

```text
tools/call  cleanup_candidates   { "root": "C:/work/app" }
tools/call  explain_candidate    { "id": 3, "root": "C:/work/app" }
# the agent shows you the rows and asks
tools/call  apply_cleanup        { "confirm": true, "root": "C:/work/app", "ids": [3] }
```

In a real chat, the agent's message in the middle reads like a plan you can say yes or no to:

```text
Two SAFE generated rows:

- id 3  cargo target     ~1.2 GB  rebuild = cargo build
- id 4  node_modules     ~812 MB  rebuild = npm ci

Nothing else is pre-selected. I have not deleted anything.
Want id 3 only, both, or stop?
```

You answer "only cargo target", and the receipt comes back: `{ "ok": true, "deleted": 1, "skipped_changed": 0, "failed": 0, "receipt": 1 }`.

Underneath, `apply_cleanup` runs the same plan, revalidate, execute, receipt path as `sweeploom clean --apply`. Rows that changed since the listing are skipped. Rows that are blocked (an active process, recent writes, uncommitted Git changes) or inspect-only are filtered out even if the agent passes their ids. An empty `ids` means every pre-selected SAFE row, which is why the bundled skills tell agents to send exactly the ids you agreed to.

And when an agent tries to skip the conversation:

```text
tools/call  apply_cleanup   { "confirm": false, "root": "C:/work/app" }

refused: MCP apply needs confirm=true after cleanup_candidates; CLI uses --apply
```

Nothing is deleted. The test `mcp_apply_without_confirm_does_not_run` keeps it that way.

## Name the product

A connected server is a catalog, and models do not always read the catalog. A request that never mentions SweepLoom can be answered from the shell even with the server connected:

```text
# connected, not named: the model may skip the catalog
Codex feels expensive. Do not delete my chats.

# connected and named: the model should call list_ai_stores
Use SweepLoom to list_ai_stores. Do not park AGENTS.md.
```

Three prompts that work well:

```text
Use SweepLoom to list cleanup_candidates for this repo and explain anything unclear. Do not apply yet.
Use SweepLoom to list_ai_stores and show always-on token tax. Do not park AGENTS.md or rules.
Use SweepLoom to list_sessions. Do not terminate live agents.
```

## What it will not do for your agent

- **Stop processes.** "Cursor is idle and 7 GB, kill it" gets a `list_sessions` call, a Keep verdict, and a polite refusal. The agent can suggest `sweeploom sessions --quiet`, which prints a dry-run plan.
- **Touch native tool storage.** Docker objects, simulators, models, archives and the Trash are desktop-app actions, confirmed by a person.
- **Edit your agent's contract.** `AGENTS.md`, `*.mdc` and `rules` are Keep. `park?` on a stale skills folder is advice for you.
- **Open secrets or SQLite.** Classification uses the leaf name only.

## Honest limits

- **Tool results go to your model provider.** Paths, sizes and process details become part of the agent's context. SweepLoom itself sends nothing anywhere, but your host does send the conversation.
- **It does not save tokens on coding work.** On 2026-09-15, five models did real coding tasks with and without the live MCP. With it, estimated spend was **75.5% higher**; agents called `disk_inventory` every run and repeated calls added context. SweepLoom helps with cleanup and with understanding a machine, not with making a coding session cheaper.
- **Wire the installed binary, not `cargo run`.** A host configured with `cargo run … -- mcp` recompiles on restart, writes build output into the MCP stream, and can disconnect when the disk is full, which is exactly when you wanted it.

Setup for every host, the full tool reference with return fields, and five example chats are on the [MCP page](/mcp/).
