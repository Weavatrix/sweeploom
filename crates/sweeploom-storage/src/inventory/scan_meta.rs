//! Per-entry metadata helpers for the inventory walk.

use std::path::Path;
use std::time::SystemTime;

/// One `lstat` worth of file facts.
pub(super) struct FileMeta {
    pub bytes: u64,
    pub allocated: Option<u64>,
    pub mtime: Option<SystemTime>,
}

/// The only metadata call per file: size, allocated blocks and mtime together.
pub(super) fn file_meta(path: &Path) -> Option<FileMeta> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    #[cfg(unix)]
    let allocated = {
        use std::os::unix::fs::MetadataExt;
        Some(meta.blocks().saturating_mul(512))
    };
    #[cfg(not(unix))]
    let allocated = None;
    Some(FileMeta {
        bytes: meta.len(),
        allocated,
        mtime: meta.modified().ok(),
    })
}
