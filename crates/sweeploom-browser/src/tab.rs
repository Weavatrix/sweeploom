//! One tab as reported by the companion. URLs must already be credential-free.

use serde::{Deserialize, Serialize};

use crate::action::{TabAction, suggested_action};
use crate::heat::{TabHeat, heat_from_access};

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

    /// Heat at `now_ms`. `active_tab_id` marks the current tab.
    #[must_use]
    pub fn heat(&self, now_ms: u64, active_tab_id: Option<i64>) -> TabHeat {
        heat_from_access(
            now_ms,
            self.last_accessed_ms,
            active_tab_id == Some(self.tab_id),
        )
    }

    /// Default reclaim action for this tab at `now_ms`.
    #[must_use]
    pub fn suggested_action(&self, now_ms: u64, active_tab_id: Option<i64>) -> TabAction {
        suggested_action(self, self.heat(now_ms, active_tab_id))
    }
}

/// Tabs payload from the extension.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionTabs {
    /// Tabs in this browser.
    pub tabs: Vec<TabSnapshot>,
    /// Current tab, if any.
    pub active_tab_id: Option<i64>,
}

impl CompanionTabs {
    /// How many tabs the default policy would Discard.
    #[must_use]
    pub fn discard_count(&self, now_ms: u64) -> usize {
        self.tabs
            .iter()
            .filter(|tab| tab.suggested_action(now_ms, self.active_tab_id) == TabAction::Discard)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incognito_is_protected() {
        let tab = TabSnapshot {
            tab_id: 2,
            window_id: 1,
            title: "secret".into(),
            url: "https://example.com".into(),
            last_accessed_ms: None,
            pinned: false,
            audible: false,
            discarded: false,
            incognito: true,
        };
        assert!(tab.is_protected());
        assert_eq!(
            tab.suggested_action(0, None),
            crate::action::TabAction::Keep
        );
    }

    #[test]
    fn discard_count_skips_protected() {
        let tabs = CompanionTabs {
            tabs: vec![TabSnapshot {
                tab_id: 1,
                window_id: 1,
                title: "docs".into(),
                url: "https://example.com".into(),
                last_accessed_ms: Some(0),
                pinned: false,
                audible: false,
                discarded: false,
                incognito: false,
            }],
            active_tab_id: None,
        };
        assert_eq!(tabs.discard_count(14 * 24 * 60 * 60 * 1000), 1);
    }
}
