import Link from "next/link";
import { CodeBlock } from "@/components/CodeBlock";
import { icons } from "@/components/Icons";
import { PageHeader } from "@/components/Section";
import { MCP_TOOLS } from "@/lib/content";
import { pageMeta } from "@/lib/meta";
import { CODEX_TOML, INSTALL, MCP_JSON, SITE, docsUrl, repoFile } from "@/lib/site";

export const metadata = pageMeta({
  title: "Docs",
  description:
    "Install SweepLoom from source, run the CLI, connect the MCP server to Cursor, Claude Code, Claude Desktop or Codex, use the Rust library, and find the full reference docs on GitHub.",
  path: "/docs/",
});

const TOC = [
  { id: "overview", label: "Overview" },
  { id: "install", label: "Install" },
  { id: "quickstart", label: "Quick start" },
  { id: "cli", label: "CLI reference" },
  { id: "mcp", label: "MCP setup" },
  { id: "codex", label: "Codex plugin" },
  { id: "library", label: "Rust library" },
  { id: "desktop", label: "Desktop app" },
  { id: "companion", label: "Browser companion" },
  { id: "reference", label: "Reference docs" },
];

const CLI_REF: [string, string][] = [
  ["sweeploom sessions [--free-ram GB] [--reduce-cpu PERCENT] [--quiet]", "Live session trees. Flags print a dry-run plan; nothing is terminated."],
  ["sweeploom scan [path]", "Disk inventory of a tree. No path means home; pass . for the current tree."],
  ["sweeploom projects [path]", "Project kinds, source and artifact heat, Git state, artifact offers."],
  ["sweeploom clean [path] [--apply]", "List generated cleanup candidates; --apply deletes pre-selected SAFE rows after revalidation."],
  ["sweeploom ai", "AI tool stores under home: classes, sizes, always-on token estimates. Never deletes."],
  ["sweeploom browser", "Browser process pressure; tab activity via the companion."],
  ["sweeploom bench", "With vs without SweepLoom on a fixed gold store."],
  ["sweeploom mcp [--list]", "Serve MCP over stdio (for hosts). --list prints the tool contract instead."],
  ["sweeploom companion-install [--chromium-id ID]", "Register the browser companion's native messaging host."],
  ["sweeploom companion-host", "What the browser launches. Not a daily command."],
];

const REFERENCE: { file: string; title: string; body: string; root?: boolean }[] = [
  { file: "INSTALL.md", title: "Install", body: "From source, desktop app, MCP hosts, Codex plugin." },
  { file: "CONCEPTS.md", title: "The idea", body: "Two kinds of waste, why live agents are Keep, scope." },
  { file: "CLI.md", title: "CLI transcripts", body: "Help, sessions, scan, projects and AI stores from a real machine." },
  { file: "CLI-CLEAN.md", title: "CLI: bench, browser, clean", body: "The second half, including apply and the agent contract." },
  { file: "MCP.md", title: "MCP for agents", body: "Setup per host, silent vs told, every tool and its arguments." },
  { file: "MCP-CHATS.md", title: "MCP chats: review, then apply", body: "Copyable agent conversations, including the refused shortcut." },
  { file: "MCP-CHATS-AGENTS.md", title: "MCP chats: tokens and live agents", body: "Token tax, idle Cursor, folder map." },
  { file: "PLUGIN.md", title: "Codex plugin", body: "Install, skills, layout, what it will not do." },
  { file: "LIBRARY.md", title: "Rust library", body: "Classify, estimate, advise, list, plan and apply from Rust." },
  { file: "SAFETY.md", title: "Safety model and classes", body: "The apply path, every class, the MCP gate." },
  { file: "CLEANUP.md", title: "Desktop Cleanup sources", body: "Docker, simulator, caches, models and archives." },
  { file: "MACOS.md", title: "Signed macOS app", body: "Building and signing the app bundle." },
  { file: "BROWSER.md", title: "Browser companion", body: "Install, what it sends, tab actions." },
  { file: "BENCHMARKS.md", title: "Benchmarks", body: "With vs without: token tax, classify, live agents, MCP gate." },
  { file: "BENCHMARKS-LIVE.md", title: "Live coding-agent check", body: "The 2026-09-15 run that found no token saving." },
  { file: "TESTING.md", title: "Tests", body: "Test inventory and CI." },
  { file: "ARCHITECTURE.md", title: "Architecture", body: "Crate map, license boundary, invariants." },
  { file: "FAQ.md", title: "FAQ", body: "Short answers." },
  { file: "TERMS.md", title: "Terms of Use", body: "The same terms as this site's Terms page.", root: true },
];

function DocSection({ id, title, children }: { id: string; title: string; children: React.ReactNode }) {
  return (
    <section id={id} aria-labelledby={`${id}-title`} className="scroll-mt-24 border-t border-line pt-10 first:border-t-0 first:pt-0">
      <h2 id={`${id}-title`} className="font-display text-2xl font-semibold tracking-tight">
        <a href={`#${id}`} className="group inline-flex items-center gap-2">
          {title}
          <span className="text-faint opacity-0 transition-opacity group-hover:opacity-100" aria-hidden="true">
            #
          </span>
        </a>
      </h2>
      <div className="mt-5 grid gap-5 leading-relaxed text-muted [&_strong]:text-text">{children}</div>
    </section>
  );
}

function C({ children }: { children: React.ReactNode }) {
  return <code className="inline-code">{children}</code>;
}

export default function DocsPage() {
  return (
    <>
      <PageHeader
        eyebrow="Docs"
        title="Get SweepLoom running, then point your agent at it."
        lead="The essentials are on this page. The complete reference, with full transcripts from a real machine, lives in the repository's docs folder."
      />
      <div className="wrap grid gap-12 py-14 lg:grid-cols-[220px_minmax(0,1fr)]">
        <nav aria-label="Docs contents" className="lg:sticky lg:top-24 lg:self-start">
          <p className="eyebrow !text-faint">On this page</p>
          <ul className="mt-4 flex flex-wrap gap-2 lg:grid lg:gap-1">
            {TOC.map((item) => (
              <li key={item.id}>
                <a
                  href={`#${item.id}`}
                  className="chip lg:rounded-md lg:border-0 lg:px-2 lg:py-1.5 lg:font-sans lg:text-sm lg:tracking-normal lg:hover:bg-panel lg:hover:text-text"
                >
                  {item.label}
                </a>
              </li>
            ))}
          </ul>
        </nav>

        <div className="grid min-w-0 gap-12">
          <DocSection id="overview" title="Overview">
            <p>
              SweepLoom is a <strong>developer-aware workstation resource manager</strong>. It answers three questions locally,
              without an LLM on classify or apply: what is sitting on disk, in RAM and in the agent host; whether it is still
              earning its keep; and how to reclaim it without losing the workspace.
            </p>
            <div className="grid gap-4 sm:grid-cols-3">
              {[
                ["CLI", "sweeploom", "For hands. --apply is consent."],
                ["MCP server", "sweeploom mcp", "For agents. Apply needs confirm=true."],
                ["Desktop app", "sweeploom-gui", "A local reviewer for people."],
              ].map(([t, cmd, b]) => (
                <div key={t} className="card p-5">
                  <p className="font-medium text-text">{t}</p>
                  <p className="mt-1 font-mono text-xs text-accent">{cmd}</p>
                  <p className="mt-2 text-sm">{b}</p>
                </div>
              ))}
            </div>
          </DocSection>

          <DocSection id="install" title="Install">
            <p>
              SweepLoom installs from source with Cargo. It is not on crates.io or npm yet, and there are no prebuilt
              releases. You need <strong>Git</strong> and Rust via{" "}
              <a href="https://rustup.rs" className="link">
                rustup
              </a>
              : <strong>1.88 or newer</strong> for the CLI and library, <strong>1.95 or newer</strong> for the desktop app.
            </p>
            <CodeBlock code={`$ ${INSTALL.cli}\n$ sweeploom --help\n$ sweeploom mcp --list`} title="CLI + MCP server" />
            <p>
              This puts two binaries in <C>~/.cargo/bin</C>: <C>sweeploom</C>, the CLI, which serves MCP as{" "}
              <C>sweeploom mcp</C>, and <C>sweeploom-companion-host</C>, which the browser companion launches. There is no
              separate <C>sweeploom-mcp</C> binary. Update by re-running the line with <C>--force</C>; remove with{" "}
              <C>cargo uninstall sweeploom</C>.
            </p>
            <p>
              For the desktop app, the signed macOS bundle and uninstalling, see{" "}
              <Link href="/download/" className="link">
                Download
              </Link>
              .
            </p>
          </DocSection>

          <DocSection id="quickstart" title="Quick start">
            <p>Look first. Every command below is read-only except the last one.</p>
            <CodeBlock
              code={`$ sweeploom sessions             # what is alive; agents are Keep\n$ sweeploom ai                   # AI stores and always-on token tax\n$ sweeploom scan .               # what is on disk in this tree\n$ sweeploom projects .           # heat, Git state, artifact offers\n$ sweeploom clean .              # cleanup candidates, dry run\n$ sweeploom clean . --apply      # delete pre-selected SAFE rows`}
              title="your first five minutes"
            />
            <p>
              <C>clean .</C> printing <C>no generated candidates</C> is a valid result: the walk found nothing SAFE to remove.
              Bare <C>sweeploom</C> prints help; it never waits on stdin.
            </p>
          </DocSection>

          <DocSection id="cli" title="CLI reference">
            <div className="table-wrap">
              <table>
                <caption className="sr-only">SweepLoom CLI commands</caption>
                <thead>
                  <tr>
                    <th scope="col">Command</th>
                    <th scope="col">What it does</th>
                  </tr>
                </thead>
                <tbody>
                  {CLI_REF.map(([cmd, what]) => (
                    <tr key={cmd}>
                      <td className="min-w-[16rem]">
                        <code>{cmd}</code>
                      </td>
                      <td className="text-muted">{what}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <p>
              Help goes to stderr. An unknown command prints help and exits with status 2; an unknown flag is rejected the same way, so there is no hidden <C>--kill</C> or <C>--yes</C>. From a clone without
              installing, replace <C>sweeploom</C> with <C>cargo run -p sweeploom --</C>; the <C>--</C> keeps the flags for
              SweepLoom instead of Cargo.
            </p>
          </DocSection>

          <DocSection id="mcp" title="MCP setup">
            <p>
              SweepLoom speaks MCP over stdio. Server id: <C>{SITE.mcpName}</C>. Your host launches{" "}
              <C>sweeploom mcp</C>; you never type it into a terminal yourself. SweepLoom is an MCP server, not a Cursor
              Marketplace plugin, so searching the marketplace will find nothing. Add the server entry, then reload the host.
            </p>
            <h3 className="font-display text-lg font-semibold text-text">Cursor and Claude Desktop</h3>
            <p>
              Cursor: <C>~/.cursor/mcp.json</C> for every workspace (<C>%USERPROFILE%\.cursor\mcp.json</C> on Windows), or{" "}
              <C>.cursor/mcp.json</C> in a project. Claude Desktop:{" "}
              <C>~/Library/Application Support/Claude/claude_desktop_config.json</C> on macOS,{" "}
              <C>%APPDATA%\Claude\claude_desktop_config.json</C> on Windows.
            </p>
            <CodeBlock code={MCP_JSON} title="mcp.json" />
            <p>
              Hosts started from the Dock, Start menu or a launcher may not see your shell <C>PATH</C>. If the server does
              not start, use the absolute path: <C>~/.cargo/bin/sweeploom</C> on macOS and Linux,{" "}
              <C>C:\Users\you\.cargo\bin\sweeploom.exe</C> on Windows.
            </p>
            <h3 className="font-display text-lg font-semibold text-text">Claude Code</h3>
            <CodeBlock code="$ claude mcp add -s user sweeploom -- sweeploom mcp" title="user scope" />
            <h3 className="font-display text-lg font-semibold text-text">Codex</h3>
            <p>
              Prefer the <a href="#codex" className="link">plugin</a>. For MCP only, add the server to{" "}
              <C>~/.codex/config.toml</C>:
            </p>
            <CodeBlock code={CODEX_TOML} title="~/.codex/config.toml" />
            <p>
              Prefer the installed binary for always-on hosts. Wiring a host to <C>cargo run … -- mcp</C> recompiles on host
              restarts, writes build diagnostics into the MCP stream, and can disconnect when the disk is full.
            </p>
            <h3 className="font-display text-lg font-semibold text-text">Using it well</h3>
            <ul className="list-disc space-y-2 pl-5 marker:text-gold">
              <li>
                <strong>Name the product.</strong> A connected server can still be ignored if the model decides to use the shell.
                Say “Use SweepLoom to…” when you want its tools.
              </li>
              <li>
                <strong>Pass the workspace path.</strong> An empty <C>root</C> on path tools means your home directory.
              </li>
              <li>
                <strong>Review, then apply.</strong> <C>cleanup_candidates</C>, then <C>explain_candidate</C>, then ask, then{" "}
                <C>apply_cleanup</C> with <C>confirm: true</C> and exactly the agreed <C>ids</C>. Empty <C>ids</C> means every
                pre-selected SAFE row.
              </li>
            </ul>
            <CodeBlock
              code={`Use SweepLoom to list cleanup_candidates for this repo and explain anything unclear. Do not apply yet.\nUse SweepLoom to list_ai_stores and show always-on token tax. Do not park AGENTS.md or rules.\nUse SweepLoom to list_sessions. Do not terminate live agents.`}
              title="prompts that work"
            />
            <div className="table-wrap">
              <table>
                <caption className="sr-only">MCP tools</caption>
                <thead>
                  <tr>
                    <th scope="col">Tool</th>
                    <th scope="col">Arguments</th>
                    <th scope="col">Writes</th>
                  </tr>
                </thead>
                <tbody>
                  {MCP_TOOLS.map((tool) => (
                    <tr key={tool.name}>
                      <td>
                        <code>{tool.name}</code>
                      </td>
                      <td className="font-mono text-xs text-muted">{tool.args}</td>
                      <td className={tool.writes === "no" ? "text-muted" : "font-medium text-warn"}>{tool.writes}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </DocSection>

          <DocSection id="codex" title="Codex plugin">
            <p>
              The Codex plugin is the same MCP server plus three skills, <C>review-cleanup</C>, <C>token-tax</C> and{" "}
              <C>live-sessions</C>, that force review-before-apply. It is listed in the <C>weavatrix</C> marketplace.
            </p>
            <CodeBlock
              code={`$ codex plugin marketplace add Weavatrix/sweeploom\n$ codex plugin add sweeploom@weavatrix`}
              title="Codex CLI"
            />
            <p>
              Start a <strong>new</strong> Codex session afterwards; skills and MCP tools load with new sessions. The
              plugin&apos;s bundled MCP entry launches <C>npx -y sweeploom mcp</C>, which needs the npm package. Until that is
              published, register <C>sweeploom mcp</C> yourself in <C>~/.codex/config.toml</C> as shown above.
            </p>
          </DocSection>

          <DocSection id="library" title="Rust library">
            <p>
              The <C>sweeploom</C> crate is the same engine as the CLI and the MCP server. Until it is on crates.io, depend on
              it from Git:
            </p>
            <CodeBlock
              code={`[dependencies]\nsweeploom = { git = "https://github.com/Weavatrix/sweeploom" }`}
              title="Cargo.toml"
            />
            <CodeBlock
              code={`use sweeploom::{classify_name, estimated_prompt_tokens, AiClass};\n\nassert_eq!(classify_name("AGENTS.md").label(), "Context");\nassert_eq!(classify_name("history.jsonl"), AiClass::History);\nassert!(!classify_name(".credentials.json").can_clean());\nassert_eq!(\n    estimated_prompt_tokens(AiClass::History, "history.jsonl", 2_000_000),\n    0\n);`}
              title="src/main.rs"
            />
            <p>
              Listing sessions, inventories and apply (<C>ApplyRequest {"{ confirm, root, ids }"}</C>) are in{" "}
              <a href={docsUrl("LIBRARY.md")} className="link">
                docs/LIBRARY.md
              </a>
              . An <C>apply_cleanup</C> with <C>confirm: false</C> returns <C>ok == false</C> and deletes nothing.
            </p>
          </DocSection>

          <DocSection id="desktop" title="Desktop app">
            <p>Needs Rust 1.95 or newer. Either run it from a clone, or install it as a <C>sweeploom-gui</C> binary:</p>
            <CodeBlock
              code={`$ ${INSTALL.guiClone}\n$ cd sweeploom\n$ ${INSTALL.guiRun}`}
              title="build and run"
            />
            <CodeBlock code={`$ ${INSTALL.gui}`} title="or install the binary" />
            <p>
              On macOS, build the signed bundle so folder permissions survive updates. It needs a code-signing identity in
              your keychain; see <Link href="/download/#macos" className="link">Download → macOS</Link>.
            </p>
          </DocSection>

          <DocSection id="companion" title="Browser companion">
            <p>
              Optional. Without it, <C>sweeploom browser</C> still reports real RSS and CPU, and tab counts stay unknown,
              never zero. With it, the extension reports each tab&apos;s title, stripped URL and <C>lastAccessed</C> to a local
              SweepLoom process over native messaging. It makes no network requests and never closes tabs by default.
            </p>
            <CodeBlock
              code={`$ sweeploom companion-install\n$ sweeploom companion-install --chromium-id <id from edge://extensions>\n$ sweeploom browser`}
              title="register the native host"
            />
            <p>
              Load <C>browser/chromium-extension</C> (Chrome, Edge and other Chromium browsers) or{" "}
              <C>browser/firefox-extension</C> (Firefox 121+) from the repository as an unpacked extension, then run{" "}
              <C>companion-install</C>. Chromium browsers also need the extension id from <C>edge://extensions</C> or{" "}
              <C>chrome://extensions</C>. <C>sweeploom browser</C> shows <C>companion=connected</C> once a fresh snapshot
              arrives.
            </p>
          </DocSection>

          <DocSection id="reference" title="Reference docs">
            <p>Everything else, with full transcripts, lives in the repository.</p>
            <ul className="grid gap-3 sm:grid-cols-2">
              {REFERENCE.map((doc) => (
                <li key={doc.file}>
                  <a href={doc.root ? repoFile(doc.file) : docsUrl(doc.file)} className="card card-hover flex h-full items-start gap-3 p-4">
                    <icons.context size={18} className="mt-0.5 shrink-0 text-accent" />
                    <span className="min-w-0">
                      <span className="block font-medium text-text">{doc.title}</span>
                      <span className="mt-0.5 block text-sm">{doc.body}</span>
                      <span className="mt-1 block font-mono text-[0.7rem] text-faint">{doc.root ? doc.file : `docs/${doc.file}`}</span>
                    </span>
                  </a>
                </li>
              ))}
            </ul>
          </DocSection>
        </div>
      </div>
    </>
  );
}
