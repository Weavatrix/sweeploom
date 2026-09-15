//! Generated-artifact authorization. Discovery is a hint, not a delete grant.

use std::path::{Component, Path, PathBuf};

/// Why a path must not become a destructive generated-cleanup target.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum ArtifactRefusal {
    /// Path is the project, workspace, home, or `.`.
    ProtectedRoot,
    /// Path escapes the owning workspace after normalization.
    EscapesOwner,
    /// Root is a symlink or reparse point.
    SymlinkRoot,
    /// Path is or contains source, `.git`, or other protected layout.
    ProtectedContent,
    /// Contents look mixed or unknown; refuse rather than guess.
    MixedOrUnknown,
}

impl ArtifactRefusal {
    /// Short UI label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ProtectedRoot => "protected root",
            Self::EscapesOwner => "outside workspace",
            Self::SymlinkRoot => "symlink root",
            Self::ProtectedContent => "protected content",
            Self::MixedOrUnknown => "unknown layout",
        }
    }
}

/// Native identity captured at plan time.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileIdentity {
    /// Volume / device id when the platform exposes one.
    pub volume: u64,
    /// File index / inode when the platform exposes one.
    pub file_id: u64,
    /// True when the path was a directory.
    pub is_dir: bool,
}

/// Bounded metadata used as a revision, not a cryptographic proof.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MetadataRevision {
    /// Files visited while measuring.
    pub file_count: u64,
    /// Logical bytes visited.
    pub logical_bytes: u64,
    /// Newest mtime as unix milliseconds, if known.
    pub max_mtime_unix_ms: Option<u64>,
    /// False when the walk stopped early or hit errors.
    pub complete: bool,
}

/// How much of a tree was actually measured.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Coverage {
    /// Walk finished without a cap or hard error.
    pub complete: bool,
    /// Errors encountered while reading.
    pub errors: u64,
}

/// Proof recorded by the validator, not filled in by an analyzer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactProof {
    /// Owning project or workspace.
    pub owner: PathBuf,
    /// Why this path was accepted as generated output.
    pub reason: &'static str,
}

/// A path the validator allowed as generated output.
///
/// Analyzers must not construct this by hand. Call [`authorize_generated`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovedArtifact {
    /// Owning project or workspace.
    pub owner: PathBuf,
    /// Normalized root that may be deleted.
    pub canonical_root: PathBuf,
    /// Validator proof.
    pub proof: ArtifactProof,
}

/// Directory names that must never be treated as generated output.
const PROTECTED_NAMES: &[&str] = &[
    "src", "source", "lib", "crates", "apps", "tests", "examples", "benches", "include", "docs",
    ".git", ".github", ".hg", ".svn",
];

/// Well-known generated directory names that may be cleaned after other checks.
const GENERATED_NAMES: &[&str] = &[
    "target",
    "incremental",
    "debug",
    "release",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
    ".next",
    ".turbo",
    ".vite",
];

/// Authorize `candidate` as generated output owned by `owner`.
///
/// `home` is refused even when it happens to sit inside `owner`.
pub fn authorize_generated(
    owner: &Path,
    candidate: &Path,
    home: Option<&Path>,
) -> Result<ApprovedArtifact, ArtifactRefusal> {
    let owner_n = normalize_path(owner);
    let candidate_n = normalize_path(candidate);
    if candidate_n.as_os_str().is_empty() || candidate_n == Path::new(".") || candidate_n == owner_n
    {
        return Err(ArtifactRefusal::ProtectedRoot);
    }
    if let Some(home) = home {
        let home_n = normalize_path(home);
        if candidate_n == home_n {
            return Err(ArtifactRefusal::ProtectedRoot);
        }
    }
    if !is_strict_child(&candidate_n, &owner_n) {
        return Err(ArtifactRefusal::EscapesOwner);
    }
    if is_symlink_root(&candidate_n) {
        return Err(ArtifactRefusal::SymlinkRoot);
    }
    if has_protected_component(&candidate_n) || contains_git_metadata(&candidate_n) {
        return Err(ArtifactRefusal::ProtectedContent);
    }
    if looks_like_source_tree(&candidate_n) {
        return Err(ArtifactRefusal::ProtectedContent);
    }
    let name = file_name_lower(&candidate_n);
    let under_generated = ancestor_has_generated_name(&candidate_n, &owner_n);
    let reason = if GENERATED_NAMES.contains(&name.as_str()) || under_generated {
        "generated-name-inside-owner"
    } else {
        "validated-custom-target"
    };
    Ok(ApprovedArtifact {
        owner: owner_n.clone(),
        canonical_root: candidate_n.clone(),
        proof: ArtifactProof {
            owner: owner_n,
            reason,
        },
    })
}

/// Collapse `.` and `..` without touching the filesystem.
#[must_use]
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn is_strict_child(child: &Path, parent: &Path) -> bool {
    let child_c: Vec<_> = child.components().collect();
    let parent_c: Vec<_> = parent.components().collect();
    child_c.starts_with(&parent_c) && child_c.len() > parent_c.len()
}

fn is_symlink_root(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

fn file_name_lower(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn has_protected_component(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component
            .as_os_str()
            .to_str()
            .unwrap_or("")
            .to_ascii_lowercase();
        PROTECTED_NAMES.contains(&name.as_str())
    })
}

fn ancestor_has_generated_name(path: &Path, owner: &Path) -> bool {
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir == owner {
            break;
        }
        if GENERATED_NAMES.contains(&file_name_lower(dir).as_str()) {
            return true;
        }
        current = dir.parent();
    }
    false
}

fn contains_git_metadata(path: &Path) -> bool {
    path.join(".git").exists()
}

fn looks_like_source_tree(path: &Path) -> bool {
    path.join("Cargo.toml").is_file()
        || path.join("package.json").is_file()
        || path.join("pyproject.toml").is_file()
        || path.join("src").is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "sweeploom-art-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|item| item.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn refuses_src_root_dot_and_escape() {
        let owner = PathBuf::from(r"C:\work\demo");
        let home = PathBuf::from(r"C:\Users\me");
        assert_eq!(
            authorize_generated(&owner, &owner.join("src"), Some(&home)).unwrap_err(),
            ArtifactRefusal::ProtectedContent
        );
        assert_eq!(
            authorize_generated(&owner, &owner, Some(&home)).unwrap_err(),
            ArtifactRefusal::ProtectedRoot
        );
        assert_eq!(
            authorize_generated(&owner, Path::new("."), Some(&home)).unwrap_err(),
            ArtifactRefusal::ProtectedRoot
        );
        let escaped = owner.join("..").join("outside");
        assert_eq!(
            authorize_generated(&owner, &escaped, Some(&home)).unwrap_err(),
            ArtifactRefusal::EscapesOwner
        );
    }

    #[test]
    fn accepts_standard_target() {
        let owner = unique("ok");
        fs::create_dir_all(owner.join("target").join("debug")).unwrap();
        let approved = authorize_generated(&owner, &owner.join("target"), None).unwrap();
        assert!(approved.canonical_root.ends_with("target"));
        let _ = fs::remove_dir_all(&owner);
    }

    #[test]
    fn refuses_nested_git() {
        let owner = unique("git");
        let target = owner.join("target");
        fs::create_dir_all(target.join(".git")).unwrap();
        assert_eq!(
            authorize_generated(&owner, &target, None).unwrap_err(),
            ArtifactRefusal::ProtectedContent
        );
        let _ = fs::remove_dir_all(&owner);
    }
}
