//! Node/Bun `node_modules` analyzer.

use std::path::{Path, PathBuf};

use sweeploom_core::{Blocker, ProcessSnapshot, RebuildCost};

use crate::git::{GitSafety, inspect};
use crate::size::dir_logical_bytes;

/// One Node dependency-tree cleanup offer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeOffer {
    /// Project root.
    pub project: PathBuf,
    /// `node_modules` path.
    pub path: PathBuf,
    /// Logical bytes.
    pub logical_bytes: u64,
    /// Rebuild cost.
    pub rebuild: RebuildCost,
    /// True when auto-select is forbidden.
    pub blocked: bool,
    /// Why it is blocked, if it is.
    pub blocker: Option<Blocker>,
}

/// Discover `node_modules` offers for one project.
#[must_use]
pub fn node_offers(project: &Path, processes: &[ProcessSnapshot]) -> Vec<NodeOffer> {
    if !project.join("package.json").is_file() {
        return Vec::new();
    }
    let path = project.join("node_modules");
    if !path.is_dir() {
        return Vec::new();
    }
    let logical_bytes = dir_logical_bytes(&path);
    if logical_bytes == 0 {
        return Vec::new();
    }
    let blocker = node_blocker(project, processes);
    vec![NodeOffer {
        project: project.to_path_buf(),
        path,
        logical_bytes,
        rebuild: RebuildCost::Medium,
        blocked: blocker.is_some(),
        blocker,
    }]
}

fn node_blocker(project: &Path, processes: &[ProcessSnapshot]) -> Option<Blocker> {
    if processes
        .iter()
        .any(|process| process_blocks(process, project))
    {
        return Some(Blocker::ActiveProcess);
    }
    let git = inspect(project);
    if matches!(git, GitSafety::Unknown) {
        return Some(Blocker::UnknownGitState);
    }
    git.assessment().blockers.first().cloned()
}

fn process_blocks(process: &ProcessSnapshot, project: &Path) -> bool {
    let Some(cwd) = &process.cwd else {
        return false;
    };
    if !cwd.starts_with(project) {
        return false;
    }
    let name = process.name.to_ascii_lowercase();
    ["node", "npm", "pnpm", "yarn", "bun"]
        .iter()
        .any(|needle| name.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn missing_package_json_yields_nothing() {
        assert!(node_offers(Path::new("/no-such-sweeploom-node"), &[]).is_empty());
    }

    #[test]
    fn discovers_node_modules() {
        let root = std::env::temp_dir().join(format!("sweeploom-node-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(root.join("package.json"), "{}\n").unwrap();
        fs::write(root.join("node_modules").join("pkg.js"), vec![0_u8; 1024]).unwrap();
        let offers = node_offers(&root, &[]);
        assert_eq!(offers.len(), 1);
        assert!(offers[0].logical_bytes >= 1024);
        let _ = fs::remove_dir_all(&root);
    }
}
