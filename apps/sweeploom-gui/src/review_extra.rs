//! Review rows: generated project output first, then temp / Downloads / AI.

use std::path::{Path, PathBuf};

use sweeploom_ai::inspect_offers;
use sweeploom_core::ProcessSnapshot;
use sweeploom_dev::{ReviewRow, collect_review};
use sweeploom_general::collect_offers;
use sweeploom_platform::UserLocations;
use sweeploom_storage::{discover_projects_from, review_scan_roots};

/// Build the Review list for `scan_root`.
#[must_use]
pub fn all_rows(
    scan_root: &Path,
    locations: &UserLocations,
    inventory_projects: &[PathBuf],
    processes: &[ProcessSnapshot],
) -> Vec<ReviewRow> {
    let mut roots = review_scan_roots(scan_root, &locations.home);
    for project in inventory_projects {
        if !roots.iter().any(|item| item == project) {
            roots.push(project.clone());
        }
    }
    let projects = discover_projects_from(&roots, 48);
    let mut rows = collect_review(&projects, processes);
    rows.extend(extra_rows(locations));
    rows
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
