use super::*;

#[test]
fn unreadable_or_missing_root_is_incomplete_instead_of_an_exact_empty_store() {
    let path = unique_root().join("missing");
    let listed = list_store(&path, Limits::default());
    assert!(listed.capped);
    assert_eq!(listed.file_count, 0);
}
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_root() -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "sweeploom-ai-{}-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|item| item.as_nanos())
            .unwrap_or(0)
    ))
}

#[test]
fn lists_every_immediate_child() {
    let root = unique_root();
    fs::create_dir_all(root.join("cache")).unwrap();
    fs::write(root.join("cache").join("a.bin"), b"abcd").unwrap();
    fs::write(root.join(".credentials.json"), b"nope").unwrap();
    fs::write(root.join("history.jsonl"), b"xy").unwrap();
    let listed = list_store(&root, Limits::default());
    fs::remove_dir_all(&root).ok();
    let names: Vec<_> = listed
        .entries
        .iter()
        .map(|item| item.relative.as_str())
        .collect();
    assert!(names.contains(&"cache"), "{names:?}");
    assert!(names.contains(&".credentials.json"), "{names:?}");
    assert!(names.contains(&"history.jsonl"), "{names:?}");
    assert_eq!(listed.file_count, 3);
    assert_eq!(listed.logical_bytes, 10);
    assert!(!listed.capped);
}

#[test]
fn file_cap_is_honest() {
    let root = unique_root();
    fs::create_dir_all(root.join("nested")).unwrap();
    fs::write(root.join("nested").join("one.txt"), b"a").unwrap();
    fs::write(root.join("nested").join("two.txt"), b"b").unwrap();
    let listed = list_store(
        &root,
        Limits {
            max_files: 1,
            max_depth: 8,
            max_entries: 256,
        },
    );
    fs::remove_dir_all(&root).ok();
    assert_eq!(listed.file_count, 1);
    assert!(listed.capped);
}

#[test]
fn deep_files_are_counted() {
    let root = unique_root();
    let mut dir = root.join("projects");
    for name in ["a", "b", "c", "d", "e", "f"] {
        dir.push(name);
    }
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("session.jsonl"), vec![0_u8; 64]).unwrap();
    let listed = list_store(&root, Limits::default());
    fs::remove_dir_all(&root).ok();
    assert_eq!(listed.file_count, 1);
    assert_eq!(listed.logical_bytes, 64);
    assert!(!listed.capped);
}

#[test]
fn shallow_depth_misses_nested_payload() {
    let root = unique_root();
    let mut dir = root.join("projects");
    for name in ["a", "b", "c", "d", "e", "f"] {
        dir.push(name);
    }
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("session.jsonl"), vec![0_u8; 64]).unwrap();
    let shallow = list_store(
        &root,
        Limits {
            max_files: 100,
            max_depth: 4,
            max_entries: 256,
        },
    );
    let full = list_store(&root, Limits::default());
    fs::remove_dir_all(&root).ok();
    assert!(shallow.capped);
    assert_eq!(shallow.logical_bytes, 0);
    assert_eq!(full.logical_bytes, 64);
    assert!(!full.capped);
}

#[test]
fn sibling_dirs_keep_independent_file_budgets() {
    let root = unique_root();
    fs::create_dir_all(root.join("a")).unwrap();
    fs::create_dir_all(root.join("b")).unwrap();
    fs::write(root.join("a").join("1"), b"aa").unwrap();
    fs::write(root.join("a").join("2"), b"bb").unwrap();
    fs::write(root.join("b").join("1"), b"cc").unwrap();
    fs::write(root.join("b").join("2"), b"dd").unwrap();
    let listed = list_store(
        &root,
        Limits {
            max_files: 2,
            max_depth: 8,
            max_entries: 256,
        },
    );
    fs::remove_dir_all(&root).ok();
    assert_eq!(listed.file_count, 4);
    let left = listed
        .entries
        .iter()
        .find(|item| item.relative == "a")
        .expect("a");
    let right = listed
        .entries
        .iter()
        .find(|item| item.relative == "b")
        .expect("b");
    assert_eq!(left.file_count, 2);
    assert_eq!(right.file_count, 2);
}
