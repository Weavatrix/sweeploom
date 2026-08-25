//! Termination ladder. Force-kill is never the default.

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use sweeploom_core::ProcessKey;
use sweeploom_platform::{ProcessControlBackend, Result};

/// `sysinfo`-backed process control. Verifies `ProcessKey` before signalling.
#[derive(Debug, Default)]
pub struct SysinfoProcessControl;

impl SysinfoProcessControl {
    /// Construct.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn with_matching_process<F>(&self, key: ProcessKey, action: F) -> Result<()>
    where
        F: FnOnce(&sysinfo::Process) -> bool,
    {
        let mut system = System::new();
        let pid = Pid::from_u32(key.pid);
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::everything(),
        );
        let Some(process) = system.process(pid) else {
            return Err(sweeploom_platform::Error::Capability(
                "process no longer exists",
            ));
        };
        let live_start_ms = process.start_time().saturating_mul(1000);
        if key.started_at_unix_ms != 0 && live_start_ms != key.started_at_unix_ms {
            return Err(sweeploom_platform::Error::Capability(
                "process key no longer matches (PID reused)",
            ));
        }
        if !action(process) {
            return Err(sweeploom_platform::Error::Capability(
                "process signal was not accepted",
            ));
        }
        Ok(())
    }
}

impl ProcessControlBackend for SysinfoProcessControl {
    fn request_graceful_stop(&self, key: ProcessKey) -> Result<()> {
        self.with_matching_process(key, |process| {
            process.kill_with(sysinfo::Signal::Term).unwrap_or(false)
        })
    }

    fn terminate(&self, key: ProcessKey) -> Result<()> {
        self.with_matching_process(key, sysinfo::Process::kill)
    }

    fn force_kill(&self, key: ProcessKey) -> Result<()> {
        self.with_matching_process(key, |process| {
            process
                .kill_with(sysinfo::Signal::Kill)
                .unwrap_or_else(|| process.kill())
        })
    }
}
