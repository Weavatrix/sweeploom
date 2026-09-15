//! `sweeploom ai` — inspect-only listing of local AI stores.

use std::time::SystemTime;

use sweeploom_ai::{advise_context, estimated_prompt_tokens_with_files, inspect_offers};
use sweeploom_core::DeletionStrategy;
use sweeploom_platform::UserLocations;

use crate::bytes::format_bytes;

pub fn run() {
    let offers = inspect_offers(&UserLocations::current());
    if offers.is_empty() {
        println!("no local AI stores under the home directory");
        return;
    }
    for offer in offers {
        println!(
            "[ ] {} files={} size={} capped={} inspect-only",
            offer.title,
            offer.candidate.file_count,
            format_bytes(offer.candidate.logical_bytes),
            offer.capped
        );
        for entry in &offer.entries {
            let policy = if entry.candidate.deletion == DeletionStrategy::InspectOnly {
                "inspect-only"
            } else {
                "cleanable"
            };
            let tokens = estimated_prompt_tokens_with_files(
                entry.class,
                &entry.relative,
                entry.candidate.logical_bytes,
                entry.candidate.file_count,
            );
            let tokens = if tokens == 0 {
                String::new()
            } else {
                format!(" ~{tokens} tok")
            };
            let idle = entry
                .candidate
                .activity
                .latest_any_modified
                .and_then(|stamp| SystemTime::now().duration_since(stamp).ok());
            let advice = advise_context(&entry.relative, entry.class, idle);
            let advice = match advice.label() {
                "" => String::new(),
                label => format!(" {label}"),
            };
            println!(
                "    [{}] {} {} files={} size={}{tokens}{advice} {policy}",
                if entry.selected { "x" } else { " " },
                entry.class.label(),
                entry.relative,
                entry.candidate.file_count,
                format_bytes(entry.candidate.logical_bytes)
            );
        }
    }
}
