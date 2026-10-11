use super::*;
use sweeploom_core::*;
fn row(path: &str, bytes: u64) -> ReviewRow {
    ReviewRow {
        title: "Downloads".into(),
        selected: true,
        candidate: Candidate {
            id: CandidateId(bytes),
            kind: CandidateKind::OldInstaller,
            owner: CandidateOwner::User,
            path: path.into(),
            logical_bytes: bytes,
            allocated_bytes: None,
            file_count: 1,
            activity: ActivityEvidence::default(),
            safety: SafetyAssessment::review(),
            rebuild: RebuildAssessment::default(),
            deletion: DeletionStrategy::InspectOnly,
            evidence: vec![],
            user_policy: UserPolicy::Default,
        },
    }
}
#[test]
fn selection_does_not_authorize_generated_deletion_and_sizes_do_not_overlap() {
    let mut item = row("/home/Downloads/installer.dmg", 10);
    assert!(can_select(&item));
    assert!(!can_auto_select(&item));
    assert!(!crate::disk_actions::can_clean(&item.candidate));
    item.candidate.safety = SafetyAssessment::blocked(Blocker::ProtectedPath);
    assert!(can_select(&item));
    assert!(!crate::disk_actions::can_clean(&item.candidate));
    let rows = [row("/home/target", 100), row("/home/target/debug", 80)];
    assert_eq!(selected_bytes(&rows), 100);
}
