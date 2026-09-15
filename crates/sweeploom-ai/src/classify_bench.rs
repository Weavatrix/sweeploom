//! Gold-set accuracy vs the previous substring rules, plus a microbench.

use crate::classify::{
    AiClass, classify_name, is_history, is_log, is_sqlite, leaf_name, naive_is_secret,
};

/// Snapshot of classify_name before the Secret/Cache/Context tightening.
///
/// Secret uses the shared naive substring rule. Sqlite / log / history match
/// production. Cache and settings stay on the old looser branches (no Context).
pub(crate) fn classify_legacy(name: &str) -> AiClass {
    let leaf = leaf_name(name);
    let leaf = leaf.as_str();
    if naive_is_secret(leaf) {
        AiClass::Secret
    } else if is_sqlite(leaf) {
        AiClass::Sqlite
    } else if matches!(
        leaf,
        "cache"
            | "caches"
            | "cacheddata"
            | "tmp"
            | "temp"
            | "statsig"
            | "mcp-needs-auth-cache.json"
    ) || leaf.contains("cache")
    {
        AiClass::Cache
    } else if is_log(leaf) {
        AiClass::Log
    } else if is_history(leaf) {
        AiClass::History
    } else if leaf.contains("settings") || leaf.contains("config") || leaf == "argv.json" {
        AiClass::Settings
    } else {
        AiClass::Other
    }
}

const GOLD: &[(&str, AiClass)] = &[
    (".credentials.json", AiClass::Secret),
    ("auth.json", AiClass::Secret),
    ("token.txt", AiClass::Secret),
    ("password.json", AiClass::Secret),
    ("password-reset.md", AiClass::Other),
    ("state.vscdb", AiClass::Sqlite),
    ("foo.sqlite", AiClass::Sqlite),
    ("foo.db-wal", AiClass::Sqlite),
    ("cache", AiClass::Cache),
    ("CachedData", AiClass::Cache),
    ("mcp-needs-auth-cache.json", AiClass::Cache),
    ("statsig", AiClass::Cache),
    ("tmp", AiClass::Cache),
    ("cachet.json", AiClass::Other),
    ("debug", AiClass::Log),
    (".last-cleanup", AiClass::Log),
    ("logs", AiClass::Log),
    ("history.jsonl", AiClass::History),
    ("projects", AiClass::History),
    ("archived_sessions", AiClass::History),
    ("transcripts", AiClass::History),
    ("remote-settings.json", AiClass::Settings),
    ("argv.json", AiClass::Settings),
    ("config.toml", AiClass::Settings),
    ("AGENTS.md", AiClass::Context),
    ("always-on.mdc", AiClass::Context),
    ("skills", AiClass::Context),
    ("plugins", AiClass::Context),
    ("unknown-blob.bin", AiClass::Other),
    ("session-store.json", AiClass::Other),
];

fn hits(classify: fn(&str) -> AiClass) -> usize {
    GOLD.iter()
        .filter(|(name, gold)| classify(name) == *gold)
        .count()
}

fn false_clean_on_secret(classify: fn(&str) -> AiClass) -> usize {
    GOLD.iter()
        .filter(|(name, gold)| *gold == AiClass::Secret && classify(name).can_clean())
        .count()
}

#[test]
fn bench_classify_accuracy_and_speed() {
    let old = hits(classify_legacy);
    let new = hits(classify_name);
    let old_leak = false_clean_on_secret(classify_legacy);
    let new_leak = false_clean_on_secret(classify_name);
    eprintln!(
        "classify gold {}/{}  legacy {}/{}  secret-false-clean legacy={} now={}",
        new,
        GOLD.len(),
        old,
        GOLD.len(),
        old_leak,
        new_leak
    );
    for (name, gold) in GOLD {
        let was = classify_legacy(name);
        let now = classify_name(name);
        if was != now {
            eprintln!("  delta {name}: {was:?} -> {now:?} (gold {gold:?})");
        }
    }
    assert_eq!(new_leak, 0, "must never mark a gold Secret as cleanable");
    assert!(
        new > old,
        "expected accuracy to rise: now {new} vs legacy {old}"
    );

    let names: Vec<&str> = GOLD.iter().map(|(name, _)| *name).collect();
    let warmup = names.iter().fold(0_u64, |acc, name| {
        acc.wrapping_add(u64::from(classify_name(name).can_clean()))
    });
    let rounds = 50_000_u32;
    let start = std::time::Instant::now();
    let mut sink = warmup;
    for _ in 0..rounds {
        for name in &names {
            sink = sink.wrapping_add(u64::from(classify_name(name).can_clean()));
        }
    }
    let elapsed = start.elapsed();
    let calls = u64::from(rounds) * names.len() as u64;
    let ns = elapsed.as_nanos() / u128::from(calls);
    eprintln!("classify_name {calls} calls in {elapsed:?} (~{ns} ns/call) sink={sink}");
    assert!(
        ns < 50_000,
        "classify_name got slower than 50µs/call: {ns} ns"
    );
}

#[test]
fn naive_secret_matches_and_remaining_branches_stay_distinct() {
    use crate::with_without_bench::classify_without;

    for name in [
        ".credentials.json",
        "auth.json",
        "token.txt",
        "password.json",
        "password-reset.md",
    ] {
        assert_eq!(classify_legacy(name), classify_without(name), "{name}");
        assert_eq!(classify_legacy(name), AiClass::Secret, "{name}");
    }

    assert_eq!(classify_legacy("state.vscdb"), AiClass::Sqlite);
    assert_eq!(classify_without("state.vscdb"), AiClass::Other);
    assert_eq!(classify_legacy("archived_sessions"), AiClass::History);
    assert_eq!(classify_without("archived_sessions"), AiClass::Other);
    assert_eq!(classify_legacy("statsig"), AiClass::Cache);
    assert_eq!(classify_without("statsig"), AiClass::Other);
    assert_eq!(classify_legacy("debug"), AiClass::Log);
    assert_eq!(classify_without("debug"), AiClass::Other);
    assert_eq!(classify_legacy("argv.json"), AiClass::Settings);
    assert_eq!(classify_without("argv.json"), AiClass::Other);
}
