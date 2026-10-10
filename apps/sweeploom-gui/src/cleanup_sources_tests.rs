use super::*;

struct Home(PathBuf);
impl Home {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("sweeploom-cleanup-{name}-{}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn file(&self, relative: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        path
    }
}
impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn package_versions_and_projects_can_be_chosen_individually() {
    let home = Home::new("versions");
    home.file(".nuget/packages/example/1.0/package.dll", b"one");
    home.file(".nuget/packages/example/2.0/package.dll", b"two");
    home.file(".npm/_npx/abc/node_modules/tool/main.js", b"tool");
    home.file(".npm/_npx/def/node_modules/tool/main.js", b"other tool");
    home.file(
        "Library/Developer/Xcode/DerivedData/Project-one/build.o",
        b"build",
    );
    let listing = discover(&home.0, Pane::Caches);
    assert_eq!(listing.items.len(), 5);
    assert!(
        listing
            .items
            .iter()
            .all(|item| !item.selected && item.bytes.is_none())
    );
    assert!(
        listing
            .items
            .iter()
            .any(|item| item.path.as_ref() == Some(&home.0.join(".nuget/packages/example/1.0")))
    );
    assert!(
        !listing
            .items
            .iter()
            .any(|item| item.path.as_ref() == Some(&home.0.join(".nuget/packages")))
    );
}

#[test]
fn browser_profiles_credentials_and_environments_are_not_cache_targets() {
    let home = Home::new("profiles");
    let cache = home.file(
        "Library/Application Support/Google/Chrome/Default/Cache/data",
        b"cache",
    );
    let login = home.file(
        "Library/Application Support/Google/Chrome/Default/Login Data",
        b"credentials",
    );
    let bookmarks = home.file(
        "Library/Application Support/Google/Chrome/Default/Bookmarks",
        b"bookmarks",
    );
    home.file(
        "Library/Caches/ms-playwright-mcp/mcp-chrome/Login Data",
        b"profile",
    );
    home.file(
        "Library/Caches/pypoetry/virtualenvs/project/bin/python",
        b"runtime",
    );
    home.file(
        "Library/Caches/ms-playwright/daemon/session/data",
        b"persistent session",
    );
    home.file(
        "Library/Caches/ms-playwright/chromium-123/browser",
        b"download",
    );
    let listing = discover(&home.0, Pane::Apps);
    assert!(listing.items.iter().any(|item| {
        item.path
            .as_ref()
            .is_some_and(|path| cache.starts_with(path))
            && item.kind == Kind::Cache
    }));
    assert!(!listing.items.iter().any(|item| {
        item.path
            .as_ref()
            .is_some_and(|path| login.starts_with(path) || bookmarks.starts_with(path))
    }));
    assert!(
        listing
            .items
            .iter()
            .filter(|item| item.name.contains("ms-playwright-mcp"))
            .all(|item| item.kind == Kind::Inspect)
    );
    let packages = discover(&home.0, Pane::Caches);
    assert!(
        !packages
            .items
            .iter()
            .any(|item| item.id.contains("virtualenvs") || item.id.contains("daemon"))
    );
    assert!(
        packages
            .items
            .iter()
            .any(|item| item.id.contains("chromium-123"))
    );
}

#[test]
fn archives_backups_models_and_downloads_only_offer_recoverable_removal() {
    let home = Home::new("data");
    home.file(
        "Library/Developer/Xcode/Archives/2026-01-01/App.xcarchive/dSYMs/release",
        b"debug symbols",
    );
    home.file(
        "Library/Application Support/MobileSync/Backup/device/backup",
        b"backup",
    );
    home.file("Downloads/installer.dmg", b"installer");
    home.file("Downloads/small.txt", b"personal file");
    home.file(
        ".cache/huggingface/hub/models--team--model/blobs/model",
        b"model",
    );
    let listing = discover(&home.0, Pane::Data);
    assert_eq!(listing.items.len(), 3);
    assert!(listing.items.iter().all(|item| item.kind == Kind::Trash));
    let models = discover(&home.0, Pane::Models);
    assert_eq!(models.items.len(), 1);
    assert_eq!(models.items[0].kind, Kind::Trash);
}

#[test]
fn discovery_streams_targets_before_sizes_and_files_use_logical_size_for_deletion() {
    let home = Home::new("stream");
    let file = home.file(
        "Library/Caches/Homebrew/downloads/installer.tar.gz",
        b"seven!!",
    );
    let (tx, rx) = crossbeam_channel::unbounded();
    stream(&home.0, Pane::Caches, &tx);
    let ListingMsg::Initial(initial) = rx.recv().unwrap() else {
        panic!("targets must arrive first");
    };
    assert_eq!(initial.items.len(), 1);
    assert_eq!(initial.items[0].bytes, None);
    let ListingMsg::Measured(item) = rx.recv().unwrap() else {
        panic!("expected full measurement");
    };
    assert!(item.enabled);
    let candidate = super::super::cache_candidate(&item, 0).unwrap();
    assert_eq!(candidate.logical_bytes, 7);
    let ctx = sweeploom_core::ExecutionContext::observed(&[]);
    let plan = sweeploom_exec::build_plan_with(&[candidate], None, &ctx);
    let (report, _) = sweeploom_exec::apply_plan_with(&plan, &ctx);
    assert_eq!(report.counts.deleted, 1);
    assert!(!file.exists());
    assert!(matches!(rx.recv().unwrap(), ListingMsg::Done));
}

#[cfg(unix)]
#[test]
fn symlinked_cache_roots_cannot_escape_home() {
    use std::os::unix::fs::symlink;
    let home = Home::new("symlink");
    let outside = Home::new("outside");
    outside.file("_cacache/important", b"outside");
    symlink(&outside.0, home.0.join(".npm")).unwrap();
    let listing = discover(&home.0, Pane::Caches);
    assert!(listing.items.is_empty());
    assert!(outside.0.join("_cacache/important").exists());
}

#[cfg(unix)]
#[test]
fn sparse_downloads_show_allocated_size_and_never_an_invented_zero() {
    let home = Home::new("sparse");
    let path = home.file("Downloads/disk.iso", b"data");
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(1_000_000_000)
        .unwrap();
    let mut listing = discover(&home.0, Pane::Data);
    assert_eq!(listing.items.len(), 1);
    measure(&mut listing.items[0]);
    let usage = listing.items[0].usage.unwrap();
    assert_eq!(usage.logical_bytes, 1_000_000_000);
    assert!(usage.bytes > 0 && usage.bytes < usage.logical_bytes);
    let mut missing = listing.items[0].clone();
    missing.path = Some(home.0.join("missing"));
    missing.bytes = None;
    measure(&mut missing);
    assert_eq!(missing.bytes, None);
    assert!(missing.status.starts_with("Cannot inspect"));
}
