//! Project table rows. Grouping does not walk the disk.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use sweeploom_core::CandidateOwner;
use sweeploom_dev::{DevKind, classify_project};
use sweeploom_storage::InventoryReport;

use crate::app::SweepLoomApp;
use crate::format::{format_bytes, row_caption};
use crate::sort::{Col, Sort};

use super::project_facts::{
    Acc, Bit, artifact_label, folder_label, inventory_artifact_bytes, reclaimable_bytes,
};

/// How Projects cluster rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectGroup {
    /// Cargo / Node / Python.
    Kind,
    /// Immediate parent folder.
    Parent,
    /// Flat sortable list.
    None,
}

impl ProjectGroup {
    /// Toolbar label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Kind => "Kind",
            Self::Parent => "Folder",
            Self::None => "None",
        }
    }
}

pub(crate) struct ProjectCard {
    pub path: PathBuf,
    pub bytes: u64,
    pub kinds: String,
    pub group_kind: &'static str,
    pub folder: String,
    pub artifacts: String,
}

pub(crate) enum Line {
    Group { key: String, title: String },
    Project(usize),
}

pub(crate) fn collect_cards(app: &SweepLoomApp) -> Vec<ProjectCard> {
    let mut map: BTreeMap<PathBuf, Acc> = BTreeMap::new();
    for path in &app.project_roots {
        map.entry(path.clone()).or_insert_with(|| Acc::new(path));
    }
    if let Some(report) = &app.inventory {
        for path in &report.projects {
            map.entry(path.clone()).or_insert_with(|| Acc::new(path));
        }
    }
    for row in &app.review {
        let CandidateOwner::Project(id) = &row.candidate.owner else {
            continue;
        };
        let acc = map.entry(id.0.clone()).or_insert_with(|| Acc::new(&id.0));
        acc.bits.push(Bit {
            path: row.candidate.path.clone(),
            bytes: row.candidate.logical_bytes,
            title: row_caption(&row.title),
        });
    }
    map.into_values()
        .map(|acc| ProjectCard::from_acc(acc, app.inventory.as_ref()))
        .collect()
}

pub(crate) fn sort_cards(cards: &mut [ProjectCard], sort: Sort) {
    cards.sort_by(|left, right| match sort.col {
        Col::Name => left.name().cmp(right.name()),
        Col::Status => left
            .kinds
            .cmp(&right.kinds)
            .then(left.path.cmp(&right.path)),
        _ => left
            .bytes
            .cmp(&right.bytes)
            .then(left.path.cmp(&right.path)),
    });
    if sort.desc {
        cards.reverse();
    }
}

pub(crate) fn table_lines(
    cards: &[ProjectCard],
    group: ProjectGroup,
    collapsed: &HashSet<String>,
) -> Vec<Line> {
    if group == ProjectGroup::None {
        return (0..cards.len()).map(Line::Project).collect();
    }
    let mut buckets: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, card) in cards.iter().enumerate() {
        buckets
            .entry(card.group_key(group))
            .or_default()
            .push(index);
    }
    let mut keys: Vec<String> = buckets.keys().cloned().collect();
    if group == ProjectGroup::Kind {
        keys.sort_by_key(|key| kind_rank(key));
    }
    let mut lines = Vec::new();
    for key in keys {
        let indexes = buckets.remove(&key).unwrap_or_default();
        let bytes: u64 = indexes.iter().map(|&i| cards[i].bytes).sum();
        lines.push(Line::Group {
            key: key.clone(),
            title: format!("{key}  ·  {}  ·  {}", indexes.len(), format_bytes(bytes)),
        });
        if !collapsed.contains(&key) {
            lines.extend(indexes.into_iter().map(Line::Project));
        }
    }
    lines
}

impl ProjectCard {
    pub(crate) fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project")
    }

    fn group_key(&self, group: ProjectGroup) -> String {
        match group {
            ProjectGroup::Kind => self.group_kind.to_owned(),
            ProjectGroup::Parent => self.folder.clone(),
            ProjectGroup::None => String::new(),
        }
    }

    fn from_acc(acc: Acc, inventory: Option<&InventoryReport>) -> Self {
        let kinds = classify_project(&acc.path);
        let group_kind = kinds.first().copied().unwrap_or(DevKind::Other).label();
        let labels = kinds
            .iter()
            .map(|kind| kind.label())
            .collect::<Vec<_>>()
            .join(", ");
        let mut bytes = reclaimable_bytes(&acc.bits);
        if bytes == 0
            && let Some(report) = inventory
        {
            bytes = inventory_artifact_bytes(report, &acc.path);
        }
        Self {
            folder: folder_label(&acc.path),
            artifacts: artifact_label(&acc.path, &acc.bits),
            path: acc.path,
            bytes,
            kinds: labels,
            group_kind,
        }
    }
}

fn kind_rank(label: &str) -> u8 {
    match label {
        "Cargo" => 0,
        "Node" => 1,
        "Python" => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(path: &str, bytes: u64, kind: &'static str) -> ProjectCard {
        let path = PathBuf::from(path);
        ProjectCard {
            folder: folder_label(&path),
            path,
            bytes,
            kinds: kind.to_owned(),
            group_kind: kind,
            artifacts: "none".to_owned(),
        }
    }

    fn keys(lines: &[Line]) -> Vec<String> {
        lines
            .iter()
            .map(|line| match line {
                Line::Group { key, .. } => format!("g:{key}"),
                Line::Project(index) => format!("p:{index}"),
            })
            .collect()
    }

    #[test]
    fn kind_groups_hide_collapsed_projects() {
        let cards = [
            card("repos/alpha", 30, "Node"),
            card("repos/beta", 10, "Cargo"),
            card("repos/gamma", 20, "Node"),
        ];
        let open = table_lines(&cards, ProjectGroup::Kind, &HashSet::new());
        assert_eq!(keys(&open), ["g:Cargo", "p:1", "g:Node", "p:0", "p:2"]);
        let collapsed = HashSet::from(["Node".to_owned()]);
        let shut = table_lines(&cards, ProjectGroup::Kind, &collapsed);
        assert_eq!(keys(&shut), ["g:Cargo", "p:1", "g:Node"]);
    }

    #[test]
    fn folder_groups_use_the_parent_name() {
        let cards = [card("src/one", 1, "Node"), card("lib/two", 2, "Node")];
        let lines = table_lines(&cards, ProjectGroup::Parent, &HashSet::new());
        assert_eq!(keys(&lines), ["g:lib", "p:1", "g:src", "p:0"]);
    }
}
