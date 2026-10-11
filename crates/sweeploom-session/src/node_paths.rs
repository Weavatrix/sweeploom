//! Path-only Node evidence: manager versions and tool installation roots.

use std::path::Path;

/// A Node manager version in the actual executable path, when explicit.
#[must_use]
pub fn version_from_path(path: &Path) -> Option<String> {
    let text = path.to_string_lossy().replace('\\', "/");
    // Date/build identifiers in Cursor's bundle are not Node versions.
    if text.contains("/cursor-agent/versions/") {
        return None;
    }
    text.split('/').rev().find_map(|component| {
        let value = component
            .strip_prefix('v')
            .or_else(|| component.strip_prefix("node-v"))?;
        let numbers: Vec<_> = value.split('.').collect();
        (numbers.len() == 3
            && numbers
                .iter()
                .all(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())))
        .then(|| format!("v{value}"))
    })
}

/// Installed tool directories are origins, not user project roots.
#[must_use]
pub fn is_tool_installation(path: &Path) -> bool {
    let text = path.to_string_lossy().replace('\\', "/");
    if text
        .split('/')
        .any(|part| matches!(part, ".nvm" | ".fnm" | "node_modules" | ".npm"))
    {
        return true;
    }
    [
        "/plugins/cache/",
        "/cursor-agent/versions/",
        "/Applications/",
    ]
    .iter()
    .any(|part| text.contains(part))
}
