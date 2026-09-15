use std::collections::HashSet;

use crate::sort::Sort;
use sweeploom_core::SessionKind;

use super::session_rows::{Line, SessionRow, table_lines};

fn row(title: &str, rss: u64, index: usize) -> SessionRow {
    SessionRow {
        index,
        title: title.to_owned(),
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
