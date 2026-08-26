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
}
