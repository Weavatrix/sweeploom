import Link from "next/link";
import { CodeBlock } from "@/components/CodeBlock";
import { icons } from "@/components/Icons";
import { PageHeader, Section } from "@/components/Section";
import { BENCH_TRANSCRIPT, CLASSES, CLEAN_TRANSCRIPT, HARD_LINES, MCP_TOOLS } from "@/lib/content";
import { pageMeta } from "@/lib/meta";
import { docsUrl } from "@/lib/site";

export const metadata = pageMeta({
  title: "Features",
  description:
    "Live session trees that keep your agents alive, generated build output and package caches, Xcode, iOS Simulator, Docker and AI model cleanup, the always-on prompt tax, browser pressure, and a gated apply path for people and agents.",
  path: "/features/",
});

const JUMP = [
  { id: "sessions", label: "Live sessions" },
  { id: "disk", label: "Disk & projects" },
  { id: "cleanup", label: "Desktop Cleanup" },
  { id: "ai", label: "AI context" },
  { id: "browser", label: "Browser" },
  { id: "safety", label: "Safety model" },
  { id: "agents", label: "For agents" },
  { id: "platforms", label: "Platforms" },
];

const SESSION_COLUMNS = [
  ["[ ] / [x]", "Whether the session is in a dry-run plan (--quiet, --free-ram, --reduce-cpu)"],
  ["Cursor / Codex / Claude Code", "The whole tree, not 43 Task Manager lines"],
  ["proc=", "Members of that tree"],
  ["rss=", "Sum of resident memory. Not uniquely reclaimable"],
  ["rec=Keep", "Do not terminate: the chat context lives here"],
  ["last column", "Attributed project path, or - when unknown"],
];

const CLEANUP_SECTIONS = [
  ["Docker", "Individual images, stopped containers, unused volumes and old build cache", "Native Docker operations; in-use objects are protected"],
  ["iOS Simulator", "Individual devices and removable runtimes", "Native simctl delete or erase, with running-device checks"],
  ["Build & packages", "DerivedData by project; iOS, watchOS and tvOS device support by version; npm/npx, pnpm, Yarn, Bun, Python, Cargo, Gradle, NuGet, Go, Homebrew, Playwright/Puppeteer, SwiftPM and other download caches", "Confirmed generated cleanup, or move selected paths to Trash"],
  ["App & browser caches", "User application caches and exact cache subdirectories in browser, IDE and Electron profiles", "Generated cleanup for recognised cache paths; Trash for other application caches"],
  ["AI models", "Ollama by model, Hugging Face by repository or dataset, LM Studio, PyTorch, Whisper and Core ML downloads", "Ollama's local API, or Trash for individual downloaded model files"],
  ["Archives & large data", "Xcode release archives, device backups, large downloads and installers, logs, Maven artifacts, Rust toolchains and Node versions", "Trash for user data; rustup uninstall for inactive, non-default compilers"],
];

const SURFACE_GATES = [
  ["CLI", "sweeploom clean . (dry run)", "--apply is consent; only pre-selected SAFE rows", "receipt=1 deleted=2 skipped_changed=0 failed=0"],
  ["MCP", "cleanup_candidates, explain_candidate", "apply_cleanup with confirm: true and the agreed ids", "{ ok, deleted, skipped_changed, failed, receipt }"],
  ["Desktop app", "Review and Cleanup tables", "A confirmation that lists exactly the eligible selected rows", "Result line per action, scan history updated"],
];

const PLATFORMS = [
  ["CLI, MCP server, Rust library", "yes", "yes", "yes", "CI matrix: Ubuntu, Windows, macOS"],
  ["Desktop app (egui)", "yes", "yes", "yes", "Follows light/dark and interface scale"],
  ["Tray icon", "yes", "yes", "no", "Hide to tray; scans keep running"],
  ["Xcode & iOS Simulator cleanup", "no", "yes", "no", "Uses xcrun simctl"],
  ["Browser companion", "yes", "yes", "yes", "Chromium and Firefox extensions, native messaging"],
];

function H3({ children }: { children: React.ReactNode }) {
  return <h3 className="font-display text-xl font-semibold">{children}</h3>;
}

function P({ children }: { children: React.ReactNode }) {
  return <p className="mt-3 leading-relaxed text-muted">{children}</p>;
}

function Yes({ v }: { v: string }) {
  return v === "yes" ? (
    <span className="inline-flex items-center gap-1 text-ok">
      <icons.check size={15} /> <span className="sr-only">yes</span>
    </span>
  ) : (
    <span className="text-faint">—<span className="sr-only">no</span></span>
  );
}

export default function FeaturesPage() {
  return (
    <>
      <PageHeader
        eyebrow="Features"
        title="One engine. A CLI for hands, MCP for agents, an app for review."
        lead="SweepLoom answers three questions locally and without an LLM: what is sitting on disk, in RAM and in the agent host; whether it is still earning its keep; and how to reclaim it without losing the workspace."
      >
        <nav aria-label="On this page">
          <ul className="flex flex-wrap gap-2">
            {JUMP.map((item) => (
              <li key={item.id}>
                <a href={`#${item.id}`} className="chip transition-colors hover:border-gold hover:text-text">
                  {item.label}
                </a>
              </li>
            ))}
          </ul>
        </nav>
      </PageHeader>

      <Section
        id="sessions"
        eyebrow="Live sessions"
        title="Sessions, not processes. Agents stay Keep."
        lead="Processes are grouped into the trees you actually run: an agent with its MCP servers, terminals, dev servers and build children. Idle Claude Code, Codex, Cursor, OpenCode, Gemini and Grok are Keep; an MCP server with no live agent parent can be Optional."
      >
        <div className="grid gap-8 lg:grid-cols-2">
          <div className="table-wrap">
            <table>
              <caption className="sr-only">How to read a sweeploom sessions row</caption>
              <thead>
                <tr>
                  <th scope="col">Column</th>
                  <th scope="col">Meaning</th>
                </tr>
              </thead>
              <tbody>
                {SESSION_COLUMNS.map(([c, m]) => (
                  <tr key={c}>
                    <td>
                      <code>{c}</code>
                    </td>
                    <td className="text-muted">{m}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div>
            <CodeBlock
              code={`$ sweeploom sessions --quiet\n$ sweeploom sessions --free-ram 4\n$ sweeploom sessions --reduce-cpu 20\nplan=N session(s); dry-run only — terminate is not offered on the CLI`}
              title="dry-run plans"
            />
            <ul className="mt-6 grid gap-3 text-[0.95rem] leading-relaxed text-muted">
              <li>
                <strong className="text-text">Process identity is PID plus start time,</strong> so a recycled PID is never confused
                with the process you meant.
              </li>
              <li>
                <strong className="text-text">Command lines are redacted</strong> before they reach the UI, logs or receipts.
              </li>
              <li>
                <strong className="text-text">Observed, not guessed:</strong> CPU averages appear only after SweepLoom has watched
                a session; TCP bytes appear only where the OS exposes them.
              </li>
              <li>
                <strong className="text-text">The desktop app can stop a session</strong> when a person explicitly confirms it:
                membership is frozen first, the confirmation is cancelled if it changes, and graceful stop comes before force.
              </li>
            </ul>
            <Link href="/blog/why-sweeploom-never-kills-idle-agents/" className="link mt-6 inline-flex items-center gap-1.5 text-sm font-medium">
              Why SweepLoom will never kill your idle agent <icons.arrow size={14} />
            </Link>
          </div>
        </div>
      </Section>

      <Section
        id="disk"
        eyebrow="Disk & projects"
        title="Generated output is SAFE. Your source is not a candidate."
        lead="scan maps a tree, projects scores source and artifact heat with Git state, and clean lists only rebuildable artifacts. .git is listed but never a clean candidate; git=dirty tells you source is live, it never authorizes deleting it."
        className="border-t border-line"
      >
        <div className="grid gap-8 lg:grid-cols-2">
          <CodeBlock code={CLEAN_TRANSCRIPT} title="look, then --apply" />
          <div className="grid gap-4">
            <CodeBlock
              code={`$ sweeploom projects C:\\work\\app\nC:\\work\\app  kind=[Cargo]  source=Warm  artifact=Hot  git=Clean\n  cargo Target  1.2 GB  rebuild=Cheap`}
              title="projects: heat, Git, offers"
            />
            <p className="leading-relaxed text-muted">
              <code className="inline-code">[x]</code> means pre-selected SAFE generated.{" "}
              <code className="inline-code">[ ]</code> with <code className="inline-code">BLOCKED</code> will not go out with{" "}
              <code className="inline-code">--apply</code>, even if you pass it without looking. Cargo target size is
              attributed to the workspace root; Python generated output is offered too.
            </p>
          </div>
        </div>
      </Section>

      <Section
        id="cleanup"
        eyebrow="Desktop Cleanup"
        title="Docker, iOS Simulator, caches and models, itemised."
        lead="The desktop app's Cleanup screen discovers known locations and streams full measurements in the background. Every row shows its origin path or native object ID, size, file count when known, change since the last scan, and the consequence of removing it."
        className="border-t border-line"
      >
        <div className="table-wrap">
          <table>
            <caption className="sr-only">Cleanup sections, sources and actions</caption>
            <thead>
              <tr>
                <th scope="col">Section</th>
                <th scope="col">Sources and granularity</th>
                <th scope="col">Action</th>
              </tr>
            </thead>
            <tbody>
              {CLEANUP_SECTIONS.map(([s, src, a]) => (
                <tr key={s}>
                  <td className="font-medium whitespace-nowrap">{s}</td>
                  <td className="text-muted">{src}</td>
                  <td className="text-muted">{a}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="mt-8 grid gap-4 md:grid-cols-3">
          {[
            ["Allocated blocks, not sparse capacities", "Sizes reflect what is actually occupied. Unreadable measurements stay unknown or partial, never an exact zero."],
            ["Partial scans authorize nothing", "File cleanup checks live working directories, executables and arguments; generated cleanup also revalidates metadata revisions."],
            ["Data is not cache", "Archives, backups and downloaded models are never assumed regenerable. Trash is restorable in Finder; its space stays used until you empty it."],
          ].map(([t, b]) => (
            <div key={t} className="card p-5">
              <p className="font-medium">{t}</p>
              <p className="mt-2 text-sm leading-relaxed text-muted">{b}</p>
            </div>
          ))}
        </div>
        <p className="mt-6 text-sm text-muted">
          Ollama models are removed through Ollama&apos;s own API on <code className="inline-code">127.0.0.1:11434</code>, after
          re-checking loaded models and the selected digest. Android images and virtual machines are measured and shown for
          inspection with guidance to their native manager. Source catalog:{" "}
          <a href={docsUrl("CLEANUP.md")} className="link">
            docs/CLEANUP.md
          </a>
          .{" "}
          <Link href="/blog/reclaiming-xcode-and-ios-simulator-disk/" className="link">
            Reclaiming Xcode and iOS Simulator disk safely
          </Link>
          .
        </p>
      </Section>

      <Section
        id="ai"
        eyebrow="AI context"
        title="The always-on prompt tax, counted honestly."
        lead="sweeploom ai lists the stores your coding agents keep under your home directory and estimates what is injected every turn. History is archive and counts as zero. Classification uses the leaf name only and never opens the file."
        className="border-t border-line"
      >
        <div className="table-wrap">
          <table>
            <caption className="sr-only">AI store classes</caption>
            <thead>
              <tr>
                <th scope="col">Leaf</th>
                <th scope="col">Class</th>
                <th scope="col">Clean?</th>
                <th scope="col">Always-on tokens</th>
              </tr>
            </thead>
            <tbody>
              {CLASSES.map((row) => (
                <tr key={row.leaf}>
                  <td>
                    <code>{row.leaf}</code>
                  </td>
                  <td>{row.cls}</td>
                  <td className={row.clean === "yes" ? "text-ok" : "text-muted"}>{row.clean}</td>
                  <td className="text-muted">{row.tokens}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="mt-8 grid gap-8 lg:grid-cols-2">
          <CodeBlock code={BENCH_TRANSCRIPT} title="fixed gold store, not your disk" />
          <ul className="grid content-start gap-3 text-[0.95rem] leading-relaxed text-muted">
            <li>
              <strong className="text-text">keep</strong> for <code className="inline-code">AGENTS.md</code>,{" "}
              <code className="inline-code">*.mdc</code> and <code className="inline-code">rules</code>, even when idle.
            </li>
            <li>
              <strong className="text-text">park?</strong> for <code className="inline-code">skills</code> or{" "}
              <code className="inline-code">plugins</code> idle 30 days or more. Advice only: SweepLoom never flips{" "}
              <code className="inline-code">alwaysApply</code>.
            </li>
            <li>
              <strong className="text-text">Secrets and SQLite</strong> are never opened and never cleanable. Only cache and log
              rows are cleanable, and <code className="inline-code">sweeploom ai</code> itself never deletes.
            </li>
            <li>
              The fixture is policy, not spend: a live coding-agent run found no coding-task token saving.{" "}
              <Link href="/blog/always-on-prompt-tax/" className="link">
                Read the full write-up
              </Link>
              .
            </li>
          </ul>
        </div>
      </Section>

      <Section
        id="browser"
        eyebrow="Browser"
        title="Real pressure. Tabs are never guessed."
        lead="sweeploom browser reports RSS and CPU per browser tree. Without the companion, tab activity is shown as unknown, because tabs=0 would be a lie."
        className="border-t border-line"
      >
        <div className="grid gap-8 lg:grid-cols-2">
          <CodeBlock
            code={`$ sweeploom browser\ncompanion=disconnected hosts=1 rss=8.2 GB\ntab lastAccessed unavailable without the companion; not shown as zero\nEdge     sessions=1   proc=62   rss=8.2 GB     cpu= 13.2%`}
            title="without the companion"
          />
          <div className="leading-relaxed text-muted">
            <p>
              The optional <strong className="text-text">SweepLoom Companion</strong> extension for Chromium browsers and Firefox
              reports each tab&apos;s title, URL (with credentials, query string and fragment stripped) and{" "}
              <code className="inline-code">lastAccessed</code> to the local SweepLoom process over native messaging. It never
              closes tabs by default and makes no network requests. When you choose it in the desktop app, it can discard a tab
              (unload it from memory, keeping it in the strip), focus a tab, or bookmark and close one, closing only if the
              bookmark exists and the URL is unchanged. Pinned, audible, incognito and active tabs are never discarded or
              closed. The Later shelf saves a tab&apos;s title and URL locally without closing anything.
            </p>
            <CodeBlock
              className="mt-5"
              code={`$ sweeploom companion-install\n$ sweeploom companion-install --chromium-id <id from edge://extensions>`}
              title="install the native host"
            />
          </div>
        </div>
      </Section>

      <Section
        id="safety"
        eyebrow="Safety model"
        title="Plan, revalidate, execute, receipt. On every surface."
        lead="Safety and recommendation are independent axes, and a recommendation never overrides a blocker. If you rebuilt target/ after the plan, that row is skipped, not forced."
        className="border-t border-line"
      >
        <div className="table-wrap">
          <table>
            <caption className="sr-only">How each surface gates apply</caption>
            <thead>
              <tr>
                <th scope="col">Surface</th>
                <th scope="col">Look</th>
                <th scope="col">Consent</th>
                <th scope="col">Receipt</th>
              </tr>
            </thead>
            <tbody>
              {SURFACE_GATES.map(([s, l, c, r]) => (
                <tr key={s}>
                  <td className="font-medium whitespace-nowrap">{s}</td>
                  <td className="text-muted">{l}</td>
                  <td className="text-muted">{c}</td>
                  <td>
                    <code>{r}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <ul className="mt-8 grid gap-3 md:grid-cols-2">
          {HARD_LINES.map((line) => (
            <li key={line} className="flex gap-3 rounded-xl border border-line bg-panel p-4 text-[0.95rem] leading-relaxed text-muted">
              <icons.shield size={18} className="mt-0.5 shrink-0 text-accent" />
              <span>{line}</span>
            </li>
          ))}
        </ul>
      </Section>

      <Section
        id="agents"
        eyebrow="For agents"
        title="Nine MCP tools. One of them writes, and only with confirm=true."
        lead="SweepLoom is an MCP server over stdio (io.github.Weavatrix/sweeploom). The Codex plugin bundles the same server with three skills (review-cleanup, token-tax and live-sessions) that encode review-before-apply."
        className="border-t border-line"
      >
        <div className="table-wrap">
          <table>
            <caption className="sr-only">MCP tools and arguments</caption>
            <thead>
              <tr>
                <th scope="col">Tool</th>
                <th scope="col">Arguments</th>
                <th scope="col">Writes</th>
                <th scope="col">What it returns</th>
              </tr>
            </thead>
            <tbody>
              {MCP_TOOLS.map((tool) => (
                <tr key={tool.name}>
                  <td>
                    <code>{tool.name}</code>
                  </td>
                  <td className="font-mono text-xs whitespace-nowrap text-muted">{tool.args}</td>
                  <td className={tool.writes === "no" ? "text-muted" : "font-medium text-warn"}>{tool.writes}</td>
                  <td className="text-muted">{tool.does}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="mt-8 grid gap-8 lg:grid-cols-2">
          <CodeBlock
            code={`tools/call  apply_cleanup  { "confirm": false, "root": "C:/work/app" }\n\nrefused: MCP apply needs confirm=true after cleanup_candidates; CLI uses --apply`}
            title="an agent that tries to skip review"
            copy={false}
          />
          <div className="leading-relaxed text-muted">
            <p>
              There is no <code className="inline-code">terminate</code>, no <code className="inline-code">park_file</code> and no{" "}
              <code className="inline-code">rewrite_agents</code>. Read-only tools publish{" "}
              <code className="inline-code">readOnlyHint</code>. Empty <code className="inline-code">root</code> means your home
              directory, so agents should always pass the workspace path.
            </p>
            <Link href="/docs/#mcp" className="link mt-4 inline-flex items-center gap-1.5 text-sm font-medium">
              Set up MCP in Cursor, Claude Code, Claude Desktop or Codex <icons.arrow size={14} />
            </Link>
          </div>
        </div>
      </Section>

      <Section id="platforms" eyebrow="Platforms" title="Where it runs." className="border-t border-line">
        <div className="table-wrap">
          <table>
            <caption className="sr-only">Platform support</caption>
            <thead>
              <tr>
                <th scope="col">Component</th>
                <th scope="col">Windows</th>
                <th scope="col">macOS</th>
                <th scope="col">Linux</th>
                <th scope="col">Notes</th>
              </tr>
            </thead>
            <tbody>
              {PLATFORMS.map(([c, w, m, l, n]) => (
                <tr key={c}>
                  <td className="font-medium">{c}</td>
                  <td>
                    <Yes v={w} />
                  </td>
                  <td>
                    <Yes v={m} />
                  </td>
                  <td>
                    <Yes v={l} />
                  </td>
                  <td className="text-muted">{n}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Section>
    </>
  );
}
