//! Inspect-first review rows for discovered AI stores.

use sweeploom_core::{
    ActivityEvidence, Blocker, Candidate, CandidateId, CandidateKind, CandidateOwner,
    DeletionStrategy, Evidence, RebuildAssessment, RebuildCost, SafetyAssessment, UserPolicy,
};
use sweeploom_platform::UserLocations;

use std::path::PathBuf;
use std::time::SystemTime;

use crate::classify::{AiClass, classify_name};
use crate::discover_stores;
use crate::inventory::{Limits, StoreEntry, list_store};

/// One inspect-only AI store row, plus classified children.
#[derive(Clone, Debug)]
pub struct AiOffer {
    /// Human title.
    pub title: String,
    /// Always false. Internal DBs are never auto-selected.
    pub selected: bool,
    /// True when the walk hit a depth, file, or entry cap.
    pub capped: bool,
    /// Candidate for the store root. Always inspect-only.
    pub candidate: Candidate,
    /// Immediate children. Cache/log rows may be cleaned one at a time.
    pub entries: Vec<AiEntry>,
}

/// One classified child of an AI store.
#[derive(Clone, Debug)]
pub struct AiEntry {
    /// Path relative to the store root.
    pub relative: String,
    /// Classification.
    pub class: AiClass,
    /// True when the user checked this row. Never pre-set.
    pub selected: bool,
    /// Candidate. Secrets/sqlite stay inspect-only.
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
    let title = format!("AI {tool} · {}", path.display());
    let names: Vec<String> = listed
        .entries
        .iter()
        .map(|item| item.relative.clone())
        .collect();
    let entries = listed
        .entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            entry_from(
                id.saturating_add(1_000)
                    .saturating_add(u64::try_from(index).unwrap_or(u64::MAX)),
                tool,
                &path,
                entry,
            )
        })
        .collect();
    AiOffer {
        title: title.clone(),
        selected: false,
        capped: listed.capped,
        entries,
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
            evidence: listing_evidence(&title, &names, listed.capped),
            user_policy: UserPolicy::NeverClean,
        },
    }
}

fn entry_from(id: u64, tool: &str, store: &std::path::Path, listed: StoreEntry) -> AiEntry {
    let class = classify_name(&listed.relative);
    let path = store.join(listed.relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    let mtime = path_mtime(&path);
    let title = format!("AI {tool} · {}", listed.relative);
    let (kind, deletion, safety, policy) = if class.can_clean() {
        (
            match class {
                AiClass::Log => CandidateKind::Log,
                _ => CandidateKind::AiCache,
            },
            DeletionStrategy::PermanentGenerated,
            SafetyAssessment::review(),
            UserPolicy::AskEveryTime,
        )
    } else if matches!(class, AiClass::Secret | AiClass::Sqlite) {
        (
            CandidateKind::AiSession,
            DeletionStrategy::InspectOnly,
            SafetyAssessment::blocked(Blocker::ProtectedPath),
            UserPolicy::NeverClean,
        )
    } else {
        (
            CandidateKind::AiSession,
            DeletionStrategy::InspectOnly,
            SafetyAssessment::review(),
            UserPolicy::NeverClean,
        )
    };
    AiEntry {
        relative: listed.relative,
        class,
        selected: false,
        candidate: Candidate {
            id: CandidateId(id),
            kind,
            owner: CandidateOwner::Application(tool.to_owned()),
            path,
            logical_bytes: listed.logical_bytes,
            allocated_bytes: None,
            file_count: listed.file_count,
            activity: ActivityEvidence {
                latest_any_modified: mtime,
                ..ActivityEvidence::default()
            },
            safety,
            rebuild: RebuildAssessment {
                cost: RebuildCost::Low,
                observed_duration_ms: None,
            },
            deletion,
            evidence: vec![Evidence::exact("ai-store-entry", title)],
            user_policy: policy,
        },
    }
}

fn path_mtime(path: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}

fn listing_evidence(title: &str, names: &[String], capped: bool) -> Vec<Evidence> {
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
    for name in names.iter().take(32) {
        evidence.push(Evidence::exact("ai-store-entry", name));
    }
    evidence
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn offers_are_never_preselected() {
        let root = std::env::temp_dir().join(format!(
            "sweeploom-ai-empty-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|item| item.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(root.join("cache")).unwrap();
        fs::write(root.join("cache").join("a"), b"aa").unwrap();
        let offer = offer_from_store(1, "claude", root.clone());
        assert!(!offer.selected);
        assert_eq!(offer.candidate.deletion, DeletionStrategy::InspectOnly);
        assert_eq!(offer.candidate.user_policy, UserPolicy::NeverClean);
        assert!(offer.entries.iter().all(|entry| !entry.selected));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cache_child_is_explicit_clean_and_secrets_are_blocked() {
        let root = std::env::temp_dir().join(format!(
            "sweeploom-ai-offer-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|item| item.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(root.join("cache")).unwrap();
        fs::write(root.join("cache").join("a"), b"aa").unwrap();
        fs::write(root.join(".credentials.json"), b"no").unwrap();
        let offer = offer_from_store(1, "claude", root.clone());
        let cache = offer
            .entries
            .iter()
            .find(|item| item.relative == "cache")
            .expect("cache");
        let secret = offer
            .entries
            .iter()
            .find(|item| item.relative == ".credentials.json")
            .expect("secret");
        assert!(!cache.selected);
        assert_eq!(
            cache.candidate.deletion,
            DeletionStrategy::PermanentGenerated
        );
        assert!(!cache.candidate.safety.is_blocked());
        assert_eq!(secret.candidate.deletion, DeletionStrategy::InspectOnly);
        assert!(secret.candidate.safety.is_blocked());
        assert_eq!(offer.candidate.deletion, DeletionStrategy::InspectOnly);
        let _ = fs::remove_dir_all(&root);
    }
}
