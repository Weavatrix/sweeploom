---
title: "The always-on prompt tax: what your AGENTS.md and skills actually cost"
date: "2026-10-08"
summary: "Your ~/.claude folder is almost a gigabyte, but almost none of it is sent to the model. How SweepLoom separates always-on context from archive, why history counts as zero, how the estimate is capped, and what a live benchmark says about the limits of this number."
tags: ["ai", "tokens", "context", "benchmarks"]
---

Ask an agent why your coding assistant feels expensive and it will often do something like this: measure `~/.claude` or `~/.codex`, divide the bytes by four, and announce how many tokens of "context" you are carrying.

On a real workstation in September that folder was 875 MB for Claude and 8.7 GB for Codex. Divide by four and you get hundreds of millions of tokens. The number is enormous, alarming, and wrong. This post is about the much smaller number that is actually paid every turn, and about what that number does and does not tell you.

## Always-on versus archive

An agent host injects some files into every conversation whether you mention them or not: `AGENTS.md`, rule files such as `*.mdc`, the descriptions in a `skills` index, plugin manifests. That is the **always-on** context, a tax paid on every turn.

Everything else in those folders is something else:

- **History**: `history.jsonl`, `projects`, `sessions`, `archived_sessions`. Transcripts of past chats. They sit on disk. The host does not replay them into each prompt.
- **Cache**, logs and temporary files.
- **Secrets**: `.credentials.json`, `token.txt`.
- **SQLite** databases: `*.sqlite`, `state.vscdb`.
- **Settings**: `mcp.json`, `settings.json`.

Here is part of what `sweeploom ai` printed on that machine:

```text
[ ] AI claude · C:\Users\SergiiZiborov\.claude files=4265 size=875.7 MB capped=false inspect-only
    [ ] History projects files=3165 size=862.3 MB inspect-only
    [ ] Context plugins files=1019 size=10.7 MB ~4096 tok keep inspect-only
    [ ] Cache cache files=2 size=412.5 KB cleanable
    [ ] Secret .credentials.json files=1 size=10.5 KB inspect-only
    [ ] History history.jsonl files=1 size=10.4 KB inspect-only
    [ ] Context skills files=2 size=7.2 KB ~512 tok park? inspect-only
```

862 MB of that store is History. Its always-on cost is **zero tokens**. Billing it as if the host injected it every turn is how you arrive at a fake 500,000-token tax.

## How the estimate works

SweepLoom classifies each entry by its **leaf name only**. It never opens the file, which is also why it can look at a credentials file without reading a credential.

| Leaf | Class | Cleanable | Always-on tokens |
| --- | --- | --- | --- |
| `AGENTS.md`, `*.mdc`, `rules` | Context | no | `min(bytes / 4, 8000)` per file |
| `skills`, `plugins`, `rules` directories | Context | no | capped at **4096** |
| `history.jsonl`, `projects`, `sessions` | History | no | 0 |
| `cache`, `tmp`, `CachedData` | Cache | yes | 0 |
| `.credentials.json`, `token.txt` | Secret | no | 0 |
| `*.sqlite`, `*.vscdb` | SQLite | no | 0 |
| `*.log`, `.last-cleanup` | Log | yes | 0 |
| `password-reset.md`, `cachet.json` | Other | no | 0 |

The directory cap exists because a host does not inject the body of every skill. It injects an index: names and descriptions. An 80 KB `skills` tree is therefore estimated at 4,096 tokens, not 20,000. A single always-on file is capped at 8,000 tokens per file.

The last row is there on purpose. Naive substring rules call `password-reset.md` a secret and `cachet.json` a cache. Leaf-aware rules get both right, and get `AGENTS.md` and `skills` right as Context instead of "other junk".

## The gold store

The numbers are locked in a fixed fixture, the "gold store", so they cannot drift without a failing test:

```text
  AGENTS.md            12000 B  class=Context  old=3000 tok    now=3000 tok
  always-on.mdc         4000 B  class=Context  old=1000 tok    now=1000 tok
  skills               80000 B  class=Context  old=20000 tok   now=4096 tok
  plugins              40000 B  class=Context  old=10000 tok   now=4096 tok
  rules                24000 B  class=Context  old=6000 tok    now=4096 tok
  history.jsonl      2000000 B  class=History  old=500000 tok  now=0 tok
  projects            500000 B  class=History  old=125000 tok  now=0 tok
  archived_sessions  1200000 B  class=History  old=300000 tok  now=0 tok
  cache            900000000 B  class=Cache    old=0 tok       now=0 tok
SUM always-on tokens: old(overcount)=965000  now=16288  history_dropped=925000
```

Three ways of answering "how much context is this?" on the same store:

```text
WITHOUT dump-store/4          238466250 tok
WITHOUT History-as-prompt     965000 tok
WITH    always-on estimate    16288 tok
```

The first line is the folder divided by four, cache included. The second skips the cache but still bills History. The third is SweepLoom's estimate of what is actually always on. The test fails if the estimate ever reaches the History-as-prompt figure, if any Context entry becomes cleanable, or if History, Cache or Secret entries are given a non-zero cost.

## Advice, not edits

The tax is real, so SweepLoom gives advice about it. It does not act on it.

- `AGENTS.md`, `*.mdc` and `rules` are **keep**, even if untouched for 90 days. They are your project's contract with the agent.
- A `skills` or `plugins` tree idle for **30 days or more** is marked **park?**: a suggestion that you might move it out of the always-on path.
- History gets no advice at all. It is not a prompt cost.

`park?` is advice in the most literal sense. SweepLoom never flips `alwaysApply`, never edits a rule file, and the MCP server has no `park_file` or `rewrite_agents` tool. If you want to park a skill, you do it.

## No model on this path

Why not ask an LLM to classify these files? It was measured. On 25 names, a fast hosted coding model took 9.76 seconds and 18,128 tokens with a JSON schema, and 17.17 seconds freeform. The local Rust classifier takes about 2 µs per call, scores 30 out of 30 on the gold names (the old substring rules managed 24), and costs nothing. On the one path that must be predictable, the deterministic option is better on every axis.

## What this number is not

This is the part that is easy to leave out of a launch post, so it goes in bold: **16,288 is not a promise that your agent becomes cheaper.**

It is a fixed policy fixture that compares ways of *counting* a store. It is not provider billing, and it does not mean an ordinary coding task suddenly costs 99.99% less.

We checked. On 2026-09-15, five models did real coding work twice each, finding a real bug and removing duplicated classifier logic, once without SweepLoom and once in clean, isolated clones with the live SweepLoom MCP connected. Measured as transcript estimates rather than billing:

| | Without | With live MCP | Delta |
| --- | ---: | ---: | ---: |
| Total estimated spend | 6,115,798 | 10,733,758 | **+75.5%** |
| Summed peak context | 438,734 | 539,916 | **+23.1%** |

There was **no coding-task token saving**. The with-MCP prompts were longer and some needed feedback, so the delta is not a clean causal cost estimate, but it is direct evidence against claiming a saving. Agents called `disk_inventory` in every run; it helped confirm scope, and repeated calls added context. SweepLoom is not a context compressor. It does not intercept requests or shrink what an agent reads. If budgeted repository evidence is what you need, that is a different tool: the separate Cortex Loom project.

The same run did produce four real fixes, now covered by regression tests, which is a better use of a benchmark than a slide.

## Using it

```text
$ sweeploom ai       # your stores, classes and always-on estimates
$ sweeploom bench    # the fixture comparison above, re-run locally
```

Or from an agent, naming the product so the model uses the tool instead of guessing:

```text
Use SweepLoom to list_ai_stores and show always-on token tax.
Do not park AGENTS.md or rules.
```

Then act on the Context rows that are yours to act on: keep `AGENTS.md` lean, prune skills you have stopped using, and stop worrying about the gigabyte of history. It costs disk, not tokens.
