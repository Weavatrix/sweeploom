use serde_json::Value;
use std::{
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub(super) fn command(program: &str, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args);
    execute(program, command)
}

pub(super) fn execute(program: &str, command: Command) -> Result<String, String> {
    execute_for(program, command, Duration::from_secs(20))
}

/// Run with a deadline; the child is killed when it does not finish in time.
pub(super) fn execute_for(
    program: &str,
    mut command: Command,
    timeout: Duration,
) -> Result<String, String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes);
        bytes
    });
    let err = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.read_to_end(&mut bytes);
        bytes
    });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        if started.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "{program} did not respond within {} seconds. Check that the tool is running, then Refresh listing.",
                timeout.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let text = String::from_utf8_lossy(&out.join().unwrap_or_default()).into_owned();
    let error = String::from_utf8_lossy(&err.join().unwrap_or_default()).into_owned();
    if status.success() {
        Ok(text)
    } else {
        Err(format!("{program}: {}", error.trim()))
    }
}

pub(super) fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}
pub(super) fn value_bytes(value: &Value, key: &str) -> Option<u64> {
    let value = value.get(key)?;
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(parse_size))
}
pub(super) fn parse_size(text: &str) -> Option<u64> {
    let number: String = text
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let n: f64 = number.parse().ok()?;
    let unit = text.trim()[number.len()..].trim().to_ascii_lowercase();
    let multiplier = match unit.as_str() {
        "b" | "" => 1.0,
        "kb" | "kib" => 1_000.0,
        "mb" | "mib" => 1_000_000.0,
        "gb" | "gib" => 1_000_000_000.0,
        "tb" | "tib" => 1_000_000_000_000.0,
        _ => return None,
    };
    Some((n * multiplier) as u64)
}
pub(super) fn value_count(value: &Value, key: &str) -> u64 {
    value
        .get(key)
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
        .unwrap_or(0)
}
pub(super) fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
}
#[cfg(unix)]
pub(super) fn allocated(meta: &std::fs::Metadata) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    Some(meta.blocks().saturating_mul(512))
}
#[cfg(not(unix))]
pub(super) fn allocated(_meta: &std::fs::Metadata) -> Option<u64> {
    None
}
