<div align="center">

# SweepLoom

### Reclaim your workstation without losing your workspace

[![CI](https://github.com/Weavatrix/sweeploom/actions/workflows/ci.yml/badge.svg)](https://github.com/Weavatrix/sweeploom/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MPL--2.0-orange)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.88%2B-000000?logo=rust)](https://www.rust-lang.org/)

**SweepLoom — by Weavatrix** · [sweeploom.com](https://sweeploom.com)

</div>

SweepLoom is a local-first **workspace resource cleaner**. It maps disk,
processes, projects, terminals, AI agent sessions, and browser tabs into one
evidence-driven workstation model — then lets you reclaim space, RAM, and
background CPU without guessing.

It is **not** a RAM booster, PC optimizer, or registry cleaner. Destructive
actions are deterministic, inspectable, and never require an LLM.

## What it understands

```text
STORAGE                         LIVE RESOURCES
  projects                        forgotten terminals
  build artifacts                 Claude Code / Codex sessions
  dependency trees                MCP servers
  package/tool caches             Node / Vite / Cargo / Python
  AI histories                    listening ports
  temp / logs / dumps             observed CPU / RSS / I/O
  Downloads                       browser tab heat (optional)
```

## Safety is not a recommendation

Every candidate has two independent axes:

| Safety | Recommendation |
| --- | --- |
| `SAFE` `LOW_RISK` `REVIEW` `DANGEROUS` `BLOCKED` | `STRONGLY_RECOMMENDED` `RECOMMENDED` `OPTIONAL` `KEEP` |

A recommendation score never overrides a safety blocker. Cleanup goes through
`plan → review → revalidate → execute → verify → receipt`.

## Workspace

```text
crates/
  sweeploom-core        OS-neutral data model
  sweeploom-platform    paths, trash, process control, disk
  sweeploom-process     sysinfo snapshots, trees, history
  sweeploom-session     logical sessions, detectors, forgotten score
  sweeploom-network     connections / listeners (capability-gated)
  sweeploom-storage     weavatrix-scan inventory, folder inspector
  sweeploom-rules       declarative TOML cleaners
  sweeploom-exec        CleanPlan + revalidation + receipts
  sweeploom-dev         Cargo / Node / Python analyzers
  sweeploom-ai          Claude / Codex storage (inspect-first)
  sweeploom-general     temp / logs / Downloads
  sweeploom-browser     native-messaging companion protocol
  sweeploom-history     bounded observed history
  sweeploom-cli         scan / sessions / plan
apps/
  sweeploom-gui         egui + eframe (glow)
```

Weavatrix libraries (`weavatrix-scan`, `weavatrix-git`) stay **MIT** in their
own repositories. SweepLoom depends on them; it does not vendor or relicense
them.

## Run

```text
cargo run -p sweeploom-gui
cargo run -p sweeploom-cli -- sessions
cargo run -p sweeploom-cli -- scan <path>
cargo run -p sweeploom-cli -- projects <path>
cargo run -p sweeploom-cli -- clean <path>
cargo run -p sweeploom-cli -- clean <path> --apply
```

## Principles

- Local-first. No account, no cloud scan, no file upload.
- Evidence before action.
- PID reuse protection (`PID + start time`).
- Command lines are redacted before UI / logs / receipts.
- Telemetry default: none.

## License

SweepLoom source is **MPL-2.0**. See [`LICENSE`](LICENSE).

Weavatrix crates remain **MIT** — do not change those licenses when consuming
them from this product.
