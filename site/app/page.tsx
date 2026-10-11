import Link from "next/link";
import type { Metadata } from "next";
import { CodeBlock } from "@/components/CodeBlock";
import { HeroVisual } from "@/components/HeroVisual";
import { icons } from "@/components/Icons";
import { Screenshot } from "@/components/Screenshot";
import { Faq, Section } from "@/components/Section";
import { Tabs } from "@/components/Tabs";
import {
  CLEAN_TRANSCRIPT,
  CLI_COMMANDS,
  FINDINGS,
  GUI_SCREENS,
  HARD_LINES,
  MCP_TOOLS,
} from "@/lib/content";
import { INSTALL, MCP_JSON, SITE } from "@/lib/site";
import { getAllPosts, formatDate } from "@/lib/blog";

export const metadata: Metadata = {
  alternates: { canonical: "/" },
};

const STATS = [
  {
    value: "16,288",
    unit: "tokens",
    label: "always-on estimate on the gold store, where billing History as prompt gives 965,000",
  },
  { value: "6 / 6", unit: "kept", label: "idle 2 GB agents stay Keep; a naive “free RAM” agent would kill all six" },
  { value: "0", unit: "deleted", label: "files removed when apply is called with confirm=false" },
  { value: "30 / 30", unit: "classified", label: "gold names, about 2 µs per call, no LLM involved" },
];

const STEPS = [
  {
    n: "01",
    title: "Plan",
    body: "SweepLoom lists what it found. SAFE generated rows are pre-selected; inspect-only rows are shown as BLOCKED and cannot be applied.",
  },
  {
    n: "02",
    title: "Revalidate",
    body: "Right before anything is removed, each row is checked again. If you rebuilt target/ after the plan, that row is skipped, not forced.",
  },
  {
    n: "03",
    title: "Execute",
    body: "Only what you agreed to: --apply on the CLI, confirm=true with explicit ids over MCP, a confirmation in the desktop app.",
  },
  {
    n: "04",
    title: "Receipt",
    body: "Every apply returns what happened: deleted, skipped because it changed, failed. Nothing is silent.",
  },
];

const FAQ = [
  {
    q: "Will SweepLoom kill my idle Cursor, Claude Code or Codex session?",
    a: (
      <p>
        No. Idle Claude Code, Codex, Cursor, OpenCode, Gemini and Grok sessions are <strong>Keep</strong>, and so are MCP
        servers attached to them. Killing a live agent drops the chat you were trying to make cheaper. The CLI and MCP
        have no terminate tool at all; the desktop app only stops a session when a person explicitly confirms it.
      </p>
    ),
  },
  {
    q: "Can an AI agent delete files through the MCP server?",
    a: (
      <p>
        Only through <code>apply_cleanup</code> with <code>confirm: true</code>, after review tools have listed the rows,
        and only for SAFE generated rows. Without <code>confirm=true</code> the call is refused and deletes nothing. The
        bundled Codex skills tell the agent to list, explain, ask, and pass exactly the ids you agreed to.
      </p>
    ),
  },
  {
    q: "Does SweepLoom send anything over the network?",
    a: (
      <p>
        No telemetry, no account, no update checks. The CLI and MCP server make no network requests. The desktop app&apos;s
        only network use is a local Ollama server on <code>127.0.0.1:11434</code>, to list and remove models, and the
        browser companion talks only to a local process. See the <Link href="/privacy/">privacy policy</Link>.
      </p>
    ),
  },
  {
    q: "Is the 16,288 token figure a promise that my agent gets cheaper?",
    a: (
      <p>
        No. It is a fixed policy fixture: a whole-store dump versus SweepLoom&apos;s capped always-on estimate. It is not
        provider billing. A live coding-agent run on 2026-09-15 found <strong>no</strong> coding-task token saving, and we
        publish that result next to the fixtures.
      </p>
    ),
  },
  {
    q: "Which platforms are supported?",
    a: (
      <p>
        The CLI, MCP server and library are tested on Windows, macOS and Linux in CI. The desktop app runs on all three;
        the tray icon is available on Windows and macOS. Xcode and iOS Simulator cleanup is macOS-only, naturally.
      </p>
    ),
  },
  {
    q: "Why is it not on crates.io or npm yet?",
    a: (
      <p>
        Packages are coming soon. Today you install from source with Cargo in one command; see{" "}
        <Link href="/download/">Download</Link>.
      </p>
    ),
  },
  {
    q: "What does it cost?",
    a: (
      <p>
        Nothing. SweepLoom is open source under the Mozilla Public License 2.0. See <Link href="/license/">License</Link>.
      </p>
    ),
  },
];

function SurfaceChips({ surfaces }: { surfaces: string[] }) {
  return (
    <div className="flex flex-wrap gap-1.5">
      {surfaces.map((s) => (
        <span key={s} className={"chip " + (s === "App" ? "" : "chip-gold")}>
          {s}
        </span>
      ))}
    </div>
  );
}

export default function HomePage() {
  const posts = getAllPosts().slice(0, 3);

  const jsonLd = {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: "SweepLoom",
    applicationCategory: "DeveloperApplication",
    operatingSystem: "Windows, macOS, Linux",
    description: SITE.description,
    url: SITE.url,
    softwareVersion: SITE.version,
    license: "https://www.mozilla.org/en-US/MPL/2.0/",
    image: `${SITE.url}/mark-512.png`,
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    author: { "@type": "Person", name: SITE.author },
    publisher: { "@type": "Organization", name: SITE.company, url: SITE.companyUrl },
    codeRepository: SITE.repo,
  };

  return (
    <>
      <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }} />

      {/* ---------- hero ---------- */}
      <section aria-labelledby="hero-title" className="relative overflow-hidden">
        <div className="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
          <div className="weave-bg drift absolute -inset-[44px] opacity-80" />
        </div>
        <div className="wrap relative grid items-center gap-14 pt-14 pb-20 sm:pt-20 lg:grid-cols-[1.05fr_1fr] lg:gap-10 lg:pt-24 lg:pb-28">
          <div>
            <p className="rise eyebrow flex flex-wrap items-center gap-x-3 gap-y-1">
              <span>A Weavatrix product</span>
              <span className="text-faint" aria-hidden="true">/</span>
              <span>v{SITE.version}</span>
              <span className="text-faint" aria-hidden="true">/</span>
              <span>MPL-2.0</span>
            </p>
            <h1
              id="hero-title"
              className="rise mt-5 font-display text-[2.6rem] leading-[1.02] font-semibold tracking-[-0.035em] text-balance sm:text-6xl lg:text-[4.1rem]"
              style={{ animationDelay: "60ms" }}
            >
              Reclaim your workstation <span className="text-accent">without losing your workspace.</span>
            </h1>
            <p
              className="rise mt-6 max-w-xl text-lg leading-relaxed text-pretty text-muted"
              style={{ animationDelay: "120ms" }}
            >
              SweepLoom is a developer-aware resource manager. It finds build output, package caches, Xcode and Docker
              leftovers, forgotten helpers and the always-on context your agents carry, then reclaims space through a plan
              you review. It never kills the chat you are in.
            </p>
            <div className="rise mt-8 flex flex-wrap gap-3" style={{ animationDelay: "180ms" }}>
              <Link href="/download/" className="btn btn-primary">
                Install from source
                <icons.arrow size={16} />
              </Link>
              <Link href="/docs/" className="btn btn-ghost">
                Read the docs
              </Link>
            </div>
            <div className="rise mt-8 max-w-xl" style={{ animationDelay: "240ms" }}>
              <CodeBlock code={`$ ${INSTALL.cli}`} title="CLI + MCP server" wrap />
            </div>
            <ul
              className="rise mt-6 flex flex-wrap gap-x-5 gap-y-2 text-sm text-muted"
              style={{ animationDelay: "300ms" }}
            >
              {["Local-first", "No telemetry", "No LLM on classify or apply", "Rust"].map((item) => (
                <li key={item} className="inline-flex items-center gap-2">
                  <icons.check size={15} className="text-accent" />
                  {item}
                </li>
              ))}
            </ul>
          </div>
          <HeroVisual />
        </div>
      </section>

      {/* ---------- proof ---------- */}
      <section aria-labelledby="proof-title" className="border-y border-line bg-bg-2">
        <div className="wrap py-12">
          <h2 id="proof-title" className="sr-only">
            Numbers you can re-run
          </h2>
          <dl className="grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
            {STATS.map((stat) => (
              <div key={stat.label} className="min-w-0">
                <dt className="sr-only">{stat.unit}</dt>
                <dd>
                  <span className="font-display text-4xl font-semibold tracking-tight text-text">{stat.value}</span>
                  <span className="ml-2 font-mono text-xs tracking-widest text-accent uppercase">{stat.unit}</span>
                  <p className="mt-2 text-sm leading-relaxed text-muted">{stat.label}</p>
                </dd>
              </div>
            ))}
          </dl>
          <p className="mt-8 text-xs leading-relaxed text-faint">
            Fixed fixtures asserted in tests, captured 2026-09-14. Re-run with <code className="inline-code">sweeploom bench</code>.
            They measure classification policy, not provider-billed tokens.{" "}
            <Link href="/blog/always-on-prompt-tax/" className="link">
              Why that distinction matters
            </Link>
            .
          </p>
        </div>
      </section>

      {/* ---------- what it finds ---------- */}
      <Section
        id="finds"
        eyebrow="What it finds"
        title="Everything a developer machine accumulates, sorted by whether it still earns its keep."
        lead="Three questions, answered locally: what is sitting on disk, in RAM and in the agent host; is it still earning its keep; and how do you reclaim it without losing the workspace."
      >
        <ul className="grid gap-4 sm:grid-cols-2 lg:grid-cols-6 lg:[&>li]:col-span-2 lg:[&>li:nth-child(n+7)]:col-span-3">
          {FINDINGS.map((item) => {
            const Icon = icons[item.icon];
            return (
              <li key={item.id} className="card card-hover flex flex-col p-6">
                <div className="flex items-start justify-between gap-3">
                  <span className="grid h-10 w-10 place-items-center rounded-xl border border-line bg-bg-2 text-accent">
                    <Icon size={20} />
                  </span>
                  <SurfaceChips surfaces={item.surfaces} />
                </div>
                <h3 className="mt-5 font-display text-lg font-semibold">{item.title}</h3>
                <p className="mt-2 text-[0.95rem] leading-relaxed text-muted">{item.body}</p>
                <p className="mt-4 font-mono text-[0.72rem] leading-relaxed text-faint">{item.examples.join(" · ")}</p>
              </li>
            );
          })}
        </ul>
        <div className="mt-8">
          <Link href="/features/" className="link inline-flex items-center gap-1.5 text-sm font-medium">
            All features in detail <icons.arrow size={14} />
          </Link>
        </div>
      </Section>

      {/* ---------- gated apply ---------- */}
      <Section
        id="gated"
        eyebrow="How apply is gated"
        title="Nothing is deleted without plan, revalidate, execute, receipt."
        lead="Review is the product. The same four steps run whether a person types --apply, an agent calls apply_cleanup, or you confirm in the desktop app."
        className="border-t border-line"
      >
        <ol className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {STEPS.map((step) => (
            <li key={step.n} className="card p-6">
              <span className="font-mono text-xs text-accent">{step.n}</span>
              <h3 className="mt-3 font-display text-xl font-semibold">{step.title}</h3>
              <p className="mt-2 text-[0.95rem] leading-relaxed text-muted">{step.body}</p>
            </li>
          ))}
        </ol>
        <div className="mt-10 grid gap-8 lg:grid-cols-[1.1fr_1fr] lg:items-start">
          <CodeBlock code={CLEAN_TRANSCRIPT} title="a workspace that has built" />
          <div>
            <h3 className="font-display text-lg font-semibold">Lines SweepLoom does not cross</h3>
            <ul className="mt-4 grid gap-3">
              {HARD_LINES.map((line) => (
                <li key={line} className="flex gap-3 text-[0.95rem] leading-relaxed text-muted">
                  <icons.shield size={18} className="mt-0.5 shrink-0 text-accent" />
                  <span>{line}</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </Section>

      {/* ---------- interfaces ---------- */}
      <Section
        id="interfaces"
        eyebrow="Three ways in"
        title="A CLI for hands, MCP for agents, a desktop app for review."
        lead="One engine behind all three. On the CLI, --apply is consent. Over MCP, apply needs confirm=true. In the app, a person confirms."
        className="border-t border-line"
      >
        <Tabs
          label="SweepLoom interfaces"
          items={[
            {
              id: "cli",
              label: "CLI",
              hint: "sweeploom",
              content: (
                <div className="grid gap-6 lg:grid-cols-[1.3fr_1fr]">
                  <CodeBlock code={CLI_COMMANDS} title="one-liners you will actually type" />
                  <div className="text-[0.97rem] leading-relaxed text-muted">
                    <p>
                      No path means your home directory; pass <code className="inline-code">.</code> for the current tree.
                      Session plans such as <code className="inline-code">--quiet</code>,{" "}
                      <code className="inline-code">--free-ram 4</code> and <code className="inline-code">--reduce-cpu 20</code>{" "}
                      are dry runs: terminate is not offered on the CLI.
                    </p>
                    <p className="mt-4">
                      <code className="inline-code">sweeploom clean .</code> printing{" "}
                      <code className="inline-code">no generated candidates</code> is a valid answer. SweepLoom does not
                      invent junk.
                    </p>
                    <Link href="/docs/#cli" className="link mt-5 inline-flex items-center gap-1.5 text-sm font-medium">
                      CLI reference <icons.arrow size={14} />
                    </Link>
                  </div>
                </div>
              ),
            },
            {
              id: "mcp",
              label: "MCP",
              hint: "sweeploom mcp",
              content: (
                <div className="grid gap-6 lg:grid-cols-2">
                  <div className="grid gap-4">
                    <CodeBlock code={MCP_JSON} title="~/.cursor/mcp.json · claude_desktop_config.json" />
                    <CodeBlock code="$ claude mcp add -s user sweeploom -- sweeploom mcp" title="Claude Code" />
                  </div>
                  <div className="table-wrap">
                    <table>
                      <caption className="sr-only">SweepLoom MCP tools</caption>
                      <thead>
                        <tr>
                          <th scope="col">Tool</th>
                          <th scope="col">Writes</th>
                        </tr>
                      </thead>
                      <tbody>
                        {MCP_TOOLS.map((tool) => (
                          <tr key={tool.name}>
                            <td>
                              <code>{tool.name}</code>
                              <div className="mt-1 text-xs text-muted">{tool.does}</div>
                            </td>
                            <td className={tool.writes === "no" ? "text-muted" : "font-medium text-warn"}>{tool.writes}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              ),
            },
            {
              id: "app",
              label: "Desktop app",
              hint: "egui",
              content: (
                <div className="grid gap-6 lg:grid-cols-[1fr_1.2fr]">
                  <div className="grid content-start gap-4">
                    <CodeBlock
                      code={`$ ${INSTALL.guiClone}\n$ cd sweeploom\n$ ${INSTALL.guiRun}`}
                      title="run the desktop app"
                    />
                    <p className="text-[0.97rem] leading-relaxed text-muted">
                      A native Rust app (egui) for Windows, macOS and Linux that follows your light, dark and scale
                      settings, keeps a quiet tray icon on Windows and macOS, and continues scanning while hidden. On macOS,
                      build the <Link href="/download/#macos" className="link">signed app bundle</Link> so folder
                      permissions survive updates.
                    </p>
                  </div>
                  <div className="grid gap-4 sm:grid-cols-3">
                    {GUI_SCREENS.map((group) => (
                      <div key={group.section} className="card p-4">
                        <p className="eyebrow !text-faint">{group.section}</p>
                        <ul className="mt-3 grid gap-3">
                          {group.items.map((item) => (
                            <li key={item.name}>
                              <span className="text-sm font-medium">{item.name}</span>
                              <span className="mt-0.5 block text-xs leading-snug text-muted">{item.does}</span>
                            </li>
                          ))}
                        </ul>
                      </div>
                    ))}
                  </div>
                </div>
              ),
            },
          ]}
        />
      </Section>

      {/* ---------- screenshots ---------- */}
      <Section
        id="screenshots"
        eyebrow="The desktop app"
        title="A local reviewer, not a dashboard you have to trust."
        lead="Every row shows its origin path or native object ID, size, change since the last scan, and the consequence of removing it."
        className="border-t border-line"
      >
        <div className="grid gap-8 md:grid-cols-2">
          <Screenshot
            file="overview.png"
            screen="Overview"
            alt="SweepLoom Overview screen with machine pressure cards"
            caption="Machine pressure at a glance; every card opens the screen that explains it."
          />
          <Screenshot
            file="sessions.png"
            screen="Sessions"
            alt="SweepLoom Sessions screen with agent session trees"
            caption="Agents grouped with their MCP servers, terminals and dev servers, each with a Keep or Optional verdict."
          />
          <Screenshot
            file="cleanup.png"
            screen="Cleanup"
            alt="SweepLoom Cleanup screen listing Docker, simulator and cache items"
            caption="Docker, iOS Simulator, package caches and AI models, itemised with the consequence of removal."
          />
          <Screenshot
            file="ai.png"
            screen="AI"
            alt="SweepLoom AI screen showing AI stores and token estimates"
            caption="AI stores split into Context, History, Cache and Secret, with the always-on token estimate."
          />
        </div>
      </Section>

      {/* ---------- what it is not ---------- */}
      <Section
        id="not"
        eyebrow="Honest scope"
        title="What SweepLoom is not."
        className="border-t border-line"
      >
        <div className="grid gap-4 md:grid-cols-2">
          {[
            {
              t: "Not a RAM booster",
              b: "It will not “kill idle Cursor to free 8 GB”. RSS is reported as a sum, not as reclaimable memory, and live agents stay Keep.",
            },
            {
              t: "Not a context compressor",
              b: "It estimates the always-on prompt tax of rules, skills and plugins. It does not intercept model requests or shrink what an agent reads. For budgeted repository evidence, see the separate Cortex Loom project.",
            },
            {
              t: "Not an AI that decides what to delete",
              b: "Classification is deterministic Rust, in microseconds. An LLM would add seconds and thousands of tokens to the one path that must be predictable.",
            },
            {
              t: "Not a cloud service",
              b: "No account, no sync, no telemetry. The desktop app is a local reviewer; agents use MCP over stdio.",
            },
          ].map((item) => (
            <div key={item.t} className="card flex gap-4 p-6">
              <icons.x size={20} className="mt-0.5 shrink-0 text-warn" />
              <div>
                <h3 className="font-display text-lg font-semibold">{item.t}</h3>
                <p className="mt-1.5 text-[0.95rem] leading-relaxed text-muted">{item.b}</p>
              </div>
            </div>
          ))}
        </div>
      </Section>

      {/* ---------- blog ---------- */}
      {posts.length ? (
        <Section id="writing" eyebrow="From the blog" title="Notes on reclaiming space safely." className="border-t border-line">
          <ul className="grid gap-4 md:grid-cols-3">
            {posts.map((post) => (
              <li key={post.slug}>
                <Link href={`/blog/${post.slug}/`} className="card card-hover flex h-full flex-col p-6">
                  <time dateTime={post.date} className="font-mono text-xs text-faint">
                    {formatDate(post.date)}
                  </time>
                  <h3 className="mt-3 font-display text-lg leading-snug font-semibold text-balance">{post.title}</h3>
                  <p className="mt-2 line-clamp-3 text-sm leading-relaxed text-muted">{post.summary}</p>
                  <span className="mt-auto pt-5 text-sm font-medium text-accent">Read →</span>
                </Link>
              </li>
            ))}
          </ul>
        </Section>
      ) : null}

      {/* ---------- faq ---------- */}
      <Section id="faq" eyebrow="FAQ" title="Questions people ask first." className="border-t border-line">
        <Faq items={FAQ} />
      </Section>

      {/* ---------- final cta ---------- */}
      <section aria-labelledby="cta-title" className="wrap">
        <div className="card relative overflow-hidden p-8 sm:p-12">
          <div className="gold-glow pointer-events-none absolute -top-32 -right-24 h-80 w-[36rem]" aria-hidden="true" />
          <div className="relative grid gap-8 lg:grid-cols-[1.2fr_1fr] lg:items-center">
            <div>
              <h2 id="cta-title" className="font-display text-3xl font-semibold tracking-tight text-balance sm:text-4xl">
                Look first. Then apply.
              </h2>
              <p className="mt-3 max-w-lg text-muted">
                Install the CLI and MCP server from source, point your agent host at <code className="inline-code">sweeploom mcp</code>,
                and ask it what is safe to remove.
              </p>
              <div className="mt-6 flex flex-wrap gap-3">
                <Link href="/download/" className="btn btn-primary">
                  Get SweepLoom
                </Link>
                <a href={SITE.repo} className="btn btn-ghost">
                  Star on GitHub
                </a>
              </div>
            </div>
            <CodeBlock code={`$ sweeploom clean .\n$ sweeploom clean . --apply`} title="the whole idea" />
          </div>
        </div>
      </section>
    </>
  );
}
