//! Write native-messaging manifests and register them for Firefox / Chromium.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::install::{HOST_NAME, chromium_host_json, firefox_host_json, is_chromium_extension_id};

/// Paths written by [`install_native_host`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeHostInstall {
    /// `sweeploom-companion-host` binary used in the manifests.
    pub host_exe: PathBuf,
    /// Firefox host manifest.
    pub firefox_manifest: PathBuf,
    /// Chromium/Edge host manifest, when an extension id was supplied.
    pub chromium_manifest: Option<PathBuf>,
}

/// File that remembers the unpacked Chromium/Edge extension id.
#[must_use]
pub fn chromium_id_path(app_data: &Path) -> PathBuf {
    app_data.join("chromium-extension-id.txt")
}

/// Load a previously saved Chromium extension id, if any.
pub fn load_chromium_id(app_data: &Path) -> io::Result<Option<String>> {
    let path = chromium_id_path(app_data);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let id = raw.trim();
    if id.is_empty() {
        return Ok(None);
    }
    if !is_chromium_extension_id(id) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "saved chromium id must be 32 characters in a–p",
        ));
    }
    Ok(Some(id.to_owned()))
}

/// Persist the unpacked Chromium/Edge extension id.
pub fn save_chromium_id(app_data: &Path, id: &str) -> io::Result<()> {
    if !is_chromium_extension_id(id) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "chromium id must be 32 characters in a–p (edge://extensions)",
        ));
    }
    fs::create_dir_all(app_data)?;
    fs::write(chromium_id_path(app_data), format!("{id}\n"))
}

/// `--chromium-id` wins; otherwise use the saved id.
pub fn resolve_chromium_id(flag: Option<&str>, app_data: &Path) -> io::Result<Option<String>> {
    if let Some(id) = flag {
        if !is_chromium_extension_id(id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "chromium id must be 32 characters in a–p (edge://extensions)",
            ));
        }
        return Ok(Some(id.to_owned()));
    }
    load_chromium_id(app_data)
}

/// `sweeploom-companion-host` next to the current executable.
pub fn sibling_host_exe() -> io::Result<PathBuf> {
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

/// Write manifests under `app_data/native-messaging` and register them.
pub fn install_native_host(
    app_data: &Path,
    host_exe: &Path,
    chromium_id: Option<&str>,
) -> io::Result<NativeHostInstall> {
    let written = write_native_hosts(app_data, host_exe, chromium_id)?;
    if let Some(id) = chromium_id {
        save_chromium_id(app_data, id)?;
    }
    register_firefox(&written.firefox_manifest)?;
    if let Some(path) = &written.chromium_manifest {
        register_chromium(path)?;
    }
    Ok(written)
}

/// Write host JSON without touching the OS registry.
pub fn write_native_hosts(
    app_data: &Path,
    host_exe: &Path,
    chromium_id: Option<&str>,
) -> io::Result<NativeHostInstall> {
    if let Some(id) = chromium_id
        && !is_chromium_extension_id(id)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "chromium id must be 32 characters in a–p (edge://extensions)",
        ));
    }
    let dir = app_data.join("native-messaging");
    fs::create_dir_all(&dir)?;
    let firefox_manifest = dir.join(format!("{HOST_NAME}.firefox.json"));
    fs::write(
        &firefox_manifest,
        firefox_host_json(host_exe).map_err(json_err)?,
    )?;
    let chromium_manifest = match chromium_id {
        Some(id) => {
            let path = dir.join(format!("{HOST_NAME}.chromium.json"));
            fs::write(&path, chromium_host_json(host_exe, id).map_err(json_err)?)?;
            Some(path)
        }
        None => None,
    };
    Ok(NativeHostInstall {
        host_exe: host_exe.to_path_buf(),
        firefox_manifest,
        chromium_manifest,
    })
}

fn json_err(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
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
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let name = format!("{HOST_NAME}.json");
    let mut wrote = false;
    for rel in rel_dirs {
        if rel.starts_with("Library") && !cfg!(target_os = "macos") {
            continue;
        }
        if rel.starts_with(".config") && cfg!(target_os = "macos") {
            continue;
        }
        let dir = home.join(rel);
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

#[cfg(test)]
#[path = "host_setup_tests.rs"]
mod tests;
