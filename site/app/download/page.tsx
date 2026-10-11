import Link from "next/link";
import { CodeBlock } from "@/components/CodeBlock";
import { icons } from "@/components/Icons";
import { PageHeader } from "@/components/Section";
import { pageMeta } from "@/lib/meta";
import { CODEX_TOML, INSTALL, MCP_JSON, SITE, docsUrl } from "@/lib/site";

export const metadata = pageMeta({
  title: "Download",
  description:
    "Install the SweepLoom CLI and MCP server from source with one Cargo command, build the desktop app for macOS, Windows or Linux, and connect your coding agent.",
  path: "/download/",
});

function Step({ n, title, children, id }: { n: string; title: string; children: React.ReactNode; id?: string }) {
  return (
    <section id={id} aria-labelledby={id ? `${id}-title` : undefined} className="card scroll-mt-24 p-6 sm:p-8">
      <div className="flex items-center gap-3">
        <span className="grid h-8 w-8 place-items-center rounded-full bg-gold font-mono text-sm font-semibold text-ink">{n}</span>
        <h2 id={id ? `${id}-title` : undefined} className="font-display text-xl font-semibold sm:text-2xl">
          {title}
        </h2>
      </div>
      <div className="mt-5 grid gap-5 leading-relaxed text-muted [&_strong]:text-text">{children}</div>
    </section>
  );
}

function C({ children }: { children: React.ReactNode }) {
  return <code className="inline-code">{children}</code>;
}

export default function DownloadPage() {
  return (
    <>
      <PageHeader
        eyebrow={`Download · v${SITE.version}`}
        title="Install SweepLoom from source."
        lead="One Cargo command builds the CLI and MCP server on Windows, macOS and Linux. The desktop app builds from a clone. Packages on crates.io and npm are coming soon."
      >
        <div className="flex flex-wrap gap-3">
          <a href="#cli" className="btn btn-primary">
            CLI + MCP
          </a>
          <a href="#desktop" className="btn btn-ghost">
            Desktop app
          </a>
          <a href="#agents" className="btn btn-ghost">
            Connect an agent
          </a>
        </div>
      </PageHeader>

      <div className="wrap grid gap-6 py-14 lg:grid-cols-[minmax(0,1fr)_300px] lg:items-start">
        <div className="grid min-w-0 gap-6">
          <Step n="0" title="Requirements" id="requirements">
            <ul className="grid gap-2">
              <li className="flex gap-2">
                <icons.check size={18} className="mt-0.5 shrink-0 text-accent" />
                <span>
                  <strong>Git</strong>, to fetch the source.
                </span>
              </li>
              <li className="flex gap-2">
                <icons.check size={18} className="mt-0.5 shrink-0 text-accent" />
                <span>
                  <strong>Rust via rustup</strong>: 1.88 or newer for the CLI and library, 1.95 or newer for the desktop
                  app. Get it at{" "}
                  <a href="https://rustup.rs" className="link">
                    rustup.rs
                  </a>
                  . Inside a clone, <code className="inline-code">rust-toolchain.toml</code> pins the toolchain and rustup
                  installs it on the first build.
                </span>
              </li>
              <li className="flex gap-2">
                <icons.check size={18} className="mt-0.5 shrink-0 text-accent" />
                <span>
                  <strong>No account, no key, no network at run time.</strong> SweepLoom is local-first.
                </span>
              </li>
            </ul>
          </Step>

          <Step n="1" title="CLI and MCP server" id="cli">
            <p>
              Builds and installs two binaries into <C>~/.cargo/bin</C> (<C>%USERPROFILE%\.cargo\bin</C> on Windows):{" "}
              <C>sweeploom</C>, the CLI, which serves MCP as <C>sweeploom mcp</C>, and <C>sweeploom-companion-host</C> for
              the optional browser companion. There is no separate <C>sweeploom-mcp</C> binary.
            </p>
            <CodeBlock code={`$ ${INSTALL.cli}`} title="install" wrap />
            <CodeBlock code={`$ sweeploom --help\n$ sweeploom mcp --list\n$ sweeploom clean .`} title="check it works" />
            <p>
              Update by running the same line with <C>--force</C>. Remove with <C>cargo uninstall sweeploom</C>. Working
              from a clone? Use <C>cargo install --path crates/sweeploom-cli --locked</C>.
            </p>
          </Step>

          <Step n="2" title="Desktop app" id="desktop">
            <p>
              The desktop app is a native Rust (egui) reviewer for Windows, macOS and Linux, with a tray icon on Windows and
              macOS. It needs Rust 1.95 or newer and is optional; the CLI and MCP server do not need it.
            </p>
            <CodeBlock code={`$ ${INSTALL.guiClone}\n$ cd sweeploom\n$ ${INSTALL.guiRun}`} title="build and run from a clone" />
            <CodeBlock code={`$ ${INSTALL.gui}`} title="or install it as a sweeploom-gui binary" wrap />
            <div id="macos" className="scroll-mt-24 rounded-xl border border-line bg-bg-2 p-5">
              <h3 className="font-display text-lg font-semibold text-text">macOS: build the signed app bundle</h3>
              <p className="mt-2">
                macOS ties folder permissions to an app&apos;s code signature. A <C>cargo run</C> binary gets a new ad-hoc
                identity on every rebuild, so macOS keeps asking for Documents, Downloads or Desktop access. The signed bundle
                keeps one identity across updates.
              </p>
              <CodeBlock className="mt-4" code={`$ ${INSTALL.macApp}\n$ ${INSTALL.macOpen}`} title="from the clone" />
              <ul className="mt-4 grid list-disc gap-1.5 pl-5 text-sm marker:text-gold">
                <li>
                  Needs a valid code-signing identity in your keychain. List them with{" "}
                  <C>security find-identity -v -p codesigning</C>; with more than one, set <C>SWEEPLOOM_SIGN_IDENTITY</C> or
                  pass <C>--identity</C>.
                </li>
                <li>Quit the running app before updating. The script refuses an incompatible update and never falls back to an ad-hoc signature.</li>
                <li>
                  <C>--no-build</C> packages an existing release binary; <C>SWEEPLOOM_RUST_TOOLCHAIN</C> picks another toolchain.
                </li>
              </ul>
              <a href={docsUrl("MACOS.md")} className="link mt-4 inline-flex items-center gap-1.5 text-sm">
                docs/MACOS.md <icons.external size={14} />
              </a>
            </div>
          </Step>

          <Step n="3" title="Connect your coding agent" id="agents">
            <p>
              Add SweepLoom as an MCP server in Cursor (<C>~/.cursor/mcp.json</C>), Claude Desktop, or any MCP host:
            </p>
            <CodeBlock code={MCP_JSON} title="mcp.json" />
            <CodeBlock code="$ claude mcp add -s user sweeploom -- sweeploom mcp" title="Claude Code" />
            <CodeBlock code={CODEX_TOML} title="Codex · ~/.codex/config.toml" />
            <p>
              Reload the host, then ask: <em>“Use SweepLoom to list cleanup_candidates for this repo. Do not apply yet.”</em>{" "}
              If a host started from the Dock or Start menu cannot find <C>sweeploom</C>, use the absolute path to the
              binary. The Codex plugin adds review skills on top; see{" "}
              <Link href="/docs/#codex" className="link">
                Docs → Codex plugin
              </Link>
              .
            </p>
          </Step>
        </div>

        <aside className="grid gap-4 lg:sticky lg:top-24">
          <div className="card p-5">
            <p className="eyebrow">At a glance</p>
            <dl className="mt-4 grid gap-3 text-sm">
              {[
                ["Version", SITE.version],
                ["License", "MPL-2.0"],
                ["Language", "Rust"],
                ["Platforms", "Windows · macOS · Linux"],
                ["Telemetry", "None"],
              ].map(([k, v]) => (
                <div key={k} className="flex justify-between gap-4 border-b border-line pb-2 last:border-0 last:pb-0">
                  <dt className="text-muted">{k}</dt>
                  <dd className="text-right font-medium">{v}</dd>
                </div>
              ))}
            </dl>
          </div>
          <div className="card p-5 text-sm leading-relaxed text-muted">
            <p className="font-medium text-text">Read before you run</p>
            <p className="mt-2">
              Building from source means you can read every line that touches your disk. The destructive path is{" "}
              <code className="inline-code">plan → revalidate → execute → receipt</code>.
            </p>
            <a href={SITE.repo} className="link mt-3 inline-flex items-center gap-1.5">
              Browse the source <icons.external size={14} />
            </a>
          </div>
          <div className="card p-5 text-sm leading-relaxed text-muted">
            <p className="font-medium text-text">Something broke?</p>
            <p className="mt-2">Open an issue with your OS, Rust version and the command you ran.</p>
            <a href={SITE.newIssue} className="link mt-3 inline-flex items-center gap-1.5">
              New issue <icons.external size={14} />
            </a>
          </div>
        </aside>
      </div>
    </>
  );
}
