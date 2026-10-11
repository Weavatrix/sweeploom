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
fn descendant_newer_than_directory_is_not_a_post_plan_write() {
    let root =
        std::env::temp_dir().join(format!("sweeploom-exec-dir-mtime-{}", std::process::id()));
    let target = root.join("target");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("a.bin"), b"stale").unwrap();
    let old = SystemTime::now() - std::time::Duration::from_secs(86400);
    open_dir(&target)
        .set_times(fs::FileTimes::new().set_modified(old))
        .unwrap();
    let mut candidate = generated(55, target.clone(), 5);
    candidate.activity.latest_any_modified = Some(old);
    let ctx = ExecutionContext::observed(&[]);
    let plan = build_plan_with(&[candidate], None, &ctx);
    assert_eq!(revalidate_with(&plan.entries[0], &ctx), None);
    fs::remove_dir_all(root).unwrap();
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

/// A directory handle that can change its times. Windows only opens
/// directories with `FILE_FLAG_BACKUP_SEMANTICS` and attribute write access.
fn open_dir(path: &std::path::Path) -> fs::File {
    #[cfg(windows)]
    let file = {
        use std::os::windows::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .write(true)
            .custom_flags(0x0200_0000)
            .open(path)
    };
    #[cfg(not(windows))]
    let file = fs::File::open(path);
    file.unwrap()
}
