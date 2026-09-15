//! Auto-plan eligibility. Safety, policy, and recency are separate from checkboxes.

use std::time::{Duration, SystemTime};

use crate::candidate::Candidate;
use crate::plan::DeletionStrategy;
use crate::policy::UserPolicy;

/// Window during which a write forbids automatic cleanup.
pub const RECENT_WRITE: Duration = Duration::from_secs(15 * 60);

/// True when a planner may auto-select this candidate.
#[must_use]
pub fn auto_eligible(candidate: &Candidate, now: SystemTime) -> bool {
    if candidate.safety.is_blocked() {
        return false;
    }
    if candidate.deletion == DeletionStrategy::InspectOnly {
        return false;
    }
    if !matches!(
        candidate.user_policy,
        UserPolicy::Default | UserPolicy::AlwaysCleanWhenCold
    ) {
        return false;
    }
    if recent_write(candidate, now) {
        return false;
    }
    true
}

fn recent_write(candidate: &Candidate, now: SystemTime) -> bool {
    candidate
        .activity
        .latest_any_modified
        .and_then(|stamp| now.duration_since(stamp).ok())
        .is_some_and(|age| age < RECENT_WRITE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::CandidateId;
    use crate::rebuild::RebuildAssessment;
    use crate::safety::SafetyAssessment;
    use crate::{ActivityEvidence, CandidateKind, CandidateOwner};
    use std::path::PathBuf;

    fn candidate(policy: UserPolicy, modified: Option<SystemTime>) -> Candidate {
        Candidate {
            id: CandidateId(1),
            kind: CandidateKind::TempFile,
            owner: CandidateOwner::User,
            path: PathBuf::from("/tmp/a.tmp"),
            logical_bytes: 8,
            allocated_bytes: None,
            file_count: 1,
            activity: ActivityEvidence {
                latest_any_modified: modified,
                ..ActivityEvidence::default()
            },
            safety: SafetyAssessment::safe(),
            rebuild: RebuildAssessment::default(),
            deletion: DeletionStrategy::PermanentGenerated,
            evidence: Vec::new(),
            user_policy: policy,
        }
    }

    #[test]
    fn policy_and_fresh_write_block_auto_plan() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(3_600);
        assert!(auto_eligible(
            &candidate(UserPolicy::Default, Some(SystemTime::UNIX_EPOCH)),
            now
        ));
        assert!(!auto_eligible(
            &candidate(UserPolicy::NeverClean, Some(SystemTime::UNIX_EPOCH)),
            now
        ));
        assert!(!auto_eligible(
            &candidate(UserPolicy::PinProject, Some(SystemTime::UNIX_EPOCH)),
            now
        ));
        assert!(!auto_eligible(
            &candidate(UserPolicy::Default, Some(now - Duration::from_secs(60))),
            now
        ));
    }
}
