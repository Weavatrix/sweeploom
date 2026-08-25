//! Session termination. Force-kill is never the default.

use sweeploom_core::ProcessKey;
use sweeploom_platform::{ProcessControlBackend, Result};

/// Ask every member of a logical session to stop, children first.
///
/// `keys` should already be the session members. PID reuse is rejected by
/// the backend via [`ProcessKey`].
pub fn stop_session_gracefully(
    keys: &[ProcessKey],
    control: &impl ProcessControlBackend,
) -> Result<()> {
    let mut last_error = None;
    for key in keys.iter().rev() {
        if let Err(error) = control.request_graceful_stop(*key) {
            last_error = Some(error);
        }
    }
    match last_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
