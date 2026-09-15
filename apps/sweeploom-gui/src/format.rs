//! Display helpers shared by every screen.

use std::path::Path;

use sweeploom_core::SafetyAssessment;

/// Human-readable byte count for cards and tables.
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    format_bytes_inner(bytes)
}

/// Lower-bound size. Incomplete walks must not look exact.
#[must_use]
pub fn format_bytes_bound(bytes: u64, complete: bool) -> String {
    let text = format_bytes_inner(bytes);
    if complete {
        text
    } else {
        format!("≥ {text}")
    }
}

fn format_bytes_inner(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let value = bytes as f64;
    if value >= GIB {
        format!("{:.1} GB", value / GIB)
    } else if value >= MIB {
        format!("{:.1} MB", value / MIB)
    } else {
        format!("{} KB", (value / 1024.0).round())
    }
}

/// Drop the Windows `\\?\` prefix so paths fit the name column.
#[must_use]
pub fn short_path(path: &Path) -> String {
    let raw = path.display().to_string();
    raw.strip_prefix(r"\\?\").unwrap_or(&raw).to_owned()
}

/// Title before the first ` · ` separator.
#[must_use]
pub fn row_caption(title: &str) -> String {
    title.split(" · ").next().unwrap_or(title).to_owned()
}

/// True when `text` is a global scan/rebuild status that should not occupy a page.
#[must_use]
pub fn is_scan_chatter(text: &str) -> bool {
    let text = text.trim();
    text.ends_with("candidates")
        || text.contains("Folders ready")
        || text.starts_with("Scanning ")
        || text.starts_with("Rebuilding review in the background")
}

/// Human safety cell. Avoids Debug truncation.
#[must_use]
pub fn safety_text(assessment: &SafetyAssessment) -> String {
    if assessment.is_blocked() {
        let reason = assessment
            .blockers
            .iter()
            .map(|item| item.label())
            .collect::<Vec<_>>()
            .join(", ");
        if reason.is_empty() {
            "Blocked".to_owned()
        } else {
            format!("Blocked · {reason}")
        }
    } else {
        assessment.level.label().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_status_is_chatter() {
        assert!(is_scan_chatter("252 candidates"));
        assert!(is_scan_chatter(
            "Folders ready. Building Review in the background…"
        ));
        assert!(!is_scan_chatter("selected 1.2 GB toward 2.0 GB"));
        assert!(!is_scan_chatter("deleted=1 skipped_changed=0 failed=0"));
    }
}
