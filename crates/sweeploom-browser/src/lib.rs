//! Native-messaging protocol types for the optional browser companion.
//!
//! The desktop app can show browser process totals without the extension.
//! Tab-level `lastAccessed` requires the companion.

#![cfg_attr(not(test), warn(missing_docs))]

use serde::{Deserialize, Serialize};

/// Tab heat derived from `lastAccessed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabHeat {
    /// Current tab.
    Active,
    /// Accessed < 1h.
    Hot,
    /// < 1d.
    Warm,
    /// 1–3d.
    Cool,
    /// 3–14d.
    Cold,
    /// > 14d.
    Dormant,
    /// > 60d.
    Archival,
}

/// Safe action. Discard is the default memory reclaim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabAction {
    /// Leave the tab.
    Keep,
    /// Unload contents, keep the tab strip entry.
    Discard,
    /// Bookmark then close, transactional.
    BookmarkAndClose,
    /// Close. Never the default.
    Close,
}

/// One tab as reported by the extension.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabSnapshot {
    /// Browser tab id.
    pub tab_id: i64,
    /// Window id.
    pub window_id: i64,
    /// Title.
    pub title: String,
    /// URL (never includes credentials; the extension must strip them).
    pub url: String,
    /// Last accessed, unix ms.
    pub last_accessed_ms: Option<u64>,
    /// Pinned.
    pub pinned: bool,
    /// Audible.
    pub audible: bool,
    /// Already discarded.
    pub discarded: bool,
    /// Incognito. Never auto-closed.
    pub incognito: bool,
}

impl TabSnapshot {
    /// True when default policy protects this tab.
    #[must_use]
    pub fn is_protected(&self) -> bool {
        self.pinned || self.audible || self.incognito
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_tabs_are_protected() {
        let tab = TabSnapshot {
            tab_id: 1,
            window_id: 1,
            title: "docs".into(),
            url: "https://example.com".into(),
            last_accessed_ms: None,
            pinned: true,
            audible: false,
            discarded: false,
            incognito: false,
        };
        assert!(tab.is_protected());
    }
}
