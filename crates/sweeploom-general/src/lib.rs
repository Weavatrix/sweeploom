//! General (non-project) cleaner roots. User files stay REVIEW.

#![cfg_attr(not(test), warn(missing_docs))]

use std::path::PathBuf;

use sweeploom_platform::UserLocations;

/// A general-cleaner root with a default safety posture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneralRoot {
    /// Category id.
    pub id: &'static str,
    /// Path.
    pub path: PathBuf,
    /// True when auto-select is allowed for obvious temp/cache.
    pub auto_select_allowed: bool,
}

/// Default general roots for this machine.
#[must_use]
pub fn default_roots(locations: &UserLocations) -> Vec<GeneralRoot> {
    let mut roots = vec![GeneralRoot {
        id: "user-temp",
        path: locations.temp.clone(),
        auto_select_allowed: true,
    }];
    if let Some(cache) = &locations.cache {
        roots.push(GeneralRoot {
            id: "user-cache",
            path: cache.clone(),
            auto_select_allowed: true,
        });
    }
    if let Some(downloads) = &locations.downloads {
        roots.push(GeneralRoot {
            id: "downloads",
            path: downloads.clone(),
            auto_select_allowed: false,
        });
    }
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downloads_are_never_auto_selected() {
        let roots = default_roots(&UserLocations::current());
        assert!(
            roots
                .iter()
                .filter(|item| item.id == "downloads")
                .all(|item| !item.auto_select_allowed)
        );
    }
}
