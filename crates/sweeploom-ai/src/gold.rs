//! Fixed gold store for WITH vs WITHOUT benches. Not the user's disk.

use crate::classify::{AiClass, classify_name, estimated_prompt_tokens_with_files};

/// One leaf in the gold store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoldLeaf {
    /// File or directory name.
    pub name: &'static str,
    /// Logical bytes.
    pub bytes: u64,
    /// Child file count (dirs).
    pub files: u64,
    /// Expected class.
    pub class: AiClass,
}

/// Same leaves as `sweeploom bench` and the AI crate benches.
pub const GOLD_STORE: &[GoldLeaf] = &[
    GoldLeaf {
        name: "AGENTS.md",
        bytes: 12_000,
        files: 1,
        class: AiClass::Context,
    },
    GoldLeaf {
        name: "always-on.mdc",
        bytes: 4_000,
        files: 1,
        class: AiClass::Context,
    },
    GoldLeaf {
        name: "skills",
        bytes: 80_000,
        files: 200,
        class: AiClass::Context,
    },
    GoldLeaf {
        name: "plugins",
        bytes: 40_000,
        files: 80,
        class: AiClass::Context,
    },
    GoldLeaf {
        name: "rules",
        bytes: 24_000,
        files: 30,
        class: AiClass::Context,
    },
    GoldLeaf {
        name: "history.jsonl",
        bytes: 2_000_000,
        files: 1,
        class: AiClass::History,
    },
    GoldLeaf {
        name: "projects",
        bytes: 500_000,
        files: 40,
        class: AiClass::History,
    },
    GoldLeaf {
        name: "archived_sessions",
        bytes: 1_200_000,
        files: 20,
        class: AiClass::History,
    },
    GoldLeaf {
        name: "cache",
        bytes: 900_000_000,
        files: 10_000,
        class: AiClass::Cache,
    },
    GoldLeaf {
        name: ".credentials.json",
        bytes: 2_000,
        files: 1,
        class: AiClass::Secret,
    },
    GoldLeaf {
        name: "state.vscdb",
        bytes: 50_000_000,
        files: 1,
        class: AiClass::Sqlite,
    },
    GoldLeaf {
        name: "password-reset.md",
        bytes: 3_000,
        files: 1,
        class: AiClass::Other,
    },
];

/// Whole store billed as prompt (`bytes/4`).
pub const GOLD_DUMP: u64 = 238_466_250;
/// Context + History billed as prompt; cache/secrets/sqlite dropped.
pub const GOLD_HISTORY_AS_PROMPT: u64 = 965_000;
/// SweepLoom always-on estimate on this fixture.
pub const GOLD_WITH: u64 = 16_288;

/// WITH vs WITHOUT totals for the gold store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoldTax {
    /// Naive folder dump.
    pub dump: u64,
    /// History still billed.
    pub history_as_prompt: u64,
    /// SweepLoom always-on.
    pub with: u64,
}

/// Sum the gold store through the live classifier and caps.
#[must_use]
pub fn gold_tax() -> GoldTax {
    let mut dump = 0_u64;
    let mut history_as_prompt = 0_u64;
    let mut with = 0_u64;
    for leaf in GOLD_STORE {
        let class = classify_name(leaf.name);
        dump += leaf.bytes / 4;
        if matches!(class, AiClass::Context | AiClass::History) {
            history_as_prompt += leaf.bytes / 4;
        }
        with += estimated_prompt_tokens_with_files(class, leaf.name, leaf.bytes, leaf.files);
    }
    GoldTax {
        dump,
        history_as_prompt,
        with,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gold_tax_is_locked() {
        let tax = gold_tax();
        assert_eq!(tax.dump, GOLD_DUMP);
        assert_eq!(tax.history_as_prompt, GOLD_HISTORY_AS_PROMPT);
        assert_eq!(tax.with, GOLD_WITH);
        for leaf in GOLD_STORE {
            assert_eq!(classify_name(leaf.name), leaf.class, "{}", leaf.name);
        }
    }
}
