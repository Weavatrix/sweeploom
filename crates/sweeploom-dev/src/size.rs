//! Bounded directory sizing. Does not follow symlinks.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Sum of `metadata.len()` under `root`.
#[must_use]
pub fn dir_logical_bytes(root: &Path) -> u64 {
    let mut total = 0_u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            accumulate(&mut total, &mut stack, &entry.path());
        }
    }
    total
}

fn accumulate(total: &mut u64, stack: &mut Vec<PathBuf>, path: &Path) {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    if meta.is_dir() {
        stack.push(path.to_path_buf());
    } else {
        *total = total.saturating_add(meta.len());
    }
}

/// Directory or file mtime. `None` when the path cannot be read.
#[must_use]
pub fn path_mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}
