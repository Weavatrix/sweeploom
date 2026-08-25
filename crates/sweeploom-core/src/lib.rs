//! OS-neutral SweepLoom data model.
//!
//! This crate knows nothing about egui, sysinfo, or OS APIs. Safety and
//! recommendation are independent axes: a recommendation score never overrides
//! a safety blocker.

#![cfg_attr(not(test), warn(missing_docs))]

pub mod activity;
pub mod candidate;
pub mod evidence;
pub mod ids;
pub mod live;
pub mod plan;
pub mod policy;
pub mod rebuild;
pub mod recommendation;
pub mod redaction;
pub mod safety;

pub use activity::{ActivityEvidence, ActivityState};
pub use candidate::{Candidate, CandidateKind, CandidateOwner};
pub use evidence::{Confidence, Evidence};
pub use ids::{CandidateId, PlanId, ProcessKey, ProjectId, SessionId, ToolId};
pub use live::{
    LiveSession, NetworkSnapshot, ProcessSafetyClass, ProcessSnapshot, ProjectAttribution,
    SessionActivity, SessionDiskUsage, SessionKind, SessionNetworkUsage, SessionRecommendation,
    SessionSafety,
};
pub use plan::{
    CleanPlan, CleanPlanEntry, DeletionStrategy, ExecutionReport, Receipt, ReceiptCounts,
    SafetyPrecondition, SkipReason,
};
pub use policy::UserPolicy;
pub use rebuild::{RebuildAssessment, RebuildCost};
pub use recommendation::Recommendation;
pub use redaction::redact_command;
pub use safety::{Blocker, SafetyAssessment, SafetyLevel, Warning};
