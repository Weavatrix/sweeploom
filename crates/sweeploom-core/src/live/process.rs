//! Process snapshot types. Command lines must already be redacted.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::evidence::Confidence;
use crate::ids::{ProcessKey, ProjectId, SessionId};

/// How a process relates to the OS and to developer work.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProcessSafetyClass {
    /// Kernel, init, desktop shell, security software. Terminate disabled.
    SystemCritical,
    /// Ordinary OS service.
    SystemService,
    /// Regular user application.
    UserApp,
    /// Compiler, runtime, language server, agent.
    DeveloperTool,
    /// Recognized dev server.
    DevServer,
    /// Claude / Codex / similar.
    Agent,
    /// Helper spawned by an agent or server.
    Helper,
    /// Orphan-looking helper. Still not a hard kill signal on its own.
    OrphanCandidate,
    /// Unclassified.
    Unknown,
}

/// Best-effort network counters for one process.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NetworkSnapshot {
    /// True when connection metadata is available.
    pub connections_available: bool,
    /// True when byte counters are available.
    pub byte_rate_available: bool,
    /// Observed receive bytes since SweepLoom started watching.
    pub observed_rx_bytes: u64,
    /// Observed transmit bytes since SweepLoom started watching.
    pub observed_tx_bytes: u64,
    /// Listening ports attributed to this process.
    pub listening_ports: Vec<u16>,
    /// Established remote endpoints as `host:port` (already redacted).
    pub remotes: Vec<String>,
}

/// One sampled process.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessSnapshot {
    /// PID + start time.
    pub key: ProcessKey,
    /// OS pid.
    pub pid: u32,
    /// Parent key when the parent is still alive and matched.
    pub parent: Option<ProcessKey>,
    /// Process image name.
    pub name: String,
    /// Executable path.
    pub exe: Option<PathBuf>,
    /// Current working directory.
    pub cwd: Option<PathBuf>,
    /// Redacted command line tokens.
    pub command: Vec<String>,
    /// Start time.
    pub started_at: Option<SystemTime>,
    /// Runtime.
    pub runtime: Duration,
    /// Resident set size in bytes. Primary memory metric.
    pub rss_bytes: u64,
    /// Virtual size. Do not use as the primary macOS pressure signal.
    pub virtual_bytes: u64,
    /// CPU percent over the last sample interval.
    pub cpu_percent: f32,
    /// Accumulated CPU time in milliseconds.
    pub accumulated_cpu_ms: u64,
    /// Disk read bytes since previous sample.
    pub disk_read_delta: u64,
    /// Disk write bytes since previous sample.
    pub disk_write_delta: u64,
    /// Network snapshot.
    pub network: NetworkSnapshot,
    /// Project attribution.
    pub project: Option<ProjectAttribution>,
    /// Logical session.
    pub session: Option<SessionId>,
    /// Safety class.
    pub safety_class: ProcessSafetyClass,
}

/// Project attribution with confidence. Never guess from process name alone.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectAttribution {
    /// Project id.
    pub project: ProjectId,
    /// How the link was established.
    pub confidence: Confidence,
}
