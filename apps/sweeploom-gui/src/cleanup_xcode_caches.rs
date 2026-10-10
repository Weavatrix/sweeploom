//! DerivedData and DeviceSupport rows annotated with project, Git and version evidence.
use super::{
    super::Item,
    Projects, derived_data, modified, plist, project_name, recorded_workspaces,
    verdict::{self, Verdict},
    workspace_of,
};
use std::{collections::HashMap, path::Path};

/// Update Xcode rows in place and return the indexes that changed.
pub(in super::super) fn enrich(home: &Path, items: &mut [Item]) -> Vec<usize> {
    let derived = derived_data(home);
    let xcode = home.join("Library/Developer/Xcode");
    let projects = items
        .iter()
        .any(|item| parent(item) == Some(derived.as_path()))
        .then(|| Projects::load(home, &recorded_workspaces(home)));
    let mut newest: HashMap<String, Vec<u32>> = HashMap::new();
    for item in items.iter() {
        if let Some((platform, version)) = support(item, &xcode) {
            let entry = newest.entry(platform).or_default();
            if version > *entry {
                *entry = version;
            }
        }
    }
    let now = verdict::now();
    let mut changed = Vec::new();
    for (index, item) in items.iter_mut().enumerate() {
        let updated = if parent(item) == Some(derived.as_path()) {
            projects
                .as_ref()
                .is_some_and(|projects| derived_row(item, projects, now))
        } else if let Some((platform, version)) = support(item, &xcode) {
            support_row(item, &version, &newest[&platform], now);
            true
        } else {
            false
        };
        if updated {
            changed.push(index);
        }
    }
    changed
}

fn parent(item: &Item) -> Option<&Path> {
    item.path.as_deref()?.parent()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn derived_row(item: &mut Item, projects: &Projects, now: i64) -> bool {
    let Some(path) = item.path.clone() else {
        return false;
    };
    let name = file_name(&path);
    if name.ends_with(".noindex") || name == "SDKExplicitPrecompiledModules" {
        item.status = Verdict::Review
            .status("Shared module cache for every project; rebuilt on the next build");
        return true;
    }
    let written = modified(&path).map_or_else(
        || "age unknown".to_owned(),
        |at| format!("last written {}", verdict::ago(now - at)),
    );
    if let Some(workspace) = workspace_of(&path) {
        let (verdict, detail) = projects.verdict(&workspace);
        let opened = plist::read(&path.join("info.plist"), "LastAccessedDate")
            .and_then(|date| verdict::parse_date(&date))
            .map_or(written, |at| {
                format!("opened in Xcode {}", verdict::ago(now - at))
            });
        item.name = format!("Xcode DerivedData · {}", project_name(&workspace));
        item.status = verdict.status(&format!("{detail}; {opened} · {}", workspace.display()));
    } else {
        let (stem, suffix) = name.rsplit_once('-').unwrap_or((name.as_str(), ""));
        item.name = format!("Xcode DerivedData · {stem}");
        item.status = if let Some(workspace) = projects.checkout(suffix) {
            let (verdict, detail) = projects.verdict(workspace);
            verdict.status(&format!(
                "{detail}; {written} · {} (matched by folder name)",
                workspace.display()
            ))
        } else {
            let fresh = modified(&path).is_none_or(|at| now - at < 3600);
            if fresh {
                Verdict::Review
            } else {
                Verdict::Suggested
            }
            .status(&format!(
                "No checkout on disk builds here (removed worktree, temp or CI checkout); {written}"
            ))
        };
    }
    true
}

/// Platform folder and OS version of a `* DeviceSupport/<device> <version> (<build>)` row.
fn support(item: &Item, xcode: &Path) -> Option<(String, Vec<u32>)> {
    let path = item.path.as_deref()?;
    let platform = path.parent()?;
    if platform.parent()? != xcode {
        return None;
    }
    let platform = file_name(platform);
    if !platform.ends_with("DeviceSupport") {
        return None;
    }
    Some((platform, version(&file_name(path))?))
}

pub(super) fn version(name: &str) -> Option<Vec<u32>> {
    name.split_whitespace().find_map(|token| {
        token
            .split('.')
            .map(|part| part.parse::<u32>().ok())
            .collect::<Option<Vec<_>>>()
            .filter(|parts| parts.len() >= 2)
    })
}

fn support_row(item: &mut Item, version: &[u32], newest: &[u32], now: i64) {
    let text = |parts: &[u32]| {
        parts
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(".")
    };
    let used = item
        .path
        .as_deref()
        .and_then(modified)
        .map(|at| format!("; last used {}", verdict::ago(now - at)))
        .unwrap_or_default();
    item.status = if version < newest {
        Verdict::Suggested.status(&format!(
            "Superseded by {} symbols; re-copied if a device on {} connects{used}",
            text(newest),
            text(version)
        ))
    } else {
        Verdict::Review.status(&format!(
            "Newest symbols ({}); Xcode copies them again from the device on next connect{used}",
            text(version)
        ))
    };
}
