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
        assert_eq!(scored.recommendation.recommendation, Recommendation::Keep);
        assert_eq!(scored.recommendation.estimated_reclaimable_rss, 0);
        assert_eq!(scored.activity, SessionActivity::SleepingMemoryHeavy);
    }

    #[test]
    fn quiet_mcp_through_wrapper_keeps_live_agent_ownership_after_rescore() {
        let root = crate::tests::proc(10, None, "codex", None, &["codex"], 1, 0.0);
        let wrapper = crate::tests::proc(11, Some(10), "helper", None, &["helper"], 1, 0.0);
        let server =
            crate::tests::proc(12, Some(11), "node", None, &["node", "mcp-server"], 1, 0.0);
        let mut agent = session(100_000_000, SessionKind::Codex);
        agent.processes = vec![root.key];
        let mut helper = session(2_000_000_000, SessionKind::Mcp);
        helper.processes = vec![server.key];
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        helper = score_session(&helper, now, None);
        assert_eq!(
            helper.recommendation.recommendation,
            Recommendation::Recommended
        );
        let mut sessions = [agent, helper];
        mark_orphan_mcp(&mut sessions, &[root, wrapper, server]);
        assert_eq!(
            sessions[1].recommendation.recommendation,
            Recommendation::Keep
        );
        assert_eq!(sessions[1].recommendation.estimated_reclaimable_rss, 0);
        assert_ne!(sessions[1].activity, SessionActivity::LikelyForgotten);
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

    #[test]
    fn browser_tree_is_never_auto_reclaimed() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        let scored = score_session(&session(8_000_000_000, SessionKind::Browser), now, None);
        assert_eq!(scored.recommendation.recommendation, Recommendation::Keep);
        assert_eq!(scored.recommendation.estimated_reclaimable_rss, 0);
    }

    #[test]
    fn unknown_idle_claude_is_keep() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        let mut value = session(2_000_000_000, SessionKind::ClaudeCode);
        value.observed_last_activity = None;
        let scored = score_session(&value, now, None);
        assert_eq!(scored.recommendation.recommendation, Recommendation::Keep);
    }

    #[test]
    fn claude_with_cpu_is_keep() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        let mut value = session(2_000_000_000, SessionKind::ClaudeCode);
        value.cpu_percent = 3.0;
        let scored = score_session(&value, now, None);
        assert_eq!(scored.recommendation.recommendation, Recommendation::Keep);
        assert_eq!(scored.activity, SessionActivity::Active);
    }

    #[test]
    fn generic_app_is_never_auto_recommended() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        let scored = score_session(&session(2_000_000_000, SessionKind::GenericApp), now, None);
        assert_eq!(scored.recommendation.recommendation, Recommendation::Keep);
        assert_eq!(scored.recommendation.estimated_reclaimable_rss, 0);
    }

    #[test]
    fn idle_cursor_keeps_context() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        let scored = score_session(&session(2_000_000_000, SessionKind::Cursor), now, None);
        assert_eq!(scored.recommendation.recommendation, Recommendation::Keep);
        assert_eq!(scored.recommendation.estimated_reclaimable_rss, 0);
        assert_eq!(scored.activity, SessionActivity::SleepingMemoryHeavy);
    }

    #[test]
    fn idle_light_agent_is_likely_forgotten_but_kept() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        let scored = score_session(&session(500_000_000, SessionKind::ClaudeCode), now, None);
        assert_eq!(scored.recommendation.recommendation, Recommendation::Keep);
        assert_eq!(scored.activity, SessionActivity::LikelyForgotten);
    }

    #[test]
    fn claude_with_disk_is_keep() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
        let mut value = session(2_000_000_000, SessionKind::ClaudeCode);
        value.disk.write_bytes = 4096;
        let scored = score_session(&value, now, None);
        assert_eq!(scored.recommendation.recommendation, Recommendation::Keep);
        assert_eq!(scored.activity, SessionActivity::Active);
    }
}
