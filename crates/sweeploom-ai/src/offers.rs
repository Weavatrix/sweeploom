//! Inspect-only review rows for discovered AI stores.

use sweeploom_core::{
    ActivityEvidence, Candidate, CandidateId, CandidateKind, CandidateOwner, DeletionStrategy,
    Evidence, RebuildAssessment, RebuildCost, SafetyAssessment, UserPolicy,
};
use sweeploom_platform::UserLocations;

use std::path::PathBuf;

use crate::discover_stores;
use crate::inventory::{Limits, list_store};

/// One inspect-only AI store row.
#[derive(Clone, Debug)]
pub struct AiOffer {
    /// Human title.
    pub title: String,
    /// Always false. Internal DBs are never auto-selected.
    pub selected: bool,
    /// Relative sample paths from the bounded walk.
    pub samples: Vec<String>,
    /// True when the walk hit a depth or file cap.
    pub capped: bool,
    /// Candidate, inspect-only.
    pub candidate: Candidate,
}

/// Discover local AI stores as Review candidates.
#[must_use]
pub fn inspect_offers(locations: &UserLocations) -> Vec<AiOffer> {
    let mut offers = Vec::new();
    for (id, store) in (20_000_u64..).zip(discover_stores(locations)) {
        offers.push(offer_from_store(id, store.tool, store.path));
    }
    offers
}

fn offer_from_store(id: u64, tool: &str, path: PathBuf) -> AiOffer {
    let listed = list_store(&path, Limits::default());
    let title = format!("AI {} · {}", tool, path.display());
    AiOffer {
        title: title.clone(),
        selected: false,
        samples: listed.samples.clone(),
        capped: listed.capped,
        candidate: Candidate {
            id: CandidateId(id),
            kind: CandidateKind::AiSession,
            owner: CandidateOwner::Application(tool.to_owned()),
            path,
            logical_bytes: listed.logical_bytes,
            allocated_bytes: None,
            file_count: listed.file_count,
            activity: ActivityEvidence::default(),
            safety: SafetyAssessment::review(),
            rebuild: RebuildAssessment {
                cost: RebuildCost::Unknown,
                observed_duration_ms: None,
            },
            deletion: DeletionStrategy::InspectOnly,
            evidence: listing_evidence(&title, &listed.samples, listed.capped),
            user_policy: UserPolicy::NeverClean,
        },
    }
}

fn listing_evidence(title: &str, samples: &[String], capped: bool) -> Vec<Evidence> {
    let mut evidence = vec![
        Evidence::exact("ai-store-inspect", title),
        Evidence::exact(
            "ai-store-no-sqlite",
            "file contents and sqlite internals were not opened",
        ),
    ];
    if capped {
        evidence.push(Evidence::exact(
            "ai-store-capped",
            "listing hit a depth or file cap; not a complete inventory",
        ));
    }
    for sample in samples {
        evidence.push(Evidence::exact("ai-store-sample", sample));
    }
    evidence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_are_never_preselected() {
        for offer in inspect_offers(&UserLocations::current()) {
            assert!(!offer.selected);
            assert_eq!(offer.candidate.deletion, DeletionStrategy::InspectOnly);
            assert_eq!(offer.candidate.user_policy, UserPolicy::NeverClean);
        }
    }
}
