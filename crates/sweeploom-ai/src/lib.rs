//! AI session storage. Inspect-first; never default-delete internal DBs.

#![cfg_attr(not(test), warn(missing_docs))]

use std::path::PathBuf;

use sweeploom_platform::UserLocations;

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
    let candidates = [
        ("claude", locations.home.join(".claude")),
        ("codex", locations.home.join(".codex")),
        ("cursor", locations.home.join(".cursor")),
    ];
    candidates
        .into_iter()
        .filter(|(_, path)| path.exists())
        .map(|(tool, path)| AiStore { tool, path })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_does_not_panic() {
        let _ = discover_stores(&UserLocations::current());
    }
}
