//! AI session storage. Inspect-first; never default-delete internal DBs.

#![cfg_attr(not(test), warn(missing_docs))]

mod classify;
mod gold;
mod inventory;
mod offers;

use std::path::PathBuf;

use sweeploom_platform::UserLocations;

pub use classify::{
    AiClass, ContextAdvice, advise_context, classify_name, estimated_prompt_tokens,
    estimated_prompt_tokens_with_files,
};
pub use gold::{
    GOLD_DUMP, GOLD_HISTORY_AS_PROMPT, GOLD_STORE, GOLD_WITH, GoldLeaf, GoldTax, gold_tax,
};
pub use inventory::{Limits, StoreEntry, StoreInventory, list_store};
pub use offers::{AiEntry, AiOffer, inspect_offers};

/// Known on-disk AI session roots. Presence does not mean it is safe to delete.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiStore {
    /// Tool name.
    pub tool: &'static str,
    /// Path if it exists.
    pub path: PathBuf,
}

/// Discover local AI stores without reading their contents.
#[must_use]
pub fn discover_stores(locations: &UserLocations) -> Vec<AiStore> {
    let mut candidates = vec![
        ("claude", locations.home.join(".claude")),
        ("claude", locations.home.join(".claude-server-commander")),
        ("codex", locations.home.join(".codex")),
        ("cursor", locations.home.join(".cursor")),
        ("opencode", locations.home.join(".opencode")),
        ("gemini", locations.home.join(".gemini")),
        ("grok", locations.home.join(".grok")),
        ("weavatrix", locations.home.join(".weavatrix")),
    ];
    if let Some(roaming) = std::env::var_os("APPDATA") {
        candidates.push(("cursor", PathBuf::from(roaming).join("Cursor")));
    }
    candidates
        .into_iter()
        .filter(|(_, path)| path.exists())
        .map(|(tool, path)| AiStore { tool, path })
        .collect()
}

#[cfg(test)]
mod inventory_tests;

#[cfg(test)]
mod classify_bench;

#[cfg(test)]
mod token_bench;

#[cfg(test)]
mod with_without_bench;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_does_not_panic() {
        let _ = discover_stores(&UserLocations::current());
    }

    #[test]
    fn discover_finds_claude_and_server_commander() {
        let home = std::env::temp_dir().join(format!(
            "sweeploom-ai-home-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|item| item.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(home.join(".claude")).unwrap();
        std::fs::create_dir_all(home.join(".claude-server-commander")).unwrap();
        let locations = UserLocations {
            downloads: None,
            temp: home.join("tmp"),
            cache: None,
            app_config: home.join("cfg"),
            app_data: home.join("data"),
            home: home.clone(),
        };
        let stores = discover_stores(&locations);
        let _ = std::fs::remove_dir_all(&home);
        assert!(
            stores.iter().any(|item| item.path.ends_with(".claude")),
            "{stores:?}"
        );
        assert!(
            stores
                .iter()
                .any(|item| item.path.ends_with(".claude-server-commander")),
            "{stores:?}"
        );
    }
}
