//! Artifact size and Cargo unit labels for the Projects table.

use std::path::{Path, PathBuf};

use sweeploom_dev::{read_manifest, workspace_root};
use sweeploom_storage::{InventoryReport, PathCategory};

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

pub(super) fn folder_label(path: &Path) -> String {
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("root")
        .to_owned()
}

pub(super) fn artifact_label(path: &Path, bits: &[Bit]) -> String {
    let cargo = read_manifest(path).map(|manifest| {
        let units = manifest.units_label();
        if workspace_root(path) != path {
            format!("{units} · shared target")
        } else {
            units
        }
    });
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

pub(super) fn inventory_artifact_bytes(report: &InventoryReport, project: &Path) -> u64 {
    let Some(node) = report.node(project) else {
        return 0;
    };
    node.children
        .iter()
        .filter(|child| {
            matches!(
                child.category,
                PathCategory::Generated | PathCategory::Dependencies | PathCategory::Cache
            )
        })
        .map(|child| child.logical_bytes)
        .sum()
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
}
