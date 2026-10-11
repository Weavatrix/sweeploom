# Browser companion

[README](../README.md) · [All docs](../README.md#documentation)

Without the companion, SweepLoom still sees browser **process trees** and
their RSS. It does not see tabs, and it does not guess: tab counts stay
unknown (`tabs: null`), never zero. The optional companion extension
reports tab `lastAccessed` so the desktop Browser screen can show tab heat.

## Install

```text
cargo build -p sweeploom --release --bins
sweeploom companion-install
sweeploom companion-install --chromium-id <id>
```

Extensions: `browser/chromium-extension`, `browser/firefox-extension`.

1. Install the CLI. `cargo install --git …` ([INSTALL.md](INSTALL.md))
   already places `sweeploom-companion-host` next to `sweeploom`; from a
   clone, build both with the `cargo build` line above.
2. Load `browser/chromium-extension` (Chrome, Edge, other Chromium) or
   `browser/firefox-extension` (Firefox 121+) **unpacked**.
3. Run `sweeploom companion-install`. Chromium browsers also need the
   extension id from `edge://extensions` / `chrome://extensions`: pass
   `--chromium-id <id>`, or set it from SweepLoom → Browser → Tabs.
4. Check: `sweeploom browser` shows `companion=connected` once a fresh
   snapshot arrives.

`companion-install` writes the native-messaging host manifests
(`com.sweeploom.companion`) and registers them. `companion-host` is what
the browser launches — not a daily command.

## What it sends and where

The extension talks only to the local host process through the browser's
**native messaging** channel (stdin/stdout of `sweeploom-companion-host`).
It makes no network requests. Per tab it sends: tab and window id, title,
URL with query string, fragment, and user info stripped, `lastAccessed`,
and the pinned / audible / discarded / incognito flags. The host stores
the last snapshot in SweepLoom's local app-data directory; a snapshot
older than 15 minutes no longer counts as a live companion.

## What it can do

The extension understands three actions. The desktop app queues them only
when you choose them (discarding suggested tabs asks for confirmation
first), and the companion applies them on its next tabs ping:

| Action | Effect |
| --- | --- |
| Discard | Unloads a tab from memory; the tab stays in the strip |
| Focus | Brings the tab and its window to the front |
| Bookmark and close | Creates a bookmark first, closes the tab only if the bookmark exists and the URL is unchanged |

Pinned, audible, incognito, and active tabs are never discarded or
closed. Close is never the suggested default action. The **Later** shelf
in the desktop app saves a tab's title and http(s) URL locally without
closing anything in the live browser.

CLI view: [CLI-CLEAN.md](CLI-CLEAN.md#7-browser--rss-is-real-tabs-are-not-guessed).
Library: `list_browser()` in [LIBRARY.md](LIBRARY.md#browser-pressure).
