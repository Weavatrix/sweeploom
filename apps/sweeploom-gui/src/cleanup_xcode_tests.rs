use super::verdict::parse_date;
use super::*;

/// `plutil -convert binary1` of a dict with a long ASCII id, a UTF-16 name and a nested dict.
const INFO_BPLIST: &str = "62706c6973743030d401020304050608095f101a434642756e646c6553686f727456657273696f6e537472696e67564e65737465645f1012434642756e646c654964656e7469666965725c434642756e646c654e616d6553312e32d1010751395f1023636f6d2e6578616d706c652e4c6f6e6742756e646c654964656e7469666965724170706400430061006600e908112e354a575b5e60860000000000000101000000000000000a0000000000000000000000000000008f";

fn hex(text: &str) -> Vec<u8> {
    let digits: Vec<u8> = text.bytes().filter(u8::is_ascii_hexdigit).collect();
    digits
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

struct Home(PathBuf);
impl Home {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("sweeploom-xcode-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn file(&self, relative: &str, text: &str) -> PathBuf {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        path
    }
}
impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn plist_xml(pairs: &[(&str, &str)]) -> String {
    let body: String = pairs
        .iter()
        .map(|(key, value)| format!("\t<key>{key}</key>\n\t<string>{value}</string>\n"))
        .collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\">\n<dict>\n{body}</dict>\n</plist>\n"
    )
}

#[test]
fn binary_and_xml_plists_read_top_level_values_only() {
    let bytes = hex(INFO_BPLIST);
    assert_eq!(
        plist::value(&bytes, "CFBundleIdentifier").as_deref(),
        Some("com.example.LongBundleIdentifierApp")
    );
    assert_eq!(
        plist::value(&bytes, "CFBundleName").as_deref(),
        Some("Café")
    );
    assert_eq!(
        plist::value(&bytes, "CFBundleShortVersionString").as_deref(),
        Some("1.2")
    );
    assert_eq!(plist::value(&bytes, "Missing"), None);
    assert_eq!(plist::value(&bytes[..40], "CFBundleName"), None);
    let xml = "<plist><dict><key>Nested</key><dict><key>name</key><string>inner</string></dict>\
        <key>name</key><string>Tom &amp; Jerry</string><key>isDeleted</key><false/>\
        <key>lastUsedAt</key><date>2026-10-10T19:20:54Z</date><key>state</key><integer>1</integer></dict></plist>";
    assert_eq!(
        plist::value(xml.as_bytes(), "name").as_deref(),
        Some("Tom & Jerry")
    );
    assert_eq!(
        plist::value(xml.as_bytes(), "isDeleted").as_deref(),
        Some("false")
    );
    assert_eq!(plist::value(xml.as_bytes(), "state").as_deref(), Some("1"));
    assert_eq!(
        plist::value(xml.as_bytes(), "lastUsedAt").and_then(|text| parse_date(&text)),
        Some(1_791_660_054)
    );
}

#[test]
fn bundle_ids_skip_build_setting_references() {
    let home = Home::new("bundles");
    let pbxproj = home.file(
        "dev/app/App.xcodeproj/project.pbxproj",
        "\t\tPRODUCT_BUNDLE_IDENTIFIER = com.example.app;\n\t\tPRODUCT_BUNDLE_IDENTIFIER = \"com.example.app.ui-tests\";\n\t\tPRODUCT_BUNDLE_IDENTIFIER = \"$(BASE).watch\";\n",
    );
    let yml = home.file(
        "dev/gen/project.yml",
        "name: GenApp\ntargets:\n  App:\n    settings:\n      PRODUCT_BUNDLE_IDENTIFIER: com.example.gen\n",
    );
    assert_eq!(
        bundle_ids(pbxproj.parent().unwrap()),
        vec!["com.example.app", "com.example.app.ui-tests"]
    );
    assert_eq!(bundle_ids(&yml), vec!["com.example.gen"]);
    let projects = Projects::load(&home.0, &[]);
    let (name, verdict, _) = projects.app("com.example.gen").unwrap();
    assert_eq!((name.as_str(), verdict), ("GenApp", Verdict::Keep));
    assert!(projects.app("com.example.unknown").is_none());
}

#[test]
fn git_branch_follows_linked_worktrees() {
    let home = Home::new("branch");
    home.file("repo/.git/HEAD", "ref: refs/heads/feature/x\n");
    home.file("main/.git/worktrees/wt/HEAD", "0123456789abcdef\n");
    home.file(
        "wt/.git",
        &format!(
            "gitdir: {}\n",
            home.0.join("main/.git/worktrees/wt").display()
        ),
    );
    assert_eq!(
        git::branch(&home.0.join("repo")).as_deref(),
        Some("feature/x")
    );
    assert_eq!(
        git::branch(&home.0.join("wt")).as_deref(),
        Some("detached 01234567")
    );
    assert_eq!(
        git::repo_root(&home.0.join("repo/a/b")),
        Some(home.0.join("repo"))
    );
}

fn row(path: PathBuf) -> super::super::Item {
    super::super::Item {
        id: path.display().to_string(),
        name: "row".into(),
        kind: super::super::Kind::Cache,
        bytes: None,
        usage: None,
        status: String::new(),
        scope: None,
        path: Some(path),
        selected: false,
        enabled: false,
    }
}

#[test]
fn derived_data_and_device_support_rows_get_verdicts() {
    let home = Home::new("derived");
    let derived = "Library/Developer/Xcode/DerivedData";
    let here = home.file("dev/Here/Here.xcodeproj/project.pbxproj", "");
    let here = here.parent().unwrap().display().to_string();
    let gone = [("WorkspacePath", "/nonexistent/Gone.xcodeproj")];
    home.file(&format!("{derived}/Gone-abc/info.plist"), &plist_xml(&gone));
    home.file(
        &format!("{derived}/Here-def/info.plist"),
        &plist_xml(&[("WorkspacePath", &here)]),
    );
    home.file(&format!("{derived}/ModuleCache.noindex/x"), "");
    home.file(&format!("{derived}/Cli-ghi/Logs/x"), "");
    let support = "Library/Developer/Xcode/iOS DeviceSupport";
    home.file(&format!("{support}/iPhone14,4 17.5 (21F79)/x"), "");
    home.file(&format!("{support}/iPhone14,4 26.5 (23F77)/x"), "");
    let relative = [
        format!("{derived}/Gone-abc"),
        format!("{derived}/Here-def"),
        format!("{derived}/ModuleCache.noindex"),
        format!("{derived}/Cli-ghi"),
        format!("{support}/iPhone14,4 17.5 (21F79)"),
        format!("{support}/iPhone14,4 26.5 (23F77)"),
        ".npm/_cacache".to_owned(),
        format!("{derived}/Here-{}", hash::derived_suffix(&here)),
    ];
    fs::create_dir_all(home.0.join(&relative[7])).unwrap();
    let mut items: Vec<_> = relative.iter().map(|path| row(home.0.join(path))).collect();
    assert_eq!(enrich(&home.0, &mut items), vec![0, 1, 2, 3, 4, 5, 7]);
    assert_eq!(items[0].name, "Xcode DerivedData · Gone");
    assert!(
        items[0]
            .status
            .starts_with("Suggested · Project deleted — safe to remove")
    );
    assert!(
        items[1].status.starts_with("Keep · Project edited"),
        "{}",
        items[1].status
    );
    assert!(items[2].status.starts_with("Review · Shared module cache"));
    assert_eq!(items[3].name, "Xcode DerivedData · Cli");
    assert!(
        items[3]
            .status
            .starts_with("Review · No checkout on disk builds here")
    );
    assert!(
        items[7].status.starts_with("Keep · Project edited"),
        "{}",
        items[7].status
    );
    assert!(items[7].status.ends_with("(matched by folder name)"));
    assert!(
        items[4]
            .status
            .starts_with("Suggested · Superseded by 26.5 symbols")
    );
    assert!(
        items[5]
            .status
            .starts_with("Review · Newest symbols (26.5)")
    );
    assert!(items[6].status.is_empty());
}
