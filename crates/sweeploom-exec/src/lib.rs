//! Plan → revalidate → execute → receipt.
//!
//! If a candidate changed, it is skipped. Never "delete anyway".

#![cfg_attr(not(test), warn(missing_docs))]

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use sweeploom_core::{
    Candidate, CandidateId, CleanPlan, CleanPlanEntry, DeletionStrategy, ExecutionReport, PlanId,
    Receipt, SafetyPrecondition, SkipReason,
};

/// Build an immutable plan from selected candidates.
#[must_use]
pub fn build_plan(candidates: &[Candidate], requested_free_bytes: Option<u64>) -> CleanPlan {
    let entries = candidates
        .iter()
        .filter(|candidate| !candidate.safety.is_blocked())
        .filter(|candidate| candidate.deletion != DeletionStrategy::InspectOnly)
        .map(|candidate| CleanPlanEntry {
            candidate_id: candidate.id,
            path: candidate.path.clone(),
            expected_identity: None,
            expected_latest_write: candidate.activity.latest_any_modified,
            expected_bytes: candidate.logical_bytes,
            strategy: candidate.deletion,
            required_safety: vec![
                SafetyPrecondition::PathKindUnchanged,
                SafetyPrecondition::NoNewerWrites,
                SafetyPrecondition::NoSymlinkEscape,
                SafetyPrecondition::GitStateUnchanged,
            ],
        })
        .collect::<Vec<_>>();
    let estimated_reclaimable_bytes = candidates
        .iter()
        .filter(|candidate| !candidate.safety.is_blocked())
        .map(Candidate::estimated_reclaimable_bytes)
        .sum();
    CleanPlan {
        version: CleanPlan::VERSION,
        id: PlanId(now_id()),
        created_at: SystemTime::now(),
        entries,
        requested_free_bytes,
        estimated_reclaimable_bytes,
    }
}

/// Revalidate a single entry. Fail closed.
#[must_use]
pub fn revalidate(entry: &CleanPlanEntry) -> Option<SkipReason> {
    let path = &entry.path;
    if is_symlink(path) {
        return Some(SkipReason::Changed);
    }
    if !path.exists() {
        return Some(SkipReason::Missing);
    }
    if let Some(expected) = entry.expected_latest_write
        && let Ok(meta) = fs::metadata(path)
        && let Ok(modified) = meta.modified()
        && modified > expected
    {
        return Some(SkipReason::Changed);
    }
    if entry
        .required_safety
        .contains(&SafetyPrecondition::GitStateUnchanged)
    {
        let assessment = sweeploom_dev::inspect(path).assessment();
        if let Some(blocker) = assessment.blockers.first() {
            return Some(SkipReason::Blocked(blocker.clone()));
        }
    }
    None
}

/// Apply a plan. Only `PermanentGenerated` of empty-enough test dirs is
/// implemented in this first cut; everything else is skipped as inspect-only.
#[must_use]
pub fn apply_plan(plan: &CleanPlan) -> (ExecutionReport, Receipt) {
    let started = SystemTime::now();
    let mut report = ExecutionReport::default();
    for entry in &plan.entries {
        if let Some(reason) = revalidate(entry) {
            if matches!(reason, SkipReason::Changed) {
                report.counts.skipped_changed += 1;
            }
            report.skipped.push((entry.candidate_id, reason));
            continue;
        }
        match entry.strategy {
            DeletionStrategy::PermanentGenerated => match delete_generated(&entry.path) {
                Ok(()) => report.counts.deleted += 1,
                Err(message) => {
                    report.counts.failed += 1;
                    report.failures.push((entry.candidate_id, message));
                }
            },
            _ => {
                report
                    .skipped
                    .push((entry.candidate_id, SkipReason::Cancelled));
            }
        }
    }
    let finished = SystemTime::now();
    let receipt = Receipt {
        plan: plan.id,
        started,
        finished,
        selected_logical_bytes: plan.entries.iter().map(|item| item.expected_bytes).sum(),
        estimated_physical_bytes: plan.estimated_reclaimable_bytes,
        actual_free_space_delta: 0,
        counts: report.counts,
    };
    (report, receipt)
}

fn delete_generated(path: &Path) -> Result<(), String> {
    if is_symlink(path) {
        return Err("refusing to delete symlink".to_owned());
    }
    let meta = fs::metadata(path).map_err(|error| error.to_string())?;
    if meta.is_dir() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())
    } else {
        fs::remove_file(path).map_err(|error| error.to_string())
    }
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

fn now_id() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(1, |item| item.as_millis() as u64)
}

/// Helper used by tests to build a synthetic candidate id.
#[must_use]
pub const fn test_candidate_id(raw: u64) -> CandidateId {
    CandidateId(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use sweeploom_core::{
        ActivityEvidence, Candidate, CandidateKind, CandidateOwner, RebuildAssessment,
        SafetyAssessment, UserPolicy,
    };

    #[test]
    fn changed_file_is_skipped_not_deleted() {
        let root = std::env::temp_dir().join(format!("sweeploom-exec-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let target = root.join("stale.bin");
        fs::write(&target, b"one").unwrap();
        let candidate = Candidate {
            id: CandidateId(1),
            kind: CandidateKind::BuildArtifact,
            owner: CandidateOwner::User,
            path: target.clone(),
            logical_bytes: 3,
            allocated_bytes: None,
            file_count: 1,
            activity: ActivityEvidence {
                latest_any_modified: Some(SystemTime::UNIX_EPOCH),
                ..ActivityEvidence::default()
            },
            safety: SafetyAssessment::safe(),
            rebuild: RebuildAssessment::default(),
            deletion: DeletionStrategy::PermanentGenerated,
            evidence: Vec::new(),
            user_policy: UserPolicy::Default,
        };
        let plan = build_plan(&[candidate], None);
        fs::write(&target, b"changed-after-plan").unwrap();
        let (report, _) = apply_plan(&plan);
        assert_eq!(report.counts.skipped_changed, 1);
        assert_eq!(report.counts.deleted, 0);
        assert!(target.exists(), "changed candidate must survive");
        let _ = fs::remove_dir_all(&root);
    }
}
