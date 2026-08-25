//! Inspect-only review rows for discovered AI stores.

use sweeploom_core::{
    ActivityEvidence, Candidate, CandidateId, CandidateKind, CandidateOwner, DeletionStrategy,
    Evidence, RebuildAssessment, RebuildCost, SafetyAssessment, UserPolicy,
};
use sweeploom_platform::UserLocations;

use crate::discover_stores;

/// One inspect-only AI store row.
#[derive(Clone, Debug)]
pub struct AiOffer {
    /// Human title.
    pub title: String,
    /// Always false. Internal DBs are never auto-selected.
    pub selected: bool,
    /// Candidate, inspect-only.
    pub candidate: Candidate,
}

/// Discover local AI stores as Review candidates.
#[must_use]
pub fn inspect_offers(locations: &UserLocations) -> Vec<AiOffer> {
    let mut offers = Vec::new();
    for (id, store) in (20_000_u64..).zip(discover_stores(locations)) {
        let title = format!("AI {} · {}", store.tool, store.path.display());
        offers.push(AiOffer {
            title: title.clone(),
            selected: false,
            candidate: Candidate {
                id: CandidateId(id),
                kind: CandidateKind::AiSession,
                owner: CandidateOwner::Application(store.tool.to_owned()),
                path: store.path,
                logical_bytes: 0,
                allocated_bytes: None,
                file_count: 0,
                activity: ActivityEvidence::default(),
                safety: SafetyAssessment::review(),
                rebuild: RebuildAssessment {
                    cost: RebuildCost::Unknown,
                    observed_duration_ms: None,
                },
                deletion: DeletionStrategy::InspectOnly,
                evidence: vec![Evidence::exact("ai-store-inspect", title)],
                user_policy: UserPolicy::NeverClean,
            },
        });
    }
    offers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_are_never_preselected() {
        for offer in inspect_offers(&UserLocations::current()) {
            assert!(!offer.selected);
            assert_eq!(offer.candidate.deletion, DeletionStrategy::InspectOnly);
        }
    }
}
