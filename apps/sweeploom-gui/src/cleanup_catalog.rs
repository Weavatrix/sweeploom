//! Explicit storage locations: generated caches and user-owned data stay distinct.
use super::super::Pane;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Policy {
    Cache,
    Trash,
    Inspect,
}

pub(super) struct Rule {
    pub pane: Pane,
    pub label: &'static str,
    pub path: &'static str,
    pub depth: usize,
    pub policy: Policy,
    pub consequence: &'static str,
}

macro_rules! cache {
    ($name:literal, $path:literal, $depth:literal) => {
        Rule {
            pane: Pane::Caches,
            label: $name,
            path: $path,
            depth: $depth,
            policy: Policy::Cache,
            consequence: "Regenerable; downloads or builds will be needed again",
        }
    };
}
macro_rules! model {
    ($name:literal, $path:literal, $depth:literal) => {
        Rule { pane: Pane::Models, label: $name, path: $path, depth: $depth,
            policy: Policy::Trash, consequence: "Model/downloaded data; move only selected items to Trash; offline use may stop" }
    };
}
macro_rules! data {
    ($name:literal, $path:literal, $depth:literal, $hint:literal) => {
        Rule {
            pane: Pane::Data,
            label: $name,
            path: $path,
            depth: $depth,
            policy: Policy::Trash,
            consequence: $hint,
        }
    };
}

pub(super) const RULES: &[Rule] = &[
    cache!(
        "Xcode DerivedData",
        "Library/Developer/Xcode/DerivedData",
        1
    ),
    cache!(
        "iOS device support",
        "Library/Developer/Xcode/iOS DeviceSupport",
        1
    ),
    cache!(
        "watchOS device support",
        "Library/Developer/Xcode/watchOS DeviceSupport",
        1
    ),
    cache!(
        "tvOS device support",
        "Library/Developer/Xcode/tvOS DeviceSupport",
        1
    ),
    cache!(
        "visionOS device support",
        "Library/Developer/Xcode/visionOS DeviceSupport",
        1
    ),
    cache!(
        "macOS device support",
        "Library/Developer/Xcode/macOS DeviceSupport",
        1
    ),
    cache!(
        "Xcode Organizer products",
        "Library/Developer/Xcode/Products",
        1
    ),
    cache!(
        "Xcode documentation cache",
        "Library/Developer/Xcode/DocumentationCache",
        1
    ),
    cache!("Xcode cache", "Library/Caches/com.apple.dt.Xcode", 0),
    cache!(
        "xcodebuild cache",
        "Library/Caches/com.apple.dt.xcodebuild",
        0
    ),
    cache!("Xcode downloads", "Library/Developer/DVTDownloads", 1),
    cache!("SwiftPM cache", "Library/Caches/org.swift.swiftpm", 0),
    cache!("CocoaPods cache", "Library/Caches/CocoaPods", 0),
    cache!(
        "Carthage cache",
        "Library/Caches/org.carthage.CarthageKit",
        0
    ),
    cache!("SwiftPM cache", ".cache/org.swift.swiftpm", 0),
    cache!("npm download cache", ".npm/_cacache", 0),
    cache!("npx installed packages", ".npm/_npx", 1),
    cache!("npm prebuilt binaries", ".npm/_prebuilds", 0),
    cache!("npm logs", ".npm/_logs", 0),
    cache!("pnpm store", "Library/pnpm/store", 1),
    cache!("pnpm metadata cache", "Library/Caches/pnpm", 0),
    cache!("pnpm store", ".local/share/pnpm/store", 1),
    cache!("pnpm store", ".pnpm-store", 1),
    cache!("Yarn cache", "Library/Caches/Yarn", 0),
    cache!("Yarn cache", ".cache/yarn", 0),
    cache!("Bun package cache", ".bun/install/cache", 0),
    cache!("Bun cache", "Library/Caches/bun", 0),
    cache!("pip cache", "Library/Caches/pip", 0),
    cache!("pip cache", ".cache/pip", 0),
    cache!("uv package cache", ".cache/uv", 0),
    cache!("uv package cache", "Library/Caches/uv", 0),
    cache!("Poetry cache", "Library/Caches/pypoetry/cache", 0),
    cache!("Poetry cache", ".cache/pypoetry/cache", 0),
    cache!(
        "Poetry package archives",
        "Library/Caches/pypoetry/artifacts",
        0
    ),
    cache!("Poetry package archives", ".cache/pypoetry/artifacts", 0),
    cache!("Conda download cache", "miniconda3/pkgs", 0),
    cache!("Conda download cache", "anaconda3/pkgs", 0),
    cache!("Cargo package archives", ".cargo/registry/cache", 1),
    cache!("Cargo extracted sources", ".cargo/registry/src", 1),
    cache!("Cargo registry index", ".cargo/registry/index", 1),
    cache!("Cargo Git checkout cache", ".cargo/git/checkouts", 1),
    cache!("Cargo Git database cache", ".cargo/git/db", 1),
    cache!("Rustup partial downloads", ".rustup/downloads", 0),
    cache!("Rustup temporary files", ".rustup/tmp", 0),
    cache!("sccache", "Library/Caches/Mozilla.sccache", 0),
    cache!("sccache", ".cache/sccache", 0),
    cache!("Gradle build/dependency cache", ".gradle/caches", 1),
    cache!("Gradle wrapper downloads", ".gradle/wrapper/dists", 1),
    cache!("NuGet packages", ".nuget/packages", 2),
    cache!("NuGet HTTP cache", ".local/share/NuGet/v3-cache", 0),
    cache!("NuGet plugin cache", ".local/share/NuGet/plugins-cache", 0),
    cache!("Go build cache", "Library/Caches/go-build", 0),
    cache!("Go build cache", ".cache/go-build", 0),
    cache!("Go module downloads", "go/pkg/mod/cache/download", 0),
    cache!("Homebrew downloads", "Library/Caches/Homebrew/downloads", 1),
    cache!("Homebrew API cache", "Library/Caches/Homebrew/api", 0),
    cache!("node-gyp headers", "Library/Caches/node-gyp", 1),
    cache!("node-gyp headers", ".cache/node-gyp", 1),
    cache!(
        "Playwright browser binaries",
        "Library/Caches/ms-playwright",
        1
    ),
    cache!("Playwright browser binaries", ".cache/ms-playwright", 1),
    cache!("Puppeteer browser binaries", ".cache/puppeteer", 2),
    cache!(
        "Electron build downloads",
        "Library/Caches/electron-builder",
        0
    ),
    cache!("Electron downloads", "Library/Caches/electron", 0),
    cache!("wasm-pack downloads", "Library/Caches/.wasm-pack", 0),
    cache!("Prisma engine cache", ".cache/prisma", 0),
    cache!("Android download cache", ".android/cache", 0),
    cache!("Composer cache", "Library/Caches/composer", 0),
    cache!("Composer cache", ".cache/composer", 0),
    model!("Hugging Face repository", ".cache/huggingface/hub", 1),
    model!(
        "Hugging Face dataset cache",
        ".cache/huggingface/datasets",
        1
    ),
    model!("Hugging Face Xet cache", ".cache/huggingface/xet", 1),
    model!("PyTorch checkpoints", ".cache/torch/hub/checkpoints", 1),
    model!("Whisper models", ".cache/whisper", 1),
    model!("LM Studio models", ".lmstudio/models", 2),
    model!("LM Studio models", ".cache/lm-studio/models", 2),
    model!("Core ML downloads", "Library/Caches/coreai-cache", 1),
    data!(
        "Xcode release archive",
        "Library/Developer/Xcode/Archives",
        2,
        "Release archive and dSYMs; cannot be regenerated reliably; Move to Trash"
    ),
    data!(
        "iPhone/iPad backup",
        "Library/Application Support/MobileSync/Backup",
        1,
        "Device backup; restore points will be unavailable after removal; Move to Trash"
    ),
    data!(
        "Xcode device logs",
        "Library/Developer/Xcode/iOS Device Logs",
        1,
        "Crash and console logs copied from devices; Move to Trash"
    ),
    data!(
        "Application logs",
        "Library/Logs",
        1,
        "Saved diagnostic history; Move to Trash"
    ),
    data!(
        "Maven local repository",
        ".m2/repository",
        1,
        "May include locally published artifacts; inspect before Move to Trash"
    ),
    Rule {
        pane: Pane::Data,
        label: "Android virtual device",
        path: ".android/avd",
        depth: 1,
        policy: Policy::Inspect,
        consequence: "Device data; use Android Studio Device Manager to delete a shut-down device",
    },
    Rule {
        pane: Pane::Data,
        label: "Android SDK system images",
        path: "Library/Android/sdk/system-images",
        depth: 3,
        policy: Policy::Inspect,
        consequence: "Use Android Studio SDK Manager to uninstall selected images",
    },
    Rule {
        pane: Pane::Data,
        label: "Rust toolchain",
        path: ".rustup/toolchains",
        depth: 1,
        policy: Policy::Inspect,
        consequence: "Installed compiler; use rustup toolchain uninstall after checking pinned projects",
    },
    Rule {
        pane: Pane::Data,
        label: "Node runtime",
        path: ".nvm/versions/node",
        depth: 1,
        policy: Policy::Trash,
        consequence: "Installed Node runtime; projects/default aliases may need reinstalling; Move to Trash",
    },
    Rule {
        pane: Pane::Data,
        label: "Parallels VM",
        path: "Parallels",
        depth: 1,
        policy: Policy::Inspect,
        consequence: "Virtual machine data; shut down and remove through Parallels",
    },
    Rule {
        pane: Pane::Data,
        label: "VirtualBox VM",
        path: "VirtualBox VMs",
        depth: 1,
        policy: Policy::Inspect,
        consequence: "Virtual machine data; shut down and remove through VirtualBox",
    },
    Rule {
        pane: Pane::Data,
        label: "VMware VM",
        path: "Virtual Machines.localized",
        depth: 1,
        policy: Policy::Inspect,
        consequence: "Virtual machine data; shut down and remove through VMware",
    },
];
