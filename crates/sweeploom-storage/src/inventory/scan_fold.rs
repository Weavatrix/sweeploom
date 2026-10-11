//! Bottom-up aggregation: folders fold into their parent once complete.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::node::{DirectoryNode, InventoryLimits};
use super::roots::is_discoverable_below;
use super::scan_meta::FileMeta;
use crate::classify::{
    PathCategory, classify_path_component, is_project_marker, is_source_extension,
    keep_nested_children,
};

struct Pending {
    node: DirectoryNode,
    /// Inside `.git`, generated, dependency or cache output. Computed once per folder.
    noise: bool,
}

impl Pending {
    fn new(node: DirectoryNode) -> Self {
        let noise = is_inside_noise(&node.path);
        Self { node, noise }
    }
}

#[derive(Default)]
pub(super) struct Fold {
    pending: HashMap<PathBuf, Pending>,
    pub projects: Vec<PathBuf>,
    project_set: HashSet<PathBuf>,
    pub project_bytes: BTreeMap<PathBuf, u64>,
    pub project_disk_bytes: BTreeMap<PathBuf, u64>,
    pub visited_logical: u64,
    pub errors: u64,
}

impl Fold {
    pub fn record_project(&mut self, marker: &Path, scan_root: &Path, max_projects: usize) {
        if self.projects.len() >= max_projects || !is_project_marker(marker) {
            return;
        }
        let Some(parent) = marker.parent() else {
            return;
        };
        // Registry checkouts, hidden tool homes and Library are not the user's projects.
        if is_inside_noise(parent) || !is_discoverable_below(scan_root, parent) {
            return;
        }
        if self.project_set.insert(parent.to_path_buf()) {
            self.projects.push(parent.to_path_buf());
        }
    }

    pub fn file(&mut self, path: &Path, meta: &FileMeta, limits: InventoryLimits) {
        self.visited_logical = self.visited_logical.saturating_add(meta.bytes);
        let parent = path.parent().unwrap_or(path);
        let slot = self.slot(parent);
        let node = &mut slot.node;
        node.logical_bytes = node.logical_bytes.saturating_add(meta.bytes);
        node.allocated_bytes = sum_allocated(node.allocated_bytes, meta.allocated);
        node.files = node.files.saturating_add(1);
        DirectoryNode::bump_mtime(&mut node.newest_mtime, meta.mtime);
        if is_source_extension(path) && !slot.noise {
            DirectoryNode::bump_mtime(&mut node.newest_source_mtime, meta.mtime);
        }
        if slot.noise {
            DirectoryNode::bump_mtime(&mut node.newest_generated_mtime, meta.mtime);
        }
        if meta.bytes >= limits.large_file_bytes {
            let leaf = DirectoryNode::file_leaf(
                path.to_path_buf(),
                meta.bytes,
                meta.allocated,
                meta.mtime,
            );
            insert_child(&mut node.children, leaf, limits.max_children_per_dir);
        }
    }

    /// A folder's contents are complete: keep it at `stop_at`, otherwise fold it into its parent.
    pub fn directory(
        &mut self,
        path: &Path,
        stop_at: &Path,
        scan_root: &Path,
        max_children: usize,
    ) {
        let node = self
            .pending
            .remove(path)
            .map_or_else(|| DirectoryNode::new(path.to_path_buf()), |slot| slot.node);
        match path.parent() {
            Some(parent) if path != stop_at => self.attach(parent, node, max_children, scan_root),
            _ => self.insert_node(node),
        }
    }

    pub fn insert_node(&mut self, node: DirectoryNode) {
        self.pending.insert(node.path.clone(), Pending::new(node));
    }

    pub fn peek(&self, path: &Path) -> Option<&DirectoryNode> {
        self.pending.get(path).map(|slot| &slot.node)
    }

    pub fn take(&mut self, path: &Path) -> DirectoryNode {
        self.pending
            .remove(path)
            .map_or_else(|| DirectoryNode::new(path.to_path_buf()), |slot| slot.node)
    }

    /// Projects and counters from one folder read off the lock.
    pub fn merge(&mut self, other: Self, max_projects: usize) {
        for project in other.projects {
            if self.projects.len() < max_projects && self.project_set.insert(project.clone()) {
                self.projects.push(project);
            }
        }
        self.project_bytes.extend(other.project_bytes);
        self.project_disk_bytes.extend(other.project_disk_bytes);
        self.visited_logical = self.visited_logical.saturating_add(other.visited_logical);
        self.errors = self.errors.saturating_add(other.errors);
    }

    pub fn is_project(&self, path: &Path) -> bool {
        self.project_set.contains(path)
    }

    fn attach(
        &mut self,
        parent: &Path,
        mut child: DirectoryNode,
        max_children: usize,
        scan_root: &Path,
    ) {
        if !keep_nested_children(&child.path) || too_deep(scan_root, &child.path) {
            child.children.clear();
        }
        if self.project_set.contains(&child.path) {
            self.project_bytes
                .insert(child.path.clone(), child.logical_bytes);
            self.project_disk_bytes
                .insert(child.path.clone(), child.disk_bytes());
        }
        let node = &mut self.slot(parent).node;
        accumulate(node, &child);
        node.directories = node.directories.saturating_add(1);
        node.incomplete |= child.incomplete;
        insert_child(&mut node.children, child, max_children);
    }

    fn slot(&mut self, dir: &Path) -> &mut Pending {
        if !self.pending.contains_key(dir) {
            self.insert_node(DirectoryNode::new(dir.to_path_buf()));
        }
        self.pending.get_mut(dir).expect("slot was just inserted")
    }
}

/// Add a child's totals and newest times into `node` (children are not linked).
pub(super) fn accumulate(node: &mut DirectoryNode, child: &DirectoryNode) {
    node.logical_bytes = node.logical_bytes.saturating_add(child.logical_bytes);
    node.allocated_bytes = sum_allocated(node.allocated_bytes, child.allocated_bytes);
    node.files = node.files.saturating_add(child.files);
    DirectoryNode::bump_mtime(&mut node.newest_mtime, child.newest_mtime);
    DirectoryNode::bump_mtime(&mut node.newest_source_mtime, child.newest_source_mtime);
    DirectoryNode::bump_mtime(
        &mut node.newest_generated_mtime,
        child.newest_generated_mtime,
    );
}

/// Keep children largest first (path breaks ties) without re-sorting per insert.
/// A total order keeps the kept set independent of the order workers finish.
pub(super) fn insert_child(children: &mut Vec<DirectoryNode>, child: DirectoryNode, max: usize) {
    let bytes = child.disk_bytes();
    let at = children.partition_point(|item| {
        let size = item.disk_bytes();
        size > bytes || (size == bytes && item.path <= child.path)
    });
    if at < max {
        children.insert(at, child);
        children.truncate(max);
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

fn too_deep(scan_root: &Path, path: &Path) -> bool {
    path.strip_prefix(scan_root)
        .map(|rel| rel.components().count() > 8)
        .unwrap_or(false)
}

fn sum_allocated(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    Some(left?.saturating_add(right?))
}
