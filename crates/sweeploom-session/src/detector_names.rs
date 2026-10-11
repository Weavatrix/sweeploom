//! Name and command-needle tables for the built-in detectors.

/// MCP server binaries, matched on process name or executable file name.
pub(super) const MCP_BINARIES: &[&str] = &[
    "mcp-server",
    "mcp-server.exe",
    "mcp_server",
    "mcp_server.exe",
    "weavatrix",
    "weavatrix.exe",
    "weavatrix-mcp",
    "weavatrix-mcp.exe",
    "weavatrix-git",
    "weavatrix-git.exe",
    "weavatrix-md",
    "weavatrix-md.exe",
    "sweeploom-mcp",
    "sweeploom-mcp.exe",
];

/// MCP signatures in command arguments.
pub(super) const MCP_NEEDLES: &[&str] = &[
    "mcp-server",
    "mcp_server",
    "@modelcontextprotocol",
    "--mcp",
    "weavatrix",
    "weavatrix-mcp",
    "weavatrix-git",
    "weavatrix-md",
    "sweeploom-mcp",
];

/// JS dev-server / bundler entrypoints.
pub(super) const DEV_SERVER_NEEDLES: &[&str] = &[
    "vite", "next", "nuxt", "webpack", "parcel", "nodemon", "tsx", "esbuild",
];

/// Language server binaries.
pub(super) const LANGUAGE_SERVERS: &[&str] = &[
    "rust-analyzer",
    "rust-analyzer.exe",
    "gopls",
    "typescript-language-server",
];

/// Shells and terminal hosts.
pub(super) const TERMINALS: &[&str] = &[
    "cmd",
    "cmd.exe",
    "powershell",
    "powershell.exe",
    "pwsh",
    "pwsh.exe",
    "WindowsTerminal.exe",
    "bash",
    "bash.exe",
    "zsh",
    "fish",
    "wt.exe",
];
