//! Parallel walk and bottom-up aggregation.
//!
//! The root (and, for narrow trees, a few levels below it) is listed first;
//! every remaining subtree is walked contents-first on a worker and folded
//! into the root as it finishes. Each file costs exactly one `lstat`.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use weavatrix_scan::WalkError;

use super::node::{DirectoryNode, InventoryLimits, InventoryReport};
use super::scan_fold::Fold;
use super::scan_plan::{Preview, Shared, plan, walk};
use crate::parallel::{parallel_map, walk_workers};

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
    let scan_root = super::scan_fold::map_entry(&canonical).into_owned();
    let workers = walk_workers();
    let shared = Shared::new(limits);
    let mut fold = Fold::default();
    let plan = plan(&scan_root, &shared, &mut fold, workers);
    let mut preview = Preview::new(&scan_root, &plan, &fold);
    let mut finished: Vec<Option<(DirectoryNode, Fold)>> = plan.tasks.iter().map(|_| None).collect();
    let tasks: Vec<(usize, PathBuf)> = plan.tasks.iter().cloned().enumerate().collect();
    let (tx, rx) = mpsc::channel();
    std::thread::scope(|scope| {
        let (shared, scan_root) = (&shared, scan_root.as_path());
        scope.spawn(move || {
            parallel_map(tasks, workers, |(index, dir)| {
                let _ = tx.send((index, walk(&dir, scan_root, shared)));
            });
        });
        let mut last_tick = Instant::now();
        loop {
            match rx.recv_timeout(TICK) {
                Ok((index, (node, task))) => {
                    preview.finished(scan_root, &node);
                    finished[index] = Some((node, task));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if last_tick.elapsed() >= TICK {
                last_tick = Instant::now();
                let root = preview.root(scan_root, &fold, limits.max_children_per_dir);
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
    // Merge in task order so project lists do not depend on worker timing.
    for (node, task) in finished.into_iter().flatten() {
        fold.merge(task, limits.max_projects);
        fold.insert_node(node);
    }
    shared.add_entries(plan.shells.len() as u64);
    Ok(finish(fold, &scan_root, &shared))
}

fn finish(mut fold: Fold, scan_root: &Path, shared: &Shared) -> InventoryReport {
    let capped = shared.capped();
    let mut tree = fold.collapse(scan_root, shared.limits.max_children_per_dir, scan_root);
    tree.incomplete |= capped;
    tree.logical_bytes = tree.logical_bytes.max(fold.visited_logical);
    if fold.is_project(scan_root) {
        fold.project_bytes
            .insert(scan_root.to_path_buf(), tree.logical_bytes);
        fold.project_disk_bytes
            .insert(scan_root.to_path_buf(), tree.disk_bytes());
    }
    InventoryReport {
        root: scan_root.to_path_buf(),
        tree,
        projects: fold.projects,
        project_bytes: fold.project_bytes.into_iter().collect(),
        project_disk_bytes: fold.project_disk_bytes.into_iter().collect(),
        entries: shared.entries(),
        errors: fold.errors,
        capped,
    }
}
