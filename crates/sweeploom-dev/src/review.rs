//! Turn analyzer offers into CleanPlan candidates.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use sweeploom_core::{
    ActivityEvidence, Candidate, CandidateId, CandidateKind, CandidateOwner, DeletionStrategy,
    Evidence, ProcessSnapshot, ProjectId, RebuildAssessment, RebuildCost, SafetyAssessment,
    UserPolicy,
};

use crate::cargo::{CargoOffer, CargoTrim, cargo_offers_with};
use crate::cargo_manifest::workspace_root;
use crate::git::GitMemo;
use crate::node::{NodeOffer, node_offers_with};
use crate::python::{PythonOffer, python_offers_with};
use crate::size::path_mtime;
use sweeploom_storage::{discover_projects, parallel_map, walk_workers};

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
    collect_review_with(projects, processes, |_| {})
}

/// [`collect_review`] on parallel workers. `on_project` receives each project's
/// rows as soon as they are sized (any thread, any order, ids not final); the
/// returned list keeps project order with ids `1..`.
///
/// Workspace members share the root `target`, so each owner is sized once. Git
/// state is read once per repository and only when something is offered.
#[must_use]
pub fn collect_review_with(
    projects: &[impl AsRef<Path>],
    processes: &[ProcessSnapshot],
    on_project: impl Fn(&[ReviewRow]) + Sync,
) -> Vec<ReviewRow> {
    let projects: Vec<PathBuf> = projects
        .iter()
        .map(|item| item.as_ref().to_path_buf())
        .collect();
    let workers = walk_workers();
    let owners = parallel_map(projects.clone(), workers, |project| {
        project
            .join("Cargo.toml")
            .is_file()
            .then(|| workspace_root(&project))
    });
    let mut cargo_owners = HashSet::<PathBuf>::new();
    let jobs: Vec<(PathBuf, Option<PathBuf>)> = projects
        .into_iter()
        .zip(owners)
        .map(|(project, owner)| {
            let owner = owner.filter(|owner| cargo_owners.insert(owner.clone()));
            (project, owner)
        })
        .collect();
    let memo = GitMemo::default();
    let git = |path: &Path| memo.inspect(path);
    let found = parallel_map(jobs, workers, |(project, owner)| {
        let mut rows = Vec::new();
        if let Some(owner) = owner {
            let offers = cargo_offers_with(&owner, processes, &git);
            rows.extend(offers.into_iter().map(|offer| cargo_row(offer, 0)));
        }
        let offers = node_offers_with(&project, processes, &git);
        rows.extend(offers.into_iter().map(|offer| node_row(offer, 0)));
        let offers = python_offers_with(&project, processes, &git);
        rows.extend(offers.into_iter().map(|offer| python_row(offer, 0)));
        if !rows.is_empty() {
            on_project(&rows);
        }
        rows
    });
    let mut rows: Vec<ReviewRow> = found.into_iter().flatten().collect();
    for (index, row) in rows.iter_mut().enumerate() {
        row.candidate.id = CandidateId(index as u64 + 1);
    }
    rows
}

fn cargo_row(offer: CargoOffer, id: u64) -> ReviewRow {
    let selected = !offer.blocked && matches!(offer.mode, CargoTrim::Light);
    let size = if offer.size_complete {
        String::new()
    } else {
        " · ≥".to_owned()
    };
    let title = format!("Cargo {:?}{size} · {}", offer.mode, offer.path.display());
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
    let vite = offer.label != "node_modules";
    let title = if vite {
        format!("Vite {} · {}", offer.label, offer.path.display())
    } else {
        format!("node_modules · {}", offer.path.display())
    };
    let title = if offer.size_complete {
        title
    } else {
        format!("{title} · ≥")
    };
    let safety = safety_of(offer.blocked, offer.blocker);
    let activity = generated_activity(&offer.path);
    ReviewRow {
        candidate: Candidate {
            id: CandidateId(id),
            kind: if vite {
                CandidateKind::BuildArtifact
            } else {
                CandidateKind::DependencyTree
            },
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
            evidence: vec![Evidence::exact(
                if vite { "vite-cache" } else { "node-modules" },
                title.clone(),
            )],
            user_policy: UserPolicy::Default,
        },
        selected: offer.preselect,
        title,
    }
}

fn python_row(offer: PythonOffer, id: u64) -> ReviewRow {
    let title = format!("Python {} · {}", offer.label, offer.path.display());
    let title = if offer.size_complete {
        title
    } else {
        format!("{title} · ≥")
    };
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
#[path = "review_tests.rs"]
mod tests;
