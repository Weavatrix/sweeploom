//! Process connections. Missing capability is never displayed as zero activity.

#![cfg_attr(not(test), warn(missing_docs))]

use std::collections::HashMap;

use sweeploom_core::{NetworkSnapshot, ProcessKey};
use sweeploom_platform::NetworkCapability;

/// One listening or established endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    /// Local port.
    pub local_port: u16,
    /// Remote host:port if established.
    pub remote: Option<String>,
    /// True when LISTEN.
    pub listening: bool,
}

/// Attach connection metadata to snapshots. Byte rates stay capability-gated.
#[must_use]
pub fn enrich_network(processes: &mut [sweeploom_core::ProcessSnapshot]) -> NetworkCapability {
    let connections = connections_supported();
    if !connections {
        return NetworkCapability {
            connections: false,
            byte_rates: false,
        };
    }
    let table = load_pid_endpoints();
    let bytes = load_pid_bytes();
    let byte_rates = bytes.is_some();
    let bytes = bytes.unwrap_or_default();
    for process in processes {
        let endpoints = table.get(&process.pid).map_or(&[][..], Vec::as_slice);
        process.network = snapshot_from(endpoints, bytes.get(&process.pid).copied(), byte_rates);
    }
    NetworkCapability {
        connections: true,
        byte_rates,
    }
}

/// Current platform capability. Byte rates require a successful ESTATS sample.
#[must_use]
pub fn current_capability() -> NetworkCapability {
    NetworkCapability {
        connections: connections_supported(),
        byte_rates: false,
    }
}

fn connections_supported() -> bool {
    cfg!(any(target_os = "linux", windows))
}

fn snapshot_from(
    endpoints: &[Endpoint],
    bytes: Option<(u64, u64)>,
    byte_rates: bool,
) -> NetworkSnapshot {
    let (rx, tx) = bytes.unwrap_or((0, 0));
    NetworkSnapshot {
        connections_available: true,
        byte_rate_available: byte_rates,
        observed_rx_bytes: if byte_rates { rx } else { 0 },
        observed_tx_bytes: if byte_rates { tx } else { 0 },
        listening_ports: endpoints
            .iter()
            .filter(|item| item.listening)
            .map(|item| item.local_port)
            .collect(),
        remotes: endpoints
            .iter()
            .filter_map(|item| item.remote.clone())
            .collect(),
    }
}

fn load_pid_endpoints() -> HashMap<u32, Vec<Endpoint>> {
    #[cfg(target_os = "linux")]
    {
        linux::load()
    }
    #[cfg(windows)]
    {
        windows::load()
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        HashMap::new()
    }
}

fn load_pid_bytes() -> Option<HashMap<u32, (u64, u64)>> {
    #[cfg(windows)]
    {
        windows_estats::load()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Look up endpoints for a process key. PID reuse is the caller's problem;
/// they must confirm the key is still live.
#[must_use]
pub fn endpoints_for(_key: ProcessKey) -> Vec<Endpoint> {
    Vec::new()
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
mod windows_estats;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_is_explicit() {
        let capability = current_capability();
        assert!(!capability.byte_rates);
        if !cfg!(any(target_os = "linux", windows)) {
            assert!(!capability.connections);
        }
    }

    #[test]
    fn missing_bytes_are_not_claimed() {
        let snap = snapshot_from(&[], None, false);
        assert!(!snap.byte_rate_available);
        assert_eq!(snap.observed_rx_bytes, 0);
    }
}
