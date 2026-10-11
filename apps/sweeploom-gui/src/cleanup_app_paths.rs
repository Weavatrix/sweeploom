use super::*;

pub(super) fn app_caches(home: &Path, items: &mut Vec<Item>, seen: &mut HashSet<PathBuf>) {
    let root = home.join("Library/Caches");
    match fs::read_dir(&root) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with('.')
                    || RULES.iter().any(|rule| {
                        let known = home.join(rule.path);
                        path.starts_with(&known) || known.starts_with(&path)
                    })
                {
                    continue;
                }
                let (policy, hint) = if name == "ms-playwright-mcp" {
                    (
                        Policy::Inspect,
                        "Automation browser profiles may contain sign-ins/site data; inspect in Finder",
                    )
                } else {
                    (
                        Policy::Trash,
                        "Application cache; close the app before removal; cached/offline data may need downloading again",
                    )
                };
                add(
                    home,
                    path,
                    &format!("Application cache · {name}"),
                    policy,
                    hint,
                    items,
                    seen,
                );
            }
        }
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            add_error(&root, "Application caches", error.to_string(), items, seen)
        }
        _ => {}
    }
    // Only cache subdirectories of browser/IDE/Electron profiles, never the profile itself.
    for (label, relative) in [
        ("Chrome", "Library/Application Support/Google/Chrome"),
        ("Edge", "Library/Application Support/Microsoft Edge"),
        (
            "Brave",
            "Library/Application Support/BraveSoftware/Brave-Browser",
        ),
        ("Chromium", "Library/Application Support/Chromium"),
        ("Arc", "Library/Application Support/Arc/User Data"),
        ("Vivaldi", "Library/Application Support/Vivaldi"),
    ] {
        let root = home.join(relative);
        if !safe_path(home, &root) {
            continue;
        }
        if let Ok(profiles) = fs::read_dir(&root) {
            for profile in profiles.flatten() {
                let name = profile.file_name().to_string_lossy().into_owned();
                if name == "Default" || name.starts_with("Profile ") {
                    cache_parts(
                        home,
                        &profile.path(),
                        &format!("{label} · {name}"),
                        items,
                        seen,
                    );
                }
            }
        }
    }
    for name in [
        "Code",
        "Cursor",
        "Codex",
        "Claude",
        "discord",
        "Slack",
        "Microsoft/Teams",
        "Spotify",
        "Zed",
    ] {
        cache_parts(
            home,
            &home.join("Library/Application Support").join(name),
            name,
            items,
            seen,
        );
    }
}

fn cache_parts(
    home: &Path,
    root: &Path,
    label: &str,
    items: &mut Vec<Item>,
    seen: &mut HashSet<PathBuf>,
) {
    for cache in [
        "Cache",
        "Code Cache",
        "GPUCache",
        "DawnCache",
        "CachedData",
        "CachedExtensionVSIXs",
        "CachedProfilesData",
    ] {
        add(
            home,
            root.join(cache),
            &format!("{label} · {cache}"),
            Policy::Cache,
            "Regenerable cache; close the application before cleanup",
            items,
            seen,
        );
    }
}

pub(super) fn downloads(home: &Path, items: &mut Vec<Item>, seen: &mut HashSet<PathBuf>) {
    let root = home.join("Downloads");
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            add_error(&root, "Downloads", error.to_string(), items, seen);
            return;
        }
        _ => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let extension = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if !["dmg", "pkg", "iso", "zip", "gz", "tgz", "tar", "xz", "7z"]
            .contains(&extension.as_str())
            && meta.len() < 100_000_000
        {
            continue;
        }
        let days = meta
            .modified()
            .ok()
            .and_then(|at| at.elapsed().ok())
            .map(|age| age.as_secs() / 86400);
        let hint = format!(
            "Downloaded file; {} days since modified; Move to Trash",
            days.map(|n| n.to_string())
                .unwrap_or_else(|| "unknown".into())
        );
        add(
            home,
            path.clone(),
            &format!(
                "Download · {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            Policy::Trash,
            &hint,
            items,
            seen,
        );
    }
}
