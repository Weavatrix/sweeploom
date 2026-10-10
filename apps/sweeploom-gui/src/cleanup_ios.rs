//! iOS, watchOS, tvOS and visionOS simulators: disk first, then simctl, apps and Git evidence.
use super::*;
use crossbeam_channel::Sender;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    process::Command,
    time::Duration,
};
#[path = "cleanup_ios_apps.rs"]
mod apps;
#[path = "cleanup_ios_disk.rs"]
mod disk;
#[path = "cleanup_ios_merge.rs"]
mod merge;
#[path = "cleanup_ios_apply.rs"]
mod remove;
#[path = "cleanup_ios_runtimes.rs"]
mod runtimes;
use merge::confirm;
pub(super) use remove::{remove_device, remove_files, remove_unavailable};
use xcode::verdict::{self, Verdict};

/// simctl can stall for a minute while CoreSimulator boots or migrates devices.
const LIST_TIMEOUT: Duration = Duration::from_secs(90);
const NOTE: &str = "Devices are read from disk first, then confirmed with simctl. Suggestions use last boot, installed apps and their Xcode projects' Git state; nothing is selected automatically. Root-owned runtime images and dyld caches are shown read-only.";

/// One simulator device from device.plist and simctl.
#[derive(Clone, Debug, Default)]
pub(super) struct Sim {
    pub udid: String,
    pub name: String,
    /// Runtime identifier, e.g. `com.apple.CoreSimulator.SimRuntime.iOS-26-5`.
    pub runtime: String,
    /// simctl state; empty until simctl answers.
    pub state: String,
    pub unavailable: Option<String>,
    /// Runtime is installed but CoreSimulator has not loaded it yet: unavailable only for now.
    pub loading: bool,
    pub last_used: Option<i64>,
    pub bytes: Option<u64>,
    pub data: Option<PathBuf>,
}

pub(super) fn simctl(
    scope: Option<&str>,
    args: &[&str],
    timeout: Duration,
) -> Result<String, String> {
    let mut command = Command::new("/usr/bin/xcrun");
    command.arg("simctl");
    if let Some(scope) = scope {
        command.args(["--set", scope]);
    }
    command.args(args);
    commands::execute_for("xcrun simctl", command, timeout)
}

pub(super) fn list(scope: Option<&str>) -> Result<Value, String> {
    simctl(scope, &["list", "devices", "--json"], LIST_TIMEOUT)
        .and_then(|text| serde_json::from_str(&text).map_err(|e| e.to_string()))
}

/// `com.apple.CoreSimulator.SimRuntime.iOS-26-5` → `iOS 26.5`.
fn pretty_runtime(id: &str) -> String {
    let tail = id.rsplit('.').next().unwrap_or(id);
    match tail.split_once('-') {
        Some((os, version)) => format!("{os} {}", version.replace('-', ".")),
        None => tail.to_owned(),
    }
}

fn sims(value: &Value) -> Vec<Sim> {
    let mut out = Vec::new();
    for (runtime, devices) in value
        .get("devices")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        for device in devices.as_array().into_iter().flatten() {
            let unavailable = (device.get("isAvailable").and_then(Value::as_bool) == Some(false))
                .then(|| match text(device, "availabilityError") {
                    error if error.is_empty() => {
                        format!("{} not installed", pretty_runtime(runtime))
                    }
                    error => error,
                });
            out.push(Sim {
                udid: text(device, "udid"),
                name: text(device, "name"),
                runtime: runtime.clone(),
                state: text(device, "state"),
                unavailable,
                loading: false,
                last_used: ["lastBootedAt", "lastUsedAt"].iter().find_map(|key| {
                    device
                        .get(*key)
                        .and_then(Value::as_str)
                        .and_then(verdict::parse_date)
                }),
                bytes: value_bytes(device, "dataPathSize"),
                data: device
                    .get("dataPath")
                    .and_then(Value::as_str)
                    .map(PathBuf::from),
            });
        }
    }
    out
}

fn item(sim: &Sim, set: &disk::Set) -> Item {
    let state = if sim.state.is_empty() {
        "State unknown until simctl answers"
    } else {
        sim.state.as_str()
    };
    Item {
        id: sim.udid.clone(),
        name: if set.scope.is_some() {
            format!("{} · {}", set.label, sim.name)
        } else {
            sim.name.clone()
        },
        kind: Kind::Device,
        bytes: sim.bytes,
        usage: None,
        status: format!(
            "{state} · {} · delete device and data",
            pretty_runtime(&sim.runtime)
        ),
        scope: set.scope.clone(),
        path: sim.data.clone(),
        selected: false,
        enabled: sim.state == "Shutdown",
    }
}

pub(super) fn parse_devices(value: &Value) -> Vec<Item> {
    let set = disk::Set {
        label: "Simulator",
        path: PathBuf::new(),
        scope: None,
    };
    sims(value).iter().map(|sim| item(sim, &set)).collect()
}

/// Synchronous form for the job runner: the whole stream folded into one listing.
pub(super) fn ios_listing() -> Listing {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let (tx, rx) = crossbeam_channel::unbounded();
    stream(&home, &tx);
    drop(tx);
    let mut listing = Listing::default();
    for message in rx.try_iter() {
        match message {
            ListingMsg::Initial(next) => listing = next,
            ListingMsg::Measured(item) => {
                if let Some(old) = listing.items.iter_mut().find(|old| old.id == item.id) {
                    *old = item;
                }
            }
            ListingMsg::Failed(error) => listing.note = error,
            ListingMsg::Done => {}
        }
    }
    listing
}

/// Rows from disk at once, simctl confirmation next, then app/Git verdicts and sizes.
pub(super) fn stream(home: &Path, tx: &Sender<ListingMsg>) {
    let host = command("/usr/sbin/sysctl", &["-n", "kern.osversion"])
        .ok()
        .map(|text| text.trim().to_owned());
    let (items, found) = disk::discover(home, host.as_deref());
    let mut facts: HashMap<String, Sim> = found
        .into_iter()
        .map(|sim| (sim.udid.clone(), sim))
        .collect();
    let mut listing = Listing {
        items,
        note: NOTE.into(),
        complete: false,
    };
    if tx.send(ListingMsg::Initial(listing.clone())).is_err() {
        return;
    }
    for note in confirm(home, &mut listing.items, &mut facts) {
        listing.note.push_str(&format!("\n{note}"));
    }
    if tx.send(ListingMsg::Initial(listing.clone())).is_err() {
        return;
    }
    for index in apps::enrich(home, &mut listing.items, &facts) {
        if tx
            .send(ListingMsg::Measured(listing.items[index].clone()))
            .is_err()
        {
            return;
        }
    }
    for item in &mut listing.items {
        if item.bytes.is_none() && item.path.is_some() && !item.status.starts_with("Cannot inspect")
        {
            disk::measure(item);
            if tx.send(ListingMsg::Measured(item.clone())).is_err() {
                return;
            }
        }
    }
    let _ = tx.send(ListingMsg::Done);
}

#[cfg(test)]
#[path = "cleanup_ios_tests.rs"]
mod tests;
