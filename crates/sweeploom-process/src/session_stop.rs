//! Session termination. Force-kill is never the default.

use sweeploom_core::{ProcessKey, ProcessSnapshot};
use sweeploom_platform::{ProcessControlBackend, Result};

use crate::find_process;

/// Ask every member of a logical session to stop, children first.
///
/// `keys` should already be the session members. PID reuse is rejected by
/// the backend via [`ProcessKey`].
pub fn stop_session_gracefully(
    keys: &[ProcessKey],
    control: &impl ProcessControlBackend,
) -> Result<()> {
    signal_all(keys, |key| control.request_graceful_stop(key))
}

/// Members that still match a live snapshot. Empty means the session is gone.
#[must_use]
pub fn still_running(keys: &[ProcessKey], processes: &[ProcessSnapshot]) -> Vec<ProcessKey> {
    keys.iter()
        .copied()
        .filter(|key| find_process(processes, *key).is_some())
        .collect()
}

/// Last-resort kill. Call only after an explicit user confirmation.
pub fn force_stop_session(keys: &[ProcessKey], control: &impl ProcessControlBackend) -> Result<()> {
    signal_all(keys, |key| control.force_kill(key))
}

fn signal_all(keys: &[ProcessKey], mut signal: impl FnMut(ProcessKey) -> Result<()>) -> Result<()> {
    let mut last_error = None;
    for key in keys.iter().rev() {
        if let Err(error) = signal(*key) {
            last_error = Some(error);
        }
    }
    match last_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sweeploom_core::ProcessKey;

    #[test]
    fn missing_keys_are_not_still_running() {
        let key = ProcessKey::new(9_999_999, None);
        assert!(still_running(&[key], &[]).is_empty());
    }
}
