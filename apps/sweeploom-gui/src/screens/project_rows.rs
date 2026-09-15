//! Project table rows. Grouping does not walk the disk.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use sweeploom_core::CandidateOwner;
use sweeploom_dev::{DevKind, classify_project};
use sweeploom_storage::InventoryReport;

use crate::app::SweepLoomApp;
use crate::format::row_caption;
use crate::sort::{Col, Sort};

use super::project_facts::{
    Acc, Bit, artifact_label, cluster_parent, cluster_title, inventory_artifact_bytes,
    reclaimable_bytes,
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

#[derive(Clone)]
pub(crate) struct ProjectCard {
    pub path: PathBuf,
    pub bytes: u64,
    pub kinds: String,
    pub group_kind: &'static str,
    pub folder: String,
    pub folder_key: String,
    pub cluster_bytes: u64,
    pub artifacts: String,
}

pub(crate) enum Line {
    Group {
        key: String,
        title: String,
        count: usize,
        bytes: u64,
        expanded: bool,
    },
    Project(usize),
}

pub(crate) fn refresh_cards(app: &mut SweepLoomApp) {
    let stamp = card_stamp(app);
    if stamp == app.project_card_stamp {
        return;
    }
    app.project_cards = collect_cards(app);
    app.project_card_stamp = stamp;
}

fn card_stamp(app: &SweepLoomApp) -> u64 {
    let entries = app.inventory.as_ref().map(|item| item.entries).unwrap_or(0);
    let first = app
        .review
        .first()
        .map(|row| row.candidate.logical_bytes)
        .unwrap_or(0);
    (app.review.len() as u64)
        .wrapping_mul(1_000_003)
        .wrapping_add((app.project_roots.len() as u64).wrapping_mul(97))
        .wrapping_add(entries)
        .wrapping_add(first)
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
    let paths = collect_paths(app);
    map.into_values()
        .map(|acc| ProjectCard::from_acc(acc, app.inventory.as_ref(), &paths))
        .collect()
}

fn collect_paths(app: &SweepLoomApp) -> HashSet<PathBuf> {
    let mut paths: HashSet<PathBuf> = app.project_roots.iter().cloned().collect();
    if let Some(report) = &app.inventory {
        paths.extend(report.projects.iter().cloned());
    }
    for row in &app.review {
        if let CandidateOwner::Project(id) = &row.candidate.owner {
            paths.insert(id.0.clone());
        }
    }
    paths
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
    sort: Sort,
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
    keys.sort_by(|left, right| {
        let left_bytes: u64 = buckets
            .get(left)
            .map(|indexes| indexes.iter().map(|&i| cards[i].bytes).sum())
            .unwrap_or(0);
        let right_bytes: u64 = buckets
            .get(right)
            .map(|indexes| indexes.iter().map(|&i| cards[i].bytes).sum())
            .unwrap_or(0);
        match sort.col {
            Col::Size => left_bytes.cmp(&right_bytes),
            Col::Name => left.cmp(right),
            _ if group == ProjectGroup::Kind => kind_rank(left).cmp(&kind_rank(right)),
            _ => left.cmp(right),
        }
    });
    if sort.desc && sort.col == Col::Size {
        keys.reverse();
    }
    let mut lines = Vec::new();
    for key in keys {
        let indexes = buckets.remove(&key).unwrap_or_default();
        let bytes = group_bytes(cards, &indexes);
        lines.push(Line::Group {
            key: key.clone(),
            title: group_title(cards, group, &key, &indexes),
            count: indexes.len(),
            bytes,
            expanded: !collapsed.contains(&key),
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
            ProjectGroup::Parent => self.folder_key.clone(),
            ProjectGroup::None => String::new(),
        }
    }

    fn from_acc(
        acc: Acc,
        inventory: Option<&InventoryReport>,
        projects: &HashSet<PathBuf>,
    ) -> Self {
        let kinds = classify_project(&acc.path);
        let group_kind = kinds.first().copied().unwrap_or(DevKind::Other).label();
        let labels = kinds
            .iter()
            .map(|kind| kind.label())
            .collect::<Vec<_>>()
            .join(", ");
        let parent = cluster_parent(&acc.path, projects);
        let cluster_bytes = inventory
            .and_then(|report| report.folder_bytes(&parent))
            .unwrap_or(0);
        let mut bytes = inventory
            .and_then(|report| report.folder_bytes(&acc.path))
            .unwrap_or(0);
        if bytes == 0 {
            bytes = reclaimable_bytes(&acc.bits);
        }
        if bytes == 0
            && let Some(report) = inventory
        {
            bytes = inventory_artifact_bytes(report, &acc.path);
        }
        Self {
            folder: cluster_title(&parent),
            folder_key: parent.to_string_lossy().into_owned(),
            cluster_bytes,
            artifacts: artifact_label(&acc.path, &acc.bits),
            path: acc.path,
            bytes,
            kinds: labels,
            group_kind,
        }
    }
}

fn group_title(cards: &[ProjectCard], group: ProjectGroup, key: &str, indexes: &[usize]) -> String {
    if group == ProjectGroup::Parent {
        return indexes
            .first()
            .and_then(|&index| cards.get(index))
            .map(|card| card.folder.clone())
            .unwrap_or_else(|| key.to_owned());
    }
    key.to_owned()
}

fn group_bytes(cards: &[ProjectCard], indexes: &[usize]) -> u64 {
    let cluster = indexes
        .first()
        .and_then(|&index| cards.get(index))
        .map(|card| card.cluster_bytes)
        .unwrap_or(0);
    if cluster > 0 {
        return cluster;
    }
    indexes
        .iter()
        .filter(|&&index| {
            !indexes
                .iter()
                .any(|&other| other != index && cards[index].path.starts_with(&cards[other].path))
        })
        .map(|&index| cards[index].bytes)
        .sum()
}

fn kind_rank(label: &str) -> u8 {
    match label {
        "Cargo" => 0,
        "Node" => 1,
        "Go" => 2,
        "Python" => 3,
        _ => 4,
    }
}
