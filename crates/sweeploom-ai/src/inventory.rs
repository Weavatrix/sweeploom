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
    /// Relative paths kept for inspect UI.
    pub max_samples: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_files: 4_000,
            max_depth: 4,
            max_samples: 8,
        }
    }
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
    /// Relative sample paths. Not a full listing.
    pub samples: Vec<String>,
}

struct Walk<'a> {
    root: &'a Path,
    limits: Limits,
    logical_bytes: u64,
    file_count: u64,
    capped: bool,
    samples: Vec<String>,
    stack: Vec<(PathBuf, u8)>,
}

/// Walk `root` using only directory listing and metadata.
#[must_use]
pub fn list_store(root: &Path, limits: Limits) -> StoreInventory {
    if let Some(listed) = list_if_file(root) {
        return listed;
    }
    let mut walk = Walk {
        root,
        limits,
        logical_bytes: 0,
        file_count: 0,
        capped: false,
        samples: Vec::new(),
        stack: vec![(root.to_path_buf(), 0)],
    };
    walk.run();
    StoreInventory {
        logical_bytes: walk.logical_bytes,
        file_count: walk.file_count,
        capped: walk.capped,
        samples: walk.samples,
    }
}

fn list_if_file(root: &Path) -> Option<StoreInventory> {
    let meta = fs::symlink_metadata(root).ok()?;
    if meta.file_type().is_symlink() || meta.is_dir() {
        return None;
    }
    let samples = relative_sample(root.parent().unwrap_or(root), root)
        .into_iter()
        .collect();
    Some(StoreInventory {
        logical_bytes: meta.len(),
        file_count: 1,
        capped: false,
        samples,
    })
}

impl Walk<'_> {
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
        if meta.file_type().is_symlink() {
            return;
        }
        if meta.is_dir() {
            self.push_dir(path, depth);
            return;
        }
        self.add_file(path, meta.len());
    }

    fn push_dir(&mut self, path: &Path, depth: u8) {
        if depth < self.limits.max_depth {
            self.stack
                .push((path.to_path_buf(), depth.saturating_add(1)));
        } else {
            self.capped = true;
        }
    }

    fn add_file(&mut self, path: &Path, len: u64) {
        self.logical_bytes = self.logical_bytes.saturating_add(len);
        self.file_count = self.file_count.saturating_add(1);
        if self.samples.len() < self.limits.max_samples
            && let Some(sample) = relative_sample(self.root, path)
        {
            self.samples.push(sample);
        }
    }
}

fn relative_sample(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let text = rel.to_string_lossy();
    (!text.is_empty()).then(|| text.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn unique_root() -> PathBuf {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "sweeploom-ai-{}-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|item| item.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn counts_files_from_metadata_only() {
        let root = unique_root();
        fs::create_dir_all(root.join("a")).unwrap();
        fs::write(root.join("a").join("hello.txt"), b"abcd").unwrap();
        fs::write(root.join("note.md"), b"xy").unwrap();
        let listed = list_store(&root, Limits::default());
        fs::remove_dir_all(&root).ok();
        assert_eq!(listed.file_count, 2);
        assert_eq!(listed.logical_bytes, 6);
        assert!(!listed.capped);
        assert!(
            listed
                .samples
                .iter()
                .any(|item| item.ends_with("hello.txt"))
        );
    }

    #[test]
    fn file_cap_is_honest() {
        let root = unique_root();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("one.txt"), b"a").unwrap();
        fs::write(root.join("two.txt"), b"b").unwrap();
        let listed = list_store(
            &root,
            Limits {
                max_files: 1,
                max_depth: 4,
                max_samples: 8,
            },
        );
        fs::remove_dir_all(&root).ok();
        assert_eq!(listed.file_count, 1);
        assert!(listed.capped);
    }
}
