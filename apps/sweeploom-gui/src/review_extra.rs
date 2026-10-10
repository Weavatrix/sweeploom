//! Review rows: generated project output first, then temp / Downloads / AI.

use std::path::{Path, PathBuf};

use sweeploom_ai::inspect_offers;
use sweeploom_dev::ReviewRow;
use sweeploom_general::collect_offers;
use sweeploom_platform::UserLocations;
use sweeploom_storage::{
    discover_projects_from, is_discoverable_below, is_project_marker_name, review_scan_roots,
};

/// Discovery gives every top-level tree its own budget; this caps the total.
const MAX_PROJECTS: usize = 2048;

/// Discover project roots before the slower cleanup/store measurements start.
///
/// Discovery always covers the developer folders under home plus the launch
/// folder and current project. It never depends on the Explorer root: projects
/// from the last Explorer scan (rooted at `inventory_root`) are merged in.
pub fn project_roots(
    inventory_root: Option<&Path>,
    locations: &UserLocations,
    inventory_projects: &[PathBuf],
    current_project: Option<&Path>,
) -> Vec<PathBuf> {
    let cwd = std::env::current_dir().ok();
    let roots = discovery_roots(&locations.home, current_project, cwd.as_deref());
    let mut projects = discover_projects_from(&roots, MAX_PROJECTS);
    if let Some(root) = inventory_root {
        merge_inventory(&mut projects, root, inventory_projects);
    }
    prepend_project(&mut projects, current_project);
    prepend_project(&mut projects, cwd.as_deref());
    projects.truncate(MAX_PROJECTS);
    projects
}

fn discovery_roots(home: &Path, current: Option<&Path>, cwd: Option<&Path>) -> Vec<PathBuf> {
    let mut roots = review_scan_roots(home, home);
    for path in [current, cwd].into_iter().flatten() {
        prepend_unique(&mut roots, path, home);
    }
    roots
}

/// Explorer projects join discovery; registry checkouts and hidden tool homes do not.
fn merge_inventory(projects: &mut Vec<PathBuf>, root: &Path, inventory: &[PathBuf]) {
    let mut known: std::collections::HashSet<PathBuf> = projects.iter().cloned().collect();
    for project in inventory {
        if is_discoverable_below(root, project) && known.insert(project.clone()) {
            projects.push(project.clone());
        }
    }
}

fn looks_like_project(path: &Path) -> bool {
    std::fs::read_dir(path).is_ok_and(|entries| {
        entries.flatten().any(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(is_project_marker_name)
        })
    })
}

fn prepend_unique(roots: &mut Vec<PathBuf>, path: &Path, home: &Path) {
    // Finder launches apps in `/`, and home itself is covered by its developer
    // folders; walking either would spend the budget on Library and media.
    if path.parent().is_none() || path == home || !path.is_dir() {
        return;
    }
    roots.retain(|item| item != path);
    roots.insert(0, path.to_path_buf());
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

#[cfg(test)]
#[path = "review_extra_tests.rs"]
mod tests;
