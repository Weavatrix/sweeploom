//! Folder-level work queue. Workers read one folder at a time; a folder folds
//! into its parent as soon as its last child finishes, so memory stays bounded
//! by open folders and a single huge repository still uses every worker.

use std::collections::HashMap;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

use super::node::{DirectoryNode, InventoryLimits};
use super::scan_fold::{Fold, accumulate, insert_child};
use super::scan_meta::file_meta;

/// Counters every worker updates without the queue lock.
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
    fn count(&self) -> bool {
        let seen = self.entries.fetch_add(1, Ordering::Relaxed) + 1;
        if self.limits.max_entries.is_some_and(|max| seen >= max) {
            self.capped.store(true, Ordering::Relaxed);
            return false;
        }
        true
    }

    fn add_entries(&self, count: u64) {
        self.entries.fetch_add(count, Ordering::Relaxed);
    }

    pub fn entries(&self) -> u64 {
        self.entries.load(Ordering::Relaxed)
    }

    pub fn capped(&self) -> bool {
        self.capped.load(Ordering::Relaxed)
    }

    fn set_hint(&self, path: &Path) {
        if let (Ok(mut hint), Some(name)) = (self.hint.try_lock(), path.file_name()) {
            *hint = name.to_string_lossy().into_owned();
        }
    }

    pub fn hint(&self) -> String {
        self.hint
            .lock()
            .map(|hint| hint.clone())
            .unwrap_or_default()
    }
}

#[derive(Default)]
struct State {
    fold: Fold,
    /// Children of each open folder that have not finished yet.
    remaining: HashMap<PathBuf, usize>,
    stack: Vec<PathBuf>,
    active: usize,
    done: bool,
    /// Running totals of top-level folders that are still being read.
    partial: HashMap<PathBuf, DirectoryNode>,
}

pub(super) struct Queue<'a> {
    root: &'a Path,
    shared: &'a Shared,
    state: Mutex<State>,
    ready: Condvar,
}

/// One folder read off the lock.
struct Read {
    fold: Fold,
    node: DirectoryNode,
    children: Vec<PathBuf>,
}

impl<'a> Queue<'a> {
    pub fn new(root: &'a Path, shared: &'a Shared) -> Self {
        let state = State {
            stack: vec![root.to_path_buf()],
            ..State::default()
        };
        Self {
            root,
            shared,
            state: Mutex::new(state),
            ready: Condvar::new(),
        }
    }

    pub fn is_done(&self) -> bool {
        self.lock().done
    }

    /// Worker loop; returns once every folder is folded into the root.
    pub fn work(&self) {
        while let Some(dir) = self.next() {
            let read = catch_unwind(AssertUnwindSafe(|| self.read(&dir))).unwrap_or_else(|_| {
                let mut fold = Fold::default();
                fold.errors = 1;
                Read {
                    fold,
                    node: DirectoryNode::new(dir.clone()),
                    children: Vec::new(),
                }
            });
            self.finish(dir, read);
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn next(&self) -> Option<PathBuf> {
        let mut state = self.lock();
        loop {
            if let Some(dir) = state.stack.pop() {
                state.active += 1;
                return Some(dir);
            }
            if state.done || state.active == 0 {
                state.done = true;
                self.ready.notify_all();
                return None;
            }
            state = self
                .ready
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// List `dir` and size its files: the only I/O, done without the lock.
    fn read(&self, dir: &Path) -> Read {
        let limits = self.shared.limits;
        let mut fold = Fold::default();
        let mut node = DirectoryNode::new(dir.to_path_buf());
        if self.shared.capped() {
            node.incomplete = true;
            return Read {
                fold,
                node,
                children: Vec::new(),
            };
        }
        let mut children = Vec::new();
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries {
                    let Ok(entry) = entry else {
                        fold.errors += 1;
                        continue;
                    };
                    let path = entry.path();
                    match entry.file_type() {
                        Ok(kind) if kind.is_symlink() => self.shared.add_entries(1),
                        Ok(kind) if kind.is_dir() => {
                            fold.record_project(&path, self.root, limits.max_projects);
                            children.push(path);
                        }
                        Ok(_) => match file_meta(&path) {
                            Some(meta) if !self.shared.capped() => {
                                self.shared.count();
                                fold.record_project(&path, self.root, limits.max_projects);
                                fold.file(&path, &meta, limits);
                            }
                            Some(_) => {}
                            None => fold.errors += 1,
                        },
                        Err(_) => fold.errors += 1,
                    }
                }
                self.shared.set_hint(dir);
            }
            Err(_) => fold.errors += 1,
        }
        if fold.peek(dir).is_some() {
            node = fold.take(dir);
        }
        if self.shared.capped() {
            node.incomplete = true;
            children.clear();
        }
        children.sort();
        Read {
            fold,
            node,
            children,
        }
    }

    fn finish(&self, dir: PathBuf, read: Read) {
        let mut state = self.lock();
        // Soft cap; the report keeps the first `max_projects` in path order.
        let soft_cap = self.shared.limits.max_projects.saturating_mul(8);
        state.fold.merge(read.fold, soft_cap);
        if let Some(top) = top_level(self.root, &dir) {
            let partial = state
                .partial
                .entry(top.clone())
                .or_insert_with(|| DirectoryNode::new(top));
            accumulate(partial, &read.node);
        }
        state.fold.insert_node(read.node);
        let pushed = !read.children.is_empty();
        if pushed {
            state.remaining.insert(dir, read.children.len());
            state.stack.extend(read.children.into_iter().rev());
        } else {
            self.complete(&mut state, dir);
        }
        state.active -= 1;
        if pushed || (state.active == 0 && state.stack.is_empty()) {
            self.ready.notify_all();
        }
    }

    /// `dir` and all of its children are read: fold it up as far as possible.
    fn complete(&self, state: &mut State, mut dir: PathBuf) {
        let max_children = self.shared.limits.max_children_per_dir;
        loop {
            // A folder counts as an entry once its contents are in, like a contents-first walk.
            self.shared.add_entries(1);
            if dir == self.root {
                return;
            }
            state.remaining.remove(&dir);
            state
                .fold
                .directory(&dir, self.root, self.root, max_children);
            if dir.parent() == Some(self.root) {
                state.partial.remove(&dir);
            }
            let Some(parent) = dir.parent().map(Path::to_path_buf) else {
                return;
            };
            match state.remaining.get_mut(&parent) {
                Some(left) if *left > 1 => {
                    *left -= 1;
                    return;
                }
                Some(_) => dir = parent,
                None => return,
            }
        }
    }

    /// Root totals with finished top-level folders plus running totals of the rest.
    pub fn preview(&self) -> DirectoryNode {
        let state = self.lock();
        let max_children = self.shared.limits.max_children_per_dir;
        let mut root = state.fold.peek(self.root).map_or_else(
            || DirectoryNode::new(self.root.to_path_buf()),
            DirectoryNode::preview,
        );
        for partial in state.partial.values() {
            let mut leaf = partial.clone();
            leaf.incomplete = true;
            accumulate(&mut root, &leaf);
            root.directories = root.directories.saturating_add(1);
            root.incomplete = true;
            insert_child(&mut root.children, leaf, max_children);
        }
        root
    }

    pub fn into_fold(self) -> Fold {
        self.state
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner)
            .fold
    }
}

fn top_level(root: &Path, path: &Path) -> Option<PathBuf> {
    let first = path.strip_prefix(root).ok()?.components().next()?;
    Some(root.join(first))
}
