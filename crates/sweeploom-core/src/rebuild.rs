//! Rebuild-cost model. Deleting a 20 GB `target` is not free.

/// Qualitative rebuild cost after deleting generated data.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RebuildCost {
    /// Nothing to rebuild.
    None,
    /// Seconds to a couple of minutes.
    Low,
    /// Typical debug rebuild.
    Medium,
    /// Release / native deps / large node_modules.
    High,
    /// Multi-hour or networked restore.
    VeryHigh,
    /// Not estimated.
    Unknown,
}

impl RebuildCost {
    /// Short UI label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::VeryHigh => "Very high",
            Self::Unknown => "Unknown",
        }
    }
}

/// Rebuild assessment attached to a candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RebuildAssessment {
    /// Qualitative cost.
    pub cost: RebuildCost,
    /// Optional previously observed duration, milliseconds.
    pub observed_duration_ms: Option<u64>,
}

impl Default for RebuildAssessment {
    fn default() -> Self {
        Self {
            cost: RebuildCost::Unknown,
            observed_duration_ms: None,
        }
    }
}
