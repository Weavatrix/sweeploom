//! Frozen process-stop intent. Numeric session ids must not be reused after confirm.

use std::time::SystemTime;

use crate::ids::ProcessKey;

/// User-approved termination of a specific process set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingTermination {
    /// Root process the user originally selected.
    pub root: ProcessKey,
    /// Exact keys approved at confirm time.
    pub approved_keys: Vec<ProcessKey>,
    /// When the confirmation was created.
    pub created_at: SystemTime,
}

impl PendingTermination {
    /// Freeze the currently displayed keys. Unknown start times are refused.
    pub fn freeze(root: ProcessKey, keys: &[ProcessKey]) -> Option<Self> {
        if root.started_at_unix_ms == 0 {
            return None;
        }
        if keys.iter().any(|key| key.started_at_unix_ms == 0) {
            return None;
        }
        if !keys.contains(&root) {
            return None;
        }
        Some(Self {
            root,
            approved_keys: keys.to_vec(),
            created_at: SystemTime::now(),
        })
    }

    /// True when live membership still matches the frozen approval.
    #[must_use]
    pub fn still_valid(&self, live_keys: &[ProcessKey]) -> bool {
        same_set(&self.approved_keys, live_keys) && live_keys.contains(&self.root)
    }
}

fn same_set(left: &[ProcessKey], right: &[ProcessKey]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter().all(|key| right.contains(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(pid: u32, start: u64) -> ProcessKey {
        ProcessKey {
            pid,
            started_at_unix_ms: start,
        }
    }

    #[test]
    fn refresh_and_reorder_do_not_retarget() {
        let root = key(20, 100);
        let pending = PendingTermination::freeze(root, &[root, key(21, 101)]).unwrap();
        assert!(pending.still_valid(&[key(21, 101), root]));
        assert!(!pending.still_valid(&[key(30, 200), key(31, 201)]));
        assert!(!pending.still_valid(&[root]));
        assert!(PendingTermination::freeze(key(1, 0), &[key(1, 0)]).is_none());
    }
}
