//! Split a scan into parallel subtrees and stream their totals while they finish.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use weavatrix_scan::{RootSymlinkPolicy, WalkBuilder, WalkOptions};

use super::node::{DirectoryNode, InventoryLimits};
use super::scan_fold::{FileMeta, Fold, accumulate, file_meta, insert_child};
use crate::parallel::parallel_map;

/// Expand at most this many levels below the root on the calling side.
const MAX_SPLIT_DEPTH: usize = 3;

/// Counters every worker updates.
pub(super) struct Shared {
    pub limits: InventoryLimits,
    entries: AtomicU64,
    capped: AtomicBool,
    hint: Mutex<String>,
}

impl Shared {
    pub fn new(limits: InventoryLimits) -> Self {
        Self {
            limits,
            entries: AtomicU64::new(0),
            capped: AtomicBool::new(false),
            hint: Mutex::new(String::new()),
        }
    }

    /// Count one entry. `false` once this entry reaches the cap.
    pub fn count(&self) -> bool {
        let seen = self.entries.fetch_add(1, Ordering::Relaxed) + 1;
        if self.limits.max_entries.is_some_and(|max| seen >= max) {
            self.capped.store(true, Ordering::Relaxed);
            return false;
        }
        true
    }

    pub fn add_entries(&self, count: u64) {
        self.entries.fetch_add(count, Ordering::Relaxed);
    }

    pub fn entries(&self) -> u64 {
        self.entries.load(Ordering::Relaxed)
    }

    pub fn capped(&self) -> bool {
        self.capped.load(Ordering::Relaxed)
    }

    pub fn set_hint(&self, path: &Path) {
        if let (Ok(mut hint), Some(name)) = (self.hint.lock(), path.file_name()) {
            *hint = name.to_string_lossy().into_owned();
        }
    }

    pub fn hint(&self) -> String {
        self.hint.lock().map(|hint| hint.clone()).unwrap_or_default()
    }
}

/// Folders read on the calling side (`shells`) and subtrees left for workers.
pub(super) struct Plan {
    pub shells: Vec<PathBuf>,
    pub tasks: Vec<PathBuf>,
}

struct Listing {
    dir: PathBuf,
    files: Vec<(PathBuf, FileMeta)>,
    dirs: Vec<PathBuf>,
    symlinks: u64,
    errors: u64,
}

/// Read the root, then a few more levels while there are too few subtrees to
/// keep `workers` busy. Shell files are absorbed into `fold` immediately.
pub(super) fn plan(scan_root: &Path, shared: &Shared, fold: &mut Fold, workers: usize) -> Plan {
    let target = workers.saturating_mul(4);
    let mut shells = Vec::new();
    let mut frontier = vec![scan_root.to_path_buf()];
    for level in 0..MAX_SPLIT_DEPTH {
        if level > 0 && (frontier.len() >= target || shared.capped()) {
            break;
        }
        let listings = parallel_map(std::mem::take(&mut frontier), workers, list);
        for listing in listings {
            absorb_listing(listing, scan_root, shared, fold, &mut shells, &mut frontier);
        }
    }
    Plan {
        shells,
        tasks: frontier,
    }
}

fn absorb_listing(
    listing: Listing,
    scan_root: &Path,
    shared: &Shared,
    fold: &mut Fold,
    shells: &mut Vec<PathBuf>,
    frontier: &mut Vec<PathBuf>,
) {
    let limits = shared.limits;
    fold.errors = fold.errors.saturating_add(listing.errors);
    fold.record_project(&listing.dir, scan_root, limits.max_projects);
    for (path, meta) in &listing.files {
        if shared.capped() {
            break;
        }
        shared.count();
        fold.record_project(path, scan_root, limits.max_projects);
        fold.file(path, meta, limits);
    }
    shared.add_entries(listing.symlinks);
    if fold.peek(&listing.dir).is_none() {
        fold.insert_node(DirectoryNode::new(listing.dir.clone()));
    }
    frontier.extend(listing.dirs);
    shells.push(listing.dir);
}

fn list(dir: PathBuf) -> Listing {
    let mut listing = Listing {
        dir,
        files: Vec::new(),
        dirs: Vec::new(),
        symlinks: 0,
        errors: 0,
    };
    let Ok(entries) = fs::read_dir(&listing.dir) else {
        listing.errors = 1;
        return listing;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            listing.errors += 1;
            continue;
        };
        let path = entry.path();
        match entry.file_type() {
            Ok(kind) if kind.is_symlink() => listing.symlinks += 1,
            Ok(kind) if kind.is_dir() => listing.dirs.push(path),
            Ok(_) => match file_meta(&path) {
                Some(meta) => listing.files.push((path, meta)),
                None => listing.errors += 1,
            },
            Err(_) => listing.errors += 1,
        }
    }
    listing.dirs.sort();
    listing
}

/// Contents-first walk of one subtree on a worker thread.
pub(super) fn walk(dir: &Path, scan_root: &Path, shared: &Shared) -> (DirectoryNode, Fold) {
    let mut fold = Fold::default();
    let walker = WalkBuilder::new(dir)
        .options(
            WalkOptions::default()
                .with_follow_links(false)
                .with_same_file_system(false)
                .with_root_symlink_policy(RootSymlinkPolicy::Reject),
        )
        .contents_first(true)
        .build();
    let mut capped = shared.capped();
    let mut seen = 0_u32;
    for item in walker {
        if capped || shared.capped() {
            capped = true;
            break;
        }
        let Ok(entry) = item else {
            fold.errors += 1;
            continue;
        };
        let more = shared.count();
        seen = seen.wrapping_add(1);
        if seen % 512 == 0 {
            shared.set_hint(entry.path());
        }
        fold.absorb(&entry, dir, scan_root, shared.limits);
        capped = !more;
    }
    let node = if capped {
        fold.fold_partial(dir)
    } else {
        fold.take(dir)
    };
    (node, fold)
}

/// Leaf totals per top-level folder, updated as subtrees finish.
pub(super) struct Preview {
    top: HashMap<PathBuf, (DirectoryNode, usize)>,
}

impl Preview {
    pub fn new(scan_root: &Path, plan: &Plan, fold: &Fold) -> Self {
        let mut top = HashMap::new();
        for shell in plan.shells.iter().filter(|shell| *shell != scan_root) {
            if let (Some(node), Some((summary, _))) =
                (fold.peek(shell), summary(&mut top, scan_root, shell))
            {
                accumulate(summary, node);
            }
        }
        for task in &plan.tasks {
            if let Some((summary, pending)) = summary(&mut top, scan_root, task) {
                *pending += 1;
                summary.incomplete = true;
            }
        }
        Self { top }
    }

    pub fn finished(&mut self, scan_root: &Path, node: &DirectoryNode) {
        let Some(key) = top_level(scan_root, &node.path) else {
            return;
        };
        if let Some((summary, pending)) = self.top.get_mut(&key) {
            accumulate(summary, node);
            *pending = pending.saturating_sub(1);
            summary.incomplete = *pending > 0 || node.incomplete;
        }
    }

    /// Root totals with one leaf row per top-level folder, partial ones included.
    pub fn root(&self, scan_root: &Path, fold: &Fold, max_children: usize) -> DirectoryNode {
        let mut root = fold
            .peek(scan_root)
            .map_or_else(|| DirectoryNode::new(scan_root.to_path_buf()), DirectoryNode::preview);
        for (summary, _) in self.top.values() {
            accumulate(&mut root, summary);
            root.directories = root.directories.saturating_add(1);
            root.incomplete |= summary.incomplete;
            insert_child(&mut root.children, summary.clone(), max_children);
        }
        root
    }
}

fn summary<'a>(
    top: &'a mut HashMap<PathBuf, (DirectoryNode, usize)>,
    scan_root: &Path,
    path: &Path,
) -> Option<&'a mut (DirectoryNode, usize)> {
    let key = top_level(scan_root, path)?;
    Some(
        top.entry(key.clone())
            .or_insert_with(|| (DirectoryNode::new(key), 0)),
    )
}

fn top_level(scan_root: &Path, path: &Path) -> Option<PathBuf> {
    let first = path.strip_prefix(scan_root).ok()?.components().next()?;
    Some(scan_root.join(first))
}
