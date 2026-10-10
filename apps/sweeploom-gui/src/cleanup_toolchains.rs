//! Uninstall individual Rust versions through rustup; protect active/default compilers.
use super::{Item, Kind};
use std::{path::Path, process::Command};

fn run(program: &str, home: &Path, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new(program);
    // Inventory must never install the toolchain pinned by the GUI's working directory.
    command
        .args(args)
        .current_dir(home)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("RUSTUP_TOOLCHAIN", "stable");
    super::commands::execute(program, command)
}

pub(super) fn classify(home: &Path, items: &mut [Item]) {
    let program = home.join(".cargo/bin/rustup");
    if !program.is_file() {
        return;
    }
    let program = program.to_string_lossy();
    let listing = run(&program, home, &["toolchain", "list"]);
    for item in items
        .iter_mut()
        .filter(|item| item.name.starts_with("Rust toolchain"))
    {
        let Some(path) = &item.path else {
            continue;
        };
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        match &listing {
            Ok(listing) => {
                item.kind = Kind::Toolchain;
                item.id = name.to_string();
                item.scope = Some(program.to_string());
                item.enabled = can_remove(listing, &name).is_ok();
                item.status = if item.enabled {
                    "Uninstall this compiler with rustup; projects pinned to it will need reinstalling".into()
                } else {
                    "Active/default compiler or unavailable inventory — cannot uninstall".into()
                };
            }
            Err(error) => item.status = format!("Cannot verify rustup inventory: {error}"),
        }
    }
}

fn can_remove(listing: &str, name: &str) -> Result<(), String> {
    let line = listing
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .ok_or("Toolchain is no longer installed")?;
    let annotation = line
        .split_once(' ')
        .map_or("", |(_, annotation)| annotation);
    if annotation.contains("default") || annotation.contains("active") {
        return Err("Active/default toolchain cannot be uninstalled".into());
    }
    Ok(())
}

pub(super) fn remove(
    item: &Item,
    processes: &[sweeploom_core::ProcessSnapshot],
) -> Result<String, String> {
    let path = item.path.as_ref().ok_or("Toolchain path unavailable")?;
    if processes.iter().any(|process| process.uses_path(path)) {
        return Err("A running process uses this toolchain".into());
    }
    let home = path.ancestors().nth(3).ok_or("Invalid toolchain path")?;
    let program = item.scope.as_deref().ok_or("rustup identity unavailable")?;
    let listing = run(program, home, &["toolchain", "list"])?;
    can_remove(&listing, &item.id)?;
    run(program, home, &["toolchain", "uninstall", &item.id])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn active_default_and_missing_toolchains_are_protected() {
        let listing = "stable-host (active, default)\nnightly-host (active)\n1.85-host\n1.84-host (default)\n";
        assert!(can_remove(listing, "stable-host").is_err());
        assert!(can_remove(listing, "nightly-host").is_err());
        assert!(can_remove(listing, "1.84-host").is_err());
        assert!(can_remove(listing, "missing").is_err());
        assert!(can_remove(listing, "1.85-host").is_ok());
    }
}
