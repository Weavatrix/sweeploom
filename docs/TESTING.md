# Tests

[README](../README.md) · [All docs](../README.md#documentation)

Captured 2026-09-14, `--lib` counts (not the GUI):

| Crate | What it locks | Command |
| --- | --- | --- |
| `sweeploom` | 11 tests: MCP refuse, with/without card, catalog, plugin JSON, session flags, `--chromium-id` | `cargo test -p sweeploom --lib` |
| `sweeploom` doctest | 2 rustdoc examples (`classify_name`, `advise_one`) | `cargo test -p sweeploom --doc` |
| `sweeploom-ai` | 16: classify gold, inventory caps, token/classify/with-without benches | `cargo test -p sweeploom-ai --lib` |
| `sweeploom-session` | 25: detectors, Keep/orphan, grouping + with/without benches | `cargo test -p sweeploom-session --lib` |
| `sweeploom-core` | 16: safety vs recommendation, redact, artifact refuse | `cargo test -p sweeploom-core --lib` |

`sweeploom --lib` names:

```text
api::tests::mcp_apply_without_confirm_does_not_run
cmd_bench::tests::bench_with_without_card
api::tests::advise_does_not_claim_history
surface::tests::cli_apply_is_flag_not_mcp_phrase
surface::tests::identities_split_hands_and_agents
surface::tests::mcp_writes_only_through_confirm
surface::tests::published_mcp_surface
surface::tests::weavatrix_marketplace_and_codex_plugin_parse
cmd_sessions::tests::parses_plan_flags
cmd_companion_install::tests::parse_reads_chromium_id_flag
cmd_companion_install::tests::parse_without_flag_is_none
```

`advise_does_not_claim_history` asserts `history.jsonl` advice is empty
and `AGENTS.md` is `keep`.

`weavatrix_marketplace_and_codex_plugin_parse` reads
`.agents/plugins/marketplace.json` and
`plugins/sweeploom/.codex-plugin/plugin.json` so the Codex layout
cannot drift silently.

Session policy tests (same crate as the benches):

```text
forgotten::tests::idle_cursor_keeps_context
forgotten::tests::idle_light_agent_is_likely_forgotten_but_kept
forgotten::tests::unknown_idle_claude_is_keep
forgotten::tests::browser_tree_is_never_auto_reclaimed
forgotten::tests::generic_app_is_never_auto_recommended
tests::mcp_child_of_cursor_is_attached_not_orphan
tests::stray_mcp_is_orphan_candidate
```

Classify / safety tests:

```text
classify::tests::secrets_and_sqlite_cannot_clean
classify::tests::docs_are_not_secrets_and_cachet_is_not_cache
classify::tests::history_and_settings_stay_inspect
safety::tests::blockers_force_blocked_level
recommendation::tests::recommendation_cannot_bypass_blocker
redaction::tests::redacts_flag_value_and_next_token
artifact::tests::refuses_src_root_dot_and_escape
artifact::tests::refuses_nested_git
```

CI (`.github/workflows/ci.yml`): `fmt`, `clippy -D warnings`,
`cargo test --workspace --locked`, `cargo doc -D warnings`,
`cargo deny`, line budget. Platform matrix: Ubuntu, Windows, macOS.

More tables: [BENCHMARKS.md](BENCHMARKS.md).
