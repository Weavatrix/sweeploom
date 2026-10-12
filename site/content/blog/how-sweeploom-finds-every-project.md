---
title: "How SweepLoom finds every project on your machine, and got about 3× faster"
date: "2026-10-11"
summary: "Project discovery used to start from the folder the app was launched in. Now it covers every developer folder with fair per-folder budgets, walks on bounded worker threads, and streams rows as they finish. It found 598 projects where the old walk found 237, in less time. How it works, the numbers, and the limits."
tags: ["performance", "projects", "disk", "rust", "release"]
---

The Projects tab can only be as good as its list of projects. Until this release, SweepLoom's desktop app discovered projects starting from the folder it was launched in. Launch it from a repository and you saw that repository's neighbourhood; launch it from anywhere else and the list changed with it. On the development Mac, that walk found **237** projects.

The new discovery finds **598** on the same machine. This post is about how SweepLoom finds them now, why it is faster despite doing more than twice the work, and where the approach deliberately stops.

## Start where projects actually live

Walking an entire home folder is slow and mostly pointless: `Library`, `AppData`, caches and media dominate it. So discovery starts from the folders developers actually keep code in. Any folder directly in your home whose name matches one of these, case-insensitively, is a root:

```rust
const ROOT_NAMES: &[&str] = &[
    "dev", "developer", "development", "projects", "code", "src", "source",
    "repos", "repositories", "work", "workspace", "workspaces",
    "github", "gitlab", "git", "sites", "desktop",
];
```

So is any folder ending in `Projects` (IntelliJ's `IdeaProjects`, PyCharm's `PycharmProjects`), plus a few nested conventions: `Documents/GitHub`, `Documents/Projects`, `Documents/Code`, `Documents/Developer`, `source/repos` and `go/src`. A loose project sitting directly in your home folder, `~/rust_test` say, becomes a root of its own if it has a project marker. If there is no `Documents/GitHub`, all of `Documents` is walked instead.

On top of that, the desktop app always includes the folder it was launched from and the current project, and merges in projects found by your last Explorer scan. Roots that are inside another root are dropped, so nothing is walked twice.

## What counts as a project

A folder is a project when it contains a marker. The list covers the ecosystems we see on developer machines:

```text
Cargo.toml  package.json  deno.json  pyproject.toml  setup.py  Pipfile
go.mod  pom.xml  build.gradle(.kts)  settings.gradle(.kts)
Package.swift  Podfile  CMakeLists.txt  Gemfile  composer.json
pubspec.yaml  mix.exs  build.zig  .git
*.xcodeproj  *.xcworkspace  *.playground  *.sln  *.csproj  *.fsproj  *.vbproj
```

This release added the Xcode, Swift, JVM and .NET markers. Bundles are opaque: an `.xcodeproj` marks its parent folder as a project, but discovery never walks into the bundle itself, and the same goes for `.app`, `.framework`, `.xcassets` and similar.

Equally important is where discovery never goes. Hidden folders, anything SweepLoom classifies as generated output, dependencies or cache (`target`, `node_modules`, `DerivedData`, `Pods`, `Carthage`, `site-packages` and friends), and OS noise such as `AppData`, `Library` and `$Recycle.Bin` are skipped by name. Symlinked folders are not followed.

## Fair budgets, so one monorepo cannot win

A naive walk over a developer folder has a failure mode: the first huge monorepo eats the whole budget and every repository after it is missing. Discovery avoids that by giving **every top-level folder of every root its own worker and its own budget**: up to 10 levels deep, 10,000 folders and 48 projects per tree, with a shared ceiling of 250,000 folders overall.

When the total still exceeds the cap (2,048 projects in the desktop app), projects are taken round-robin across trees, outer projects first:

```rust
/// Keep at most `max` projects, taking them round-robin across trees so a
/// monorepo early in the list cannot crowd out the repositories after it.
fn fair_take(found: Vec<Vec<PathBuf>>, max: usize) -> Vec<PathBuf> {
    let mut quota = vec![0_usize; found.len()];
    let mut taken = 0;
    for round in 0.. {
        let before = taken;
        for (tree, slot) in found.iter().zip(quota.iter_mut()) {
            if taken < max && tree.len() > round {
                *slot += 1;
                taken += 1;
            }
        }
        if taken == before {
            break;
        }
    }
    // ...then each tree contributes its first `quota` projects
}
```

Each folder costs one `read_dir`. Names and the directory entry's file type decide everything, so there is no `stat` per entry during discovery.

## Parallel, without an async runtime

SweepLoom has no async runtime on purpose (`cargo deny` bans `tokio`), and this kind of work does not need one. Filesystem metadata calls are blocking anyway; what helps is running a bounded number of them at once. The whole release is built on one small helper:

```rust
/// Worker count for metadata-heavy walks: the CPU count, clamped so a slow
/// disk is not flooded with concurrent `stat` calls.
pub fn walk_workers() -> usize {
    std::thread::available_parallelism()
        .map_or(4, std::num::NonZeroUsize::get)
        .clamp(2, 12)
}
```

`parallel_map` takes a list of jobs and runs them on that many scoped threads, results in input order. Discovery uses it for trees. The Project output rebuild uses it twice: once to find which workspace owns each project, once to run the per-project analyzers. Full project sizes go through the `weavatrix-scan` parallel walker with the same worker count.

Explorer needed something different, because a single enormous folder is one job. Its walk is a folder-level work queue: workers pull one folder at a time from a shared stack, each file costs exactly one `lstat`, and a folder folds its total into its parent as soon as its last child finishes. Memory stays bounded by the folders that are open, not the size of the tree, and one huge repository still keeps every worker busy.

## Streaming, and results that stay

Faster is half of it. The other half is not making you wait for the whole thing. Every scan now streams: rows appear as soon as their measurement finishes, and the table never blanks during a rescan. A refresh keeps the previous rows on screen, marked as saved, until fresh ones replace them, and project sizes measured in the last 12 hours are not re-walked at launch.

Results also survive navigation. Each Cleanup source keeps its listing, selection, filter, sort and scroll position while the app is open. Scans keep running when you switch tabs or hide the window to the tray, and revisiting a tab does not restart its scan.

## The numbers

Measured on the development Mac before and after this release. The machine was under heavy load during both (simulators, builds and agents running), so treat these as ranges from real use, not a controlled benchmark:

| Work | Before | After |
| --- | --- | --- |
| Project discovery | 3.2–7.7 s, 237 projects | 0.5–3.5 s, 598 projects |
| Project output across 95 repositories | 343–390 s | 52–109 s |
| Full project sizes | 194–201 s | 59–76 s |
| Explorer, about 3.7 million entries | about 358 s | 130–162 s |

Roughly three times faster for sizes and Explorer, more for the Project output rebuild, and discovery is both faster and finds two and a half times as many projects.

## A Windows footnote

Faster discovery surfaced a correctness bug on Windows. `canonicalize` returns paths with the `\\?\` verbatim prefix, while the inventory scan stripped it, so the same project could appear under two paths and Explorer could rescan a root it had already measured. SweepLoom now has one canonical path shape everywhere (UNC-aware, no verbatim prefix), and discovery, project lookups and Explorer agree on it.

## Limits

- **Budgets are budgets.** A tree with more than 48 projects, or deeper than 10 levels, is not exhaustively listed. That is the trade for not letting one tree starve the rest.
- **Unusual locations need a hint.** A project outside the known roots is found if it is the launch folder or current project, or if an Explorer scan covered it.
- **Symlinks are not followed**, so a repository reachable only through a symlink is not discovered through it.
- **One machine, under load.** The timings above are from one Mac on real days. Your disk, CPU count and folder shapes will move them.

The Projects tab and the rest of the desktop app are covered in the [tour](/app/#projects).
