use std::collections::HashSet;

use crate::sort::Sort;
use sweeploom_core::SessionKind;

use super::session_rows::{Line, SessionRow, table_lines};

fn row(title: &str, rss: u64, index: usize) -> SessionRow {
    SessionRow {
        index,
        title: title.to_owned(),
        group_key: title.to_owned(),
        subtitle: String::new(),
        node_procs: 0,
        status_detail: String::new(),
        source_tip: String::new(),
        kind: SessionKind::GenericApp,
        rss,
        cpu: 1.0,
        procs: 2,
        status: "Active now".to_owned(),
        project: "Unknown".to_owned(),
    }
}

#[test]
fn two_cursors_collapse_to_one_group() {
    let rows = [
        row("Cursor", 100, 0),
        row("Cursor", 40, 1),
        row("Github", 10, 2),
    ];
    let lines = table_lines(&rows, Sort::size_desc(), &HashSet::new());
    assert_eq!(lines.len(), 2);
    assert!(matches!(
        &lines[0],
        Line::Group {
            title,
            count: 2,
            rss: 140,
            expanded: false,
            ..
        } if title == "Cursor"
    ));
    assert!(matches!(
        &lines[1],
        Line::Session {
            index: 2,
            nested: false,
            ..
        }
    ));
}

#[test]
fn expanding_a_group_shows_members() {
    let rows = [row("Cursor", 100, 0), row("Cursor", 40, 1)];
    let open = HashSet::from(["Cursor".into()]);
    let lines = table_lines(&rows, Sort::size_desc(), &open);
    assert_eq!(lines.len(), 3);
    assert!(matches!(&lines[0], Line::Group { expanded: true, .. }));
    assert!(matches!(&lines[1], Line::Session { nested: true, .. }));
    assert!(matches!(&lines[2], Line::Session { nested: true, .. }));
}

#[test]
fn distinct_node_scripts_do_not_merge_by_the_same_display_name() {
    let mut first = row("Node · server.js", 100, 0);
    first.group_key = "script-a".into();
    first.node_procs = 1;
    let mut second = row("Node · server.js", 40, 1);
    second.group_key = "script-b".into();
    second.node_procs = 1;
    let lines = table_lines(&[first, second], Sort::size_desc(), &HashSet::new());
    assert_eq!(lines.len(), 2);
    assert!(
        lines
            .iter()
            .all(|line| matches!(line, Line::Session { .. }))
    );
}

#[test]
fn counts_distinguish_launches_all_members_and_node_executables() {
    let mut first = row("Cursor agent", 100, 0);
    first.procs = 5;
    first.node_procs = 1;
    let mut second = row("Cursor agent", 40, 1);
    second.procs = 4;
    second.node_procs = 1;
    let lines = table_lines(&[first, second], Sort::size_desc(), &HashSet::new());
    assert!(matches!(
        lines[0],
        Line::Group {
            count: 2,
            procs: 9,
            node_procs: 2,
            ..
        }
    ));
}

#[test]
fn session_names_sort_in_both_directions() {
    let rows = [row("Cursor", 100, 0), row("Node", 40, 1)];
    for (desc, expected) in [(false, 0), (true, 1)] {
        let lines = table_lines(
            &rows,
            Sort {
                col: crate::sort::Col::Name,
                desc,
            },
            &HashSet::new(),
        );
        assert!(matches!(lines[0], Line::Session { index, .. } if index == expected));
    }
}
