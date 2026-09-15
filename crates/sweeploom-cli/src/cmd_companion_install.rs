//! `sweeploom companion-install` — write host manifests and register them.

use std::io;

use sweeploom_browser::{install_native_host, resolve_chromium_id, sibling_host_exe};
use sweeploom_platform::UserLocations;

pub fn run(args: impl Iterator<Item = String>) {
    if let Err(error) = install(parse_chromium_id(args)) {
        eprintln!("companion-install failed: {error}");
        std::process::exit(1);
    }
}

fn parse_chromium_id(args: impl Iterator<Item = String>) -> Option<String> {
    let args: Vec<String> = args.collect();
    args.windows(2)
        .find(|pair| pair[0] == "--chromium-id")
        .map(|pair| pair[1].clone())
}

fn install(chromium_id: Option<String>) -> io::Result<()> {
    let host = sibling_host_exe()?;
    if !host.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing {} — build sweeploom-companion-host next to this binary",
                host.display()
            ),
        ));
    }
    let app_data = UserLocations::current().app_data;
    let id = resolve_chromium_id(chromium_id.as_deref(), &app_data)?;
    let written = install_native_host(&app_data, &host, id.as_deref())?;
    println!("firefox host {}", written.firefox_manifest.display());
    match &written.chromium_manifest {
        Some(path) => println!("chromium/edge host {}", path.display()),
        None => println!(
            "Edge/Chrome still need an extension id: SweepLoom → Browser → Tabs, or --chromium-id"
        ),
    }
    println!("host binary {}", written.host_exe.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reads_chromium_id_flag() {
        let id = parse_chromium_id(
            ["--chromium-id", "abcdefghijklmnopabcdefghijklmnop"]
                .into_iter()
                .map(str::to_owned),
        );
        assert_eq!(id.as_deref(), Some("abcdefghijklmnopabcdefghijklmnop"));
    }

    #[test]
    fn parse_without_flag_is_none() {
        let id = parse_chromium_id(std::iter::empty());
        assert_eq!(id, None);
    }
}
