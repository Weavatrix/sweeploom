# Rust GUI on macOS

[README](../README.md) · [All docs](../README.md#documentation)

The [Cleanup source catalog](CLEANUP.md) describes storage categories and actions.

Build and launch the signed app from a checkout:

```sh
python3 scripts/macos-app.py
open target/SweepLoom.app
```

Quit the running GUI before updating it. The script builds with `cargo +stable`,
assembles the bundle at the same path, signs the entire bundle, and verifies it
before replacing the previous version. Use `--no-build` to package an existing
`target/release/sweeploom-gui`. Set `SWEEPLOOM_RUST_TOOLCHAIN` to select another
installed Rust toolchain.

The signing certificate must be a valid code-signing identity in the macOS
keychain. If there is exactly one, the script selects it automatically; otherwise
set `SWEEPLOOM_SIGN_IDENTITY` to its SHA-1 fingerprint or use `--identity` with its
exact name. List available identities with:

```sh
security find-identity -v -p codesigning
```

Keep the signing identity compatible across updates and the bundle identifier
`com.weavatrix.sweeploom` unchanged. The script also checks updates against the
installed app's designated requirement. It refuses an incompatible update and
never falls back to an ad-hoc signature.

macOS identifies privacy permissions using the code's designated requirement.
A Rust linker ad-hoc signature identifies a particular binary hash, so replacing
that binary can trigger folder-access prompts again. A certificate-signed bundle
has a stable identity across compatible updates. See Apple's
[TN3127: Inside Code Signing: Requirements](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements).

Migrating from an ad-hoc build to the signed app may require granting Documents,
Downloads, or Desktop access once for the new identity. Launch the `.app` bundle
for subsequent runs and updates; launching the bare binary with `cargo run` does
not use that bundle's identity. macOS owns these grants; the app does not copy,
reset, or store them. Scan history remains in the existing local data directory.

## Mac App Store edition

The App Store build is the same GUI with the `mas` cargo feature. It runs in the
App Sandbox (`com.weavatrix.sweeploom.mas`), so it reads only folders the user
grants with the folder panel; app-scoped security-scoped bookmarks restore those
grants on launch. Scans, project discovery, Explorer and Cleanup sources stay
inside the grants, and Move to Trash / Show in Finder use `NSFileManager` and
`NSWorkspace`. Features that need other apps' processes or command-line tools
(Sessions, stopping processes, Docker, simulators, the browser companion, Ollama,
rustup) are explained in the app with a link to this repository. All Objective-C
and libc calls live in `crates/sweeploom-macos`, the only crate with `unsafe`
code compiled on macOS (`sweeploom-network` has some, for Windows only). Git
state is read in pure Rust with `weavatrix-git`. The build spawns no
subprocesses, has no network entitlement, and collects no data.

Status: uploaded to App Store Connect for TestFlight testing only; not yet
approved or on sale. Planned price USD 1.99; the full edition stays free. Apple's
Standard EULA covers App Store copies, and under MPL-2.0 §3.2 Settings → About
states that the source of that build is available in this repository
([TERMS.md](../TERMS.md#5-the-mac-app-store-edition) section 5).

```sh
python3 scripts/macos-mas.py --dev                     # sandboxed, development-signed
python3 scripts/macos-mas.py --dev --native-only --fast   # quicker local iteration
ASC_ISSUER_ID=… python3 scripts/asc.py profile          # MAC_APP_STORE profile → target/
python3 scripts/macos-mas.py --store --profile target/SweepLoom_Mac_App_Store.provisionprofile
```

`--store` builds a universal binary, embeds the profile, signs with Apple
Distribution and writes `target/SweepLoom-mas.pkg` signed with the Mac installer
certificate. Use a released Xcode (`--xcode`); App Store Connect rejects beta
SDKs. `--build N` sets `CFBundleVersion` (default: UTC `YYYYMMDDHHMM`). Debug
builds also accept `SWEEPLOOM_MAS_GRANTS=/a:/b` to test grant-scoped screens in a
sandboxed bundle without the panel.
