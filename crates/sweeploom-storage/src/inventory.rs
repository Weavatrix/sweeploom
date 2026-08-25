//! Streaming inventory using `weavatrix-scan`'s walker.
//!
//! Follows no symlinks, stays on the same filesystem, and aggregates after
//! descendants so Folder Inspector can show largest children without retaining
//! every file path.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use weavatrix_scan::{RootSymlinkPolicy, WalkBuilder, WalkEntry, WalkError, WalkOptions};

use sweeploom_core::{ActivityEvidence, ActivityState};

use crate::classify::{
    PathCategory, classify_path_component, is_project_marker, is_source_extension,
};

/// Limits that keep RAM bounded on huge trees.
#[derive(Clone, Copy, Debug)]
pub struct InventoryLimits {
    /// Stop after this many entries. `None` means no cap.
    pub max_entries: Option<u64>,
    /// Keep at most this many child rows per directory in the inspector tree.
    pub max_children_per_dir: usize,
}

impl Default for InventoryLimits {
    fn default() -> Self {
        Self {
            max_entries: Some(2_000_000),
            max_children_per_dir: 64,
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
    fn new(path: PathBuf) -> Self {
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

    fn bump_mtime(slot: &mut Option<SystemTime>, candidate: Option<SystemTime>) {
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
    /// Source / artifact heat for a discovered project directory.
    #[must_use]
    pub fn project_heat(&self, project: &Path, now: SystemTime) -> (ActivityState, ActivityState) {
        let Some(node) = find_node(&self.tree, project) else {
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

/// Scan `root` into a folder-inspector tree.
pub fn scan_inventory(root: &Path, limits: InventoryLimits) -> Result<InventoryReport, WalkError> {
    let scan_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let options = WalkOptions::default()
        .with_metadata(true)
        .with_follow_links(false)
        .with_same_file_system(true)
        .with_root_symlink_policy(RootSymlinkPolicy::Reject);
    let walker = WalkBuilder::new(&scan_root)
        .options(options)
        .contents_first(true)
        .build();

    let mut pending: BTreeMap<PathBuf, DirectoryNode> = BTreeMap::new();
    let mut projects = Vec::new();
    let mut entries = 0_u64;
    let mut errors = 0_u64;
    let mut capped = false;

    for item in walker {
        let entry = match item {
            Ok(entry) => entry,
            Err(_) => {
                errors += 1;
                continue;
            }
        };
        entries += 1;
        if limits.max_entries.is_some_and(|max| entries > max) {
            capped = true;
            break;
        }
        absorb_entry(
            &mut pending,
            &mut projects,
            &entry,
            limits.max_children_per_dir,
            &scan_root,
        );
    }

    let root_node = pending
        .remove(&scan_root)
        .unwrap_or_else(|| DirectoryNode::new(scan_root.clone()));
    Ok(InventoryReport {
        root: scan_root,
        tree: root_node,
        projects,
        entries,
        errors,
        capped,
    })
}

fn absorb_entry(
    pending: &mut BTreeMap<PathBuf, DirectoryNode>,
    projects: &mut Vec<PathBuf>,
    entry: &WalkEntry,
    max_children: usize,
    scan_root: &Path,
) {
    let path = entry.path();
    if entry.is_symlink() {
        return;
    }
    if is_project_marker(path)
        && let Some(parent) = path.parent()
        && !projects.iter().any(|item| item == parent)
    {
        projects.push(parent.to_path_buf());
    }

    let mtime = entry.version().and_then(|version| {
        version.modified_ns.map(|ns| {
            SystemTime::UNIX_EPOCH
                + Duration::from_nanos(u64::try_from(ns.min(u128::from(u64::MAX))).unwrap_or(0))
        })
    });
    let bytes = entry.bytes().unwrap_or(0);
    let category = path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or(PathCategory::Unknown, classify_path_component);

    if entry.is_dir() {
        let mut node = pending
            .remove(path)
            .unwrap_or_else(|| DirectoryNode::new(path.to_path_buf()));
        DirectoryNode::bump_mtime(&mut node.newest_mtime, mtime);
        if path == scan_root {
            pending.insert(path.to_path_buf(), node);
        } else if let Some(parent) = path.parent() {
            attach_child(pending, parent, node, max_children);
        } else {
            pending.insert(path.to_path_buf(), node);
        }
        return;
    }

    let parent = path.parent().unwrap_or(path);
    let node = pending
        .entry(parent.to_path_buf())
        .or_insert_with(|| DirectoryNode::new(parent.to_path_buf()));
    node.logical_bytes = node.logical_bytes.saturating_add(bytes);
    node.files = node.files.saturating_add(1);
    DirectoryNode::bump_mtime(&mut node.newest_mtime, mtime);
    if is_source_extension(path) {
        DirectoryNode::bump_mtime(&mut node.newest_source_mtime, mtime);
    }
    if matches!(
        category,
        PathCategory::Generated | PathCategory::Dependencies | PathCategory::Cache
    ) {
        DirectoryNode::bump_mtime(&mut node.newest_generated_mtime, mtime);
    }
}

fn attach_child(
    pending: &mut BTreeMap<PathBuf, DirectoryNode>,
    parent: &Path,
    child: DirectoryNode,
    max_children: usize,
) {
    let parent_node = pending
        .entry(parent.to_path_buf())
        .or_insert_with(|| DirectoryNode::new(parent.to_path_buf()));
    parent_node.logical_bytes = parent_node
        .logical_bytes
        .saturating_add(child.logical_bytes);
    parent_node.files = parent_node.files.saturating_add(child.files);
    parent_node.directories = parent_node.directories.saturating_add(1);
    DirectoryNode::bump_mtime(&mut parent_node.newest_mtime, child.newest_mtime);
    DirectoryNode::bump_mtime(
        &mut parent_node.newest_source_mtime,
        child.newest_source_mtime,
    );
    DirectoryNode::bump_mtime(
        &mut parent_node.newest_generated_mtime,
        child.newest_generated_mtime,
    );
    parent_node.children.push(child);
    parent_node
        .children
        .sort_by(|left, right| right.logical_bytes.cmp(&left.logical_bytes));
    if parent_node.children.len() > max_children {
        parent_node.children.truncate(max_children);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn paths_match(left: &Path, right: &Path) -> bool {
        if left == right {
            return true;
        }
        match (fs::canonicalize(left), fs::canonicalize(right)) {
            (Ok(a), Ok(b)) => a == b,
            _ => left.file_name() == right.file_name(),
        }
    }

    #[test]
    fn inventory_finds_cargo_project_and_target_bytes() {
        let root = std::env::temp_dir().join(format!("sweeploom-inventory-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();
        fs::write(root.join("src").join("lib.rs"), "pub fn x() {}\n").unwrap();
        fs::write(root.join("target").join("big.bin"), vec![0_u8; 4096]).unwrap();

        let report = scan_inventory(&root, InventoryLimits::default()).expect("scan");
        assert!(
            report.projects.iter().any(|item| paths_match(item, &root)),
            "expected project root among {:?}",
            report.projects
        );
        assert!(
            report.tree.logical_bytes >= 4096,
            "logical={}",
            report.tree.logical_bytes
        );
        let target = report.tree.children.iter().find(|child| {
            child
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("target"))
        });
        assert!(
            target.is_some(),
            "target directory should be a child of {:?}",
            report
                .tree
                .children
                .iter()
                .map(|item| item.path.clone())
                .collect::<Vec<_>>()
        );
        let _ = fs::remove_dir_all(&root);
    }
}
