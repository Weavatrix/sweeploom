//! Plain-language cleanup verdicts from project, Git and simulator evidence.
const DAY: i64 = 86_400;
const ACTIVE_DAYS: i64 = 14;
const STALE_DAYS: i64 = 90;
/// A simulator unused this long is worth a look even when its project is active.
const IDLE_DEVICE_DAYS: i64 = 30;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GitState {
    pub branch: Option<String>,
    /// Unix seconds of HEAD's committer date.
    pub last_commit: Option<i64>,
    /// Changed paths; `None` when status timed out or failed.
    pub dirty: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Project {
    Missing,
    Plain { modified: Option<i64> },
    Git(GitState),
}

/// Ordered so the status column sorts Keep < Review < Suggested.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Verdict {
    Keep,
    Review,
    Suggested,
}

impl Verdict {
    pub(crate) fn status(self, detail: &str) -> String {
        let label = match self {
            Self::Keep => "Keep",
            Self::Review => "Review",
            Self::Suggested => "Suggested",
        };
        format!("{label} · {detail}")
    }
}

pub(crate) fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |span| i64::try_from(span.as_secs()).unwrap_or(i64::MAX))
}

pub(crate) fn ago(seconds: i64) -> String {
    let seconds = seconds.max(0);
    if seconds < 3600 {
        format!("{}m ago", seconds / 60)
    } else if seconds < 2 * DAY {
        format!("{}h ago", seconds / 3600)
    } else {
        format!("{} days ago", seconds / DAY)
    }
}

pub(crate) fn project(project: &Project, now: i64) -> (Verdict, String) {
    match project {
        Project::Missing => (
            Verdict::Suggested,
            "Project deleted — safe to remove".into(),
        ),
        Project::Plain { modified } => match modified.map(|at| now - at) {
            Some(age) if age < ACTIVE_DAYS * DAY => (
                Verdict::Keep,
                format!("Project edited {} (not in Git) — keep", ago(age)),
            ),
            Some(age) if age >= STALE_DAYS * DAY => (
                Verdict::Suggested,
                format!("Project untouched for {} days (not in Git)", age / DAY),
            ),
            Some(age) => (
                Verdict::Review,
                format!("Project edited {} (not in Git)", ago(age)),
            ),
            None => (Verdict::Review, "Project present; not in Git".into()),
        },
        Project::Git(git) => git_verdict(git, now),
    }
}

fn git_verdict(git: &GitState, now: i64) -> (Verdict, String) {
    let branch = git
        .branch
        .as_deref()
        .map(|branch| format!(" on {branch}"))
        .unwrap_or_default();
    if let Some(dirty) = git.dirty.filter(|&dirty| dirty > 0) {
        return (
            Verdict::Keep,
            format!("Active project ({dirty} uncommitted changes{branch}) — keep"),
        );
    }
    let Some(at) = git.last_commit else {
        return (Verdict::Review, "Git state unavailable (timed out)".into());
    };
    let age = now - at;
    if age < ACTIVE_DAYS * DAY {
        (
            Verdict::Keep,
            format!("Active project (commit {}{branch}) — keep", ago(age)),
        )
    } else if age >= STALE_DAYS * DAY {
        (
            Verdict::Suggested,
            format!(
                "Project untouched for {} days (last commit {})",
                age / DAY,
                date(at)
            ),
        )
    } else {
        (
            Verdict::Review,
            format!("Last commit {} ({}{branch})", ago(age), date(at)),
        )
    }
}

/// One installed app and the verdict of the project that builds it, if known.
pub(crate) struct AppEvidence {
    pub bundle: String,
    pub project: Option<(String, Verdict, String)>,
}

pub(crate) fn device(
    booted: bool,
    unavailable: Option<&str>,
    last_used: Option<i64>,
    apps: &[AppEvidence],
    now: i64,
) -> (Verdict, String) {
    let idle = last_used.map(|at| now - at);
    let booted_text = idle.map_or_else(
        || "never booted".to_owned(),
        |age| format!("last booted {}", ago(age)),
    );
    if booted {
        return (
            Verdict::Keep,
            "Running — shut it down before removal".into(),
        );
    }
    if let Some(reason) = unavailable {
        return (
            Verdict::Suggested,
            format!("Unavailable runtime ({reason})"),
        );
    }
    if let Some((name, _, detail)) = apps
        .iter()
        .filter_map(|app| app.project.as_ref())
        .find(|(_, verdict, _)| *verdict == Verdict::Keep)
    {
        return match idle.filter(|&age| age >= IDLE_DEVICE_DAYS * DAY) {
            Some(age) => (
                Verdict::Review,
                format!(
                    "{name} is active, but this simulator was last booted {}",
                    ago(age)
                ),
            ),
            None => (Verdict::Keep, format!("Has {name}: {detail}")),
        };
    }
    if idle.is_some_and(|age| age >= STALE_DAYS * DAY) {
        return (Verdict::Suggested, format!("Simulator {booted_text}"));
    }
    let linked: Vec<_> = apps.iter().filter_map(|app| app.project.as_ref()).collect();
    if !linked.is_empty()
        && linked.len() == apps.len()
        && linked
            .iter()
            .all(|(_, verdict, _)| *verdict == Verdict::Suggested)
    {
        let names: Vec<_> = linked.iter().map(|(name, ..)| name.as_str()).collect();
        return (
            Verdict::Suggested,
            format!(
                "Apps only from deleted/stale projects ({}); {booted_text}",
                names.join(", ")
            ),
        );
    }
    if apps.is_empty() && idle.is_none_or(|age| age >= IDLE_DEVICE_DAYS * DAY) {
        return (
            Verdict::Suggested,
            format!("No apps installed; {booted_text}"),
        );
    }
    let names: Vec<_> = apps.iter().take(3).map(|app| app.bundle.as_str()).collect();
    let apps_text = if names.is_empty() {
        "no apps".to_owned()
    } else {
        format!("apps: {}", names.join(", "))
    };
    (
        Verdict::Review,
        format!("Simulator {booted_text}; {apps_text}"),
    )
}

/// `YYYY-MM-DD` for Unix seconds (UTC).
pub(crate) fn date(unix: i64) -> String {
    let days = unix.div_euclid(DAY);
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let doe = shifted - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Unix seconds from `YYYY-MM-DDTHH:MM:SSZ` (simctl and plist dates).
pub(crate) fn parse_date(text: &str) -> Option<i64> {
    let number = |range: std::ops::Range<usize>| text.get(range)?.parse::<i64>().ok();
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * DAY + hour * 3600 + minute * 60 + second)
}
