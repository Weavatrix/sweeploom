# SweepLoom Terms of Use

**Effective date:** 2026-10-12

These terms cover the SweepLoom software — the `sweeploom` CLI, the MCP
server, the Codex plugin, the Rust library, the desktop app (including
its Mac App Store edition), and the browser companion extension — and
the website at
[sweeploom.com](https://sweeploom.com). SweepLoom is built by Sergii
Ziborov at [Weavatrix](https://github.com/Weavatrix) (“we”, “us”).

By using SweepLoom or the website, you agree to these terms. If you do
not agree, do not use them.

## 1. The license governs the code

SweepLoom's source code is licensed under the
[Mozilla Public License 2.0](LICENSE) (MPL-2.0). Your rights to use,
copy, modify, and distribute the code come from that license. If
anything in these terms conflicts with the MPL-2.0 as applied to the
code, the MPL-2.0 wins. Nothing here takes away a right the MPL-2.0
gives you.

The Mac App Store edition is a compiled copy of the desktop app that
Apple distributes under Apple's terms. Section 5 explains how that fits
with the MPL-2.0.

Third-party components keep their own licenses. For example,
`weavatrix-scan`, `weavatrix-git`, and `mcport` are MIT-licensed.

## 2. What SweepLoom does

SweepLoom runs on your computer. It inspects disk usage, running
processes, developer projects, AI tool folders, and (with the optional
companion) browser tabs. When you confirm, it can delete generated
files, move files to the Trash, remove items through native tools (for
example Docker, the iOS Simulator, Ollama, or rustup), stop processes
from the desktop app, and discard or close browser tabs.

## 3. Your responsibilities

**Review before you confirm.** SweepLoom shows a plan first. Passing
`--apply` on the CLI, approving an agent's `apply_cleanup` call with
`confirm: true`, or confirming in the desktop app is your instruction to
act on the rows in that plan. Read the plan before you give it.

**Know what is permanent.** Different actions have different outcomes:

- Generated build output and caches (such as `target/` or
  `node_modules/`) are **deleted permanently**. They are expected to be
  regenerable, but rebuilding takes time, may need network access, and
  may not reproduce exactly the same files.
- Items moved to the **Trash** can be restored until you empty the
  Trash. Their space stays in use until then.
- Removals through native tools (Docker images and volumes, simulator
  devices and runtimes, Ollama models, Rust toolchains) are handled by
  those tools and are usually not recoverable from SweepLoom.
- Stopping a process in the desktop app can lose unsaved work in that
  process.

**Keep backups.** Keep current backups of anything you cannot afford to
lose, especially before cleaning large folders or many items at once.
SweepLoom revalidates rows before acting and refuses some paths, but
these checks reduce risk; they do not remove it.

**You are responsible for your agents.** If you connect SweepLoom to an
AI coding agent, you are responsible for what that agent asks it to do.
The `confirm: true` gate is meant to follow your explicit approval;
SweepLoom cannot verify that an agent actually asked you.

**Use it only where you are allowed to.** Run SweepLoom on computers and
accounts you own or are authorized to administer.

## 4. Privacy and your data

**Local only, no telemetry.** SweepLoom does not require an account and
does not collect telemetry, analytics, crash reports, or usage data. It
does not check for updates over the network.

**Network use.** The CLI and the MCP server make no network requests. The
desktop app connects only to a local Ollama server at `127.0.0.1:11434`,
if one is running, to list and remove models. The Mac App Store edition
makes no network connections at all. The browser companion
extension exchanges messages only with a local SweepLoom process through
the browser's native-messaging channel; it makes no network requests.
Cleanup actions may run local tools (such as `docker`, `simctl`, or
`rustup`), which follow their own behavior and terms.

**What it reads.** SweepLoom works mostly from metadata: file and folder
names, sizes, timestamps, process names, and resource usage. On the AI
store path it classifies files by name and does not open secret or SQLite
files. Command-line arguments are redacted before they are shown or
stored.

**What it stores.** Settings, scan history, the last browser companion
snapshot, and the Later shelf of saved tab titles and URLs are stored in
SweepLoom's local app-data directory on your computer. You can delete
them at any time.

**AI agents and their providers.** When an MCP host or Codex calls
SweepLoom's tools, the results (paths, sizes, process and project
details) enter that agent's context and may be sent to the agent's model
provider under that provider's terms. Choose what you ask your agent to
inspect accordingly.

**The website.** sweeploom.com does not require an account. Like any
website, its hosting provider processes basic request data, such as IP
address and browser type, to deliver and protect the pages.

## 5. The Mac App Store edition

The desktop app also has a Mac App Store edition: the same app, built to
run in Apple's App Sandbox. It is coming to the Mac App Store and is
being tested through Apple's TestFlight; it is not on sale yet. The full
edition stays free and open source, and you can still build it from
source.

**Buying it.** The planned price is USD 1.99. Purchases are handled by
Apple through the App Store, including payment; we do not receive your
payment details.

**Which license applies.** Your license to an App Store copy is Apple's
Standard End User License Agreement (the “Licensed Application End User
License Agreement”,
<https://www.apple.com/legal/internet-services/itunes/dev/stdeula/>).
For that copy, if Apple's agreement and these terms conflict, Apple's
agreement prevails. The rest of these terms still apply to it where
they do not conflict.

**The source stays open.** Section 3.2 of the MPL-2.0 lets a compiled
program be distributed under different terms, as long as those terms do
not limit your rights in the source code and you are told where to get
it. So the App Store executable is distributed under Apple's terms,
while the complete source code of the same app remains available under
the MPL-2.0 at <https://github.com/Weavatrix/sweeploom>. Apple's terms
do not reduce your rights in that source. Settings → About in the app
says the same and links to it.

**Refunds.** Refunds for App Store purchases are handled by Apple, not
by us. Request one at <https://reportaproblem.apple.com>.

**What it can reach.** The App Store edition works only inside folders
you grant with the macOS folder picker. It remembers each grant as a
security-scoped bookmark stored in its own app container, and you can
remove a grant in Settings at any time. It moves items to the Trash
through macOS, reads Git state from the repository files itself, starts
no other programs, and makes no network connections. Features that need
more access than the sandbox allows (Sessions and stopping processes,
Docker, iOS Simulator devices and runtimes, the browser companion,
Ollama models, Rust toolchain removal, and the CLI and MCP server) are
only in the full edition.

**You still review every cleanup.** Section 3 applies in full. A folder
grant lets SweepLoom read and clean inside that folder after you
confirm; it does not make a cleanup safe. Generated output is still
deleted permanently. Inside the sandbox, macOS does not let SweepLoom
see which other apps are running, so it cannot tell whether a build,
simulator, or app is using a file: quit them before you confirm.

**No data collection.** The App Store edition collects no data and
contains no analytics. Apple processes purchase and download data under
its own privacy policy; see our
[Privacy Policy](https://sweeploom.com/privacy/).

## 6. No warranty

SweepLoom and the website are provided **“as is”**, without warranty of
any kind, express, implied, or statutory, consistent with Section 6 of
the MPL-2.0. This includes, without limitation, any warranty that the
software is free of defects, merchantable, fit for a particular purpose,
or non-infringing.

In particular, we do not promise that classifications and
recommendations are correct for your files, that reported sizes are
exact, that a cleanup frees a particular amount of space, that a
benchmark reproduces on your machine, or that the software works with
every version of the third-party tools it inspects. The entire risk as
to the quality and performance of the software is with you.

## 7. Limitation of liability

To the fullest extent permitted by law, and consistent with Section 7 of
the MPL-2.0, neither Weavatrix nor any contributor is liable to you for
any direct, indirect, special, incidental, or consequential damages of
any kind, including loss of data, lost work, lost profits, loss of
goodwill, work stoppage, or computer failure or malfunction, arising from
your use of, or inability to use, SweepLoom or the website — even if we
were told such damages were possible.

Some jurisdictions do not allow the exclusion or limitation of certain
damages. Where that applies, our liability is limited to the smallest
amount the law allows.

## 8. Acceptable use

When you use SweepLoom or the website, do not:

- use SweepLoom to delete, alter, or disrupt data, processes, or systems
  you are not authorized to manage;
- attack, overload, or try to gain unauthorized access to the website or
  the infrastructure behind it;
- present a modified version of SweepLoom as the official Weavatrix
  release, or use the SweepLoom or Weavatrix names in a way that suggests
  endorsement you do not have;
- use SweepLoom or the website in violation of applicable law.

The MPL-2.0 lets you fork and redistribute the code. As its Section 2.3
states, it does not grant rights in contributors' trademarks, service
marks, or logos. You may refer to SweepLoom truthfully, for example
“based on SweepLoom”.

## 9. Third-party products and trademarks

SweepLoom inspects and works alongside software made by others. Apple,
App Store, Mac App Store, TestFlight, macOS, iOS, Xcode, Docker, Claude, Claude Code, Codex, Cursor, OpenCode,
Gemini, Grok, Ollama, Hugging Face, LM Studio, Windows, Edge, Chrome,
Firefox, and other names mentioned in SweepLoom or its documentation are
trademarks of their respective owners (for example Apple Inc., Docker,
Inc., Anthropic, OpenAI, Anysphere, Google, xAI, Microsoft, and Mozilla).

They are used only to describe what SweepLoom works with. SweepLoom and
Weavatrix are not affiliated with, endorsed by, or sponsored by any of
these owners. Your use of their products is governed by their own terms.

## 10. Contributions

Unless you state otherwise, issues, pull requests, and other
contributions you submit to the SweepLoom repository are provided under
the MPL-2.0, the same license as the code.

## 11. Changes

SweepLoom is under active development. Features may change or be removed
at any time.

We may update these terms. When we do, we change the effective date at
the top of this file, and the full history stays in the repository's Git
log. Continued use of SweepLoom or the website after an update means you
accept the updated terms. An update does not change the license of a
version of the code you already received.

If any part of these terms is found unenforceable, the rest stays in
effect.

## 12. Contact

Questions about these terms, bug reports, and security concerns go
through GitHub issues:
<https://github.com/Weavatrix/sweeploom/issues>. For a security problem,
open an issue that asks for a private contact and leave out exploit
details and secrets.
