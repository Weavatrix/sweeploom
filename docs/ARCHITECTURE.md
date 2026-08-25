# SweepLoom architecture

SweepLoom is a local-first workstation resource manager. The full product
plan lives in [`product/`](product/README.md), split so every file stays
under 300 lines.

## License boundary

| Code | License |
| --- | --- |
| SweepLoom (`Weavatrix/sweeploom`) | MPL-2.0 |
| Weavatrix crates (`weavatrix-scan`, `weavatrix-git`, …) | MIT — **do not relicense** |

Missing Git/scan APIs are implemented in those crates and consumed here.

SweepLoom depends on Weavatrix libraries. It does not vendor, fork, or
relicense them.

## Crate map

```text
sweeploom-core        no OS APIs, no egui
sweeploom-platform    paths, trash, process control
sweeploom-process     sysinfo snapshots
sweeploom-session     logical grouping + forgotten score
sweeploom-network     capability-gated connections
sweeploom-storage     weavatrix-scan inventory
sweeploom-exec        plan / revalidate / receipt
sweeploom-gui         egui + eframe (glow)
```

## Invariants

- Safety and Recommendation are independent axes.
- Recommendation never bypasses a blocker.
- Processes are keyed by `PID + start time`.
- Command lines are redacted before UI / logs / receipts.
- No LLM on the destructive path.
- No default telemetry.
