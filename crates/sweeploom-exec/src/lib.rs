//! Plan → revalidate → execute → receipt.
//!
//! If a candidate changed, it is skipped. Never "delete anyway".

#![cfg_attr(not(test), warn(missing_docs))]

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use sweeploom_core::{
    Blocker, Candidate, CandidateId, CandidateOwner, CleanPlan, CleanPlanEntry, DeletionStrategy,
    ExecutionContext, ExecutionReport, MetadataRevision, PlanId, ProcessEvidence, ProcessSnapshot,
    Receipt, SafetyPrecondition, SkipReason, authorize_generated, reclaim_set,
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
        expected_latest_write: candidate.activity.latest_any_modified,
        expected_bytes: candidate.logical_bytes,
        strategy: candidate.deletion,
        required_safety: required,
    }
}

/// Revalidate a single entry. Fail closed.
#[must_use]
pub fn revalidate(entry: &CleanPlanEntry) -> Option<SkipReason> {
    revalidate_with(entry, &ExecutionContext::unknown())
}

/// Revalidate with shared evidence.
#[must_use]
pub fn revalidate_with(entry: &CleanPlanEntry, ctx: &ExecutionContext<'_>) -> Option<SkipReason> {
    let path = &entry.path;
    if is_symlink(path) {
        return Some(SkipReason::Changed);
    }
    if !path.exists() {
        return Some(SkipReason::Missing);
    }
    let is_dir = path.is_dir();
    if is_dir != entry.expected_is_dir {
        return Some(SkipReason::Changed);
    }
    if let Some(owner) = &entry.owner_path
        && authorize_generated(owner, path, None).is_err()
    {
        return Some(SkipReason::Blocked(Blocker::OutOfArtifactScope));
    }
    match file_identity(path) {
        Some(live) => {
            if let Some(expected) = entry.expected_identity
                && live != expected
            {
                return Some(SkipReason::Changed);
            }
        }
        None => {
            if entry
                .required_safety
                .contains(&SafetyPrecondition::FileIdentityMatch)
            {
                return Some(SkipReason::UnknownEvidence);
            }
        }
    }
    if let Some(reason) = revision_changed(entry, path) {
        return Some(reason);
    }
    if entry
        .required_safety
        .contains(&SafetyPrecondition::NoActiveProcess)
    {
        match ctx.processes {
            ProcessEvidence::Unknown => {
                return Some(SkipReason::UnknownEvidence);
            }
            ProcessEvidence::Observed(processes) => {
                if process_blocks(path, processes) {
                    return Some(SkipReason::Blocked(Blocker::ActiveProcess));
                }
            }
        }
    }
    if entry
        .required_safety
        .contains(&SafetyPrecondition::GitStateUnchanged)
    {
        let assessment = sweeploom_dev::inspect(path).assessment();
        if let Some(blocker) = assessment.blockers.first() {
            return Some(SkipReason::Blocked(*blocker));
        }
    }
    None
}

fn revision_changed(entry: &CleanPlanEntry, path: &Path) -> Option<SkipReason> {
    let live = capture_revision(path);
    if !live.complete {
        return Some(SkipReason::UnknownEvidence);
    }
    if let Ok(meta) = fs::metadata(path) {
        if meta.is_file() && meta.len() != entry.expected_bytes {
            return Some(SkipReason::Changed);
        }
        if let Some(expected) = entry.expected_latest_write
            && let Some(max) = live_max_mtime(&live, &meta)
            && max > expected
        {
            return Some(SkipReason::Changed);
        }
    } else {
        return Some(SkipReason::UnknownEvidence);
    }
    if let Some(expected) = entry.expected_revision {
        if expected.complete
            && (expected.file_count != live.file_count
                || expected.logical_bytes != live.logical_bytes
                || expected.max_mtime_unix_ms != live.max_mtime_unix_ms)
        {
            return Some(SkipReason::Changed);
        }
        if !expected.complete {
            return Some(SkipReason::UnknownEvidence);
        }
    }
    None
}

fn live_max_mtime(live: &MetadataRevision, meta: &fs::Metadata) -> Option<SystemTime> {
    live.max_mtime_unix_ms
        .map(|ms| UNIX_EPOCH + std::time::Duration::from_millis(ms))
        .or_else(|| meta.modified().ok())
}

fn process_blocks(path: &Path, processes: &[ProcessSnapshot]) -> bool {
    processes.iter().any(|process| {
        process
            .cwd
            .as_ref()
            .is_some_and(|cwd| cwd.starts_with(path) || path.starts_with(cwd))
    })
}

fn capture_revision(path: &Path) -> MetadataRevision {
    let Ok(meta) = fs::metadata(path) else {
        return MetadataRevision {
            file_count: 0,
            logical_bytes: 0,
            max_mtime_unix_ms: None,
            complete: false,
        };
    };
    if meta.is_file() {
        return MetadataRevision {
            file_count: 1,
            logical_bytes: meta.len(),
            max_mtime_unix_ms: mtime_ms(meta.modified().ok()),
            complete: true,
        };
    }
    let mut files = 0_u64;
    let mut bytes = 0_u64;
    let mut max_mtime = meta.modified().ok();
    let mut complete = true;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            complete = false;
            continue;
        };
        for entry in entries.flatten() {
            let child = entry.path();
            let Ok(child_meta) = fs::symlink_metadata(&child) else {
                complete = false;
                continue;
            };
            if child_meta.file_type().is_symlink() {
                continue;
            }
            DirectoryNode::bump(&mut max_mtime, child_meta.modified().ok());
            if child_meta.is_dir() {
                stack.push(child);
            } else {
                files += 1;
                bytes = bytes.saturating_add(child_meta.len());
                if files > 12_000 {
                    complete = false;
                    break;
                }
            }
        }
        if !complete && files > 12_000 {
            break;
        }
    }
    MetadataRevision {
        file_count: files,
        logical_bytes: bytes,
        max_mtime_unix_ms: mtime_ms(max_mtime),
        complete,
    }
}

struct DirectoryNode;

impl DirectoryNode {
    fn bump(slot: &mut Option<SystemTime>, candidate: Option<SystemTime>) {
        match (*slot, candidate) {
            (None, Some(value)) => *slot = Some(value),
            (Some(current), Some(value)) if value > current => *slot = Some(value),
            _ => {}
        }
    }
}

fn mtime_ms(time: Option<SystemTime>) -> Option<u64> {
    time.and_then(|stamp| stamp.duration_since(UNIX_EPOCH).ok())
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
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
    use std::path::PathBuf;
    use sweeploom_core::{
        ActivityEvidence, Candidate, CandidateKind, CandidateOwner, RebuildAssessment,
        SafetyAssessment, UserPolicy,
    };

    fn generated(id: u64, path: PathBuf, bytes: u64) -> Candidate {
        Candidate {
            id: CandidateId(id),
            kind: CandidateKind::BuildArtifact,
            owner: CandidateOwner::Project(sweeploom_core::ProjectId(
                path.parent().unwrap_or(&path).to_path_buf(),
            )),
            path,
            logical_bytes: bytes,
            allocated_bytes: None,
            file_count: 1,
            activity: ActivityEvidence::default(),
            safety: SafetyAssessment::safe(),
            rebuild: RebuildAssessment::default(),
            deletion: DeletionStrategy::PermanentGenerated,
            evidence: Vec::new(),
            user_policy: UserPolicy::Default,
        }
    }

    #[test]
    fn changed_descendant_is_skipped() {
        let root = std::env::temp_dir().join(format!("sweeploom-exec-rev-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let target = root.join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("artifact.bin"), b"one").unwrap();
        let candidate = generated(1, target.clone(), 3);
        let ctx = ExecutionContext::observed(&[]);
        let plan = build_plan_with(&[candidate], None, &ctx);
        fs::write(target.join("artifact.bin"), b"changed-after-plan").unwrap();
        let (report, _) = apply_plan_with(&plan, &ctx);
        assert_eq!(report.counts.deleted, 0);
        assert!(target.exists(), "changed descendant must survive");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn generated_directory_is_deleted_when_evidence_is_known() {
        let root = std::env::temp_dir().join(format!("sweeploom-exec-del-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let target = root.join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("a.bin"), b"stale").unwrap();
        let candidate = generated(2, target.clone(), 5);
        let ctx = ExecutionContext::observed(&[]);
        let plan = build_plan_with(&[candidate], None, &ctx);
        let (report, receipt) = apply_plan_with(&plan, &ctx);
        assert_eq!(report.counts.deleted, 1);
        assert_eq!(receipt.counts.deleted, 1);
        assert!(receipt.actual_free_space_delta.is_none());
        assert!(!target.exists(), "generated path must be removed");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn unknown_process_evidence_refuses_apply() {
        let root = std::env::temp_dir().join(format!("sweeploom-exec-unk-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let target = root.join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("a.bin"), b"stale").unwrap();
        let candidate = generated(3, target.clone(), 5);
        let planned = build_plan_with(&[candidate], None, &ExecutionContext::observed(&[]));
        let (report, _) = apply_plan_with(&planned, &ExecutionContext::unknown());
        assert_eq!(report.counts.deleted, 0);
        assert!(target.exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn src_is_never_planned() {
        let root = std::env::temp_dir().join(format!("sweeploom-exec-src-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), b"fn x() {}").unwrap();
        let candidate = generated(4, src.clone(), 8);
        let ctx = ExecutionContext::observed(&[]);
        let plan = build_plan_with(&[candidate], None, &ctx);
        assert!(plan.entries.is_empty());
        let _ = fs::remove_dir_all(&root);
    }
}
