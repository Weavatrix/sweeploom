//! Plan → revalidate → execute → receipt.
//!
//! If a candidate changed, it is skipped. Never "delete anyway".

#![cfg_attr(not(test), warn(missing_docs))]

mod manual_trash;
mod revalidation;
pub use manual_trash::{
    TrashTarget, apply_trash_using, apply_trash_with, prepare_trash, trash_path_reason,
};
pub use revalidation::{revalidate, revalidate_with};

use revalidation::{capture_revision, is_symlink};

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use sweeploom_core::{
    Candidate, CandidateId, CandidateOwner, CleanPlan, CleanPlanEntry, DeletionStrategy,
    ExecutionContext, ExecutionReport, PlanId, Receipt, SafetyPrecondition, SkipReason,
    authorize_generated, reclaim_set,
};
use sweeploom_platform::file_identity;

/// Build an immutable plan from selected candidates.
#[must_use]
pub fn build_plan(candidates: &[Candidate], requested_free_bytes: Option<u64>) -> CleanPlan {
    build_plan_with(
        candidates,
        requested_free_bytes,
        &ExecutionContext::unknown(),
    )
}

/// Build a plan with shared GUI/CLI evidence.
#[must_use]
pub fn build_plan_with(
    candidates: &[Candidate],
    requested_free_bytes: Option<u64>,
    ctx: &ExecutionContext<'_>,
) -> CleanPlan {
    let now = SystemTime::now();
    let _ = now;
    let entries = candidates
        .iter()
        .filter(|candidate| apply_allowed(candidate))
        .map(|candidate| plan_entry(candidate, ctx))
        .collect::<Vec<_>>();
    let unique = reclaim_set(
        &candidates
            .iter()
            .filter(|candidate| {
                candidate.deletion != DeletionStrategy::InspectOnly
                    && !candidate.safety.is_blocked()
            })
            .cloned()
            .collect::<Vec<_>>(),
    );
    CleanPlan {
        version: CleanPlan::VERSION,
        id: PlanId(now_id()),
        created_at: now,
        entries,
        requested_free_bytes,
        estimated_reclaimable_bytes: unique.unique_estimate.bytes,
    }
}

fn apply_allowed(candidate: &Candidate) -> bool {
    if candidate.safety.is_blocked() || candidate.deletion == DeletionStrategy::InspectOnly {
        return false;
    }
    if matches!(
        candidate.user_policy,
        sweeploom_core::UserPolicy::NeverClean
            | sweeploom_core::UserPolicy::Keep
            | sweeploom_core::UserPolicy::PinProject
    ) {
        return false;
    }
    artifact_allowed(candidate)
}

fn artifact_allowed(candidate: &Candidate) -> bool {
    match &candidate.owner {
        CandidateOwner::Project(owner) => {
            authorize_generated(&owner.0, &candidate.path, None).is_ok()
        }
        _ => true,
    }
}

fn plan_entry(candidate: &Candidate, ctx: &ExecutionContext<'_>) -> CleanPlanEntry {
    let owner_path = match &candidate.owner {
        CandidateOwner::Project(id) => Some(id.0.clone()),
        _ => None,
    };
    let identity = file_identity(&candidate.path);
    let revision = capture_revision(&candidate.path);
    let _ = ctx;
    let required = vec![
        SafetyPrecondition::PathKindUnchanged,
        SafetyPrecondition::FileIdentityMatch,
        SafetyPrecondition::NoNewerWrites,
        SafetyPrecondition::NoSymlinkEscape,
        SafetyPrecondition::GitStateUnchanged,
        SafetyPrecondition::NoActiveProcess,
    ];
    CleanPlanEntry {
        candidate_id: candidate.id,
        path: candidate.path.clone(),
        owner_path,
        expected_identity: identity,
        expected_revision: Some(revision),
        expected_is_dir: candidate.path.is_dir(),
        // Directory mtime does not track writes in descendants. The plan's
        // complete metadata revision is the authoritative baseline.
        expected_latest_write: revision
            .max_mtime_unix_ms
            .map(|ms| UNIX_EPOCH + std::time::Duration::from_millis(ms)),
        expected_bytes: candidate.logical_bytes,
        strategy: candidate.deletion,
        required_safety: required,
    }
}

/// Apply a plan. `PermanentGenerated` deletes after revalidation;
/// every other strategy is skipped as inspect-only.
#[must_use]
pub fn apply_plan(plan: &CleanPlan) -> (ExecutionReport, Receipt) {
    apply_plan_with(plan, &ExecutionContext::unknown())
}

/// Apply with shared GUI/CLI evidence.
#[must_use]
pub fn apply_plan_with(plan: &CleanPlan, ctx: &ExecutionContext<'_>) -> (ExecutionReport, Receipt) {
    let started = SystemTime::now();
    let mut report = ExecutionReport::default();
    for entry in &plan.entries {
        if let Some(reason) = revalidate_with(entry, ctx) {
            match reason {
                SkipReason::Changed => report.counts.skipped_changed += 1,
                SkipReason::UnknownEvidence | SkipReason::Unsupported => {
                    report.counts.failed += 1;
                }
                _ => {}
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
                    .push((entry.candidate_id, SkipReason::Unsupported));
            }
        }
    }
    let finished = SystemTime::now();
    let unique = plan.entries.iter().map(|item| item.expected_bytes).sum();
    let receipt = Receipt {
        plan: plan.id,
        started,
        finished,
        selected_logical_bytes: unique,
        estimated_physical_bytes: plan.estimated_reclaimable_bytes,
        actual_free_space_delta: None,
        counts: report.counts,
    };
    (report, receipt)
}

fn delete_generated(path: &Path) -> Result<(), String> {
    if is_symlink(path) {
        return Err("refusing to delete symlink".to_owned());
    }
    if let Some(parent) = path.parent()
        && authorize_generated(parent, path, None).is_err()
    {
        // Repeat the boundary even when the owner was not a project.
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("src"))
        {
            return Err("refusing to delete protected path".to_owned());
        }
    }
    let meta = fs::metadata(path).map_err(|error| error.to_string())?;
    if meta.is_dir() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())
    } else {
        fs::remove_file(path).map_err(|error| error.to_string())
    }
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
#[path = "lib_tests.rs"]
mod tests;
