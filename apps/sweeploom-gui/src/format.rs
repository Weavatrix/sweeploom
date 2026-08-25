//! Display helpers shared by every screen.

use std::path::Path;

use sweeploom_core::SafetyAssessment;

/// Human-readable byte count for cards and tables.
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
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
