//! Deterministic forgotten-session policy. Hard safety is applied first.

use std::time::{Duration, SystemTime};

use sweeploom_core::{
    LiveSession, ProcessSafetyClass, ProjectId, Recommendation, SessionActivity, SessionKind,
    SessionRecommendation, SessionSafety,
};

/// Inputs the scorer is allowed to consider.
#[derive(Clone, Copy, Debug)]
pub struct ForgottenInput {
    /// Session age.
    pub age: Duration,
    /// Observed idle duration, if known.
    pub idle: Option<Duration>,
    /// Combined RSS.
    pub rss_bytes: u64,
    /// Combined CPU percent.
    pub cpu_percent: f32,
    /// Network currently active (capability-gated).
    pub network_active: bool,
    /// Session belongs to the current project.
    pub is_current_project: bool,
    /// System-critical members exist.
    pub system_critical: bool,
    /// Known helper / agent / dev-server.
    pub known_dev: bool,
    /// Looks orphaned (no living user parent).
    pub orphan: bool,
}

/// Score a session in place and return it.
#[must_use]
pub fn score_session(
    session: &LiveSession,
    now: SystemTime,
    current_project: Option<&ProjectId>,
) -> LiveSession {
    let mut scored = session.clone();
    if scored.safety.terminate_disabled
        || scored.safety.assessment.is_blocked()
            && scored
                .safety
                .assessment
                .blockers
                .iter()
                .any(|item| matches!(item, sweeploom_core::Blocker::SystemCriticalProcess))
    {
        scored.recommendation.recommendation = Recommendation::Keep;
        scored.activity = SessionActivity::Unknown;
        return scored;
    }

    let age = scored
        .started_at
        .and_then(|start| now.duration_since(start).ok())
        .unwrap_or_default();
    let idle = scored
        .observed_last_activity
        .and_then(|last| now.duration_since(last).ok());
    let is_current = match (current_project, scored.project.as_ref()) {
        (Some(current), Some(project)) => current == project,
        _ => false,
    };
    let input = ForgottenInput {
        age,
        idle,
        rss_bytes: scored.rss_bytes,
        cpu_percent: scored.cpu_percent,
        network_active: scored.network.byte_rate_available
            && (scored.network.observed_rx_bytes + scored.network.observed_tx_bytes > 0)
            && idle.is_some_and(|item| item < Duration::from_mins_compat(5)),
        is_current_project: is_current,
        system_critical: scored.safety.terminate_disabled,
        known_dev: matches!(
            scored.kind,
            SessionKind::ClaudeCode
                | SessionKind::Codex
                | SessionKind::Mcp
                | SessionKind::DevServer
                | SessionKind::Build
        ),
        orphan: scored.processes.len() == 1 && scored.kind != SessionKind::Unknown,
    };
    apply_policy(&mut scored, input);
    scored
}

trait DurationMins {
    fn from_mins_compat(mins: u64) -> Duration;
}

impl DurationMins for Duration {
    fn from_mins_compat(mins: u64) -> Duration {
        Duration::from_secs(mins.saturating_mul(60))
    }
}

fn apply_policy(session: &mut LiveSession, input: ForgottenInput) {
    if input.system_critical {
        session.safety = SessionSafety::system_critical();
        session.recommendation = SessionRecommendation {
            recommendation: Recommendation::Keep,
            estimated_reclaimable_rss: 0,
        };
        return;
    }
    if input.is_current_project {
        session.activity = SessionActivity::Active;
        session.recommendation.recommendation = Recommendation::Keep;
        return;
    }
    if input.cpu_percent > 15.0 {
        session.activity = SessionActivity::RunawayCpu;
        session.recommendation.recommendation = Recommendation::Keep;
        return;
    }
    if input.network_active {
        session.activity = SessionActivity::NetworkActive;
        session.recommendation.recommendation = Recommendation::Keep;
        return;
    }

    let idle = input.idle.unwrap_or(input.age);
    let idle_hours = idle.as_secs() / 3600;
    if input.orphan && idle > Duration::from_secs(30 * 60) && input.known_dev {
        session.activity = SessionActivity::LikelyForgotten;
        session.recommendation.recommendation = Recommendation::StronglyRecommended;
        session.recommendation.estimated_reclaimable_rss = estimate_reclaim(input.rss_bytes);
        return;
    }
    if idle_hours >= 2 && input.rss_bytes > 1_000_000_000 {
        session.activity = SessionActivity::SleepingMemoryHeavy;
        session.recommendation.recommendation = Recommendation::Recommended;
        session.recommendation.estimated_reclaimable_rss = estimate_reclaim(input.rss_bytes);
        return;
    }
    if idle_hours >= 2 {
        session.activity = SessionActivity::Idle;
        session.recommendation.recommendation = Recommendation::Optional;
        session.recommendation.estimated_reclaimable_rss = estimate_reclaim(input.rss_bytes);
        return;
    }
    session.activity = SessionActivity::BackgroundActive;
    session.recommendation.recommendation = Recommendation::Keep;
}

fn estimate_reclaim(rss: u64) -> u64 {
    // Shared pages mean RSS is not uniquely reclaimable. Be honest.
    rss.saturating_mul(75) / 100
}

/// Map a process safety class onto session safety.
#[must_use]
pub fn safety_from_class(class: ProcessSafetyClass) -> SessionSafety {
    match class {
        ProcessSafetyClass::SystemCritical => SessionSafety::system_critical(),
        _ => SessionSafety::user(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sweeploom_core::{
        SessionDiskUsage, SessionId, SessionKind, SessionNetworkUsage, SessionRecommendation,
        SessionSafety,
    };

    fn session(rss: u64, kind: SessionKind) -> LiveSession {
        LiveSession {
            id: SessionId(1),
            kind,
            project: None,
            processes: Vec::new(),
            started_at: Some(SystemTime::UNIX_EPOCH),
            observed_last_activity: Some(SystemTime::UNIX_EPOCH),
            rss_bytes: rss,
            cpu_percent: 0.0,
            disk: SessionDiskUsage::default(),
            network: SessionNetworkUsage::default(),
            activity: SessionActivity::Unknown,
            safety: SessionSafety::user(),
            recommendation: SessionRecommendation {
                recommendation: Recommendation::Keep,
                estimated_reclaimable_rss: 0,
            },
        }
    }

    #[test]
    fn idle_heavy_session_is_recommended() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        let scored = score_session(&session(2_000_000_000, SessionKind::ClaudeCode), now, None);
        assert_eq!(
            scored.recommendation.recommendation,
            Recommendation::Recommended
        );
        assert_eq!(scored.activity, SessionActivity::SleepingMemoryHeavy);
    }

    #[test]
    fn system_critical_stays_keep() {
        let mut value = session(2_000_000_000, SessionKind::Unknown);
        value.safety = SessionSafety::system_critical();
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        let scored = score_session(&value, now, None);
        assert_eq!(scored.recommendation.recommendation, Recommendation::Keep);
        assert!(scored.safety.terminate_disabled);
    }
}
