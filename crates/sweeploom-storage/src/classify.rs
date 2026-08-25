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
        "cache" | ".cache" | "caches" => PathCategory::Cache,
        "downloads" | "documents" | "desktop" | "pictures" => PathCategory::UserData,
        _ => PathCategory::Unknown,
    }
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
}
