//! Ollama's shared blob store is only changed through its local native API.
use super::{Item, Kind};
use serde_json::Value;
use std::{collections::HashSet, fs, path::Path};
use sweeploom_storage::DiskUsage;

const HOST: &str = "http://127.0.0.1:11434";

fn api(endpoint: &str) -> Result<Value, String> {
    let raw = super::command(
        "curl",
        &[
            "--noproxy",
            "*",
            "--connect-timeout",
            "1",
            "--max-time",
            "3",
            "--fail",
            "--silent",
            "--show-error",
            &format!("{HOST}/api/{endpoint}"),
        ],
    )?;
    let value: Value = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    if !value.get("models").is_some_and(Value::is_array) {
        return Err("Ollama returned an invalid model inventory".into());
    }
    Ok(value)
}

pub(super) fn listing(home: &Path) -> (Vec<Item>, String) {
    let root = home.join(".ollama/models");
    let mut items = if super::sources::safe_path(home, &root) {
        offline(&root)
    } else {
        Vec::new()
    };
    let tags = api("tags");
    let running = api("ps");
    if let Ok(tags) = &tags {
        for model in tags["models"].as_array().into_iter().flatten() {
            let name = super::text(model, "name");
            if name.is_empty() {
                continue;
            }
            let index = match items.iter().position(|item| item.id == name) {
                Some(index) => index,
                None => {
                    items.push(Item {
                        id: name.clone(),
                        name: format!("Ollama · {name}"),
                        kind: Kind::Model,
                        bytes: model.get("size").and_then(Value::as_u64),
                        usage: None,
                        status: String::new(),
                        scope: None,
                        path: None,
                        selected: false,
                        enabled: false,
                    });
                    items.len() - 1
                }
            };
            let item = &mut items[index];
            let digest = super::text(model, "digest");
            item.scope = Some(digest.clone());
            let loaded = running
                .as_ref()
                .map(|value| is_loaded(value, &name, &digest));
            item.enabled = matches!(loaded, Ok(false)) && !digest.is_empty();
            item.status = match loaded {
                Ok(true) => "Loaded in Ollama — unload explicitly before removal".into(),
                Ok(false) => {
                    "Remove this model through Ollama; shared layers are retained where needed"
                        .into()
                }
                Err(_) => {
                    "Cannot verify loaded models; start Ollama and refresh before removal".into()
                }
            };
            if item.usage.is_some_and(|usage| !usage.complete) {
                item.status = format!("Partial local measurement; {}", item.status);
            }
        }
    }
    let note = if tags.is_err() || running.is_err() {
        "Ollama is unavailable or its running models could not be verified. Local manifests are shown; start Ollama, then refresh to enable per-model removal.".into()
    } else {
        "Local model sizes use allocated blobs when available; other models use Ollama-reported sizes. Models can share layers, so their sizes must not be added as guaranteed reclaimable space.".into()
    };
    (items, note)
}

fn is_loaded(value: &Value, name: &str, digest: &str) -> bool {
    value["models"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|model| {
            super::text(model, "name") == name
                || (!digest.is_empty() && super::text(model, "digest") == digest)
        })
}

fn offline(root: &Path) -> Vec<Item> {
    let manifests = root.join("manifests");
    let mut stack = vec![(manifests.clone(), 0)];
    let mut items = Vec::new();
    while let Some((directory, depth)) = stack.pop() {
        if depth > 6
            || fs::symlink_metadata(&directory).is_ok_and(|meta| meta.file_type().is_symlink())
        {
            continue;
        }
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                stack.push((path, depth + 1));
                continue;
            }
            if !meta.is_file() || meta.len() > 1_000_000 {
                continue;
            }
            let Some(name) = manifest_name(&manifests, &path) else {
                continue;
            };
            let Ok(raw) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<Value>(&raw) else {
                continue;
            };
            let usage = blob_usage(root, &value);
            items.push(Item {
                id: name.clone(),
                name: format!("Ollama · {name}"),
                kind: Kind::Model,
                bytes: if usage.files == 0 && !usage.complete {
                    None
                } else {
                    Some(usage.bytes)
                },
                usage: Some(usage),
                status: "Ollama is offline; start it and refresh to remove this model".into(),
                scope: None,
                path: Some(path),
                selected: false,
                enabled: false,
            });
        }
    }
    items
}

fn manifest_name(root: &Path, path: &Path) -> Option<String> {
    let parts: Vec<_> = path
        .strip_prefix(root)
        .ok()?
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.len() < 4 {
        return None;
    }
    let tag = parts.last()?;
    let namespace = &parts[1..parts.len() - 1];
    let repository = if namespace.first().is_some_and(|part| part == "library") {
        namespace[1..].join("/")
    } else {
        namespace.join("/")
    };
    let prefix = if parts[0] == "registry.ollama.ai" {
        String::new()
    } else {
        format!("{}/", parts[0])
    };
    Some(format!("{prefix}{repository}:{tag}"))
}

fn blob_usage(root: &Path, value: &Value) -> DiskUsage {
    let mut usage = DiskUsage::default();
    let mut seen = HashSet::new();
    let descriptors = value
        .get("layers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .chain(value.get("config"));
    for descriptor in descriptors {
        let digest = super::text(descriptor, "digest");
        if !valid_digest(&digest) {
            usage.errors += 1;
            continue;
        }
        if !seen.insert(digest.clone()) {
            continue;
        }
        let path = root.join("blobs").join(digest.replace(':', "-"));
        match fs::symlink_metadata(path) {
            Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => {
                usage.bytes = usage
                    .bytes
                    .saturating_add(super::allocated(&meta).unwrap_or(meta.len()));
                usage.logical_bytes = usage.logical_bytes.saturating_add(meta.len());
                usage.files += 1;
            }
            _ => usage.errors += 1,
        }
    }
    if seen.is_empty() {
        usage.errors += 1;
    }
    usage.complete = usage.errors == 0;
    usage
}

fn valid_digest(digest: &str) -> bool {
    digest
        .strip_prefix("sha256:")
        .is_some_and(|hash| hash.len() == 64 && hash.chars().all(|ch| ch.is_ascii_hexdigit()))
}

pub(super) fn remove(item: &Item) -> Result<String, String> {
    let tags = api("tags")?;
    let running = api("ps")?;
    validate_removal(item, &tags, &running)?;
    let body = serde_json::json!({"model": item.id}).to_string();
    super::command(
        "curl",
        &[
            "--noproxy",
            "*",
            "--connect-timeout",
            "1",
            "--max-time",
            "15",
            "--fail",
            "--silent",
            "--show-error",
            "--request",
            "DELETE",
            "--header",
            "Content-Type: application/json",
            "--data",
            &body,
            &format!("{HOST}/api/delete"),
        ],
    )
}

fn validate_removal(item: &Item, tags: &Value, running: &Value) -> Result<(), String> {
    if !running.get("models").is_some_and(Value::is_array) {
        return Err("Cannot verify running models".into());
    }
    let live = tags["models"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|model| super::text(model, "name") == item.id)
        .ok_or("Model is no longer present")?;
    let digest = super::text(live, "digest");
    if item.scope.as_deref() != Some(digest.as_str()) || digest.is_empty() {
        return Err("Model changed since confirmation; refresh and select again".into());
    }
    if is_loaded(running, &item.id, &digest) {
        return Err("Model is loaded; unload it explicitly before removal".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "cleanup_models_tests.rs"]
mod tests;
