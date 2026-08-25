//! Command-line redaction. Tokens, passwords, and URI credentials must never
//! reach the UI, logs, or receipts.

const SECRET_FLAGS: &[&str] = &[
    "--token",
    "--password",
    "--passwd",
    "--secret",
    "--api-key",
    "--apikey",
    "--access-token",
    "--auth",
    "--authorization",
    "--connection-string",
    "--conn",
    "-p",
    "-P",
];

const SECRET_ENV_PREFIXES: &[&str] = &[
    "TOKEN=",
    "PASSWORD=",
    "SECRET=",
    "API_KEY=",
    "ACCESS_TOKEN=",
    "AUTHORIZATION=",
    "AUTH=",
];

/// Redact a command-line token list.
#[must_use]
pub fn redact_command(args: &[impl AsRef<str>]) -> Vec<String> {
    let mut redacted = Vec::with_capacity(args.len());
    let mut hide_next = false;
    for arg in args {
        let arg = arg.as_ref();
        if hide_next {
            redacted.push("***".to_owned());
            hide_next = false;
            continue;
        }
        if let Some(stripped) = secret_flag_value(arg) {
            redacted.push(stripped);
            continue;
        }
        if is_secret_flag(arg) {
            redacted.push(arg.to_owned());
            hide_next = true;
            continue;
        }
        if looks_like_secret_assignment(arg) {
            redacted.push(redact_assignment(arg));
            continue;
        }
        redacted.push(redact_uri_credentials(arg));
    }
    redacted
}

fn is_secret_flag(arg: &str) -> bool {
    SECRET_FLAGS
        .iter()
        .any(|flag| arg.eq_ignore_ascii_case(flag))
}

fn secret_flag_value(arg: &str) -> Option<String> {
    for flag in SECRET_FLAGS {
        let prefix = [*flag, "="].concat();
        if let Some(rest) = arg
            .get(..prefix.len())
            .filter(|head| head.eq_ignore_ascii_case(&prefix))
            .and_then(|_| arg.get(prefix.len()..))
        {
            if rest.is_empty() {
                return Some(prefix + "***");
            }
            return Some(prefix + "***");
        }
    }
    None
}

fn looks_like_secret_assignment(arg: &str) -> bool {
    SECRET_ENV_PREFIXES
        .iter()
        .any(|prefix| arg.len() >= prefix.len() && arg[..prefix.len()].eq_ignore_ascii_case(prefix))
}

fn redact_assignment(arg: &str) -> String {
    match arg.split_once('=') {
        Some((key, _)) => format!("{key}=***"),
        None => "***".to_owned(),
    }
}

fn redact_uri_credentials(arg: &str) -> String {
    let Some(scheme_end) = arg.find("://") else {
        return arg.to_owned();
    };
    let rest = &arg[scheme_end + 3..];
    let Some(at) = rest.find('@') else {
        return arg.to_owned();
    };
    let userinfo = &rest[..at];
    if !userinfo.contains(':') && !userinfo.contains('%') {
        return arg.to_owned();
    }
    format!("{}://***@{}", &arg[..scheme_end], &rest[at + 1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_flag_value_and_next_token() {
        let cmd = redact_command(&["claude", "--token", "sk-live-secret", "--api-key=abcd"]);
        assert_eq!(cmd, ["claude", "--token", "***", "--api-key=***"]);
    }

    #[test]
    fn redacts_uri_userinfo() {
        let cmd = redact_command(&["curl", "https://user:hunter2@example.com/x"]);
        assert_eq!(cmd, ["curl", "https://***@example.com/x"]);
    }

    #[test]
    fn leaves_ordinary_args_alone() {
        let cmd = redact_command(&["cargo", "build", "--release"]);
        assert_eq!(cmd, ["cargo", "build", "--release"]);
    }
}
