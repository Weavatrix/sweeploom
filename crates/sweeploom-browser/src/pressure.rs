//! Process-level browser pressure. Tab counts stay unknown without the companion.

use sweeploom_core::{BrowserPart, LiveSession, ProcessSnapshot, SessionKind, browser_identity};

/// One browser family rolled up from live sessions.
#[derive(Clone, Debug, PartialEq)]
pub struct BrowserHost {
    /// Chrome, Edge, Firefox, Brave, Safari, or Browser.
    pub family: &'static str,
    /// Logical SweepLoom sessions in this family.
    pub sessions: usize,
    /// Member processes.
    pub processes: usize,
    /// Observed main browser processes. Background services are not open browsers.
    pub main_processes: usize,
    /// Standalone browser-specific background services.
    pub background_services: usize,
    /// Combined RSS. Not uniquely reclaimable by killing the tree.
    pub rss_bytes: u64,
    /// Combined CPU percent.
    pub cpu_percent: f32,
}

/// Workstation browser pressure visible without the extension.
#[derive(Clone, Debug, PartialEq)]
pub struct BrowserPressure {
    /// Native-messaging companion. False until the host is connected.
    pub companion_connected: bool,
    /// Rolled-up browser families, largest RSS first.
    pub hosts: Vec<BrowserHost>,
}

impl BrowserPressure {
    /// Roll up `SessionKind::Browser` sessions. Tab heat is not inferred.
    #[must_use]
    pub fn from_live(sessions: &[LiveSession], processes: &[ProcessSnapshot]) -> Self {
        let mut hosts: Vec<BrowserHost> = Vec::new();
        for session in sessions
            .iter()
            .filter(|item| item.kind == SessionKind::Browser)
        {
            let family = family_of(session, processes);
            let (mains, services) = session
                .processes
                .iter()
                .filter_map(|key| processes.iter().find(|process| process.key == *key))
                .filter_map(browser_identity)
                .fold((0, 0), |(mains, services), identity| {
                    (
                        mains + usize::from(identity.part == BrowserPart::Main),
                        services + usize::from(identity.part == BrowserPart::Service),
                    )
                });
            match hosts.iter_mut().find(|host| host.family == family) {
                Some(host) => {
                    host.sessions += 1;
                    host.processes += session.processes.len();
                    host.main_processes += mains;
                    host.background_services += services;
                    host.rss_bytes = host.rss_bytes.saturating_add(session.rss_bytes);
                    host.cpu_percent += session.cpu_percent;
                }
                None => hosts.push(BrowserHost {
                    family,
                    sessions: 1,
                    processes: session.processes.len(),
                    main_processes: mains,
                    background_services: services,
                    rss_bytes: session.rss_bytes,
                    cpu_percent: session.cpu_percent,
                }),
            }
        }
        hosts.sort_by_key(|host| std::cmp::Reverse(host.rss_bytes));
        Self {
            companion_connected: false,
            hosts,
        }
    }

    /// Combined RSS across browser families.
    #[must_use]
    pub fn rss_bytes(&self) -> u64 {
        self.hosts.iter().map(|host| host.rss_bytes).sum()
    }
}

/// Map a process name to a browser family.
#[must_use]
pub fn family_from_name(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.contains("msedge") || lower.contains("microsoft edge") {
        "Edge"
    } else if lower.contains("chrome") {
        "Chrome"
    } else if lower.contains("firefox") {
        "Firefox"
    } else if lower.contains("brave") {
        "Brave"
    } else if lower.contains("mobilesafari") {
        "iOS Safari"
    } else if lower.contains("safari") {
        "Safari"
    } else if lower.contains("chromium") {
        "Chromium"
    } else if lower.contains("vivaldi") {
        "Vivaldi"
    } else if lower.contains("opera") {
        "Opera"
    } else if lower == "arc" {
        "Arc"
    } else {
        "Browser"
    }
}

fn family_of(session: &LiveSession, processes: &[ProcessSnapshot]) -> &'static str {
    session
        .processes
        .first()
        .and_then(|key| processes.iter().find(|process| process.key == *key))
        .map(|process| {
            browser_identity(process)
                .map(|identity| identity.family)
                .unwrap_or_else(|| family_from_name(&process.name))
        })
        .unwrap_or("Browser")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    use sweeploom_core::{
        ProcessKey, Recommendation, SessionActivity, SessionDiskUsage, SessionId,
        SessionNetworkUsage, SessionRecommendation, SessionSafety,
    };

    fn chrome_session() -> LiveSession {
        LiveSession {
            id: SessionId(1),
            kind: SessionKind::Browser,
            project: None,
            processes: vec![ProcessKey {
                pid: 10,
                started_at_unix_ms: 1,
            }],
            started_at: Some(SystemTime::UNIX_EPOCH),
            observed_last_activity: Some(SystemTime::UNIX_EPOCH),
            rss_bytes: 2_000_000_000,
            cpu_percent: 1.5,
            disk: SessionDiskUsage::default(),
            network: SessionNetworkUsage::default(),
            activity: SessionActivity::BackgroundActive,
            safety: SessionSafety::user(),
            recommendation: SessionRecommendation {
                recommendation: Recommendation::Keep,
                estimated_reclaimable_rss: 0,
            },
        }
    }

    fn chrome_process() -> ProcessSnapshot {
        ProcessSnapshot {
            key: ProcessKey {
                pid: 10,
                started_at_unix_ms: 1,
            },
            pid: 10,
            parent: None,
            name: "chrome.exe".into(),
            exe: None,
            cwd: None,
            command: vec!["chrome.exe".into()],
            started_at: None,
            runtime: std::time::Duration::from_secs(1),
            rss_bytes: 2_000_000_000,
            virtual_bytes: 2_000_000_000,
            cpu_percent: 1.5,
            accumulated_cpu_ms: 0,
            disk_read_delta: 0,
            disk_write_delta: 0,
            network: sweeploom_core::NetworkSnapshot::default(),
            project: None,
            session: None,
            safety_class: sweeploom_core::ProcessSafetyClass::UserApp,
        }
    }

    #[test]
    fn rolls_up_chrome_from_session() {
        let process = chrome_process();
        let pressure = BrowserPressure::from_live(&[chrome_session()], &[process]);
        assert_eq!(pressure.hosts.len(), 1);
        assert_eq!(pressure.hosts[0].family, "Chrome");
        assert_eq!(pressure.hosts[0].main_processes, 1);
        assert_eq!(pressure.hosts[0].background_services, 0);
        assert!(!pressure.companion_connected);
        assert_eq!(pressure.rss_bytes(), 2_000_000_000);
    }

    #[test]
    fn executable_family_and_service_counts_remain_distinct() {
        let mut chrome = chrome_process();
        chrome.name = "Google Chro".into();
        chrome.exe = Some("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".into());
        let mut safari = chrome_process();
        safari.key.pid = 20;
        safari.pid = 20;
        safari.name = "SafariNotificationAgent".into();
        safari.command.clear();
        let mut service = chrome_session();
        service.id = SessionId(2);
        service.processes = vec![safari.key];
        let pressure = BrowserPressure::from_live(&[chrome_session(), service], &[chrome, safari]);
        let chrome = pressure
            .hosts
            .iter()
            .find(|host| host.family == "Chrome")
            .unwrap();
        let safari = pressure
            .hosts
            .iter()
            .find(|host| host.family == "Safari")
            .unwrap();
        assert_eq!(chrome.main_processes, 1);
        assert_eq!(safari.main_processes, 0);
        assert_eq!(safari.background_services, 1);
    }
}
