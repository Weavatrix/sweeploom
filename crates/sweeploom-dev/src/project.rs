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
    /// Xcode project or workspace.
    Xcode,
    /// Swift package.
    Swift,
    /// Gradle or Maven build.
    Jvm,
    /// .NET solution or project.
    Dotnet,
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
            Self::Xcode => "Xcode",
            Self::Swift => "Swift",
            Self::Jvm => "JVM",
            Self::Dotnet => ".NET",
            Self::Other => "Other",
        }
    }

    fn of_marker(name: &str) -> Option<Self> {
        let ext = name
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_ascii_lowercase());
        Some(match name {
            "Cargo.toml" => Self::Cargo,
            "package.json" => Self::Node,
            "go.mod" => Self::Go,
            "pyproject.toml" | "requirements.txt" | "setup.py" | "Pipfile" => Self::Python,
            "Package.swift" => Self::Swift,
            "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
            | "settings.gradle"
            | "settings.gradle.kts" => Self::Jvm,
            _ => match ext.as_deref() {
                Some("xcodeproj" | "xcworkspace") => Self::Xcode,
                Some("sln" | "csproj" | "fsproj" | "vbproj") => Self::Dotnet,
                _ => return None,
            },
        })
    }
}

/// Classify a project directory from its marker files, in [`DevKind`] order.
/// One directory read; nothing below the project is visited.
#[must_use]
pub fn classify_project(root: &Path) -> Vec<DevKind> {
    let mut kinds: Vec<DevKind> = std::fs::read_dir(root)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| entry.file_name().to_str().and_then(DevKind::of_marker))
                .collect()
        })
        .unwrap_or_default();
    kinds.sort_by_key(|kind| *kind as u8);
    kinds.dedup();
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

    #[test]
    fn xcode_swift_and_cargo_markers_are_ordered() {
        let root = std::env::temp_dir().join(format!("sweeploom-kinds-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("App.xcodeproj")).unwrap();
        std::fs::write(root.join("Package.swift"), "// swift-tools-version:5.9\n").unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();
        assert_eq!(
            classify_project(&root),
            [DevKind::Cargo, DevKind::Xcode, DevKind::Swift]
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
