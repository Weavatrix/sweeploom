//! Explicit, recoverable manual removal. Never used by automatic cleanup.
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use sweeploom_core::{FileIdentity, ProcessSnapshot};
use sweeploom_platform::file_identity;

/// Immutable identity captured before showing the confirmation.
#[derive(Clone, Debug)]
pub struct TrashTarget {
    /// Exact canonical path.
    pub path: PathBuf,
    identity: FileIdentity,
    modified: Option<SystemTime>,
}

/// Reason a path cannot be manually moved to Trash.
#[must_use]
pub fn trash_path_reason(path: &Path, home: &Path) -> Option<String> {
    let canonical = std::fs::canonicalize(path).ok()?;
    let home = std::fs::canonicalize(home).unwrap_or_else(|_| home.to_path_buf());
    if canonical == home || !canonical.starts_with(&home) {
        return Some(
            "Only individual files/folders inside your home can be moved to Trash.".into(),
        );
    }
    let relative = canonical.strip_prefix(&home).ok()?;
    let text = relative.to_string_lossy().replace('\\', "/");
    if matches!(
        text.as_str(),
        "Library"
            | "Library/Containers"
            | "Library/Developer"
            | "Library/Application Support"
            | "Library/Keychains"
    ) || [
        ".ssh",
        ".gnupg",
        ".Trash",
        ".codex/worktrees",
        ".codex/sqlite",
        "Library/Developer/CoreSimulator",
        "Library/Containers/com.docker.docker",
    ]
    .iter()
    .any(|root| text == *root || text.starts_with(&format!("{root}/")))
        || [".codex", ".cursor", ".claude", ".grok"].contains(&text.as_str())
        || canonical
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                let lower = name.to_ascii_lowercase();
                lower == ".git"
                    || lower.contains("auth")
                    || lower.contains("credential")
                    || lower.contains("secret")
                    || (lower
                        .split(|ch: char| !ch.is_ascii_alphanumeric())
                        .any(|part| part == "token")
                        && !lower.ends_with(".md"))
                    || lower.ends_with(".sqlite")
                    || lower.ends_with(".sqlite-wal")
                    || lower.ends_with(".sqlite-shm")
                    || lower.ends_with(".sqlite3")
                    || lower.ends_with(".sqlite3-wal")
                    || lower.ends_with(".sqlite3-shm")
                    || lower.ends_with(".db")
                    || lower.ends_with(".db-wal")
                    || lower.ends_with(".db-shm")
                    || lower.contains(".vscdb")
            })
    {
        return Some("Protected application store, credentials, database, or managed worktree. Open in Finder to inspect.".into());
    }
    None
}

/// Capture a manually selected path; reject symlink targets and protected roots.
pub fn prepare_trash(path: &Path, home: &Path) -> Result<TrashTarget, String> {
    let meta = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("Symlinks cannot be trashed from the inspector.".into());
    }
    if let Some(reason) = trash_path_reason(path, home) {
        return Err(reason);
    }
    let canonical = std::fs::canonicalize(path).map_err(|error| error.to_string())?;
    let identity = file_identity(&canonical).ok_or("File identity unavailable")?;
    Ok(TrashTarget {
        path: canonical,
        identity,
        modified: meta.modified().ok(),
    })
}

/// Recheck identities and live process evidence, then use the OS Trash backend.
/// Returns successful paths and per-path errors, without a permanent-delete fallback.
pub fn apply_trash_with(
    targets: &[TrashTarget],
    home: &Path,
    processes: &[ProcessSnapshot],
) -> (Vec<PathBuf>, Vec<String>) {
    apply_with(targets, home, processes, |path| {
        trash::delete(path).map_err(|error| error.to_string())
    })
}

fn apply_with(
    targets: &[TrashTarget],
    home: &Path,
    processes: &[ProcessSnapshot],
    mut send: impl FnMut(&Path) -> Result<(), String>,
) -> (Vec<PathBuf>, Vec<String>) {
    let mut removed = Vec::new();
    let mut errors = Vec::new();
    for target in targets {
        let check = prepare_trash(&target.path, home).and_then(|live| {
            if live.identity != target.identity || live.modified != target.modified {
                return Err("Changed since confirmation; select again.".into());
            }
            if processes
                .iter()
                .any(|process| process.uses_path(&target.path))
            {
                return Err("A running process is using this folder.".into());
            }
            Ok(())
        });
        match check.and_then(|()| send(&target.path)) {
            Ok(()) => removed.push(target.path.clone()),
            Err(error) => errors.push(format!("{}: {error}", target.path.display())),
        }
    }
    (removed, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replacement_and_protected_paths_never_reach_backend() {
        let home = std::env::temp_dir().join(format!("sweeploom-trash-{}", std::process::id()));
        std::fs::create_dir_all(home.join(".ssh")).unwrap();
        let path = home.join("installer.dmg");
        std::fs::write(&path, b"old").unwrap();
        let target = prepare_trash(&path, &home).unwrap();
        assert!(prepare_trash(&home, &home).is_err());
        assert!(prepare_trash(&home.join(".ssh"), &home).is_err());
        std::fs::rename(&path, home.join("old")).unwrap();
        std::fs::write(&path, b"new").unwrap();
        let (removed, errors) = apply_with(&[target], &home, &[], |_| {
            panic!("must not trash replacement")
        });
        assert!(removed.is_empty());
        assert_eq!(errors.len(), 1);
        std::fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn explicit_file_reaches_only_recoverable_backend() {
        let home = std::env::temp_dir().join(format!("sweeploom-trash-ok-{}", std::process::id()));
        std::fs::create_dir_all(&home).unwrap();
        let path = home.join("download.zip");
        std::fs::write(&path, b"download").unwrap();
        let target = prepare_trash(&path, &home).unwrap();
        let mut called = 0;
        let (removed, errors) = apply_with(&[target], &home, &[], |_| {
            called += 1;
            Ok(())
        });
        assert_eq!(called, 1);
        assert_eq!(removed.len(), 1);
        assert!(errors.is_empty());
        std::fs::remove_dir_all(home).unwrap();
    }
}
