//! Project kind from marker files.

use std::path::Path;

/// Kind of developer project discovered from markers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevKind {
    /// Cargo workspace or package.
    Cargo,
    /// Node / Bun package.
    Node,
    /// Python project.
    Python,
    /// Other marker.
    Other,
}

/// Classify a project directory from its marker files.
#[must_use]
pub fn classify_project(root: &Path) -> Vec<DevKind> {
    let mut kinds = Vec::new();
    if root.join("Cargo.toml").is_file() {
        kinds.push(DevKind::Cargo);
    }
    if root.join("package.json").is_file() {
        kinds.push(DevKind::Node);
    }
    if root.join("pyproject.toml").is_file() || root.join("requirements.txt").is_file() {
        kinds.push(DevKind::Python);
    }
    if kinds.is_empty() {
        kinds.push(DevKind::Other);
    }
    kinds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_markers_are_other() {
        assert_eq!(
            classify_project(Path::new("/definitely-missing-sweeploom")),
            [DevKind::Other]
        );
    }
}
