# Desktop Cleanup sources

[README](../README.md) · [All docs](../README.md#documentation)

Cleanup discovers existing locations and streams full metadata measurements in
the background. Sizes use allocated blocks, not sparse-file capacities. Missing
or unreadable measurements remain unknown or partial; they do not become exact
zeroes. Completed measurements are recorded in Scan history.

Each Cleanup tab retains its listing, selection, filter, sorting and scroll
position while the app is open. Scans continue and record completed measurements
while another tab or screen is visible, or the app is hidden to the tray. Revisiting
a tab does not restart its scan. Refresh updates only that tab, keeping previous
results visible until the new inventory arrives. Cleanup makes affected listings
stale without discarding their view state; file listings refresh when revisited.

| Section | Sources and granularity | Action |
| --- | --- | --- |
| Docker | Individual images, stopped containers, unused volumes and old build cache | Native Docker operations; in-use objects are protected |
| iOS Simulator | Individual devices and removable runtimes | Native simctl delete/erase, with running-device checks |
| Build & packages | DerivedData by project; iOS/watchOS/tvOS support by version; npm/npx, pnpm, Yarn, Bun, Python, Cargo, Gradle, NuGet, Go, Homebrew, Playwright/Puppeteer, SwiftPM and other download caches | Confirm generated cleanup, or move selected paths to Trash |
| App & browser caches | User application caches and exact cache subdirectories in browser, IDE and Electron profiles | Generated cleanup for recognized cache paths; Trash for other application caches |
| AI models | Ollama by model, Hugging Face by repository/dataset, LM Studio, PyTorch, Whisper and Core ML downloads | Ollama's local API, or Trash for individual downloaded model directories/files |
| Archives & large data | Xcode release archives, device backups, large downloads/installers, logs, Maven artifacts, Rust toolchains and Node versions | Trash for user data; rustup uninstall for individual inactive/nondefault compilers |

Android devices/SDK images and virtual machines are measured and offered for
Finder inspection, with guidance to their native manager. They are not treated
as generated caches. The macOS Trash is also visible for inspection.

Each row shows its origin path or native object ID, size, file count when known,
history change and the consequence of removing it. Filtering and select-all apply
to the visible rows. Selection can be used to reveal inspect-only items in Finder;
only eligible items are passed to a cleanup confirmation.

Ollama's on-disk manifests are visible while its server is offline. Start Ollama
and refresh to enable native removal. The app uses only `127.0.0.1:11434`, checks
loaded models and the selected model digest again before removal, and leaves
shared layer management to Ollama. Per-model sizes can overlap and are not a
promise of space freed.

File cleanup checks live cwd, executable and command argument paths. Generated
cleanup also revalidates metadata revisions. Partial scans never authorize
generated removal. Rustup inventory disables automatic toolchain installation;
active/default compilers are protected. Archives, backups and downloaded models
are not assumed regenerable. Trash can be restored in Finder; its space remains
occupied until the user empties it.

Source contracts: [npm cache](https://docs.npmjs.com/cli/cache/),
[Cargo home](https://doc.rust-lang.org/cargo/guide/cargo-home.html),
[NuGet cache folders](https://learn.microsoft.com/en-us/nuget/consume-packages/managing-the-global-packages-and-cache-folders),
[Gradle caches](https://docs.gradle.org/current/userguide/directory_layout.html),
[Playwright browser binaries](https://playwright.dev/docs/browsers),
[Hugging Face cache](https://huggingface.co/docs/huggingface_hub/main/guides/manage-cache),
[Ollama loaded models](https://docs.ollama.com/api/ps).
