use super::hash;
use super::verdict::{AppEvidence, GitState, Project, Verdict, date, device, parse_date, project};

const DAY: i64 = 86_400;

#[test]
fn dates_round_trip_through_civil_calendar() {
    assert_eq!(parse_date("2026-10-10T19:20:54Z"), Some(1_791_660_054));
    assert_eq!(parse_date("2000-02-29T00:00:00Z"), Some(951_782_400));
    assert_eq!(parse_date("garbage"), None);
    assert_eq!(date(951_782_400), "2000-02-29");
    assert_eq!(date(-1), "1969-12-31");
    assert_eq!(date(1_791_660_054), "2026-10-10");
}

#[test]
fn project_verdicts_follow_git_activity() {
    let now = 1_800_000_000;
    let git = |last: Option<i64>, dirty| {
        Project::Git(GitState {
            branch: Some("main".into()),
            last_commit: last,
            dirty,
        })
    };
    let (verdict, text) = project(&Project::Missing, now);
    assert_eq!(
        (verdict, text.as_str()),
        (Verdict::Suggested, "Project deleted — safe to remove")
    );
    let (verdict, text) = project(&git(Some(now - 7200), Some(0)), now);
    assert_eq!(verdict, Verdict::Keep);
    assert_eq!(text, "Active project (commit 2h ago on main) — keep");
    let (verdict, text) = project(&git(Some(now - 400 * DAY), Some(3)), now);
    assert_eq!(verdict, Verdict::Keep);
    assert!(text.starts_with("Active project (3 uncommitted changes on main)"));
    let (verdict, text) = project(&git(Some(now - 120 * DAY), Some(0)), now);
    assert_eq!(verdict, Verdict::Suggested);
    assert_eq!(
        text,
        format!(
            "Project untouched for 120 days (last commit {})",
            date(now - 120 * DAY)
        )
    );
    assert_eq!(
        project(&git(Some(now - 30 * DAY), Some(0)), now).0,
        Verdict::Review
    );
    assert_eq!(project(&git(None, None), now).0, Verdict::Review);
    let plain = Project::Plain {
        modified: Some(now - DAY),
    };
    assert_eq!(project(&plain, now).0, Verdict::Keep);
    assert_eq!(Verdict::Suggested.status("x"), "Suggested · x");
    assert!(Verdict::Keep < Verdict::Review && Verdict::Review < Verdict::Suggested);
}

#[test]
fn device_verdicts_protect_running_and_active_project_simulators() {
    let now = 1_800_000_000;
    let app = |verdict: Option<Verdict>| AppEvidence {
        bundle: "com.example.app".into(),
        project: verdict.map(|verdict| ("Example".into(), verdict, "detail".into())),
    };
    let old = Some(now - 214 * DAY);
    assert_eq!(device(true, None, old, &[], now).0, Verdict::Keep);
    let (verdict, text) = device(false, Some("iOS 17.0 not installed"), old, &[], now);
    assert_eq!(verdict, Verdict::Suggested);
    assert_eq!(text, "Unavailable runtime (iOS 17.0 not installed)");
    let recent = Some(now - 3 * DAY);
    let (verdict, text) = device(false, None, recent, &[app(Some(Verdict::Keep))], now);
    assert_eq!(
        (verdict, text.as_str()),
        (Verdict::Keep, "Has Example: detail")
    );
    let (verdict, text) = device(false, None, old, &[app(Some(Verdict::Keep))], now);
    assert_eq!(verdict, Verdict::Review);
    assert_eq!(
        text,
        "Example is active, but this simulator was last booted 214 days ago"
    );
    let (verdict, text) = device(false, None, old, &[app(None)], now);
    assert_eq!(
        (verdict, text.as_str()),
        (Verdict::Suggested, "Simulator last booted 214 days ago")
    );
    assert_eq!(
        device(false, None, recent, &[app(Some(Verdict::Suggested))], now).0,
        Verdict::Suggested
    );
    assert_eq!(
        device(false, None, recent, &[app(None)], now).0,
        Verdict::Review
    );
    assert_eq!(device(false, None, recent, &[], now).0, Verdict::Review);
    assert_eq!(device(false, None, None, &[], now).0, Verdict::Suggested);
}

#[test]
fn derived_data_suffix_matches_xcode_naming() {
    let hex = |bytes: [u8; 16]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    assert_eq!(hex(hash::md5(b"")), "d41d8cd98f00b204e9800998ecf8427e");
    assert_eq!(hex(hash::md5(b"abc")), "900150983cd24fb0d6963f7d28e17f72");
    assert_eq!(hex(hash::md5(&[b'x'; 100])).len(), 32);
    assert_eq!(
        hash::derived_suffix("/Users/me/dev/kablay-il/KablayIL.xcodeproj").len(),
        28
    );
    assert_eq!(
        hash::derived_suffix(
            "/Users/serhiirihgt/.codex/worktrees/academy-single-reader/crabrix-poc/Crabrix.xcodeproj"
        ),
        "glbrszzudognzhadgcnycyecwxiq"
    );
}
