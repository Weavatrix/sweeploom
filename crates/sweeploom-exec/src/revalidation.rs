//! Revalidate a plan entry against live metadata. Fail closed.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use sweeploom_core::{
    Blocker, CleanPlanEntry, ExecutionContext, MetadataRevision, ProcessEvidence, ProcessSnapshot,
    SafetyPrecondition, SkipReason, authorize_generated,
};
use sweeploom_platform::file_identity;

/// Revalidate a single entry. Fail closed.
#[must_use]
pub fn revalidate(entry: &CleanPlanEntry) -> Option<SkipReason> {
    revalidate_with(entry, &ExecutionContext::unknown())
}

/// Revalidate with shared evidence.
#[must_use]
pub fn revalidate_with(entry: &CleanPlanEntry, ctx: &ExecutionContext<'_>) -> Option<SkipReason> {
    let path = &entry.path;
    if is_symlink(path) {
        return Some(SkipReason::Changed);
    }
    if !path.exists() {
        return Some(SkipReason::Missing);
    }
    let is_dir = path.is_dir();
    if is_dir != entry.expected_is_dir {
        return Some(SkipReason::Changed);
    }
    if let Some(owner) = &entry.owner_path
        && authorize_generated(owner, path, None).is_err()
    {
        return Some(SkipReason::Blocked(Blocker::OutOfArtifactScope));
    }
    match file_identity(path) {
        Some(live) => {
            if let Some(expected) = entry.expected_identity
                && live != expected
            {
                return Some(SkipReason::Changed);
            }
        }
        None => {
            if entry
                .required_safety
                .contains(&SafetyPrecondition::FileIdentityMatch)
            {
                return Some(SkipReason::UnknownEvidence);
            }
        }
    }
    if let Some(reason) = revision_changed(entry, path) {
        return Some(reason);
    }
    if entry
        .required_safety
        .contains(&SafetyPrecondition::NoActiveProcess)
    {
        match ctx.processes {
            ProcessEvidence::Unknown => {
                return Some(SkipReason::UnknownEvidence);
            }
            ProcessEvidence::Observed(processes) => {
                if process_blocks(path, processes) {
                    return Some(SkipReason::Blocked(Blocker::ActiveProcess));
                }
            }
        }
    }
    if entry
        .required_safety
        .contains(&SafetyPrecondition::GitStateUnchanged)
    {
        let assessment = sweeploom_dev::inspect(path).assessment();
        if let Some(blocker) = assessment.blockers.first() {
            return Some(SkipReason::Blocked(*blocker));
        }
    }
    None
}

fn revision_changed(entry: &CleanPlanEntry, path: &Path) -> Option<SkipReason> {
    let live = capture_revision(path);
    if !live.complete {
        return Some(SkipReason::UnknownEvidence);
    }
    if let Ok(meta) = fs::metadata(path) {
        if meta.is_file() && meta.len() != entry.expected_bytes {
            return Some(SkipReason::Changed);
        }
        if let Some(expected) = entry.expected_latest_write
            && let Some(max) = live_max_mtime(&live, &meta)
            && max > expected
        {
            return Some(SkipReason::Changed);
        }
    } else {
        return Some(SkipReason::UnknownEvidence);
    }
    if let Some(expected) = entry.expected_revision {
        if expected.complete
            && (expected.file_count != live.file_count
                || expected.logical_bytes != live.logical_bytes
                || expected.max_mtime_unix_ms != live.max_mtime_unix_ms)
        {
            return Some(SkipReason::Changed);
        }
        if !expected.complete {
            return Some(SkipReason::UnknownEvidence);
        }
    }
    None
}

fn live_max_mtime(live: &MetadataRevision, meta: &fs::Metadata) -> Option<SystemTime> {
    live.max_mtime_unix_ms
        .map(|ms| UNIX_EPOCH + std::time::Duration::from_millis(ms))
        .or_else(|| meta.modified().ok())
}

fn process_blocks(path: &Path, processes: &[ProcessSnapshot]) -> bool {
    processes.iter().any(|process| process.uses_path(path))
}

pub(crate) fn capture_revision(path: &Path) -> MetadataRevision {
    let Ok(meta) = fs::metadata(path) else {
        return MetadataRevision {
            file_count: 0,
            logical_bytes: 0,
            max_mtime_unix_ms: None,
            complete: false,
        };
    };
    if meta.is_file() {
        return MetadataRevision {
            file_count: 1,
            logical_bytes: meta.len(),
            max_mtime_unix_ms: mtime_ms(meta.modified().ok()),
            complete: true,
        };
    }
    let mut files = 0_u64;
    let mut bytes = 0_u64;
    let mut max_mtime = meta.modified().ok();
    let mut complete = true;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            complete = false;
            continue;
        };
        for entry in entries.flatten() {
            let child = entry.path();
            let Ok(child_meta) = fs::symlink_metadata(&child) else {
                complete = false;
                continue;
            };
            if child_meta.file_type().is_symlink() {
                continue;
            }
            DirectoryNode::bump(&mut max_mtime, child_meta.modified().ok());
            if child_meta.is_dir() {
                stack.push(child);
            } else {
                files += 1;
                bytes = bytes.saturating_add(child_meta.len());
            }
        }
    }
    MetadataRevision {
        file_count: files,
        logical_bytes: bytes,
        max_mtime_unix_ms: mtime_ms(max_mtime),
        complete,
    }
}

struct DirectoryNode;

impl DirectoryNode {
    fn bump(slot: &mut Option<SystemTime>, candidate: Option<SystemTime>) {
        match (*slot, candidate) {
            (None, Some(value)) => *slot = Some(value),
            (Some(current), Some(value)) if value > current => *slot = Some(value),
            _ => {}
        }
    }
}

fn mtime_ms(time: Option<SystemTime>) -> Option<u64> {
    time.and_then(|stamp| stamp.duration_since(UNIX_EPOCH).ok())
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
}

pub(crate) fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}
