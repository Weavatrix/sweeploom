//! Path category classification. Data, not an LLM.

use std::path::Path;

/// High-level folder category for the inspector.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum PathCategory {
    /// User source.
    Source,
    /// Generated build output.
    Generated,
    /// Dependency trees.
    Dependencies,
    /// Caches.
    Cache,
    /// User data (documents, downloads).
    UserData,
    /// Unclassified.
    Unknown,
}

impl PathCategory {
    /// Short UI label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Source => "Source",
            Self::Generated => "Generated",
            Self::Dependencies => "Dependencies",
            Self::Cache => "Cache",
            Self::UserData => "User data",
            Self::Unknown => "Unknown",
        }
    }
}

/// Classify a single path component (file or directory name).
#[must_use]
pub fn classify_path_component(name: &str) -> PathCategory {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "target" | "build" | "dist" | "out" | "coverage" | ".next" | ".nuxt" | ".turbo"
        | ".vite" | ".parcel-cache" | "__pycache__" | ".pytest_cache" | ".mypy_cache"
        | ".ruff_cache" | "deriveddata" => PathCategory::Generated,
        "node_modules" | ".venv" | "venv" | "vendor" | ".gradle" | ".pnpm-store" => {
            PathCategory::Dependencies
        }
        "cache" | ".cache" | "caches" | "docker" | "overlay2" => PathCategory::Cache,
        "downloads" | "documents" | "desktop" | "pictures" => PathCategory::UserData,
        _ => PathCategory::Unknown,
    }
}

/// True when the inspector should keep nested folders under `path`.
///
/// Caches, dependency trees, and Windows package payloads stay one row so a
/// home scan cannot freeze the UI with a hundred-thousand-node tree.
#[must_use]
pub fn keep_nested_children(path: &Path) -> bool {
    // Accept both separators: a Windows path can arrive in a scan report
    // inspected on a Unix host, where `Path::file_name` treats '\\' as text.
    let text = path.to_string_lossy();
    let Some(name) = text.rsplit(['/', '\\']).find(|part| !part.is_empty()) else {
        return true;
    };
    if matches!(
        classify_path_component(name),
        PathCategory::Generated | PathCategory::Dependencies | PathCategory::Cache
    ) {
        return false;
    }
    let lower = name.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        ".git"
            | "inetcache"
            | "code cache"
            | "gpucache"
            | "shadercache"
            | "cacheddata"
            | "blob_storage"
            | "localstate"
            | "tempstate"
            | "containers"
    ) {
        return false;
    }
    text.trim_end_matches(['/', '\\'])
        .rsplit_once(['/', '\\'])
        .and_then(|(parent, _)| parent.rsplit(['/', '\\']).find(|part| !part.is_empty()))
        .is_none_or(|parent| !parent.eq_ignore_ascii_case("packages"))
}

/// True when `path` is a known project marker file.
#[must_use]
pub fn is_project_marker(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| crate::PROJECT_MARKERS.contains(&name))
}

/// True when the file looks like user source rather than generated output.
#[must_use]
pub fn is_source_extension(path: &Path) -> bool {
    const SOURCE: &[&str] = &[
        "rs", "go", "ts", "tsx", "js", "jsx", "py", "cs", "java", "kt", "swift", "cpp", "cc", "h",
        "hpp", "c", "toml", "json", "yml", "yaml", "md", "sql",
    ];
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| SOURCE.iter().any(|item| ext.eq_ignore_ascii_case(item)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn target_is_generated_and_rs_is_source() {
        assert_eq!(classify_path_component("target"), PathCategory::Generated);
        assert!(is_source_extension(Path::new("src/lib.rs")));
        assert!(is_project_marker(Path::new("/work/app/Cargo.toml")));
    }

    #[test]
    fn bulky_windows_trees_are_leaves() {
        assert!(!keep_nested_children(Path::new(
            r"C:\Users\me\AppData\Local\Docker"
        )));
        assert!(!keep_nested_children(Path::new(
            r"C:\Users\me\AppData\Local\Packages\Claude_x"
        )));
        assert!(keep_nested_children(Path::new(
            r"C:\Users\me\AppData\Local\Packages"
        )));
        assert!(keep_nested_children(Path::new(
            r"C:\Users\me\Documents\GitHub"
        )));
    }
}
