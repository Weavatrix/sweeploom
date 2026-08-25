//! Bounded observed history. SweepLoom never claims history from before it started.

#![cfg_attr(not(test), warn(missing_docs))]

mod store;
mod summary;

use sweeploom_core::ProcessKey;

pub use store::HistoryStore;
pub use summary::{CpuSummary, summarize_cpu};

/// One sample in a ring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    /// Unix ms when sampled.
    pub at_unix_ms: u64,
    /// CPU percent.
    pub cpu_percent: f32,
    /// RSS.
    pub rss_bytes: u64,
}

/// Fixed-size ring buffer.
#[derive(Clone, Debug)]
pub struct Ring<T> {
    slots: Vec<T>,
    cap: usize,
    next: usize,
    len: usize,
}

impl<T: Copy> Ring<T> {
    /// Create a ring with `cap` slots.
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            slots: Vec::with_capacity(cap),
            cap: cap.max(1),
            next: 0,
            len: 0,
        }
    }

    /// Push, dropping the oldest sample when full.
    pub fn push(&mut self, value: T) {
        if self.slots.len() < self.cap {
            self.slots.push(value);
            self.len += 1;
            self.next = self.len % self.cap;
            return;
        }
        self.slots[self.next] = value;
        self.next = (self.next + 1) % self.cap;
        self.len = self.cap;
    }

    /// Number of stored samples.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True when empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Samples oldest-first. Empty when SweepLoom has not observed this key.
    #[must_use]
    pub fn chrono(&self) -> Vec<T> {
        if self.slots.len() < self.cap {
            return self.slots.clone();
        }
        let mut out = Vec::with_capacity(self.cap);
        out.extend_from_slice(&self.slots[self.next..]);
        out.extend_from_slice(&self.slots[..self.next]);
        out
    }
}

/// Per-process rings. History starts at first observation.
#[derive(Clone, Debug)]
pub struct ProcessHistory {
    /// Process key.
    pub key: ProcessKey,
    /// ~1s samples, 10 minutes.
    pub fast: Ring<Sample>,
    /// ~1m samples, 1 hour.
    pub slow: Ring<Sample>,
    slow_at_ms: u64,
}

impl ProcessHistory {
    /// New history for a process.
    #[must_use]
    pub fn new(key: ProcessKey) -> Self {
        Self {
            key,
            fast: Ring::new(600),
            slow: Ring::new(60),
            slow_at_ms: 0,
        }
    }

    /// Record one sample. The slow ring keeps at most one point per minute.
    pub(crate) fn push(&mut self, sample: Sample) {
        self.fast.push(sample);
        if self.slow.is_empty() || sample.at_unix_ms.saturating_sub(self.slow_at_ms) >= 60_000 {
            self.slow.push(sample);
            self.slow_at_ms = sample.at_unix_ms;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_drops_oldest() {
        let mut ring = Ring::new(3);
        ring.push(1);
        ring.push(2);
        ring.push(3);
        ring.push(4);
        assert_eq!(ring.len(), 3);
    }
}
