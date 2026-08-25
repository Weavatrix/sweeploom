//! CPU windows from SweepLoom's own rings. Never back-filled.

use crate::Sample;

/// Observed CPU for one process. Missing windows stay `None`, not zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CpuSummary {
    /// Latest sample.
    pub now: f32,
    /// Peak among fast samples.
    pub peak: f32,
    /// Average over the last 5 minutes, if watched long enough.
    pub avg_5m: Option<f32>,
    /// Average over the last hour, if watched long enough.
    pub avg_1h: Option<f32>,
    /// Fast-ring length.
    pub fast_samples: usize,
}

/// Summarize CPU. `now_ms` is the latest sample timestamp.
#[must_use]
pub fn summarize_cpu(fast: &[Sample], slow: &[Sample], now_ms: u64) -> CpuSummary {
    CpuSummary {
        now: fast.last().map(|item| item.cpu_percent).unwrap_or(0.0),
        peak: fast
            .iter()
            .map(|item| item.cpu_percent)
            .fold(0.0_f32, f32::max),
        avg_5m: window_avg(fast, now_ms, 5 * 60 * 1000, 2 * 60 * 1000),
        avg_1h: window_avg(slow, now_ms, 60 * 60 * 1000, 20 * 60 * 1000),
        fast_samples: fast.len(),
    }
}

fn window_avg(samples: &[Sample], now_ms: u64, window_ms: u64, min_span_ms: u64) -> Option<f32> {
    let start = now_ms.saturating_sub(window_ms);
    let in_window: Vec<&Sample> = samples
        .iter()
        .filter(|item| item.at_unix_ms >= start)
        .collect();
    if in_window.len() < 2 {
        return None;
    }
    let first = in_window.first()?.at_unix_ms;
    let last = in_window.last()?.at_unix_ms;
    if last.saturating_sub(first) < min_span_ms {
        return None;
    }
    let sum: f32 = in_window.iter().map(|item| item.cpu_percent).sum();
    Some(sum / in_window.len() as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(at: u64, cpu: f32) -> Sample {
        Sample {
            at_unix_ms: at,
            cpu_percent: cpu,
            rss_bytes: 1,
        }
    }

    #[test]
    fn short_watch_does_not_invent_averages() {
        let fast = [sample(1_000, 10.0), sample(2_000, 20.0)];
        let summary = summarize_cpu(&fast, &[], 2_000);
        assert_eq!(summary.now, 20.0);
        assert_eq!(summary.peak, 20.0);
        assert_eq!(summary.avg_5m, None);
        assert_eq!(summary.avg_1h, None);
    }

    #[test]
    fn five_minute_average_needs_span() {
        let fast = [sample(0, 10.0), sample(60_000, 20.0), sample(120_000, 30.0)];
        let summary = summarize_cpu(&fast, &[], 120_000);
        assert_eq!(summary.avg_5m, Some(20.0));
        assert_eq!(summary.avg_1h, None);
    }
}
