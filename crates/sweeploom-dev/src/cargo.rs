//! Cargo target analyzer. Git safety comes from weavatrix-git, not a local Git.

use std::path::{Path, PathBuf};

use sweeploom_core::{Blocker, ProcessSnapshot, RebuildCost};

use crate::git::{GitSafety, inspect};
use crate::size::dir_logical_bytes;

/// How aggressively generated Cargo output can be trimmed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CargoTrim {
    /// `target/incremental` only.
    Light,
    /// Incremental plus `target/debug`.
    Balanced,
    /// Whole `target` directory.
    Full,
}

/// One Cargo-generated cleanup offer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoOffer {
    /// Project root.
    pub project: PathBuf,
    /// Path that would be deleted.
    pub path: PathBuf,
    /// Trim mode.
    pub mode: CargoTrim,
    /// Logical bytes under the path.
    pub logical_bytes: u64,
    /// Rebuild cost if deleted.
    pub rebuild: RebuildCost,
    /// True when auto-select is forbidden.
    pub blocked: bool,
    /// Why it is blocked, if it is.
    pub blocker: Option<Blocker>,
}

/// Discover Cargo generated-output offers for one project.
#[must_use]
pub fn cargo_offers(project: &Path, processes: &[ProcessSnapshot]) -> Vec<CargoOffer> {
    let blocker = cargo_blocker(project, processes);
    let blocked = blocker.is_some();
    let mut offers = Vec::new();
    for target in cargo_target_dirs(project) {
        if !target.is_dir() {
            continue;
        }
        push_target_offers(&mut offers, project, &target, blocked, blocker);
    }
    offers
}

impl CargoTrim {
    /// Short UI label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Light => "incremental",
            Self::Balanced => "debug",
            Self::Full => "full target",
        }
    }
}

fn cargo_target_dirs(project: &Path) -> Vec<PathBuf> {
    let Some(primary) =
        resolve_target_dir(project, std::env::var_os("CARGO_TARGET_DIR").as_deref())
    else {
        return Vec::new();
    };
    let local = project.join("target");
    if primary == local {
        vec![primary]
    } else {
        vec![primary, local]
    }
}

fn push_target_offers(
    offers: &mut Vec<CargoOffer>,
    project: &Path,
    target: &Path,
    blocked: bool,
    blocker: Option<Blocker>,
) {
    push_offer(
        offers,
        project,
        &target.join("incremental"),
        CargoTrim::Light,
        RebuildCost::Low,
        blocked,
        blocker,
    );
    push_offer(
        offers,
        project,
        &target.join("debug"),
        CargoTrim::Balanced,
        RebuildCost::Medium,
        blocked,
        blocker,
    );
    push_offer(
        offers,
        project,
        target,
        CargoTrim::Full,
        RebuildCost::High,
        blocked,
        blocker,
    );
}

fn resolve_target_dir(project: &Path, custom: Option<&std::ffi::OsStr>) -> Option<PathBuf> {
    if !project.join("Cargo.toml").is_file() {
        return None;
    }
    if let Some(custom) = custom {
        let path = PathBuf::from(custom);
        let resolved = if path.is_absolute() {
            path
        } else {
            project.join(path)
        };
        if resolved.starts_with(project) {
            return Some(resolved);
        }
    }
    Some(project.join("target"))
}

fn cargo_blocker(project: &Path, processes: &[ProcessSnapshot]) -> Option<Blocker> {
    if processes
        .iter()
        .any(|process| process_blocks(process, project))
    {
        return Some(Blocker::ActiveProcess);
    }
    let git = inspect(project);
    if matches!(git, GitSafety::Unknown) {
        return Some(Blocker::UnknownGitState);
    }
    git.assessment().blockers.first().copied()
}

fn process_blocks(process: &ProcessSnapshot, project: &Path) -> bool {
    let Some(cwd) = &process.cwd else {
        return false;
    };
    if !cwd.starts_with(project) {
        return false;
    }
    let name = process.name.to_ascii_lowercase();
    name.contains("cargo") || name.contains("rustc")
}

fn push_offer(
    offers: &mut Vec<CargoOffer>,
    project: &Path,
    path: &Path,
    mode: CargoTrim,
    rebuild: RebuildCost,
    blocked: bool,
    blocker: Option<Blocker>,
) {
    if !path.exists() {
        return;
    }
    let logical_bytes = dir_logical_bytes(path);
    if logical_bytes == 0 {
        return;
    }
    offers.push(CargoOffer {
        project: project.to_path_buf(),
        path: path.to_path_buf(),
        mode,
        logical_bytes,
        rebuild,
        blocked,
        blocker,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_cargo_toml_yields_nothing() {
        assert!(cargo_offers(Path::new("/no-such-sweeploom-cargo"), &[]).is_empty());
    }

    #[test]
    fn discovers_debug_and_full_target() {
        use std::fs;
        let root = std::env::temp_dir().join(format!("sweeploom-cargo-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("target").join("debug")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();
        fs::write(
            root.join("target").join("debug").join("a.rlib"),
            vec![0_u8; 2048],
        )
        .unwrap();
        let offers = cargo_offers(&root, &[]);
        assert!(offers.iter().any(|item| item.mode == CargoTrim::Balanced));
        assert!(offers.iter().any(|item| item.mode == CargoTrim::Full));
        assert!(offers.iter().all(|item| item.logical_bytes >= 2048));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn custom_target_outside_project_falls_back() {
        use std::fs;
        let root = std::env::temp_dir().join(format!("sweeploom-cargo-out-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();
        let outside = std::env::temp_dir().join("sweeploom-shared-target");
        let resolved = resolve_target_dir(&root, Some(outside.as_os_str())).expect("toml");
        assert_eq!(resolved, root.join("target"));
        let inside = resolve_target_dir(&root, Some(std::ffi::OsStr::new("custom-target")))
            .expect("relative");
        assert_eq!(inside, root.join("custom-target"));
        let _ = fs::remove_dir_all(&root);
    }
}
