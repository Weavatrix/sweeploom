//! Simulator runtime disk images, with the simulators that still depend on them.
use super::{Item, Kind, PathBuf, Sim, Value, Verdict, pretty_runtime, text, value_bytes, verdict};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

pub(super) fn add(
    json: &str,
    items: &mut Vec<Item>,
    facts: &HashMap<String, Sim>,
) -> Result<(), String> {
    let value: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    items.extend(rows(&value, facts, verdict::now()));
    let installed: HashSet<(String, String)> = value
        .as_object()
        .into_iter()
        .flatten()
        .map(|(_, runtime)| (text(runtime, "runtimeIdentifier"), text(runtime, "build")))
        .collect();
    stale_caches(items, &installed);
    let registered: Vec<PathBuf> = value
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(_, runtime)| runtime.get("path").and_then(Value::as_str))
        .map(PathBuf::from)
        .collect();
    items.extend(unregistered(Path::new(IMAGES), &registered));
    Ok(())
}

const IMAGES: &str = "/Library/Developer/CoreSimulator/Cryptex/Images/bundle";

/// Non-empty runtime bundles that simctl no longer reports: root-owned leftovers.
pub(super) fn unregistered(root: &Path, registered: &[PathBuf]) -> Vec<Item> {
    let mut out = Vec::new();
    for entry in fs::read_dir(root).into_iter().flatten().flatten() {
        let path = entry.path();
        let empty = fs::read_dir(&path).map_or(true, |mut children| children.next().is_none());
        if empty || registered.iter().any(|known| known.starts_with(&path)) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        out.push(Item {
            id: format!("image:{}", name.replace(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-', "_")),
            name: format!("Unregistered runtime image · {name}"),
            kind: Kind::Inspect,
            bytes: None,
            usage: None,
            status: Verdict::Suggested.status(
                "simctl no longer lists this runtime image; root-owned, remove as admin after checking",
            ),
            scope: None,
            path: Some(path),
            selected: false,
            enabled: false,
        });
    }
    out
}

/// Runtime identifiers with an installed disk image.
pub(super) fn identifiers(json: &str) -> Option<HashSet<String>> {
    let value: Value = serde_json::from_str(json).ok()?;
    Some(
        value
            .as_object()?
            .values()
            .map(|runtime| text(runtime, "runtimeIdentifier"))
            .collect(),
    )
}

pub(super) fn rows(value: &Value, facts: &HashMap<String, Sim>, now: i64) -> Vec<Item> {
    let mut out = Vec::new();
    for (id, runtime) in value.as_object().into_iter().flatten() {
        let identifier = text(runtime, "runtimeIdentifier");
        let users = facts
            .values()
            .filter(|sim| sim.runtime == identifier)
            .count();
        let deletable = runtime
            .get("deletable")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let used = runtime
            .get("lastUsedAt")
            .and_then(Value::as_str)
            .and_then(verdict::parse_date)
            .map_or_else(
                || "never used".to_owned(),
                |at| format!("last used {}", verdict::ago(now - at)),
            );
        let label = format!(
            "{} ({})",
            pretty_runtime(&identifier),
            text(runtime, "build")
        );
        let status = if !deletable {
            "Runtime is managed by Xcode; cannot remove".to_owned()
        } else if users == 0 {
            Verdict::Suggested.status(&format!(
                "No simulators use {label}; {used}; remove runtime"
            ))
        } else {
            Verdict::Review.status(&format!(
                "{users} simulators use {label}; {used}; removing makes them unavailable"
            ))
        };
        out.push(Item {
            id: id.clone(),
            name: format!("Runtime: {label}"),
            kind: Kind::Runtime,
            bytes: value_bytes(runtime, "sizeBytes").or_else(|| value_bytes(runtime, "diskSize")),
            usage: None,
            status,
            scope: None,
            path: runtime
                .get("path")
                .and_then(Value::as_str)
                .map(PathBuf::from),
            selected: false,
            enabled: deletable,
        });
    }
    out
}

/// System dyld caches for runtimes that are no longer installed are dead weight.
fn stale_caches(items: &mut [Item], installed: &HashSet<(String, String)>) {
    for item in items.iter_mut().filter(|item| item.kind == Kind::Inspect) {
        let Some(path) = item.path.as_deref() else {
            continue;
        };
        if !path.starts_with("/Library/Developer/CoreSimulator/Caches/dyld") {
            continue;
        }
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Some((runtime, build)) = name.rsplit_once('.') else {
            continue;
        };
        if !installed.contains(&(runtime.to_owned(), build.to_owned())) {
            item.status = Verdict::Suggested
                .status("Runtime no longer installed — stale cache; root-owned, remove as admin");
        }
    }
}
