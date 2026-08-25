//! Declarative cleaner rules. Rules are data, never shell.

#![cfg_attr(not(test), warn(missing_docs))]

mod load;

use serde::Deserialize;
use sweeploom_core::{DeletionStrategy, SafetyLevel};

pub use load::{LoadedRuleFile, load_packs};

/// A loaded rule pack.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RulePack {
    /// Schema version.
    pub schema: u32,
    /// Cleaners.
    #[serde(default)]
    pub cleaner: Vec<CleanerRule>,
}

/// One declarative cleaner.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct CleanerRule {
    /// Stable id.
    pub id: String,
    /// UI label.
    #[serde(default)]
    pub label: Option<String>,
    /// Category key.
    #[serde(default)]
    pub category: Option<String>,
    /// Platforms (`windows`, `macos`, `linux`). Empty means all.
    #[serde(default)]
    pub platforms: Vec<String>,
    /// Marker files that identify a project.
    #[serde(default)]
    pub markers: Vec<String>,
    /// Relative paths to consider.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Risk label from the rule file. Mapped onto [`SafetyLevel`].
    #[serde(default)]
    pub risk: Option<String>,
    /// Strategy label.
    #[serde(default)]
    pub strategy: Option<String>,
}

impl CleanerRule {
    /// Map the rule's risk string onto a safety level. Unknown → Review.
    #[must_use]
    pub fn safety_level(&self) -> SafetyLevel {
        match self.risk.as_deref() {
            Some("safe") => SafetyLevel::Safe,
            Some("low-risk" | "low_risk") => SafetyLevel::LowRisk,
            Some("dangerous") => SafetyLevel::Dangerous,
            Some("blocked") => SafetyLevel::Blocked,
            _ => SafetyLevel::Review,
        }
    }

    /// Map strategy string.
    #[must_use]
    pub fn deletion_strategy(&self) -> DeletionStrategy {
        match self.strategy.as_deref() {
            Some("permanent-generated" | "permanent_generated") => {
                DeletionStrategy::PermanentGenerated
            }
            Some("trash") => DeletionStrategy::Trash,
            Some("native-tool" | "native_tool") => DeletionStrategy::NativeTool,
            Some("archive") => DeletionStrategy::Archive,
            Some("truncate") => DeletionStrategy::Truncate,
            _ => DeletionStrategy::InspectOnly,
        }
    }
}

/// Parse a TOML rule pack.
pub fn parse_pack(text: &str) -> Result<RulePack, toml::de::Error> {
    toml::from_str(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vite_cache_rule() {
        let pack = parse_pack(
            r#"
schema = 1

[[cleaner]]
id = "vite-cache"
label = "Vite cache"
category = "build-cache"
risk = "safe"
strategy = "permanent-generated"
markers = ["package.json"]
paths = ["node_modules/.vite", ".vite"]
"#,
        )
        .unwrap();
        assert_eq!(pack.cleaner[0].id, "vite-cache");
        assert_eq!(pack.cleaner[0].safety_level(), SafetyLevel::Safe);
        assert_eq!(
            pack.cleaner[0].deletion_strategy(),
            DeletionStrategy::PermanentGenerated
        );
    }
}
