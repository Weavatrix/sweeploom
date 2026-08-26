//! Default tab action. Close is never the suggested action.

use serde::{Deserialize, Serialize};

use crate::heat::TabHeat;
use crate::tab::TabSnapshot;

/// Safe action. Discard is the default memory reclaim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TabAction {
    /// Leave the tab.
    Keep,
    /// Unload contents, keep the tab strip entry.
    Discard,
    /// Bookmark then close, transactional.
    BookmarkAndClose,
    /// Focus the tab and its window. Not a reclaim action.
    Focus,
    /// Close. Never the default.
    Close,
}

/// Default action from heat + protections. Never `Close`.
#[must_use]
pub fn suggested_action(tab: &TabSnapshot, heat: TabHeat) -> TabAction {
    if tab.is_protected() || tab.discarded {
        return TabAction::Keep;
    }
    match heat {
        TabHeat::Cold | TabHeat::Dormant | TabHeat::Archival => TabAction::Discard,
        TabHeat::Active | TabHeat::Hot | TabHeat::Warm | TabHeat::Cool => TabAction::Keep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tab(pinned: bool, discarded: bool) -> TabSnapshot {
        TabSnapshot {
            tab_id: 1,
            window_id: 1,
            title: "docs".into(),
            url: "https://example.com".into(),
            last_accessed_ms: Some(0),
            pinned,
            audible: false,
            discarded,
            incognito: false,
        }
    }

    #[test]
    fn cold_unprotected_is_discard() {
        assert_eq!(
            suggested_action(&tab(false, false), TabHeat::Cold),
            TabAction::Discard
        );
    }

    #[test]
    fn pinned_stays_keep() {
        assert_eq!(
            suggested_action(&tab(true, false), TabHeat::Archival),
            TabAction::Keep
        );
    }

    #[test]
    fn never_defaults_to_close() {
        for heat in [
            TabHeat::Active,
            TabHeat::Hot,
            TabHeat::Warm,
            TabHeat::Cool,
            TabHeat::Cold,
            TabHeat::Dormant,
            TabHeat::Archival,
        ] {
            assert_ne!(suggested_action(&tab(false, false), heat), TabAction::Close);
        }
    }
}
