use super::*;
use crate::{NetworkSnapshot, ProcessKey, ProcessSafetyClass};
use std::{path::PathBuf, time::Duration};

fn process(name: &str, exe: Option<&str>, command: &[&str]) -> ProcessSnapshot {
    ProcessSnapshot {
        key: ProcessKey::new(10, None),
        pid: 10,
        parent: None,
        name: name.into(),
        exe: exe.map(PathBuf::from),
        cwd: None,
        command: command.iter().map(|arg| (*arg).into()).collect(),
        started_at: None,
        runtime: Duration::ZERO,
        rss_bytes: 1,
        virtual_bytes: 1,
        cpu_percent: 0.0,
        accumulated_cpu_ms: 0,
        disk_read_delta: 0,
        disk_write_delta: 0,
        network: NetworkSnapshot::default(),
        project: None,
        session: None,
        safety_class: ProcessSafetyClass::Unknown,
    }
}

#[test]
fn recognizes_macos_browser_names_and_helpers() {
    for (name, family) in [
        ("Google Chrome", "Chrome"),
        ("Google Chrome Canary", "Chrome"),
        ("Microsoft Edge", "Edge"),
        ("Brave Browser", "Brave"),
        ("Safari", "Safari"),
        ("Safari Technology Preview", "Safari"),
        ("MobileSafari", "iOS Safari"),
        ("firefox.exe", "Firefox"),
    ] {
        assert_eq!(
            browser_identity(&process(name, None, &[])),
            Some(BrowserIdentity {
                family,
                part: BrowserPart::Main
            }),
            "{name}"
        );
    }
    for name in ["Google Chrome Helper", "Google Chrome Helper (Renderer)"] {
        assert_eq!(
            browser_identity(&process(name, None, &[])),
            Some(BrowserIdentity {
                family: "Chrome",
                part: BrowserPart::Helper
            })
        );
    }
    assert_eq!(
        browser_identity(&process(
            "chrome.exe",
            None,
            &["chrome.exe", "--type=renderer"]
        ))
        .unwrap()
        .part,
        BrowserPart::Helper
    );
}

#[test]
fn full_executable_and_argv_zero_recover_truncated_names() {
    let exe = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
    let identity = BrowserIdentity {
        family: "Chrome",
        part: BrowserPart::Main,
    };
    assert_eq!(
        browser_identity(&process("Google Chro", Some(exe), &[])),
        Some(identity)
    );
    assert_eq!(browser_identity(&process("", None, &[exe])), Some(identity));
    let helper = "/Applications/Google Chrome.app/Contents/Frameworks/Google Chrome Framework.framework/Helpers/chrome_crashpad_handler";
    assert_eq!(
        browser_identity(&process("chrome_crashpad_", Some(helper), &[])),
        Some(BrowserIdentity {
            family: "Chrome",
            part: BrowserPart::Helper
        })
    );
}

#[test]
fn safari_background_services_do_not_prove_a_main_browser() {
    for name in SAFARI_SERVICES {
        assert_eq!(
            browser_identity(&process(name, None, &[])),
            Some(BrowserIdentity {
                family: "Safari",
                part: BrowserPart::Service
            })
        );
    }
    let exe = "/Library/Developer/CoreSimulator/RuntimeRoot/usr/libexec/SafariBookmarksSyncAgent";
    assert_eq!(
        browser_identity(&process("SafariBookmarks", Some(exe), &[])),
        Some(BrowserIdentity {
            family: "iOS Safari",
            part: BrowserPart::Service
        })
    );
}

#[test]
fn urls_and_shared_web_engines_are_not_browser_evidence() {
    for (name, exe, command) in [
        (
            "node",
            Some("/usr/bin/node"),
            vec!["node", "https://example.test/chrome", "Safari"],
        ),
        ("com.apple.WebKit.WebContent", None, vec![]),
        (
            "Electron",
            Some("/Applications/Cursor.app/Contents/MacOS/Electron"),
            vec![],
        ),
        ("chromedriver", None, vec!["chromedriver"]),
        ("msedgewebview2.exe", None, vec![]),
        ("SafariWidgetExtension", None, vec![]),
        ("python", None, vec!["https://example.test/chrome"]),
    ] {
        assert_eq!(
            browser_identity(&process(name, exe, &command)),
            None,
            "{name}"
        );
    }
}
