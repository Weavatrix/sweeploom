use super::*;
use crate::format::format_bytes;
use serde_json::Value;

pub(super) fn docker_program() -> String {
    [
        "/usr/local/bin/docker",
        "/opt/homebrew/bin/docker",
        "/Applications/Docker.app/Contents/Resources/bin/docker",
    ]
    .into_iter()
    .find(|path| std::path::Path::new(path).exists())
    .unwrap_or("docker")
    .into()
}
pub(super) fn docker_listing(home: &std::path::Path) -> Listing {
    let mut listing = Listing::default();
    let raw = home.join("Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw");
    if let Ok(meta) = std::fs::metadata(&raw) {
        listing.items.push(Item {
            id: raw.display().to_string(),
            name: "Docker VM disk (allocated blocks)".into(),
            kind: Kind::Inspect,
            bytes: allocated(&meta),
            usage: Some(sweeploom_storage::DiskUsage {
                bytes: allocated(&meta).unwrap_or(meta.len()),
                logical_bytes: meta.len(),
                files: 1,
                ..Default::default()
            }),
            status: format!(
                "Logical capacity {}; clean Docker objects below",
                format_bytes(meta.len())
            ),
            scope: None,
            path: Some(raw),
            selected: false,
            enabled: false,
        });
    }
    let context = command(&docker_program(), &["context", "show"])
        .ok()
        .map(|name| name.trim().to_owned());
    match command(
        &docker_program(),
        &["system", "df", "--verbose", "--format", "{{json .}}"],
    )
    .and_then(|text| {
        serde_json::from_str::<Value>(&text)
            .map_err(|e| format!("Could not parse Docker inventory: {e}"))
    }) {
        Ok(value) => {
            let mut items = parse_docker(&value);
            for item in &mut items {
                item.scope = context.clone();
            }
            listing.items.extend(items);
            listing.note = "Image sizes use unique layers where available. Shared layers and VM compaction affect physical space freed.".into();
        }
        Err(error) => listing.note = error,
    }
    listing
}
pub(super) fn parse_docker(value: &Value) -> Vec<Item> {
    let mut out = Vec::new();
    for (key, kind) in [
        ("Images", Kind::Image),
        ("Containers", Kind::Container),
        ("Volumes", Kind::Volume),
    ] {
        for item in value
            .get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let id = text(item, "ID");
            let id = if id.is_empty() {
                text(item, "Name")
            } else {
                id
            };
            let in_use = value_count(item, "Containers") > 0
                || text(item, "State") == "running"
                || text(item, "Status").starts_with("Up ")
                || value_count(item, "Links") > 0;
            let mut name = text(item, "Repository");
            if !name.is_empty() {
                name = format!("{}:{}", name, text(item, "Tag"));
            } else {
                name = text(item, "Names");
                if name.is_empty() {
                    name = id.clone();
                }
            }
            let bytes = value_bytes(item, "UniqueSize")
                .or_else(|| value_bytes(item, "Size"))
                .or_else(|| {
                    item.get("UsageData")
                        .and_then(|usage| value_bytes(usage, "Size"))
                });
            out.push(Item {
                id,
                name,
                kind: kind.clone(),
                bytes,
                usage: None,
                status: if in_use {
                    "In use — cannot remove".into()
                } else {
                    format!("Remove {}", key.to_ascii_lowercase())
                },
                scope: None,
                path: None,
                selected: false,
                enabled: !in_use,
            });
        }
    }
    let cache = value
        .get("BuildCache")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| value_bytes(item, "Size"))
                .sum()
        });
    if cache.is_some_and(|bytes| bytes > 0) {
        out.push(Item {
            id: "older-than-7-days".into(),
            name: "Unused build cache older than 7 days".into(),
            kind: Kind::BuildCache,
            bytes: None,
            usage: None,
            status: format!(
                "Prune only old unused entries; total cache {}",
                cache.map(format_bytes).unwrap_or_default()
            ),
            scope: None,
            path: None,
            selected: false,
            enabled: true,
        });
    }
    out
}
pub(super) fn docker_command(item: &Item, args: &[&str]) -> Result<String, String> {
    let context = item
        .scope
        .as_deref()
        .ok_or("Docker context was unavailable; refresh inventory first")?;
    let mut scoped = vec!["--context", context];
    scoped.extend_from_slice(args);
    command(&docker_program(), &scoped)
}
