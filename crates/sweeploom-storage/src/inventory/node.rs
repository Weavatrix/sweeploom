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
    /// Promote files at least this large into their own inspector rows.
    pub large_file_bytes: u64,
}

impl Default for InventoryLimits {
    fn default() -> Self {
        Self {
            max_entries: Some(500_000),
            max_children_per_dir: 64,
            max_projects: 256,
            large_file_bytes: 32 * 1024 * 1024,
        }
    }
}

impl InventoryLimits {
    /// GUI walk. No entry cap — the scan is off-thread and streams progress.
    #[must_use]
    pub const fn gui() -> Self {
        Self {
            max_entries: None,
            max_children_per_dir: 64,
            max_projects: 512,
            large_file_bytes: 32 * 1024 * 1024,
        }
    }
}

/// One aggregated directory (or large file) in the inspector.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DirectoryNode {
    /// Path.
    pub path: PathBuf,
    /// Logical bytes (sum of `metadata.len()`).
    pub logical_bytes: u64,
    /// Sum of allocated file blocks when available (sparse files may be much smaller).
    pub allocated_bytes: Option<u64>,
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
    /// True when this node is a single large file, not a folder.
    pub is_file: bool,
    /// True when the walk did not finish this subtree.
    pub incomplete: bool,
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
            allocated_bytes: if cfg!(unix) { Some(0) } else { None },
            files: 0,
            directories: 0,
            newest_mtime: None,
            newest_source_mtime: None,
            newest_generated_mtime: None,
            category,
            children: Vec::new(),
            is_file: false,
            incomplete: false,
        }
    }

    pub(crate) fn file_leaf(
        path: PathBuf,
        bytes: u64,
        allocated: Option<u64>,
        mtime: Option<SystemTime>,
    ) -> Self {
        let mut node = Self::new(path);
        node.logical_bytes = bytes;
        node.allocated_bytes = allocated;
        node.files = 1;
        node.is_file = true;
        node.newest_mtime = mtime;
        node
    }

    /// Bytes occupying disk blocks, falling back to logical size if unavailable.
    #[must_use]
    pub fn disk_bytes(&self) -> u64 {
        self.allocated_bytes.unwrap_or(self.logical_bytes)
    }

    /// Immediate children only. Nested trees stay on the walker until the scan finishes.
    #[must_use]
    pub fn preview(&self) -> Self {
        Self {
            path: self.path.clone(),
            logical_bytes: self.logical_bytes,
            allocated_bytes: self.allocated_bytes,
            files: self.files,
            directories: self.directories,
            newest_mtime: self.newest_mtime,
            newest_source_mtime: self.newest_source_mtime,
            newest_generated_mtime: self.newest_generated_mtime,
            category: self.category,
            children: self.children.iter().map(Self::leaf).collect(),
            is_file: self.is_file,
            incomplete: self.incomplete,
        }
    }

    fn leaf(child: &Self) -> Self {
        Self {
            path: child.path.clone(),
            logical_bytes: child.logical_bytes,
            allocated_bytes: child.allocated_bytes,
            files: child.files,
            directories: child.directories,
            newest_mtime: child.newest_mtime,
            newest_source_mtime: child.newest_source_mtime,
            newest_generated_mtime: child.newest_generated_mtime,
            category: child.category,
            children: Vec::new(),
            is_file: child.is_file,
            incomplete: child.incomplete,
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InventoryReport {
    /// Scan root.
    pub root: PathBuf,
    /// Tree rooted at `root`.
    pub tree: DirectoryNode,
    /// Discovered project roots (directories containing a marker).
    pub projects: Vec<PathBuf>,
    /// Logical bytes for each discovered project root, even if the inspector tree dropped it.
    pub project_bytes: Vec<(PathBuf, u64)>,
    /// Allocated byte rollups for discovered projects.
    pub project_disk_bytes: Vec<(PathBuf, u64)>,
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

    /// Folder size from the scan: project rollup first, then the inspector node.
    #[must_use]
    pub fn folder_bytes(&self, path: &Path) -> Option<u64> {
        self.project_bytes
            .iter()
            .find(|(item, _)| item == path)
            .map(|(_, bytes)| *bytes)
            .or_else(|| {
                let canonical = std::fs::canonicalize(path).ok()?;
                self.project_bytes
                    .iter()
                    .find(|(item, _)| item == &canonical)
                    .map(|(_, bytes)| *bytes)
            })
            .or_else(|| self.node(path).map(|node| node.logical_bytes))
    }

    /// Allocated folder size for disk-oriented UI, falling back to logical bytes.
    #[must_use]
    pub fn folder_disk_bytes(&self, path: &Path) -> Option<u64> {
        let canonical = std::fs::canonicalize(path).ok();
        self.project_disk_bytes
            .iter()
            .find(|(item, _)| item == path || canonical.as_ref() == Some(item))
            .map(|(_, bytes)| *bytes)
            .or_else(|| self.node(path).map(DirectoryNode::disk_bytes))
            .or_else(|| self.folder_bytes(path))
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

/// Descend only along `path`'s ancestors instead of searching the whole tree.
fn find_node<'a>(node: &'a DirectoryNode, path: &Path) -> Option<&'a DirectoryNode> {
    if node.path == path {
        return Some(node);
    }
    if !path.starts_with(&node.path) {
        return None;
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, path))
}
