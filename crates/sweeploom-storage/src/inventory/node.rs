//! Aggregated directory nodes for Folder Inspector.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use sweeploom_core::{ActivityEvidence, ActivityState};

use crate::classify::{PathCategory, classify_path_component};

/// Limits that keep RAM bounded on huge trees.
#[derive(Clone, Copy, Debug)]
pub struct InventoryLimits {
    /// Stop after this many entries. `None` means no cap.
    pub max_entries: Option<u64>,
    /// Keep at most this many child rows per directory in the inspector tree.
    pub max_children_per_dir: usize,
    /// Stop recording project markers after this many roots.
    pub max_projects: usize,
}

impl Default for InventoryLimits {
    fn default() -> Self {
        Self {
            max_entries: Some(500_000),
            max_children_per_dir: 64,
            max_projects: 256,
        }
    }
}

impl InventoryLimits {
    /// Tighter caps for the interactive GUI so a home scan cannot freeze the UI.
    #[must_use]
    pub const fn gui() -> Self {
        Self {
            max_entries: Some(80_000),
            max_children_per_dir: 48,
            max_projects: 128,
        }
    }
}

/// One aggregated directory (or large file) in the inspector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryNode {
    /// Path.
    pub path: PathBuf,
    /// Logical bytes (sum of `metadata.len()`).
    pub logical_bytes: u64,
    /// File count under this node (files only).
    pub files: u64,
    /// Direct child directories counted.
    pub directories: u64,
    /// Newest mtime under this node.
    pub newest_mtime: Option<SystemTime>,
    /// Newest source mtime under this node.
    pub newest_source_mtime: Option<SystemTime>,
    /// Newest generated mtime under this node.
    pub newest_generated_mtime: Option<SystemTime>,
    /// Category derived from the node name.
    pub category: PathCategory,
    /// Direct children, largest first, capped.
    pub children: Vec<DirectoryNode>,
}

impl DirectoryNode {
    pub(crate) fn new(path: PathBuf) -> Self {
        let category = path
            .file_name()
            .and_then(|name| name.to_str())
            .map_or(PathCategory::Unknown, classify_path_component);
        Self {
            path,
            logical_bytes: 0,
            files: 0,
            directories: 0,
            newest_mtime: None,
            newest_source_mtime: None,
            newest_generated_mtime: None,
            category,
            children: Vec::new(),
        }
    }

    pub(crate) fn bump_mtime(slot: &mut Option<SystemTime>, candidate: Option<SystemTime>) {
        match (*slot, candidate) {
            (None, Some(value)) => *slot = Some(value),
            (Some(current), Some(value)) if value > current => *slot = Some(value),
            _ => {}
        }
    }
}

/// Inventory result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryReport {
    /// Scan root.
    pub root: PathBuf,
    /// Tree rooted at `root`.
    pub tree: DirectoryNode,
    /// Discovered project roots (directories containing a marker).
    pub projects: Vec<PathBuf>,
    /// Entries visited.
    pub entries: u64,
    /// Walk errors (typed, no panic).
    pub errors: u64,
    /// True when the entry cap stopped the walk.
    pub capped: bool,
}

impl InventoryReport {
    /// Folder at `path`, if the scan visited it.
    #[must_use]
    pub fn node(&self, path: &Path) -> Option<&DirectoryNode> {
        find_node(&self.tree, path).or_else(|| {
            let canonical = std::fs::canonicalize(path).ok()?;
            find_node(&self.tree, &canonical)
        })
    }

    /// Source / artifact heat for a discovered project directory.
    #[must_use]
    pub fn project_heat(&self, project: &Path, now: SystemTime) -> (ActivityState, ActivityState) {
        let Some(node) = self.node(project) else {
            return (ActivityState::Unknown, ActivityState::Unknown);
        };
        let evidence = ActivityEvidence {
            latest_source_modified: node.newest_source_mtime,
            latest_generated_modified: node.newest_generated_mtime,
            latest_any_modified: node.newest_mtime,
            ..ActivityEvidence::default()
        };
        (evidence.source_heat(now), evidence.artifact_heat(now))
    }
}

fn find_node<'a>(node: &'a DirectoryNode, path: &Path) -> Option<&'a DirectoryNode> {
    if node.path == path {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, path))
}
