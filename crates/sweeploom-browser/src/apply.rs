//! Queued companion actions. The GUI writes; the host sends them on the next tabs ping.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::action::TabAction;
use crate::message::TabCommand;

/// Pending apply file for one browser connection.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyQueue {
    /// Target browser instance.
    #[serde(default)]
    pub instance_id: String,
    /// Target connection epoch.
    #[serde(default)]
    pub epoch: u64,
    /// Queued actions.
    #[serde(default)]
    pub actions: Vec<TabCommand>,
}

/// Path of the pending apply file.
#[must_use]
pub fn apply_path(app_data: &Path) -> PathBuf {
    app_data.join("companion-apply.json")
}

/// Queue actions for the next native-messaging tabs reply. Appends; never Close.
pub fn save_apply(app_data: &Path, actions: Vec<TabCommand>) -> io::Result<()> {
    let incoming: Vec<TabCommand> = actions
        .into_iter()
        .filter(|item| allowed(&item.action))
        .collect();
    if incoming.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(app_data)?;
    let mut queue = load_queue(app_data).unwrap_or_default();
    let instance = incoming
        .first()
        .map(|item| item.instance_id.clone())
        .unwrap_or_default();
    let epoch = incoming.first().map(|item| item.epoch).unwrap_or(0);
    if queue.instance_id != instance || queue.epoch != epoch {
        queue = ApplyQueue {
            instance_id: instance,
            epoch,
            actions: Vec::new(),
        };
    }
    queue.actions.extend(incoming);
    write_queue(app_data, &queue)
}

/// Take queued actions for `instance`/`epoch`. Wrong instance sees nothing.
pub fn take_apply_for(
    app_data: &Path,
    instance_id: &str,
    epoch: u64,
) -> io::Result<Vec<TabCommand>> {
    let path = apply_path(app_data);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let queue = load_queue(app_data)?;
    if !queue.instance_id.is_empty()
        && (!instance_id.is_empty() && queue.instance_id != instance_id
            || epoch != 0 && queue.epoch != 0 && queue.epoch != epoch)
    {
        return Ok(Vec::new());
    }
    let _ = fs::remove_file(&path);
    let now = unix_ms();
    Ok(queue
        .actions
        .into_iter()
        .filter(|item| allowed(&item.action))
        .filter(|item| item.expires_unix_ms == 0 || item.expires_unix_ms > now)
        .collect())
}

/// Take queued actions, removing the file. Missing file is empty.
pub fn take_apply(app_data: &Path) -> io::Result<Vec<TabCommand>> {
    take_apply_for(app_data, "", 0)
}

fn allowed(action: &TabAction) -> bool {
    matches!(
        action,
        TabAction::Discard | TabAction::BookmarkAndClose | TabAction::Focus
    )
}

fn load_queue(app_data: &Path) -> io::Result<ApplyQueue> {
    let path = apply_path(app_data);
    if !path.is_file() {
        return Ok(ApplyQueue::default());
    }
    let bytes = fs::read(path)?;
    if let Ok(queue) = serde_json::from_slice::<ApplyQueue>(&bytes) {
        return Ok(queue);
    }
    let actions: Vec<TabCommand> = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(ApplyQueue {
        instance_id: String::new(),
        epoch: 0,
        actions,
    })
}

fn write_queue(app_data: &Path, queue: &ApplyQueue) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(queue)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(apply_path(app_data), bytes)
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| u64::try_from(item.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_is_never_queued() {
        let dir = std::env::temp_dir().join(format!("sweeploom-apply-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        save_apply(
            &dir,
            vec![
                TabCommand {
                    tab_id: 1,
                    action: TabAction::Close,
                    instance_id: "a".into(),
                    epoch: 1,
                    action_id: "1".into(),
                    expires_unix_ms: 0,
                    expected_url: String::new(),
                },
                TabCommand {
                    tab_id: 2,
                    action: TabAction::Discard,
                    instance_id: "a".into(),
                    epoch: 1,
                    action_id: "2".into(),
                    expires_unix_ms: 0,
                    expected_url: String::new(),
                },
            ],
        )
        .unwrap();
        let got = take_apply_for(&dir, "a", 1).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].tab_id, 2);
        assert!(!apply_path(&dir).is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn other_instance_does_not_take_queue() {
        let dir = std::env::temp_dir().join(format!("sweeploom-apply-ns-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        save_apply(
            &dir,
            vec![TabCommand {
                tab_id: 4,
                action: TabAction::Discard,
                instance_id: "chrome".into(),
                epoch: 9,
                action_id: "x".into(),
                expires_unix_ms: 0,
                expected_url: String::new(),
            }],
        )
        .unwrap();
        assert!(take_apply_for(&dir, "edge", 9).unwrap().is_empty());
        let got = take_apply_for(&dir, "chrome", 9).unwrap();
        assert_eq!(got.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn second_save_appends() {
        let dir = std::env::temp_dir().join(format!("sweeploom-apply-ap-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let cmd = |id, tab| TabCommand {
            tab_id: tab,
            action: TabAction::Discard,
            instance_id: "a".into(),
            epoch: 1,
            action_id: id,
            expires_unix_ms: 0,
            expected_url: String::new(),
        };
        save_apply(&dir, vec![cmd("1".into(), 1)]).unwrap();
        save_apply(&dir, vec![cmd("2".into(), 2)]).unwrap();
        let got = take_apply_for(&dir, "a", 1).unwrap();
        assert_eq!(got.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }
}
