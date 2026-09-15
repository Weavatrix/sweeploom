use super::*;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("sweeploom-host-setup-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn saved_id_roundtrips() {
    let dir = temp_dir("id");
    assert_eq!(load_chromium_id(&dir).unwrap(), None);
    save_chromium_id(&dir, "abcdefghijklmnopabcdefghijklmnop").unwrap();
    assert_eq!(
        load_chromium_id(&dir).unwrap().as_deref(),
        Some("abcdefghijklmnopabcdefghijklmnop")
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn flag_wins_over_saved_id() {
    let dir = temp_dir("resolve");
    save_chromium_id(&dir, "abcdefghijklmnopabcdefghijklmnop").unwrap();
    let id = resolve_chromium_id(Some("ponmlkjihgfedcbaponmlkjihgfedcba"), &dir).unwrap();
    assert_eq!(id.as_deref(), Some("ponmlkjihgfedcbaponmlkjihgfedcba"));
    let saved = resolve_chromium_id(None, &dir).unwrap();
    assert_eq!(saved.as_deref(), Some("abcdefghijklmnopabcdefghijklmnop"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn write_without_id_skips_chromium() {
    let dir = temp_dir("ff-only");
    let host = dir.join("sweeploom-companion-host");
    let written = write_native_hosts(&dir, &host, None).unwrap();
    assert!(written.firefox_manifest.is_file());
    assert!(written.chromium_manifest.is_none());
    assert!(
        !dir.join("native-messaging")
            .join(format!("{HOST_NAME}.chromium.json"))
            .is_file()
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn write_with_id_includes_edge_origin() {
    let dir = temp_dir("edge");
    let host = dir.join("sweeploom-companion-host.exe");
    let written =
        write_native_hosts(&dir, &host, Some("abcdefghijklmnopabcdefghijklmnop")).unwrap();
    let json = fs::read_to_string(written.chromium_manifest.unwrap()).unwrap();
    assert!(json.contains("chrome-extension://abcdefghijklmnopabcdefghijklmnop/"));
    assert!(json.contains("sweeploom-companion-host.exe"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn reject_bad_id() {
    let dir = temp_dir("bad");
    assert!(save_chromium_id(&dir, "not-an-id").is_err());
    assert!(write_native_hosts(&dir, Path::new("host"), Some("nope")).is_err());
    let _ = fs::remove_dir_all(&dir);
}
