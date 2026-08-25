//! Tab heat from `lastAccessed`. Unknown access is never treated as cold.

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

const HOUR_MS: u64 = 3_600_000;
const DAY_MS: u64 = 86_400_000;

/// Classify a tab. Missing `lastAccessed` stays Warm so it is not discarded.
#[must_use]
pub fn heat_from_access(now_ms: u64, last_accessed_ms: Option<u64>, is_active: bool) -> TabHeat {
    if is_active {
        return TabHeat::Active;
    }
    let Some(last) = last_accessed_ms else {
        return TabHeat::Warm;
    };
    let age = now_ms.saturating_sub(last);
    if age < HOUR_MS {
        TabHeat::Hot
    } else if age < DAY_MS {
        TabHeat::Warm
    } else if age < 3 * DAY_MS {
        TabHeat::Cool
    } else if age < 14 * DAY_MS {
        TabHeat::Cold
    } else if age < 60 * DAY_MS {
        TabHeat::Dormant
    } else {
        TabHeat::Archival
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_wins_over_old_access() {
        assert_eq!(
            heat_from_access(60 * DAY_MS, Some(0), true),
            TabHeat::Active
        );
    }

    #[test]
    fn missing_access_is_warm() {
        assert_eq!(heat_from_access(DAY_MS, None, false), TabHeat::Warm);
    }

    #[test]
    fn cold_starts_at_three_days() {
        assert_eq!(heat_from_access(3 * DAY_MS, Some(0), false), TabHeat::Cold);
        assert_eq!(
            heat_from_access(3 * DAY_MS - 1, Some(0), false),
            TabHeat::Cool
        );
    }
}
