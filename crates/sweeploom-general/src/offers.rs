//! Top-level temp and Downloads offers. Downloads stay REVIEW.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use sweeploom_core::{
    ActivityEvidence, Candidate, CandidateId, CandidateKind, CandidateOwner, DeletionStrategy,
    Evidence, RebuildAssessment, RebuildCost, SafetyAssessment, UserPolicy,
};
use sweeploom_platform::UserLocations;

use crate::roots::{GeneralRoot, default_roots};

const TEMP_AGE: Duration = Duration::from_secs(7 * 24 * 3600);
const DOWNLOAD_AGE: Duration = Duration::from_secs(30 * 24 * 3600);
const MAX_PER_ROOT: usize = 80;

/// One general cleaner row ready for Review.
#[derive(Clone, Debug)]
pub struct GeneralOffer {
    /// Human title.
    pub title: String,
    /// Pre-selected only for obvious SAFE temp.
    pub selected: bool,
    /// Cleanup candidate.
    pub candidate: Candidate,
}

/// Collect temp and Downloads offers for this machine.
#[must_use]
pub fn collect_offers(locations: &UserLocations) -> Vec<GeneralOffer> {
    collect_offers_at(&default_roots(locations), SystemTime::now(), 10_000)
}

/// Collect offers from explicit roots. `id_base` avoids colliding with project ids.
#[must_use]
pub fn collect_offers_at(
    roots: &[GeneralRoot],
    now: SystemTime,
    id_base: u64,
) -> Vec<GeneralOffer> {
    let mut offers = Vec::new();
    let mut next = id_base;
    for root in roots {
        if !root.path.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&root.path) else {
            continue;
        };
        for entry in entries.flatten() {
            if offers.len() >= MAX_PER_ROOT * roots.len() {
                break;
            }
            if let Some(offer) = offer_from_entry(root, &entry.path(), now, CandidateId(next)) {
                offers.push(offer);
                next += 1;
            }
        }
    }
    offers
}

fn offer_from_entry(
    root: &GeneralRoot,
    path: &Path,
    now: SystemTime,
    id: CandidateId,
) -> Option<GeneralOffer> {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return None;
    };
    if meta.file_type().is_symlink() || meta.is_dir() {
        return None;
    }
    let modified = meta.modified().ok();
    let age = modified.and_then(|stamp| now.duration_since(stamp).ok());
    let logical = meta.len();
    if logical == 0 {
        return None;
    }
    match root.id {
        "user-temp" => temp_offer(root, path, logical.max(1), modified, age, id),
        "downloads" => download_offer(path, logical.max(1), modified, age, id),
        "crash-dumps" => dump_offer(path, logical.max(1), modified, age, id),
        _ => None,
    }
}

fn temp_offer(
    root: &GeneralRoot,
    path: &Path,
    logical: u64,
    modified: Option<SystemTime>,
    age: Option<Duration>,
    id: CandidateId,
) -> Option<GeneralOffer> {
    if !is_obvious_temp(path) {
        return None;
    }
    let stale = age.is_some_and(|item| item >= TEMP_AGE);
    let selected = root.auto_select_allowed && stale;
    Some(build(
        id,
        CandidateKind::TempFile,
        path,
        logical,
        modified,
        SafetyAssessment::safe(),
        DeletionStrategy::PermanentGenerated,
        RebuildCost::Low,
        selected,
        format!("Temp · {}", path.display()),
        "general-temp",
    ))
}

fn dump_offer(
    path: &Path,
    logical: u64,
    modified: Option<SystemTime>,
    age: Option<Duration>,
    id: CandidateId,
) -> Option<GeneralOffer> {
    let name = path
        .file_name()
        .and_then(|item| item.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !name.ends_with(".dmp") {
        return None;
    }
    let stale = age.is_some_and(|item| item >= TEMP_AGE);
    Some(build(
        id,
        CandidateKind::CrashDump,
        path,
        logical,
        modified,
        SafetyAssessment::safe(),
        DeletionStrategy::PermanentGenerated,
        RebuildCost::None,
        stale,
        format!("Crash dump · {}", path.display()),
        "general-crash-dump",
    ))
}

fn download_offer(
    path: &Path,
    logical: u64,
    modified: Option<SystemTime>,
    age: Option<Duration>,
    id: CandidateId,
) -> Option<GeneralOffer> {
    if !is_installer_or_archive(path) {
        return None;
    }
    if age.is_some_and(|item| item < DOWNLOAD_AGE) {
        return None;
    }
    Some(build(
        id,
        CandidateKind::OldInstaller,
        path,
        logical,
        modified,
        SafetyAssessment::review(),
        DeletionStrategy::InspectOnly,
        RebuildCost::None,
        false,
        format!("Downloads · {}", path.display()),
        "general-downloads",
    ))
}

fn is_obvious_temp(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|item| item.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    name.ends_with(".tmp")
        || name.ends_with(".temp")
        || name.ends_with(".dmp")
        || name.ends_with(".log")
        || name.contains("tmp")
        || name.contains("temp")
        || name == "crashdumps"
}

fn is_installer_or_archive(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|item| item.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    [
        ".exe", ".msi", ".dmg", ".pkg", ".deb", ".rpm", ".zip", ".7z", ".rar", ".iso",
    ]
    .iter()
    .any(|ext| name.ends_with(ext))
}

#[allow(clippy::too_many_arguments)]
fn build(
    id: CandidateId,
    kind: CandidateKind,
    path: &Path,
    logical: u64,
    modified: Option<SystemTime>,
    safety: SafetyAssessment,
    deletion: DeletionStrategy,
    rebuild: RebuildCost,
    selected: bool,
    title: String,
    evidence: &str,
) -> GeneralOffer {
    GeneralOffer {
        title: title.clone(),
        selected,
        candidate: Candidate {
            id,
            kind,
            owner: CandidateOwner::User,
            path: path.to_path_buf(),
            logical_bytes: logical,
            allocated_bytes: None,
            file_count: 1,
            activity: ActivityEvidence {
                latest_any_modified: modified,
                ..ActivityEvidence::default()
            },
            safety,
            rebuild: RebuildAssessment {
                cost: rebuild,
                observed_duration_ms: None,
            },
            deletion,
            evidence: vec![Evidence::exact(evidence, title)],
            user_policy: UserPolicy::Default,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn temp_tmp_is_offered_downloads_exe_is_review() {
        let root = std::env::temp_dir().join(format!("sweeploom-gen-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let temp = root.join("temp");
        let downloads = root.join("Downloads");
        fs::create_dir_all(&temp).unwrap();
        fs::create_dir_all(&downloads).unwrap();
        fs::write(temp.join("stale.tmp"), vec![0_u8; 128]).unwrap();
        fs::write(downloads.join("Setup.exe"), vec![0_u8; 256]).unwrap();
        let now = SystemTime::now() + Duration::from_secs(40 * 24 * 3600);
        let offers = collect_offers_at(
            &[
                GeneralRoot {
                    id: "user-temp",
                    path: temp,
                    auto_select_allowed: true,
                },
                GeneralRoot {
                    id: "downloads",
                    path: downloads,
                    auto_select_allowed: false,
                },
            ],
            now,
            50,
        );
        let tmp = offers
            .iter()
            .find(|item| item.title.contains("stale.tmp"))
            .expect("temp");
        assert!(tmp.selected);
        assert!(!tmp.candidate.safety.is_blocked());
        let exe = offers
            .iter()
            .find(|item| item.title.contains("Setup.exe"))
            .expect("downloads");
        assert!(!exe.selected);
        assert_eq!(exe.candidate.deletion, DeletionStrategy::InspectOnly);
        let _ = fs::remove_dir_all(&root);
    }
}
