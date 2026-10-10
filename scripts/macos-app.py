#!/usr/bin/env python3
"""Build and sign the local macOS GUI without changing its privacy identity."""

import argparse
import os
from pathlib import Path
import plistlib
import re
import shutil
import subprocess
import sys
import tempfile
import uuid


ROOT = Path(__file__).resolve().parents[1]
BUNDLE_ID = "com.weavatrix.sweeploom"
APP = ROOT / "target" / "SweepLoom.app"


def run(*args, capture=False, env=None):
    return subprocess.run(
        [str(arg) for arg in args], check=True, text=True, env=env,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )


def identity(requested):
    output = run("/usr/bin/security", "find-identity", "-v", "-p", "codesigning", capture=True).stdout
    identities = re.findall(r'^\s*\d+\) ([A-Fa-f0-9]{40}) "([^"]+)"', output, re.MULTILINE)
    if requested:
        matches = [fingerprint for fingerprint, name in identities
                   if requested.upper() == fingerprint.upper() or requested == name]
        if len(matches) != 1:
            raise ValueError("The requested identity must match one valid code-signing certificate (SHA-1 or exact name).")
        return matches[0]
    if len(identities) != 1:
        raise ValueError("Set SWEEPLOOM_SIGN_IDENTITY to a valid code-signing certificate SHA-1. No ad-hoc fallback is used.")
    return identities[0][0]


def previous_requirement():
    if not APP.exists():
        return None
    if APP.is_symlink() or not APP.is_dir():
        raise ValueError(f"Refusing to replace an unexpected path: {APP}")
    with (APP / "Contents" / "Info.plist").open("rb") as stream:
        if plistlib.load(stream).get("CFBundleIdentifier") != BUNDLE_ID:
            raise ValueError("The existing app has a different bundle identifier; refusing to replace it.")
    details = subprocess.run(["/usr/bin/codesign", "-dv", str(APP)], capture_output=True, text=True)
    if details.returncode or "Signature=adhoc" in details.stderr:
        return None  # Migrate the old unsigned/linker-signed bundle once.
    requirement = run("/usr/bin/codesign", "-dr", "-", APP, capture=True)
    match = re.search(r"^designated => (.+)$", requirement.stdout + requirement.stderr, re.MULTILINE)
    if not match:
        raise ValueError("Could not read the existing app's designated requirement.")
    return match[1]


def version():
    manifest = (ROOT / "Cargo.toml").read_text()
    section = re.search(r"^\[workspace\.package\]\s*\n([^\[]*)", manifest, re.MULTILINE)
    match = re.search(r'^version\s*=\s*"([^"]+)"', section[1] if section else "", re.MULTILINE)
    if not match:
        raise ValueError("Missing workspace package version in Cargo.toml.")
    return match[1]


def package(signer, old_requirement):
    binary = ROOT / "target" / "release" / "sweeploom-gui"
    if not binary.is_file():
        raise ValueError(f"Missing GUI binary: {binary}")
    APP.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".sweeploom-package-", dir=APP.parent) as staging:
        candidate = Path(staging) / "SweepLoom.app"
        contents = candidate / "Contents"
        (contents / "MacOS").mkdir(parents=True)
        shutil.copy2(binary, contents / "MacOS" / "sweeploom-gui")
        with (contents / "Info.plist").open("wb") as stream:
            plistlib.dump({
                "CFBundleDisplayName": "SweepLoom",
                "CFBundleExecutable": "sweeploom-gui",
                "CFBundleIdentifier": BUNDLE_ID,
                "CFBundleName": "SweepLoom",
                "CFBundlePackageType": "APPL",
                "CFBundleShortVersionString": version(),
                "CFBundleVersion": "1",
                "LSMinimumSystemVersion": "12.0",
                "NSHighResolutionCapable": True,
                "NSDocumentsFolderUsageDescription": "Scan folder sizes and review files you choose to clean in Documents.",
                "NSDownloadsFolderUsageDescription": "Scan downloads and review files you choose to move to Trash.",
                "NSDesktopFolderUsageDescription": "Scan folder sizes and review files you choose to clean on the Desktop.",
            }, stream)
        # Sign the whole bundle, binding its plist and resources to the signature.
        run("/usr/bin/codesign", "--force", "--sign", signer,
            "--identifier", BUNDLE_ID, "--timestamp=none", candidate)
        run("/usr/bin/codesign", "--verify", "--strict", "--verbose=2", candidate)
        if old_requirement:
            # A new certificate/channel must still satisfy the installed app's identity.
            run("/usr/bin/codesign", "--verify", "--strict", "--test-requirement",
                "=" + old_requirement, candidate)
        # Keep the backup outside staging so even a failed rollback preserves it.
        backup = APP.with_name(f".SweepLoom.previous-{uuid.uuid4().hex}.app")
        if APP.exists():
            APP.rename(backup)
        try:
            candidate.rename(APP)
        except OSError:
            if backup.exists():
                backup.rename(APP)
            raise
        if backup.exists():
            shutil.rmtree(backup)
    print(f"Signed app: {APP}\nLaunch this bundle after each update to keep its macOS privacy identity.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--no-build", action="store_true", help="Package the existing release binary.")
    parser.add_argument("--identity", default=os.environ.get("SWEEPLOOM_SIGN_IDENTITY"),
                        help="Valid code-signing certificate SHA-1 or exact name.")
    args = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("This script requires macOS.")
    signer = identity(args.identity)
    old_requirement = previous_requirement()
    if not args.no_build:
        env = os.environ.copy()
        env.setdefault("CARGO_INCREMENTAL", "0")
        env.setdefault("CARGO_PROFILE_RELEASE_LTO", "false")
        env.setdefault("CARGO_PROFILE_RELEASE_CODEGEN_UNITS", "16")
        toolchain = env.get("SWEEPLOOM_RUST_TOOLCHAIN", "stable")
        run("cargo", "+" + toolchain, "build", "--release", "--locked",
            "--manifest-path", ROOT / "Cargo.toml", "--target-dir", ROOT / "target",
            "-p", "sweeploom-gui", "--bin", "sweeploom-gui", env=env)
    package(signer, old_requirement)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"macOS packaging failed: {error}", file=sys.stderr)
        sys.exit(1)
