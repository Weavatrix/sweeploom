//! Semantic developer analyzers. Git evidence comes from weavatrix-git.

#![cfg_attr(not(test), warn(missing_docs))]

mod cargo;
mod cargo_manifest;
mod git;
mod node;
mod project;
mod python;
mod review;
mod size;

pub use cargo::{CargoOffer, CargoTrim, cargo_offers};
pub use cargo_manifest::{CargoManifest, read_manifest, resolved_target_dir, workspace_root};
pub use git::{GitSafety, inspect};
pub use node::{NodeOffer, node_offers};
pub use project::{DevKind, classify_project};
pub use python::{PythonOffer, python_offers};
pub use review::{ReviewRow, collect_review, collect_review_from};
pub use size::{dir_logical_bytes, path_mtime};
