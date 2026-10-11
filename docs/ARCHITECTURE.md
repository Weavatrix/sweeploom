# SweepLoom architecture

[README](../README.md) · [All docs](../README.md#documentation)

SweepLoom is a local-first workstation resource manager published by
**Weavatrix** (the company). SweepLoom is the product. The full product
plan lives in [`product/`](product/README.md), split so every file stays
under 300 lines.

Agent-facing UI is the [Codex plugin](PLUGIN.md) (MCP + skills). The egui
desktop app is an optional local reviewer. A web UI is a later product.

## License boundary

| Code | License |
| --- | --- |
| SweepLoom (`Weavatrix/sweeploom`) | MPL-2.0 |
| Other Weavatrix libraries (`weavatrix-scan`, `weavatrix-git`, …) | MIT — **do not relicense** |

Missing Git/scan APIs live in those libraries and are consumed here.
SweepLoom does not vendor, fork, or relicense them.

## Crate map

```text
sweeploom-core        no OS APIs, no egui
sweeploom-platform    paths, trash, process control
sweeploom-process     sysinfo snapshots
sweeploom-session     logical grouping + forgotten score
sweeploom-network     capability-gated connections
sweeploom-storage     weavatrix-scan inventory
sweeploom-exec        plan / revalidate / receipt
sweeploom-dev         Cargo / Node / Python workspace analyzers
sweeploom-ai          inspect-first AI stores, classify, token tax
sweeploom-general     temp, logs, dumps, Downloads review
sweeploom-rules       declarative TOML cleaner rules
sweeploom-browser     companion protocol: tab heat, discard, Later
sweeploom-history     bounded in-memory observed history
sweeploom (cli)       CLI, MCP server, Rust library facade
sweeploom-gui         egui + eframe (glow)
```

## Invariants

- Safety and Recommendation are independent axes.
- Recommendation never bypasses a blocker.
- Processes are keyed by `PID + start time`.
- Command lines are redacted before UI / logs / receipts.
- No LLM on the destructive path.
- No default telemetry.
