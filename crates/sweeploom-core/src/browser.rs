//! Browser identities from executable metadata, never from URLs or tab titles.

use crate::ProcessSnapshot;

/// Which part of a browser installation a process represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserPart {
    /// Main browser executable.
    Main,
    /// Browser helper, including renderers and crash handlers.
    Helper,
    /// Safari background service; it does not prove Safari is open.
    Service,
}

/// A browser family supported by executable evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrowserIdentity {
    /// Display family.
    pub family: &'static str,
    /// Main executable, helper, or standalone background service.
    pub part: BrowserPart,
}

/// Identify a process using its full executable path, name, or argv[0].
/// Generic WebKit/Electron processes are not attributed to Safari/Chrome.
#[must_use]
pub fn browser_identity(process: &ProcessSnapshot) -> Option<BrowserIdentity> {
    let exe = process.exe.as_deref().and_then(|path| path.to_str());
    let mut identity = exe
        .and_then(identity_from_executable)
        .or_else(|| identity_from_name(&process.name))
        .or_else(|| {
            process
                .command
                .first()
                .and_then(|arg| identity_from_executable(arg))
        })?;
    if identity.part == BrowserPart::Main
        && process
            .command
            .iter()
            .any(|arg| arg.starts_with("--type=") || arg.eq_ignore_ascii_case("-contentproc"))
    {
        identity.part = BrowserPart::Helper;
    }
    Some(identity)
}

fn identity_from_executable(executable: &str) -> Option<BrowserIdentity> {
    if executable.contains("://") {
        return None;
    }
    let parts: Vec<_> = executable.split(['/', '\\']).collect();
    if let Some(mut identity) = parts.last().and_then(|name| identity_from_name(name)) {
        if identity.family == "Safari" && parts.contains(&"RuntimeRoot") {
            identity.family = "iOS Safari";
        }
        return Some(identity);
    }
    // Helpers with generic names (e.g. crashpad/plugin-container) require a
    // known browser bundle, not the name of a random command argument.
    parts.windows(2).find_map(|parts| {
        let bundle = parts[0].strip_suffix(".app")?;
        if parts[1] != "Contents" {
            return None;
        }
        let mut identity = identity_from_name(bundle)?;
        identity.part = BrowserPart::Helper;
        Some(identity)
    })
}

fn identity_from_name(name: &str) -> Option<BrowserIdentity> {
    let name = name.to_ascii_lowercase();
    let image = name.strip_suffix(".exe").unwrap_or(&name);
    if SAFARI_SERVICES.contains(&name.as_str()) {
        return Some(BrowserIdentity {
            family: "Safari",
            part: BrowserPart::Service,
        });
    }
    for (family, aliases) in FAMILIES {
        for alias in *aliases {
            if image == *alias {
                return Some(BrowserIdentity {
                    family,
                    part: BrowserPart::Main,
                });
            }
            if name.strip_prefix(alias).is_some_and(|tail| {
                tail == " helper" || tail.starts_with(" helper (") && tail.ends_with(')')
            }) {
                return Some(BrowserIdentity {
                    family,
                    part: BrowserPart::Helper,
                });
            }
        }
    }
    if name.starts_with("firefoxcp ") {
        return Some(BrowserIdentity {
            family: "Firefox",
            part: BrowserPart::Helper,
        });
    }
    None
}

const FAMILIES: &[(&str, &[&str])] = &[
    (
        "Chrome",
        &[
            "chrome",
            "google chrome",
            "google chrome canary",
            "google chrome beta",
            "google chrome dev",
        ],
    ),
    (
        "Edge",
        &[
            "msedge",
            "microsoft edge",
            "microsoft edge beta",
            "microsoft edge dev",
            "microsoft edge canary",
        ],
    ),
    (
        "Firefox",
        &["firefox", "firefox nightly", "firefox developer edition"],
    ),
    (
        "Brave",
        &[
            "brave",
            "brave browser",
            "brave browser beta",
            "brave browser nightly",
        ],
    ),
    ("Safari", &["safari", "safari technology preview"]),
    ("iOS Safari", &["mobilesafari"]),
    ("Chromium", &["chromium", "chromium-browser"]),
    ("Vivaldi", &["vivaldi"]),
    ("Opera", &["opera", "opera gx"]),
    ("Arc", &["arc"]),
];

const SAFARI_SERVICES: &[&str] = &[
    "safarinotificationagent",
    "safaribookmarkssyncagent",
    "com.apple.safari.history",
    "com.apple.safariplatformsupport.helper",
    "com.apple.safari.safebrowsing.service",
];

#[cfg(test)]
#[path = "browser_tests.rs"]
mod tests;
