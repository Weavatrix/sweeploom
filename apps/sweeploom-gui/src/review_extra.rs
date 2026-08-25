//! Review rows that are not project analyzers: temp, Downloads, AI stores.

use sweeploom_ai::inspect_offers;
use sweeploom_dev::ReviewRow;
use sweeploom_general::collect_offers;
use sweeploom_platform::UserLocations;

/// General + AI inspect rows.
#[must_use]
pub fn extra_rows(locations: &UserLocations) -> Vec<ReviewRow> {
    let mut rows = Vec::new();
    for offer in collect_offers(locations) {
        rows.push(ReviewRow {
            candidate: offer.candidate,
            selected: offer.selected,
            title: offer.title,
        });
    }
    for offer in inspect_offers(locations) {
        rows.push(ReviewRow {
            candidate: offer.candidate,
            selected: offer.selected,
            title: offer.title,
        });
    }
    rows
}
