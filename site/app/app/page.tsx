import Link from "next/link";
import { CodeBlock } from "@/components/CodeBlock";
import { icons } from "@/components/Icons";
import { Screenshot } from "@/components/Screenshot";
import { ScrollSpy } from "@/components/ScrollSpy";
import { PageHeader, Section } from "@/components/Section";
import { GUI_SCREENS } from "@/lib/content";
import { pageMeta } from "@/lib/meta";
import { INSTALL, docsUrl } from "@/lib/site";

export const metadata = pageMeta({
  title: "Desktop app tour",
  description:
    "A tour of SweepLoom's desktop app: nine tabs from Overview to Settings, the seven Cleanup sources, per-core CPU with an Apple Silicon P/E split, iOS & Xcode verdicts, project discovery across your developer folders, and one safety model.",
  path: "/app/",
});

type Shot = { file: string; alt: string; caption: string; screen: string; width: number; height: number; frame?: boolean };

type Tab = {
  id: string;
  name: string;
  section: string;
  lead: string;
  sub?: { name: string; body: string }[];
  points: string[];
  shots?: Shot[];
};

const TABS: Tab[] = [
  {
    id: "overview",
    name: "Overview",
    section: "Live",
    lead: "Live pressure across memory, CPU and disk on one screen. Every tile and every session row opens the screen that explains it.",
    points: [
      "Four tiles: memory available, CPU load with the core count, idle sessions, and free disk with the share taken by project output.",
      "Per-core CPU with a short history per core. On Apple Silicon, performance cores come first, then efficiency cores, read from the system's own perflevel counters; elsewhere cores appear in OS order.",
      "Memory split by session and by workload (AI agents, MCP servers, browsers, apps). Session slices are scaled from resident memory, because shared pages make raw RSS add up past installed RAM.",
      "Disk: used, project output and free, plus the largest measured locations with their change since the last scan.",
    ],
    shots: [
      {
        file: "overview.png",
        screen: "Overview",
        alt: "SweepLoom Overview: memory, CPU and disk tiles, ten per-core CPU charts split into four performance and six efficiency cores, memory and disk breakdowns",
        caption: "A 10-core Apple Silicon Mac (4P + 6E) under full load, with simulators holding most of the memory. Process names and paths are blurred.",
        width: 1600,
        height: 1000,
      },
    ],
  },
  {
    id: "sessions",
    name: "Sessions",
    section: "Live",
    lead: "Processes grouped into the trees you actually run: an agent with its MCP servers, terminals, dev servers and build children, each with a Keep or Optional verdict.",
    sub: [
      { name: "Dev sessions", body: "Agents, terminals, dev servers, simulators and their helpers." },
      { name: "All apps", body: "Every running application, grouped by app." },
      { name: "Process tree", body: "The ungrouped process table." },
    ],
    points: [
      "Filter by tool, Node version, PID or folder. Select a row for its script, runtime, parent and idle evidence, observed activity, and the session's share of host memory and per-core CPU.",
      "The planner (Free RAM, Cut CPU by, Quiet workstation) only selects rows. Idle agents stay Keep; a recommendation never bypasses a blocker.",
      "Stopping anything is explicit. Terminate session, or Terminate planned… for a planner selection, asks for confirmation, freezes membership by PID plus start time, tries a graceful stop first, and warns when the project has uncommitted Git changes.",
      "Git state for a session's project is read off the UI thread and cached, so selecting a row never stalls the window.",
    ],
    shots: [
      {
        file: "sessions.png",
        screen: "Sessions",
        alt: "SweepLoom Sessions: simulator and app sessions with process counts, RSS, CPU, status and project, a cleanup planner above and per-core host CPU below",
        caption: "Sessions sorted by resident memory, with the planner on top and the selected session's activity below. Window titles are blurred.",
        width: 1600,
        height: 1000,
      },
    ],
  },
  {
    id: "history",
    name: "History",
    section: "Live",
    lead: "What SweepLoom has actually observed, never back-filled. Two views share one tab.",
    sub: [
      {
        name: "Process activity",
        body: "CPU and memory per session since SweepLoom started. Groups start collapsed; click a member to open its session.",
      },
      {
        name: "Disk scans",
        body: "Saved Explorer scans and folder measurements. They survive restarts, so you can see which folders grew back.",
      },
    ],
    points: [
      "Nothing is invented for the time before history was enabled.",
      "Cleanup, Projects and AI link here (Disk scans ›). Their Change column compares finished scans of the same kind.",
      "Only finished measurements become history samples, so a partial scan is never compared with a full one.",
    ],
  },
  {
    id: "cleanup",
    name: "Cleanup",
    section: "Disk",
    lead: "One place for everything reclaimable, by source. Every row shows its origin path or native object ID, size, change since the last scan, and the consequence of removing it.",
    sub: [
      {
        name: "Project output",
        body: "Generated builds, dependencies and temp files across your projects, with rebuild cost and safety. Applied through plan → revalidate → execute → receipt.",
      },
      { name: "Docker", body: "Images, stopped containers, unused volumes and old build cache, removed through Docker itself. In-use objects are protected." },
      {
        name: "iOS & Xcode",
        body: "Simulators read from disk, then confirmed with simctl; runtimes with the devices that depend on them; leftover device folders and caches. Keep / Review / Suggested verdicts; nothing is pre-selected.",
      },
      {
        name: "Build & packages",
        body: "DerivedData per project with a Git-aware verdict, device support per OS version, and npm, pnpm, Yarn, Bun, Python, Cargo, Gradle, NuGet, Go, Homebrew, SwiftPM and Playwright caches.",
      },
      { name: "App & browser caches", body: "Application caches and exact cache subdirectories in browser, IDE and Electron profiles." },
      { name: "AI models", body: "Ollama per model through its local API; Hugging Face, LM Studio, PyTorch, Whisper and Core ML downloads to Trash." },
      {
        name: "Archives & large data",
        body: "Xcode archives, device backups, large downloads, logs, Maven artifacts, Node versions; inactive Rust toolchains via rustup.",
      },
    ],
    points: [
      "Free up N GB → Select picks eligible Project output rows until the target is met: cheapest rebuild first, then largest. Rows blocked by uncommitted Git changes, an active process or a recent write cannot be applied.",
      "In iOS & Xcode, devices whose runtime is gone are offered as one row that deletes exactly the listed shut-down UDIDs, never a blanket simctl delete unavailable, and aborts if that set changed.",
      "User data (archives, backups, downloaded models) goes to the Trash, which is restorable; its space stays used until you empty it.",
      "Each tab keeps its listing, selection, filter, sort and scroll position. Scans keep running in other tabs or in the tray, and a refresh leaves the old rows visible until the new ones land.",
    ],
    shots: [
      {
        file: "project-output.png",
        screen: "Cleanup · Project output",
        alt: "SweepLoom Cleanup, Project output tab: Cargo targets with size, change, rebuild cost and a Safety column showing Blocked · uncommitted git or Safe",
        caption: "Generated output with rebuild cost on every row. Rows in repositories with uncommitted changes are Blocked; the rest are Safe. Paths are blurred.",
        width: 1600,
        height: 909,
        frame: true,
      },
      {
        file: "cleanup.png",
        screen: "Cleanup · iOS & Xcode",
        alt: "SweepLoom Cleanup, iOS & Xcode tab: 136 GB across runtimes and simulator devices with Review, Keep and Suggested verdicts",
        caption: "Runtimes list the simulators that still use them; running devices and devices with apps from active projects are Keep. Device names are blurred.",
        width: 1600,
        height: 1000,
      },
    ],
  },
  {
    id: "projects",
    name: "Projects",
    section: "Disk",
    lead: "Every project under your developer folders, not just the folder the app started in, with full sizes, change since the last scan, and artifact summaries.",
    points: [
      "Discovery covers dev, Developer, Projects, code, src, repos, work, Documents/GitHub, any *Projects folder and loose projects in your home folder, with fair per-folder budgets so one monorepo cannot crowd out the rest.",
      "Markers include Cargo, Node and Deno, Python, Go, Gradle and Maven, Swift, CocoaPods and Xcode, .NET, CMake, Ruby, PHP, Dart, Elixir, Zig, and plain Git repositories. Projects from your last Explorer scan are merged in.",
      "Sizes cover the entire project folder, source included. Folder groups total their projects and nested projects count once. Group by kind, folder or none.",
      "Saved sizes stay on screen, marked Saved, until fresh measurements replace them. Double-click a project to open it in Explorer.",
    ],
    shots: [
      {
        file: "projects.png",
        screen: "Projects",
        alt: "SweepLoom Projects: 556 projects in a dev folder grouped by folder, with kind, size on disk, change since the last scan and artifacts",
        caption: "556 projects in one developer folder, sorted by size on disk, with Cargo workspace and artifact summaries. Project names are blurred.",
        width: 1600,
        height: 1004,
      },
    ],
  },
  {
    id: "explorer",
    name: "Explorer",
    section: "Disk",
    lead: "A folder inspector for when you need to see where the space actually is.",
    points: [
      "The scan runs in the background on a bounded worker pool; each file costs one lstat and folders fold into their parent as soon as their last child finishes.",
      "Sizes are allocated disk blocks, not apparent sizes. Symlinks are not followed.",
      "Expand with the arrow, select with the checkbox, right-click for actions, double-click to scan a folder. Collapsing never re-walks the disk.",
      "Finished scans are saved to History → Disk scans.",
    ],
  },
  {
    id: "browser",
    name: "Browser",
    section: "Workspace",
    lead: "Real memory and CPU per browser. Tabs come from the optional companion extension and are never guessed.",
    sub: [
      { name: "Process trees", body: "Live browser processes, memory and CPU. Open a tree to inspect its helpers." },
      { name: "Tabs", body: "Companion tab table with last access: save to Later, discard, or focus." },
      { name: "Later", body: "Saved tab titles and URLs, reopened without closing anything in the live browser." },
    ],
    points: [
      "Without the companion, tab activity is shown as unknown, not as zero.",
      "Individual helper processes can be stopped on request; the browser process itself cannot.",
      "Pinned, audible, incognito and active tabs are never discarded or closed, and closing is never the suggested default.",
    ],
  },
  {
    id: "ai",
    name: "AI",
    section: "Workspace",
    lead: "Inspect-first stores for Claude, Codex, Cursor, OpenCode, Gemini and Grok, classified by leaf name, with the always-on token estimate.",
    points: [
      "Group by tool, category or none. Each row has a class (Context, History, Cache, Secret, SQLite, Log, Settings, Other) and a policy.",
      "Only regenerable caches and logs are cleanable. History and generated media can be moved to the Trash explicitly. Credentials, databases and managed worktrees stay protected.",
      "History is archive and costs zero always-on tokens. AGENTS.md, *.mdc and rules are Keep; SweepLoom never flips alwaysApply.",
    ],
    shots: [
      {
        file: "ai.png",
        screen: "AI",
        alt: "SweepLoom AI screen: a 79 GB Codex store with sessions, worktrees, SQLite files and caches, each with kind, size, change, file count and policy",
        caption: "A 79 GB Codex store: sessions are History (manual Trash), SQLite files and worktrees are blocked, only tmp and cache are cleanable.",
        width: 1600,
        height: 1000,
      },
    ],
  },
  {
    id: "settings",
    name: "Settings",
    section: "App",
    lead: "Local-only. Telemetry stays off, and there is no switch to turn it on.",
    points: [
      "Appearance: Auto, Dark or Light theme, and interface size.",
      "Background: keep running in the tray when the window closes (Windows and macOS), start in the tray at sign-in (Windows), or launch hidden with sweeploom-gui --tray.",
      "About: telemetry (None), license, the local home and config paths, and the hidden-launch command.",
    ],
  },
];

const NEW = [
  {
    t: "A design system, nine tabs",
    b: "Spacing scale, cards, segmented controls, status pills, stacked bars and sparklines in light and dark. Two screens merged into the tabs where they belong: project output under Cleanup, saved disk scans under History.",
  },
  {
    t: "Per-core CPU",
    b: "Every core with a short history on Overview and Sessions, with the performance / efficiency split on Apple Silicon.",
  },
  {
    t: "iOS & Xcode from disk",
    b: "Simulators read from device records first (simctl could take 40–80 s and time out), then confirmed. Orphan devices, dyld caches, unused runtimes, and Git-aware Keep / Review / Suggested verdicts.",
  },
  {
    t: "Every project, everywhere",
    b: "Discovery across all developer folders instead of the launch directory, with Xcode, Swift, JVM and .NET markers. 598 projects found where the old walk found 237.",
  },
  {
    t: "Parallel, streaming scans",
    b: "Discovery, Project output, project sizes and Explorer run on bounded worker pools and stream rows as they finish. Results survive tab switches and refreshes.",
  },
  {
    t: "Crash log and Windows fixes",
    b: "Panics are written to a log and release builds unwind, so a failed background scan no longer takes the app down. Windows paths now have one canonical shape.",
  },
];

const SPEED: [string, string, string][] = [
  ["Project discovery", "3.2–7.7 s, 237 projects", "0.5–3.5 s, 598 projects"],
  ["Project output across 95 repositories", "343–390 s", "52–109 s"],
  ["Full project sizes", "194–201 s", "59–76 s"],
  ["Explorer, ~3.7 million entries", "~358 s", "130–162 s"],
];

const ICON: Record<string, (p: { size?: number; className?: string }) => React.ReactElement> = {
  overview: icons.grid,
  sessions: icons.sessions,
  history: icons.clock,
  cleanup: icons.build,
  projects: icons.projects,
  explorer: icons.search,
  browser: icons.browser,
  ai: icons.model,
  settings: icons.gear,
};

function TabSection({ tab, index }: { tab: Tab; index: number }) {
  return (
    <section id={tab.id} aria-labelledby={`${tab.id}-title`} className="scroll-mt-24 border-t border-line pt-12 first:border-t-0 first:pt-0">
      <p className="eyebrow">
        {tab.section} · {index + 1} of 9
      </p>
      <h3 id={`${tab.id}-title`} className="mt-2 font-display text-[1.7rem] leading-tight font-semibold tracking-[-0.02em] sm:text-3xl">
        <a href={`#${tab.id}`} className="group inline-flex items-center gap-2">
          {tab.name}
          <span className="text-faint opacity-0 transition-opacity group-hover:opacity-100" aria-hidden="true">
            #
          </span>
        </a>
      </h3>
      <p className="mt-3 max-w-2xl text-[1.05rem] leading-relaxed text-pretty text-muted">{tab.lead}</p>

      {tab.sub ? (
        <div className="mt-6">
          <p className="font-mono text-[0.68rem] tracking-[0.12em] text-faint uppercase">
            {tab.sub.length} {tab.sub.length === 1 ? "view" : "views"}
          </p>
          <ul className={"mt-3 grid gap-3 " + (tab.sub.length > 3 ? "sm:grid-cols-2" : "sm:grid-cols-3")}>
            {tab.sub.map((sub) => (
              <li key={sub.name} className="rounded-xl border border-line bg-panel p-4">
                <p className="text-sm font-medium">{sub.name}</p>
                <p className="mt-1 text-sm leading-relaxed text-muted">{sub.body}</p>
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      {tab.shots ? (
        <div className={"mt-8 grid gap-8 " + (tab.shots.length > 1 ? "xl:grid-cols-2" : "")}>
          {tab.shots.map((shot) => (
            <Screenshot key={shot.file} {...shot} />
          ))}
        </div>
      ) : null}

      <ul className="mt-8 grid gap-3">
        {tab.points.map((point) => (
          <li key={point} className="flex gap-3 text-[0.97rem] leading-relaxed text-muted">
            <icons.check size={18} className="mt-1 shrink-0 text-accent" />
            <span>{point}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

export default function AppTourPage() {
  return (
    <>
      <PageHeader
        eyebrow="Desktop app"
        title="Nine tabs. One safety model."
        lead="SweepLoom's desktop app is a native Rust (egui) reviewer for Windows, macOS and Linux. Live pressure, disk and your workspace, each in its own tab, and nothing destructive without a confirmation that lists exactly what will happen."
      >
        <div className="flex flex-wrap gap-3">
          <Link href="/download/#desktop" className="btn btn-primary">
            Build the app <icons.arrow size={16} />
          </Link>
          <a href="#new" className="btn btn-ghost">
            What is new
          </a>
        </div>
      </PageHeader>

      <Section
        id="new"
        eyebrow="In this release"
        title="Redesigned around nine tabs, and much faster on big machines."
        lead="The desktop app is the part of SweepLoom that changed most since 0.1.0. Everything below is on main today."
      >
        <ul className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {NEW.map((item) => (
            <li key={item.t} className="card p-5">
              <h3 className="font-display text-lg font-semibold">{item.t}</h3>
              <p className="mt-2 text-[0.95rem] leading-relaxed text-muted">{item.b}</p>
            </li>
          ))}
        </ul>
        <div className="mt-8 table-wrap">
          <table>
            <caption className="px-4 pt-4 text-left text-sm text-muted">
              Before and after on the development Mac, measured under heavy machine load, so treat them as ranges, not a
              benchmark.
            </caption>
            <thead>
              <tr>
                <th scope="col">Work</th>
                <th scope="col">Before</th>
                <th scope="col">After</th>
              </tr>
            </thead>
            <tbody>
              {SPEED.map(([work, before, after]) => (
                <tr key={work}>
                  <td className="font-medium">{work}</td>
                  <td className="text-muted">{before}</td>
                  <td className="text-ok">{after}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <p className="mt-4 text-sm text-muted">
          How discovery and the parallel walks work:{" "}
          <Link href="/blog/how-sweeploom-finds-every-project/" className="link">
            How SweepLoom finds every project on your machine
          </Link>
          .
        </p>
      </Section>

      <section aria-labelledby="tour-title" className="border-t border-line">
        <div className="wrap py-16 sm:py-20">
          <div className="max-w-2xl">
            <p className="eyebrow">The tour</p>
            <h2 id="tour-title" className="mt-3 font-display text-[1.75rem] leading-tight font-semibold tracking-[-0.02em] text-balance sm:text-[2.2rem]">
              The sidebar, tab by tab.
            </h2>
            <p className="mt-4 text-[1.05rem] leading-relaxed text-pretty text-muted">
              Same order as the app. Screenshots are real captures with names and paths blurred; tabs without one are described
              rather than mocked up.
            </p>
          </div>

          <div className="mt-10 grid gap-10 lg:grid-cols-[220px_minmax(0,1fr)] lg:gap-14">
            <nav id="app-tabs" aria-label="App tabs" className="app-nav lg:sticky lg:top-24 lg:self-start">
              <div className="flex gap-6 overflow-x-auto pb-2 lg:block lg:overflow-visible lg:rounded-2xl lg:border lg:border-line lg:bg-[var(--term-bg)] lg:p-4 lg:pb-4">
                {GUI_SCREENS.map((group) => (
                  <div key={group.section} className="shrink-0 lg:mb-4 lg:last:mb-0">
                    <p className="font-mono text-[0.62rem] tracking-[0.16em] text-faint uppercase lg:text-[var(--term-dim)]">{group.section}</p>
                    <ul className="mt-2 flex gap-1 lg:grid lg:gap-0.5">
                      {group.items.map((item) => {
                        const Icon = ICON[item.id];
                        return (
                          <li key={item.id}>
                            <a
                              href={`#${item.id}`}
                              className="flex items-center gap-2 rounded-md border-l-2 border-transparent px-2.5 py-1.5 text-sm whitespace-nowrap text-muted transition-colors hover:text-text lg:text-[var(--term-fg)]/80 lg:hover:bg-white/[0.04] lg:hover:text-[var(--term-fg)]"
                            >
                              <Icon size={15} className="shrink-0 text-accent lg:text-[var(--term-gold)]" />
                              {item.name}
                            </a>
                          </li>
                        );
                      })}
                    </ul>
                  </div>
                ))}
              </div>
            </nav>

            <div className="grid min-w-0 gap-12">
              {TABS.map((tab, index) => (
                <TabSection key={tab.id} tab={tab} index={index} />
              ))}
            </div>
            <ScrollSpy navId="app-tabs" ids={TABS.map((tab) => tab.id)} />
          </div>
        </div>
      </section>

      <Section
        id="safety"
        eyebrow="One safety model"
        title="The app is a reviewer. You are the one who confirms."
        lead="The same rules as the CLI and MCP, plus the checks that only a desktop app with native tools needs."
        className="border-t border-line"
      >
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {[
            ["Plan", "Rows are built from metadata. Only SAFE generated rows can be pre-selected, and native objects are never selected for you."],
            ["Revalidate", "Before acting: live working directories, executables and arguments, running simulators, in-use Docker objects, loaded Ollama models."],
            ["Execute", "Generated output is deleted; user data goes to the Trash; Docker, simctl, Ollama and rustup remove their own objects."],
            ["Receipt", "A result line per action, and finished measurements recorded in History → Disk scans."],
          ].map(([t, b], i) => (
            <div key={t} className="card p-5">
              <span className="font-mono text-xs text-accent">0{i + 1}</span>
              <h3 className="mt-2 font-display text-lg font-semibold">{t}</h3>
              <p className="mt-2 text-sm leading-relaxed text-muted">{b}</p>
            </div>
          ))}
        </div>
        <ul className="mt-8 grid gap-3 md:grid-cols-2">
          {[
            "Partial scans authorize nothing. An unreadable size stays unknown or partial, never an exact zero.",
            "Sizes are allocated blocks, so sparse simulator images do not promise space they cannot free.",
            "Idle agents are Keep. Stopping a session is a separate, confirmed action; browsers cannot be stopped from Sessions.",
            "If the app ever panics, the message goes to ~/Library/Logs/SweepLoom/panic.log on macOS (SweepLoom/panic.log under %LOCALAPPDATA% on Windows, ~/.local/state on Linux).",
          ].map((line) => (
            <li key={line} className="flex gap-3 rounded-xl border border-line bg-panel p-4 text-[0.95rem] leading-relaxed text-muted">
              <icons.shield size={18} className="mt-0.5 shrink-0 text-accent" />
              <span>{line}</span>
            </li>
          ))}
        </ul>
      </Section>

      <section aria-labelledby="build-title" className="wrap">
        <div className="card relative overflow-hidden p-8 sm:p-12">
          <div className="gold-glow pointer-events-none absolute -top-32 -right-24 h-80 w-[36rem]" aria-hidden="true" />
          <div className="relative grid gap-8 lg:grid-cols-[1.1fr_1fr] lg:items-center">
            <div>
              <h2 id="build-title" className="font-display text-3xl font-semibold tracking-tight text-balance sm:text-4xl">
                Build it from source.
              </h2>
              <p className="mt-3 max-w-lg text-muted">
                The app needs Rust 1.95 or newer and is optional; the CLI and MCP server do not need it. On macOS, build the{" "}
                <a href={docsUrl("MACOS.md")} className="link">
                  signed app bundle
                </a>{" "}
                so folder permissions survive updates.
              </p>
              <div className="mt-6 flex flex-wrap gap-3">
                <Link href="/download/#desktop" className="btn btn-primary">
                  Download instructions
                </Link>
                <Link href="/blog/desktop-app-tour/" className="btn btn-ghost">
                  Read the tour post
                </Link>
              </div>
            </div>
            <CodeBlock code={`$ ${INSTALL.guiClone}\n$ cd sweeploom\n$ ${INSTALL.guiRun}`} title="build and run" />
          </div>
        </div>
      </section>
    </>
  );
}
