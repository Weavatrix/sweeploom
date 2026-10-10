use std::collections::HashSet;
use std::path::PathBuf;

use crate::sort::{Col, Sort};

use super::project_facts::cluster_title;
use super::project_rows::{Line, ProjectCard, ProjectGroup, table_lines};

fn card(path: &str, bytes: u64, kind: &'static str) -> ProjectCard {
    let path = PathBuf::from(path);
    let parent = path.parent().unwrap_or(path.as_path());
    let folder_key = parent.to_string_lossy().into_owned();
    ProjectCard {
        folder: cluster_title(parent),
        folder_key,
        path,
        bytes: Some(sweeploom_storage::DiskUsage {
            bytes,
            logical_bytes: bytes,
            ..Default::default()
        }),
        size_error: None,
        artifact_bytes: 0,
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
    let open = table_lines(
        &cards,
        ProjectGroup::Kind,
        &HashSet::new(),
        Sort {
            col: Col::Status,
            desc: false,
        },
    );
    assert_eq!(keys(&open), ["g:Cargo", "p:1", "g:Node", "p:0", "p:2"]);
    let collapsed = HashSet::from(["Node".to_owned()]);
    let shut = table_lines(
        &cards,
        ProjectGroup::Kind,
        &collapsed,
        Sort {
            col: Col::Status,
            desc: false,
        },
    );
    assert_eq!(keys(&shut), ["g:Cargo", "p:1", "g:Node"]);
    let Line::Group { expanded, .. } = &shut[2] else {
        panic!("node group");
    };
    assert!(!*expanded);
}

#[test]
fn folder_groups_use_the_parent_name() {
    let cards = [card("src/one", 1, "Node"), card("lib/two", 2, "Node")];
    let lines = table_lines(
        &cards,
        ProjectGroup::Parent,
        &HashSet::new(),
        Sort {
            col: Col::Name,
            desc: false,
        },
    );
    assert_eq!(keys(&lines), ["g:lib", "p:1", "g:src", "p:0"]);
}

#[test]
fn folder_size_sort_uses_the_size_displayed_and_reverses_names() {
    let cards = [
        card("a/small-project", 10, "Node"),
        card("z/large-project", 100, "Node"),
    ];
    let lines = table_lines(
        &cards,
        ProjectGroup::Parent,
        &HashSet::new(),
        Sort::size_desc(),
    );
    assert_eq!(keys(&lines), ["g:z", "p:1", "g:a", "p:0"]);
    let names = table_lines(
        &cards,
        ProjectGroup::Parent,
        &HashSet::new(),
        Sort {
            col: Col::Name,
            desc: true,
        },
    );
    assert_eq!(keys(&names), ["g:z", "p:1", "g:a", "p:0"]);
    let kinds = table_lines(
        &cards,
        ProjectGroup::Kind,
        &HashSet::new(),
        Sort::size_desc(),
    );
    assert!(matches!(&kinds[0], Line::Group { bytes: Some(usage), .. } if usage.bytes == 110));
}

#[test]
fn unknown_sizes_stay_unknown_and_sort_below_measured_sizes_in_both_directions() {
    use super::project_rows::{size_caption, sort_cards};
    let mut cards = [
        card("repos/pending", 0, "Node"),
        card("repos/empty", 0, "Node"),
        card("repos/full", 20, "Node"),
    ];
    cards[0].bytes = None;
    assert_eq!(size_caption(cards[0].bytes, None), "Measuring…");
    assert_eq!(size_caption(None, Some("denied")), "Unavailable");
    assert_eq!(size_caption(cards[1].bytes, None), "0 KB");
    sort_cards(&mut cards, Sort::size_desc());
    assert_eq!(cards[0].name(), "full");
    assert_eq!(cards[2].name(), "pending");
    sort_cards(
        &mut cards,
        Sort {
            col: Col::Size,
            desc: false,
        },
    );
    assert_eq!(cards[0].name(), "empty");
    assert_eq!(cards[2].name(), "pending");
}

#[test]
fn group_totals_are_lower_bounds_until_root_projects_are_measured() {
    let mut cards = [
        card("repos/app", 100, "Node"),
        card("repos/app/child", 20, "Node"),
        card("repos/pending", 0, "Node"),
    ];
    cards[2].bytes = None;
    let lines = table_lines(
        &cards,
        ProjectGroup::Kind,
        &HashSet::new(),
        Sort::size_desc(),
    );
    let Line::Group {
        bytes: Some(usage), ..
    } = lines[0]
    else {
        panic!("group size");
    };
    assert_eq!(usage.bytes, 100);
    assert!(!usage.complete);
    cards[2].bytes = Some(sweeploom_storage::DiskUsage {
        bytes: 50,
        ..Default::default()
    });
    let lines = table_lines(
        &cards,
        ProjectGroup::Kind,
        &HashSet::new(),
        Sort::size_desc(),
    );
    let Line::Group {
        bytes: Some(usage), ..
    } = lines[0]
    else {
        panic!("group size");
    };
    assert_eq!(usage.bytes, 150);
    assert!(usage.complete);
}
