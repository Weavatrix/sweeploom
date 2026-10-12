#!/usr/bin/env python3
"""Build the sandboxed Mac App Store edition of SweepLoom (cargo feature `mas`).

  --dev    sign with the Apple Development identity; the App Sandbox is still
           enforced, so the bundle runs locally exactly as reviewers see it.
  --store  embed the provisioning profile, sign with Apple Distribution and
           wrap the app in an installer package for App Store Connect.

Nothing is uploaded. Validate/upload target/SweepLoom-mas.pkg yourself.
"""

import argparse
import datetime
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TEAM = "XMS5ZC28UJ"
BUNDLE_ID = "com.weavatrix.sweeploom.mas"
NAME = "SweepLoom"
EXECUTABLE = "sweeploom-gui"
MIN_MACOS = "12.0"
TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin")
OUT = ROOT / "target" / "mas"
APP = OUT / f"{NAME}.app"
PKG = ROOT / "target" / f"{NAME}-mas.pkg"
DEV_IDENTITY = "Apple Development: sergii.ziborov@gmail.com (VXSL3A36W6)"
STORE_IDENTITY = f"Apple Distribution: Serhii Ziborov ({TEAM})"
INSTALLER_IDENTITY = f"3rd Party Mac Developer Installer: Serhii Ziborov ({TEAM})"
SANDBOX = {
    "com.apple.security.app-sandbox": True,
    "com.apple.security.files.user-selected.read-write": True,
    "com.apple.security.files.bookmarks.app-scope": True,
}
# Restricted: only valid together with the matching provisioning profile.
STORE_ONLY = {
    "com.apple.application-identifier": f"{TEAM}.{BUNDLE_ID}",
    "com.apple.developer.team-identifier": TEAM,
}


def run(*args, capture=False, env=None):
    result = subprocess.run([str(arg) for arg in args], check=True, text=True, env=env,
                            stdout=subprocess.PIPE if capture else None)
    return result.stdout.strip() if capture else ""


def version():
    manifest = (ROOT / "Cargo.toml").read_text()
    section = re.search(r"^\[workspace\.package\]\s*\n([^\[]*)", manifest, re.MULTILINE)
    match = re.search(r'^version\s*=\s*"([^"]+)"', section[1] if section else "", re.MULTILINE)
    if not match:
        raise ValueError("Missing workspace package version in Cargo.toml.")
    return match[1]


def xcode_facts(developer_dir):
    """DT* keys App Store validation reads to check the Xcode/SDK that built the app."""
    env = dict(os.environ, DEVELOPER_DIR=str(developer_dir))
    xcode = run("xcodebuild", "-version", capture=True, env=env)
    build = xcode.split("Build version")[-1].strip()
    sdk = run("xcrun", "--sdk", "macosx", "--show-sdk-version", capture=True, env=env)
    sdk_build = run("xcrun", "--sdk", "macosx", "--show-sdk-build-version", capture=True, env=env)
    # Apple beta builds carry a 5xxx build number, e.g. 27A5228h (release: 27A266a).
    if any(re.fullmatch(r"\d+[A-Z]5\d{3}[a-z]?", item) for item in (build, sdk_build)):
        print(f"warning: Xcode {build} / SDK {sdk_build} is a beta; App Store Connect rejects "
              "apps built with beta SDKs. Pass --xcode with a released Xcode.", file=sys.stderr)
    with (Path(developer_dir).parent / "Info.plist").open("rb") as stream:
        dt_xcode = plistlib.load(stream).get("DTXcode", "")
    return {
        "BuildMachineOSBuild": run("sw_vers", "-buildVersion", capture=True),
        "DTCompiler": "com.apple.compilers.llvm.clang.1_0",
        "DTPlatformBuild": sdk_build,
        "DTPlatformName": "macosx",
        "DTPlatformVersion": sdk,
        "DTSDKBuild": sdk_build,
        "DTSDKName": f"macosx{sdk}",
        "DTXcode": dt_xcode,
        "DTXcodeBuild": build,
    }


def ensure_targets(toolchain, targets):
    installed = run("rustup", "+" + toolchain, "target", "list", "--installed", capture=True).split()
    missing = [target for target in targets if target not in installed]
    if missing:
        run("rustup", "target", "add", "--toolchain", toolchain, *missing)


def build(args, targets, developer_dir):
    toolchain = os.environ.get("SWEEPLOOM_RUST_TOOLCHAIN", "stable")
    ensure_targets(toolchain, targets)
    env = dict(os.environ, MACOSX_DEPLOYMENT_TARGET=MIN_MACOS, DEVELOPER_DIR=str(developer_dir))
    if args.fast:
        env.update(CARGO_PROFILE_RELEASE_LTO="false", CARGO_PROFILE_RELEASE_CODEGEN_UNITS="16")
    binaries = []
    for target in targets:
        run("cargo", "+" + toolchain, "build", "--release", "--locked",
            "--manifest-path", ROOT / "Cargo.toml", "--target-dir", ROOT / "target",
            "-p", "sweeploom-gui", "--bin", EXECUTABLE, "--features", "mas",
            "--target", target, env=env)
        binaries.append(ROOT / "target" / target / "release" / EXECUTABLE)
    return binaries


def host_binary(binaries):
    arch = "aarch64" if platform.machine() == "arm64" else "x86_64"
    return next((item for item in binaries if f"/{arch}-apple-darwin/" in str(item)), binaries[0])


def icon(binary, resources, staging):
    iconset = Path(staging) / "AppIcon.iconset"
    run(binary, "--write-iconset", iconset)
    run("iconutil", "--convert", "icns", "--output", resources / "AppIcon.icns", iconset)


def info_plist(build_number, developer_dir):
    info = {
        "CFBundleDevelopmentRegion": "en",
        "CFBundleDisplayName": NAME,
        "CFBundleExecutable": EXECUTABLE,
        "CFBundleIconFile": "AppIcon",
        "CFBundleIdentifier": BUNDLE_ID,
        "CFBundleInfoDictionaryVersion": "6.0",
        "CFBundleName": NAME,
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version(),
        "CFBundleSupportedPlatforms": ["MacOSX"],
        "CFBundleVersion": build_number,
        "ITSAppUsesNonExemptEncryption": False,
        "LSApplicationCategoryType": "public.app-category.developer-tools",
        "LSMinimumSystemVersion": MIN_MACOS,
        "NSHighResolutionCapable": True,
        "NSHumanReadableCopyright": "© 2026 Weavatrix",
        "NSSupportsAutomaticGraphicsSwitching": True,
    }
    info.update(xcode_facts(developer_dir))
    return info


def check_profile(profile):
    decoded = subprocess.run(["/usr/bin/security", "cms", "-D", "-i", str(profile)],
                             check=True, capture_output=True).stdout
    data = plistlib.loads(decoded)
    granted = data.get("Entitlements", {}).get("com.apple.application-identifier")
    if granted != STORE_ONLY["com.apple.application-identifier"]:
        raise ValueError(f"Profile is for {granted!r}, not {TEAM}.{BUNDLE_ID}.")
    if "OSX" not in data.get("Platform", []):
        raise ValueError("Profile is not a macOS profile.")
    expires = data.get("ExpirationDate")
    if expires and expires < datetime.datetime.now(expires.tzinfo):
        raise ValueError(f"Profile expired on {expires}.")


def sign(app, identity, entitlements, staging, store):
    path = Path(staging) / "entitlements.plist"
    with path.open("wb") as stream:
        plistlib.dump(entitlements, stream)
    timestamp = [] if store else ["--timestamp=none"]
    run("/usr/bin/codesign", "--force", "--sign", identity, *timestamp,
        "--entitlements", path, "--identifier", BUNDLE_ID, app)
    run("/usr/bin/codesign", "--verify", "--strict", "--verbose=2", app)


def package(args, binaries, developer_dir):
    if APP.is_symlink() or (APP.exists() and not APP.is_dir()):
        raise ValueError(f"Refusing to replace an unexpected path: {APP}")
    OUT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".sweeploom-mas-", dir=OUT) as staging:
        candidate = Path(staging) / f"{NAME}.app"
        contents = candidate / "Contents"
        (contents / "MacOS").mkdir(parents=True)
        (contents / "Resources").mkdir()
        executable = contents / "MacOS" / EXECUTABLE
        if len(binaries) > 1:
            run("lipo", "-create", *binaries, "-output", executable)
        else:
            shutil.copy2(binaries[0], executable)
        icon(host_binary(binaries), contents / "Resources", staging)
        with (contents / "Info.plist").open("wb") as stream:
            plistlib.dump(info_plist(args.build, developer_dir), stream)
        (contents / "PkgInfo").write_text("APPL????")
        entitlements = dict(SANDBOX)
        if args.store:
            shutil.copy2(args.profile, contents / "embedded.provisionprofile")
            entitlements.update(STORE_ONLY)
        sign(candidate, STORE_IDENTITY if args.store else DEV_IDENTITY, entitlements,
             staging, args.store)
        if APP.exists():
            shutil.rmtree(APP)
        candidate.rename(APP)
    print(f"Signed {'App Store' if args.store else 'development'} app: {APP}")
    print(run("lipo", "-archs", APP / "Contents" / "MacOS" / EXECUTABLE, capture=True))


def installer():
    PKG.unlink(missing_ok=True)
    run("productbuild", "--component", APP, "/Applications", "--sign", INSTALLER_IDENTITY, PKG)
    run("pkgutil", "--check-signature", PKG)
    print(f"Installer package: {PKG}\nValidate and upload it with Transporter or altool.")


def default_xcode():
    release = Path("/Applications/Xcode.app/Contents/Developer")
    if release.is_dir():
        return release
    return Path(run("xcode-select", "--print-path", capture=True))


def parse():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--dev", action="store_true", help="Development signing, sandbox on.")
    mode.add_argument("--store", action="store_true", help="Distribution signing + .pkg.")
    parser.add_argument("--profile", type=Path, help="Mac App Store .provisionprofile (--store).")
    parser.add_argument("--build", default=datetime.datetime.now(datetime.timezone.utc)
                        .strftime("%Y%m%d%H%M"), help="CFBundleVersion (default: UTC timestamp).")
    parser.add_argument("--xcode", type=Path, help="Xcode.app or its Developer directory "
                        "(default: /Applications/Xcode.app, else xcode-select).")
    parser.add_argument("--native-only", action="store_true",
                        help="--dev only: build just this Mac's architecture.")
    parser.add_argument("--fast", action="store_true", help="--dev only: skip fat LTO.")
    parser.add_argument("--no-build", action="store_true", help="Reuse the release binaries.")
    args = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("This script requires macOS.")
    if args.store and (args.native_only or args.fast):
        parser.error("--store always builds the universal, fully optimized binary.")
    if args.store and (not args.profile or not args.profile.is_file()):
        parser.error("--store needs --profile path/to/profile.provisionprofile.")
    if not re.fullmatch(r"\d+(\.\d+){0,2}", args.build):
        parser.error("--build must be a number such as 202610121530 or 1.2.3.")
    return args


def main():
    args = parse()
    if args.store:
        check_profile(args.profile)  # fail before a long universal build
    developer_dir = args.xcode or default_xcode()
    if developer_dir.suffix == ".app":
        developer_dir = developer_dir / "Contents" / "Developer"
    targets = TARGETS
    if args.native_only:
        targets = tuple(t for t in TARGETS if t.startswith(
            "aarch64" if platform.machine() == "arm64" else "x86_64"))
    binaries = ([ROOT / "target" / t / "release" / EXECUTABLE for t in targets]
                if args.no_build else build(args, targets, developer_dir))
    missing = [str(item) for item in binaries if not item.is_file()]
    if missing:
        raise ValueError(f"Missing release binaries: {', '.join(missing)}")
    package(args, binaries, developer_dir)
    if args.store:
        installer()


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"Mac App Store packaging failed: {error}", file=sys.stderr)
        sys.exit(1)
