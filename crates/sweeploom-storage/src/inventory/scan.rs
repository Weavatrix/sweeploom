//! Parallel walk and bottom-up aggregation.
//!
//! Folders are read on a bounded worker pool from a shared stack; each folder
//! folds into its parent once its last child is done. Each file costs exactly
//! one `lstat`; folder names and types come from `readdir`.

use std::path::Path;
use std::time::{Duration, Instant};

use weavatrix_scan::WalkError;

use super::node::{DirectoryNode, InventoryLimits, InventoryReport};
use super::scan_fold::Fold;
use super::scan_queue::{Queue, Shared};
use crate::parallel::walk_workers;

const TICK: Duration = Duration::from_millis(150);

/// Live walk counters for the Explorer UI.
#[derive(Clone, Debug, Default)]
pub struct ScanTick {
    /// Entries visited so far.
    pub entries: u64,
    /// Logical bytes rolled up to the scan root, if known.
    pub logical_bytes: u64,
    /// Allocated bytes so far, when available.
    pub allocated_bytes: Option<u64>,
    /// Path currently being absorbed.
    pub hint: String,
    /// Snapshot of the root node so the table can stream folders.
    pub root: Option<DirectoryNode>,
}

/// Scan `root` into a folder-inspector tree.
pub fn scan_inventory(root: &Path, limits: InventoryLimits) -> Result<InventoryReport, WalkError> {
    scan_inventory_with(root, limits, |_| {})
}

/// Scan and report progress. `on_tick` runs on the calling thread while
/// subtree workers walk in parallel.
pub fn scan_inventory_with(
    root: &Path,
    limits: InventoryLimits,
    mut on_tick: impl FnMut(ScanTick),
) -> Result<InventoryReport, WalkError> {
    let canonical = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let scan_root = super::scan_meta::map_entry(&canonical).into_owned();
    let shared = Shared::new(limits);
    let queue = Queue::new(&scan_root, &shared);
    std::thread::scope(|scope| {
        for _ in 0..walk_workers() {
            scope.spawn(|| queue.work());
        }
        let mut last_tick = Instant::now();
        while !queue.is_done() {
            std::thread::sleep(Duration::from_millis(20));
            if last_tick.elapsed() >= TICK {
                last_tick = Instant::now();
                let root = queue.preview();
                on_tick(ScanTick {
                    entries: shared.entries(),
                    logical_bytes: root.logical_bytes,
                    allocated_bytes: root.allocated_bytes,
                    hint: shared.hint(),
                    root: Some(root),
                });
            }
        }
    });
    Ok(finish(queue.into_fold(), &scan_root, &shared))
}

fn finish(mut fold: Fold, scan_root: &Path, shared: &Shared) -> InventoryReport {
    let capped = shared.capped();
    let mut tree = fold.take(scan_root);
    tree.incomplete |= capped;
    tree.logical_bytes = tree.logical_bytes.max(fold.visited_logical);
    if fold.is_project(scan_root) {
        fold.project_bytes
            .insert(scan_root.to_path_buf(), tree.logical_bytes);
        fold.project_disk_bytes
            .insert(scan_root.to_path_buf(), tree.disk_bytes());
    }
    // Workers finish in any order; keep a stable, path-ordered project list.
    let mut projects = fold.projects;
    projects.sort();
    projects.truncate(shared.limits.max_projects);
    let kept: std::collections::HashSet<&Path> =
        projects.iter().map(|path| path.as_path()).collect();
    fold.project_bytes
        .retain(|path, _| kept.contains(path.as_path()));
    fold.project_disk_bytes
        .retain(|path, _| kept.contains(path.as_path()));
    InventoryReport {
        root: scan_root.to_path_buf(),
        tree,
        projects,
        project_bytes: fold.project_bytes.into_iter().collect(),
        project_disk_bytes: fold.project_disk_bytes.into_iter().collect(),
        entries: shared.entries(),
        errors: fold.errors,
        capped,
    }
}
