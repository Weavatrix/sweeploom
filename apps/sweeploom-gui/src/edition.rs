//! Build edition: the free open-source app, or the sandboxed Mac App Store
//! edition (`--features mas`). Store-only behavior branches on [`STORE`].
//!
//! Unit tests always see the open-source edition, so `--all-features` test runs
//! keep exercising the shared logic; store-only helpers take explicit inputs.

use eframe::egui::{self, RichText};
use sweeploom_platform::UserLocations;

use crate::{theme, widgets};

/// True in the App Store build (App Sandbox, folder grants, no subprocesses).
pub const STORE: bool = cfg!(all(feature = "mas", not(test)));

/// Source code of the MPL-2.0 open-source edition.
pub const SOURCE_URL: &str = "https://github.com/Weavatrix/sweeploom";
/// Product site.
pub const SITE_URL: &str = "https://sweeploom.com";
/// Terms of use.
pub const TERMS_URL: &str = "https://sweeploom.com/terms/";
/// Privacy policy.
pub const PRIVACY_URL: &str = "https://sweeploom.com/privacy/";

/// Edition name shown in Settings → About.
#[must_use]
pub const fn label() -> &'static str {
    if STORE {
        "App Store edition"
    } else {
        "Open-source edition"
    }
}

/// What the App Store edition leaves out, and why.
pub const FULL_ONLY: &[(&str, &str)] = &[
    (
        "Sessions, process trees and process history",
        "the App Sandbox hides other apps' processes",
    ),
    (
        "Stopping or terminating processes",
        "sandboxed apps cannot signal other apps",
    ),
    ("Docker cleanup", "needs the docker command-line tool"),
    (
        "Simulator devices and runtimes",
        "needs xcrun simctl and CoreSimulator",
    ),
    (
        "Browser companion and tab discard",
        "installs a native-messaging host into your browsers",
    ),
    (
        "Ollama models and Rust toolchain uninstall",
        "need the ollama and rustup tools",
    ),
    (
        "Sign-in autostart from SweepLoom",
        "add SweepLoom in System Settings → General → Login Items instead",
    ),
];

/// User locations. The App Store edition works on the real home folder; `$HOME`
/// is the app container, which keeps SweepLoom's own settings and history.
#[must_use]
pub fn locations() -> UserLocations {
    let mut locations = UserLocations::current();
    if STORE && let Some(home) = bridge::real_home() {
        locations.downloads = Some(home.join("Downloads"));
        locations.cache = Some(home.join("Library").join("Caches"));
        locations.home = home;
    }
    locations
}

/// App Store edition: hand the CPU performance/efficiency split to the sampler
/// so it never spawns `/usr/sbin/sysctl`. Call before the first sample.
pub fn prepare_sampler() {
    if STORE {
        sweeploom_process::provide_core_split(bridge::performance_levels().map(
            |(performance, efficiency)| sweeploom_process::CoreSplit {
                performance,
                efficiency,
            },
        ));
    }
}

/// Card for a screen whose feature exists only in the open-source edition.
pub fn full_only(ui: &mut egui::Ui, title: &str, body: &str) {
    widgets::card(ui, |ui| {
        ui.label(RichText::new(title).size(15.5).strong());
        ui.add_space(theme::XS);
        widgets::caption(ui, body);
        ui.add_space(theme::SM);
        ui.hyperlink_to("Available in the full open-source edition", SOURCE_URL);
    });
    ui.add_space(theme::MD);
}

/// One muted line for a part of a screen the App Store edition hides.
pub fn full_only_note(ui: &mut egui::Ui, text: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(text).size(12.5).color(theme::muted(ui)));
        let link = RichText::new("Full open-source edition")
            .size(12.5)
            .color(ui.visuals().hyperlink_color);
        ui.hyperlink_to(link, SOURCE_URL);
    });
}

/// The only place the GUI calls the macOS bridge. Without the `mas` feature
/// every call reports that it belongs to the App Store edition.
pub(crate) mod bridge {
    use std::path::{Path, PathBuf};

    /// Open security-scoped access; stops when dropped.
    #[cfg(feature = "mas")]
    pub(crate) type Access = sweeploom_macos::ScopedAccess;
    /// Placeholder: the open-source edition never holds scoped access.
    #[cfg(not(feature = "mas"))]
    #[derive(Debug)]
    #[allow(dead_code)]
    pub(crate) struct Access;

    /// A folder from the panel and its bookmark (or why none was made).
    #[cfg_attr(not(feature = "mas"), allow(dead_code))]
    pub(crate) struct Picked {
        pub path: PathBuf,
        pub bookmark: Result<Vec<u8>, String>,
    }

    #[cfg(feature = "mas")]
    pub(crate) fn real_home() -> Option<PathBuf> {
        sweeploom_macos::real_home()
    }

    #[cfg(feature = "mas")]
    pub(crate) fn performance_levels() -> Option<(usize, usize)> {
        sweeploom_macos::performance_levels()
    }

    #[cfg(feature = "mas")]
    pub(crate) fn pick(start: Option<&Path>, message: &str) -> Result<Vec<Picked>, String> {
        let request = sweeploom_macos::FolderRequest {
            message: message.to_owned(),
            prompt: "Grant Access".to_owned(),
            start: start.map(Path::to_path_buf),
            show_hidden: start.is_some_and(|path| {
                path.components()
                    .any(|part| part.as_os_str().to_string_lossy().starts_with('.'))
            }),
            multiple: true,
        };
        let picked = sweeploom_macos::pick_folders(&request).map_err(|error| error.to_string())?;
        Ok(picked
            .into_iter()
            .map(|item| Picked {
                path: item.path,
                bookmark: item.bookmark.map_err(|error| error.to_string()),
            })
            .collect())
    }

    /// Resolved path, a refreshed bookmark when the old one went stale, and access.
    #[cfg(feature = "mas")]
    pub(crate) fn resolve(data: &[u8]) -> Result<(PathBuf, Option<Vec<u8>>, Access), String> {
        let resolved =
            sweeploom_macos::resolve_bookmark(data).map_err(|error| error.to_string())?;
        if !resolved.access.is_active() {
            return Err("macOS refused access; grant the folder again".to_owned());
        }
        Ok((resolved.path, resolved.refreshed, resolved.access))
    }

    #[cfg(feature = "mas")]
    pub(crate) fn trash(path: &Path) -> Result<(), String> {
        sweeploom_macos::move_to_trash(path)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    #[cfg(feature = "mas")]
    pub(crate) fn reveal(paths: &[PathBuf]) -> Result<(), String> {
        sweeploom_macos::reveal_in_finder(paths).map_err(|error| error.to_string())
    }

    #[cfg(not(feature = "mas"))]
    const ONLY: &str = "Only in the App Store edition";

    #[cfg(not(feature = "mas"))]
    pub(crate) fn real_home() -> Option<PathBuf> {
        None
    }

    #[cfg(not(feature = "mas"))]
    pub(crate) fn performance_levels() -> Option<(usize, usize)> {
        None
    }

    #[cfg(not(feature = "mas"))]
    pub(crate) fn pick(_start: Option<&Path>, _message: &str) -> Result<Vec<Picked>, String> {
        Err(ONLY.to_owned())
    }

    #[cfg(not(feature = "mas"))]
    pub(crate) fn resolve(_data: &[u8]) -> Result<(PathBuf, Option<Vec<u8>>, Access), String> {
        Err(ONLY.to_owned())
    }

    #[cfg(not(feature = "mas"))]
    pub(crate) fn trash(_path: &Path) -> Result<(), String> {
        Err(ONLY.to_owned())
    }

    #[cfg(not(feature = "mas"))]
    pub(crate) fn reveal(_paths: &[PathBuf]) -> Result<(), String> {
        Err(ONLY.to_owned())
    }
}
