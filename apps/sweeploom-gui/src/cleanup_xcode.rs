//! Xcode projects on this Mac: DerivedData records, bundle identifiers and Git state.
#[path = "cleanup_xcode_caches.rs"]
mod caches;
#[path = "cleanup_xcode_git.rs"]
mod git;
#[path = "cleanup_xcode_hash.rs"]
mod hash;
#[path = "cleanup_xcode_plist.rs"]
pub(super) mod plist;
#[path = "cleanup_xcode_verdict.rs"]
pub(super) mod verdict;
pub(super) use caches::enrich;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
use verdict::{GitState, Project, Verdict};

/// Common checkout roots, scanned shallowly for `*.xcodeproj` and XcodeGen `project.yml`.
const ROOTS: &[&str] = &[
    "dev",
    "Developer",
    "Projects",
    "code",
    "src",
    "repos",
    "workspace",
    "Documents/GitHub",
    "GitHub",
    ".codex/worktrees",
];
const SKIP: &[&str] = &[
    "node_modules",
    "Pods",
    "build",
    "DerivedData",
    "target",
    "dist",
    "vendor",
    "Carthage",
    "SourcePackages",
];
const MAX_DEPTH: usize = 5;
const MAX_ENTRIES: usize = 30_000;

pub(super) fn derived_data(home: &Path) -> PathBuf {
    home.join("Library/Developer/Xcode/DerivedData")
}

/// `WorkspacePath` recorded by Xcode for one DerivedData folder.
pub(super) fn workspace_of(folder: &Path) -> Option<PathBuf> {
    plist::read(&folder.join("info.plist"), "WorkspacePath").map(PathBuf::from)
}

pub(super) fn recorded_workspaces(home: &Path) -> Vec<PathBuf> {
    fs::read_dir(derived_data(home))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| workspace_of(&entry.path()))
        .collect()
}

pub(super) struct Projects {
    now: i64,
    git: HashMap<PathBuf, GitState>,
    bundles: HashMap<String, Vec<PathBuf>>,
    /// DerivedData folder suffix → checkout that builds into it.
    folders: HashMap<String, PathBuf>,
}

impl Projects {
    pub(super) fn load(home: &Path, workspaces: &[PathBuf]) -> Self {
        let mut paths = scan(home);
        for workspace in workspaces.iter().filter(|path| path.exists()) {
            if workspace.extension().is_some_and(|ext| ext == "xcodeproj") {
                paths.push(workspace.clone());
            } else if let Some(parent) = workspace.parent() {
                paths.extend(siblings(parent));
            }
        }
        let mut seen = HashSet::new();
        paths.retain(|path| seen.insert(path.clone()));
        let mut bundles: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for path in &paths {
            for id in bundle_ids(path) {
                bundles.entry(id).or_default().push(path.clone());
            }
        }
        let mut roots: Vec<_> = paths
            .iter()
            .chain(workspaces.iter().filter(|path| path.exists()))
            .filter_map(|path| git::repo_root(path))
            .collect();
        roots.sort();
        roots.dedup();
        let folders = paths
            .iter()
            .filter(|path| path.is_dir())
            .map(|path| {
                (
                    hash::derived_suffix(&path.display().to_string()),
                    path.clone(),
                )
            })
            .collect();
        Self {
            now: verdict::now(),
            git: git::states(&roots),
            bundles,
            folders,
        }
    }

    pub(super) fn project(&self, path: &Path) -> Project {
        if !path.exists() {
            return Project::Missing;
        }
        match git::repo_root(path).and_then(|root| self.git.get(&root)) {
            Some(state) => Project::Git(state.clone()),
            None => Project::Plain {
                modified: modified(&path.join("project.pbxproj")).or_else(|| modified(path)),
            },
        }
    }

    /// Checkout whose path hashes to this DerivedData suffix (folders without info.plist).
    pub(super) fn checkout(&self, suffix: &str) -> Option<&Path> {
        self.folders.get(suffix).map(PathBuf::as_path)
    }

    pub(super) fn verdict(&self, path: &Path) -> (Verdict, String) {
        verdict::project(&self.project(path), self.now)
    }

    /// Most protective verdict among checkouts that build this bundle identifier.
    pub(super) fn app(&self, bundle: &str) -> Option<(String, Verdict, String)> {
        self.bundles
            .get(bundle)?
            .iter()
            .map(|path| {
                let (verdict, detail) = self.verdict(path);
                (project_name(path), verdict, detail)
            })
            .min_by_key(|(_, verdict, _)| *verdict)
    }
}

pub(super) fn project_name(path: &Path) -> String {
    let path = if path.file_name().is_some_and(|name| name == "project.yml") {
        let named = fs::read_to_string(path).ok().and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("name:"))
                .map(|name| name.trim().trim_matches(['"', '\'']).to_owned())
        });
        if let Some(name) = named.filter(|name| !name.is_empty()) {
            return name;
        }
        path.parent().unwrap_or(path)
    } else {
        path
    };
    path.file_stem().map_or_else(
        || path.display().to_string(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

pub(super) fn modified(path: &Path) -> Option<i64> {
    let at = fs::symlink_metadata(path).ok()?.modified().ok()?;
    let secs = at.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    i64::try_from(secs).ok()
}

fn siblings(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "xcodeproj"))
        .collect()
}

fn scan(home: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut budget = MAX_ENTRIES;
    let mut stack: Vec<(PathBuf, usize)> = ROOTS.iter().map(|root| (home.join(root), 0)).collect();
    while let Some((dir, depth)) = stack.pop() {
        for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
            if budget == 0 {
                return found;
            }
            budget -= 1;
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            let bundle = name.ends_with(".xcodeproj") || name.ends_with(".xcworkspace");
            if (bundle && kind.is_dir()) || (name == "project.yml" && kind.is_file()) {
                found.push(entry.path());
            } else if kind.is_dir()
                && depth < MAX_DEPTH
                && !name.starts_with('.')
                && !SKIP.contains(&name.as_str())
                && !name.ends_with(".app")
            {
                stack.push((entry.path(), depth + 1));
            }
        }
    }
    found
}

/// Literal `PRODUCT_BUNDLE_IDENTIFIER` values from a project.pbxproj or project.yml.
pub(super) fn bundle_ids(project: &Path) -> Vec<String> {
    let file = if project.is_dir() {
        project.join("project.pbxproj")
    } else {
        project.to_owned()
    };
    let Ok(meta) = fs::metadata(&file) else {
        return Vec::new();
    };
    if meta.len() > 16 << 20 {
        return Vec::new();
    }
    let mut ids: Vec<String> = fs::read_to_string(&file)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let (_, value) = line.trim().split_once("PRODUCT_BUNDLE_IDENTIFIER")?;
            let value = value.trim_start().strip_prefix(['=', ':'])?;
            let value = value
                .trim()
                .trim_end_matches(';')
                .trim()
                .trim_matches(['"', '\'']);
            (!value.is_empty() && !value.contains('$')).then(|| value.to_owned())
        })
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

#[cfg(test)]
#[path = "cleanup_xcode_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "cleanup_xcode_verdict_tests.rs"]
mod verdict_tests;
