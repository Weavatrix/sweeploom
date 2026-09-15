//! Bounded metadata listing. Never follows symlinks or opens file contents.

use std::fs;
use std::path::{Path, PathBuf};

/// Walk caps. Tests pass smaller values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Stop after this many regular files.
    pub max_files: u32,
    /// Directories deeper than this are not entered.
    pub max_depth: u8,
    /// Immediate children kept for the inspect table.
    pub max_entries: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_files: 2_000_000,
            max_depth: 32,
            max_entries: 4_096,
        }
    }
}

/// One immediate child of an AI store root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreEntry {
    /// Path relative to the store root.
    pub relative: String,
    /// Rolled-up logical bytes.
    pub logical_bytes: u64,
    /// Regular files under this child.
    pub file_count: u64,
    /// True when this child is a directory.
    pub is_dir: bool,
    /// True when a cap stopped the walk under this child.
    pub capped: bool,
}

/// Metadata-only inventory of an AI store root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreInventory {
    /// Sum of `metadata.len()` for visited files.
    pub logical_bytes: u64,
    /// Number of regular files visited.
    pub file_count: u64,
    /// True when a cap stopped the walk.
    pub capped: bool,
    /// Immediate children. Not a nested dump of every file.
    pub entries: Vec<StoreEntry>,
}

struct SizeWalk {
    limits: Limits,
    logical_bytes: u64,
    file_count: u64,
    capped: bool,
    stack: Vec<(PathBuf, u8)>,
    seen: std::collections::HashSet<PathBuf>,
}

fn size_tree(root: &Path, limits: Limits) -> StoreInventory {
    let mut walk = SizeWalk {
        limits,
        logical_bytes: 0,
        file_count: 0,
        capped: false,
        stack: vec![(root.to_path_buf(), 0)],
        seen: std::collections::HashSet::new(),
    };
    if let Ok(canon) = fs::canonicalize(root) {
        walk.seen.insert(canon);
    }
    walk.run();
    StoreInventory {
        logical_bytes: walk.logical_bytes,
        file_count: walk.file_count,
        capped: walk.capped,
        entries: Vec::new(),
    }
}

/// Walk `root` using only directory listing and metadata.
#[must_use]
pub fn list_store(root: &Path, limits: Limits) -> StoreInventory {
    if let Some(listed) = list_if_file(root) {
        return listed;
    }
    list_directory(root, limits)
}

fn list_if_file(root: &Path) -> Option<StoreInventory> {
    let meta = fs::symlink_metadata(root).ok()?;
    if meta.file_type().is_symlink() || meta.is_dir() {
        return None;
    }
    let relative = relative_sample(root.parent().unwrap_or(root), root).unwrap_or_default();
    Some(StoreInventory {
        logical_bytes: meta.len(),
        file_count: 1,
        capped: false,
        entries: vec![StoreEntry {
            relative,
            logical_bytes: meta.len(),
            file_count: 1,
            is_dir: false,
            capped: false,
        }],
    })
}

fn list_directory(root: &Path, limits: Limits) -> StoreInventory {
    let Ok(read) = fs::read_dir(root) else {
        return StoreInventory {
            logical_bytes: 0,
            file_count: 0,
            capped: false,
            entries: Vec::new(),
        };
    };
    let mut children: Vec<PathBuf> = read.flatten().map(|entry| entry.path()).collect();
    children.sort();
    let mut entries = Vec::new();
    let mut logical_bytes = 0_u64;
    let mut file_count = 0_u64;
    let mut capped = false;
    for path in children {
        let Some(entry) = child_entry(root, &path, limits) else {
            continue;
        };
        logical_bytes = logical_bytes.saturating_add(entry.logical_bytes);
        file_count = file_count.saturating_add(entry.file_count);
        if entry.capped {
            capped = true;
        }
        entries.push(entry);
    }
    entries.sort_by_key(|item| std::cmp::Reverse(item.logical_bytes));
    if entries.len() > limits.max_entries {
        capped = true;
        entries.truncate(limits.max_entries);
    }
    StoreInventory {
        logical_bytes,
        file_count,
        capped,
        entries,
    }
}

fn child_entry(root: &Path, path: &Path, limits: Limits) -> Option<StoreEntry> {
    let meta = fs::symlink_metadata(path).ok()?;
    if meta.file_type().is_symlink() && !meta.is_dir() {
        return None;
    }
    let relative = relative_sample(root, path)?;
    if meta.is_dir() {
        let sized = size_tree(
            path,
            Limits {
                max_files: limits.max_files,
                max_depth: limits.max_depth.saturating_sub(1),
                max_entries: 0,
            },
        );
        return Some(StoreEntry {
            relative,
            logical_bytes: sized.logical_bytes,
            file_count: sized.file_count,
            is_dir: true,
            capped: sized.capped,
        });
    }
    Some(StoreEntry {
        relative,
        logical_bytes: meta.len(),
        file_count: 1,
        is_dir: false,
        capped: false,
    })
}

impl SizeWalk {
    fn run(&mut self) {
        while let Some((dir, depth)) = self.stack.pop() {
            if self.file_count >= u64::from(self.limits.max_files) {
                self.capped = true;
                break;
            }
            self.visit_dir(&dir, depth);
        }
    }

    fn visit_dir(&mut self, dir: &Path, depth: u8) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if self.file_count >= u64::from(self.limits.max_files) {
                self.capped = true;
                return;
            }
            self.visit_entry(&entry.path(), depth);
        }
    }

    fn visit_entry(&mut self, path: &Path, depth: u8) {
        let Ok(meta) = fs::symlink_metadata(path) else {
            return;
        };
        if meta.file_type().is_symlink() && !meta.is_dir() {
            return;
        }
        if meta.is_dir() {
            if depth >= self.limits.max_depth {
                self.capped = true;
                return;
            }
            if let Ok(canon) = fs::canonicalize(path)
                && !self.seen.insert(canon)
            {
                return;
            }
            self.stack
                .push((path.to_path_buf(), depth.saturating_add(1)));
            return;
        }
        self.logical_bytes = self.logical_bytes.saturating_add(meta.len());
        self.file_count = self.file_count.saturating_add(1);
    }
}

fn relative_sample(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let text = rel.to_string_lossy();
    (!text.is_empty()).then(|| text.replace('\\', "/"))
}
