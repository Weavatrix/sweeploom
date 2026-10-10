//! Where developer projects live, and which folders discovery never enters.

use std::fs;
use std::path::{Path, PathBuf};

use crate::classify::{PathCategory, classify_path_component, is_project_marker_name};

/// Home folders (matched case-insensitively) that usually hold repositories.
/// Any `*Projects` folder (IdeaProjects, PycharmProjects, ...) also counts.
const ROOT_NAMES: &[&str] = &[
    "dev",
    "developer",
    "development",
    "projects",
    "code",
    "src",
    "source",
    "repos",
    "repositories",
    "work",
    "workspace",
    "workspaces",
    "github",
    "gitlab",
    "git",
    "sites",
    "desktop",
];

const NESTED_ROOTS: &[&str] = &[
    "Documents/GitHub",
    "Documents/Projects",
    "Documents/Code",
    "Documents/Developer",
    "source/repos",
    "go/src",
];

/// Bundles are opaque: an `.xcodeproj` marks its parent, it is never a tree to walk.
const BUNDLES: &[&str] = &[
    "xcodeproj",
    "xcworkspace",
    "playground",
    "app",
    "appex",
    "framework",
    "xcframework",
    "bundle",
    "xcassets",
    "lproj",
    "photoslibrary",
];

/// Roots under home where developer projects usually live.
///
/// Prefers `Documents/GitHub` over walking all of `Documents`. Loose projects
/// directly under home (`~/rust_test`) become roots of their own.
#[must_use]
pub fn developer_roots(home: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for (name, path) in home_children(home) {
        let lower = name.to_ascii_lowercase();
        let named = ROOT_NAMES.contains(&lower.as_str()) || lower.ends_with("projects");
        if named || (!is_skip_name(&name) && !is_user_data(&lower) && has_marker(&path)) {
            roots.push(path);
        }
    }
    for rel in NESTED_ROOTS {
        let path = home.join(rel);
        if path.is_dir() && !roots.contains(&path) {
            roots.push(path);
        }
    }
    let documents = home.join("Documents");
    if documents.is_dir() && !documents.join("GitHub").is_dir() {
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

/// True when discovery starting at `root` may enter every folder down to `path`:
/// nothing hidden, generated, dependency, cache or OS noise in between.
#[must_use]
pub fn is_discoverable_below(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    relative
        .components()
        .all(|part| part.as_os_str().to_str().is_none_or(|name| !is_skip_name(name)))
}

/// Folder names discovery never descends into.
pub(super) fn is_skip_name(name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }
    let lower = name.to_ascii_lowercase();
    matches!(
        classify_path_component(&lower),
        PathCategory::Generated | PathCategory::Dependencies | PathCategory::Cache
    ) || is_home_noise(&lower)
        || matches!(
            lower.as_str(),
            "pods"
                | "carthage"
                | "sourcepackages"
                | "bower_components"
                | "site-packages"
                | "jspm_packages"
                | "elm-stuff"
                | "zig-cache"
                | "zig-out"
        )
        || lower
            .rsplit_once('.')
            .is_some_and(|(_, ext)| BUNDLES.contains(&ext))
}

fn is_home_noise(lower: &str) -> bool {
    matches!(
        lower,
        "appdata"
            | "application data"
            | "library"
            | "temp"
            | "tmp"
            | "windows"
            | "$recycle.bin"
            | "system volume information"
    )
}

fn is_user_data(lower: &str) -> bool {
    matches!(
        lower,
        "documents" | "downloads" | "pictures" | "movies" | "music" | "public" | "applications"
    )
}

fn home_children(home: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(home) else {
        return Vec::new();
    };
    let mut children: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_owned();
            let path = entry.path();
            (!name.starts_with('.') && path.is_dir()).then_some((name, path))
        })
        .collect();
    children.sort();
    children
}

fn has_marker(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|entry| entry.file_name().to_str().is_some_and(is_project_marker_name))
    })
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
