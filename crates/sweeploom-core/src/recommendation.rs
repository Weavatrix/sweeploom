//! Recommendation is not safety. Keep them on separate axes.

/// What SweepLoom would do if the user asked for a suggested selection.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Recommendation {
    /// High-confidence generated waste.
    StronglyRecommended,
    /// Reasonable cleanup with rebuild cost shown.
    Recommended,
    /// Optional, never pre-selected for user data.
    Optional,
    /// Leave it alone.
    Keep,
}

impl Recommendation {
    /// A recommendation may never promote a blocked candidate.
    #[must_use]
    pub fn constrained_by(self, safety: &crate::safety::SafetyAssessment) -> Self {
        if safety.is_blocked() {
            Self::Keep
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::safety::{Blocker, SafetyAssessment};

    #[test]
    fn recommendation_cannot_bypass_blocker() {
        let blocked = SafetyAssessment::blocked(Blocker::ActiveProcess);
        assert_eq!(
            Recommendation::StronglyRecommended.constrained_by(&blocked),
            Recommendation::Keep
        );
    }
}
