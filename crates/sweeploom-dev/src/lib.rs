//! Semantic developer analyzers. Git evidence comes from weavatrix-git.

#![cfg_attr(not(test), warn(missing_docs))]

mod cargo;
mod git;
mod project;

pub use cargo::{CargoOffer, CargoTrim, cargo_offers};
pub use git::{GitSafety, inspect};
pub use project::{DevKind, classify_project};
