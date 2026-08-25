//! Process snapshots built on `sysinfo`. CPU is measured over two refreshes.

#![cfg_attr(not(test), warn(missing_docs))]

mod classify;
mod control;
mod snapshot;

pub use classify::classify_process;
pub use control::SysinfoProcessControl;
pub use snapshot::{
    HostCpu, HostMemory, ProcessSampler, ProcessSnapshotSet, host_cpu, host_memory, volume_space,
};

use sweeploom_core::{ProcessKey, ProcessSnapshot};

/// Look up a live snapshot by key, refusing PID reuse.
#[must_use]
pub fn find_process<'a>(
    processes: &'a [ProcessSnapshot],
    key: ProcessKey,
) -> Option<&'a ProcessSnapshot> {
    processes.iter().find(|process| process.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn sampler_returns_self() {
        let mut sampler = ProcessSampler::new();
        let set = sampler.refresh(Duration::from_millis(150));
        assert!(
            set.processes
                .iter()
                .any(|process| process.pid == std::process::id()),
            "current process must appear in the snapshot"
        );
        assert!(set.total_rss_bytes > 0);
    }
}
