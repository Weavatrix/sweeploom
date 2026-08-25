//! Turn analyzer offers into CleanPlan candidates.

use std::path::Path;

use sweeploom_core::{
    ActivityEvidence, Candidate, CandidateId, CandidateKind, CandidateOwner, DeletionStrategy,
    Evidence, ProcessSnapshot, ProjectId, RebuildAssessment, RebuildCost, SafetyAssessment,
    UserPolicy,
};

use crate::cargo::{CargoOffer, CargoTrim, cargo_offers};
use crate::node::{NodeOffer, node_offers};
use crate::python::{PythonOffer, python_offers};
use crate::size::path_mtime;
use sweeploom_storage::discover_projects;

/// A review row: candidate plus whether the UI pre-selects it.
#[derive(Clone, Debug)]
pub struct ReviewRow {
    /// Cleanup candidate.
    pub candidate: Candidate,
    /// Pre-selected when SAFE generated.
    pub selected: bool,
    /// Human title.
    pub title: String,
}

/// Discover projects under `root` (skipping generated trees) and collect offers.
#[must_use]
pub fn collect_review_from(
    root: &Path,
    processes: &[ProcessSnapshot],
    max_projects: usize,
) -> Vec<ReviewRow> {
    collect_review(&discover_projects(root, max_projects), processes)
}

/// Collect Cargo, Node, and Python offers for discovered projects.
#[must_use]
pub fn collect_review(
    projects: &[impl AsRef<Path>],
    processes: &[ProcessSnapshot],
) -> Vec<ReviewRow> {
    let mut rows = Vec::new();
    let mut id = 1_u64;
    for project in projects {
        let project = project.as_ref();
        for offer in cargo_offers(project, processes) {
            rows.push(cargo_row(offer, id));
            id += 1;
        }
        for offer in node_offers(project, processes) {
            rows.push(node_row(offer, id));
            id += 1;
        }
        for offer in python_offers(project, processes) {
            rows.push(python_row(offer, id));
            id += 1;
        }
    }
    rows
}

fn cargo_row(offer: CargoOffer, id: u64) -> ReviewRow {
    let selected = !offer.blocked && matches!(offer.mode, CargoTrim::Light);
    let title = format!("Cargo {:?} · {}", offer.mode, offer.path.display());
    let safety = safety_of(offer.blocked, offer.blocker);
    let activity = generated_activity(&offer.path);
    ReviewRow {
        candidate: Candidate {
            id: CandidateId(id),
            kind: CandidateKind::BuildArtifact,
            owner: CandidateOwner::Project(ProjectId(offer.project)),
            path: offer.path,
            logical_bytes: offer.logical_bytes,
            allocated_bytes: None,
            file_count: 0,
            activity,
            safety,
            rebuild: RebuildAssessment {
                cost: offer.rebuild,
                observed_duration_ms: None,
            },
            deletion: DeletionStrategy::PermanentGenerated,
            evidence: vec![Evidence::exact("cargo-generated", title.clone())],
            user_policy: UserPolicy::Default,
        },
        selected,
        title,
    }
}

fn node_row(offer: NodeOffer, id: u64) -> ReviewRow {
    let title = format!("node_modules · {}", offer.path.display());
    let safety = safety_of(offer.blocked, offer.blocker);
    let activity = generated_activity(&offer.path);
    ReviewRow {
        candidate: Candidate {
            id: CandidateId(id),
            kind: CandidateKind::DependencyTree,
            owner: CandidateOwner::Project(ProjectId(offer.project)),
            path: offer.path,
            logical_bytes: offer.logical_bytes,
            allocated_bytes: None,
            file_count: 0,
            activity,
            safety,
            rebuild: RebuildAssessment {
                cost: offer.rebuild,
                observed_duration_ms: None,
            },
            deletion: DeletionStrategy::PermanentGenerated,
            evidence: vec![Evidence::exact("node-modules", title.clone())],
            user_policy: UserPolicy::Default,
        },
        selected: false,
        title,
    }
}

fn python_row(offer: PythonOffer, id: u64) -> ReviewRow {
    let title = format!("Python {} · {}", offer.label, offer.path.display());
    let safety = safety_of(offer.blocked, offer.blocker);
    let activity = generated_activity(&offer.path);
    let kind = if offer.rebuild == RebuildCost::High {
        CandidateKind::DependencyTree
    } else {
        CandidateKind::BuildArtifact
    };
    ReviewRow {
        candidate: Candidate {
            id: CandidateId(id),
            kind,
            owner: CandidateOwner::Project(ProjectId(offer.project)),
            path: offer.path,
            logical_bytes: offer.logical_bytes,
            allocated_bytes: None,
            file_count: 0,
            activity,
            safety,
            rebuild: RebuildAssessment {
                cost: offer.rebuild,
                observed_duration_ms: None,
            },
            deletion: DeletionStrategy::PermanentGenerated,
            evidence: vec![Evidence::exact("python-generated", title.clone())],
            user_policy: UserPolicy::Default,
        },
        selected: offer.preselect,
        title,
    }
}

fn generated_activity(path: &Path) -> ActivityEvidence {
    let modified = path_mtime(path);
    ActivityEvidence {
        latest_generated_modified: modified,
        latest_any_modified: modified,
        ..ActivityEvidence::default()
    }
}

fn safety_of(blocked: bool, blocker: Option<sweeploom_core::Blocker>) -> SafetyAssessment {
    if let Some(blocker) = blocker {
        SafetyAssessment::blocked(blocker)
    } else if blocked {
        SafetyAssessment::blocked(sweeploom_core::Blocker::UnknownGitState)
    } else {
        SafetyAssessment::safe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn cargo_light_is_preselected_node_is_not() {
        let root = std::env::temp_dir().join(format!("sweeploom-review-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("target").join("incremental")).unwrap();
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();
        fs::write(root.join("package.json"), "{}\n").unwrap();
        fs::write(
            root.join("target").join("incremental").join("a"),
            vec![0_u8; 512],
        )
        .unwrap();
        fs::write(root.join("node_modules").join("pkg.js"), vec![0_u8; 512]).unwrap();
        let rows = collect_review(&[root.as_path()], &[]);
        let light = rows
            .iter()
            .find(|row| row.title.contains("Light"))
            .expect("light cargo row");
        assert!(light.selected);
        assert!(!light.candidate.safety.is_blocked());
        let node = rows
            .iter()
            .find(|row| row.title.contains("node_modules"))
            .expect("node row");
        assert!(!node.selected);
        let _ = fs::remove_dir_all(&root);
    }
}
