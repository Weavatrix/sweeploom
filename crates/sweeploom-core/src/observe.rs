//! Sample coverage for idle claims. Gaps must not grow proven idle.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::ids::ProcessKey;

/// How long a missing sample may last before idle becomes unknown.
pub const SAMPLE_GAP: Duration = Duration::from_secs(90);

/// Per-process observation bookkeeping.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObservationTracker {
    first_seen: HashMap<ProcessKey, SystemTime>,
    last_sample: HashMap<ProcessKey, SystemTime>,
    last_busy: HashMap<ProcessKey, SystemTime>,
    last_network: HashMap<ProcessKey, SystemTime>,
    last_global_sample: Option<SystemTime>,
    gapped: bool,
}

impl ObservationTracker {
    /// Record one sample. `gapped` is set when the previous global sample is stale.
    pub fn record(
        &mut self,
        keys: impl IntoIterator<Item = ProcessKey>,
        busy: impl IntoIterator<Item = ProcessKey>,
        network: impl IntoIterator<Item = ProcessKey>,
        at: SystemTime,
    ) {
        self.gapped = self.last_global_sample.is_some_and(|prev| {
            at.duration_since(prev)
                .ok()
                .is_none_or(|span| span > SAMPLE_GAP)
        });
        for key in keys {
            self.first_seen.entry(key).or_insert(at);
            self.last_sample.insert(key, at);
        }
        for key in busy {
            self.last_busy.insert(key, at);
        }
        for key in network {
            self.last_network.insert(key, at);
        }
        self.last_global_sample = Some(at);
    }

    /// Proven idle start. `None` when coverage has a gap or the key was never sampled.
    #[must_use]
    pub fn observed_idle_since(&self, key: ProcessKey, now: SystemTime) -> Option<SystemTime> {
        if self.gapped {
            return None;
        }
        let last_sample = self.last_sample.get(&key).copied()?;
        if now
            .duration_since(last_sample)
            .ok()
            .is_none_or(|span| span > SAMPLE_GAP)
        {
            return None;
        }
        Some(
            self.last_busy
                .get(&key)
                .copied()
                .unwrap_or_else(|| self.first_seen.get(&key).copied().unwrap_or(last_sample)),
        )
    }

    /// Last observed network activity, independent of CPU.
    #[must_use]
    pub fn last_network(&self, key: ProcessKey) -> Option<SystemTime> {
        self.last_network.get(&key).copied()
    }

    /// True when the last global sample is too old to claim continuous idle.
    #[must_use]
    pub const fn has_gap(&self) -> bool {
        self.gapped
    }

    /// Drop live maps when the UI hides, but remember that coverage broke.
    pub fn mark_gap(&mut self) {
        self.gapped = true;
        self.last_global_sample = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gap_and_missing_sample_are_unknown() {
        let key = ProcessKey {
            pid: 8,
            started_at_unix_ms: 1,
        };
        let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(10);
        let t1 = t0 + Duration::from_secs(200);
        let mut tracker = ObservationTracker::default();
        tracker.record([key], [], [], t0);
        assert!(
            tracker
                .observed_idle_since(key, t0 + Duration::from_secs(10))
                .is_some()
        );
        tracker.record([key], [], [], t1);
        assert!(tracker.has_gap());
        assert!(tracker.observed_idle_since(key, t1).is_none());
    }
}
