//! Storage inventory and explicit cleanup for native tools, caches and archives.
#[path = "cleanup_apply.rs"]
mod apply;
#[path = "cleanup_commands.rs"]
mod commands;
#[path = "cleanup_docker.rs"]
mod docker;
#[path = "cleanup_ios.rs"]
mod ios;
#[path = "cleanup_jobs.rs"]
mod jobs;
#[path = "cleanup_models.rs"]
mod models;
#[path = "cleanup_sources.rs"]
mod sources;
#[path = "cleanup_state.rs"]
mod state;
#[path = "cleanup_toolchains.rs"]
mod toolchains;
#[path = "cleanup_ui.rs"]
mod view;
#[path = "cleanup_xcode.rs"]
mod xcode;
use crate::{app::SweepLoomApp, sort::Sort};
use apply::{apply_native, cache_candidate};
use commands::{allocated, command, text, valid_id, value_bytes, value_count};
use crossbeam_channel::Receiver;
use docker::{docker_command, docker_listing};
use ios::parse_devices;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Pane {
    Docker,
    Ios,
    Caches,
    Apps,
    Models,
    Data,
}
impl Pane {
    const ALL: [Self; 6] = [
        Self::Docker,
        Self::Ios,
        Self::Caches,
        Self::Apps,
        Self::Models,
        Self::Data,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Docker => "Docker",
            Self::Ios => "iOS Simulator",
            Self::Caches => "Build & packages",
            Self::Apps => "App & browser caches",
            Self::Models => "AI models",
            Self::Data => "Archives & large data",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Image,
    Container,
    Volume,
    BuildCache,
    Device,
    Runtime,
    SimFiles,
    SimUnavailable,
    Cache,
    Trash,
    Model,
    Toolchain,
    Inspect,
}
#[derive(Clone, Debug)]
struct Item {
    id: String,
    name: String,
    kind: Kind,
    bytes: Option<u64>,
    usage: Option<sweeploom_storage::DiskUsage>,
    status: String,
    scope: Option<String>,
    path: Option<PathBuf>,
    selected: bool,
    enabled: bool,
}
impl Item {
    fn history_key(&self) -> PathBuf {
        if self.kind == Kind::Model {
            return PathBuf::from(format!("local/Model/{}", self.id));
        }
        self.path.clone().unwrap_or_else(|| {
            PathBuf::from(format!(
                "{}/{:?}/{}",
                self.scope.as_deref().unwrap_or("local"),
                self.kind,
                self.id
            ))
        })
    }
    fn native(&self) -> bool {
        !matches!(self.kind, Kind::Cache | Kind::Trash | Kind::Inspect)
    }
    fn kind_label(&self) -> &'static str {
        match self.kind {
            Kind::Cache => "Cache",
            Kind::Trash => "User data",
            Kind::Inspect => "Inspect",
            Kind::Model => "Model",
            Kind::Toolchain => "Toolchain",
            Kind::Image => "Image",
            Kind::Container => "Container",
            Kind::Volume => "Volume",
            Kind::BuildCache => "Build cache",
            Kind::Device => "Device",
            Kind::Runtime => "Runtime",
            Kind::SimFiles => "Sim files",
            Kind::SimUnavailable => "Unavailable",
        }
    }
    fn cleanable(&self, processes: Option<&[sweeploom_core::ProcessSnapshot]>) -> bool {
        self.enabled
            && ((self.native() && self.kind != Kind::Toolchain)
                || self.path.as_ref().is_some_and(|path| {
                    processes.is_some_and(|processes| {
                        !processes.iter().any(|process| process.uses_path(path))
                    })
                }))
    }
}
#[derive(Clone, Default)]
struct Listing {
    items: Vec<Item>,
    note: String,
    complete: bool,
}
enum ListingMsg {
    Initial(Listing),
    Measured(Item),
    Done,
    Failed(String),
}
pub(crate) struct NativeCleanup {
    pane: Pane,
    panes: BTreeMap<Pane, state::PaneState>,
    file_busy: bool,
    pending: Option<(Pane, Vec<Item>)>,
    applying: Option<(Pane, Receiver<String>)>,
}
impl Default for NativeCleanup {
    fn default() -> Self {
        Self {
            pane: Pane::Docker,
            panes: Pane::ALL
                .into_iter()
                .map(|pane| (pane, state::PaneState::default()))
                .collect(),
            file_busy: false,
            pending: None,
            applying: None,
        }
    }
}

pub(crate) fn ui(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    view::ui(app, ui);
}

#[cfg(test)]
mod tests {
    use super::*;
    use docker::parse_docker;
    #[test]
    fn running_simulator_is_not_cleanable() {
        let listing = parse_devices(
            &serde_json::json!({"devices":{"iOS":[{"name":"Active","udid":"ABC","state":"Booted","dataPathSize":100},{"name":"Old","udid":"DEF","state":"Shutdown","dataPathSize":20}]}}),
        );
        assert!(!listing[0].enabled);
        assert!(listing[1].enabled);
        assert_eq!(listing[1].bytes, Some(20));
    }
    #[test]
    fn docker_sizes_and_in_use_objects_are_separate() {
        let listing = parse_docker(
            &serde_json::json!({"Images":[{"ID":"sha256:abc","Repository":"test","Tag":"latest","Size":"2.5GB","Containers":"1"}],"Volumes":[{"Name":"data","Links":"0","Size":"64MB"}]}),
        );
        assert_eq!(listing.len(), 2);
        assert_eq!(listing[0].bytes, Some(2_500_000_000));
        assert!(!listing[0].enabled);
        assert!(listing[1].enabled);
        assert!(!valid_id("--force"));
        assert!(!valid_id("all; rm /"));
    }
}
