//! Full directory usage, independent of generated-artifact discovery limits.

use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use weavatrix_scan::{
    ParallelWalker, RootSymlinkPolicy, WalkControl, WalkEntry, WalkEvent, WalkOptions,
};

/// A completed directory walk, with explicit coverage and allocated disk usage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DiskUsage {
    /// Allocated file and directory bytes; logical file bytes on non-Unix systems.
    pub bytes: u64,
    /// Logical bytes of files, counted once per hard-linked inode on Unix.
    pub logical_bytes: u64,
    /// Files counted.
    pub files: u64,
    /// Read errors. A result with errors is a lower bound, never an exact zero.
    pub errors: u64,
    /// True only when the entire walk completed without read errors.
    pub complete: bool,
}

impl Default for DiskUsage {
    fn default() -> Self {
        Self {
            bytes: 0,
            logical_bytes: 0,
            files: 0,
            errors: 0,
            complete: true,
        }
    }
}

/// Measure the entire directory, including source, hidden files and generated
/// output. No file-count cap or inspector row cap is applied. Symlinks are not
/// followed; sparse files use allocated blocks and hard links count once on Unix.
/// Folders are read in parallel on the Weavatrix Scan worker pool.
pub fn directory_disk_usage(root: &Path) -> io::Result<DiskUsage> {
    let metadata = fs::symlink_metadata(root)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a directory, not a symlink",
        ));
    }
    // A missing or unreadable root must fail, rather than looking like an empty tree.
    drop(fs::read_dir(root)?);
    let totals = Arc::new(Totals::default());
    let visitor = Arc::clone(&totals);
    ParallelWalker::new(root)
        .options(
            WalkOptions::default()
                .with_follow_links(false)
                .with_root_symlink_policy(RootSymlinkPolicy::Reject),
        )
        .with_parallelism(crate::walk_workers())
        .visit(move |event| {
            match event {
                WalkEvent::Entry(entry) => visitor.absorb(entry),
                WalkEvent::Error(_) => visitor.error(),
            }
            WalkControl::Continue
        })
        .map_err(|error| io::Error::other(error.to_string()))?;
    let load = |value: &AtomicU64| value.load(Ordering::Relaxed);
    let errors = load(&totals.errors);
    Ok(DiskUsage {
        bytes: load(&totals.bytes),
        logical_bytes: load(&totals.logical_bytes),
        files: load(&totals.files),
        errors,
        complete: errors == 0,
    })
}

#[derive(Default)]
struct Totals {
    bytes: AtomicU64,
    logical_bytes: AtomicU64,
    files: AtomicU64,
    errors: AtomicU64,
    #[cfg(unix)]
    hard_links: std::sync::Mutex<std::collections::HashSet<(u64, u64)>>,
}

impl Totals {
    fn error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    fn absorb(&self, entry: &WalkEntry) {
        if entry.is_symlink() {
            return;
        }
        let Ok(metadata) = fs::symlink_metadata(entry.path()) else {
            self.error();
            return;
        };
        if metadata.file_type().is_symlink() {
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.is_file()
                && metadata.nlink() > 1
                && !self.hard_links.lock().map_or(true, |mut seen| {
                    seen.insert((metadata.dev(), metadata.ino()))
                })
            {
                return;
            }
            let blocks = metadata.blocks().saturating_mul(512);
            self.bytes.fetch_add(blocks, Ordering::Relaxed);
        }
        if metadata.is_file() {
            #[cfg(not(unix))]
            self.bytes.fetch_add(metadata.len(), Ordering::Relaxed);
            self.logical_bytes
                .fetch_add(metadata.len(), Ordering::Relaxed);
            self.files.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_size_includes_source_and_files_beyond_the_artifact_cap() {
        let root = std::env::temp_dir().join(format!("sweeploom-full-size-{}", std::process::id()));
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(root.join("package.json"), b"{}").unwrap();
        fs::write(root.join(".gitignore"), b"node_modules\n").unwrap();
        let mut expected = 2 + 13;
        for index in 0..12_050 {
            fs::write(
                root.join("node_modules").join(index.to_string()),
                b"dependency",
            )
            .unwrap();
            expected += 10;
        }
        fs::write(root.join("source.ts"), b"source code").unwrap();
        expected += 11;
        let usage = directory_disk_usage(&root).unwrap();
        assert!(usage.complete);
        assert_eq!(usage.files, 12_053);
        assert_eq!(usage.logical_bytes, expected);
        assert!(usage.bytes > 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn sparse_files_hard_links_and_external_symlinks_match_disk_usage() {
        use std::os::unix::fs::{MetadataExt, symlink};
        let root = std::env::temp_dir().join(format!("sweeploom-disk-size-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let sparse = root.join("sparse");
        fs::File::create(&sparse)
            .unwrap()
            .set_len(1_000_000_000)
            .unwrap();
        fs::write(root.join("data"), vec![1; 8192]).unwrap();
        fs::hard_link(root.join("data"), root.join("linked")).unwrap();
        symlink(std::env::temp_dir(), root.join("external")).unwrap();
        let usage = directory_disk_usage(&root).unwrap();
        assert!(usage.complete);
        assert_eq!(usage.files, 2);
        assert_eq!(usage.logical_bytes, 1_000_008_192);
        let expected = fs::metadata(&root).unwrap().blocks() * 512
            + fs::metadata(&sparse).unwrap().blocks() * 512
            + fs::metadata(root.join("data")).unwrap().blocks() * 512;
        assert_eq!(usage.bytes, expected);
        assert!(usage.bytes < usage.logical_bytes);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_directory_is_an_error_not_an_exact_zero() {
        let path =
            std::env::temp_dir().join(format!("sweeploom-size-missing-{}", std::process::id()));
        assert!(directory_disk_usage(&path).is_err());
    }
}
