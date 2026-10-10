//! Core topology: physical count and the Apple Silicon P/E split. Read once.

use std::sync::OnceLock;

/// Logical cores per performance level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoreSplit {
    /// Performance cores (`hw.perflevel0.logicalcpu`).
    pub performance: usize,
    /// Efficiency cores (`hw.perflevel1.logicalcpu`).
    pub efficiency: usize,
}

impl CoreSplit {
    /// True when `index` (OS order) is an efficiency core. macOS numbers the
    /// efficiency cluster first (IODeviceTree `cluster-type` = E for cpu0..).
    #[must_use]
    pub const fn is_efficiency(self, index: usize) -> bool {
        index < self.efficiency
    }
}

/// Physical core count and P/E split, cached for the process lifetime.
pub(super) fn topology() -> (Option<usize>, Option<CoreSplit>) {
    static CACHE: OnceLock<(Option<usize>, Option<CoreSplit>)> = OnceLock::new();
    *CACHE.get_or_init(|| (sysinfo::System::physical_core_count(), detect_split()))
}

#[cfg(target_os = "macos")]
fn detect_split() -> Option<CoreSplit> {
    let output = std::process::Command::new("/usr/sbin/sysctl")
        .args(["-n", "hw.perflevel0.logicalcpu", "hw.perflevel1.logicalcpu"])
        .output()
        .ok()?;
    parse_split(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(not(target_os = "macos"))]
fn detect_split() -> Option<CoreSplit> {
    None
}

fn parse_split(text: &str) -> Option<CoreSplit> {
    let mut lines = text.lines().map(|line| line.trim().parse::<usize>().ok());
    let performance = lines.next()??;
    let efficiency = lines.next()??;
    (performance > 0 && efficiency > 0).then_some(CoreSplit {
        performance,
        efficiency,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_needs_both_levels() {
        assert_eq!(
            parse_split("4\n6\n"),
            Some(CoreSplit {
                performance: 4,
                efficiency: 6
            })
        );
        assert_eq!(parse_split("8\n"), None);
        assert_eq!(parse_split(""), None);
        assert!(parse_split("4\n6").unwrap().is_efficiency(5));
        assert!(!parse_split("4\n6").unwrap().is_efficiency(6));
    }
}
