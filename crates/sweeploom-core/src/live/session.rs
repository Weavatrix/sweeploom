//! Logical session types sitting on the process tree.

use std::time::SystemTime;

use crate::ids::{ProcessKey, ProjectId, SessionId};
use crate::recommendation::Recommendation;
use crate::safety::{SafetyAssessment, SafetyLevel};

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

impl SessionActivity {
    /// Short UI label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Active => "Active now",
            Self::BackgroundActive => "Working",
            Self::Idle => "Idle",
            Self::SleepingMemoryHeavy => "Idle, heavy RAM",
            Self::RunawayCpu => "High CPU",
            Self::NetworkActive => "Network active",
            Self::LikelyForgotten => "Forgotten",
            Self::OrphanCandidate => "Orphan helper",
            Self::Unknown => "Unknown",
        }
    }
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
