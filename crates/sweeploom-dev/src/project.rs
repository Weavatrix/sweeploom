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
    /// Go module.
    Go,
    /// Other marker.
    Other,
}

impl DevKind {
    /// Short UI label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cargo => "Cargo",
            Self::Node => "Node",
            Self::Python => "Python",
            Self::Go => "Go",
            Self::Other => "Other",
        }
    }
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
    if root.join("go.mod").is_file() {
        kinds.push(DevKind::Go);
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

    #[test]
    fn go_mod_is_go() {
        let root = std::env::temp_dir().join(format!("sweeploom-go-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("go.mod"), "module demo\n").unwrap();
        assert_eq!(classify_project(&root), [DevKind::Go]);
        let _ = std::fs::remove_dir_all(&root);
    }
}
