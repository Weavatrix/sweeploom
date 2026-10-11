use super::*;
use std::fs;

const A: &str = "33D4F1B4-E53A-43B4-A6DF-806BC8762660";
const B: &str = "8C67417C-9AC7-4527-BF41-6D00E2A93038";
const C: &str = "47B77C6B-6CF6-41C3-8984-CEE09FD19F3D";
const D: &str = "04B1C95C-9FA9-4D1D-8688-21EF87234AAC";
const E: &str = "A2D5FDAF-57DB-4409-AB34-2578A89B203F";

struct Home(PathBuf);
impl Home {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("sweeploom-ios-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn file(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn device(&self, set: &str, udid: &str, extra: &str) {
        self.file(
            &format!("{set}/{udid}/device.plist"),
            &format!(
                "<?xml version=\"1.0\"?>\n<plist version=\"1.0\">\n<dict>\n\t<key>UDID</key>\n\t<string>{udid}</string>\n\t<key>lastUsedAt</key>\n\t<date>2026-01-02T03:04:05Z</date>\n\t<key>name</key>\n\t<string>Phone {udid}</string>\n\t<key>runtime</key>\n\t<string>com.apple.CoreSimulator.SimRuntime.iOS-26-5</string>\n{extra}</dict>\n</plist>\n"
            ),
        );
    }
}
impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const DEVICES: &str = "Library/Developer/CoreSimulator/Devices";
const PREVIEWS: &str = "Library/Developer/Xcode/UserData/Previews/Simulator Devices";

fn ours<'a>(home: &Home, items: &'a [Item]) -> Vec<&'a Item> {
    items
        .iter()
        .filter(|item| {
            item.path
                .as_ref()
                .is_none_or(|path| path.starts_with(&home.0))
        })
        .collect()
}

#[test]
fn simctl_json_keeps_unavailable_and_last_boot_facts() {
    let value = serde_json::json!({"devices": {
        "com.apple.CoreSimulator.SimRuntime.iOS-17-0": [
            {"name": "Old", "udid": A, "state": "Shutdown", "isAvailable": false,
             "availabilityError": "runtime profile not found", "lastBootedAt": "2026-01-02T03:04:05Z",
             "dataPath": "/tmp/a/data", "dataPathSize": 20}
        ],
        "com.apple.CoreSimulator.SimRuntime.watchOS-26-5": [
            {"name": "Watch", "udid": C, "state": "Booted", "isAvailable": true, "lastUsedAt": "2026-10-10T19:20:54Z"}
        ]
    }});
    let found = sims(&value);
    assert_eq!(found.len(), 2);
    let old = found.iter().find(|sim| sim.udid == A).unwrap();
    assert_eq!(
        old.unavailable.as_deref(),
        Some("runtime profile not found")
    );
    assert_eq!(old.last_used, verdict::parse_date("2026-01-02T03:04:05Z"));
    let rows = parse_devices(&value);
    let watch = rows.iter().find(|row| row.id == C).unwrap();
    assert!(!watch.enabled);
    assert!(watch.status.starts_with("Booted · watchOS 26.5"));
    assert_eq!(
        pretty_runtime("com.apple.CoreSimulator.SimRuntime.xrOS-2-0"),
        "xrOS 2.0"
    );
}

#[test]
fn disk_finds_devices_leftovers_alternate_sets_and_caches() {
    let home = Home::new("disk");
    home.device(DEVICES, A, "");
    home.file(&format!("{DEVICES}/{B}/data/x"), "leftover");
    home.file(&format!("{DEVICES}/device_set.plist"), "<plist/>");
    home.device("Library/Developer/XCTestDevices", C, "");
    home.device(PREVIEWS, D, "\t<key>isDeleted</key>\n\t<true/>\n");
    home.file(
        "Library/Developer/CoreSimulator/Caches/dyld/OLD1/x",
        "cache",
    );
    home.file(
        "Library/Developer/CoreSimulator/Caches/Personalization/x",
        "keep",
    );
    let (items, found) = disk::discover(&home.0, Some("NEW2"));
    let items = ours(&home, &items);
    assert_eq!(found.len(), 2);
    let device = items.iter().find(|item| item.id == A).unwrap();
    assert_eq!((device.kind.clone(), device.enabled), (Kind::Device, false));
    assert!(device.status.starts_with("State unknown"));
    let clone = items.iter().find(|item| item.id == C).unwrap();
    assert!(clone.name.starts_with("XCTest clone · "));
    assert!(
        clone
            .scope
            .as_deref()
            .is_some_and(|scope| scope.ends_with("XCTestDevices"))
    );
    let leftover = items
        .iter()
        .find(|item| item.id == format!("orphan:{B}"))
        .unwrap();
    assert_eq!(leftover.kind, Kind::SimFiles);
    assert!(leftover.status.starts_with("Suggested · "));
    let deleted = items
        .iter()
        .find(|item| item.id == format!("orphan:{D}"))
        .unwrap();
    assert!(
        deleted
            .scope
            .as_deref()
            .is_some_and(|scope| scope.ends_with("Simulator Devices"))
    );
    let dyld = items
        .iter()
        .find(|item| item.name.ends_with("dyld cache · OLD1"))
        .unwrap();
    assert_eq!(dyld.kind, Kind::SimFiles);
    assert!(dyld.status.contains("stale") && valid_id(&dyld.id));
    let managed = items
        .iter()
        .find(|item| item.name.ends_with("Personalization"))
        .unwrap();
    assert_eq!(
        (managed.kind.clone(), managed.enabled),
        (Kind::Inspect, false)
    );
}

#[test]
fn simctl_merge_turns_unlisted_folders_into_leftovers_and_offers_unavailable_cleanup() {
    let home = Home::new("merge");
    home.device(DEVICES, A, "");
    home.device(DEVICES, E, "");
    let (mut items, found) = disk::discover(&home.0, None);
    let mut facts: HashMap<String, Sim> = found
        .into_iter()
        .map(|sim| (sim.udid.clone(), sim))
        .collect();
    let value = serde_json::json!({"devices": {"com.apple.CoreSimulator.SimRuntime.iOS-17-0": [
        {"name": "Phone", "udid": A, "state": "Shutdown", "isAvailable": false, "dataPathSize": 7,
         "dataPath": home.0.join(DEVICES).join(A).join("data")}
    ]}});
    let set = disk::sets(&home.0).remove(0);
    let mut loading = items.clone();
    let mut loading_facts = facts.clone();
    let installed = HashSet::from(["com.apple.CoreSimulator.SimRuntime.iOS-17-0".to_owned()]);
    merge::merge(
        &mut loading,
        &mut loading_facts,
        &set,
        sims(&value),
        Some(&installed),
    );
    assert!(!loading.iter().any(|item| item.kind == Kind::SimUnavailable));
    assert!(loading_facts[A].loading);
    let status = apps::status(&loading_facts[A], &[], verdict::now());
    assert!(status.starts_with("Review · iOS 17.0 is installed but not loaded"));
    merge::merge(
        &mut items,
        &mut facts,
        &set,
        sims(&value),
        Some(&HashSet::new()),
    );
    let device = items.iter().find(|item| item.id == A).unwrap();
    assert!(device.enabled && device.bytes == Some(7));
    assert!(
        items
            .iter()
            .any(|item| item.id == format!("orphan:{E}") && item.kind == Kind::SimFiles)
    );
    let unavailable = items
        .iter()
        .find(|item| item.kind == Kind::SimUnavailable)
        .unwrap();
    assert_eq!(unavailable.id, "unavailable:Simulator:1");
    assert!(
        valid_id(&unavailable.id) && unavailable.scope.is_none() && unavailable.bytes == Some(7)
    );
    assert!(!facts.contains_key(E));
    let now = verdict::now();
    let status = apps::status(&facts[A], &[], now);
    assert!(status.starts_with("Suggested · Unavailable runtime (iOS 17.0 not installed)"));
}

#[test]
fn installed_apps_come_from_bundle_info_plists() {
    let home = Home::new("apps");
    let bundle = format!("{DEVICES}/{A}/data/Containers/Bundle/Application/X");
    home.file(
        &format!("{bundle}/Demo.app/Info.plist"),
        "<plist><dict><key>CFBundleIdentifier</key><string>com.example.demo</string></dict></plist>",
    );
    home.file(&format!("{bundle}/BundleMetadata.plist"), "<plist/>");
    let data = home.0.join(DEVICES).join(A).join("data");
    assert_eq!(apps::installed(&data), vec!["com.example.demo"]);
    let sim = Sim {
        state: "Shutdown".into(),
        runtime: "com.apple.CoreSimulator.SimRuntime.iOS-26-5".into(),
        last_used: Some(verdict::now() - 3600),
        ..Sim::default()
    };
    let evidence = [verdict::AppEvidence {
        bundle: "com.example.demo".into(),
        project: Some((
            "Demo".into(),
            Verdict::Keep,
            "Active project (commit 2h ago) — keep".into(),
        )),
    }];
    let status = apps::status(&sim, &evidence, verdict::now());
    assert_eq!(
        status,
        "Keep · Has Demo: Active project (commit 2h ago) — keep · iOS 26.5 · Shutdown"
    );
}

#[test]
fn runtimes_without_simulators_are_suggested() {
    let value = serde_json::json!({
        "R1": {"runtimeIdentifier": "com.apple.CoreSimulator.SimRuntime.iOS-18-2", "build": "22C150",
               "deletable": true, "sizeBytes": 8_706_942_096_u64},
        "R2": {"runtimeIdentifier": "com.apple.CoreSimulator.SimRuntime.iOS-26-5", "build": "23F77",
               "deletable": true, "lastUsedAt": "2026-10-10T19:21:33Z"},
        "R3": {"runtimeIdentifier": "com.apple.CoreSimulator.SimRuntime.iOS-26-0", "build": "23A1",
               "deletable": false}
    });
    let facts = HashMap::from([(
        A.to_owned(),
        Sim {
            runtime: "com.apple.CoreSimulator.SimRuntime.iOS-26-5".into(),
            ..Sim::default()
        },
    )]);
    let rows = runtimes::rows(&value, &facts, verdict::now());
    let row = |id: &str| rows.iter().find(|row| row.id == id).unwrap();
    assert_eq!(row("R1").name, "Runtime: iOS 18.2 (22C150)");
    assert!(
        row("R1")
            .status
            .starts_with("Suggested · No simulators use iOS 18.2")
    );
    assert_eq!(row("R1").bytes, Some(8_706_942_096));
    assert!(
        row("R2")
            .status
            .starts_with("Review · 1 simulators use iOS 26.5")
    );
    assert!(!row("R3").enabled);
    let home = Home::new("images");
    home.file("bundle/SimRuntimeBundle-A/Restore/a.dmg", "image");
    home.file("bundle/SimRuntimeBundle-B/Restore/b.dmg", "image");
    fs::create_dir_all(home.0.join("bundle/SimRuntimeBundle-C")).unwrap();
    let known = [home.0.join("bundle/SimRuntimeBundle-A/Restore/a.dmg")];
    let leftovers = runtimes::unregistered(&home.0.join("bundle"), &known);
    assert_eq!(leftovers.len(), 1);
    assert_eq!(
        leftovers[0].name,
        "Unregistered runtime image · SimRuntimeBundle-B"
    );
    assert!(!leftovers[0].enabled && valid_id(&leftovers[0].id));
}
