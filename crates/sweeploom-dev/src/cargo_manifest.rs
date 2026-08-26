//! Read Cargo.toml and `.cargo/config.toml`. Never executes Cargo.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Package / workspace facts used by the Projects table and target resolver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoManifest {
    /// Directory that contains this `Cargo.toml`.
    pub dir: PathBuf,
    /// `[package].name`, if this is a package.
    pub package: Option<String>,
    /// True when `[lib]` exists or `src/lib.rs` is present.
    pub has_lib: bool,
    /// `[[bin]]` names, plus the default binary from `src/main.rs`.
    pub bins: Vec<String>,
    /// `[[example]]` names.
    pub examples: Vec<String>,
    /// True when this toml defines a workspace.
    pub is_workspace: bool,
    /// Workspace members, with a single `/*` glob expanded.
    pub members: Vec<PathBuf>,
}

impl CargoManifest {
    /// Short Projects/CLI label: `lib, bin foo` or `workspace · 12 crates`.
    #[must_use]
    pub fn units_label(&self) -> String {
        if self.is_workspace && self.package.is_none() {
            return format!("workspace · {} crates", self.members.len().max(1));
        }
        let mut parts = Vec::new();
        if self.has_lib {
            parts.push("lib".to_owned());
        }
        for name in &self.bins {
            parts.push(format!("bin {name}"));
        }
        for name in &self.examples {
            parts.push(format!("ex {name}"));
        }
        if self.is_workspace {
            parts.push(format!("{} members", self.members.len()));
        }
        if parts.is_empty() {
            self.package.clone().unwrap_or_else(|| "package".to_owned())
        } else {
            parts.join(", ")
        }
    }
}

/// Load `dir/Cargo.toml` when it exists.
#[must_use]
pub fn read_manifest(dir: &Path) -> Option<CargoManifest> {
    let text = fs::read_to_string(dir.join("Cargo.toml")).ok()?;
    let raw: RawManifest = toml::from_str(&text).ok()?;
    let package = raw.package.as_ref().map(|item| item.name.clone());
    let bins = bin_names(dir, package.as_deref(), &raw);
    let has_lib = raw.lib.is_some() || dir.join("src").join("lib.rs").is_file();
    let is_workspace = raw.workspace.is_some();
    let members = raw
        .workspace
        .as_ref()
        .map(|ws| expand_members(dir, &ws.members))
        .unwrap_or_default();
    Some(CargoManifest {
        dir: dir.to_path_buf(),
        package,
        has_lib,
        bins,
        examples: raw
            .example
            .into_iter()
            .flatten()
            .filter_map(|item| item.name)
            .collect(),
        is_workspace,
        members,
    })
}

/// Nearest ancestor workspace, or `dir` itself when this is a standalone package.
#[must_use]
pub fn workspace_root(dir: &Path) -> PathBuf {
    let mut current = dir.to_path_buf();
    for _ in 0..16 {
        if let Some(manifest) = read_manifest(&current)
            && manifest.is_workspace
        {
            return current;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    dir.to_path_buf()
}

/// Resolved Cargo target directory for `project` (workspace-aware).
#[must_use]
pub fn resolved_target_dir(project: &Path) -> PathBuf {
    let root = workspace_root(project);
    if let Some(custom) = env_target_dir(&root) {
        return custom;
    }
    if let Some(custom) = config_target_dir(&root) {
        return custom;
    }
    root.join("target")
}

fn env_target_dir(workspace: &Path) -> Option<PathBuf> {
    let value = std::env::var_os("CARGO_TARGET_DIR")?;
    let path = PathBuf::from(value);
    let resolved = if path.is_absolute() {
        path
    } else {
        workspace.join(path)
    };
    resolved.starts_with(workspace).then_some(resolved)
}

fn config_target_dir(workspace: &Path) -> Option<PathBuf> {
    for name in [".cargo/config.toml", ".cargo/config"] {
        let Ok(text) = fs::read_to_string(workspace.join(name)) else {
            continue;
        };
        let Ok(raw) = toml::from_str::<RawConfig>(&text) else {
            continue;
        };
        let Some(rel) = raw.build.and_then(|build| build.target_dir) else {
            continue;
        };
        let path = PathBuf::from(rel);
        let resolved = if path.is_absolute() {
            path
        } else {
            workspace.join(path)
        };
        if resolved.starts_with(workspace) {
            return Some(resolved);
        }
    }
    None
}

fn bin_names(dir: &Path, package: Option<&str>, raw: &RawManifest) -> Vec<String> {
    let mut names: Vec<String> = raw
        .bin
        .iter()
        .flatten()
        .filter_map(|item| item.name.clone())
        .collect();
    if names.is_empty()
        && dir.join("src").join("main.rs").is_file()
        && let Some(name) = package
    {
        names.push(name.to_owned());
    }
    names
}

fn expand_members(root: &Path, patterns: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for pattern in patterns {
        if let Some(prefix) = pattern.strip_suffix("/*") {
            let base = root.join(prefix);
            let Ok(entries) = fs::read_dir(&base) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("Cargo.toml").is_file() {
                    out.push(path);
                }
            }
        } else {
            let path = root.join(pattern);
            if path.join("Cargo.toml").is_file() {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[derive(Deserialize)]
struct RawManifest {
    package: Option<RawPackage>,
    lib: Option<toml::Value>,
    #[serde(default)]
    bin: Option<Vec<RawBin>>,
    #[serde(default)]
    example: Option<Vec<RawBin>>,
    workspace: Option<RawWorkspace>,
}

#[derive(Deserialize)]
struct RawPackage {
    name: String,
}

#[derive(Deserialize)]
struct RawBin {
    name: Option<String>,
}

#[derive(Default, Deserialize)]
struct RawWorkspace {
    #[serde(default)]
    members: Vec<String>,
}

#[derive(Deserialize)]
struct RawConfig {
    build: Option<RawBuild>,
}

#[derive(Deserialize)]
struct RawBuild {
    #[serde(rename = "target-dir")]
    target_dir: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("sweeploom-mf-{tag}-{}", std::process::id()))
    }

    #[test]
    fn reads_lib_and_default_bin() {
        let root = unique("pkg");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
        fs::write(root.join("src").join("lib.rs"), "").unwrap();
        fs::write(root.join("src").join("main.rs"), "fn main() {}").unwrap();
        let manifest = read_manifest(&root).expect("toml");
        fs::remove_dir_all(&root).ok();
        assert_eq!(manifest.package.as_deref(), Some("demo"));
        assert!(manifest.has_lib);
        assert_eq!(manifest.bins, ["demo"]);
        assert!(manifest.units_label().contains("lib"));
        assert!(manifest.units_label().contains("bin demo"));
    }

    #[test]
    fn workspace_root_is_the_member_parent() {
        let root = unique("ws");
        fs::create_dir_all(root.join("crates").join("api").join("src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("crates").join("api").join("Cargo.toml"),
            "[package]\nname = \"api\"\n",
        )
        .unwrap();
        fs::write(
            root.join("crates").join("api").join("src").join("lib.rs"),
            "",
        )
        .unwrap();
        let member = root.join("crates").join("api");
        let found = workspace_root(&member);
        let members = read_manifest(&root).expect("ws").members;
        fs::remove_dir_all(&root).ok();
        assert_eq!(found, root);
        assert_eq!(members.len(), 1);
    }

    #[test]
    fn config_target_dir_stays_inside_workspace() {
        let root = unique("cfg");
        fs::create_dir_all(root.join(".cargo")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
        fs::write(
            root.join(".cargo").join("config.toml"),
            "[build]\ntarget-dir = \"build-target\"\n",
        )
        .unwrap();
        let target = resolved_target_dir(&root);
        fs::remove_dir_all(&root).ok();
        assert_eq!(target, root.join("build-target"));
    }
}
