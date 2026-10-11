//! Node/Bun `node_modules` analyzer.

use std::path::{Path, PathBuf};

use sweeploom_core::{Blocker, ProcessSnapshot, RebuildCost};

use crate::git::{GitSafety, blocker_for, inspect};
use crate::size::dir_size;

/// One Node dependency-tree or Vite cache cleanup offer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeOffer {
    /// Project root.
    pub project: PathBuf,
    /// Path that would be deleted.
    pub path: PathBuf,
    /// `node_modules`, `.vite`, or `node_modules/.vite`.
    pub label: &'static str,
    /// Logical bytes.
    pub logical_bytes: u64,
    /// False when a cap or read error made this a lower bound.
    pub size_complete: bool,
    /// Rebuild cost.
    pub rebuild: RebuildCost,
    /// True when auto-select is forbidden.
    pub blocked: bool,
    /// Why it is blocked, if it is.
    pub blocker: Option<Blocker>,
    /// Pre-select SAFE Vite cache. Never pre-select the whole `node_modules` tree.
    pub preselect: bool,
}

/// Discover `node_modules` and Vite cache offers (`rules/common/vite-cache.toml`).
#[must_use]
pub fn node_offers(project: &Path, processes: &[ProcessSnapshot]) -> Vec<NodeOffer> {
    node_offers_with(project, processes, &inspect)
}

/// [`node_offers`] with a shared Git reader. Git is read only when an offer exists.
pub(crate) fn node_offers_with(
    project: &Path,
    processes: &[ProcessSnapshot],
    git: &dyn Fn(&Path) -> GitSafety,
) -> Vec<NodeOffer> {
    if !project.join("package.json").is_file() {
        return Vec::new();
    }
    let mut offers = Vec::new();
    push_offer(
        &mut offers,
        project,
        "node_modules",
        RebuildCost::Medium,
        false,
    );
    for rel in ["node_modules/.vite", ".vite"] {
        push_offer(&mut offers, project, rel, RebuildCost::Low, true);
    }
    if !offers.is_empty() {
        let tools = ["node", "npm", "pnpm", "yarn", "bun"];
        let blocker = blocker_for(project, processes, &tools, git);
        for offer in &mut offers {
            offer.blocked = blocker.is_some();
            offer.blocker = blocker;
            offer.preselect &= blocker.is_none();
        }
    }
    offers
}

fn push_offer(
    offers: &mut Vec<NodeOffer>,
    project: &Path,
    rel: &'static str,
    rebuild: RebuildCost,
    preselect: bool,
) {
    let path = project.join(rel);
    if !path.is_dir() {
        return;
    }
    let size = dir_size(&path);
    let logical_bytes = size.bytes;
    if logical_bytes == 0 {
        return;
    }
    offers.push(NodeOffer {
        project: project.to_path_buf(),
        path,
        label: rel,
        logical_bytes,
        size_complete: size.complete,
        rebuild,
        blocked: false,
        blocker: None,
        preselect,
    });
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
        assert_eq!(offers[0].label, "node_modules");
        assert!(!offers[0].preselect);
        assert!(offers[0].logical_bytes >= 1024);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn discovers_vite_caches_from_shipped_rule() {
        let root = std::env::temp_dir().join(format!(
            "sweeploom-vite-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|item| item.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("node_modules").join(".vite")).unwrap();
        fs::create_dir_all(root.join(".vite")).unwrap();
        fs::write(root.join("package.json"), "{}\n").unwrap();
        fs::write(
            root.join("node_modules").join(".vite").join("dep"),
            vec![0_u8; 256],
        )
        .unwrap();
        fs::write(root.join(".vite").join("chunk"), vec![0_u8; 128]).unwrap();
        let offers = node_offers(&root, &[]);
        let labels: Vec<_> = offers.iter().map(|item| item.label).collect();
        assert!(labels.contains(&"node_modules/.vite"), "{labels:?}");
        assert!(labels.contains(&".vite"), "{labels:?}");
        let vite = offers
            .iter()
            .find(|item| item.label == ".vite")
            .expect(".vite");
        assert!(vite.preselect);
        assert_eq!(vite.rebuild, RebuildCost::Low);
        let _ = fs::remove_dir_all(&root);
    }
}
