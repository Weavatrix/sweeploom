import Link from "next/link";
import { Chat, type ChatTurn } from "@/components/Chat";
import { CodeBlock } from "@/components/CodeBlock";
import { icons } from "@/components/Icons";
import { Faq, PageHeader, Section } from "@/components/Section";
import { Tabs } from "@/components/Tabs";
import { MCP_TOOLS } from "@/lib/content";
import { pageMeta } from "@/lib/meta";
import { CLAUDE_CODE_ADD, CODEX_PLUGIN, CODEX_TOML, INSTALL, MCP_JSON, MCP_JSON_WINDOWS, SITE, docsUrl } from "@/lib/site";

export const metadata = pageMeta({
  title: "MCP for coding agents",
  description:
    "SweepLoom's MCP server gives Claude Code, Cursor, Claude Desktop and Codex eight read-only tools and one gated apply. Per-host setup, the full tool reference, the confirm=true gate, example chats, the Codex plugin and troubleshooting.",
  path: "/mcp/",
});

const JUMP = [
  { id: "why", label: "What it is for" },
  { id: "install", label: "Install" },
  { id: "hosts", label: "Connect a host" },
  { id: "tools", label: "Tool reference" },
  { id: "gate", label: "The apply gate" },
  { id: "limits", label: "Can and cannot" },
  { id: "chats", label: "Example chats" },
  { id: "codex", label: "Codex plugin" },
  { id: "troubleshooting", label: "Troubleshooting" },
];

const MCP_LIST = `$ sweeploom mcp --list
sweeploom  SweepLoom
io.github.Weavatrix/sweeploom  mcp_tools=9
  list_sessions          read         cli=sessions mcp
  disk_inventory         read         cli=scan mcp
  list_projects          read         cli=projects mcp
  list_ai_stores         read         cli=ai mcp
  bench_compare          read         cli=bench
  advise_context         read         mcp
  cleanup_candidates     read         cli=clean mcp
  explain_candidate      read         mcp
  list_browser           read         cli=browser mcp
  apply_cleanup          mcp-confirm  mcp
  clean_apply            cli-write    cli=clean
  companion_install      cli-write    cli=companion-install
  companion_host         read         cli=companion-host`;

const GATE_CALLS = `tools/call  cleanup_candidates   { "root": "C:/work/app" }
tools/call  explain_candidate    { "id": 3, "root": "C:/work/app" }
# the agent shows you row 3 and asks
tools/call  apply_cleanup        { "confirm": true, "root": "C:/work/app", "ids": [3] }`;

const RECEIPT = `{
  "ok": true,
  "error": null,
  "deleted": 1,
  "skipped_changed": 0,
  "failed": 0,
  "receipt": 1
}`;

const CHAT_A: ChatTurn[] = [
  { role: "user", text: "This workspace is huge. What can SweepLoom delete safely? Show me the rows. Do not delete yet." },
  { role: "call", text: `cleanup_candidates   { "root": "C:/work/app" }\nexplain_candidate    { "id": 3, "root": "C:/work/app" }` },
  {
    role: "tool",
    text: `[
  { "id": 3, "title": "cargo target", "logical_bytes": 1288490188,
    "selected": true, "blocked": false, "deletion": "PermanentGenerated" },
  { "id": 4, "title": "node_modules", "logical_bytes": 851443712,
    "selected": true, "blocked": false, "deletion": "PermanentGenerated" }
]`,
  },
  {
    role: "agent",
    text: "Two SAFE generated rows:\n• id 3  cargo target  ~1.2 GB  rebuild = cargo build\n• id 4  node_modules  ~812 MB  rebuild = npm ci\nNothing else is pre-selected. I have not deleted anything. Want id 3 only, both, or stop?",
  },
  { role: "user", text: "Only cargo target." },
  { role: "call", text: `apply_cleanup   { "confirm": true, "root": "C:/work/app", "ids": [3] }` },
  { role: "tool", text: `{ "ok": true, "deleted": 1, "skipped_changed": 0, "failed": 0, "receipt": 1 }` },
  { role: "agent", text: "Deleted id 3 (target). Receipt 1. node_modules left alone." },
];

const CHAT_B: ChatTurn[] = [
  { role: "user", text: "Just wipe the build cache." },
  { role: "call", text: `apply_cleanup   { "confirm": false, "root": "C:/work/app" }` },
  { role: "tool", label: "SweepLoom returns an MCP error", text: "refused: MCP apply needs confirm=true after cleanup_candidates; CLI uses --apply" },
  { role: "agent", text: "Nothing was deleted. Let me list the candidates first and show you what would go." },
];

const CHAT_C: ChatTurn[] = [
  { role: "user", text: "Codex feels expensive. Is it history on disk or always-on context? Do not delete my chats. Do not edit AGENTS.md." },
  {
    role: "call",
    text: `list_ai_stores\nadvise_context   { "relative": "AGENTS.md" }\nadvise_context   { "relative": "history.jsonl" }`,
  },
  {
    role: "tool",
    text: `{ "relative": "AGENTS.md", "class": "Context", "advice": "keep" }\n{ "relative": "history.jsonl", "class": "History", "advice": "" }`,
  },
  {
    role: "agent",
    text: "~/.claude/projects is 862 MB of History: archive, 0 always-on tokens. I will not delete it.\nplugins is ~4096 always-on tokens (capped), keep. skills is park? (idle 30+ days); SweepLoom will not flip alwaysApply.\nAGENTS.md and rules stay keep. .credentials.json is Secret and never opened.",
  },
];

const CHAT_D: ChatTurn[] = [
  { role: "user", text: "Cursor is idle and 7 GB. Kill it." },
  { role: "call", text: "list_sessions" },
  {
    role: "tool",
    text: `[
  { "kind": "Cursor", "processes": 43, "rss_bytes": 7838315315,
    "recommendation": "Keep", "activity": "Idle", "planned": false },
  { "kind": "Codex", "processes": 3, "recommendation": "Keep" },
  { "kind": "Claude Code", "processes": 2, "recommendation": "Keep" }
]`,
  },
  {
    role: "agent",
    text: "SweepLoom has no terminate tool. Cursor, Codex and Claude are Keep so this chat stays. I will not shell out to taskkill.\nIf you want a dry-run plan of Optional helpers only: sweeploom sessions --quiet",
  },
];

const CHAT_E: ChatTurn[] = [
  { role: "user", text: "What is under C:/work/app besides source?" },
  { role: "call", text: `disk_inventory   { "root": "C:/work/app" }\nlist_projects    { "root": "C:/work/app" }\nlist_browser` },
  {
    role: "tool",
    text: `{ "root": "C:\\\\work\\\\app", "entries": 18420, "capped": false,
  "children": [ { "path": "C:\\\\work\\\\app\\\\target", "logical_bytes": 1288490188,
                  "category": "Generated" } ] }
[ { "path": "C:\\\\work\\\\app", "kind": "Cargo", "source": "Warm",
    "artifact": "Hot", "git": "Clean" } ]
{ "companion": "disconnected", "hosts": 1, "rss_bytes": 8804682956, "tabs": null }`,
  },
  {
    role: "agent",
    text: "1.2 GB of target/ is generated output in a clean Cargo project. Browser tabs are unknown (tabs: null), not zero. Want me to list cleanup candidates?",
  },
];

const SKILLS = [
  {
    name: "review-cleanup",
    does: "Review generated cleanup rows, then apply only after the user agrees. Never delete first.",
    loop: "cleanup_candidates → explain_candidate → show the plan → apply_cleanup with confirm: true and the agreed ids",
  },
  {
    name: "token-tax",
    does: "Inspect always-on AI context. History is archive. Do not disable AGENTS.md or rules.",
    loop: "list_ai_stores → advise_context → report prompt_tokens and advice; never invent counts",
  },
  {
    name: "live-sessions",
    does: "List live coding sessions. Keep live agents. Only an MCP server with no parent agent may be Optional.",
    loop: "list_sessions → explain Keep / Optional; dry-run plans are a CLI suggestion, never a kill",
  },
];

const TROUBLE = [
  {
    q: "The server is connected, but the agent ignores it and uses the shell",
    a: (
      <>
        <p>
          A connected server is a catalog the model may or may not consult. If you never say “SweepLoom”, it can decide that{" "}
          <code>du</code> is good enough. Name the product, or a skill, when you want its tools:
        </p>
        <CodeBlock
          className="mt-4"
          copy={false}
          code={`# connected, not named: the model may skip the catalog\nCodex feels expensive. Do not delete my chats.\n\n# connected and named: the model should call list_ai_stores\nUse SweepLoom to list_ai_stores. Do not park AGENTS.md.`}
          title="silent vs told"
        />
      </>
    ),
  },
  {
    q: "The server does not start from Claude Desktop or Cursor",
    a: (
      <p>
        Hosts launched from the Dock, the Start menu or a launcher may not inherit your shell <code>PATH</code>, so the bare
        command <code>sweeploom</code> is not found. Use the absolute path of the installed binary:{" "}
        <code>/Users/you/.cargo/bin/sweeploom</code> on macOS, <code>/home/you/.cargo/bin/sweeploom</code> on Linux,{" "}
        <code>C:\Users\you\.cargo\bin\sweeploom.exe</code> on Windows (see the Absolute path tab above). Then reload the host.
      </p>
    ),
  },
  {
    q: "Searching the Cursor Marketplace for SweepLoom finds nothing",
    a: (
      <p>
        That is expected. SweepLoom is an MCP server, not a Marketplace plugin. Add the <code>mcpServers</code> entry and toggle
        it under Cursor → Settings → MCP.
      </p>
    ),
  },
  {
    q: "npx -y sweeploom mcp fails, or the Codex plugin's tools never appear",
    a: (
      <p>
        SweepLoom is not on npm or crates.io yet, so <code>npx -y sweeploom mcp</code>, which the Codex plugin&apos;s bundled
        entry uses, cannot start today. Install from source and register <code>sweeploom mcp</code> in{" "}
        <code>~/.codex/config.toml</code> yourself. Then start a <strong>new</strong> Codex session: skills and MCP tools load
        with new sessions, not mid-chat.
      </p>
    ),
  },
  {
    q: "The agent scanned my whole home directory",
    a: (
      <p>
        An empty or missing <code>root</code> on <code>disk_inventory</code>, <code>list_projects</code>,{" "}
        <code>cleanup_candidates</code>, <code>explain_candidate</code> and <code>apply_cleanup</code> means your home
        directory. Ask the agent to pass the absolute path of the workspace, and to use the same <code>root</code> for the
        review and the apply.
      </p>
    ),
  },
  {
    q: "apply_cleanup answered “refused” or “nothing eligible to apply”",
    a: (
      <p>
        <code>refused: MCP apply needs confirm=true…</code> is the gate working: the agent tried to apply without your
        agreement. <code>nothing eligible to apply</code> means no SAFE, unblocked row matched the ids (or, with empty ids,
        nothing was pre-selected). A clean tree is a valid answer; SweepLoom does not invent junk.
      </p>
    ),
  },
  {
    q: "Running sweeploom mcp in a terminal just sits there",
    a: (
      <p>
        It is serving MCP over stdio and waiting for a host. Leave that to Claude Code, Cursor, Claude Desktop or Codex. To see
        the contract without serving, run <code>sweeploom mcp --list</code>. Bare <code>sweeploom</code> prints help; it never
        waits on stdin.
      </p>
    ),
  },
  {
    q: "I wired the host to cargo run and it keeps disconnecting",
    a: (
      <p>
        <code>cargo run … -- mcp</code> recompiles when the host restarts, writes build diagnostics into the MCP process, and
        can fail with an OS disk-full error. It is a development command. Install the binary once (
        <code>cargo install --path crates/sweeploom-cli --locked</code> from a clone) and point the host at{" "}
        <code>sweeploom mcp</code>.
      </p>
    ),
  },
];

const DOCS = [
  { file: "MCP.md", title: "MCP for agents", body: "Setup per host, silent vs told, every tool and its arguments." },
  { file: "MCP-CHATS.md", title: "Chats A–B", body: "Review, then apply; the refused shortcut; unknown ids." },
  { file: "MCP-CHATS-AGENTS.md", title: "Chats C–E", body: "Token tax, the idle Cursor, mapping a folder." },
  { file: "PLUGIN.md", title: "Codex plugin", body: "Install, skills, layout, what it will not do." },
  { file: "SAFETY.md", title: "Safety model", body: "Classes, the apply path and the MCP gate test." },
  { file: "INSTALL.md", title: "Install", body: "From source, from a clone, every binary." },
];

function C({ children }: { children: React.ReactNode }) {
  return <code className="inline-code">{children}</code>;
}

function HostPanel({ children }: { children: React.ReactNode }) {
  return <div className="grid gap-5 leading-relaxed text-muted lg:grid-cols-2 lg:items-start [&_strong]:text-text">{children}</div>;
}

export default function McpPage() {
  return (
    <>
      <PageHeader
        eyebrow="MCP · for coding agents"
        title="A cleanup tool your agent can use, and cannot misuse."
        lead="SweepLoom is an MCP server over stdio. Your agent gets eight read-only tools to see what is on disk, in RAM and in the agent host, and one tool that deletes only reviewed SAFE generated rows, only with confirm=true. There is no terminate tool."
      >
        <div className="flex flex-wrap gap-3">
          <a href="#hosts" className="btn btn-primary">
            Connect a host <icons.arrow size={16} />
          </a>
          <a href="#tools" className="btn btn-ghost">
            Tool reference
          </a>
          <a href={docsUrl("MCP.md")} className="btn btn-ghost">
            docs/MCP.md <icons.external size={15} />
          </a>
        </div>
        <nav aria-label="On this page" className="mt-8">
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
        id="why"
        eyebrow="What it is for"
        title="Ground truth for the agent, a gate for you."
        lead="Ask an agent without SweepLoom what is safe to delete and it improvises: du on a guess, folder names as evidence, ~/.claude divided by four, or the idle editor killed to free RAM. With SweepLoom it reads classified rows with a safety verdict, and the one destructive call needs your yes."
      >
        <ul className="grid gap-4 md:grid-cols-3">
          {[
            {
              icon: icons.projects,
              t: "Evidence instead of guesses",
              b: "Sessions grouped into trees, folders with categories, projects with source and artifact heat and Git state, AI stores with classes and capped always-on token estimates.",
            },
            {
              icon: icons.shield,
              t: "Read-only by default",
              b: "Eight tools only read and publish readOnlyHint, so hosts can treat them as safe. apply_cleanup refuses unless confirm is true, revalidates every row, and returns a receipt.",
            },
            {
              icon: icons.sessions,
              t: "Local and quiet",
              b: "The host launches sweeploom mcp and talks JSON-RPC over stdin and stdout. No network requests, no account, no telemetry, no LLM inside the tool.",
            },
          ].map((item) => (
            <li key={item.t} className="card p-6">
              <span className="grid h-10 w-10 place-items-center rounded-xl border border-line bg-bg-2 text-accent">
                <item.icon size={20} />
              </span>
              <h3 className="mt-5 font-display text-lg font-semibold">{item.t}</h3>
              <p className="mt-2 text-[0.95rem] leading-relaxed text-muted">{item.b}</p>
            </li>
          ))}
        </ul>
        <div className="mt-6 grid gap-4 lg:grid-cols-2">
          <p className="rounded-xl border border-line bg-panel p-5 text-sm leading-relaxed text-muted">
            <strong className="text-text">What reaches your model provider.</strong> Tool results (paths, sizes, process and
            project details) enter the agent&apos;s context, so they reach whichever model provider your host uses. SweepLoom
            itself sends nothing anywhere. See the <Link href="/privacy/" className="link">privacy policy</Link>.
          </p>
          <p className="rounded-xl border border-line bg-panel p-5 text-sm leading-relaxed text-muted">
            <strong className="text-text">Not a token saver.</strong> SweepLoom does not compress context. A live coding-agent
            run on 2026-09-15 found no coding-task token saving with the MCP attached, and we{" "}
            <Link href="/blog/always-on-prompt-tax/" className="link">
              publish that result
            </Link>
            .
          </p>
        </div>
      </Section>

      <Section
        id="install"
        eyebrow="Install"
        title="One binary. The host starts it as sweeploom mcp."
        lead="Install the CLI from source; it is also the MCP server. Packages on crates.io and npm are not published yet."
        className="border-t border-line"
      >
        <div className="grid gap-6 lg:grid-cols-[1fr_1.15fr]">
          <div className="grid content-start gap-4">
            <CodeBlock code={`$ ${INSTALL.cli}`} title="install the CLI + MCP server" wrap />
            <ul className="grid gap-3 text-[0.95rem] leading-relaxed text-muted">
              <li className="flex gap-3">
                <icons.check size={18} className="mt-1 shrink-0 text-accent" />
                <span>
                  The server is the <C>sweeploom</C> binary with the argument <C>mcp</C>. There is no separate{" "}
                  <C>sweeploom-mcp</C> binary; that name is only the alias of a future npm launcher.
                </span>
              </li>
              <li className="flex gap-3">
                <icons.check size={18} className="mt-1 shrink-0 text-accent" />
                <span>
                  Server id <C>{SITE.mcpName}</C>, transport stdio. You never type <C>tools/call</C> yourself; the host does.
                </span>
              </li>
              <li className="flex gap-3">
                <icons.check size={18} className="mt-1 shrink-0 text-accent" />
                <span>
                  <C>sweeploom mcp --list</C> prints the contract without serving, so you can check the install before wiring
                  a host.
                </span>
              </li>
            </ul>
          </div>
          <CodeBlock code={MCP_LIST} title="the agent contract, as the CLI prints it" />
        </div>
      </Section>

      <Section
        id="hosts"
        eyebrow="Connect a host"
        title="Claude Code, Cursor, Claude Desktop and Codex."
        lead="These are the hosts the docs cover. Any MCP host that launches stdio servers can use the same command and args. Reload the host after adding the entry."
        className="border-t border-line"
      >
        <Tabs
          label="MCP hosts"
          items={[
            {
              id: "claude-code",
              label: "Claude Code",
              content: (
                <HostPanel>
                  <div className="grid gap-4">
                    <CodeBlock code={`$ ${CLAUDE_CODE_ADD}`} title="user scope, every project" wrap />
                    <CodeBlock code={MCP_JSON} title=".mcp.json · project scope" />
                  </div>
                  <div className="grid gap-4">
                    <p>
                      <strong>User scope</strong> makes SweepLoom available in every project. For one repository, commit a{" "}
                      <C>.mcp.json</C> with the same <C>sweeploom mcp</C> launch; the SweepLoom repository ships exactly that
                      file.
                    </p>
                    <p>
                      Working on SweepLoom itself? You can point Claude Code at a clone without installing, but it recompiles on
                      every host restart and is for development only:
                    </p>
                    <CodeBlock
                      code="$ claude mcp add -s user sweeploom -- cargo run -q --manifest-path C:/Users/you/work/sweeploom/Cargo.toml -p sweeploom -- mcp"
                      title="from a clone (development only)"
                      wrap
                    />
                  </div>
                </HostPanel>
              ),
            },
            {
              id: "cursor",
              label: "Cursor",
              content: (
                <HostPanel>
                  <CodeBlock code={MCP_JSON} title="~/.cursor/mcp.json" />
                  <div className="grid gap-4">
                    <p>
                      <C>~/.cursor/mcp.json</C> applies to every workspace (<C>%USERPROFILE%\.cursor\mcp.json</C> on Windows);{" "}
                      <C>.cursor/mcp.json</C> inside a project applies to that project. Then toggle <strong>sweeploom</strong>{" "}
                      under Cursor → Settings → MCP.
                    </p>
                    <p>
                      Do not look for it under Plugins or the Marketplace. SweepLoom is an MCP server, and the Marketplace will
                      not list it.
                    </p>
                  </div>
                </HostPanel>
              ),
            },
            {
              id: "claude-desktop",
              label: "Claude Desktop",
              content: (
                <HostPanel>
                  <CodeBlock code={MCP_JSON} title="claude_desktop_config.json" />
                  <div className="grid gap-4">
                    <p>Add the same object to the Claude Desktop config file, then restart the app:</p>
                    <ul className="grid gap-2 text-sm">
                      <li>
                        <strong>macOS</strong> <C>~/Library/Application Support/Claude/claude_desktop_config.json</C>
                      </li>
                      <li>
                        <strong>Windows</strong> <C>%APPDATA%\Claude\claude_desktop_config.json</C>
                      </li>
                    </ul>
                    <p>
                      A desktop app started from the Dock or Start menu may not see your shell <C>PATH</C>. If the server does
                      not start, replace <C>&quot;sweeploom&quot;</C> with the absolute path, as on the Absolute path tab.
                    </p>
                  </div>
                </HostPanel>
              ),
            },
            {
              id: "codex",
              label: "Codex",
              content: (
                <HostPanel>
                  <div className="grid gap-4">
                    <CodeBlock code={CODEX_TOML} title="~/.codex/config.toml" />
                    <CodeBlock code={`$ ${CODEX_PLUGIN.marketplace}\n$ ${CODEX_PLUGIN.add}`} title="optional: the plugin's skills" />
                  </div>
                  <div className="grid gap-4">
                    <p>
                      For MCP only, add the server to <C>~/.codex/config.toml</C>. The{" "}
                      <a href="#codex" className="link">
                        Codex plugin
                      </a>{" "}
                      adds three review skills on top of the same server.
                    </p>
                    <p>
                      Register the server in <C>config.toml</C> even if you install the plugin: the plugin&apos;s own entry runs{" "}
                      <C>npx -y sweeploom mcp</C>, which needs the npm package that is not published yet. Start a new Codex
                      session afterwards.
                    </p>
                  </div>
                </HostPanel>
              ),
            },
            {
              id: "windows",
              label: "Absolute path",
              hint: "Windows",
              content: (
                <HostPanel>
                  <CodeBlock code={MCP_JSON_WINDOWS} title="mcp.json with an explicit path" />
                  <div className="grid gap-4">
                    <p>
                      When a GUI host cannot find <C>sweeploom</C> on <C>PATH</C>, give it the full path to the installed
                      binary. Cargo installs it to <C>%USERPROFILE%\.cargo\bin</C> on Windows and <C>~/.cargo/bin</C> on macOS
                      and Linux. JSON needs doubled backslashes.
                    </p>
                    <p>
                      Prefer the installed binary in every always-on host. A <C>cargo run</C> launch recompiles on restart and can
                      disconnect when the disk is full.
                    </p>
                  </div>
                </HostPanel>
              ),
            },
          ]}
        />
        <div className="mt-8 rounded-xl border border-line bg-bg-2 p-5 text-[0.95rem] leading-relaxed text-muted">
          <p className="font-medium text-text">Check it works</p>
          <p className="mt-2">
            After a reload the host should list nine SweepLoom tools. Then ask, naming the product:{" "}
            <em>“Use SweepLoom to list cleanup_candidates for this repo and explain anything unclear. Do not apply yet.”</em>
          </p>
        </div>
      </Section>

      <Section
        id="tools"
        eyebrow="Tool reference"
        title="Nine tools. Eight read. One writes, behind confirm=true."
        lead="Arguments, access and what each tool returns, from the server's own catalog. Every read-only tool publishes readOnlyHint, and a test fails if the served annotations drift from that contract."
        className="border-t border-line"
      >
        <div className="table-wrap">
          <table>
            <caption className="sr-only">SweepLoom MCP tools: arguments, access, CLI equivalent and returned fields</caption>
            <thead>
              <tr>
                <th scope="col">Tool</th>
                <th scope="col">Arguments</th>
                <th scope="col">Access</th>
                <th scope="col">CLI twin</th>
                <th scope="col">Returns</th>
              </tr>
            </thead>
            <tbody>
              {MCP_TOOLS.map((tool) => (
                <tr key={tool.name}>
                  <td className="min-w-[13rem]">
                    <code>{tool.name}</code>
                    <div className="mt-1 text-xs leading-snug text-muted">{tool.does}</div>
                  </td>
                  <td className="font-mono text-xs whitespace-nowrap text-muted">{tool.args}</td>
                  <td>
                    {tool.readOnly ? (
                      <span className="chip chip-ok">read-only</span>
                    ) : (
                      <span className="chip chip-warn">gated</span>
                    )}
                  </td>
                  <td className="font-mono text-xs whitespace-nowrap text-muted">{tool.cli}</td>
                  <td className="min-w-[14rem] font-mono text-[0.72rem] leading-relaxed text-muted">{tool.returns}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <ul className="mt-6 grid gap-3 text-[0.95rem] leading-relaxed text-muted md:grid-cols-2">
          <li className="rounded-xl border border-line bg-panel p-4">
            <strong className="text-text">Empty root is your home directory.</strong> Path tools default to home when{" "}
            <C>root</C> is missing or empty. Agents should always pass the workspace&apos;s absolute path.
          </li>
          <li className="rounded-xl border border-line bg-panel p-4">
            <strong className="text-text">Ids come from review.</strong> <C>explain_candidate</C> resolves an id from{" "}
            <C>cleanup_candidates</C> (or an AI store row from <C>list_ai_stores</C>); an unknown id returns{" "}
            <C>unknown candidate &lt;id&gt;</C>.
          </li>
          <li className="rounded-xl border border-line bg-panel p-4">
            <strong className="text-text">Unknown stays unknown.</strong> <C>list_browser</C> returns <C>tabs: null</C> without
            a fresh browser companion snapshot. Null is not zero.
          </li>
          <li className="rounded-xl border border-line bg-panel p-4">
            <strong className="text-text">Not in the catalog:</strong> no <C>terminate</C>, no <C>park_file</C>, no{" "}
            <C>rewrite_agents</C>. Those are not hidden flags; they do not exist.
          </li>
        </ul>
      </Section>

      <Section
        id="gate"
        eyebrow="The apply gate"
        title="List, explain, ask, then apply exactly what was agreed."
        lead="apply_cleanup runs the same plan → revalidate → execute → receipt path as sweeploom clean --apply. The difference is consent: on the CLI --apply is yours; over MCP the agent must send confirm=true, and only after you said yes."
        className="border-t border-line"
      >
        <div className="grid gap-8 lg:grid-cols-[1.15fr_1fr]">
          <div className="grid content-start gap-4">
            <CodeBlock code={GATE_CALLS} title="the only apply path" copy={false} />
            <ol className="grid gap-3 text-[0.95rem] leading-relaxed text-muted">
              {[
                ["Review tools first.", "cleanup_candidates, then explain_candidate for anything unclear."],
                ["Say the plan in plain words.", "Path, size, why it is SAFE generated, what it costs to rebuild."],
                ["confirm: true only after the user agrees.", "Never as a retry after a refusal."],
                [
                  "ids = exactly what was agreed.",
                  "Empty ids means every pre-selected SAFE row, the same as sweeploom clean --apply. Do not send empty ids unless that is what the user asked for.",
                ],
              ].map(([t, b], i) => (
                <li key={t} className="flex gap-3">
                  <span className="grid h-6 w-6 shrink-0 place-items-center rounded-full bg-gold font-mono text-xs font-semibold text-ink">
                    {i + 1}
                  </span>
                  <span>
                    <strong className="text-text">{t}</strong> {b}
                  </span>
                </li>
              ))}
            </ol>
          </div>
          <div className="grid content-start gap-4">
            <CodeBlock code={RECEIPT} title="a receipt" copy={false} />
            <div className="grid gap-3 text-sm leading-relaxed text-muted">
              <p>
                <strong className="text-text">Revalidated at apply time.</strong> If <C>target/</C> changed since the listing, that
                row is counted in <C>skipped_changed</C>, not forced.
              </p>
              <p>
                <strong className="text-text">Blocked rows never go.</strong> Rows that are blocked (an active process, recent
                writes, uncommitted Git changes and similar) or inspect-only are filtered out even if their ids are passed.
              </p>
              <p>
                <strong className="text-text">Locked by a test.</strong>{" "}
                <C>mcp_apply_without_confirm_does_not_run</C> asserts that <C>confirm: false</C> returns <C>ok=false</C> and
                deletes nothing.
              </p>
            </div>
          </div>
        </div>
        <div className="mt-8 grid gap-4 md:grid-cols-3">
          {[
            ["confirm missing or false", "refused: MCP apply needs confirm=true after cleanup_candidates; CLI uses --apply"],
            ["an id that was never listed", "unknown candidate 999999"],
            ["nothing SAFE matches", "nothing eligible to apply"],
          ].map(([when, msg]) => (
            <div key={when} className="card p-5">
              <p className="font-mono text-[0.68rem] tracking-[0.12em] text-faint uppercase">{when}</p>
              <p className="mt-3 font-mono text-[0.8rem] leading-relaxed text-warn">{msg}</p>
            </div>
          ))}
        </div>
      </Section>

      <Section id="limits" eyebrow="Boundaries" title="What an agent can and cannot do." className="border-t border-line">
        <div className="grid gap-4 md:grid-cols-2">
          <div className="card p-6">
            <h3 className="flex items-center gap-2 font-display text-lg font-semibold">
              <icons.check size={20} className="text-ok" /> Can
            </h3>
            <ul className="mt-4 grid gap-2.5 text-[0.95rem] leading-relaxed text-muted">
              <li>List live sessions grouped into trees, with Keep or Optional for each.</li>
              <li>Map a folder, list its projects with source and artifact heat and Git state.</li>
              <li>Inspect AI stores and their always-on token estimates without opening secrets or SQLite.</li>
              <li>Explain any candidate row before you decide.</li>
              <li>Delete SAFE generated rows you agreed to, after revalidation, and show you the receipt.</li>
            </ul>
          </div>
          <div className="card p-6">
            <h3 className="flex items-center gap-2 font-display text-lg font-semibold">
              <icons.x size={20} className="text-warn" /> Cannot
            </h3>
            <ul className="mt-4 grid gap-2.5 text-[0.95rem] leading-relaxed text-muted">
              <li>Terminate any process. Idle agents and their attached MCP servers stay Keep.</li>
              <li>Delete anything without confirm=true, or delete blocked and inspect-only rows at all.</li>
              <li>Edit AGENTS.md, *.mdc or rules, or flip alwaysApply. park? is advice for you.</li>
              <li>
                Touch Docker, simulators, models, archives or the Trash. Those live in the{" "}
                <Link href="/app/#cleanup" className="link">
                  desktop app
                </Link>
                , where a person confirms.
              </li>
              <li>See browser tabs without the companion. It reports unknown, not zero.</li>
            </ul>
          </div>
        </div>
      </Section>

      <Section
        id="chats"
        eyebrow="Example chats"
        title="What good and bad requests look like."
        lead="Condensed from the copy-paste chats in the repository. The JSON is the shape the tools return; sizes are from the 2026-09-14 capture."
        className="border-t border-line"
      >
        <Tabs
          label="Example MCP chats"
          items={[
            {
              id: "a",
              label: "Review, then apply",
              content: <Chat title="“This repo is huge, clean it”" verdict={{ text: "the only apply path", tone: "ok" }} turns={CHAT_A} />,
            },
            {
              id: "b",
              label: "Skipped review",
              content: <Chat title="An agent tries to skip review" verdict={{ text: "refused, 0 deleted", tone: "warn" }} turns={CHAT_B} />,
            },
            {
              id: "c",
              label: "Token tax",
              content: <Chat title="“Why is my agent so expensive?”" verdict={{ text: "read-only", tone: "ok" }} turns={CHAT_C} />,
            },
            {
              id: "d",
              label: "Kill idle Cursor",
              content: <Chat title="“Cursor is idle, kill it”" verdict={{ text: "no terminate tool", tone: "warn" }} turns={CHAT_D} />,
            },
            {
              id: "e",
              label: "Map a folder",
              content: <Chat title="Map a folder, then decide" verdict={{ text: "read-only", tone: "ok" }} turns={CHAT_E} />,
            },
          ]}
        />
        <div className="mt-8 grid gap-4 lg:grid-cols-[1.3fr_1fr] lg:items-start">
          <CodeBlock
            code={`Use SweepLoom to list cleanup_candidates for this repo and explain anything unclear. Do not apply yet.\nUse SweepLoom to list_ai_stores and show always-on token tax. Do not park AGENTS.md or rules.\nUse SweepLoom to list_sessions. Do not terminate live agents.`}
            title="prompts that work"
            wrap
          />
          <p className="text-[0.95rem] leading-relaxed text-muted">
            The full transcripts, including the JSON for every call, are in{" "}
            <a href={docsUrl("MCP-CHATS.md")} className="link">
              docs/MCP-CHATS.md
            </a>{" "}
            and{" "}
            <a href={docsUrl("MCP-CHATS-AGENTS.md")} className="link">
              docs/MCP-CHATS-AGENTS.md
            </a>
            .
          </p>
        </div>
      </Section>

      <Section
        id="codex"
        eyebrow="Codex plugin"
        title="The same server, plus skills that make review the default."
        lead="The plugin is listed in the weavatrix marketplace in this repository. Its three skills turn the chats above into instructions Codex follows on every request."
        className="border-t border-line"
      >
        <div className="grid gap-8 lg:grid-cols-[1fr_1.1fr]">
          <div className="grid content-start gap-4">
            <CodeBlock code={`$ ${CODEX_PLUGIN.marketplace}\n$ ${CODEX_PLUGIN.add}`} title="Codex CLI" />
            <CodeBlock code={CODEX_TOML} title="also register the server (npm is not published yet)" />
            <p className="text-[0.95rem] leading-relaxed text-muted">
              From a clone: <C>codex plugin marketplace add .</C> then <C>{CODEX_PLUGIN.add}</C>. Start a <strong className="text-text">new</strong> Codex
              session; check with <C>codex plugin list</C> or <C>/plugins</C>.
            </p>
          </div>
          <div className="grid content-start gap-4">
            {SKILLS.map((skill) => (
              <div key={skill.name} className="card p-5">
                <p className="font-mono text-sm text-accent">{skill.name}</p>
                <p className="mt-2 text-[0.95rem] leading-relaxed text-muted">{skill.does}</p>
                <p className="mt-3 font-mono text-[0.72rem] leading-relaxed text-faint">{skill.loop}</p>
              </div>
            ))}
            <p className="text-sm leading-relaxed text-muted">
              The plugin will not terminate live agents, apply without <C>confirm=true</C>, rewrite <C>AGENTS.md</C>,{" "}
              <C>*.mdc</C> or <C>rules</C>, or open secrets or SQLite.{" "}
              <Link href="/blog/codex-plugin-mcp-plus-skills/" className="link">
                Why skills matter
              </Link>
              .
            </p>
          </div>
        </div>
      </Section>

      <Section id="troubleshooting" eyebrow="Troubleshooting" title="When the agent does not do what you expect." className="border-t border-line">
        <Faq items={TROUBLE} />
      </Section>

      <Section id="docs" eyebrow="Reference" title="The MCP docs on GitHub." className="border-t border-line">
        <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {DOCS.map((doc) => (
            <li key={doc.file}>
              <a href={docsUrl(doc.file)} className="card card-hover flex h-full items-start gap-3 p-4">
                <icons.context size={18} className="mt-0.5 shrink-0 text-accent" />
                <span className="min-w-0">
                  <span className="block font-medium text-text">{doc.title}</span>
                  <span className="mt-0.5 block text-sm text-muted">{doc.body}</span>
                  <span className="mt-1 block font-mono text-[0.7rem] text-faint">docs/{doc.file}</span>
                </span>
              </a>
            </li>
          ))}
        </ul>
      </Section>
    </>
  );
}
