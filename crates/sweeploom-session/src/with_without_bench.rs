//! RAM-booster AI (no SweepLoom) vs SweepLoom Keep for live agents.

use sweeploom_core::Recommendation;

use super::{IDLE_BENCH_KINDS, idle_2gb_agent, naive_idle_reclaim, score_session};

use std::time::{Duration, SystemTime};

#[test]
fn bench_agent_survive_with_and_without_sweeploom() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);

    eprintln!("=== live agents: WITHOUT SweepLoom (RAM booster) vs WITH ===");
    for &kind in IDLE_BENCH_KINDS {
        let raw = idle_2gb_agent(kind);
        let without = naive_idle_reclaim(&raw);
        let with = score_session(&raw, now, None);
        eprintln!(
            "  idle {kind:?} 2GB  WITHOUT={without:?} (would drop chat)  WITH={:?} reclaim={}",
            with.recommendation.recommendation, with.recommendation.estimated_reclaimable_rss
        );
        assert_eq!(without, Recommendation::Recommended);
        assert_eq!(with.recommendation.recommendation, Recommendation::Keep);
        assert_eq!(with.recommendation.estimated_reclaimable_rss, 0);
    }

    let (kill, keep) = super::idle_agent_survive();
    assert_eq!(kill, IDLE_BENCH_KINDS.len());
    assert_eq!(keep, IDLE_BENCH_KINDS.len());
}
