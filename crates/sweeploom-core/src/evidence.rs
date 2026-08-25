//! Typed evidence attached to candidates, sessions, and attributions.

/// How strongly a conclusion is supported.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Confidence {
    /// Direct observation, for example cwd inside the project.
    Exact,
    /// High-quality supporting signal.
    Strong,
    /// Suggestive but incomplete.
    Weak,
    /// Not established.
    Unknown,
}

/// One inspectable reason a decision was made.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Evidence {
    /// Short machine key, for example `cargo-generated`.
    pub key: String,
    /// Human-readable sentence.
    pub detail: String,
    /// Strength of this particular signal.
    pub confidence: Confidence,
}

impl Evidence {
    /// Construct evidence with exact confidence.
    #[must_use]
    pub fn exact(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            detail: detail.into(),
            confidence: Confidence::Exact,
        }
    }
}
