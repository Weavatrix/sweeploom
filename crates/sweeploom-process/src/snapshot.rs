//! sysinfo-backed sampler. Two refreshes are required before CPU is meaningful.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sysinfo::{Disks, MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System};

use sweeploom_core::{NetworkSnapshot, ProcessKey, ProcessSnapshot, redact_command};

use crate::classify::classify_process;

/// Host memory snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostMemory {
    /// Total physical RAM.
    pub total_bytes: u64,
    /// Used physical RAM as reported by the OS.
    pub used_bytes: u64,
    /// Available physical RAM.
    pub available_bytes: u64,
}

/// Host CPU snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HostCpu {
    /// Global CPU usage percent after at least two refreshes.
    pub usage_percent: f32,
}

/// One complete process snapshot plus host totals.
#[derive(Clone, Debug)]
pub struct ProcessSnapshotSet {
    /// Sample time.
    pub captured_at: SystemTime,
    /// Processes.
    pub processes: Vec<ProcessSnapshot>,
    /// Host memory.
    pub memory: HostMemory,
    /// Host CPU.
    pub cpu: HostCpu,
    /// Sum of process RSS.
    pub total_rss_bytes: u64,
}

/// Incremental sampler. Keeps a `System` so CPU deltas are valid.
pub struct ProcessSampler {
    system: System,
    previous_disk: HashMap<ProcessKey, (u64, u64)>,
    warmed: bool,
}

impl Default for ProcessSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessSampler {
    /// Create a sampler. The first `refresh` warms CPU counters.
    #[must_use]
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_specifics(refresh_kind());
        Self {
            system,
            previous_disk: HashMap::new(),
            warmed: false,
        }
    }

    /// Refresh processes. `cpu_gap` is slept only when the sampler is cold.
    pub fn refresh(&mut self, cpu_gap: Duration) -> ProcessSnapshotSet {
        if !self.warmed {
            std::thread::sleep(cpu_gap);
            self.warmed = true;
        }
        self.system.refresh_specifics(refresh_kind());
        let captured_at = SystemTime::now();
        let mut previous_disk = HashMap::new();
        let mut processes = Vec::with_capacity(self.system.processes().len());
        for (pid, process) in self.system.processes() {
            let snapshot = convert(pid, process, &self.previous_disk);
            previous_disk.insert(
                snapshot.key,
                (
                    process.disk_usage().total_read_bytes,
                    process.disk_usage().total_written_bytes,
                ),
            );
            processes.push(snapshot);
        }
        self.previous_disk = previous_disk;
        let total_rss_bytes = processes.iter().map(|item| item.rss_bytes).sum();
        ProcessSnapshotSet {
            captured_at,
            processes,
            memory: host_memory(&self.system),
            cpu: host_cpu(&self.system),
            total_rss_bytes,
        }
    }
}

fn refresh_kind() -> RefreshKind {
    RefreshKind::nothing()
        .with_memory(MemoryRefreshKind::everything())
        .with_cpu(sysinfo::CpuRefreshKind::everything())
        .with_processes(ProcessRefreshKind::everything())
}

/// Host memory from a refreshed `System`.
#[must_use]
pub fn host_memory(system: &System) -> HostMemory {
    HostMemory {
        total_bytes: system.total_memory(),
        used_bytes: system.used_memory(),
        available_bytes: system.available_memory(),
    }
}

/// Host CPU from a refreshed `System`.
#[must_use]
pub fn host_cpu(system: &System) -> HostCpu {
    HostCpu {
        usage_percent: system.global_cpu_usage(),
    }
}

/// Volume totals from `sysinfo::Disks`.
#[must_use]
pub fn volume_space() -> Vec<(PathBuf, u64, u64)> {
    let mut disks = Disks::new_with_refreshed_list();
    disks.refresh(true);
    disks
        .iter()
        .map(|disk| {
            (
                disk.mount_point().to_path_buf(),
                disk.total_space(),
                disk.available_space(),
            )
        })
        .collect()
}

fn convert(
    pid: &Pid,
    process: &sysinfo::Process,
    previous_disk: &HashMap<ProcessKey, (u64, u64)>,
) -> ProcessSnapshot {
    let pid_u32 = pid.as_u32();
    let started_at = if process.start_time() == 0 {
        None
    } else {
        Some(UNIX_EPOCH + Duration::from_secs(process.start_time()))
    };
    let key = ProcessKey::new(pid_u32, started_at);
    let parent = process.parent().map(|parent| {
        // Parent start time is unknown here; match by pid only at this layer and
        // let session grouping resolve the key against the full snapshot.
        ProcessKey {
            pid: parent.as_u32(),
            started_at_unix_ms: 0,
        }
    });
    let command: Vec<String> = process
        .cmd()
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect();
    let disk = process.disk_usage();
    let previous = previous_disk
        .get(&key)
        .copied()
        .unwrap_or((disk.total_read_bytes, disk.total_written_bytes));
    let name = os_name(process.name());
    let safety_class = classify_process(&name, process.exe().map(PathBuf::from).as_deref());
    ProcessSnapshot {
        key,
        pid: pid_u32,
        parent,
        name,
        exe: process.exe().map(PathBuf::from),
        cwd: process.cwd().map(PathBuf::from),
        command: redact_command(&command),
        started_at,
        runtime: Duration::from_secs(process.run_time()),
        rss_bytes: process.memory(),
        virtual_bytes: process.virtual_memory(),
        cpu_percent: process.cpu_usage(),
        accumulated_cpu_ms: process.accumulated_cpu_time(),
        disk_read_delta: disk.total_read_bytes.saturating_sub(previous.0),
        disk_write_delta: disk.total_written_bytes.saturating_sub(previous.1),
        network: NetworkSnapshot::default(),
        project: None,
        session: None,
        safety_class,
    }
}

fn os_name(name: &OsStr) -> String {
    name.to_string_lossy().into_owned()
}

impl ProcessSnapshotSet {
    /// Resolve parent keys using pid + start time from the same snapshot.
    pub fn resolve_parents(&mut self) {
        let by_pid: HashMap<u32, ProcessKey> = self
            .processes
            .iter()
            .map(|process| (process.pid, process.key))
            .collect();
        for process in &mut self.processes {
            if let Some(parent) = process.parent
                && parent.started_at_unix_ms == 0
            {
                process.parent = by_pid.get(&parent.pid).copied();
            }
        }
    }

    /// Children of `parent`, including only processes whose parent key matches.
    #[must_use]
    pub fn children_of(&self, parent: ProcessKey) -> Vec<&ProcessSnapshot> {
        self.processes
            .iter()
            .filter(|process| process.parent == Some(parent))
            .collect()
    }
}
