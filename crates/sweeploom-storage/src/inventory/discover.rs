//! Find project markers without walking generated trees.

use std::fs;
use std::path::{Path, PathBuf};

use crate::PROJECT_MARKERS;
use crate::classify::{PathCategory, classify_path_component};

const MAX_DIRS: usize = 12_000;

/// Discover project roots under `root`, skipping `target` / `node_modules` / AppData.
#[must_use]
pub fn discover_projects(root: &Path, max_projects: usize) -> Vec<PathBuf> {
    discover_projects_from(&[root.to_path_buf()], max_projects)
}

/// Discover across several roots. Duplicates are dropped.
#[must_use]
pub fn discover_projects_from(roots: &[PathBuf], max_projects: usize) -> Vec<PathBuf> {
    let mut projects = Vec::new();
    let mut seen = 0_usize;
    for root in roots {
        let root = fs::canonicalize(root).unwrap_or_else(|_| root.clone());
        if !root.is_dir() {
            continue;
        }
        walk(&root, &root, 0, max_projects, &mut projects, &mut seen);
        if projects.len() >= max_projects {
            break;
        }
    }
    projects
}

fn walk(
    scan_root: &Path,
    dir: &Path,
    depth: usize,
    max_projects: usize,
    projects: &mut Vec<PathBuf>,
    seen: &mut usize,
) {
    if depth > 12 || projects.len() >= max_projects || *seen >= MAX_DIRS {
        return;
    }
    *seen += 1;
    if dir != scan_root && is_skip_dir(dir) {
        return;
    }
    if has_marker(dir) && !projects.iter().any(|item| item == dir) {
        projects.push(dir.to_path_buf());
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if projects.len() >= max_projects || *seen >= MAX_DIRS {
            return;
        }
        let path = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() || !meta.is_dir() {
            continue;
        }
        if is_skip_dir(&path) {
            continue;
        }
        walk(scan_root, &path, depth + 1, max_projects, projects, seen);
    }
}

fn has_marker(dir: &Path) -> bool {
    PROJECT_MARKERS
        .iter()
        .any(|marker| dir.join(marker).is_file())
}

fn is_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if name.eq_ignore_ascii_case(".git") {
        return true;
    }
    matches!(
        classify_path_component(name),
        PathCategory::Generated | PathCategory::Dependencies | PathCategory::Cache
    ) || is_home_noise(name)
}

fn is_home_noise(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "appdata"
            | "application data"
            | "library"
            | ".cargo"
            | ".rustup"
            | ".npm"
            | ".local"
            | "temp"
            | "tmp"
            | "windows"
            | "$recycle.bin"
            | "system volume information"
    )
}

/// Extra roots under home where developer projects usually live.
///
/// Prefer `Documents/GitHub` over walking all of `Documents`.
#[must_use]
pub fn developer_roots(home: &Path) -> Vec<PathBuf> {
    let specific = [
        "Documents/GitHub",
        "Desktop",
        "src",
        "dev",
        "code",
        "projects",
        "GitHub",
    ];
    let mut roots: Vec<PathBuf> = specific
        .into_iter()
        .map(|rel| home.join(rel))
        .filter(|path| path.is_dir())
        .collect();
    let documents = home.join("Documents");
    let github_docs = documents.join("GitHub");
    if documents.is_dir() && !github_docs.is_dir() {
        roots.push(documents);
    }
    roots
}

/// Project-discovery roots for a Review scan.
///
/// A home scan uses [`developer_roots`] instead of walking AppData / Library.
#[must_use]
pub fn review_scan_roots(scan_root: &Path, home: &Path) -> Vec<PathBuf> {
    if same_dir(scan_root, home) {
        let mut roots = developer_roots(home);
        if roots.is_empty() {
            roots.push(home.to_path_buf());
        }
        return roots;
    }
    vec![scan_root.to_path_buf()]
}

fn same_dir(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
