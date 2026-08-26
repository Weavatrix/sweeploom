//! Queued companion actions. The GUI writes; the host sends them on the next tabs ping.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::action::TabAction;
use crate::message::TabCommand;

/// Path of the pending apply file.
#[must_use]
pub fn apply_path(app_data: &Path) -> PathBuf {
    app_data.join("companion-apply.json")
}

/// Queue actions for the next native-messaging tabs reply. Drops `Close`.
pub fn save_apply(app_data: &Path, actions: Vec<TabCommand>) -> io::Result<()> {
    let actions: Vec<TabCommand> = actions
        .into_iter()
        .filter(|item| {
            matches!(
                item.action,
                TabAction::Discard | TabAction::BookmarkAndClose | TabAction::Focus
            )
        })
        .collect();
    fs::create_dir_all(app_data)?;
    let bytes = serde_json::to_vec_pretty(&actions)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(apply_path(app_data), bytes)
}

/// Take queued actions, removing the file. Missing file is empty.
pub fn take_apply(app_data: &Path) -> io::Result<Vec<TabCommand>> {
    let path = apply_path(app_data);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(&path)?;
    let _ = fs::remove_file(&path);
    let actions: Vec<TabCommand> = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(actions
        .into_iter()
        .filter(|item| {
            matches!(
                item.action,
                TabAction::Discard | TabAction::BookmarkAndClose | TabAction::Focus
            )
        })
        .collect())
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
                },
                TabCommand {
                    tab_id: 2,
                    action: TabAction::Discard,
                },
            ],
        )
        .unwrap();
        let got = take_apply(&dir).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].tab_id, 2);
        assert!(!apply_path(&dir).is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn focus_is_queued() {
        let dir = std::env::temp_dir().join(format!("sweeploom-focus-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        save_apply(
            &dir,
            vec![TabCommand {
                tab_id: 4,
                action: TabAction::Focus,
            }],
        )
        .unwrap();
        let got = take_apply(&dir).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].action, TabAction::Focus);
        let _ = fs::remove_dir_all(&dir);
    }
}
