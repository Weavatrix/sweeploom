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
    let server = crate::tests::proc(12, Some(11), "node", None, &["node", "mcp-server"], 1, 0.0);
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
