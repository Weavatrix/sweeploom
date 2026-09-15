//! Chromium/Firefox process roles from the command line. Not tab attribution.

/// Role of one browser OS process.
#[must_use]
pub fn process_role(command: &[String]) -> &'static str {
    for part in command {
        let lower = part.to_ascii_lowercase();
        if let Some(kind) = lower.strip_prefix("--type=") {
            return match kind {
                "renderer" => "Renderer",
                "gpu-process" | "gpu" => "GPU",
                "utility" => "Utility",
                "crashpad-handler" | "crashpad" => "Crashpad",
                "extension" => "Extension",
                "plugin" | "ppapi" => "Plugin",
                "broker" => "Broker",
                "watcher" => "Watcher",
                _ => "Helper",
            };
        }
        if lower == "-contentproc" {
            return "Content";
        }
    }
    "Browser"
}

/// Role plus utility subtype / extension so Task Manager-style labels work.
#[must_use]
pub fn process_caption(command: &[String]) -> String {
    if has_flag(command, "--extension-process") {
        return "Extension".to_owned();
    }
    let role = process_role(command);
    match utility_subtype(command) {
        Some(sub) => format!("{role} · {sub}"),
        None => role.to_owned(),
    }
}

fn has_flag(command: &[String], flag: &str) -> bool {
    command.iter().any(|part| part.eq_ignore_ascii_case(flag))
}

fn utility_subtype(command: &[String]) -> Option<String> {
    let raw = command.iter().find_map(|part| {
        part.to_ascii_lowercase()
            .strip_prefix("--utility-sub-type=")
            .map(str::to_owned)
    })?;
    Some(friendly_utility(&raw))
}

fn friendly_utility(raw: &str) -> String {
    let last = raw.rsplit(['.', '/']).next().unwrap_or(raw);
    let trimmed = last
        .strip_suffix("service")
        .unwrap_or(last)
        .trim_end_matches('_');
    let mut out = String::new();
    for (index, word) in trimmed
        .split('_')
        .filter(|item| !item.is_empty())
        .enumerate()
    {
        if index > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.extend(chars);
        }
    }
    if out.is_empty() { raw.to_owned() } else { out }
}

/// True when stopping this helper does not kill the browser process.
#[must_use]
pub fn can_stop_helper(role: &'static str) -> bool {
    matches!(role, "Renderer" | "Content" | "Utility" | "Extension")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_edge_is_browser() {
        assert_eq!(process_role(&["msedge.exe".into()]), "Browser");
        assert!(!can_stop_helper("Browser"));
    }

    #[test]
    fn renderer_is_stoppable() {
        assert_eq!(
            process_role(&["msedge.exe".into(), "--type=renderer".into()]),
            "Renderer"
        );
        assert!(can_stop_helper("Renderer"));
    }

    #[test]
    fn gpu_is_not_stoppable() {
        assert_eq!(
            process_role(&["msedge.exe".into(), "--type=gpu-process".into()]),
            "GPU"
        );
        assert!(!can_stop_helper("GPU"));
    }

    #[test]
    fn utility_caption_uses_subtype() {
        assert_eq!(
            process_caption(&[
                "msedge.exe".into(),
                "--type=utility".into(),
                "--utility-sub-type=network.mojom.NetworkService".into(),
            ]),
            "Utility · Network"
        );
    }

    #[test]
    fn extension_renderer_caption() {
        assert_eq!(
            process_caption(&[
                "msedge.exe".into(),
                "--type=renderer".into(),
                "--extension-process".into(),
            ]),
            "Extension"
        );
    }
}
