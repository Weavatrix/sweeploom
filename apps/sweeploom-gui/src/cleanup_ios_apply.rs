//! Simulator removal through simctl; leftover folders only after fresh re-verification.
use super::{Item, Path, PathBuf, disk, list, merge, parse_devices, runtimes, simctl, sims};
use std::{fs, time::Duration};
use sweeploom_core::ProcessSnapshot;

const ACTION_TIMEOUT: Duration = Duration::from_secs(120);

fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
        .ok_or_else(|| "Home folder unavailable".to_owned())
}

/// Only the known CoreSimulator device sets may be passed to `simctl --set`.
fn checked_set(scope: Option<&str>, home: &Path) -> Result<disk::Set, String> {
    disk::sets(home)
        .into_iter()
        .find(|set| set.scope.as_deref() == scope)
        .ok_or_else(|| "Unknown simulator device set".to_owned())
}

pub(in super::super) fn remove_device(item: &Item) -> Result<String, String> {
    let set = checked_set(item.scope.as_deref(), &home()?)?;
    let live = parse_devices(&list(set.scope.as_deref())?)
        .into_iter()
        .find(|device| device.id == item.id)
        .ok_or("Device is no longer present")?;
    if !live.enabled {
        return Err("Device is running; shut it down explicitly first.".into());
    }
    let verb = if item.status.starts_with("Erase data") {
        "erase"
    } else {
        "delete"
    };
    simctl(set.scope.as_deref(), &[verb, &item.id], ACTION_TIMEOUT)
}

/// Deletes exactly the stranded devices the user confirmed; any change since listing aborts.
pub(in super::super) fn remove_unavailable(item: &Item) -> Result<String, String> {
    let set = checked_set(item.scope.as_deref(), &home()?)?;
    let expected: usize = item
        .id
        .rsplit(':')
        .next()
        .and_then(|count| count.parse().ok())
        .ok_or("Malformed unavailable-device row")?;
    let runtimes = simctl(None, &["runtime", "list", "--json"], ACTION_TIMEOUT)?;
    let installed = runtimes::identifiers(&runtimes).ok_or("Runtime inventory unreadable")?;
    let mut listed = sims(&list(set.scope.as_deref())?);
    merge::settle(&mut listed, Some(&installed));
    let doomed = merge::stranded(&listed);
    if doomed.len() != expected {
        return Err(
            "Unavailable simulators changed since listing; refresh and review again.".into(),
        );
    }
    let mut args = vec!["delete"];
    args.extend(doomed.iter().map(|sim| sim.udid.as_str()));
    simctl(set.scope.as_deref(), &args, ACTION_TIMEOUT)
}

/// Leftover device folders and CoreSimulator caches; nothing outside those parents.
pub(in super::super) fn remove_files(
    item: &Item,
    processes: &[ProcessSnapshot],
) -> Result<String, String> {
    let home = home()?;
    let path = item.path.as_deref().ok_or("No path recorded")?;
    let meta = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("Symlinks are never removed".into());
    }
    let parent = path.parent().ok_or("No parent folder")?;
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if let Some(set) = disk::sets(&home).into_iter().find(|set| set.path == parent) {
        let (registered, _) = disk::devices(&set);
        let listed = parse_devices(&list(set.scope.as_deref())?);
        if !disk::is_udid(&name)
            || registered.iter().any(|sim| sim.udid == name)
            || listed.iter().any(|device| device.id == name)
        {
            return Err("Folder belongs to a registered simulator; refresh the listing.".into());
        }
    } else if leftover_roots(&home).iter().any(|root| root == parent) {
        if parse_devices(&list(None)?)
            .iter()
            .any(|device| !device.enabled)
        {
            return Err("A simulator is running; stop it before removing caches.".into());
        }
    } else {
        return Err("Not a simulator leftover folder".into());
    }
    if processes.iter().any(|process| process.uses_path(path)) {
        return Err("A running process is using this folder.".into());
    }
    if meta.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map(|()| String::new())
    .map_err(|error| error.to_string())
}

pub(super) fn leftover_roots(home: &Path) -> [PathBuf; 2] {
    [
        home.join("Library/Developer/CoreSimulator/Caches/dyld"),
        home.join("Library/Developer/CoreSimulator/Temp"),
    ]
}
