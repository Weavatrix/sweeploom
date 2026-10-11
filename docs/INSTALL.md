# Install

[README](../README.md) · [All docs](../README.md#documentation)

SweepLoom is installed **from source** today. It is not published on
crates.io or npm yet, and there are no prebuilt GitHub releases. The
`package.json` and `npm/` launcher in this repository prepare a future npm
release; `npm i -g sweeploom` and `npx -y sweeploom` do not work yet.

## Requirements

- Rust via [rustup](https://rustup.rs/): 1.88+ for the CLI and library,
  1.95+ for the desktop app. Inside a clone, `rust-toolchain.toml` pins
  the toolchain and rustup installs it on first build.
- Git.
- Windows, macOS, or Linux (CI runs the tests on all three).

## CLI and MCP server

```text
cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom
sweeploom --help
sweeploom mcp --list
```

This puts two binaries in `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` on
Windows):

| Binary | What it is |
| --- | --- |
| `sweeploom` | The CLI. `sweeploom mcp` serves the MCP server over stdio |
| `sweeploom-companion-host` | Native-messaging host the [browser companion](BROWSER.md) launches |

There is no separate `sweeploom-mcp` binary in a Cargo install; that name
is only the alias of the future npm launcher (`npm/mcp.js`). MCP hosts
start `sweeploom` with the argument `mcp` ([MCP.md](MCP.md)).

Update: run the same `cargo install` line with `--force`. Remove:
`cargo uninstall sweeploom`.

## From a clone

```text
git clone https://github.com/Weavatrix/sweeploom.git
cd sweeploom
cargo run -p sweeploom -- --help
cargo run -p sweeploom -- sessions
cargo run -p sweeploom -- scan .
cargo run -p sweeploom -- clean .
```

`--` is required for `cargo run` so flags belong to SweepLoom, not Cargo.

For an always-on MCP host, install the clone's CLI once instead of wiring
`cargo run` into the host:

```text
cargo install --path crates/sweeploom-cli --locked
sweeploom mcp --list
```

## Desktop app

```text
git clone https://github.com/Weavatrix/sweeploom.git
cd sweeploom
cargo run --release -p sweeploom-gui
```

`cargo install --locked --git https://github.com/Weavatrix/sweeploom sweeploom-gui`
installs the same app as a `sweeploom-gui` binary.

**macOS.** For the Rust GUI on macOS, use the [signed app build](MACOS.md)
so folder permissions remain associated with the app across updates:

```sh
python3 scripts/macos-app.py
open target/SweepLoom.app
```

The script needs a code-signing identity in your keychain and never falls
back to an ad-hoc signature. Details, identity selection, and updates:
[MACOS.md](MACOS.md).

## MCP hosts

SweepLoom is an **MCP server**, not a Cursor Marketplace plugin.
Searching “sweeploom” under Plugins / Marketplace will stay empty.
Add the JSON below, then reload the host (or toggle the server in
Cursor → Settings → MCP).

```json
{
  "mcpServers": {
    "sweeploom": {
      "command": "sweeploom",
      "args": ["mcp"]
    }
  }
}
```

Every host (Cursor, Claude Code, Claude Desktop, Codex), explicit paths,
and the tool table: [MCP.md](MCP.md).

## Codex plugin

Codex plugin (new session after install):

```text
codex plugin marketplace add Weavatrix/sweeploom
codex plugin add sweeploom@weavatrix
```

From a clone: `codex plugin marketplace add .` then
`codex plugin add sweeploom@weavatrix`.

The plugin's bundled MCP entry uses `npx -y sweeploom mcp`, which needs the
unpublished npm package. Register `sweeploom mcp` yourself until then:
[PLUGIN.md](PLUGIN.md).

## Browser companion (optional)

Tab heat needs the companion extension: [BROWSER.md](BROWSER.md).

## Hostwatch staging scanner

For a Hostwatch node, the storage crate also builds a small read-only scanner:

```text
cargo build -p sweeploom-storage --bin sweeploom-staging-scan --release
sweeploom-staging-scan /srv/staging 48
```

It emits JSON for old `runtime/.next` directories in deployment staging,
including their exact paths and allocated disk bytes on Unix. It does not
delete files or classify container images as build cache. Hostwatch can run
this binary beside its Go agent; the Go agent has a native fallback on nodes
where the binary has not been installed.
