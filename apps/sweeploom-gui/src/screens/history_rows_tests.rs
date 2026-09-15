use std::collections::HashSet;

use crate::sort::Sort;

use super::history_rows::{HistRow, Line, table_lines};

fn row(name: &str, rss: u64) -> HistRow {
    HistRow {
        name: name.to_owned(),
        pid: 1,
        rss,
        peak: rss,
        cpu: 0.0,
        avg_5m: "unavailable".to_owned(),
        samples: 1,
        spark: Vec::new(),
        session: None,
    }
}

#[test]
fn groups_start_collapsed() {
    let rows = [row("Code.exe", 10), row("Code.exe", 20)];
    let lines = table_lines(&rows, &[], &[], Sort::size_desc(), &HashSet::new());
    assert_eq!(lines.len(), 1);
    let Line::Group {
        title,
        expanded,
        count,
        ..
    } = &lines[0]
    else {
        panic!("expected a group");
    };
    assert_eq!(title, "Code");
    assert_eq!(*count, 2);
    assert!(!*expanded);
}

#[test]
fn expanding_a_group_shows_members() {
    let rows = [row("Code.exe", 10), row("Code.exe", 20)];
    let open = HashSet::from(["n:Code".into()]);
    let lines = table_lines(&rows, &[], &[], Sort::size_desc(), &open);
    assert_eq!(lines.len(), 3);
    assert!(matches!(&lines[0], Line::Group { expanded: true, .. }));
}

#[test]
fn group_rolls_peak_avg_spark_and_samples() {
    let mut left = row("Code.exe", 10);
    left.peak = 100;
    left.cpu = 1.0;
    left.avg_5m = "1.5%".into();
    left.samples = 10;
    left.spark = vec![1.0, 2.0];
    let mut right = row("Code.exe", 20);
    right.peak = 40;
    right.cpu = 2.5;
    right.avg_5m = "2.0%".into();
    right.samples = 40;
    right.spark = vec![3.0, 4.0];
    let lines = table_lines(&[left, right], &[], &[], Sort::size_desc(), &HashSet::new());
    let Line::Group {
        rss,
        peak,
        cpu,
        avg_5m,
        samples,
        spark,
        count,
        ..
    } = &lines[0]
    else {
        panic!("expected a group");
    };
    assert_eq!(*rss, 30);
    assert_eq!(*peak, 100);
    assert_eq!(*cpu, 3.5);
    assert_eq!(avg_5m, "3.5%");
    assert_eq!(*samples, 40);
    assert_eq!(spark, &[4.0, 6.0]);
    assert_eq!(*count, 2);
}
