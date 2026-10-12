import Link from "next/link";
import changelog from "@/content/changelog.json";
import { PageHeader } from "@/components/Section";
import { formatDate } from "@/lib/blog";
import { pageMeta } from "@/lib/meta";
import { SITE, docsUrl } from "@/lib/site";

export const metadata = pageMeta({
  title: "Changelog",
  description: "Every change to SweepLoom, from the first workspace commit to the 0.1.0 release, generated from the repository's Git history.",
  path: "/changelog/",
});

type Commit = { hash: string; date: string; subject: string };

const RELEASE_HASH = "236a93d"; // "Ship SweepLoom CLI, MCP, GUI, and verified benchmarks"

const UNRELEASED: { title: string; body: React.ReactNode }[] = [
  {
    title: "Desktop app redesigned around nine tabs",
    body: (
      <>
        A design system (spacing scale, cards, segmented controls, status pills, stacked bars and sparklines) in light and
        dark. Generated project output moved under Cleanup and saved disk scans under History, so the sidebar is Overview,
        Sessions, History, Cleanup, Projects, Explorer, Browser, AI and Settings. Deep links and the tray follow.{" "}
        <Link href="/app/" className="link">
          Tour the app
        </Link>
        .
      </>
    ),
  },
  { title: "Per-core CPU", body: "Overview and Sessions chart every core, with the performance / efficiency split on Apple Silicon." },
  {
    title: "iOS & Xcode discovery from disk",
    body: "Simulators are read from their device records first (simctl could take 40–80 s and time out), then confirmed. Orphan device folders, dyld caches, unused runtimes and DerivedData matched to projects, with Git-aware Keep / Review / Suggested verdicts. Removing unavailable devices deletes exact UDIDs only.",
  },
  {
    title: "Projects across all developer folders",
    body: "Discovery covers every developer folder instead of the launch directory, with fair per-folder budgets and Xcode, Swift, JVM and .NET markers.",
  },
  {
    title: "Parallel, streaming scans",
    body: "Discovery, Project output, project sizes and Explorer run on bounded worker pools and stream rows. Results survive tab switches and refreshes until replaced.",
  },
  {
    title: "Crash resilience",
    body: "A release abort in redaction of non-ASCII process arguments is fixed. Panics are logged (on macOS to ~/Library/Logs/SweepLoom/panic.log) and release builds unwind, so a failed background scan no longer closes the app. Session detail no longer runs git on the UI thread.",
  },
  {
    title: "Windows paths",
    body: "Canonical paths drop the \\\\?\\ verbatim prefix everywhere, so project lookups, Explorer and discovery agree and one project no longer appears under two paths.",
  },
  {
    title: "Docs, terms and this website",
    body: (
      <>
        The README now points to focused documents in docs/, TERMS.md was added, and sweeploom.com is built from the
        repository&apos;s site/ folder and served by nginx at Hetzner. Desktop Cleanup sources are in{" "}
        <a href={docsUrl("CLEANUP.md")} className="link">
          docs/CLEANUP.md
        </a>
        ; the signed macOS bundle is in{" "}
        <a href={docsUrl("MACOS.md")} className="link">
          docs/MACOS.md
        </a>
        .
      </>
    ),
  },
];

function groupByDate(commits: Commit[]) {
  const groups = new Map<string, Commit[]>();
  for (const commit of commits) {
    const list = groups.get(commit.date) ?? [];
    list.push(commit);
    groups.set(commit.date, list);
  }
  return [...groups.entries()];
}

function CommitList({ commits }: { commits: Commit[] }) {
  return (
    <ul className="grid gap-2">
      {commits.map((commit) => (
        <li key={commit.hash} className="flex gap-3 text-[0.95rem] leading-relaxed">
          <a
            href={`${SITE.repo}/commit/${commit.hash}`}
            className="shrink-0 pt-0.5 font-mono text-xs text-accent hover:underline"
            aria-label={`Commit ${commit.hash}`}
          >
            {commit.hash}
          </a>
          <span className="text-muted">{commit.subject}</span>
        </li>
      ))}
    </ul>
  );
}

export default function ChangelogPage() {
  const commits = changelog.commits as Commit[];
  const releaseIndex = commits.findIndex((c) => c.hash === RELEASE_HASH);
  const unreleased = releaseIndex >= 0 ? commits.slice(0, releaseIndex) : [];
  const release = releaseIndex >= 0 ? [commits[releaseIndex]] : [];
  const before = releaseIndex >= 0 ? commits.slice(releaseIndex + 1) : commits;
  const releaseDate = release[0]?.date ?? "2026-09-15";

  return (
    <>
      <PageHeader
        eyebrow="Changelog"
        title="What changed, commit by commit."
        lead={
          <>
            Generated from the repository&apos;s Git history ({commits.length} commits as of{" "}
            {formatDate(changelog.generated)}). The full log is on{" "}
            <a href={`${SITE.repo}/commits/main`} className="link">
              GitHub
            </a>
            .
          </>
        }
      />
      <div className="wrap py-14">
        <ol className="relative grid gap-12 border-l border-line pl-6 sm:pl-10">
          <li className="relative">
            <span className="absolute top-1.5 -left-[31px] h-3 w-3 rounded-full border-2 border-gold bg-bg sm:-left-[47px]" aria-hidden="true" />
            <div className="flex flex-wrap items-baseline gap-3">
              <h2 className="font-display text-2xl font-semibold">Unreleased</h2>
              <span className="chip chip-gold">in development</span>
            </div>
            <p className="mt-2 text-sm text-muted">Work on main since 0.1.0, not yet tagged as a release. The workspace version is still {SITE.version}.</p>
            <ul className="mt-5 grid gap-3">
              {UNRELEASED.map((item) => (
                <li key={item.title} className="card p-4">
                  <p className="font-medium">{item.title}</p>
                  <p className="mt-1 text-sm leading-relaxed text-muted">{item.body}</p>
                </li>
              ))}
            </ul>
            {unreleased.length ? (
              <div className="mt-6">
                <h3 className="font-mono text-xs tracking-widest text-faint uppercase">Commits since 0.1.0</h3>
                <div className="mt-3">
                  <CommitList commits={unreleased} />
                </div>
              </div>
            ) : null}
          </li>

          {release.length ? (
            <li className="relative">
              <span className="absolute top-1.5 -left-[31px] h-3 w-3 rounded-full bg-gold sm:-left-[47px]" aria-hidden="true" />
              <div className="flex flex-wrap items-baseline gap-3">
                <h2 className="font-display text-2xl font-semibold">0.1.0</h2>
                <time dateTime={releaseDate} className="font-mono text-sm text-faint">
                  {formatDate(releaseDate)}
                </time>
              </div>
              <p className="mt-2 max-w-2xl text-sm leading-relaxed text-muted">
                First release: the <code className="inline-code">sweeploom</code> CLI, the MCP server with nine tools, the Codex
                plugin, the Rust library, the egui desktop app and the browser companion, with benchmarks that run as tests.
              </p>
              <div className="mt-5">
                <CommitList commits={release} />
              </div>
            </li>
          ) : null}

          {groupByDate(before).map(([date, list]) => (
            <li key={date} className="relative">
              <span className="absolute top-1.5 -left-[29px] h-2 w-2 rounded-full bg-line-strong sm:-left-[45px]" aria-hidden="true" />
              <h2 className="font-mono text-sm text-faint">
                <time dateTime={date}>{formatDate(date)}</time>
                <span className="ml-2">· development</span>
              </h2>
              <div className="mt-4">
                <CommitList commits={list} />
              </div>
            </li>
          ))}
        </ol>
        <p className="mt-12 text-sm text-muted">
          Want to follow along? The <Link href="/blog/" className="link">blog</Link> covers the reasoning behind the larger
          changes.
        </p>
      </div>
    </>
  );
}
