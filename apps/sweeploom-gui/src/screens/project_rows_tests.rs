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
        cluster_bytes: 0,
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
