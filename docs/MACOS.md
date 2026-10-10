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
