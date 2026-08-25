//! Native-messaging host manifests. Registry writes stay in the CLI.

use std::path::Path;

use serde::Serialize;

/// Chrome/Firefox native-messaging host name.
pub const HOST_NAME: &str = "com.sweeploom.companion";

/// Stable Firefox add-on id (`browser_specific_settings.gecko.id`).
pub const FIREFOX_ADDON_ID: &str = "companion@sweeploom.com";

#[derive(Serialize)]
struct ChromiumHost {
    name: &'static str,
    description: &'static str,
    path: String,
    #[serde(rename = "type")]
    kind: &'static str,
    allowed_origins: Vec<String>,
}

#[derive(Serialize)]
struct FirefoxHost {
    name: &'static str,
    description: &'static str,
    path: String,
    #[serde(rename = "type")]
    kind: &'static str,
    allowed_extensions: Vec<String>,
}

/// `chrome-extension://<id>/` origin for Chromium native messaging.
#[must_use]
pub fn chromium_origin(extension_id: &str) -> String {
    format!("chrome-extension://{extension_id}/")
}

/// True when `id` looks like a Chromium extension id (`a`–`p`, 32 chars).
#[must_use]
pub fn is_chromium_extension_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|byte| (b'a'..=b'p').contains(&byte))
}

/// Chromium-family host manifest (`allowed_origins`).
pub fn chromium_host_json(
    host_exe: &Path,
    extension_id: &str,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&ChromiumHost {
        name: HOST_NAME,
        description: "SweepLoom companion",
        path: host_exe.to_string_lossy().into_owned(),
        kind: "stdio",
        allowed_origins: vec![chromium_origin(extension_id)],
    })
}

/// Firefox host manifest (`allowed_extensions`).
pub fn firefox_host_json(host_exe: &Path) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&FirefoxHost {
        name: HOST_NAME,
        description: "SweepLoom companion",
        path: host_exe.to_string_lossy().into_owned(),
        kind: "stdio",
        allowed_extensions: vec![FIREFOX_ADDON_ID.to_owned()],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn chromium_id_is_a_to_p() {
        assert!(is_chromium_extension_id("abcdefghijklmnopabcdefghijklmnop"));
        assert!(!is_chromium_extension_id("not-an-id"));
    }

    #[test]
    fn manifests_name_the_host() {
        let exe = PathBuf::from("/opt/sweeploom-companion-host");
        let chrome = chromium_host_json(&exe, "abcdefghijklmnopabcdefghijklmnop").unwrap();
        assert!(chrome.contains(HOST_NAME));
        assert!(chrome.contains("chrome-extension://abcdefghijklmnopabcdefghijklmnop/"));
        let firefox = firefox_host_json(&exe).unwrap();
        assert!(firefox.contains(FIREFOX_ADDON_ID));
        assert!(firefox.contains("stdio"));
    }
}
