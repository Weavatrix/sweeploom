//! Streaming walk and bottom-up aggregation.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use weavatrix_scan::{RootSymlinkPolicy, WalkBuilder, WalkEntry, WalkError, WalkOptions};

use crate::classify::{
    PathCategory, classify_path_component, is_project_marker, is_source_extension,
    keep_nested_children,
};

use super::node::{DirectoryNode, InventoryLimits, InventoryReport};

/// Live walk counters for the Explorer UI.
#[derive(Clone, Debug, Default)]
pub struct ScanTick {
    /// Entries visited so far.
    pub entries: u64,
    /// Logical bytes rolled up to the scan root, if known.
    pub logical_bytes: u64,
    /// Path currently being absorbed.
    pub hint: String,
    /// Snapshot of the root node so the table can stream folders.
    pub root: Option<DirectoryNode>,
}

/// Scan `root` into a folder-inspector tree.
pub fn scan_inventory(root: &Path, limits: InventoryLimits) -> Result<InventoryReport, WalkError> {
    scan_inventory_with(root, limits, |_| {})
}

/// Scan and report progress. `on_tick` may be called from a worker thread.
pub fn scan_inventory_with(
    root: &Path,
    limits: InventoryLimits,
    mut on_tick: impl FnMut(ScanTick),
) -> Result<InventoryReport, WalkError> {
    let scan_root = map_key(&std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf()));
    let walker = WalkBuilder::new(&scan_root)
        .options(
            WalkOptions::default()
                .with_metadata(true)
                .with_follow_links(false)
                .with_same_file_system(false)
                .with_root_symlink_policy(RootSymlinkPolicy::Reject),
        )
        .contents_first(true)
        .build();
    let mut pending: BTreeMap<PathBuf, DirectoryNode> = BTreeMap::new();
    let mut projects = Vec::new();
    let mut project_bytes = BTreeMap::new();
    let mut entries = 0_u64;
    let mut errors = 0_u64;
    let mut visited_logical_bytes = 0_u64;
    let mut capped = false;
    let mut last_tick = Instant::now();
    for item in walker {
        let Ok(entry) = item else {
            errors += 1;
            continue;
        };
        entries += 1;
        absorb_entry(
            &mut pending,
            &mut projects,
            &mut project_bytes,
            &entry,
            limits,
            &scan_root,
            &mut visited_logical_bytes,
        );
        if last_tick.elapsed() >= Duration::from_millis(150) {
            last_tick = Instant::now();
            on_tick(make_tick(&pending, &scan_root, entries, entry.path(), true));
        }
        if limits.max_entries.is_some_and(|max| entries >= max) {
            capped = true;
            break;
        }
    }
    let mut tree = if capped {
        fold_pending(&mut pending, &scan_root)
    } else {
        pending
            .remove(&scan_root)
            .unwrap_or_else(|| DirectoryNode::new(scan_root.clone()))
    };
    if capped {
        tree.incomplete = true;
    }
    tree.logical_bytes = tree.logical_bytes.max(visited_logical_bytes);
    if projects.iter().any(|item| item == &scan_root) {
        project_bytes.insert(scan_root.clone(), tree.logical_bytes);
    }
    Ok(InventoryReport {
        root: scan_root,
        tree,
        projects,
        project_bytes: project_bytes.into_iter().collect(),
        entries,
        errors,
        capped,
    })
}

fn make_tick(
    pending: &BTreeMap<PathBuf, DirectoryNode>,
    scan_root: &Path,
    entries: u64,
    hint: &Path,
    include_root: bool,
) -> ScanTick {
    let root = pending.get(scan_root);
    ScanTick {
        entries,
        logical_bytes: root.map(|node| node.logical_bytes).unwrap_or(0),
        hint: hint
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".")
            .to_owned(),
        root: if include_root {
            root.map(DirectoryNode::preview)
        } else {
            None
        },
    }
}

fn map_key(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(text.as_ref()))
}

fn too_deep(scan_root: &Path, path: &Path) -> bool {
    path.strip_prefix(scan_root)
        .map(|rel| rel.components().count() > 8)
        .unwrap_or(false)
}

fn absorb_entry(
    pending: &mut BTreeMap<PathBuf, DirectoryNode>,
    projects: &mut Vec<PathBuf>,
    project_bytes: &mut BTreeMap<PathBuf, u64>,
    entry: &WalkEntry,
    limits: InventoryLimits,
    scan_root: &Path,
    visited_logical_bytes: &mut u64,
) {
    let path = map_key(entry.path());
    let path = path.as_path();
    if entry.is_symlink() {
        return;
    }
    record_project(projects, path, limits.max_projects);
    let mtime = entry_mtime(entry);
    if entry.is_dir() {
        absorb_directory(
            pending,
            projects,
            project_bytes,
            path,
            mtime,
            limits.max_children_per_dir,
            scan_root,
        );
        return;
    }
    let bytes = entry.bytes().unwrap_or(0);
    *visited_logical_bytes = visited_logical_bytes.saturating_add(bytes);
    absorb_file(
        pending,
        path,
        bytes,
        mtime,
        limits.large_file_bytes,
        limits.max_children_per_dir,
    );
}

fn record_project(projects: &mut Vec<PathBuf>, path: &Path, max_projects: usize) {
    if projects.len() >= max_projects {
        return;
    }
    if !is_project_marker(path) {
        return;
    }
    let Some(parent) = path.parent() else {
        return;
    };
    if is_inside_noise(parent) {
        return;
    }
    if !projects.iter().any(|item| item == parent) {
        projects.push(parent.to_path_buf());
    }
}

fn is_inside_noise(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_str().unwrap_or("");
        name.eq_ignore_ascii_case(".git")
            || matches!(
                classify_path_component(name),
                PathCategory::Generated | PathCategory::Dependencies | PathCategory::Cache
            )
    })
}

fn entry_mtime(entry: &WalkEntry) -> Option<SystemTime> {
    entry.version().and_then(|version| {
        version.modified_ns.map(|ns| {
            SystemTime::UNIX_EPOCH
                + Duration::from_nanos(u64::try_from(ns.min(u128::from(u64::MAX))).unwrap_or(0))
        })
    })
}

fn absorb_directory(
    pending: &mut BTreeMap<PathBuf, DirectoryNode>,
    projects: &[PathBuf],
    project_bytes: &mut BTreeMap<PathBuf, u64>,
    path: &Path,
    mtime: Option<SystemTime>,
    max_children: usize,
    scan_root: &Path,
) {
    let mut node = pending
        .remove(path)
        .unwrap_or_else(|| DirectoryNode::new(path.to_path_buf()));
    DirectoryNode::bump_mtime(&mut node.newest_mtime, mtime);
    if path == scan_root {
        pending.insert(path.to_path_buf(), node);
    } else if let Some(parent) = path.parent() {
        attach_child(
            pending,
            projects,
            project_bytes,
            parent,
            node,
            max_children,
            scan_root,
        );
    } else {
        pending.insert(path.to_path_buf(), node);
    }
}

fn absorb_file(
    pending: &mut BTreeMap<PathBuf, DirectoryNode>,
    path: &Path,
    bytes: u64,
    mtime: Option<SystemTime>,
    large_file_bytes: u64,
    max_children: usize,
) {
    let parent = path.parent().unwrap_or(path);
    let node = pending
        .entry(parent.to_path_buf())
        .or_insert_with(|| DirectoryNode::new(parent.to_path_buf()));
    node.logical_bytes = node.logical_bytes.saturating_add(bytes);
    node.files = node.files.saturating_add(1);
    DirectoryNode::bump_mtime(&mut node.newest_mtime, mtime);
    if is_source_extension(path) && !is_inside_noise(parent) {
        DirectoryNode::bump_mtime(&mut node.newest_source_mtime, mtime);
    }
    let in_artifact = is_inside_noise(parent)
        || matches!(
            node.category,
            PathCategory::Generated | PathCategory::Dependencies | PathCategory::Cache
        );
    if in_artifact {
        DirectoryNode::bump_mtime(&mut node.newest_generated_mtime, mtime);
    }
    if bytes >= large_file_bytes {
        let leaf = DirectoryNode::file_leaf(path.to_path_buf(), bytes, mtime);
        node.children.push(leaf);
        node.children
            .sort_by_key(|right| std::cmp::Reverse(right.logical_bytes));
        if node.children.len() > max_children {
            node.children.truncate(max_children);
        }
    }
}

fn attach_child(
    pending: &mut BTreeMap<PathBuf, DirectoryNode>,
    projects: &[PathBuf],
    project_bytes: &mut BTreeMap<PathBuf, u64>,
    parent: &Path,
    mut child: DirectoryNode,
    max_children: usize,
    scan_root: &Path,
) {
    if !keep_nested_children(&child.path) || too_deep(scan_root, &child.path) {
        child.children.clear();
    }
    if projects.iter().any(|item| item == &child.path) {
        project_bytes.insert(child.path.clone(), child.logical_bytes);
    }
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
    parent_node.incomplete |= child.incomplete;
    parent_node.children.push(child);
    parent_node
        .children
        .sort_by_key(|right| std::cmp::Reverse(right.logical_bytes));
    if parent_node.children.len() > max_children {
        parent_node.children.truncate(max_children);
    }
}

fn fold_pending(pending: &mut BTreeMap<PathBuf, DirectoryNode>, scan_root: &Path) -> DirectoryNode {
    while let Some(path) = pending
        .keys()
        .filter(|path| *path != scan_root)
        .max_by_key(|path| path.components().count())
        .cloned()
    {
        let Some(mut node) = pending.remove(&path) else {
            continue;
        };
        node.incomplete = true;
        let Some(parent) = path.parent() else {
            continue;
        };
        let parent_node = pending
            .entry(parent.to_path_buf())
            .or_insert_with(|| DirectoryNode::new(parent.to_path_buf()));
        parent_node.logical_bytes = parent_node.logical_bytes.saturating_add(node.logical_bytes);
        parent_node.files = parent_node.files.saturating_add(node.files);
        parent_node.incomplete = true;
        DirectoryNode::bump_mtime(&mut parent_node.newest_mtime, node.newest_mtime);
        DirectoryNode::bump_mtime(
            &mut parent_node.newest_source_mtime,
            node.newest_source_mtime,
        );
        DirectoryNode::bump_mtime(
            &mut parent_node.newest_generated_mtime,
            node.newest_generated_mtime,
        );
    }
    let mut root = pending
        .remove(scan_root)
        .unwrap_or_else(|| DirectoryNode::new(scan_root.to_path_buf()));
    root.incomplete = true;
    root
}
