//! Simulator storage read straight from disk, so rows appear even while simctl is busy.
use super::super::xcode::{plist, verdict};
use super::{Item, Kind, Sim, item, pretty_runtime};
use std::{
    fs,
    path::{Path, PathBuf},
};

const SYSTEM: &str = "/Library/Developer/CoreSimulator";

/// A CoreSimulator device set; `scope` is passed to `simctl --set` (None = default set).
pub(super) struct Set {
    pub label: &'static str,
    pub path: PathBuf,
    pub scope: Option<String>,
}

pub(super) fn sets(home: &Path) -> Vec<Set> {
    let set = |label, relative: &str, default: bool| {
        let path = home.join(relative);
        let scope = (!default).then(|| path.display().to_string());
        Set { label, path, scope }
    };
    vec![
        set("Simulator", "Library/Developer/CoreSimulator/Devices", true),
        set("XCTest clone", "Library/Developer/XCTestDevices", false),
        set(
            "SwiftUI Preview",
            "Library/Developer/Xcode/UserData/Previews/Simulator Devices",
            false,
        ),
        set("Playground", "Library/Developer/XCPGDevices", false),
    ]
}

pub(super) fn is_udid(name: &str) -> bool {
    name.len() == 36
        && name.char_indices().all(|(index, ch)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                ch == '-'
            } else {
                ch.is_ascii_hexdigit()
            }
        })
}

/// Device folders of one set: registered devices and leftovers without a live device.plist.
pub(super) fn devices(set: &Set) -> (Vec<Sim>, Vec<Item>) {
    let mut sims = Vec::new();
    let mut orphans = Vec::new();
    for entry in fs::read_dir(&set.path).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let dir = entry.path();
        if !is_udid(&name) || !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let plist = dir.join("device.plist");
        let deleted = plist::read(&plist, "isDeleted").is_some_and(|value| value == "true");
        match plist::read(&plist, "name").filter(|_| !deleted) {
            Some(device) => sims.push(Sim {
                udid: plist::read(&plist, "UDID").unwrap_or_else(|| name.clone()),
                name: device,
                runtime: plist::read(&plist, "runtime").unwrap_or_default(),
                last_used: plist::read(&plist, "lastUsedAt")
                    .and_then(|date| verdict::parse_date(&date)),
                data: Some(dir.join("data")),
                ..Sim::default()
            }),
            None => orphans.push(orphan(set, &name, dir)),
        }
    }
    (sims, orphans)
}

pub(super) fn orphan(set: &Set, udid: &str, dir: PathBuf) -> Item {
    Item {
        id: format!("orphan:{udid}"),
        name: format!("Leftover {} folder · {udid}", set.label),
        kind: Kind::SimFiles,
        bytes: None,
        usage: None,
        status: verdict::Verdict::Suggested
            .status("No registered device; simctl no longer lists this folder"),
        scope: set.scope.clone(),
        path: Some(dir),
        selected: false,
        enabled: true,
    }
}

/// Rows read from disk before simctl answers.
pub(super) fn discover(home: &Path, host_build: Option<&str>) -> (Vec<Item>, Vec<Sim>) {
    let mut items = Vec::new();
    let mut sims = Vec::new();
    for set in sets(home) {
        let (found, orphans) = devices(&set);
        items.extend(found.iter().map(|sim| item(sim, &set)));
        items.extend(orphans);
        sims.extend(found);
    }
    let caches = home.join("Library/Developer/CoreSimulator/Caches");
    for (path, name) in children(&caches) {
        if name != "dyld" {
            items.push(files(&path, &format!("CoreSimulator cache · {name}"), None));
            continue;
        }
        for (build_dir, build) in children(&path) {
            let detail = build_note(&build, host_build);
            items.push(files(
                &build_dir,
                &format!("CoreSimulator dyld cache · {build}"),
                Some(detail),
            ));
        }
    }
    for (path, name) in children(&home.join("Library/Developer/CoreSimulator/Temp")) {
        let detail = (
            verdict::Verdict::Review,
            "Temporary simulator files".to_owned(),
        );
        items.push(files(
            &path,
            &format!("CoreSimulator temp · {name}"),
            Some(detail),
        ));
    }
    items.extend(system(host_build));
    (items, sims)
}

fn build_note(build: &str, host: Option<&str>) -> (verdict::Verdict, String) {
    match host {
        Some(host) if host != build => (
            verdict::Verdict::Suggested,
            format!("Built for macOS {build}; this Mac runs {host}, so it is stale"),
        ),
        _ => (
            verdict::Verdict::Review,
            "Shared simulator libraries; rebuilt on next boot".to_owned(),
        ),
    }
}

fn children(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut out: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| {
            (
                entry.path(),
                entry.file_name().to_string_lossy().into_owned(),
            )
        })
        .filter(|(_, name)| !name.starts_with('.'))
        .collect();
    out.sort();
    out
}

/// Removable simulator files (with a verdict), or read-only rows when `detail` is None.
fn files(path: &Path, name: &str, detail: Option<(verdict::Verdict, String)>) -> Item {
    let id = path
        .display()
        .to_string()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '.' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let (kind, status, enabled) = match detail {
        Some((verdict, text)) => (
            Kind::SimFiles,
            verdict.status(&format!("{text}; removed only while no simulator runs")),
            true,
        ),
        None => (
            Kind::Inspect,
            "Managed by CoreSimulator; inspect in Finder".to_owned(),
            false,
        ),
    };
    Item {
        id: format!("sim-files:{id}"),
        name: name.into(),
        kind,
        bytes: None,
        usage: None,
        status,
        scope: None,
        path: Some(path.to_owned()),
        selected: false,
        enabled,
    }
}

/// Root-owned runtime caches and legacy runtime bundles: reported, never removed here.
fn system(host_build: Option<&str>) -> Vec<Item> {
    let mut out = Vec::new();
    let root = Path::new(SYSTEM);
    for (build_dir, build) in children(&root.join("Caches/dyld")) {
        for (path, name) in children(&build_dir) {
            if !name.contains("SimRuntime") {
                continue;
            }
            let (runtime, runtime_build) = name.rsplit_once('.').unwrap_or((&name, ""));
            let mut row = files(
                &path,
                &format!(
                    "System dyld cache · {} ({runtime_build})",
                    pretty_runtime(runtime)
                ),
                None,
            );
            let (verdict, note) = build_note(&build, host_build);
            row.status = verdict.status(&format!(
                "{note}; root-owned — removed with its runtime or `xcrun simctl runtime dyld_shared_cache remove`"
            ));
            out.push(row);
        }
    }
    for (path, name) in children(&root.join("Profiles/Runtimes")) {
        let mut row = files(&path, &format!("Legacy runtime bundle · {name}"), None);
        row.status = verdict::Verdict::Review
            .status("Root-owned runtime installed by an older Xcode; delete its simulators, then remove it as admin");
        out.push(row);
    }
    out
}

pub(super) fn measure(item: &mut Item) {
    let Some(path) = &item.path else {
        return;
    };
    match sweeploom_storage::directory_disk_usage(path) {
        Ok(usage) => {
            item.bytes = Some(usage.bytes);
            item.usage = Some(usage);
            if !usage.complete {
                item.enabled = false;
                item.status = format!(
                    "Partial measurement ({} read errors); {}",
                    usage.errors, item.status
                );
            }
        }
        Err(error) => item.status = format!("Cannot inspect: {error}; {}", item.status),
    }
}
