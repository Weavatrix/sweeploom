# SweepLoom library

[README](../README.md) · [All docs](../README.md#documentation)

Crate: `sweeploom` (`crates/sweeploom-cli`). Same engine as the CLI and the
MCP server. Weavatrix is the company that publishes it.

The crate is **not on crates.io yet**. Depend on it from Git:

```toml
[dependencies]
sweeploom = { git = "https://github.com/Weavatrix/sweeploom" }
```

Prefer this facade. Internal crates (`sweeploom-core`, `sweeploom-session`,
…) are workspace members the facade depends on.

## Classify names (no I/O)

```rust
use sweeploom::{classify_name, AiClass};

assert_eq!(classify_name("AGENTS.md"), AiClass::Context);
assert_eq!(classify_name("AGENTS.md").label(), "Context");
assert_eq!(classify_name("always-on.mdc"), AiClass::Context);
assert_eq!(classify_name("skills"), AiClass::Context);
assert_eq!(classify_name("history.jsonl"), AiClass::History);
assert_eq!(classify_name("cache"), AiClass::Cache);
assert_eq!(classify_name(".credentials.json"), AiClass::Secret);
assert_eq!(classify_name("state.vscdb"), AiClass::Sqlite);
assert_eq!(classify_name("password-reset.md"), AiClass::Other);
assert_eq!(classify_name("mcp.json"), AiClass::Settings);

assert!(classify_name("cache").can_clean());
assert!(!classify_name("AGENTS.md").can_clean());
assert!(!classify_name(".credentials.json").can_clean());
assert!(classify_name("AGENTS.md").is_prompt_context());
assert!(!classify_name("history.jsonl").is_prompt_context());
```

## Token estimate (always-on only)

```rust
use sweeploom::{classify_name, estimated_prompt_tokens, estimated_prompt_tokens_with_files, AiClass};

assert_eq!(
    estimated_prompt_tokens(AiClass::Context, "AGENTS.md", 4_000),
    1_000
);
assert_eq!(
    estimated_prompt_tokens(AiClass::History, "history.jsonl", 2_000_000),
    0
);
assert_eq!(
    estimated_prompt_tokens_with_files(AiClass::Context, "skills", 80_000, 200),
    4_096
);

let class = classify_name("plugins");
let tokens = estimated_prompt_tokens_with_files(class, "plugins", 40_000, 80);
assert_eq!(tokens, 4_096);
```

## Advise context (never writes files)

```rust
use std::time::Duration;
use sweeploom::{advise_one, advise_context, classify_name, AiClass, ContextAdvice};

assert_eq!(
    advise_one("AGENTS.md", AiClass::Context, Some(Duration::from_secs(90 * 24 * 3600))),
    ContextAdvice::Keep
);
assert_eq!(
    advise_context("rules", AiClass::Context, Some(Duration::from_secs(90 * 24 * 3600))),
    ContextAdvice::Keep
);
assert_eq!(
    advise_context("skills", AiClass::Context, Some(Duration::from_secs(3 * 24 * 3600))),
    ContextAdvice::Keep
);
assert_eq!(
    advise_context("skills", AiClass::Context, Some(Duration::from_secs(40 * 24 * 3600))),
    ContextAdvice::SuggestPark
);
assert_eq!(
    advise_context("history.jsonl", classify_name("history.jsonl"), None),
    ContextAdvice::LeaveAlone
);
```

`SuggestPark` is advice. SweepLoom does not flip `alwaysApply`.

## List live sessions

```rust
use sweeploom::list_sessions;

let rows = list_sessions();
for row in rows.iter().take(8) {
    println!(
        "{} proc={} rss={} cpu={:.1}% rec={} {}",
        row.kind,
        row.processes,
        row.rss_bytes,
        row.cpu_percent,
        row.recommendation,
        row.project.as_deref().unwrap_or("-")
    );
}
```

Dry-run plans (still no terminate):

```rust
use sweeploom::list_sessions_plan;

let quiet = list_sessions_plan(None, None, true);
let ram = list_sessions_plan(Some(4.0), None, false);
let cpu = list_sessions_plan(None, Some(20.0), false);
let _ = (quiet, ram, cpu);
```

Command lines are omitted on purpose.

## Disk inventory

```rust
use std::path::Path;
use sweeploom::disk_inventory;

let report = disk_inventory(Path::new(".")).expect("scan");
println!(
    "root={} entries={} projects={} logical={}",
    report.root, report.entries, report.projects, report.logical_bytes
);
for child in report.children.iter().take(10) {
    println!("{} {} {}", child.logical_bytes, child.category, child.path);
}
```

## Projects

```rust
use std::path::Path;
use sweeploom::list_projects;

for project in list_projects(Path::new(".")).expect("projects") {
    println!(
        "{} kind={} source={} artifact={} git={}",
        project.path, project.kind, project.source, project.artifact, project.git
    );
}
```

## AI stores

```rust
use sweeploom::list_ai_stores;

for store in list_ai_stores() {
    println!("{} {} bytes={}", store.title, store.path, store.logical_bytes);
    for entry in store.entries.iter().take(12) {
        println!(
            "  [{}] {} {} tok={} advice={} clean={}",
            entry.id,
            entry.class,
            entry.relative,
            entry.prompt_tokens,
            entry.advice,
            entry.can_clean
        );
    }
}
```

## Cleanup candidates + explain

```rust
use std::path::Path;
use sweeploom::{cleanup_candidates, explain_candidate};

let root = Path::new(".");
let rows = cleanup_candidates(root);
for row in &rows {
    println!(
        "id={} selected={} blocked={} {} {} {}",
        row.id, row.selected, row.blocked, row.logical_bytes, row.title, row.path
    );
}
if let Some(one) = rows.first().and_then(|row| explain_candidate(root, row.id)) {
    println!("explained {} {}", one.id, one.path);
}
```

## Apply (you are the caller)

MCP must pass `confirm: true`. The CLI sets that from `--apply`. The
library does not invent a second phrase.

```rust
use std::path::PathBuf;
use sweeploom::{apply_cleanup, ApplyRequest};

let dry = apply_cleanup(ApplyRequest {
    confirm: false,
    root: PathBuf::from("."),
    ids: Vec::new(),
});
assert!(!dry.ok);
assert_eq!(dry.deleted, 0);

// Only when you mean it. Empty ids = pre-selected SAFE rows.
let rows = sweeploom::cleanup_candidates(std::path::Path::new("."));
if let Some(row) = rows.iter().find(|row| row.selected && !row.blocked) {
    let report = apply_cleanup(ApplyRequest {
        confirm: true,
        root: PathBuf::from("."),
        ids: vec![row.id],
    });
    let _ = report.deleted;
}
```

## Browser pressure

```rust
use sweeploom::list_browser;

let browser = list_browser();
println!(
    "companion={} hosts={} rss={} tabs={:?}",
    browser.companion, browser.hosts, browser.rss_bytes, browser.tabs
);
```

Tab counts are `None` until the companion extension has a fresh snapshot.
Missing tabs are not shown as zero.

## Catalog (CLI vs MCP)

```rust
use sweeploom::{mcp_tools, WriteClass, CLI_NAME, MCP_SERVER, PRODUCT, TOOLS};

assert_eq!(PRODUCT, "SweepLoom");
assert_eq!(CLI_NAME, "sweeploom");
assert_eq!(MCP_SERVER, "io.github.Weavatrix/sweeploom");
assert_eq!(mcp_tools().count(), 9);

for tool in TOOLS {
    let _ = (tool.id, tool.cli, tool.mcp, tool.writes);
}
for tool in mcp_tools() {
    assert!(matches!(
        tool.writes,
        WriteClass::Read | WriteClass::McpGated
    ));
}
```

## Bytes helper

```rust
use sweeploom::bytes::format_bytes;

assert_eq!(format_bytes(512), "512 B");
assert!(format_bytes(3_000_000).contains("MB"));
```

## Lower-level AI offers

```rust
use sweeploom::inspect_offers;
use sweeploom_platform::UserLocations;

let offers = inspect_offers(&UserLocations::current());
for offer in offers {
    let _ = (offer.title, offer.candidate.logical_bytes, offer.capped);
}
```

`inspect_offers` never pre-selects a store for delete.

## What this crate will not do

- Terminate processes
- Open secret / SQLite contents
- Rewrite `AGENTS.md`
- Call a network LLM on classify or apply
