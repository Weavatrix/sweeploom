//! Find project markers without walking generated trees.
//!
//! Every top-level folder of a root is walked on its own worker with its own
//! folder and project budget, so one huge tree cannot starve its siblings.
//! One `read_dir` per folder; names and `d_type` decide, no per-entry `stat`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::roots::{is_discoverable_below, is_skip_name};
use crate::classify::is_project_marker_name;
use crate::parallel::{parallel_map, walk_workers};

const MAX_DEPTH: usize = 10;
const MAX_DIRS_PER_TREE: usize = 10_000;
const MAX_PROJECTS_PER_TREE: usize = 48;
const MAX_DIRS_TOTAL: usize = 250_000;

enum Task {
    Project(PathBuf),
    Tree(PathBuf),
}

/// Discover project roots under `root`, skipping `target` / `node_modules` / AppData.
#[must_use]
pub fn discover_projects(root: &Path, max_projects: usize) -> Vec<PathBuf> {
    discover_projects_from(&[root.to_path_buf()], max_projects)
}

/// Discover across several roots in parallel. Roots covered by another root
/// and duplicate projects are dropped; order follows `roots`, then names.
#[must_use]
pub fn discover_projects_from(roots: &[PathBuf], max_projects: usize) -> Vec<PathBuf> {
    let mut tasks = Vec::new();
    for root in distinct_roots(roots) {
        let Some((marker, children)) = scan_dir(&root) else {
            continue;
        };
        if marker {
            tasks.push(Task::Project(root));
        }
        tasks.extend(children.into_iter().map(Task::Tree));
    }
    let budget = AtomicUsize::new(MAX_DIRS_TOTAL);
    let found = parallel_map(tasks, walk_workers(), |task| match task {
        Task::Project(path) => vec![path],
        Task::Tree(path) => walk_tree(path, &budget),
    });
    let mut seen = HashSet::new();
    let found: Vec<Vec<PathBuf>> = found
        .into_iter()
        .map(|tree| tree.into_iter().filter(|path| seen.insert(path.clone())).collect())
        .collect();
    fair_take(found, max_projects)
}

/// Keep at most `max` projects, taking them round-robin across trees so a
/// monorepo early in the list cannot crowd out the repositories after it.
/// Within a tree, outer projects come first and survive.
fn fair_take(found: Vec<Vec<PathBuf>>, max: usize) -> Vec<PathBuf> {
    let mut quota = vec![0_usize; found.len()];
    let mut taken = 0;
    for round in 0.. {
        let before = taken;
        for (tree, slot) in found.iter().zip(quota.iter_mut()) {
            if taken < max && tree.len() > round {
                *slot += 1;
                taken += 1;
            }
        }
        if taken == before {
            break;
        }
    }
    found
        .into_iter()
        .zip(quota)
        .flat_map(|(tree, quota)| tree.into_iter().take(quota))
        .collect()
}

fn distinct_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut unique: Vec<PathBuf> = Vec::new();
    for root in roots {
        let root = fs::canonicalize(root).unwrap_or_else(|_| root.clone());
        if root.is_dir() && !unique.contains(&root) {
            unique.push(root);
        }
    }
    unique
        .iter()
        .filter(|root| !unique.iter().any(|other| covers(other, root)))
        .cloned()
        .collect()
}

fn covers(outer: &Path, inner: &Path) -> bool {
    outer != inner
        && inner
            .strip_prefix(outer)
            .is_ok_and(|rel| rel.components().count() < MAX_DEPTH)
        && is_discoverable_below(outer, inner)
}

fn walk_tree(start: PathBuf, total: &AtomicUsize) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![(start, 1_usize)];
    let mut dirs = 0_usize;
    while let Some((dir, depth)) = stack.pop() {
        if dirs >= MAX_DIRS_PER_TREE || found.len() >= MAX_PROJECTS_PER_TREE || !take(total) {
            break;
        }
        dirs += 1;
        let Some((marker, children)) = scan_dir(&dir) else {
            continue;
        };
        if depth < MAX_DEPTH {
            stack.extend(children.into_iter().rev().map(|child| (child, depth + 1)));
        }
        if marker {
            found.push(dir);
        }
    }
    found
}

fn take(total: &AtomicUsize) -> bool {
    total
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |left| left.checked_sub(1))
        .is_ok()
}

/// Marker presence plus child folders worth entering, sorted by name.
fn scan_dir(dir: &Path) -> Option<(bool, Vec<PathBuf>)> {
    let entries = fs::read_dir(dir).ok()?;
    let mut marker = false;
    let mut children = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        marker |= is_project_marker_name(name);
        // `file_type` comes from `d_type`; symlinked folders are not followed.
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) && !is_skip_name(name) {
            children.push(entry.path());
        }
    }
    children.sort();
    Some((marker, children))
}
