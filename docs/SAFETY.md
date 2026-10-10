# Safety model and classes

[README](../README.md) · [All docs](../README.md#documentation)

## Apply: plan → revalidate → execute → receipt

Apply is always `plan → revalidate → execute → receipt`. If you rebuilt
`target` after the plan, that row is skipped, not forced.

- **CLI:** `sweeploom clean .` is a dry run. `--apply` **is** consent
  ([CLI-CLEAN.md](CLI-CLEAN.md#8-clean--look-then---apply)).
- **MCP:** `apply_cleanup` needs `confirm: true`, sent only after the user
  agreed to the exact `ids` ([MCP-CHATS.md](MCP-CHATS.md)).
- **Desktop app:** cleanup goes through a confirmation; file cleanup
  revalidates live paths and metadata first
  ([CLEANUP.md](CLEANUP.md)).

Generated cleanup (`target/`, `node_modules/`, tool caches) is a
permanent delete of regenerable output (`"deletion": "PermanentGenerated"`).
In the desktop Cleanup screen, user data such as downloads, archives,
backups, and downloaded model files goes to the Trash instead; Trash can be
restored until you empty it, and its space stays occupied until then.
Docker objects, iOS Simulator devices, Ollama models, and Rust toolchains
are removed through their own native tools.

## Safety

- Recommendation never overrides a blocker.
- Apply: plan → revalidate → execute → receipt.
- Secrets / SQLite / History / `AGENTS.md` / `*.mdc` / `rules` are
  inspect-only.
- `park?` is advice. No `alwaysApply` writes.
- Live agents Keep. Attached MCP Keep. CLI/MCP do not terminate.
- Command lines redacted (`--token`, URI userinfo).
- Process key = `PID + start time`.
- `deny.toml` bans `tokio`, `hyper`, `reqwest`.

Only SAFE generated rows can be pre-selected (`[x]`) in a cleanup plan.
AI stores, History, secrets, user data, and live agents are never
pre-selected, and a pre-selected row still needs `--apply`,
`confirm: true`, or your confirmation in the app.

The desktop app is the only surface that can stop a process: you pick a
session, press **Terminate session**, and confirm. A graceful stop comes
first; force-kill is never automatic. A project with Git changes shows a
warning (stopping a session does not discard them). Browser trees cannot be
terminated from there.

## What the classes mean

`classify_name` uses the **leaf** only. It never opens the file.

| Leaf | Class | Clean? | Always-on tokens |
| --- | --- | --- | --- |
| `AGENTS.md`, `*.mdc`, `rules` | Context | no | `min(bytes/4, 8000)`; dirs `skills`/`plugins`/`rules` cap **4096** |
| `history.jsonl`, `projects`, `sessions` | History | no | **0** |
| `cache`, `tmp`, `CachedData` | Cache | yes | 0 |
| `.credentials.json`, `token.txt` | Secret | no | 0 |
| `*.sqlite`, `*.vscdb` | SQLite | no | 0 |
| `*.log`, `.last-cleanup` | Log | yes | 0 |
| `mcp.json`, `settings.json` | Settings | no | 0 |
| `password-reset.md`, `cachet.json` | Other | no | 0 |

Gold-store sum in tests: old History-as-prompt **965_000** tok vs now
**16_288**. Re-run: `cargo test -p sweeploom-ai bench_token_context_weight -- --nocapture`.

How a row reads in practice: [CLI.md](CLI.md#5-ai-stores--token-tax-not-delete-claude).

## MCP gate

```text
$ cargo test -p sweeploom mcp_apply_without_confirm_does_not_run -- --nocapture
```

`apply_cleanup({ confirm: false, ... })` → `ok=false`, `deleted=0`,
error contains `confirm=true`.

A bad agent that skips review gets this refusal; see
[Chat B](MCP-CHATS.md#chat-b--agent-tries-to-skip-review-must-fail).

## Invariants

The engine-level invariants (independent Safety and Recommendation axes,
`PID + start time` keys, redaction, no LLM on the destructive path, no
default telemetry) are listed in [ARCHITECTURE.md](ARCHITECTURE.md).
