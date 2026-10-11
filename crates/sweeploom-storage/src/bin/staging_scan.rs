//! Read-only JSON report of generated deployment builds for Hostwatch.

use std::path::Path;
use std::time::{Duration, SystemTime};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(root) = args.next() else {
        eprintln!("usage: sweeploom-staging-scan ROOT OLDER_HOURS");
        std::process::exit(2);
    };
    let Some(hours) = args.next().and_then(|value| value.parse::<u64>().ok()) else {
        eprintln!("OLDER_HOURS must be a positive integer");
        std::process::exit(2);
    };
    if args.next().is_some() || hours == 0 {
        eprintln!("usage: sweeploom-staging-scan ROOT OLDER_HOURS");
        std::process::exit(2);
    }
    let before = SystemTime::now() - Duration::from_secs(hours.saturating_mul(3600));
    match sweeploom_storage::scan_staging_builds(Path::new(&root), before) {
        Ok(found) => println!(
            "{}",
            serde_json::to_string(&found).expect("serialize report")
        ),
        Err(error) => {
            eprintln!("staging scan failed: {error}");
            std::process::exit(1);
        }
    }
}
