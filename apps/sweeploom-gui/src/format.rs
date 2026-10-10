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
    const GB: f64 = 1_000_000_000.0;
    const MB: f64 = 1_000_000.0;
    let value = bytes as f64;
    if value >= GB {
        format!("{:.1} GB", value / GB)
    } else if value >= MB {
        format!("{:.1} MB", value / MB)
    } else {
        format!("{} KB", (value / 1000.0).round())
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

/// Keep the filename visible when several candidates share a category.
#[must_use]
pub fn candidate_caption(title: &str, path: &Path) -> String {
    let caption = row_caption(title);
    match path.file_name() {
        Some(name) if name.to_string_lossy() != caption => {
            format!("{caption} · {}", name.to_string_lossy())
        }
        _ => caption,
    }
}

/// True when `text` is a global scan/rebuild status that should not occupy a page.
#[must_use]
pub fn is_scan_chatter(text: &str) -> bool {
    let text = text.trim();
    text.ends_with("candidates")
        || text.contains("Folders ready")
        || text.starts_with("Scanning ")
        || text.starts_with("Rebuilding review in the background")
        || text.starts_with("Projects found")
        || text.starts_with("Sizing AI stores")
}

/// "just now", "12 min ago", "3 h ago", "4 d ago" between two unix-ms stamps.
#[must_use]
pub fn ago(now_ms: u64, at_ms: u64) -> String {
    let secs = now_ms.saturating_sub(at_ms) / 1000;
    match secs {
        0..60 => "just now".into(),
        60..3600 => format!("{} min ago", secs / 60),
        3600..86_400 => format!("{} h ago", secs / 3600),
        _ => format!("{} d ago", secs / 86_400),
    }
}

/// Home-relative path for compact captions.
#[must_use]
pub fn tilde(path: &Path, home: &Path) -> String {
    path.strip_prefix(home).map_or_else(
        |_| short_path(path),
        |rest| format!("~/{}", rest.display()),
    )
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
    fn gigabytes_use_decimal_disk_units() {
        assert_eq!(format_bytes(1_000_000_000), "1.0 GB");
    }

    #[test]
    fn downloads_keep_distinct_filenames() {
        let first = Path::new("/Users/example/Downloads/Browser.dmg");
        let second = Path::new("/Users/example/Downloads/Editor.zip");
        assert_eq!(
            candidate_caption(&format!("Downloads · {}", first.display()), first),
            "Downloads · Browser.dmg"
        );
        assert_eq!(
            candidate_caption(&format!("Downloads · {}", second.display()), second),
            "Downloads · Editor.zip"
        );
    }

    #[test]
    fn relative_time_and_home_paths_stay_short() {
        assert_eq!(ago(10_000, 5_000), "just now");
        assert_eq!(ago(3_600_000 * 3 + 5, 5), "3 h ago");
        assert_eq!(
            tilde(Path::new("/Users/a/.codex"), Path::new("/Users/a")),
            "~/.codex"
        );
    }

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
