//! Filesystem helpers for disk actions: tree pruning and Finder reveal.
use std::path::PathBuf;

pub(super) fn remove_nodes(node: &mut sweeploom_storage::DirectoryNode, removed: &[PathBuf]) {
    let mut logical = 0;
    let mut allocated = 0;
    let mut files = 0;
    node.children.retain(|child| {
        if removed.iter().any(|path| child.path.starts_with(path)) {
            logical += child.logical_bytes;
            allocated += child.disk_bytes();
            files += child.files;
            false
        } else {
            true
        }
    });
    for child in &mut node.children {
        let before = (child.logical_bytes, child.disk_bytes(), child.files);
        remove_nodes(child, removed);
        logical += before.0.saturating_sub(child.logical_bytes);
        allocated += before.1.saturating_sub(child.disk_bytes());
        files += before.2.saturating_sub(child.files);
    }
    node.logical_bytes = node.logical_bytes.saturating_sub(logical);
    node.allocated_bytes = node
        .allocated_bytes
        .map(|bytes| bytes.saturating_sub(allocated));
    node.files = node.files.saturating_sub(files);
}

pub(super) fn reveal(paths: &[PathBuf]) -> Result<(), String> {
    if paths.is_empty() {
        return Err("Nothing selected.".into());
    }
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = std::process::Command::new("/usr/bin/open");
        command.arg("-R").args(paths);
        command
    };
    #[cfg(windows)]
    let mut command = {
        let mut command = std::process::Command::new("explorer.exe");
        command.arg(format!("/select,{}", paths[0].display()));
        command
    };
    #[cfg(not(any(windows, target_os = "macos")))]
    let mut command = {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(paths[0].parent().unwrap_or(&paths[0]));
        command
    };
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
