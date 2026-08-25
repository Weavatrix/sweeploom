//! `sweeploom companion-install` — write host manifests and register them.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sweeploom_browser::{
    HOST_NAME, chromium_host_json, firefox_host_json, is_chromium_extension_id,
};
use sweeploom_platform::UserLocations;

pub fn run(args: impl Iterator<Item = String>) {
    let chromium_id = parse_chromium_id(args);
    if let Err(error) = install(chromium_id.as_deref()) {
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

fn install(chromium_id: Option<&str>) -> io::Result<()> {
    if let Some(id) = chromium_id
        && !is_chromium_extension_id(id)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "chromium id must be 32 characters in a–p (chrome://extensions)",
        ));
    }
    let host = host_exe()?;
    if !host.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing {} — build sweeploom-companion-host next to this binary",
                host.display()
            ),
        ));
    }
    let dir = UserLocations::current().app_data.join("native-messaging");
    fs::create_dir_all(&dir)?;
    let firefox_path = dir.join(format!("{HOST_NAME}.firefox.json"));
    fs::write(&firefox_path, firefox_host_json(&host).map_err(json_err)?)?;
    register_firefox(&firefox_path)?;
    println!("firefox host {}", firefox_path.display());
    if let Some(id) = chromium_id {
        let chrome_path = dir.join(format!("{HOST_NAME}.chromium.json"));
        fs::write(
            &chrome_path,
            chromium_host_json(&host, id).map_err(json_err)?,
        )?;
        register_chromium(&chrome_path)?;
        println!("chromium host {}", chrome_path.display());
    } else {
        println!("pass --chromium-id from chrome://extensions to register Chrome/Edge");
    }
    println!("host binary {}", host.display());
    Ok(())
}

fn json_err(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

fn host_exe() -> io::Result<PathBuf> {
    let current = env::current_exe()?;
    let dir = current.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "current executable has no parent")
    })?;
    let name = if cfg!(windows) {
        "sweeploom-companion-host.exe"
    } else {
        "sweeploom-companion-host"
    };
    Ok(dir.join(name))
}

fn register_firefox(manifest: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        reg_set(
            &format!(r"HKCU\Software\Mozilla\NativeMessagingHosts\{HOST_NAME}"),
            manifest,
        )
    }
    #[cfg(not(windows))]
    {
        copy_user_host(
            &[
                PathBuf::from(".mozilla/native-messaging-hosts"),
                PathBuf::from("Library/Application Support/Mozilla/NativeMessagingHosts"),
            ],
            manifest,
        )
    }
}

fn register_chromium(manifest: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        for key in [
            r"HKCU\Software\Google\Chrome\NativeMessagingHosts",
            r"HKCU\Software\Microsoft\Edge\NativeMessagingHosts",
            r"HKCU\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts",
        ] {
            reg_set(&format!(r"{key}\{HOST_NAME}"), manifest)?;
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        copy_user_host(
            &[
                PathBuf::from(".config/google-chrome/NativeMessagingHosts"),
                PathBuf::from(".config/chromium/NativeMessagingHosts"),
                PathBuf::from(".config/microsoft-edge/NativeMessagingHosts"),
                PathBuf::from("Library/Application Support/Google/Chrome/NativeMessagingHosts"),
            ],
            manifest,
        )
    }
}

#[cfg(windows)]
fn reg_set(key: &str, manifest: &Path) -> io::Result<()> {
    let status = std::process::Command::new("reg")
        .args([
            "add",
            key,
            "/ve",
            "/t",
            "REG_SZ",
            "/d",
            &manifest.to_string_lossy(),
            "/f",
        ])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("reg add failed for {key}")))
    }
}

#[cfg(not(windows))]
fn copy_user_host(rel_dirs: &[PathBuf], manifest: &Path) -> io::Result<()> {
    let home = UserLocations::current().home;
    let name = format!("{HOST_NAME}.json");
    let mut wrote = false;
    for rel in rel_dirs {
        let dir = home.join(rel);
        if rel.starts_with("Library") && !cfg!(target_os = "macos") {
            continue;
        }
        if rel.starts_with(".config") && cfg!(target_os = "macos") {
            continue;
        }
        fs::create_dir_all(&dir)?;
        fs::copy(manifest, dir.join(&name))?;
        wrote = true;
    }
    if wrote {
        Ok(())
    } else {
        Err(io::Error::other("no native-messaging directory written"))
    }
}
