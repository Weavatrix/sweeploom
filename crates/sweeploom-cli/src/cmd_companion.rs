//! Native-messaging host. Stdout is frames only — never logs.

use std::io;

use sweeploom_browser::{HostMessage, handle_extension_json, read_frame, write_frame};
use sweeploom_platform::UserLocations;

pub fn run() {
    let app_data = UserLocations::current().app_data;
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    loop {
        match read_frame(&mut input) {
            Ok(None) => break,
            Ok(Some(bytes)) => {
                let reply = match handle_extension_json(&bytes, &app_data) {
                    Ok(reply) => reply,
                    Err(error) => {
                        eprintln!("companion-host: {error}");
                        ack_json(false, error)
                    }
                };
                if let Err(error) = write_frame(&mut output, &reply) {
                    eprintln!("companion-host write failed: {error}");
                    break;
                }
            }
            Err(error) => {
                eprintln!("companion-host read failed: {error}");
                break;
            }
        }
    }
}

fn ack_json(ok: bool, detail: String) -> Vec<u8> {
    serde_json::to_vec(&HostMessage::Ack { ok, detail })
        .unwrap_or_else(|_| br#"{"type":"ack","ok":false,"detail":"encode failed"}"#.to_vec())
}
