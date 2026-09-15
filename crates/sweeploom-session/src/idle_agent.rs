//! Idle live-agent fixture used by WITH vs WITHOUT benches.

use std::time::{Duration, SystemTime};

use sweeploom_core::{
    LiveSession, Recommendation, SessionActivity, SessionDiskUsage, SessionId, SessionKind,
    SessionNetworkUsage, SessionRecommendation, SessionSafety,
};

use crate::score_session;

/// Agents the gold RAM bench covers.
pub const IDLE_BENCH_KINDS: &[SessionKind] = &[
    SessionKind::ClaudeCode,
    SessionKind::Codex,
    SessionKind::Cursor,
    SessionKind::OpenCode,
    SessionKind::Gemini,
    SessionKind::Grok,
];

/// Idle 2 GB agent. SweepLoom must Keep these.
#[must_use]
pub fn idle_2gb_agent(kind: SessionKind) -> LiveSession {
    LiveSession {
        id: SessionId(1),
        kind,
        project: None,
        processes: Vec::new(),
        started_at: Some(SystemTime::UNIX_EPOCH),
        observed_last_activity: Some(SystemTime::UNIX_EPOCH),
        rss_bytes: 2_000_000_000,
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

/// What a RAM-booster does without SweepLoom: idle + ≥1 GB → reclaim.
#[must_use]
pub fn naive_idle_reclaim(session: &LiveSession) -> Recommendation {
    if session.rss_bytes >= 1_000_000_000 && session.cpu_percent < 1.0 {
        Recommendation::Recommended
    } else {
        Recommendation::Keep
    }
}

/// Count agents a booster would kill vs SweepLoom Keep. Same clock as the session bench.
#[must_use]
pub fn idle_agent_survive() -> (usize, usize) {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
    let mut without_kill = 0;
    let mut with_keep = 0;
    for &kind in IDLE_BENCH_KINDS {
        let raw = idle_2gb_agent(kind);
        if naive_idle_reclaim(&raw) != Recommendation::Keep {
            without_kill += 1;
        }
        let scored = score_session(&raw, now, None);
        if scored.recommendation.recommendation == Recommendation::Keep {
            with_keep += 1;
        }
    }
    (without_kill, with_keep)
}
