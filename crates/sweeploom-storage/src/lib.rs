//! Disk inventory on top of `weavatrix-scan`.
//!
//! Artifact discovery disables repository ignore and standard skips so `target`
//! and `node_modules` remain visible. Source heat uses the opposite policy.
//! Weavatrix Scan itself is not modified; SweepLoom sets public options.

#![cfg_attr(not(test), warn(missing_docs))]

mod classify;
mod inventory;

pub use classify::{PathCategory, classify_path_component, is_project_marker, is_source_extension};
pub use inventory::{DirectoryNode, InventoryLimits, InventoryReport, scan_inventory};

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
