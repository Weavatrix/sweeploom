//! Simulator process trees: each `launchd_sim` is one booted simulated device.
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock, PoisonError},
};
use sweeploom_core::ProcessSnapshot;

const DEVICE_TIP: &str = "iOS Simulator: launchd_sim is the simulated device's launchd. Every simulated system service and every app running in that device is its child, so this tree's memory is the whole simulated device. Shut it down in Simulator.app or with `xcrun simctl shutdown <UDID>`.";
const APP_TIP: &str = "iOS Simulator: Xcode's Simulator.app window host and CoreSimulator services. Booted devices appear as separate launchd_sim trees.";

pub(super) struct Simulator {
    pub title: String,
    pub project: String,
    pub tip: &'static str,
}

pub(super) fn describe(process: &ProcessSnapshot) -> Option<Simulator> {
    let launchd = process.name == "launchd_sim"
        || process
            .exe
            .as_deref()
            .is_some_and(|exe| exe.ends_with("launchd_sim"));
    if launchd {
        let device = process
            .command
            .iter()
            .find_map(|arg| device_dir(Path::new(arg)))
            .and_then(|dir| device(&dir));
        return Some(match device {
            Some((name, runtime)) => Simulator {
                title: format!("{} Simulator · {name}", platform(&runtime)),
                project: if runtime.is_empty() {
                    "Simulated device".into()
                } else {
                    format!("{} simulator", pretty(&runtime))
                },
                tip: DEVICE_TIP,
            },
            None => Simulator {
                title: "iOS Simulator".into(),
                project: "Simulated device".into(),
                tip: DEVICE_TIP,
            },
        });
    }
    let exe = process.exe.as_deref()?.to_string_lossy();
    let title = if exe.contains("/Simulator.app/") {
        "iOS Simulator app"
    } else if exe.contains("/CoreSimulator.framework/") {
        "iOS Simulator service"
    } else {
        return None;
    };
    Some(Simulator {
        title: title.into(),
        project: "Xcode".into(),
        tip: APP_TIP,
    })
}

fn is_udid(name: &str) -> bool {
    name.len() == 36
        && name.char_indices().all(|(index, ch)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                ch == '-'
            } else {
                ch.is_ascii_hexdigit()
            }
        })
}

/// `…/Devices/<UDID>/data/var/run/launchd_bootstrap.plist` → `…/Devices/<UDID>`.
pub(super) fn device_dir(arg: &Path) -> Option<PathBuf> {
    arg.ancestors()
        .find(|dir| {
            dir.file_name()
                .is_some_and(|name| is_udid(&name.to_string_lossy()))
        })
        .map(Path::to_path_buf)
}

/// Device folder → (name, runtime identifier) from its device.plist.
type Names = HashMap<PathBuf, Option<(String, String)>>;

/// Name and runtime from device.plist, read once per device folder.
fn device(dir: &Path) -> Option<(String, String)> {
    static CACHE: OnceLock<Mutex<Names>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    cache
        .entry(dir.to_owned())
        .or_insert_with(|| {
            let text = std::fs::read_to_string(dir.join("device.plist")).ok()?;
            Some((
                value(&text, "name")?,
                value(&text, "runtime").unwrap_or_default(),
            ))
        })
        .clone()
}

pub(super) fn value(text: &str, key: &str) -> Option<String> {
    let rest = &text[text.find(&format!("<key>{key}</key>"))?..];
    let rest = &rest[rest.find("<string>")? + 8..];
    Some(rest[..rest.find("</string>")?].replace("&amp;", "&"))
}

/// `com.apple.CoreSimulator.SimRuntime.watchOS-26-5` → `watchOS 26.5`.
pub(super) fn pretty(runtime: &str) -> String {
    let tail = runtime.rsplit('.').next().unwrap_or(runtime);
    tail.split_once('-').map_or_else(
        || tail.to_owned(),
        |(_, version)| format!("{} {}", platform(runtime), version.replace('-', ".")),
    )
}

fn platform(runtime: &str) -> &'static str {
    let tail = runtime.rsplit('.').next().unwrap_or(runtime);
    match tail.split('-').next().unwrap_or_default() {
        "watchOS" => "watchOS",
        "tvOS" => "tvOS",
        "xrOS" | "visionOS" => "visionOS",
        _ => "iOS",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launchd_sim_bootstrap_path_names_the_simulated_device() {
        let arg = "/Users/me/Library/Developer/CoreSimulator/Devices/41E3E880-A5C9-4633-9DFA-B6256A422B2C/data/var/run/launchd_bootstrap.plist";
        assert_eq!(
            device_dir(Path::new(arg)),
            Some(PathBuf::from(
                "/Users/me/Library/Developer/CoreSimulator/Devices/41E3E880-A5C9-4633-9DFA-B6256A422B2C"
            ))
        );
        assert_eq!(device_dir(Path::new("/usr/bin/launchd_sim")), None);
        let plist = "<dict>\n\t<key>name</key>\n\t<string>Echo &amp; Shots</string>\n\t<key>runtime</key>\n\t<string>com.apple.CoreSimulator.SimRuntime.watchOS-26-5</string>\n</dict>";
        assert_eq!(value(plist, "name").as_deref(), Some("Echo & Shots"));
        assert_eq!(
            pretty("com.apple.CoreSimulator.SimRuntime.watchOS-26-5"),
            "watchOS 26.5"
        );
        assert_eq!(
            platform("com.apple.CoreSimulator.SimRuntime.xrOS-2-0"),
            "visionOS"
        );
    }

    #[test]
    fn simulator_processes_are_labelled_even_without_device_evidence() {
        let mut process = ProcessSnapshot {
            key: sweeploom_core::ProcessKey::new(7, None),
            pid: 7,
            parent: None,
            name: "launchd_sim".into(),
            exe: None,
            cwd: Some(PathBuf::from("/")),
            command: vec!["launchd_sim".into()],
            started_at: None,
            runtime: std::time::Duration::ZERO,
            rss_bytes: 0,
            virtual_bytes: 0,
            cpu_percent: 0.0,
            accumulated_cpu_ms: 0,
            disk_read_delta: 0,
            disk_write_delta: 0,
            network: sweeploom_core::NetworkSnapshot::default(),
            project: None,
            session: None,
            safety_class: sweeploom_core::ProcessSafetyClass::Unknown,
        };
        let sim = describe(&process).unwrap();
        assert_eq!(
            (sim.title.as_str(), sim.project.as_str()),
            ("iOS Simulator", "Simulated device")
        );
        process.name = "Simulator".into();
        process.exe = Some(PathBuf::from(
            "/Applications/Xcode.app/Contents/Developer/Applications/Simulator.app/Contents/MacOS/Simulator",
        ));
        assert_eq!(describe(&process).unwrap().title, "iOS Simulator app");
        process.exe = Some(PathBuf::from("/usr/bin/node"));
        assert!(describe(&process).is_none());
    }
}
