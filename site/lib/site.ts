export const SITE = {
  name: "SweepLoom",
  url: "https://sweeploom.com",
  tagline: "Reclaim your workstation without losing your workspace",
  description:
    "SweepLoom is a local-first, developer-aware workstation resource manager. It finds generated build output, package caches, Xcode and Docker leftovers, forgotten helpers and always-on agent context, and never deletes anything without plan, revalidate, execute and receipt.",
  company: "Weavatrix",
  companyUrl: "https://weavatrix.com",
  companyGitHub: "https://github.com/Weavatrix",
  author: "Sergii Ziborov",
  repo: "https://github.com/Weavatrix/sweeploom",
  issues: "https://github.com/Weavatrix/sweeploom/issues",
  newIssue: "https://github.com/Weavatrix/sweeploom/issues/new",
  license: "MPL-2.0",
  version: "0.1.0",
  mcpName: "io.github.Weavatrix/sweeploom",
} as const;

export const docsUrl = (file: string) => `${SITE.repo}/blob/main/docs/${file}`;
export const repoFile = (file: string) => `${SITE.repo}/blob/main/${file}`;

export const INSTALL = {
  cli: "cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom",
  gui: "cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom-gui",
  guiClone: "git clone https://github.com/Weavatrix/sweeploom.git",
  guiRun: "cargo run --release -p sweeploom-gui",
  macApp: "python3 scripts/macos-app.py",
  macOpen: "open target/SweepLoom.app",
} as const;

export const MCP_JSON = `{
  "mcpServers": {
    "sweeploom": {
      "command": "sweeploom",
      "args": ["mcp"]
    }
  }
}`;

export const CODEX_TOML = `[mcp_servers.sweeploom]
command = "sweeploom"
args = ["mcp"]`;

/* Absolute-path variant for GUI hosts that do not inherit the shell PATH
   (docs/MCP.md). */
export const MCP_JSON_WINDOWS = `{
  "mcpServers": {
    "sweeploom": {
      "command": "C:\\\\Users\\\\you\\\\.cargo\\\\bin\\\\sweeploom.exe",
      "args": ["mcp"]
    }
  }
}`;

export const CLAUDE_CODE_ADD = "claude mcp add -s user sweeploom -- sweeploom mcp";

export const CODEX_PLUGIN = {
  marketplace: "codex plugin marketplace add Weavatrix/sweeploom",
  add: "codex plugin add sweeploom@weavatrix",
} as const;

export const NAV = [
  { href: "/features/", label: "Features" },
  { href: "/app/", label: "Desktop app" },
  { href: "/mcp/", label: "MCP" },
  { href: "/docs/", label: "Docs" },
  { href: "/download/", label: "Download" },
  { href: "/blog/", label: "Blog" },
  { href: "/changelog/", label: "Changelog" },
  { href: "/about/", label: "About" },
] as const;
