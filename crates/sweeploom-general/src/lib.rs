//! General (non-project) cleaner roots. User files stay REVIEW.

#![cfg_attr(not(test), warn(missing_docs))]

mod offers;
mod roots;

pub use offers::{GeneralOffer, collect_offers};
pub use roots::{GeneralRoot, default_roots};
