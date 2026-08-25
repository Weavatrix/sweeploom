//! Project and candidate activity buckets. Source heat and artifact heat are
//! independent.

use std::time::{Duration, SystemTime};

/// Deterministic activity bucket. Defaults match the product plan.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActivityState {
    /// Process is using the path, or a write happened in the last 15 minutes.
    ActiveNow,
    /// Meaningful source activity in the last 24 hours.
    Hot,
    /// 1–3 days.
    Warm,
    /// 3–7 days.
    Cool,
    /// 7–30 days.
    Cold,
    /// 30–180 days.
    Dormant,
    /// Older than 180 days.
    Archival,
    /// Not enough evidence to classify.
    Unknown,
}

impl ActivityState {
    /// Map an age onto the default buckets.
    #[must_use]
    pub fn from_age(age: Option<Duration>) -> Self {
        const MIN: u64 = 60;
        const HOUR: u64 = 60 * MIN;
        const DAY: u64 = 24 * HOUR;
        let Some(age) = age else {
            return Self::Unknown;
        };
        let secs = age.as_secs();
        if secs < 15 * MIN {
            Self::ActiveNow
        } else if secs < DAY {
            Self::Hot
        } else if secs < 3 * DAY {
            Self::Warm
        } else if secs < 7 * DAY {
            Self::Cool
        } else if secs < 30 * DAY {
            Self::Cold
        } else if secs < 180 * DAY {
            Self::Dormant
        } else {
            Self::Archival
        }
    }

    /// Age from `when` relative to `now`.
    #[must_use]
    pub fn from_timestamp(when: Option<SystemTime>, now: SystemTime) -> Self {
        Self::from_age(when.and_then(|stamp| now.duration_since(stamp).ok()))
    }

    /// Short UI label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ActiveNow => "Active now",
            Self::Hot => "Hot",
            Self::Warm => "Warm",
            Self::Cool => "Cool",
            Self::Cold => "Cold",
            Self::Dormant => "Dormant",
            Self::Archival => "Archival",
            Self::Unknown => "Unknown",
        }
    }
}

/// Independent activity signals for a project or candidate.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityEvidence {
    /// Latest user-source modification.
    pub latest_source_modified: Option<SystemTime>,
    /// Latest generated/artifact modification.
    pub latest_generated_modified: Option<SystemTime>,
    /// Latest modification of any file under the path.
    pub latest_any_modified: Option<SystemTime>,
    /// Birth time when the filesystem actually exposes it. Never the primary signal.
    pub latest_birth_time_if_available: Option<SystemTime>,
    /// Latest Git commit time, when known.
    pub git_last_commit: Option<SystemTime>,
    /// Whether a live process is using the project or candidate.
    pub process_activity: bool,
    /// Whether an AI/agent session is using the project.
    pub ai_session_activity: bool,
}

impl ActivityEvidence {
    /// Source heat bucket.
    #[must_use]
    pub fn source_heat(&self, now: SystemTime) -> ActivityState {
        if self.process_activity || self.ai_session_activity {
            return ActivityState::ActiveNow;
        }
        ActivityState::from_timestamp(self.latest_source_modified, now)
    }

    /// Artifact heat bucket. Fresh generated files do not make source HOT.
    #[must_use]
    pub fn artifact_heat(&self, now: SystemTime) -> ActivityState {
        ActivityState::from_timestamp(self.latest_generated_modified, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_today_does_not_make_source_hot() {
        let now = UNIX_EPOCH_PLUS_DAYS(40);
        let evidence = ActivityEvidence {
            latest_source_modified: Some(UNIX_EPOCH_PLUS_DAYS(10)),
            latest_generated_modified: Some(now),
            latest_any_modified: Some(now),
            ..ActivityEvidence::default()
        };
        assert_eq!(evidence.source_heat(now), ActivityState::Dormant);
        assert_eq!(evidence.artifact_heat(now), ActivityState::ActiveNow);
    }

    #[test]
    fn process_forces_active_now() {
        let now = UNIX_EPOCH_PLUS_DAYS(40);
        let evidence = ActivityEvidence {
            latest_source_modified: Some(UNIX_EPOCH_PLUS_DAYS(1)),
            process_activity: true,
            ..ActivityEvidence::default()
        };
        assert_eq!(evidence.source_heat(now), ActivityState::ActiveNow);
    }

    #[allow(non_snake_case)]
    fn UNIX_EPOCH_PLUS_DAYS(days: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(days * 24 * 60 * 60)
    }
}
