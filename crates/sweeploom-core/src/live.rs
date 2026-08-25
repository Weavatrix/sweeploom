//! Live process and session models. No OS APIs live here.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::evidence::Confidence;
use crate::ids::{ProcessKey, ProjectId, SessionId};
use crate::recommendation::Recommendation;
use crate::safety::{SafetyAssessment, SafetyLevel};

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

/// Best-effort network counters for one process. Missing capability must not
/// be shown as "zero activity".
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

/// One sampled process. Command lines must already be redacted.
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

/// Logical session kind.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SessionKind {
    /// Terminal / shell session.
    Terminal,
    /// Claude Code.
    ClaudeCode,
    /// Codex.
    Codex,
    /// MCP server tree.
    Mcp,
    /// Dev server (vite, next, uvicorn, ...).
    DevServer,
    /// Build (cargo, gradle, ...).
    Build,
    /// Test runner.
    TestRunner,
    /// Language server.
    LanguageServer,
    /// Browser process tree.
    Browser,
    /// Container.
    Container,
    /// Generic application.
    GenericApp,
    /// Unclassified.
    Unknown,
}

/// Observed session activity classification.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SessionActivity {
    /// Doing user-visible work.
    Active,
    /// Background but not idle.
    BackgroundActive,
    /// Idle.
    Idle,
    /// Idle but holding a lot of RAM.
    SleepingMemoryHeavy,
    /// Runaway CPU.
    RunawayCpu,
    /// Network-active.
    NetworkActive,
    /// Likely forgotten.
    LikelyForgotten,
    /// Orphan helper candidate.
    OrphanCandidate,
    /// Unknown.
    Unknown,
}

/// Disk I/O rolled up for a session.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionDiskUsage {
    /// Read bytes observed.
    pub read_bytes: u64,
    /// Write bytes observed.
    pub write_bytes: u64,
}

/// Network rolled up for a session.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionNetworkUsage {
    /// Capability: connections.
    pub connections_available: bool,
    /// Capability: byte rate.
    pub byte_rate_available: bool,
    /// Observed rx.
    pub observed_rx_bytes: u64,
    /// Observed tx.
    pub observed_tx_bytes: u64,
    /// Listening ports.
    pub listening_ports: Vec<u16>,
}

/// Session-level safety. System-critical never becomes a recommendation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionSafety {
    /// Assessment.
    pub assessment: SafetyAssessment,
    /// True when terminate actions should be disabled in the normal UI.
    pub terminate_disabled: bool,
}

impl SessionSafety {
    /// Default for a user/dev session.
    #[must_use]
    pub fn user() -> Self {
        Self {
            assessment: SafetyAssessment {
                level: SafetyLevel::Review,
                blockers: Vec::new(),
                warnings: Vec::new(),
                confidence: crate::evidence::Confidence::Strong,
            },
            terminate_disabled: false,
        }
    }

    /// System-critical session. Terminate disabled.
    #[must_use]
    pub fn system_critical() -> Self {
        Self {
            assessment: SafetyAssessment::blocked(crate::safety::Blocker::SystemCriticalProcess),
            terminate_disabled: true,
        }
    }
}

/// Planner output for a session. Constrained by safety.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionRecommendation {
    /// Recommendation.
    pub recommendation: Recommendation,
    /// Estimated RSS that might return to the OS. Not a guarantee.
    pub estimated_reclaimable_rss: u64,
}

/// Logical developer session sitting on top of the OS process tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiveSession {
    /// Session id.
    pub id: SessionId,
    /// Kind.
    pub kind: SessionKind,
    /// Optional project.
    pub project: Option<ProjectId>,
    /// Member processes.
    pub processes: Vec<ProcessKey>,
    /// When the oldest member started.
    pub started_at: Option<SystemTime>,
    /// Last observed meaningful activity.
    pub observed_last_activity: Option<SystemTime>,
    /// Sum of RSS. Not uniquely reclaimable.
    pub rss_bytes: u64,
    /// Combined CPU percent.
    pub cpu_percent: f32,
    /// Disk rollup.
    pub disk: SessionDiskUsage,
    /// Network rollup.
    pub network: SessionNetworkUsage,
    /// Activity classification.
    pub activity: SessionActivity,
    /// Safety.
    pub safety: SessionSafety,
    /// Recommendation, already constrained by safety.
    pub recommendation: SessionRecommendation,
}

impl LiveSession {
    /// Display label used in the UI and CLI.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self.kind {
            SessionKind::Terminal => "Terminal",
            SessionKind::ClaudeCode => "Claude Code",
            SessionKind::Codex => "Codex",
            SessionKind::Mcp => "MCP",
            SessionKind::DevServer => "Dev server",
            SessionKind::Build => "Build",
            SessionKind::TestRunner => "Test runner",
            SessionKind::LanguageServer => "Language server",
            SessionKind::Browser => "Browser",
            SessionKind::Container => "Container",
            SessionKind::GenericApp => "App",
            SessionKind::Unknown => "Unknown session",
        }
    }
}
