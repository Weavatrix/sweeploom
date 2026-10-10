//! Artifact size and Cargo unit labels for the Projects table.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::format::format_bytes;

pub(super) struct Bit {
    pub path: PathBuf,
    pub bytes: u64,
    pub title: String,
}

pub(super) struct Acc {
    pub path: PathBuf,
    pub bits: Vec<Bit>,
}

impl Acc {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            bits: Vec::new(),
        }
    }
}

/// Group nested projects under their outer project's parent, including intermediary folders.
pub(super) fn cluster_parent(path: &Path, projects: &HashSet<PathBuf>) -> PathBuf {
    let outer = projects
        .iter()
        .filter(|project| path.starts_with(project))
        .min_by_key(|project| project.components().count())
        .map(PathBuf::as_path)
        .unwrap_or(path);
    outer.parent().unwrap_or(outer).to_path_buf()
}

pub(super) fn cluster_title(parent: &Path) -> String {
    parent
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("root")
        .to_owned()
}

/// `cargo` is the cached Cargo unit label from [`crate::project_sizes::ProjectFacts`].
pub(super) fn artifact_label(cargo: Option<String>, bits: &[Bit]) -> String {
    let offers = offer_summary(bits);
    match (cargo, offers.as_str()) {
        (Some(units), "none") => units,
        (Some(units), rest) => format!("{units} · {rest}"),
        (None, rest) => rest.to_owned(),
    }
}

fn offer_summary(bits: &[Bit]) -> String {
    match bits {
        [] => "none".to_owned(),
        [bit] => format!("{} · {}", bit.title, format_bytes(bit.bytes)),
        rest => format!("{} artifacts", rest.len()),
    }
}

/// Parent `target` already includes `target/debug`; do not sum both.
pub(super) fn reclaimable_bytes(bits: &[Bit]) -> u64 {
    let mut items: Vec<(&Path, u64)> = bits
        .iter()
        .map(|bit| (bit.path.as_path(), bit.bytes))
        .collect();
    items.sort_by_key(|(path, _)| path.as_os_str().len());
    let mut kept: Vec<(&Path, u64)> = Vec::new();
    for (path, bytes) in items {
        if kept.iter().any(|(parent, _)| path.starts_with(parent)) {
            continue;
        }
        kept.push((path, bytes));
    }
    kept.iter().map(|(_, bytes)| *bytes).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_target_offers_count_once() {
        let bits = [
            Bit {
                path: PathBuf::from("proj/target"),
                bytes: 10_000,
                title: "full".into(),
            },
            Bit {
                path: PathBuf::from("proj/target/debug"),
                bytes: 8_000,
                title: "debug".into(),
            },
        ];
        assert_eq!(reclaimable_bytes(&bits), 10_000);
    }

    #[test]
    fn nested_project_clusters_under_github() {
        let github = PathBuf::from("Documents/GitHub");
        let frontend = github.join("frontend");
        let e2e = frontend.join("test-e2e");
        let projects = HashSet::from([frontend.clone(), e2e.clone()]);
        assert_eq!(cluster_parent(&frontend, &projects), github);
        assert_eq!(cluster_parent(&e2e, &projects), github);
        assert_eq!(cluster_title(&github), "GitHub");
    }

    #[test]
    fn nested_apps_and_workers_share_the_outer_project_group() {
        let repo = PathBuf::from("dev/profi");
        let app = repo.join("apps/web");
        let worker = repo.join("workers/triage");
        let projects = HashSet::from([repo.clone(), app.clone(), worker.clone()]);
        assert_eq!(cluster_parent(&repo, &projects), PathBuf::from("dev"));
        assert_eq!(cluster_parent(&app, &projects), PathBuf::from("dev"));
        assert_eq!(cluster_parent(&worker, &projects), PathBuf::from("dev"));
    }
}
