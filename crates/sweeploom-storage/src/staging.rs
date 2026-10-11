//! Bounded, read-only discovery of old deployment build output.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A generated Next.js directory in a deployment staging tree.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedBuild {
    /// Absolute path to the generated directory.
    pub path: PathBuf,
    /// Allocated bytes on Unix, logical bytes on other platforms.
    pub bytes: u64,
    /// Last modification time as Unix seconds.
    pub modified_at: u64,
}

/// Find old `runtime/.next` directories under a real staging root.
///
/// The result is evidence for review, not permission to delete. Symlinks are
/// never followed, and the walk is limited to eight directory levels and 500
/// build candidates.
pub fn scan_staging_builds(
    root: &Path,
    before: SystemTime,
) -> std::io::Result<Vec<GeneratedBuild>> {
    let root_meta = fs::symlink_metadata(root)?;
    if !root_meta.is_dir() || root_meta.file_type().is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "staging root must be a real directory",
        ));
    }
    let mut pending = vec![(root.to_path_buf(), 0usize)];
    let mut found = Vec::new();
    while let Some((dir, depth)) = pending.pop() {
        if depth > 8 {
            continue;
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if !kind.is_dir() || kind.is_symlink() {
                continue;
            }
            let path = entry.path();
            if entry.file_name() == ".next" && dir.file_name().is_some_and(|name| name == "runtime")
            {
                let meta = fs::symlink_metadata(&path)?;
                if meta.modified()? < before {
                    found.push(GeneratedBuild {
                        path: path.clone(),
                        bytes: allocated_tree_bytes(&path)?,
                        modified_at: meta
                            .modified()?
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    });
                    if found.len() > 500 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "staging build candidate limit exceeded",
                        ));
                    }
                }
                continue;
            }
            pending.push((path, depth + 1));
        }
    }
    found.sort_by_key(|build| std::cmp::Reverse(build.bytes));
    Ok(found)
}

fn allocated_tree_bytes(root: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    let mut pending = vec![root.to_path_buf()];
    let mut visited = 0u64;
    while let Some(path) = pending.pop() {
        visited += 1;
        if visited > 2_000_000 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "staging build entry limit exceeded",
            ));
        }
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            continue;
        }
        total = total.saturating_add(allocated_bytes(&meta));
        if meta.is_dir() {
            for entry in fs::read_dir(path)? {
                pending.push(entry?.path());
            }
        }
    }
    Ok(total)
}

#[cfg(unix)]
fn allocated_bytes(meta: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    meta.blocks().saturating_mul(512)
}

#[cfg(not(unix))]
fn allocated_bytes(meta: &fs::Metadata) -> u64 {
    meta.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_only_old_runtime_output() {
        let root = std::env::temp_dir().join(format!("sweeploom-staging-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let old = root.join("release/runtime/.next");
        let other = root.join("release/source/.next");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&other).unwrap();
        fs::write(old.join("artifact"), vec![1u8; 4096]).unwrap();
        let result = scan_staging_builds(&root, SystemTime::now()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, old);
        assert!(result[0].bytes > 0);
        fs::remove_dir_all(root).unwrap();
    }
}
