# FAQ

[README](../README.md) · [All docs](../README.md#documentation)

**Weavatrix is the product?** No. Company. SweepLoom is the product.

**Is SweepLoom on crates.io or npm?** Not yet. Both registries return
404 for `sweeploom`, and there are no prebuilt GitHub releases. Install
from source: `cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom`
([INSTALL.md](INSTALL.md)).

**Test with SweepLoom vs without?** `sweeploom bench` and the
`with_without` tests. Without it the gold store is 238M tok (folder
dump) or 965k (History as prompt); with it **16_288**, agents Keep,
apply without confirm deletes 0.

**Does that mean coding agents spend fewer tokens?** Not by itself.
Those are fixed metadata-policy fixtures, not actual agent spend. The
2026-09-15 live-MCP coding run measured **+75.5%** estimated cumulative
spend and **+23.1%** summed peak context on comparable T1/T2 rows
([BENCHMARKS-LIVE.md](BENCHMARKS-LIVE.md)). Use Cortex Loom when the goal
is budgeted repository evidence.

**Kill idle Cursor?** No. Keep. Context is the point.

**Delete `history.jsonl` via `--apply`?** Not on the SAFE generated
path. Inspect-only.

**`park?` disabled my skills?** No.

**`clean .` printed `no generated candidates`?** The walk found no
SAFE generated rows (this repo often has `target` outside the tree).
That is success, not a broken CLI.

**Bare `sweeploom` hung?** It should print help. MCP serve is
`sweeploom mcp`.

**Where is `sweeploom-mcp`?** A Cargo install has no such binary. Hosts
run `sweeploom` with the argument `mcp` ([MCP.md](MCP.md)).
`sweeploom-mcp` is only the alias of the future npm launcher.

**Does SweepLoom phone home?** No telemetry. The CLI and MCP server make
no network requests. The desktop app talks only to a local Ollama server
on `127.0.0.1:11434` (to list and remove models), and the browser
companion uses the browser's native-messaging channel to a local process
([BROWSER.md](BROWSER.md)).

**Website?** [sweeploom.com](https://sweeploom.com). Source, issues, and
these docs live in this GitHub repo.

**Web UI?** Later, separate product.
