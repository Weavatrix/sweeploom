//! `sweeploom browser` — process-level pressure, no fake tab counts.

use std::time::Duration;

use sweeploom_browser::BrowserPressure;
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::ProcessSampler;
use sweeploom_session::{AttributionRoots, sessions_from_snapshot};

use crate::bytes::format_bytes;

pub fn run() {
    let mut sampler = ProcessSampler::new();
    let mut snapshot = sampler.refresh(Duration::from_millis(200));
    snapshot.resolve_parents();
    let _capability = enrich_network(&mut snapshot.processes);
    let locations = UserLocations::current();
    let roots = AttributionRoots {
        projects: vec![locations.home.clone()],
        current_project: None,
    };
    let sessions = sessions_from_snapshot(&mut snapshot, &roots);
    let pressure = BrowserPressure::from_live(&sessions, &snapshot.processes);
    println!(
        "companion={} hosts={} rss={}",
        if pressure.companion_connected {
            "connected"
        } else {
            "disconnected"
        },
        pressure.hosts.len(),
        format_bytes(pressure.rss_bytes())
    );
    if pressure.hosts.is_empty() {
        println!("no browser process trees");
        return;
    }
    for host in &pressure.hosts {
        println!(
            "{:<8} sessions={:<3} proc={:<4} rss={:<10} cpu={:>5.1}%",
            host.family,
            host.sessions,
            host.processes,
            format_bytes(host.rss_bytes),
            host.cpu_percent
        );
    }
    println!("tab lastAccessed unavailable without the companion; not shown as zero");
}
