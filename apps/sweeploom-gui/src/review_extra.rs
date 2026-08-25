//! Review rows: generated project output first, then temp / Downloads / AI.

use std::path::{Path, PathBuf};

use sweeploom_ai::inspect_offers;
use sweeploom_core::ProcessSnapshot;
use sweeploom_dev::{ReviewRow, collect_review};
use sweeploom_general::collect_offers;
use sweeploom_platform::UserLocations;
use sweeploom_storage::{discover_projects_from, review_scan_roots};

const MAX_PROJECTS: usize = 120;

/// Result of Review discovery.
pub struct ReviewBuild {
    /// Project roots that fed the candidate list.
    pub projects: Vec<PathBuf>,
    /// Rows shown on Storage review.
    pub rows: Vec<ReviewRow>,
}

/// Build the Review list for `scan_root`.
#[must_use]
pub fn all_rows(
    scan_root: &Path,
    locations: &UserLocations,
    inventory_projects: &[PathBuf],
    processes: &[ProcessSnapshot],
) -> Vec<ReviewRow> {
    assemble(scan_root, locations, inventory_projects, None, processes).rows
}

/// Discover projects (preferring the current workspace) and collect review rows.
#[must_use]
pub fn assemble(
    scan_root: &Path,
    locations: &UserLocations,
    inventory_projects: &[PathBuf],
    current_project: Option<&Path>,
    processes: &[ProcessSnapshot],
) -> ReviewBuild {
    let mut roots = review_scan_roots(scan_root, &locations.home);
    prepend_unique(&mut roots, current_project);
    if let Ok(cwd) = std::env::current_dir() {
        prepend_unique(&mut roots, Some(cwd.as_path()));
    }
    for project in inventory_projects {
        push_unique(&mut roots, project);
    }
    let mut projects = discover_projects_from(&roots, MAX_PROJECTS);
    prepend_project(&mut projects, current_project);
    if let Ok(cwd) = std::env::current_dir() {
        prepend_project(&mut projects, Some(cwd.as_path()));
    }
    let mut rows = collect_review(&projects, processes);
    rows.extend(extra_rows(locations));
    ReviewBuild { projects, rows }
}

fn looks_like_project(path: &Path) -> bool {
    ["Cargo.toml", "package.json", "pyproject.toml", "Pipfile"]
        .iter()
        .any(|marker| path.join(marker).is_file())
}

fn prepend_unique(roots: &mut Vec<PathBuf>, path: Option<&Path>) {
    let Some(path) = path else {
        return;
    };
    if !path.is_dir() {
        return;
    }
    roots.retain(|item| item != path);
    roots.insert(0, path.to_path_buf());
}

fn push_unique(roots: &mut Vec<PathBuf>, path: &Path) {
    if !roots.iter().any(|item| item == path) {
        roots.push(path.to_path_buf());
    }
}

fn prepend_project(projects: &mut Vec<PathBuf>, path: Option<&Path>) {
    let Some(path) = path else {
        return;
    };
    if !looks_like_project(path) {
        return;
    }
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    projects.retain(|item| item != path && item != &canonical);
    projects.insert(0, canonical);
}

/// General + AI inspect rows.
#[must_use]
pub fn extra_rows(locations: &UserLocations) -> Vec<ReviewRow> {
    let mut rows = Vec::new();
    for offer in collect_offers(locations) {
        rows.push(ReviewRow {
            candidate: offer.candidate,
            selected: offer.selected,
            title: offer.title,
        });
    }
    for offer in inspect_offers(locations) {
        rows.push(ReviewRow {
            candidate: offer.candidate,
            selected: offer.selected,
            title: offer.title,
        });
    }
    rows
}
