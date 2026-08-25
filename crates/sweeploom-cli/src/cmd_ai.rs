//! `sweeploom ai` — inspect-only listing of local AI stores.

use sweeploom_ai::inspect_offers;
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
        for sample in &offer.samples {
            println!("    sample {sample}");
        }
    }
}
