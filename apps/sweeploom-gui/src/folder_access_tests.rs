use super::*;

#[test]
fn grants_cover_nested_and_canonical_paths() {
    let root = std::env::temp_dir().join(format!("sweeploom-grant-{}", std::process::id()));
    let nested = root.join("app").join("target");
    std::fs::create_dir_all(&nested).unwrap();
    let canonical = sweeploom_storage::canonical_path(&root);
    assert!(covered(std::slice::from_ref(&root), &nested));
    assert!(covered(&[canonical], &nested));
    assert!(!covered(&[root.join("app").join("src")], &nested));
    assert!(!covered(&[], &nested));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn catalog_locations_map_to_the_folder_a_user_grants() {
    let home = Path::new("/Users/me");
    assert_eq!(grant_root(home, ".npm/_cacache"), home.join(".npm"));
    assert_eq!(
        grant_root(home, "Library/Caches/pip"),
        home.join("Library/Caches")
    );
    assert_eq!(
        grant_root(home, "Library/Developer/Xcode/DerivedData"),
        home.join("Library/Developer")
    );
    assert_eq!(
        grant_root(home, "Library/Application Support/MobileSync/Backup"),
        home.join("Library/Application Support/MobileSync")
    );
    assert_eq!(short(home, &home.join("dev")), "~/dev");
    assert_eq!(short(home, Path::new("/Volumes/x")), "/Volumes/x");
}

#[test]
fn bookmark_bytes_survive_the_json_hex_round_trip() {
    let bytes = vec![0_u8, 1, 0x7f, 0x80, 0xff];
    assert_eq!(unhex(&hex(&bytes)), Some(bytes));
    assert_eq!(unhex("abc"), None);
    assert_eq!(unhex("zz"), None);
}

#[test]
fn open_source_edition_needs_no_grants() {
    let access = FolderAccess::load(std::env::temp_dir().join("sweeploom-no-grants.json"));
    assert!(access.grants().is_empty());
    assert!(allowed(Path::new("/anywhere")));
}
