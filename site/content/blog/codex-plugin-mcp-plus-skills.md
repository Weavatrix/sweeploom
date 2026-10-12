---
title: "The Codex plugin: MCP plus skills"
date: "2026-10-03"
summary: "Tools tell an agent what it can do; skills tell it how to behave. SweepLoom's Codex plugin bundles the MCP server with three skills that make review-before-apply the default. What is in it, how to install it today while npm is not published, and where the real enforcement lives."
tags: ["codex", "plugin", "mcp", "agents", "skills"]
---

An MCP server answers one question for an agent: *what can I call?* SweepLoom's answers it with nine tools, eight of them read-only. What a tool catalog cannot answer is *how should I go about this?* Should the agent list candidates before applying? Should it ask? Which ids should it pass? What should it say when you ask it to kill an idle editor?

Those are behaviors, and in Codex behaviors live in **skills**: short Markdown instructions the agent loads with the session. SweepLoom's Codex plugin is the same MCP server plus three skills that encode the conversations we want agents to have.

## What is in the plugin

The plugin lives in the SweepLoom repository, listed in a marketplace named `weavatrix`. Weavatrix is the company and SweepLoom the product, so other Weavatrix products can join the same catalog later.

```text
.agents/plugins/marketplace.json     name: weavatrix (the company catalog)
plugins/sweeploom/
  .codex-plugin/plugin.json          Codex manifest
  .mcp.json                          npx -y sweeploom mcp
  skills/review-cleanup/SKILL.md
  skills/token-tax/SKILL.md
  skills/live-sessions/SKILL.md
```

The manifest points at both halves and ships three default prompts, the same ones we recommend everywhere:

```json
{
  "name": "sweeploom",
  "skills": "./skills/",
  "mcpServers": "./.mcp.json",
  "interface": {
    "displayName": "SweepLoom",
    "category": "Developer Tools",
    "defaultPrompt": [
      "Use SweepLoom to list cleanup_candidates for this repo and explain anything unclear. Do not apply yet.",
      "Use SweepLoom to list_ai_stores and show always-on token tax. Do not park AGENTS.md or rules.",
      "Use SweepLoom to list_sessions. Do not terminate live agents."
    ]
  }
}
```

A test, `weavatrix_marketplace_and_codex_plugin_parse`, reads both the marketplace and the manifest on every CI run, so the layout cannot drift silently.

## Install it today

From the Codex CLI:

```text
$ codex plugin marketplace add Weavatrix/sweeploom
$ codex plugin add sweeploom@weavatrix
```

From a clone, `codex plugin marketplace add .` works the same way. Then start a **new** Codex session: installed skills and MCP tools load with new sessions, not in the middle of one. `codex plugin list` or `/plugins` shows what is loaded.

There is one catch, and it is better stated plainly. The plugin's bundled MCP entry starts the server with `npx -y sweeploom mcp`. SweepLoom is not published on npm yet, so that entry cannot start on its own. Until it is, install the CLI from source and register the server in Codex's own configuration:

```text
$ cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom
```

```toml
# ~/.codex/config.toml
[mcp_servers.sweeploom]
command = "sweeploom"
args = ["mcp"]
```

The skills load either way. With the server registered, they have tools to call.

## Skill one: review-cleanup

This is the skill that matters most, because it governs the only tool that deletes. Its loop is four steps:

```text
1. Call cleanup_candidates with { "root": "<absolute workspace path>" }.
2. Call explain_candidate for any row you do not understand.
3. Show the user a short plan: path, size, why it is SAFE generated.
4. Only after they say yes, call apply_cleanup with
   { "confirm": true, "root": "<same path>", "ids": [<n>, ...] }.
```

The rules around it close the gaps an eager agent would find:

- Without `confirm: true`, apply is refused and nothing is deleted. That is correct; do not retry with confirm unless the user agreed.
- Omit `ids` only when the user asked to apply all pre-selected SAFE rows, the same as `sweeploom clean --apply`.
- Never terminate processes, never rewrite `AGENTS.md`, `*.mdc` or `rules`.
- Secrets, SQLite, History and Context rows are inspect-only. Skip them.

## Skill two: token-tax

When you ask why Codex feels expensive, an agent without guidance will measure the whole `~/.codex` folder. On the workstation in our docs that folder was 8.7 GB, most of it History and SQLite that the host never sends to the model. The `token-tax` skill sends the agent to `list_ai_stores` and `advise_context` instead, and sets the rules for reading the answer:

- History on disk is archive, not prompt tax. `history.jsonl` costs zero always-on tokens.
- `AGENTS.md`, `*.mdc` and `rules` are Keep. A stale `skills` or `plugins` tree may show `park?`, which is advice for the user, never an edit.
- Never invent token counts; use the tool's numbers.
- `sweeploom bench` is a fixture. Its 16,288 and 965,000 figures describe a fixed gold store, not this machine.

That last rule exists because agents love a big round number. The fixture is useful for showing the policy; it is wrong as a description of your disk.

## Skill three: live-sessions

The shortest skill, and the one that prevents the most damage:

1. Call `list_sessions`. Listing does not kill anything.
2. Live agents (Claude, Codex, Cursor, OpenCode, Gemini, Grok) stay Keep. Killing them drops chat context.
3. An MCP server attached to a live agent stays Keep. A stray one with no agent parent can be Optional.
4. Do not terminate processes from this plugin. Terminate is not an MCP tool.

If you want to know what could go, the skill points you at the CLI's dry-run plans, which print a selection and never act:

```text
sweeploom sessions --quiet
sweeploom sessions --free-ram 4
sweeploom sessions --reduce-cpu 20
```

## Where the enforcement actually lives

It is tempting to describe skills as guardrails. They are not, and pretending otherwise would be the kind of overclaim SweepLoom is built to avoid. A skill is an instruction. A model can misread it, or a long conversation can push it out of focus.

So the plugin is layered:

- **Skills make good behavior the default.** List first, explain, ask, pass exact ids, keep live agents, do not touch the contract files.
- **The server makes bad behavior impossible.** There is no terminate tool to call. `apply_cleanup` without `confirm: true` is refused. Blocked and inspect-only rows are filtered out even if their ids are passed. Every apply revalidates and returns a receipt.

If a model ignores every skill, the worst it can do through SweepLoom is send `confirm: true` for rows that a review listed as eligible: unblocked, not inspect-only, revalidated at apply time, and reported in a receipt. That is a real risk to name, which is why the skill insists on asking first, but it is a bounded one.

## What the plugin will not do

- Terminate live agents.
- Apply without `confirm=true`.
- Rewrite `AGENTS.md`, `*.mdc` or `rules`.
- Open secrets or SQLite.

The full layout and install notes are in [docs/PLUGIN.md](https://github.com/Weavatrix/sweeploom/blob/main/docs/PLUGIN.md), and the per-host MCP setup, including Codex, is on the [MCP page](/mcp/#codex).
