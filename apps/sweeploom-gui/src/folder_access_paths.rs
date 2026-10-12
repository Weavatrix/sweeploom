//! Folders the App Store edition suggests granting, and how they are labelled.

use std::path::{Path, PathBuf};

use super::allowed;

/// Developer checkouts offered on first run.
pub const DEVELOPER: &[&str] = &[
    "dev",
    "Developer",
    "Projects",
    "code",
    "src",
    "repos",
    "workspace",
    "Documents/GitHub",
    "GitHub",
];
/// Xcode DerivedData, DeviceSupport and Archives live here.
pub const XCODE: &[&str] = &["Library/Developer"];
/// Package and build caches.
pub const CACHES: &[&str] = &[".cargo", ".npm", "Library/Caches", ".gradle", ".cache"];
/// Local AI tool stores.
pub const AI: &[&str] = &[
    ".claude",
    ".codex",
    ".cursor",
    ".opencode",
    ".gemini",
    ".grok",
    ".weavatrix",
    ".claude-server-commander",
];

/// Existing folders under `home` that SweepLoom cannot reach yet.
#[must_use]
pub fn missing(home: &Path, relatives: &[&str]) -> Vec<PathBuf> {
    relatives
        .iter()
        .map(|relative| home.join(relative))
        .filter(|path| path.is_dir() && !allowed(path))
        .collect()
}

/// Folder to grant for a catalog location, e.g. `~/.npm` for `.npm/_cacache`
/// and `~/Library/Caches` for `Library/Caches/pip`.
#[must_use]
pub fn grant_root(home: &Path, relative: &str) -> PathBuf {
    let parts: Vec<&str> = relative
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    let keep = match parts.as_slice() {
        ["Library", "Application Support", ..] => 3,
        ["Library", ..] => 2,
        _ => 1,
    };
    parts
        .iter()
        .take(keep)
        .fold(home.to_path_buf(), |path, part| path.join(part))
}

/// `~/dev` style label for a folder under home.
#[must_use]
pub fn short(home: &Path, path: &Path) -> String {
    path.strip_prefix(home).map_or_else(
        |_| path.display().to_string(),
        |rest| format!("~/{}", rest.display()),
    )
}
