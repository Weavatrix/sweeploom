//! Bounded Git probes: branch from HEAD, last commit and dirty count via the CLI.
use super::verdict::GitState;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Mutex, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

const TIMEOUT: Duration = Duration::from_secs(4);
const WORKERS: usize = 6;

pub(super) fn repo_root(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .take(24)
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Branch name from HEAD, following a linked worktree's `gitdir:` file.
pub(super) fn branch(root: &Path) -> Option<String> {
    let dot = root.join(".git");
    let git_dir = if dot.is_file() {
        let text = fs::read_to_string(&dot).ok()?;
        let target = PathBuf::from(text.trim().strip_prefix("gitdir:")?.trim());
        if target.is_absolute() {
            target
        } else {
            root.join(target)
        }
    } else {
        dot
    };
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    Some(head.strip_prefix("ref: refs/heads/").map_or_else(
        || format!("detached {}", head.get(..8).unwrap_or(head)),
        str::to_owned,
    ))
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0");
    super::super::commands::execute_for("git", command, TIMEOUT).ok()
}

fn state(root: &Path) -> GitState {
    GitState {
        branch: branch(root),
        last_commit: git(root, &["log", "-1", "--format=%ct"])
            .and_then(|text| text.trim().parse().ok()),
        dirty: git(root, &["status", "--porcelain", "--no-renames"])
            .map(|text| text.lines().filter(|line| !line.trim().is_empty()).count()),
    }
}

/// Probe each repository once, a few at a time; each Git call has its own deadline.
pub(super) fn states(roots: &[PathBuf]) -> HashMap<PathBuf, GitState> {
    let next = AtomicUsize::new(0);
    let results = Mutex::new(HashMap::new());
    std::thread::scope(|scope| {
        for _ in 0..WORKERS.min(roots.len()) {
            scope.spawn(|| {
                while let Some(root) = roots.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let state = state(root);
                    results
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .insert(root.clone(), state);
                }
            });
        }
    });
    results.into_inner().unwrap_or_else(PoisonError::into_inner)
}
