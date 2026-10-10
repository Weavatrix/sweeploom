//! Disk inventory on top of `weavatrix-scan`.
//!
//! Artifact discovery disables repository ignore and standard skips so `target`
//! and `node_modules` remain visible. Source heat uses the opposite policy.
//! Weavatrix Scan itself is not modified; SweepLoom sets public options.

#![cfg_attr(not(test), warn(missing_docs))]

mod classify;
mod disk_usage;
mod inventory;
mod parallel;

pub use disk_usage::{DiskUsage, directory_disk_usage};
pub use parallel::{parallel_map, walk_workers};

pub use classify::{
    PathCategory, classify_path_component, is_project_marker, is_project_marker_name,
    is_source_extension, keep_nested_children,
};
pub use inventory::{
    DirectoryNode, InventoryLimits, InventoryReport, ScanTick, developer_roots, discover_projects,
    discover_projects_from, is_discoverable_below, review_scan_roots, scan_inventory,
    scan_inventory_with,
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

/// Project marker names used for discovery. `.git` covers repositories
/// without a recognised build file.
pub const PROJECT_MARKERS: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "setup.py",
    "Pipfile",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "settings.gradle",
    "settings.gradle.kts",
    "Package.swift",
    "Podfile",
    "CMakeLists.txt",
    "Gemfile",
    "composer.json",
    "pubspec.yaml",
    "mix.exs",
    "deno.json",
    "build.zig",
    ".git",
];

/// Marker extensions: Xcode projects/workspaces and .NET solutions/projects.
pub const PROJECT_MARKER_EXTENSIONS: &[&str] = &[
    "xcodeproj",
    "xcworkspace",
    "playground",
    "sln",
    "csproj",
    "fsproj",
    "vbproj",
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
