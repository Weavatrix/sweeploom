//! Least-pain session selection. Never terminates; the user still confirms.

use sweeploom_core::{LiveSession, ProjectId, Recommendation, SessionId, SessionKind};

/// True when a session may be offered for RAM/CPU reclaim.
#[must_use]
pub fn is_reclaim_candidate(session: &LiveSession) -> bool {
    !session.safety.terminate_disabled
        && !session.safety.assessment.is_blocked()
        && session.recommendation.recommendation != Recommendation::Keep
        && session.kind != SessionKind::Browser
}

/// Pick forgotten sessions until estimated reclaimable RSS reaches `target_bytes`.
///
/// Strongly recommended first, then largest estimate, so the set stays small.
#[must_use]
pub fn plan_free_ram(sessions: &[LiveSession], target_bytes: u64) -> Vec<SessionId> {
    if target_bytes == 0 {
        return Vec::new();
    }
    let mut order: Vec<usize> = sessions
        .iter()
        .enumerate()
        .filter(|(_, session)| {
            is_reclaim_candidate(session) && session.recommendation.estimated_reclaimable_rss > 0
        })
        .map(|(index, _)| index)
        .collect();
    order.sort_by(|&left, &right| ram_order(&sessions[left], &sessions[right]));
    let mut acc = 0_u64;
    let mut ids = Vec::new();
    for index in order {
        if acc >= target_bytes {
            break;
        }
        acc = acc.saturating_add(sessions[index].recommendation.estimated_reclaimable_rss);
        ids.push(sessions[index].id);
    }
    ids
}

/// Pick forgotten sessions until combined CPU percent reaches `target_percent`.
#[must_use]
pub fn plan_reduce_cpu(sessions: &[LiveSession], target_percent: f32) -> Vec<SessionId> {
    if target_percent <= 0.0 {
        return Vec::new();
    }
    let mut order: Vec<usize> = sessions
        .iter()
        .enumerate()
        .filter(|(_, session)| is_reclaim_candidate(session) && session.cpu_percent > 0.05)
        .map(|(index, _)| index)
        .collect();
    order.sort_by(|&left, &right| cpu_order(&sessions[left], &sessions[right]));
    let mut acc = 0.0_f32;
    let mut ids = Vec::new();
    for index in order {
        if acc >= target_percent {
            break;
        }
        acc += sessions[index].cpu_percent;
        ids.push(sessions[index].id);
    }
    ids
}

/// Forgotten dev sessions for a quiet workstation. Protects the current project
/// and never includes browser trees or system-critical processes.
#[must_use]
pub fn plan_quiet_workstation(
    sessions: &[LiveSession],
    current_project: Option<&ProjectId>,
) -> Vec<SessionId> {
    sessions
        .iter()
        .filter(|session| is_quiet_candidate(session, current_project))
        .map(|session| session.id)
        .collect()
}

fn is_quiet_candidate(session: &LiveSession, current_project: Option<&ProjectId>) -> bool {
    if !is_reclaim_candidate(session) {
        return false;
    }
    match session.recommendation.recommendation {
        Recommendation::StronglyRecommended | Recommendation::Recommended => {}
        Recommendation::Optional | Recommendation::Keep => return false,
    }
    !matches!(
        (current_project, session.project.as_ref()),
        (Some(current), Some(project)) if current == project
    )
}

fn ram_order(left: &LiveSession, right: &LiveSession) -> std::cmp::Ordering {
    left.recommendation
        .recommendation
        .cmp(&right.recommendation.recommendation)
        .then_with(|| {
            right
                .recommendation
                .estimated_reclaimable_rss
                .cmp(&left.recommendation.estimated_reclaimable_rss)
        })
}

fn cpu_order(left: &LiveSession, right: &LiveSession) -> std::cmp::Ordering {
    left.recommendation
        .recommendation
        .cmp(&right.recommendation.recommendation)
        .then_with(|| {
            right
                .cpu_percent
                .partial_cmp(&left.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    use sweeploom_core::{
        SessionActivity, SessionDiskUsage, SessionKind, SessionNetworkUsage, SessionRecommendation,
        SessionSafety,
    };

    fn session(
        id: u64,
        rec: Recommendation,
        reclaim: u64,
        cpu: f32,
        critical: bool,
    ) -> LiveSession {
        LiveSession {
            id: SessionId(id),
            kind: SessionKind::ClaudeCode,
            project: None,
            processes: Vec::new(),
            started_at: Some(SystemTime::UNIX_EPOCH),
            observed_last_activity: Some(SystemTime::UNIX_EPOCH),
            rss_bytes: reclaim,
            cpu_percent: cpu,
            disk: SessionDiskUsage::default(),
            network: SessionNetworkUsage::default(),
            activity: SessionActivity::Idle,
            safety: if critical {
                SessionSafety::system_critical()
            } else {
                SessionSafety::user()
            },
            recommendation: SessionRecommendation {
                recommendation: rec,
                estimated_reclaimable_rss: reclaim,
            },
        }
    }

    #[test]
    fn ram_plan_skips_keep_and_critical() {
        let sessions = [
            session(1, Recommendation::Keep, 8_000_000_000, 0.0, false),
            session(
                2,
                Recommendation::StronglyRecommended,
                1_000_000_000,
                0.0,
                true,
            ),
            session(3, Recommendation::Recommended, 2_000_000_000, 0.0, false),
        ];
        let ids = plan_free_ram(&sessions, 1_500_000_000);
        assert_eq!(ids, vec![SessionId(3)]);
    }

    #[test]
    fn ram_plan_prefers_strongly_recommended() {
        let sessions = [
            session(1, Recommendation::Optional, 5_000_000_000, 0.0, false),
            session(
                2,
                Recommendation::StronglyRecommended,
                1_000_000_000,
                0.0,
                false,
            ),
        ];
        let ids = plan_free_ram(&sessions, 500_000_000);
        assert_eq!(ids, vec![SessionId(2)]);
    }

    #[test]
    fn cpu_plan_takes_forgotten_cpu_first() {
        let sessions = [
            session(1, Recommendation::Keep, 0, 40.0, false),
            session(2, Recommendation::Recommended, 0, 8.0, false),
            session(3, Recommendation::Optional, 0, 3.0, false),
        ];
        let ids = plan_reduce_cpu(&sessions, 7.0);
        assert_eq!(ids, vec![SessionId(2)]);
    }

    #[test]
    fn ram_plan_skips_browser_trees() {
        let mut browser = session(4, Recommendation::Recommended, 9_000_000_000, 0.0, false);
        browser.kind = SessionKind::Browser;
        let ids = plan_free_ram(&[browser], 1_000_000_000);
        assert!(ids.is_empty());
    }

    #[test]
    fn quiet_skips_current_project_browser_and_optional() {
        let mut current = session(1, Recommendation::Recommended, 1, 1.0, false);
        current.project = Some(ProjectId(std::path::PathBuf::from("/work/now")));
        let mut browser = session(2, Recommendation::Recommended, 1, 1.0, false);
        browser.kind = SessionKind::Browser;
        let optional = session(3, Recommendation::Optional, 1, 1.0, false);
        let forgotten = session(4, Recommendation::Recommended, 1, 1.0, false);
        let ids = plan_quiet_workstation(
            &[current, browser, optional, forgotten],
            Some(&ProjectId(std::path::PathBuf::from("/work/now"))),
        );
        assert_eq!(ids, vec![SessionId(4)]);
    }
}
