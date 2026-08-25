//! Streaming walk and bottom-up aggregation.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use weavatrix_scan::{RootSymlinkPolicy, WalkBuilder, WalkEntry, WalkError, WalkOptions};

use crate::classify::{
    PathCategory, classify_path_component, is_project_marker, is_source_extension,
};

use super::node::{DirectoryNode, InventoryLimits, InventoryReport};

/// Scan `root` into a folder-inspector tree.
pub fn scan_inventory(root: &Path, limits: InventoryLimits) -> Result<InventoryReport, WalkError> {
    let scan_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let walker = WalkBuilder::new(&scan_root)
        .options(
            WalkOptions::default()
                .with_metadata(true)
                .with_follow_links(false)
                .with_same_file_system(true)
                .with_root_symlink_policy(RootSymlinkPolicy::Reject),
        )
        .contents_first(true)
        .build();
    let mut pending: BTreeMap<PathBuf, DirectoryNode> = BTreeMap::new();
    let mut projects = Vec::new();
    let mut entries = 0_u64;
    let mut errors = 0_u64;
    let mut capped = false;
    for item in walker {
        let Ok(entry) = item else {
            errors += 1;
            continue;
        };
        entries += 1;
        if limits.max_entries.is_some_and(|max| entries > max) {
            capped = true;
            break;
        }
        absorb_entry(&mut pending, &mut projects, &entry, limits, &scan_root);
    }
    let tree = pending
        .remove(&scan_root)
        .unwrap_or_else(|| DirectoryNode::new(scan_root.clone()));
    Ok(InventoryReport {
        root: scan_root,
        tree,
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
    limits: InventoryLimits,
    scan_root: &Path,
) {
    let path = entry.path();
    if entry.is_symlink() {
        return;
    }
    record_project(projects, path, limits.max_projects);
    let mtime = entry_mtime(entry);
    if entry.is_dir() {
        absorb_directory(pending, path, mtime, limits.max_children_per_dir, scan_root);
        return;
    }
    absorb_file(pending, path, entry.bytes().unwrap_or(0), mtime);
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
        attach_child(pending, parent, node, max_children);
    } else {
        pending.insert(path.to_path_buf(), node);
    }
}

fn absorb_file(
    pending: &mut BTreeMap<PathBuf, DirectoryNode>,
    path: &Path,
    bytes: u64,
    mtime: Option<SystemTime>,
) {
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
    let category = path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or(PathCategory::Unknown, classify_path_component);
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
        .sort_by_key(|right| std::cmp::Reverse(right.logical_bytes));
    if parent_node.children.len() > max_children {
        parent_node.children.truncate(max_children);
    }
}
