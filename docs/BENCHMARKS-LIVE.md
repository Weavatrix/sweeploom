# Live coding-agent check (2026-09-15)

[README](../README.md) · [All docs](../README.md#documentation)

The fixture benchmarks in [BENCHMARKS.md](BENCHMARKS.md) measure
classification policy. A separate run checked actual coding work: find a
real bug and remove duplicate classifier logic. Five models ran once without SweepLoom and again in
clean isolated clones with the live SweepLoom MCP. Each clone used its
own Cargo target. Earlier static-capsule “with” runs were invalid and
are excluded.

Token values below are reproducible transcript estimates
(`characters / 4`, including reconstructed tool results), not provider
billing:

| Task | Without total spend | Live MCP total spend | Delta | Without peak sum | Live MCP peak sum | Delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Real bug | 3,636,327 | 6,966,918 | **+91.6%** | 252,759 | 305,750 | **+21.0%** |
| Duplicate logic | 2,479,471 | 3,766,840 | **+51.9%** | 185,975 | 234,166 | **+25.9%** |
| **Comparable total** | **6,115,798** | **10,733,758** | **+75.5%** | **438,734** | **539,916** | **+23.1%** |

This run found **no coding-task token saving**. The live-MCP prompts were
longer and required feedback, so the delta is not a clean causal cost
estimate; it is still direct evidence against claiming a saving here.

Observed use:

- All ten live-MCP runs called `disk_inventory`; some also called
  `list_projects` or `advise_context`.
- Inventory helped confirm the clean root and coarse crate scope.
- Source directories frequently remained `Unknown`; the tool exposed no
  file/symbol map, semantic duplicate finder, or CLI-to-source map.
- Agents still found and proved the code changes through normal source
  reads and tests. Repeated inventory calls added context.
- T1 produced five independently rebuilt, passing fixes. All five T2
  agents reported passing tests on unique targets; the run was stopped
  before a separate parent re-verification. The live-MCP T3 split-file
  runs were never started and are not represented as zero rows.

The run also produced four source fixes now covered by regression tests:

1. stale browser companion snapshots no longer expose tab counts;
2. missing session option values no longer swallow the next flag;
3. inspect-only MCP tools publish `readOnlyHint`;
4. duplicate naive Secret logic in the benchmark classifiers is shared
   while intentionally different branches remain separate.

The benchmark canvas contains the row-level working report. Raw
transcripts and machine-specific `file_output/` artifacts are excluded
from releases.

Policy fixtures (token tax, classify gold, live agents Keep, MCP gate):
[BENCHMARKS.md](BENCHMARKS.md). For budgeted repository evidence, see the
scope note in [CONCEPTS.md](CONCEPTS.md#scope-sweeploom-is-not-a-context-compressor).
