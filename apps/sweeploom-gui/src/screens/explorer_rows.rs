//! Explorer tree rows. Collapse does not walk the disk.

use std::collections::HashSet;
use std::path::PathBuf;

use sweeploom_storage::{DirectoryNode, PathCategory};

use crate::sort::{Col, Sort};

/// One visible folder row after expand/collapse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Nesting level under the scan root.
    pub depth: usize,
    /// Full path. Used as the collapse key.
    pub path: PathBuf,
    /// File name.
    pub name: String,
    /// Logical bytes.
    pub bytes: u64,
    /// Logical size, for sparse-file details.
    pub logical_bytes: u64,
    /// File instead of folder.
    pub is_file: bool,
    /// Files under this node.
    pub files: u64,
    /// Folder category.
    pub category: PathCategory,
    /// True when this node has child folders in the inspector tree.
    pub has_children: bool,
    /// True when those children are currently shown.
    pub expanded: bool,
}

/// Flatten the inspector tree. Only expanded folders reveal children.
#[must_use]
pub fn visible_lines(root: &DirectoryNode, sort: Sort, expanded: &HashSet<String>) -> Vec<Line> {
    let mut out = Vec::new();
    collect(root, 0, sort, expanded, &mut out);
    out
}

pub(crate) fn path_key(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn collect(
    node: &DirectoryNode,
    depth: usize,
    sort: Sort,
    expanded: &HashSet<String>,
    out: &mut Vec<Line>,
) {
    if depth > 12 {
        return;
    }
    let mut children: Vec<&DirectoryNode> = node.children.iter().collect();
    children.sort_by(|left, right| match sort.col {
        Col::Name => left
            .path
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .cmp(
                &right
                    .path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_lowercase()),
            ),
        Col::Status => left.category.label().cmp(right.category.label()),
        Col::Procs => left.files.cmp(&right.files),
        _ => left.disk_bytes().cmp(&right.disk_bytes()),
    });
    if sort.desc {
        children.reverse();
    }
    for child in children {
        if out.len() >= 256 {
            return;
        }
        let has_children = !child.children.is_empty();
        let key = path_key(&child.path);
        let is_open = has_children && expanded.contains(&key);
        out.push(Line {
            depth,
            path: child.path.clone(),
            name: child
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(".")
                .to_owned(),
            bytes: child.disk_bytes(),
            logical_bytes: child.logical_bytes,
            is_file: child.is_file,
            files: child.files,
            category: child.category,
            has_children,
            expanded: is_open,
        });
        if is_open {
            collect(child, depth + 1, sort, expanded, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, bytes: u64, children: Vec<DirectoryNode>) -> DirectoryNode {
        DirectoryNode {
            path: PathBuf::from(name),
            logical_bytes: bytes,
            allocated_bytes: None,
            files: 1,
            directories: children.len() as u64,
            newest_mtime: None,
            newest_source_mtime: None,
            newest_generated_mtime: None,
            category: PathCategory::Unknown,
            children,
            is_file: false,
            incomplete: false,
        }
    }

    fn names(lines: &[Line]) -> Vec<&str> {
        lines.iter().map(|line| line.name.as_str()).collect()
    }

    #[test]
    fn closed_folders_hide_nested_children() {
        let tree = node(
            "root",
            9,
            vec![node(
                "cache",
                8,
                vec![node("huggingface", 7, vec![node("hub", 6, vec![])])],
            )],
        );
        let closed = visible_lines(&tree, Sort::size_desc(), &HashSet::new());
        assert_eq!(names(&closed), ["cache"]);
        let mut open = HashSet::new();
        open.insert(path_key(std::path::Path::new("cache")));
        let nested = visible_lines(&tree, Sort::size_desc(), &open);
        assert_eq!(names(&nested), ["cache", "huggingface"]);
    }
    #[test]
    fn explorer_sorts_disk_size_files_and_names_in_both_directions() {
        let mut a = node("Alpha", 10000, vec![]);
        a.allocated_bytes = Some(10);
        a.files = 100;
        let mut z = node("Zulu", 100, vec![]);
        z.allocated_bytes = Some(100);
        z.files = 2;
        let root = node("root", 10100, vec![a, z]);
        assert_eq!(
            names(&visible_lines(&root, Sort::size_desc(), &HashSet::new())),
            ["Zulu", "Alpha"]
        );
        assert_eq!(
            names(&visible_lines(
                &root,
                Sort {
                    col: Col::Procs,
                    desc: true
                },
                &HashSet::new()
            )),
            ["Alpha", "Zulu"]
        );
        assert_eq!(
            names(&visible_lines(
                &root,
                Sort {
                    col: Col::Name,
                    desc: true
                },
                &HashSet::new()
            )),
            ["Zulu", "Alpha"]
        );
    }
}
