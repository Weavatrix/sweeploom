/* Product facts, transcribed from README.md and docs/*.md in the SweepLoom
   repository. Keep this file honest: if the product does not do it, it does
   not go here. */

export type Surface = "CLI" | "MCP" | "App";

export type Finding = {
  id: string;
  icon:
    | "build"
    | "package"
    | "phone"
    | "container"
    | "model"
    | "sessions"
    | "projects"
    | "browser"
    | "context"
    | "archive";
  title: string;
  body: string;
  examples: string[];
  surfaces: Surface[];
};

export const FINDINGS: Finding[] = [
  {
    id: "generated",
    icon: "build",
    title: "Generated build output",
    body: "Rebuildable artifacts inside your projects, attributed to the workspace that owns them, with the rebuild cost next to the size.",
    examples: ["target/", "node_modules/", "Python generated output"],
    surfaces: ["CLI", "MCP", "App"],
  },
  {
    id: "sessions",
    icon: "sessions",
    title: "Live sessions",
    body: "Processes grouped into the trees you actually run: an agent, its MCP servers, its terminals and dev servers. Idle agents are Keep.",
    examples: ["Claude Code", "Codex", "Cursor", "OpenCode", "Gemini", "Grok", "MCP", "dev servers"],
    surfaces: ["CLI", "MCP", "App"],
  },
  {
    id: "context",
    icon: "context",
    title: "Always-on prompt tax",
    body: "Estimated tokens for the files an agent host injects every turn. History on disk counts as zero. Secrets and SQLite are never opened.",
    examples: ["AGENTS.md", "*.mdc", "rules", "skills", "plugins"],
    surfaces: ["CLI", "MCP", "App"],
  },
  {
    id: "projects",
    icon: "projects",
    title: "Projects",
    body: "Source heat and artifact heat per project, Git state, and artifact offers such as a Cargo target with its rebuild cost.",
    examples: ["ActiveNow · Hot · Warm", "git dirty / clean", "cargo Target"],
    surfaces: ["CLI", "MCP", "App"],
  },
  {
    id: "browser",
    icon: "browser",
    title: "Browser pressure",
    body: "Real RSS and CPU per browser. Tab ages come from the optional companion extension and are never guessed. Unknown is shown as unknown, not zero.",
    examples: ["Edge", "Chrome", "Firefox", "lastAccessed via companion"],
    surfaces: ["CLI", "MCP", "App"],
  },
  {
    id: "packages",
    icon: "package",
    title: "Package & tool caches",
    body: "Download caches for the toolchains you use, measured in the background and offered as generated cleanup.",
    examples: ["npm", "pnpm", "Yarn", "Bun", "pip", "uv", "Poetry", "Cargo", "Gradle", "NuGet", "Go", "Homebrew", "Playwright", "SwiftPM"],
    surfaces: ["App"],
  },
  {
    id: "xcode",
    icon: "phone",
    title: "Xcode & iOS Simulator",
    body: "DerivedData by project, device support by OS version, simulator devices and removable runtimes through simctl, with running-device checks.",
    examples: ["DerivedData", "iOS DeviceSupport", "simctl delete", "simctl runtime delete"],
    surfaces: ["App"],
  },
  {
    id: "docker",
    icon: "container",
    title: "Docker",
    body: "Individual images, stopped containers, unused volumes and old build cache, removed through native Docker operations. In-use objects are protected.",
    examples: ["images", "stopped containers", "unused volumes", "build cache"],
    surfaces: ["App"],
  },
  {
    id: "models",
    icon: "model",
    title: "AI models",
    body: "Local model downloads, itemised per model or repository. Ollama models are removed through Ollama's own local API.",
    examples: ["Ollama", "Hugging Face", "LM Studio", "PyTorch", "Whisper", "Core ML"],
    surfaces: ["App"],
  },
  {
    id: "archives",
    icon: "archive",
    title: "Archives & large data",
    body: "User data that is not regenerable is never treated as cache. It is shown with its consequence and moved to Trash only if you choose it.",
    examples: ["Xcode archives", "device backups", "large downloads", "logs", "Rust toolchains", "Node versions"],
    surfaces: ["App"],
  },
];

/* The MCP catalog, from crates/sweeploom-cli/src/mcp.rs and surface.rs.
   `readOnly` mirrors the readOnlyHint the server publishes in tools/list. */
export type McpTool = {
  name: string;
  args: string;
  writes: string;
  readOnly: boolean;
  cli: string;
  does: string;
  returns: string;
};

export const MCP_TOOLS: McpTool[] = [
  {
    name: "list_sessions",
    args: "none",
    writes: "no",
    readOnly: true,
    cli: "sessions",
    does: "Live session trees with Keep / Optional recommendations",
    returns: "kind, processes, rss_bytes, cpu_percent, recommendation, activity, project, planned",
  },
  {
    name: "disk_inventory",
    args: "root? string",
    writes: "no",
    readOnly: true,
    cli: "scan",
    does: "What is on disk under a root, with categories (metadata only)",
    returns: "root, entries, projects, capped, logical_bytes, children[path, logical_bytes, category]",
  },
  {
    name: "list_projects",
    args: "root? string",
    writes: "no",
    readOnly: true,
    cli: "projects",
    does: "Project kinds, source and artifact heat, Git state",
    returns: "path, kind, source, artifact, git",
  },
  {
    name: "list_ai_stores",
    args: "none",
    writes: "no",
    readOnly: true,
    cli: "ai",
    does: "AI tool stores, classes and always-on token estimates; never opens secrets or SQLite",
    returns: "title, path, logical_bytes, capped, entries[id, class, relative, prompt_tokens, advice, can_clean]",
  },
  {
    name: "advise_context",
    args: "relative string",
    writes: "no",
    readOnly: true,
    cli: "—",
    does: "Advice for one leaf name: keep for Context such as AGENTS.md, empty for History; never writes",
    returns: "relative, class, advice (park? needs file age, so it comes from list_ai_stores)",
  },
  {
    name: "cleanup_candidates",
    args: "root? string",
    writes: "no",
    readOnly: true,
    cli: "clean",
    does: "Reviewable cleanup rows (dry run); SAFE generated rows pre-selected",
    returns: "id, title, path, logical_bytes, selected, blocked, deletion",
  },
  {
    name: "explain_candidate",
    args: "id u64, root? string",
    writes: "no",
    readOnly: true,
    cli: "—",
    does: "One row in detail before anyone agrees to it",
    returns: "the same row, or the error unknown candidate <id>",
  },
  {
    name: "list_browser",
    args: "none",
    writes: "no",
    readOnly: true,
    cli: "browser",
    does: "Browser pressure; tabs null when unknown",
    returns: "companion, hosts, rss_bytes, tabs (null without a fresh companion)",
  },
  {
    name: "apply_cleanup",
    args: "confirm bool, root?, ids? u64[]",
    writes: "yes, if confirm",
    readOnly: false,
    cli: "clean --apply",
    does: "Revalidate and delete the agreed ids; returns a receipt",
    returns: "ok, error, deleted, skipped_changed, failed, receipt",
  },
];

export const CLASSES: { leaf: string; cls: string; clean: string; tokens: string }[] = [
  { leaf: "AGENTS.md, *.mdc, rules", cls: "Context", clean: "no", tokens: "min(bytes/4, 8000); skills, plugins, rules dirs capped at 4096" },
  { leaf: "history.jsonl, projects, sessions", cls: "History", clean: "no", tokens: "0" },
  { leaf: "cache, tmp, CachedData", cls: "Cache", clean: "yes", tokens: "0" },
  { leaf: ".credentials.json, token.txt", cls: "Secret", clean: "no", tokens: "0" },
  { leaf: "*.sqlite, *.vscdb", cls: "SQLite", clean: "no", tokens: "0" },
  { leaf: "*.log, .last-cleanup", cls: "Log", clean: "yes", tokens: "0" },
  { leaf: "mcp.json, settings.json", cls: "Settings", clean: "no", tokens: "0" },
  { leaf: "password-reset.md, cachet.json", cls: "Other", clean: "no", tokens: "0" },
];

export const HARD_LINES: string[] = [
  "No terminate tool in the CLI or MCP. Live agents and their attached MCP servers stay Keep.",
  "Secrets, SQLite databases and History are inspect-only. Secrets are never opened.",
  "AGENTS.md, *.mdc and rules are inspect-only. park? is advice; alwaysApply is never flipped.",
  "No LLM on classify or apply. Classification is local and deterministic.",
  "A recommendation never overrides a safety blocker.",
  "Processes are keyed by PID plus start time, so a recycled PID is never mistaken for yours.",
  "Command lines are redacted (--token values, URI credentials) before UI, logs or receipts.",
  "No tokio, hyper or reqwest on the CLI path; cargo-deny bans them.",
];

export const CLI_COMMANDS = `$ sweeploom sessions             # what is alive, grouped by agent
$ sweeploom sessions --quiet     # dry-run plan; never a kill
$ sweeploom scan .               # what is on disk in this tree
$ sweeploom projects .           # heat, Git state, artifact offers
$ sweeploom ai                   # AI stores and always-on token tax
$ sweeploom browser              # browser pressure; tabs via companion
$ sweeploom bench                # with vs without, fixed gold store
$ sweeploom clean .              # look first
$ sweeploom clean . --apply      # then delete SAFE generated rows`;

export const CLEAN_TRANSCRIPT = `$ sweeploom clean .
[x]  1.2 GB    cargo target
[x]  812.0 MB  node_modules
[ ]  12.0 KB   something inspect-only  BLOCKED
dry-run; pass --apply to delete pre-selected SAFE rows after revalidation

$ sweeploom clean . --apply
receipt=1  deleted=2  skipped_changed=0  failed=0`;

export const BENCH_TRANSCRIPT = `$ sweeploom bench
SweepLoom bench — AI WITHOUT vs WITH (fixed gold store, not your disk)
WITHOUT dump-store/4          238466250 tok
WITHOUT History-as-prompt     965000 tok
WITH    always-on estimate    16288 tok
apply confirm=false           refused=true deleted=0
idle 2GB agents               WITHOUT would-kill=6  WITH Keep=6
AGENTS.md                     keep`;

export type GuiTab = { name: string; id: string; does: string; sub?: string[] };

/* The desktop app's sidebar, in order (apps/sweeploom-gui/src/nav.rs). */
export const GUI_SCREENS: { section: string; items: GuiTab[] }[] = [
  {
    section: "Live",
    items: [
      { name: "Overview", id: "overview", does: "Memory, per-core CPU and disk pressure; every tile opens the screen that explains it" },
      { name: "Sessions", id: "sessions", does: "Session trees, member processes, observed CPU, dry-run plans" },
      { name: "History", id: "history", does: "Observed process activity, plus saved disk scans", sub: ["Process activity", "Disk scans"] },
    ],
  },
  {
    section: "Disk",
    items: [
      {
        name: "Cleanup",
        id: "cleanup",
        does: "Everything reclaimable, by source, with the consequence of removal",
        sub: [
          "Project output",
          "Docker",
          "iOS & Xcode",
          "Build & packages",
          "App & browser caches",
          "AI models",
          "Archives & large data",
        ],
      },
      { name: "Projects", id: "projects", does: "Every project under your developer folders: size, change, Git, artifacts" },
      { name: "Explorer", id: "explorer", does: "Folder inspector, off the UI thread; collapsing never re-walks the disk" },
    ],
  },
  {
    section: "Workspace",
    items: [
      { name: "Browser", id: "browser", does: "Browser trees, companion tabs, the Later shelf", sub: ["Process trees", "Tabs", "Later"] },
      { name: "AI", id: "ai", does: "AI stores, classes and always-on token tax" },
    ],
  },
  {
    section: "App",
    items: [{ name: "Settings", id: "settings", does: "Theme, interface size, tray; telemetry stays off" }],
  },
];
