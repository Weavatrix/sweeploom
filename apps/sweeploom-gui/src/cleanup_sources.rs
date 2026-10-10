//! Discover paths first, then stream full metadata measurements to the window.
use super::{Item, Kind, Listing, ListingMsg, Pane};
use crossbeam_channel::Sender;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use sweeploom_storage::DiskUsage;

#[path = "cleanup_catalog.rs"]
mod catalog;
use catalog::{Policy, RULES};
#[path = "cleanup_app_paths.rs"]
mod app_paths;
use app_paths::{app_caches, downloads};

pub(super) fn discover(home: &Path, pane: Pane) -> Listing {
    let mut listing = Listing::default();
    let mut seen = HashSet::new();
    for rule in RULES.iter().filter(|rule| rule.pane == pane) {
        descend(
            home,
            &home.join(rule.path),
            rule.label,
            rule.depth,
            (rule.policy, rule.consequence),
            &mut listing.items,
            &mut seen,
        );
    }
    if pane == Pane::Apps {
        app_caches(home, &mut listing.items, &mut seen);
    }
    if pane == Pane::Data {
        downloads(home, &mut listing.items, &mut seen);
        add(
            home,
            home.join(".Trash"),
            "macOS Trash",
            Policy::Inspect,
            "Already in Trash; inspect in Finder and empty manually to release space",
            &mut listing.items,
            &mut seen,
        );
    }
    listing.note = "Sizes use allocated disk blocks, with full metadata walks. Shared APFS blocks/hard links and Trash can affect space actually freed. No selection is automatic.".into();
    listing
}

pub(super) fn stream(home: &Path, pane: Pane, tx: &Sender<ListingMsg>) {
    let mut listing = discover(home, pane);
    if pane == Pane::Data {
        super::toolchains::classify(home, &mut listing.items);
    }
    if pane == Pane::Models {
        let (models, note) = super::models::listing(home);
        listing.items.extend(models);
        if !note.is_empty() {
            listing.note.push_str(&format!("\n{note}"));
        }
    }
    let mut items = listing.items.clone();
    if tx.send(ListingMsg::Initial(listing)).is_err() {
        return;
    }
    if pane == Pane::Caches {
        // Project/Git verdicts arrive before sizes; measurement keeps the new status.
        for index in super::xcode::enrich(home, &mut items) {
            if tx.send(ListingMsg::Measured(items[index].clone())).is_err() {
                return;
            }
        }
    }
    for item in &mut items {
        if item.kind != Kind::Model
            && item.bytes.is_none()
            && item.path.is_some()
            && !item.status.starts_with("Cannot inspect")
        {
            measure(item);
            if tx.send(ListingMsg::Measured(item.clone())).is_err() {
                return;
            }
        }
    }
    let _ = tx.send(ListingMsg::Done);
}

fn descend(
    home: &Path,
    path: &Path,
    label: &str,
    depth: usize,
    (policy, hint): (Policy, &str),
    items: &mut Vec<Item>,
    seen: &mut HashSet<PathBuf>,
) {
    if !safe_path(home, path) {
        return;
    }
    if depth == 0 {
        add(home, path.to_owned(), label, policy, hint, items, seen);
        return;
    }
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            add_error(path, label, error.to_string(), items, seen);
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                add_error(path, label, error.to_string(), items, seen);
                continue;
            }
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name == "version.txt" {
            continue;
        }
        if label == "Playwright browser binaries"
            && !["chromium", "chrome", "firefox", "webkit", "ffmpeg"]
                .iter()
                .any(|prefix| name.starts_with(prefix))
        {
            continue;
        }
        let child = entry.path();
        let Ok(meta) = fs::symlink_metadata(&child) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        let display = format!("{label} · {name}");
        // Some versioned stores have an extra directory layer; files remain individually visible.
        descend(
            home,
            &child,
            &display,
            if meta.is_dir() { depth - 1 } else { 0 },
            (policy, hint),
            items,
            seen,
        );
    }
}

pub(super) fn safe_path(home: &Path, path: &Path) -> bool {
    if path == home || !path.starts_with(home) {
        return false;
    }
    let mut current = Some(path);
    while let Some(part) = current {
        if part == home {
            return true;
        }
        if fs::symlink_metadata(part).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return false;
        }
        current = part.parent();
    }
    false
}

fn add(
    home: &Path,
    path: PathBuf,
    label: &str,
    policy: Policy,
    hint: &str,
    items: &mut Vec<Item>,
    seen: &mut HashSet<PathBuf>,
) {
    if !safe_path(home, &path) || !seen.insert(path.clone()) {
        return;
    }
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_dir() || meta.is_file() => {}
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            seen.remove(&path);
            add_error(&path, label, error.to_string(), items, seen);
            return;
        }
        _ => return,
    }
    let blocked = if policy == Policy::Inspect {
        None
    } else {
        sweeploom_exec::trash_path_reason(&path, home)
    };
    let kind = if blocked.is_some() {
        Kind::Inspect
    } else {
        match policy {
            Policy::Cache => Kind::Cache,
            Policy::Trash => Kind::Trash,
            Policy::Inspect => Kind::Inspect,
        }
    };
    items.push(Item {
        id: path.display().to_string(),
        name: label.into(),
        kind,
        bytes: None,
        usage: None,
        status: blocked.unwrap_or_else(|| hint.into()),
        scope: None,
        path: Some(path),
        selected: false,
        enabled: false,
    });
}

fn add_error(
    path: &Path,
    label: &str,
    error: String,
    items: &mut Vec<Item>,
    seen: &mut HashSet<PathBuf>,
) {
    if !seen.insert(path.to_owned()) {
        return;
    }
    items.push(Item {
        id: path.display().to_string(),
        name: label.into(),
        kind: Kind::Inspect,
        bytes: None,
        usage: None,
        status: format!("Cannot inspect: {error}"),
        scope: None,
        path: Some(path.to_owned()),
        selected: false,
        enabled: false,
    });
}

fn measure(item: &mut Item) {
    let native_allowed = item.kind == Kind::Toolchain && item.enabled;
    item.enabled = false;
    item.bytes = None;
    item.usage = None;
    let Some(path) = &item.path else {
        return;
    };
    let result = fs::symlink_metadata(path).and_then(|meta| {
        if meta.is_dir() {
            sweeploom_storage::directory_disk_usage(path)
        } else if meta.is_file() {
            Ok(DiskUsage {
                bytes: super::allocated(&meta).unwrap_or(meta.len()),
                logical_bytes: meta.len(),
                files: 1,
                ..Default::default()
            })
        } else {
            Err(std::io::Error::other("Not a regular file or directory"))
        }
    });
    match result {
        Ok(usage) => {
            item.bytes = Some(usage.bytes);
            item.usage = Some(usage);
            item.enabled = usage.complete
                && (matches!(item.kind, Kind::Cache | Kind::Trash) || native_allowed);
            if !usage.complete {
                item.status = format!(
                    "Partial measurement ({} read errors); {}",
                    usage.errors, item.status
                );
            }
        }
        Err(error) => item.status = format!("Cannot inspect: {error}; {}", item.status),
    }
}

#[cfg(test)]
#[path = "cleanup_sources_tests.rs"]
mod tests;
