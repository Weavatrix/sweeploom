//! Stable identifiers. A process is never identified by PID alone.

use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Opaque candidate identifier, stable for the lifetime of one scan snapshot.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CandidateId(pub u64);

/// Immutable cleanup-plan identifier.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlanId(pub u64);

/// Project identity derived from a canonical root path.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectId(pub PathBuf);

/// Tool identity, for example `cargo` or `npm`.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ToolId(pub String);

/// Logical live-session identifier.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionId(pub u64);

/// Process identity that survives PID reuse.
///
/// Always pair the OS pid with the observed start time. History and terminate
/// actions must refuse to apply when the live process no longer matches.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessKey {
    /// Operating-system process identifier.
    pub pid: u32,
    /// Process start time as milliseconds since Unix epoch, when known.
    pub started_at_unix_ms: u64,
}

impl ProcessKey {
    /// Build a key from a pid and an optional start timestamp.
    #[must_use]
    pub fn new(pid: u32, started_at: Option<SystemTime>) -> Self {
        let started_at_unix_ms = started_at
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |duration| {
                u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
            });
        Self {
            pid,
            started_at_unix_ms,
        }
    }

    /// Start time reconstructed from the stored unix-ms value.
    #[must_use]
    pub fn started_at(self) -> Option<SystemTime> {
        if self.started_at_unix_ms == 0 {
            return None;
        }
        Some(UNIX_EPOCH + Duration::from_millis(self.started_at_unix_ms))
    }
}

impl fmt::Display for ProcessKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "pid:{}@{}", self.pid, self.started_at_unix_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_key_distinguishes_pid_reuse() {
        let first = ProcessKey::new(100, Some(UNIX_EPOCH + Duration::from_secs(10)));
        let reused = ProcessKey::new(100, Some(UNIX_EPOCH + Duration::from_secs(90)));
        assert_ne!(first, reused);
        assert_eq!(first.pid, reused.pid);
    }
}
