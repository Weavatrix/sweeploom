//! Agent questions used to compare MCP-told / MCP-silent / CLI.
//!
//! Silent = SweepLoom is **connected** (tools in the catalog) but the user
//! never says the product name. A host still lists `list_ai_stores`. A model
//! that ignores the catalog will shell-dump `~/.claude` instead.

/// One user question.
#[derive(Clone, Copy, Debug)]
pub struct Question {
    /// Stable id.
    pub id: &'static str,
    /// What the human types. Silent variants must not mention SweepLoom.
    pub ask: &'static str,
    /// Why this question exists.
    pub why: &'static str,
    /// What good looks like.
    pub good: &'static str,
    /// MCP tools a model that *notices* the catalog should call, in order.
    pub mcp_if_noticed: &'static [&'static str],
    /// CLI verbs for the same job (`sweeploom <verb>`).
    pub cli: &'static [&'static str],
    /// Typical tools when the catalog is ignored.
    pub if_ignored: &'static [&'static str],
}

/// Three independent coding asks. Every model/lane pair gets a fresh context.
pub const QUESTIONS: &[Question] = &[
    Question {
        id: "find_fix_bug",
        ask: "Find and fix one real bug or dead production path in crates/sweeploom-cli. A warning-only cleanup does not count. Add a regression test that fails before the fix, explain the failure mode, and run cargo test -p sweeploom --lib.",
        why: "This is a frequent open-ended debugging task.",
        good: "A concrete production path changes, a regression test covers it, and CLI tests pass.",
        mcp_if_noticed: &["disk_inventory"],
        cli: &["scan"],
        if_ignored: &["Shell", "Read"],
    },
    Question {
        id: "remove_duplicate",
        ask: "Find, verify, and eliminate duplicate classifier logic in crates/sweeploom-ai. Reuse one shared implementation without changing benchmark intent, then run cargo test -p sweeploom-ai --lib.",
        why: "This is a frequent duplicate-detection and refactoring task.",
        good: "Both benchmark modules use one shared implementation and all AI tests pass.",
        mcp_if_noticed: &["disk_inventory"],
        cli: &["scan"],
        if_ignored: &["Shell", "Read"],
    },
    Question {
        id: "split_api",
        ask: "Split the 506-line crates/sweeploom-cli/src/api.rs into cohesive modules so api.rs is under 300 lines. Preserve public exports and behavior, run cargo test -p sweeploom --lib, and create one focused local commit. Do not push and do not use --no-verify.",
        why: "This is the large refactoring task.",
        good: "api.rs is under 300 lines, extracted modules compile, tests pass, and one focused local commit exists.",
        mcp_if_noticed: &["disk_inventory"],
        cli: &["scan"],
        if_ignored: &["Shell", "Read"],
    },
];

/// True when the ask never names the product. Catalog can still be connected.
#[must_use]
pub fn is_silent_ask(ask: &str) -> bool {
    let lower = ask.to_ascii_lowercase();
    !ask.contains("SweepLoom")
        && !lower.contains("sweeploom.exe")
        && !lower.contains("sweeploom-mcp")
        && !lower.contains("use sweeploom")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{TOOLS, mcp_tools};

    #[test]
    fn cases_json_matches_rust_asks() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/agent_cases.json");
        let raw = std::fs::read_to_string(&path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let asks = json["asks"].as_array().unwrap();
        assert_eq!(asks.len(), QUESTIONS.len());
        for (question, row) in QUESTIONS.iter().zip(asks) {
            assert_eq!(row["id"], question.id);
            assert_eq!(row["text"], question.ask);
        }
    }

    #[test]
    fn silent_asks_never_name_the_product() {
        for question in QUESTIONS {
            assert!(
                is_silent_ask(question.ask),
                "{} names SweepLoom; silent lane would leak the hint: {}",
                question.id,
                question.ask
            );
            assert!(!question.why.is_empty(), "{}", question.id);
            assert!(!question.good.is_empty(), "{}", question.id);
        }
    }

    #[test]
    fn noticed_tools_are_real_mcp() {
        let ids: Vec<_> = mcp_tools().map(|tool| tool.id).collect();
        for question in QUESTIONS {
            for tool in question.mcp_if_noticed {
                assert!(ids.contains(tool), "{tool} missing from MCP catalog");
            }
            for verb in question.cli {
                assert!(
                    TOOLS.iter().any(|tool| tool.cli == Some(*verb)),
                    "no CLI verb {verb}"
                );
            }
        }
    }

    #[test]
    fn ignored_catalog_is_not_sweeploom() {
        for question in QUESTIONS {
            assert!(
                question.if_ignored.contains(&"Shell"),
                "{} should show a Shell fallback when the model ignores MCP",
                question.id
            );
            assert!(
                !question.if_ignored.contains(&"list_ai_stores"),
                "ignored lane must not already be SweepLoom"
            );
        }
    }
}
