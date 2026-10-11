//! Deterministic forgotten-session policy. Hard safety is applied first.

use std::time::{Duration, SystemTime};

use sweeploom_core::{
    LiveSession, ProcessSafetyClass, ProcessSnapshot, ProjectId, Recommendation, SessionActivity,
    SessionKind, SessionRecommendation, SessionSafety,
};

/// Inputs the scorer is allowed to consider.
#[derive(Clone, Copy, Debug)]
pub struct ForgottenInput {
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
    /// Disk read/write was observed in this sample.
    pub disk_busy: bool,
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

    let idle = scored
        .observed_last_activity
        .and_then(|last| now.duration_since(last).ok());
    let is_current = match (current_project, scored.project.as_ref()) {
        (Some(current), Some(project)) => current == project,
        _ => false,
    };
    let network_bytes = scored.network.byte_rate_available
        && (scored.network.observed_rx_bytes + scored.network.observed_tx_bytes > 0);
    let input = ForgottenInput {
        idle,
        rss_bytes: scored.rss_bytes,
        cpu_percent: scored.cpu_percent,
        network_active: network_bytes,
        is_current_project: is_current,
        system_critical: scored.safety.terminate_disabled,
        known_dev: scored.kind.is_known_dev(),
        disk_busy: scored.disk.read_bytes + scored.disk.write_bytes > 0,
    };
    apply_policy(&mut scored, input);
    if scored.kind == SessionKind::Browser || scored.kind.is_agent() {
        scored.recommendation.recommendation = Recommendation::Keep;
        scored.recommendation.estimated_reclaimable_rss = 0;
    }
    scored
}

/// MCP with no live agent parent is an orphan helper, not a kill signal by itself.
pub fn mark_orphan_mcp(sessions: &mut [LiveSession], processes: &[ProcessSnapshot]) {
    let agent_members: std::collections::HashSet<_> = sessions
        .iter()
        .filter(|session| session.kind.is_agent())
        .flat_map(|session| session.processes.iter().copied())
        .collect();
    for session in sessions.iter_mut() {
        if session.kind != SessionKind::Mcp {
            continue;
        }
        let attached = session.processes.iter().any(|key| {
            let mut parent = processes
                .iter()
                .find(|p| p.key == *key)
                .and_then(|p| p.parent);
            let mut visited = std::collections::HashSet::new();
            while let Some(key) = parent {
                if !visited.insert(key) {
                    break;
                }
                if agent_members.contains(&key) {
                    return true;
                }
                parent = processes
                    .iter()
                    .find(|p| p.key == key)
                    .and_then(|p| p.parent);
            }
            false
        });
        if !attached
            && !matches!(
                session.activity,
                SessionActivity::Active
                    | SessionActivity::NetworkActive
                    | SessionActivity::RunawayCpu
            )
        {
            session.activity = SessionActivity::OrphanCandidate;
            session.recommendation.recommendation = Recommendation::Optional;
            session.recommendation.estimated_reclaimable_rss = estimate_reclaim(session.rss_bytes);
        } else {
            session.recommendation.recommendation = Recommendation::Keep;
            session.recommendation.estimated_reclaimable_rss = 0;
            if attached
                && matches!(
                    session.activity,
                    SessionActivity::LikelyForgotten | SessionActivity::SleepingMemoryHeavy
                )
            {
                session.activity = SessionActivity::Idle;
            }
        }
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
    if input.cpu_percent > 0.5 || input.disk_busy {
        session.activity = SessionActivity::Active;
        session.recommendation.recommendation = Recommendation::Keep;
        return;
    }
    if input.network_active {
        session.activity = SessionActivity::NetworkActive;
        session.recommendation.recommendation = Recommendation::Keep;
        return;
    }
    let Some(idle) = input.idle else {
        session.activity = SessionActivity::BackgroundActive;
        session.recommendation.recommendation = Recommendation::Keep;
        return;
    };
    let idle_hours = idle.as_secs() / 3600;
    if input.known_dev && idle < Duration::from_secs(2 * 3600) {
        session.activity = SessionActivity::BackgroundActive;
        session.recommendation.recommendation = Recommendation::Keep;
        return;
    }
    if !input.known_dev {
        session.activity = if idle_hours >= 2 {
            SessionActivity::Idle
        } else {
            SessionActivity::BackgroundActive
        };
        session.recommendation.recommendation = Recommendation::Keep;
        session.recommendation.estimated_reclaimable_rss = 0;
        return;
    }
    if idle_hours >= 2 && input.rss_bytes > 1_000_000_000 {
        session.activity = SessionActivity::SleepingMemoryHeavy;
        session.recommendation.recommendation = Recommendation::Recommended;
        session.recommendation.estimated_reclaimable_rss = estimate_reclaim(input.rss_bytes);
        return;
    }
    if idle_hours >= 2 {
        session.activity = SessionActivity::LikelyForgotten;
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
#[path = "forgotten_tests.rs"]
mod tests;
