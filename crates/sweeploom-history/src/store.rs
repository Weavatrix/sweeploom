//! Per-process observed history. Starts at first sample.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use sweeploom_core::{ProcessKey, ProcessSnapshot};

use crate::{ProcessHistory, Sample};

const MAX_TRACKED: usize = 1_024;

/// All live process rings for this session of SweepLoom.
#[derive(Clone, Debug, Default)]
pub struct HistoryStore {
    by_key: HashMap<ProcessKey, ProcessHistory>,
}

impl HistoryStore {
    /// Record the current snapshot. Unknown keys start a new ring.
    pub fn record(&mut self, processes: &[ProcessSnapshot], now: SystemTime) {
        let at = unix_ms(now);
        let mut live = Vec::with_capacity(processes.len());
        for process in processes {
            live.push(process.key);
            let entry = self
                .by_key
                .entry(process.key)
                .or_insert_with(|| ProcessHistory::new(process.key));
            entry.fast.push(Sample {
                at_unix_ms: at,
                cpu_percent: process.cpu_percent,
                rss_bytes: process.rss_bytes,
            });
        }
        self.by_key.retain(|key, _| live.contains(key));
        if self.by_key.len() > MAX_TRACKED {
            let overflow = self.by_key.len() - MAX_TRACKED;
            let drop: Vec<ProcessKey> = self.by_key.keys().copied().take(overflow).collect();
            for key in drop {
                self.by_key.remove(&key);
            }
        }
    }

    /// History for one process, if SweepLoom has seen it.
    #[must_use]
    pub fn get(&self, key: ProcessKey) -> Option<&ProcessHistory> {
        self.by_key.get(&key)
    }

    /// How many processes are currently tracked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_key.len()
    }

    /// True when nothing has been observed yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_key.is_empty()
    }
}

fn unix_ms(now: SystemTime) -> u64 {
    now.duration_since(UNIX_EPOCH)
        .map(|item| item.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sweeploom_core::{ProcessKey, ProcessSnapshot};

    #[test]
    fn drops_keys_that_left() {
        let mut store = HistoryStore::default();
        let key = ProcessKey::new(1, Some(UNIX_EPOCH));
        let snap = dummy(key, 10);
        store.record(&[snap], UNIX_EPOCH);
        assert_eq!(store.len(), 1);
        store.record(&[], UNIX_EPOCH);
        assert!(store.is_empty());
    }

    fn dummy(key: ProcessKey, rss: u64) -> ProcessSnapshot {
        ProcessSnapshot {
            key,
            pid: key.pid,
            parent: None,
            name: "demo".into(),
            exe: None,
            cwd: None,
            command: Vec::new(),
            started_at: None,
            runtime: std::time::Duration::ZERO,
            rss_bytes: rss,
            virtual_bytes: rss,
            cpu_percent: 1.0,
            accumulated_cpu_ms: 0,
            disk_read_delta: 0,
            disk_write_delta: 0,
            network: sweeploom_core::NetworkSnapshot::default(),
            project: None,
            session: None,
            safety_class: sweeploom_core::ProcessSafetyClass::Unknown,
        }
    }
}
