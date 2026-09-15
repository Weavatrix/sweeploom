//! Bounded directory sizing. Does not follow symlinks.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use sweeploom_core::SizeMeasurement;

const MAX_FILES: u32 = 12_000;

/// Sum of `metadata.len()` under `root`. Stops after 12_000 files.
#[must_use]
pub fn dir_logical_bytes(root: &Path) -> u64 {
    dir_size(root).bytes
}

/// Logical size with coverage. A cap or read error is never reported as Exact.
#[must_use]
pub fn dir_size(root: &Path) -> SizeMeasurement {
    let mut total = 0_u64;
    let mut files = 0_u32;
    let mut errors = 0_u32;
    let mut stack = vec![root.to_path_buf()];
    let mut complete = true;
    while let Some(dir) = stack.pop() {
        if files >= MAX_FILES {
            complete = false;
            break;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => {
                errors = errors.saturating_add(1);
                complete = false;
                continue;
            }
        };
        for entry in entries {
            let Ok(entry) = entry else {
                errors = errors.saturating_add(1);
                complete = false;
                continue;
            };
            accumulate(
                &mut total,
                &mut stack,
                &mut files,
                &mut errors,
                &entry.path(),
            );
            if files >= MAX_FILES {
                complete = false;
                break;
            }
        }
    }
    if complete && errors == 0 {
        SizeMeasurement::exact(total, files)
    } else {
        SizeMeasurement::lower_bound(total, files, errors)
    }
}

fn accumulate(
    total: &mut u64,
    stack: &mut Vec<PathBuf>,
    files: &mut u32,
    errors: &mut u32,
    path: &Path,
) {
    let Ok(meta) = fs::symlink_metadata(path) else {
        *errors = errors.saturating_add(1);
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    if meta.is_dir() {
        stack.push(path.to_path_buf());
    } else {
        *total = total.saturating_add(meta.len());
        *files = files.saturating_add(1);
    }
}

/// Directory or file mtime. `None` when the path cannot be read.
#[must_use]
pub fn path_mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}
