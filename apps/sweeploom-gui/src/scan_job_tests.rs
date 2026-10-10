use super::merge::{merge_rows, replace_rows};
use super::*;
use std::fs;
use std::time::Duration;
use sweeploom_core::{
    ActivityEvidence, Blocker, CandidateId, CandidateKind, CandidateOwner, DeletionStrategy,
    RebuildAssessment, RebuildCost, SafetyAssessment, UserPolicy,
};

#[test]
fn explorer_walk_finishes_without_waiting_for_review_or_ai_stores() {
    let root =
        std::env::temp_dir().join(format!("sweeploom-independent-scan-{}", std::process::id()));
    fs::create_dir_all(root.join("child")).unwrap();
    fs::write(root.join("child/file"), b"scan").unwrap();
    let rx = spawn(root.clone());
    let mut tree_seen = false;
    loop {
        match rx.recv_timeout(Duration::from_secs(10)).unwrap() {
            ScanMsg::Progress(_) => {}
            ScanMsg::Tree(report) => {
                assert_eq!(report.tree.files, 1);
                tree_seen = true;
            }
            ScanMsg::Finished(result) => {
                assert!(result.is_ok());
                assert!(tree_seen);
                break;
            }
        }
    }
    fs::remove_dir_all(root).unwrap();
}

fn isolated_locations(root: PathBuf) -> UserLocations {
    UserLocations {
        downloads: Some(root.join("Downloads")),
        temp: root.join("tmp"),
        cache: None,
        app_config: root.join("cfg"),
        app_data: root.join("data"),
        home: root,
    }
}

#[test]
fn review_thread_streams_projects_then_rows_then_finishes() {
    let root = std::env::temp_dir().join(format!(
        "sweeploom-rebuild-{}-{}",
        std::process::id(),
        crate::scan_history::now_ms()
    ));
    let project = root.join("dev").join("web");
    fs::create_dir_all(project.join("node_modules")).unwrap();
    fs::create_dir_all(root.join("tmp")).unwrap();
    fs::write(project.join("package.json"), "{}").unwrap();
    fs::write(project.join("node_modules").join("dep.js"), vec![1_u8; 64]).unwrap();
    let rx = spawn_review(
        Some(root.clone()),
        Vec::new(),
        None,
        Vec::new(),
        isolated_locations(root.clone()),
    );
    let first = rx.recv_timeout(Duration::from_secs(30)).expect("projects");
    let RebuildMsg::Projects(projects) = first else {
        panic!("projects are discovered first");
    };
    let project = fs::canonicalize(&project).unwrap();
    assert!(projects.contains(&project), "{projects:?}");
    let mut streamed = Vec::new();
    let outcome = loop {
        match rx.recv_timeout(Duration::from_secs(30)).expect("rebuild") {
            RebuildMsg::Rows(rows) => streamed.extend(rows),
            RebuildMsg::Finished(outcome) => break outcome,
            RebuildMsg::Projects(_) => panic!("projects are sent once"),
        }
    };
    let _ = fs::remove_dir_all(&root);
    let (_, rows) = outcome.expect("rebuild finished");
    let node_modules = project.join("node_modules");
    assert!(streamed.iter().any(|row| row.candidate.path == node_modules));
    assert!(rows.iter().any(|row| row.candidate.path == node_modules));
}

fn row(path: &str, bytes: u64, selected: bool) -> ReviewRow {
    ReviewRow {
        candidate: Candidate {
            id: CandidateId(0),
            kind: CandidateKind::BuildArtifact,
            owner: CandidateOwner::Project(sweeploom_core::ProjectId(PathBuf::from("/p"))),
            path: PathBuf::from(path),
            logical_bytes: bytes,
            allocated_bytes: None,
            file_count: 0,
            activity: ActivityEvidence::default(),
            safety: SafetyAssessment::safe(),
            rebuild: RebuildAssessment {
                cost: RebuildCost::Low,
                observed_duration_ms: None,
            },
            deletion: DeletionStrategy::PermanentGenerated,
            evidence: Vec::new(),
            user_policy: UserPolicy::Default,
        },
        selected,
        title: path.to_owned(),
    }
}

#[test]
fn streamed_rows_update_in_place_and_keep_every_other_row() {
    let mut review = vec![row("/p/target", 10, false), row("/q/node_modules", 20, true)];
    merge_rows(&mut review, vec![row("/p/target", 30, true), row("/r/.venv", 5, false)]);
    let paths: Vec<_> = review.iter().map(|item| item.candidate.path.clone()).collect();
    assert_eq!(
        paths,
        ["/p/target", "/q/node_modules", "/r/.venv"].map(PathBuf::from)
    );
    assert_eq!(review[0].candidate.logical_bytes, 30);
    assert!(!review[0].selected, "the user's earlier choice wins");
    assert!(review[1].selected);
    let ids: Vec<_> = review.iter().map(|item| item.candidate.id.0).collect();
    assert_eq!(ids, [1, 2, 3]);
}

#[test]
fn final_rows_drop_vanished_paths_and_never_keep_blocked_rows_selected() {
    let mut review = vec![row("/p/target", 10, true), row("/gone", 1, true)];
    let mut blocked = row("/p/target", 12, false);
    blocked.candidate.safety = SafetyAssessment::blocked(Blocker::DirtyTrackedFiles);
    replace_rows(&mut review, vec![blocked, row("/new", 3, true)]);
    assert_eq!(review.len(), 2);
    assert!(!review[0].selected);
    assert_eq!(review[1].candidate.path, PathBuf::from("/new"));
    assert!(review.iter().all(|item| item.candidate.path != PathBuf::from("/gone")));
}
