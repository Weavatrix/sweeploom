//! Disk inventory on top of `weavatrix-scan`.
//!
//! Artifact discovery disables repository ignore and standard skips so `target`
//! and `node_modules` remain visible. Source heat uses the opposite policy.
//! Weavatrix Scan itself is not modified; SweepLoom sets public options.

#![cfg_attr(not(test), warn(missing_docs))]

mod classify;
mod inventory;

pub use classify::{PathCategory, classify_path_component, is_project_marker, is_source_extension};
pub use inventory::{
    DirectoryNode, InventoryLimits, InventoryReport, developer_roots, discover_projects,
    discover_projects_from, review_scan_roots, scan_inventory,
};

use weavatrix_scan::{IgnorePolicy, ScanOptions, StandardSkips};

/// Scan options for generated/artifact discovery.
///
/// Repository ignore and standard skips are off so `target` and `node_modules`
/// remain visible. Implemented with Weavatrix Scan helpers, not a local fork.
#[must_use]
pub fn artifact_scan_options() -> ScanOptions {
    ScanOptions::default()
        .metadata_only()
        .with_ignore_policy(IgnorePolicy::none())
        .with_standard_skips(StandardSkips::Disabled)
}

/// Scan options for Source Heat. Generated trees stay ignored.
#[must_use]
pub fn source_heat_scan_options() -> ScanOptions {
    ScanOptions::default()
        .metadata_only()
        .with_ignore_policy(IgnorePolicy::repository())
        .with_standard_skips(StandardSkips::Enabled)
}

/// Project marker files used for discovery.
pub const PROJECT_MARKERS: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "Package.swift",
    "CMakeLists.txt",
];

#[cfg(test)]
mod tests {
    use super::*;
    use weavatrix_scan::StandardSkips;

    #[test]
    fn artifact_scan_uses_weavatrix_none_policy() {
        let options = artifact_scan_options();
        assert!(!options.ignore_policy.git_ignore);
        assert_eq!(options.standard_skips, StandardSkips::Disabled);
        let source = source_heat_scan_options();
        assert!(source.ignore_policy.git_ignore);
        assert_eq!(source.standard_skips, StandardSkips::Enabled);
    }
}
