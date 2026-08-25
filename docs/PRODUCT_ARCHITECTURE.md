# SweepLoom — полный продуктовый, архитектурный и implementation plan

**Product:** **SweepLoom**  
**Primary domain:** **sweeploom.com**  
**Domain status:** зарегистрирован через Cloudflare  
**Brand relationship:** **SweepLoom — by Weavatrix**  
**Дата обновления:** 2026-08-25  
**Primary language:** Rust  
**Desktop UI:** `egui + eframe + egui_extras`  
**Platforms:** Windows, macOS, Linux  
**Core Weavatrix dependencies:** `weavatrix-scan`, `weavatrix-git`, позже optional `weavatrix-search`  
**Главный принцип:** local-first, deterministic, evidence-driven, no LLM required for destructive actions.

---

# 1. Что теперь строим

После повторного анализа идея должна быть шире обычного disk cleaner.

## 1.1. Новый продуктовый scope

SweepLoom — это **Workspace Resource Cleaner** с двумя равноправными направлениями:

```text
STORAGE
  projects
  build artifacts
  dependency trees
  package/tool caches
  AI histories
  temporary files
  logs / crash dumps
  large files
  Downloads
  Docker / simulators / model stores
  ordinary app/system caches

LIVE RESOURCES
  forgotten terminals
  Claude Code sessions
  Codex sessions
  MCP servers
  Node/Bun dev servers
  Python/Java/.NET/Go processes
  Cargo/rustc/build processes
  browser tabs/renderers
  memory-heavy background trees
  CPU-heavy forgotten work
  network-active background sessions
```

Главное обещание продукта:

> **SweepLoom understands what is consuming your workstation, whether it is still useful, and how safely you can reclaim it.**

Не позиционировать продукт как:

```text
RAM booster
PC optimizer
registry cleaner
```

И не обещать магическое «освобождение RAM» через purge OS cache.

Для памяти реальная ценность:

> найти забытые процессы и сессии, правильно сгруппировать их и безопасно завершить то, что пользователь больше не использует.

---

## 1.2. Почему Live Sessions должны быть P0

При AI-assisted development одна забытая сессия может оставить целое дерево:

```text
terminal
└─ claude
   ├─ mcp-server
   ├─ shell
   │  └─ node
   │     └─ vite
   ├─ playwright
   └─ helper workers
```

Или:

```text
terminal
└─ codex
   └─ cargo
      ├─ rustc
      └─ build-script
```

Если пользователь забыл несколько таких terminals, Task Manager показывает десятки отдельных строк:

```text
node.exe
node.exe
node.exe
claude.exe
cmd.exe
python.exe
...
```

SweepLoom должен вместо этого показывать:

```text
Kablay
Claude Code session
  12 processes
  8.4 GB RAM
  0.2% CPU now
  1.7 GB network observed
  3 listening ports
  no project activity for 3d 6h

Recommendation:
  Forgotten session candidate

[Keep] [Terminate session]
```

Это значительно понятнее обычного process manager.

---

## 1.3. Product pillars

### Pillar A — Storage Intelligence

Не просто размер файлов.

Нужно понимать:

```text
what
owner
project
activity
rebuild cost
risk
physical reclaim
safe action
```

### Pillar B — Project Heat

Отдельно:

```text
Source Heat
Artifact Heat
Git activity
Agent activity
Live process activity
```

### Pillar C — Live Session Intelligence

Понимать логические developer sessions поверх обычного OS process tree.

### Pillar D — Browser Memory Intelligence

Не убивать браузер целиком, а помогать безопасно разгружать забытые tabs.

### Pillar E — Folder Inspector

Любую папку можно открыть и понять:

```text
что именно занимает место
какие подпапки самые большие
какие данные generated/cache/source/user
что старое
что активно
что reclaimable
```

### Pillar F — Evidence before action

Любое удаление/termination:

```text
recommend
explain
review
revalidate
execute
verify
receipt
```

---

# 1.4. Новая главная навигация UI

Рекомендация:

```text
Overview
Storage
Sessions
Projects
Browser
Explorer
AI
Rules
History
Settings
```

Не прятать Sessions в Settings или Advanced.

---

# 1.5. Overview

Верхние cards:

```text
DISK
Used                 1.18 TB
Reclaimable          94 GB
Safe now             51 GB

MEMORY
Used                 32 / 48 GB
Reclaimable sessions 11.7 GB
Stale sessions       7

CPU
Background load      24%
Likely stale         13%

BROWSER
Open tabs            143
Cold tabs            62
Discard candidates   37
```

Дальше:

```text
Top opportunities

1. Forgotten Claude session        7.2 GB RAM
2. Cold Rust targets              18.8 GB disk
3. Browser tabs not visited 14d    3.1 GB estimated browser pressure
4. Old node_modules               11.3 GB disk
5. Forgotten Vite servers          1.9 GB RAM + CPU
```

---

# 1.6. Unified pressure actions

Отдельные actions:

```text
Free Disk Space
Free Memory
Reduce CPU Load
Quiet Workstation
```

### Free Disk Space

```text
Need 50 GB
```

Storage planner выбирает минимально болезненный CleanPlan.

### Free Memory

```text
Need 12 GB RAM
```

Session planner предлагает:

```text
[x] forgotten Claude tree    6.8 GB
[x] cold Node servers        3.1 GB
[x] stale Python workers     1.4 GB
[x] browser discard set      1.2 GB est.
```

### Reduce CPU Load

Сортирует stale candidates по observed CPU burn.

### Quiet Workstation

Профиль:

```text
protect current project
protect active browser/audio
stop forgotten dev sessions
discard safe cold tabs
leave system services untouched
```

Это может стать одной из наиболее заметных SweepLoom features.

---

# 1.7. Process Explorer — raw OS tree

SweepLoom должен иметь обычный process view, потому что без него пользователь не сможет проверить рекомендации.

Columns:

```text
Process
PID
Parent PID
CPU now
CPU history
RAM RSS
Virtual memory
Disk read
Disk write
Network Rx
Network Tx
Connections
Uptime
Observed idle
Project
Session
Risk
```

Но raw process view — только базовый слой.

---

# 1.8. Session Tree — слой поверх Process Tree

Это принципиально важное отличие.

## Raw process tree

Основан на:

```text
PID
PPID
process creation/start time
```

## Logical session tree

Использует дополнительные evidence:

```text
process ancestry
process group / session id
controlling terminal
terminal-specific inherited session id
cwd
project root
command line
agent executable
listening ports
container/cgroup
observed ancestry
child spawn history
```

Пример:

```text
Project: SweepLoom
Terminal: Windows Terminal / tab #4
Agent: Claude Code

claude
├─ cmd/powershell
├─ node mcp-server
├─ node vite
├─ playwright-mcp
└─ python helper

RAM:     6.4 GB
CPU:     0.1% now
Network: idle 2h
Disk I/O: idle 5h
Session age: 4d 3h
Observed meaningful activity: 19h ago
```

---

# 1.9. Session grouping levels

Группировка должна быть hierarchical.

```text
Machine
└─ Project
   └─ Terminal / Agent Session
      └─ Service group
         └─ Processes
```

В UI пользователь может переключить:

```text
Group by Session
Group by Project
Group by App
Raw Process Tree
```

---

# 1.10. Project attribution для процессов

Приоритет evidence:

```text
1. process.cwd inside known project
2. ancestor cwd inside project
3. command line contains exact canonical project path
4. observed process spawned from already-attributed session
5. known build/output path belongs to project
6. listening server launched from project
```

Нельзя угадывать project только по process name.

Каждая attribution имеет:

```text
Exact
Strong
Weak
Unknown
```

---

# 1.11. Terminal attribution

### Windows

Использовать best-effort evidence:

- process ancestry;
- console/terminal host;
- inherited environment where accessible;
- Windows Terminal session variables where present;
- shell cwd;
- observed spawn relationship.

### Linux/macOS

Использовать:

- process session id;
- process group;
- controlling TTY;
- shell ancestry;
- terminal-specific environment ids where available;
- cwd.

Если точной attribution нет:

```text
Unknown terminal
```

а не ложная группировка.

---

# 1.12. Agent detectors

Rule-based signatures:

```text
Claude Code
Codex
Cursor agent helpers
OpenCode
Gemini CLI
Grok Bot / MCP-based agents
generic MCP servers
```

Detector contract:

```rust
pub trait SessionDetector {
    fn classify(&self, process: &ProcessSnapshot) -> Option<SessionEvidence>;
}
```

Никакого cloud lookup.

---

# 1.13. Dev-service detectors

Распознавать:

```text
vite
next
nuxt
webpack
parcel
bun dev
node server
tsx
ts-node
python dev servers
uvicorn
gunicorn
django runserver
flask
cargo run
cargo watch
rustc/build scripts
dotnet watch
gradle daemons
java language servers
go run
gopls
playwright
MCP servers
```

Особенно полезно показывать listening ports:

```text
localhost:3000
localhost:5173
localhost:9229
```

---

# 1.14. Live process metrics

Для baseline использовать `sysinfo`.

На момент исследования `sysinfo 0.39.6` предоставляет:

```text
RSS memory
virtual memory
CPU usage
accumulated CPU time
process start time
run time
cwd
command/environment where permitted
disk/I/O usage
```

Важно:

- CPU нужно измерять по delta и минимум после двух refresh;
- virtual memory на macOS нельзя использовать как основной показатель resource pressure;
- primary memory metric в UI — RSS/physical resident memory.

---

# 1.15. Resource history

Task Manager показывает live data; SweepLoom должен сохранять **короткую bounded history**.

Не обещать историю до запуска SweepLoom.

Использовать термин:

```text
Observed history
Observed idle since
```

а не:

```text
Last used
```

если это невозможно доказать.

---

## Suggested retention

Tiered ring buffers:

```text
1 sec samples   -> 10 minutes
5 sec samples   -> 2 hours
1 min samples   -> 24 hours
10 min samples  -> 7 days optional persistent aggregate
```

Метрики:

```text
CPU
RSS
disk read/write delta
network rx/tx delta
connection count
child count
```

Для Session суммировать children.

---

# 1.16. Process activity model

Нельзя считать процесс stale только потому, что CPU сейчас 0%.

Сигналы:

```text
CPU delta
disk I/O delta
network I/O delta
new/closed sockets
child spawn/exit
recent file writes in project
project source heat
terminal/agent activity evidence
```

Classification:

```text
ACTIVE
BACKGROUND_ACTIVE
IDLE
SLEEPING_MEMORY_HEAVY
RUNAWAY_CPU
NETWORK_ACTIVE
LIKELY_FORGOTTEN
ORPHAN_CANDIDATE
UNKNOWN
```

---

# 1.17. Forgotten Session score

Recommendation может учитывать:

```text
session age
observed idle duration
project heat
network inactivity
disk inactivity
CPU inactivity
parent/terminal state
agent root state
number of listening ports
memory held
user policy
```

Но hard safety отдельно.

Пример deterministic policy:

```text
if system_process:
    BLOCK

if current_project:
    KEEP

if session has recent project source activity:
    KEEP

if network active:
    REVIEW

if CPU active:
    REVIEW

if idle > 2h and project cold > 3d and memory > 1 GB:
    RECOMMEND

if orphan + idle > 30m + known dev helper:
    STRONGLY_RECOMMEND
```

---

# 1.18. Orphan detection — не использовать только PPID=1

Есть существующие tools, которые считают orphan dev process через `PPID=1`.

Это полезный signal, но недостаточный.

Почему:

- system daemons тоже могут иметь системного parent;
- процесс может быть reparented намеренно;
- некоторые dev tools daemonize нормально;
- забытый dev server может всё ещё иметь living parent;
- Windows semantics отличаются.

SweepLoom:

```text
PPID/orphan evidence
+
project heat
+
resource history
+
process type
+
terminal/session evidence
```

---

# 1.19. Process actions

На row:

```text
Keep
Protect
Open folder
Show project
Show connections
Show command
Terminate process
Terminate subtree
Terminate logical session
```

Для recognized dev server:

```text
Stop server
```

Для agent:

```text
Terminate agent session
```

---

# 1.20. Termination safety ladder

Никакого мгновенного `kill -9` по умолчанию.

```text
1. Graceful app/session action if supported
2. Interrupt / graceful termination signal
3. Wait with timeout
4. Terminate process
5. Force kill only after explicit escalation
```

OS-specific adapter:

```rust
pub trait ProcessControlBackend {
    fn request_graceful_stop(...);
    fn terminate(...);
    fn force_kill(...);
    fn terminate_tree(...);
}
```

---

# 1.21. Session termination semantics

В confirm dialog:

```text
Terminate "Kablay / Claude Code"?

12 processes
8.4 GB RAM currently resident
2 listening ports
Project source last modified 4d ago
Git worktree: clean
Observed session idle: 9h

Will terminate:
  claude
  shell
  vite
  mcp-server
  playwright helper

[Cancel]
[Terminate gracefully]
[Force terminate]
```

Если Git dirty:

```text
WARNING
Project has uncommitted changes.
```

Это не обязательно означает, что процесс нельзя остановить, но auto-selection должна быть значительно консервативнее.

---

# 1.22. System process protection

Никогда автоматически не рекомендовать:

```text
kernel/system critical processes
init/systemd/launchd equivalents
desktop shell
security software
drivers
service managers
unknown elevated OS process
```

User может открыть details, но terminate action должен быть disabled/advanced для critical entries.

---

# 1.23. Command-line privacy

Process command lines иногда содержат:

```text
tokens
API keys
passwords
connection strings
signed URLs
```

Перед UI/log/receipt:

```text
redact known secret flags
redact URI credentials
redact environment secrets
```

Local app тоже не должна писать секреты в diagnostics.

---

# 1.24. Network Explorer

Это дополнительный, но полезный слой process/session UI.

На process/session:

```text
Listening ports
Established connections
Remote address
Remote port
Protocol
Connection state
Observed Rx/s
Observed Tx/s
Observed total
Last network activity
```

Не делать packet capture.

Не читать payload.

---

# 1.25. Per-process network: platform reality

`sysinfo` даёт interface-level network data, но не полноценные cross-platform per-process network byte counters.

Поэтому architecture:

```text
sweeploom-platform
  windows/network
  linux/network
  macos/network
```

с feature/capability reporting.

---

# 1.26. Windows network backend

### Connections

Использовать Windows IP Helper APIs:

```text
GetExtendedTcpTable
GetExtendedUdpTable
```

для endpoint/PID attribution.

### Byte history

Использовать ETW TCP/IP events.

Windows ETW TCP/IP send/receive events несут:

```text
PID
size
source/destination addresses
ports
```

Это позволяет SweepLoom строить observed per-process/session byte history.

### Privileges

Если tracing недоступен:

```text
connections available
exact network rate unavailable
```

UI должен показывать capability, не нули как будто activity отсутствует.

---

# 1.27. Linux network backend

Baseline:

```text
/proc/<pid>/fd
socket:[inode]
```

связать socket inode с `/proc/net/*`.

Это даёт:

```text
PID -> sockets/connections/listeners
```

### Exact byte accounting

Optional advanced backend:

```text
eBPF / cgroup / socket tracing
```

Если privileges отсутствуют:

```text
connection metadata only
```

Не делать root обязательным для основного продукта.

---

# 1.28. macOS network backend

Сделать отдельный best-effort native adapter.

MVP requirement:

```text
process connections/listeners where accessible
```

Exact byte-rate attribution может быть capability-gated.

Не использовать private undocumented API как обязательное основание продукта.

---

# 1.29. Network history UX

Для session:

```text
Network
  now           0 B/s
  last active   3h 18m ago
  observed Rx   488 MB
  observed Tx   71 MB

Connections
  localhost:5173 LISTEN
  api.github.com:443 CLOSED 3h ago
```

Историю удалённых destinations хранить bounded.

---

# 1.30. CPU history

Сильная дополнительная feature.

Показывать:

```text
CPU now
5m average
1h average
peak
accumulated CPU
```

Пример:

```text
Forgotten test runner
RAM: 1.1 GB
CPU now: 37%
CPU average 1h: 29%
Project cold: 8d
```

Это значительно более важный cleanup candidate, чем sleeping process на 100 MB.

---

# 1.31. Disk I/O history процесса

`sysinfo::Process::disk_usage()` даёт cumulative/delta I/O.

Показывать:

```text
read/s
write/s
observed read
observed write
last disk activity
```

На Windows документировать, что OS counter может отражать broader process I/O semantics.

---

# 1.32. Memory history

Primary:

```text
RSS now
RSS 5m avg
RSS peak observed
session RSS total
```

Не складывать shared memory как будто вся она uniquely reclaimable.

Поэтому:

```text
current RSS sum
estimated reclaimable RAM
```

должны быть разными величинами.

---

# 1.33. Reclaimable RAM estimate

Убийство процесса с RSS 5 GB не гарантирует +5 GB в `available`.

Причины:

- shared pages;
- file-backed pages;
- allocator/OS behavior;
- browser shared processes.

Поэтому до action:

```text
Estimated session RSS: 8.4 GB
```

После:

```text
Available RAM gained: +7.1 GB
```

Фактический post-action delta — честнее.

---

# 1.34. Browser Session Manager

Браузер должен быть отдельным domain, не просто десятком renderer processes.

Без browser companion SweepLoom видит:

```text
browser process tree
total RAM
CPU
network
```

Но не должен обещать точную tab attribution.

---

# 1.35. Optional browser companion

WebExtension для:

```text
Chrome / Chromium family
Firefox
```

Связь desktop app ↔ extension:

```text
WebExtension Native Messaging
```

Плюсы:

- local;
- не нужен localhost HTTP server;
- не нужен cloud;
- стандартная browser integration model.

---

# 1.36. Browser tab evidence

Extension может передавать:

```text
tab id
window id
title
URL
lastAccessed
active
pinned
audible
muted
discarded
group
incognito flag where permitted
```

`lastAccessed` особенно важен.

Пользователь просит не возраст вкладки, а:

> когда он последний раз реально на неё заходил.

Именно это использовать в recommendation.

---

# 1.37. Browser tab heat

```text
ACTIVE       current tab
HOT          accessed < 1h
WARM         < 1d
COOL         1–3d
COLD         3–14d
DORMANT      > 14d
ARCHIVAL     > 60d
```

Defaults configurable.

---

# 1.38. Browser protections

По умолчанию protected:

```text
active
pinned
audible
recently accessed
explicit Keep
known important domains optionally
tabs that browser refuses to discard
```

Не auto-close private/incognito tabs.

---

# 1.39. Preferred browser action: Discard

Порядок:

```text
Keep
Discard
Bookmark + Close
Close
```

### Discard

Browser unloads tab contents from RAM, но tab остаётся в tab strip и reloads при возврате.

Это идеальное default action для memory pressure.

### Почему лучше Close

Пользователь ничего не теряет из tab list.

---

# 1.40. Bookmark + Close

Это должен быть transactional action:

```text
1. create bookmark
2. receive success/bookmark id
3. only then close tab
4. if close fails, retain bookmark and report
```

Destination:

```text
SweepLoom / Later / YYYY-MM-DD
```

Options:

```text
Bookmark + Close
Bookmark + Discard
Move to Later
Export tabs to Markdown
```

`Export tabs to Markdown` можно сделать позже.

---

# 1.41. Browser safe-close semantics

Нельзя гарантировать сохранение любой unsaved web form.

Поэтому:

- first choice = `Discard`;
- browser itself may refuse discard when page has unload protection;
- Close остаётся explicit action;
- никогда не bypass native beforeunload prompt;
- active/pinned/audible protected.

---

# 1.42. Browser memory attribution limitations

Chromium multi-process model:

- один renderer может обслуживать больше одной task;
- processes могут быть shared;
- extensions/utility/GPU processes не являются одной вкладкой.

В Chrome существует `chrome.processes` API с CPU/network/privateMemory/tasks, но он помечен как **Dev channel**.

Поэтому:

```text
base production design does NOT depend on chrome.processes
```

Если API доступен:

```text
Enhanced Metrics
```

Иначе:

```text
tab activity from extension
browser process resource totals from native app
```

---

# 1.43. Browser recommendations

Пример:

```text
62 tabs not visited for > 14 days

Recommended:
[x] Discard 37 tabs
[ ] Bookmark + close 18 tabs
[-] Keep 7 pinned/audible tabs

Expected memory relief:
browser estimate + post-action measured delta
```

Не обещать точную memory savings per tab без evidence.

---

# 1.44. Folder Inspector

Любой disk candidate/path должен открываться в Explorer.

Не только:

```text
folder = 43 GB
```

А:

```text
folder = 43 GB

target/
  debug/       26 GB
  release/      9 GB
  criterion/    4 GB
  tmp/          2 GB
  other/        2 GB
```

---

# 1.45. Folder Inspector views

### Tree

Классическое дерево.

### Treemap

Как WizTree/TreeSize/WinDirStat.

### Extensions

```text
.dll  18 GB
.pdb   7 GB
.rlib  5 GB
...
```

### Activity

```text
< 1 day
1–3 days
3–7 days
7–30 days
30d+
```

### Category

```text
Source
Generated
Dependencies
Cache
User Data
Unknown
```

---

# 1.46. Folder Inspector metrics

Для каждого directory node:

```rust
DirectoryStats {
    logical_bytes,
    allocated_bytes,
    estimated_reclaimable_bytes,
    files,
    directories,
    newest_mtime,
    oldest_mtime,
    newest_source_mtime,
    newest_generated_mtime,
    hardlink_dedup_bytes,
    category_breakdown,
    extension_breakdown,
}
```

---

# 1.47. Physical vs logical folder size

Это обязательная feature после анализа конкурентов.

Показывать:

```text
Logical size
Allocated size
Deduped physical estimate
```

Причины:

```text
sparse files
hard links
compression
CoW/reflink
```

Пользователь должен видеть не только “460 GB logical”, если физически файл занимает 8 GB.

---

# 1.48. Folder → Cleanup bridge

Explorer не должен быть отдельным пассивным disk analyzer.

Любой node:

```text
Add to review
Explain
Protect
Create rule
Open in OS file manager
Show owning project
Show processes using path
```

Если node безопасный generated artifact:

```text
Clean
```

---

# 1.49. Folder → Process bridge

Полезная дополнительная feature:

```text
Who is using this path?
```

Best-effort platform adapter:

### Windows

handle/file usage APIs.

### Linux

`/proc/<pid>/fd`.

### macOS

native process/open-file inspection where permitted.

Если точный handle scan дорогой:

```text
on-demand only
```

Не выполнять для каждого folder во время обычного scan.

---

# 1.50. Process → Folder bridge

В Process details:

```text
cwd
exe
project root
open project folder
show disk usage
show target/node_modules
```

То есть пользователь может начать с 6 GB `node` process и сразу перейти к дисковому footprint проекта.

---

# 1.51. Folder growth history

После первого release можно сохранять aggregate snapshots:

```text
Yesterday     32 GB
Today         49 GB
Growth       +17 GB
```

Это особенно полезно для:

```text
target
node_modules
AI session stores
Docker data
browser profiles
logs
```

Не хранить список всех filenames в history — только bounded aggregate.

---

# 1.52. History page

История должна объединять:

```text
storage snapshots
clean actions
memory/session terminations
browser discards
browser bookmark+close actions
actual reclaimed disk
actual available RAM delta
```

Пример:

```text
13:04  Terminated forgotten Claude session
       RSS before: 7.8 GB
       available RAM: +6.9 GB

12:42  Cleaned Rust targets
       physical disk gained: +18.1 GB

Yesterday
       Browser discard: 31 tabs
       available RAM: +2.2 GB
```

---

# 1.53. Reversible vs irreversible live actions

### Process terminate

Не reversible.

Поэтому review stronger.

### Browser discard

Reversible by selecting tab.

### Bookmark + Close

Recoverable through bookmark/history.

### Disk Trash

Recoverable but does not necessarily free physical space.

### Permanent generated delete

Not recoverable, but data rebuildable.

UI должен визуально различать эти свойства.

---

# 1.54. Live Resource data model

```rust
pub struct ProcessSnapshot {
    pub key: ProcessKey,
    pub pid: u32,
    pub parent: Option<ProcessKey>,
    pub name: String,
    pub exe: Option<PathBuf>,
    pub cwd: Option<PathBuf>,
    pub command: Vec<String>,

    pub started_at: Option<SystemTime>,
    pub runtime: Duration,

    pub rss_bytes: u64,
    pub virtual_bytes: u64,
    pub cpu_percent: f32,
    pub accumulated_cpu_ms: u64,

    pub disk_read_delta: u64,
    pub disk_write_delta: u64,

    pub network: NetworkSnapshot,

    pub project: Option<ProjectAttribution>,
    pub session: Option<SessionId>,
}
```

---

# 1.55. PID reuse protection

Никогда не идентифицировать process только по PID.

Использовать:

```text
PID + process start time
```

как ProcessKey.

Иначе history/action может попасть в другой процесс после PID reuse.

---

# 1.56. Session model

```rust
pub struct LiveSession {
    pub id: SessionId,
    pub kind: SessionKind,
    pub project: Option<ProjectId>,
    pub processes: Vec<ProcessKey>,

    pub started_at: SystemTime,
    pub observed_last_activity: Option<SystemTime>,

    pub rss_bytes: u64,
    pub cpu_percent: f32,
    pub disk: SessionDiskUsage,
    pub network: SessionNetworkUsage,

    pub activity: SessionActivity,
    pub safety: SessionSafety,
    pub recommendation: SessionRecommendation,
}
```

---

# 1.57. SessionKind

```rust
Terminal
ClaudeCode
Codex
Mcp
DevServer
Build
TestRunner
LanguageServer
Browser
Container
GenericApp
Unknown
```

---

# 1.58. Resource time-series storage

Не нужен heavyweight metrics database.

MVP:

```text
in-memory bounded rings
```

Optional persistent aggregates:

```text
SQLite or compact local store
```

Если SQLite добавляется — только в `sweeploom-history`, не в core.

---

# 1.59. Monitoring overhead budget

SweepLoom не должен сам становиться resource hog.

Targets:

```text
idle CPU            < 0.5% typical
active UI CPU       < 2% excluding explicit scans
base RAM            < 100 MB target
network tracing     capability-gated
disk writes         batched / minimal
```

Это benchmark targets, не marketing guarantees до измерения.

---

# 1.60. Sampling strategy

Не refresh all expensive metrics каждую frame.

```text
UI frame             60 Hz max
process refresh      1 s
connections          2–5 s
full project mapping 10–30 s / event-driven
disk inventory       explicit/background low priority
```

Network ETW/eBPF events aggregated outside UI thread.

---

# 1.61. UI process table virtualization

Использовать:

```text
egui_extras::TableBuilder
TableBody::rows
```

Как и disk candidate list.

Сотни/тысячи processes не должны создавать постоянные widgets.

---

# 1.62. Session details UI

Правая панель:

```text
SweepLoom / Claude Code
Project: /work/sweeploom
Started: 4d ago
Observed idle: 9h

RESOURCES
RAM       6.4 GB
CPU       0.1%
Disk      idle
Network   idle

PORTS
5173 LISTEN
9229 LISTEN

PROCESSES
claude
node vite
node mcp-server
playwright

WHY RECOMMENDED
✓ project cold 5d
✓ no source writes
✓ no network 3h
✓ no CPU activity 2h
✓ terminal root appears abandoned

[Keep] [Terminate gracefully]
```

---

# 1.63. Resource graph UI

Один simple chart, не observability monster.

Per process/session:

```text
CPU
RAM
Network
Disk I/O
```

Time selector:

```text
10m
1h
2h
24h
```

Использовать `egui_plot` optional feature либо собственный minimal plot.

---

# 1.64. Network details UI

```text
Connection
Local
Remote
State
Rx
Tx
Last activity
```

Remote hostname resolution:

- optional;
- system resolver;
- never upload endpoints anywhere;
- no third-party DNS lookup by default.

---

# 1.65. Process safety categories

```text
SYSTEM_CRITICAL
SYSTEM_SERVICE
USER_APP
DEVELOPER_TOOL
DEV_SERVER
AGENT
HELPER
ORPHAN_CANDIDATE
UNKNOWN
```

Auto-termination recommendations только для known user/dev domains.

---

# 1.66. User policies for live sessions

На session:

```text
Keep
Never suggest
Suggest if idle > 2h
Suggest if project cold > 3d
Always protect this project
Auto-stop known dev server after N hours (future)
```

Автоматическое termination по умолчанию OFF.

---

# 1.67. Browser user policies

```text
Never touch pinned
Never touch audible
Protect domain
Discard after 7d
Bookmark+close after 30d
Ask every time
```

Scheduled tab actions — не P0.

---

# 1.68. Competitor re-analysis: storage/dev cleaners

На дату исследования, ближайшие категории:

## null-e

Есть:

- Rust core;
- stale project analysis;
- project/build/cache cleanup;
- Git integration;
- Docker;
- Xcode/Android/ML categories;
- trash;
- TUI + Tauri GUI;
- duplicates.

SweepLoom должен быть сильнее в:

```text
source-vs-artifact heat
Weavatrix scan speed/evidence
live sessions
browser
process/network history
folder explorer
```

## Kondo

Есть:

- Rust;
- recursive project detection;
- build artifact cleanup;
- age filtering;
- CLI/UI.

SweepLoom не должен проиграть его простоте.

## cargo-reclaim

Очень сильный semantic baseline для Cargo:

- partial target cleanup;
- active build protection;
- recent write windows;
- stale incremental/deps/build output;
- persisted plans;
- revalidation.

SweepLoom Cargo analyzer должен соответствовать этому уровню safety.

## devclean-cli (Rust)

На 2026-07/08 линия уже имеет очень серьёзные safety features:

- read-only scan;
- Git-tracked protection;
- exact plan;
- apply-time revalidation;
- same-filesystem quarantine;
- learning mode;
- `target-free`;
- watch mode;
- history/statistics;
- Docker guardrails;
- safe/review-only separation.

Это означает:

> revalidation, target-free и safe/review classification уже нельзя считать уникальными.

SweepLoom должен реализовать их качественно как baseline.

## ohing504/devclean (Go)

Умеет:

- Node/Rust/Ruby/Python/Go/Xcode;
- monorepo grouping;
- Git + filesystem activity classification;
- sparse-aware sizing;
- hard-link-aware sizing;
- LLM model stores;
- tool caches;
- interactive tree selector.

Следовательно:

```text
sparse-aware
hard-link-aware
activity classification
monorepo grouping
```

тоже должны быть baseline.

## ImL1s/devclean

Очень важный конкурент именно для новой Memory feature.

Проект прямо нацелен на:

```text
orphaned MCP servers
Flutter daemons
Gradle
iOS simulators
AI tools
```

и описывает случаи 10–20+ GB leaked RAM.

Safe mode использует orphan/PPID logic, deep mode останавливает дополнительные daemons.

SweepLoom должен взять саму problem category, но сделать её намного глубже:

```text
logical sessions
project attribution
history
network
CPU
browser
safe termination UI
```

---

# 1.69. Competitor re-analysis: process managers

## Windows Task Manager

Baseline:

- apps/background/system grouping;
- CPU;
- memory;
- disk;
- network;
- performance charts;
- terminate task.

Что SweepLoom добавляет:

```text
developer semantics
project grouping
agent/terminal grouping
stale recommendation
Git/project evidence
disk cleanup bridge
browser companion
```

## Process Explorer

Сильные стороны:

- detailed process tree;
- handles;
- DLL/mapped files;
- handle search;
- tiny/fast native tool.

SweepLoom не должен пытаться заменить deep Windows debugging.

Borrow:

```text
excellent tree readability
open-handle drilldown concept
```

## System Informer

Сильные стороны:

- detailed system activity;
- graphs/statistics;
- active network connections;
- disk activity;
- termination/control;
- deep Windows diagnostics.

Это benchmark UX reference для Live Resources.

SweepLoom differentiator:

```text
cross-platform + workspace-aware recommendations
```

---

# 1.70. Competitor re-analysis: browser tools

Existing tab managers prove demand for:

```text
old tab detection
discard
close
restore
save sessions
```

But most browser tools do not understand:

```text
whole-machine RAM pressure
developer sessions
project context
disk cleanup
```

SweepLoom combines them.

Native Chrome/Firefox APIs support the core safe flow:

```text
tab metadata
lastAccessed
discard
bookmark
native messaging
```

---

# 1.71. Competitor re-analysis: folder/disk analyzers

## TreeSize

Reference features include:

```text
treemap
allocated size
extension views
file age
top files
history
```

## WizTree / WinDirStat class

Reference:

```text
fast hierarchy
treemap
largest files/folders
```

SweepLoom Explorer must not be slower/less useful than a basic disk analyzer.

Differentiator:

```text
folder size
+
project semantics
+
reclaimability
+
process usage
+
activity heat
```

---

# 1.72. Reviewed-market conclusion

Among the tools reviewed for this plan, there are strong specialists in each individual area:

```text
disk cleanup
developer artifacts
process monitoring
orphan process killing
browser tab management
folder treemaps
```

The product opportunity is the **integration boundary**:

> **one local tool that maps disk + processes + projects + terminals + AI agents + browser tabs into the same workspace model.**

Это и должно стать SweepLoom.

---

# 1.73. Revised architecture

```text
Weavatrix/sweeploom
├─ crates/
│  ├─ sweeploom-core/
│  ├─ sweeploom-storage/
│  ├─ sweeploom-rules/
│  ├─ sweeploom-exec/
│  ├─ sweeploom-process/
│  ├─ sweeploom-session/
│  ├─ sweeploom-network/
│  ├─ sweeploom-browser/
│  ├─ sweeploom-history/
│  ├─ sweeploom-platform/
│  ├─ sweeploom-dev/
│  ├─ sweeploom-ai/
│  ├─ sweeploom-general/
│  └─ sweeploom-cli/
├─ apps/
│  └─ sweeploom-gui/
├─ browser/
│  ├─ chromium-extension/
│  └─ firefox-extension/
├─ rules/
├─ fixtures/
├─ benchmarks/
└─ docs/
```

---

# 1.74. Dependency boundaries

```text
sweeploom-core
  knows no OS APIs
  knows no egui
  knows no browser JS

sweeploom-process
  process snapshots + session-neutral metrics

sweeploom-session
  grouping/classification/recommendations

sweeploom-network
  common connection/accounting model

sweeploom-platform
  OS adapters

sweeploom-browser
  native-messaging protocol/model

sweeploom-storage
  disk inventory/candidates/folder aggregation
```

---

# 1.75. Revised implementation priorities

## P0A — Foundation

- brand/repository;
- egui shell;
- Weavatrix Scan;
- process snapshot baseline;
- common data model.

## P0B — Storage intelligence

- project detection;
- Source/Artifact Heat;
- Rust/Node/Python;
- Folder Inspector tree;
- CleanPlan/revalidation.

## P0C — Live Sessions

- process tree;
- project attribution;
- logical session grouping;
- RAM/CPU/I/O history;
- terminal/agent/dev-server detection;
- graceful termination.

## P0D — Browser safe memory reclaim

- browser totals;
- Chromium/Firefox extension;
- `lastAccessed`;
- `Discard`;
- `Bookmark + Close`.

## P1 — Network

- listeners/connections;
- Windows ETW byte history;
- Linux optional advanced accounting;
- best-effort macOS.

## P1 — Explorer treemap

- treemap;
- physical/logical;
- extension/activity views.

## P1 — AI disk histories

- Claude/Codex storage;
- optional Weavatrix Search.

---

# 1.76. First publishable release — revised

Первый публичный SweepLoom уже должен показывать идею целиком, а не только disk cleaner.

Минимальный сильный release:

```text
STORAGE
  Rust
  Node/Bun
  Python
  temp/cache
  Folder Inspector

PROJECTS
  Source Heat
  Artifact Heat
  Git safety

SESSIONS
  process/session tree
  RAM/CPU
  observed history
  project/agent attribution
  terminate session

BROWSER
  optional extension
  lastAccessed
  discard
  bookmark+close

SAFETY
  plan
  revalidation
  explicit review
```

Network byte attribution может быть P1, но:

```text
listening ports/connections
```

желательно уже в первом release хотя бы на наиболее доступных платформах.

---

# 1.77. New flagship demo

Хороший demo продукта:

```text
Machine: 48 GB RAM
Used:    32 GB

SweepLoom finds:

Claude / old-project
  7.4 GB
  idle 11h

Codex / experiment
  4.1 GB
  idle 2d

Node dev servers
  2.7 GB
  4 stale listening ports

Browser
  41 cold tabs
  safe discard set

Disk
  38 GB cold targets
  19 GB node_modules

User selects:
  terminate two forgotten sessions
  discard browser tabs
  clean cold generated artifacts

Result:
  Available RAM gain measured
  Disk free-space gain measured
```

Это намного сильнее demo «мы удалили target».

---

# 1.78. Research references for the upgraded scope

## SweepLoom foundation

- https://github.com/Weavatrix/weavatrix-scan
- https://github.com/Weavatrix/weavatrix-git
- https://github.com/Weavatrix/weavatrix-search
- https://github.com/Weavatrix/weavatrix-worktree
- https://github.com/Weavatrix/weavatrix-benchmarks

## Rust GUI

- https://github.com/emilk/egui
- https://docs.rs/egui_extras/

## Process metrics

- https://docs.rs/sysinfo/latest/sysinfo/struct.Process.html
- https://docs.rs/sysinfo/latest/sysinfo/

At research time:
```text
sysinfo 0.39.6
```

## Windows per-process network

- https://learn.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getextendedtcptable
- https://learn.microsoft.com/en-us/windows/win32/etw/tcpip
- https://learn.microsoft.com/en-us/windows/win32/etw/tcpip-sendipv4

## Linux process/socket mapping

- https://www.man7.org/linux/man-pages/man5/proc_pid_fd.5.html
- https://www.man7.org/linux/man-pages/man5/proc_pid_net.5.html
- https://www.kernel.org/doc/html/latest/bpf/libbpf/program_types.html

## Browser

- https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/tabs/Tab
- https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/tabs/discard
- https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging
- https://developer.chrome.com/docs/extensions/reference/api
- https://developer.chrome.com/docs/extensions/reference/api/processes

## Process-manager references

- https://learn.microsoft.com/en-us/sysinternals/downloads/process-explorer
- https://systeminformer.sourceforge.io/readme
- https://learn.microsoft.com/en-us/troubleshoot/windows-server/support-tools/support-tools-task-manager

## Storage/dev-cleaner competitors

- https://github.com/us/null-e
- https://github.com/tbillington/kondo
- https://github.com/ml-rust/cargo-reclaim
- https://docs.rs/crate/devclean-cli/latest
- https://github.com/ohing504/devclean
- https://github.com/ImL1s/devclean

## General cleaners / folder analyzers

- https://www.bleachbit.org/
- https://github.com/qarmin/czkawka
- https://www.jam-software.com/treesize

---


# 2. Продуктовые принципы

## 2.1. Safety и Recommendation — разные вещи

Нельзя смешивать «похоже, это стоит удалить» и «это безопасно удалить».

Должны существовать две отдельные оси:

```text
Safety:
  SAFE
  LOW_RISK
  REVIEW
  DANGEROUS
  BLOCKED

Recommendation:
  STRONGLY_RECOMMENDED
  RECOMMENDED
  OPTIONAL
  KEEP
```

Например:

- старый `target/incremental` может быть `SAFE + STRONGLY_RECOMMENDED`;
- весь `target` холодного проекта — `SAFE + RECOMMENDED`, но с высокой rebuild cost;
- старый `.zip` в Downloads — `REVIEW + OPTIONAL`;
- source directory проекта — `BLOCKED`;
- Claude conversation — `REVIEW`, даже если она старая;
- browser cache — `SAFE`, но может замедлить следующий запуск приложения.

**Recommendation score никогда не имеет права обходить Safety blocker.**

---

## 2.2. Cleaner должен объяснять каждое решение

У каждого candidate должен быть `Evidence`.

Пример:

```text
weavatrix-search / target
18.7 GB

Recommendation: CLEAN
Risk: SAFE
Rebuild cost: MEDIUM
Project heat: COLD
Last source write: 5d 7h ago
Last generated write: 4d 22h ago
Last Git commit: 6d ago
Git worktree: clean
Active process: none
Untracked user files: none

Why selected:
- Cargo-generated directory
- no source changes for > 5 days
- no process currently uses project
- repository has no protected changes
- candidate passed current filesystem revalidation
```

И наоборот:

```text
old-service / target
31.2 GB

Recommendation: KEEP
Risk: BLOCKED

Why:
- rustc process PID 18402 has cwd inside project
- target changed 23 seconds ago
```

---

## 2.3. Никакого «AI magic» в destructive path

LLM здесь не нужен.

Можно позже использовать AI только для:

- объяснений;
- поиска human-friendly названий categories;
- natural-language query типа «освободи 30 GB, но не трогай проекты за последнюю неделю».

Но сам `CleanPlan` должен быть:

- deterministic;
- versioned;
- inspectable;
- testable;
- reproducible;
- evidence-based.

---

# 3. Главная killer feature: Project Heat

Это нужно сделать центральным элементом продукта.

Не просто:

```text
mtime(project)
```

и не просто:

```text
git log -1
```

А несколько независимых сигналов активности.

---

## 3.1. Не использовать filesystem creation time как основной сигнал

Пользователю полезно видеть «где недавно появились файлы», но portable filesystem semantics здесь плохие:

- `created()/birth time` доступен не везде;
- Unix `ctime` — это **не creation time**, а metadata change;
- копирование/restore может менять timestamps;
- Git checkout может массово обновить mtimes;
- generated files могут быть свежими при давно неактивном source.

Поэтому хранить:

```rust
ActivityEvidence {
    latest_source_modified,
    latest_generated_modified,
    latest_any_modified,
    latest_birth_time_if_available,
    git_last_commit,
    git_dirty_since_if_known,
    process_activity,
    ai_session_activity,
}
```

`birth_time` — только дополнительный UI evidence.

Основной portable сигнал — `modified time`.

---

## 3.2. Source Heat и Artifact Heat должны быть раздельными

Очень важная деталь.

### Source Heat

Смотрим файлы, которые относятся к пользовательской работе:

- `*.rs`
- `*.go`
- `*.ts`, `*.tsx`
- `*.js`, `*.jsx`
- `*.py`
- `*.cs`
- `*.java`
- `*.kt`
- `*.swift`
- `*.cpp`, `*.h`
- config/manifest files;
- migrations;
- scripts;
- tests;
- docs проекта при необходимости.

При этом source scan должен уважать repository ignore logic и исключать:

- `target`
- `node_modules`
- `.venv`
- `dist`
- `build`
- caches;
- generated output.

### Artifact Heat

Отдельно смотрим:

- `target`
- `node_modules`
- `.venv`
- `.next`
- `.turbo`
- Gradle caches;
- build dirs;
- benchmarks output;
- coverage;
- generated binaries.

Это отвечает на другой вопрос:

> когда последний раз этот build/cache реально использовался?

### Почему это важно

Если Rust-проект не редактировали 12 дней, но какая-то автоматическая задача вчера тронула `target`, он не должен становиться «HOT source project».

И наоборот: если пользователь сегодня редактировал `src/lib.rs`, но `target` последний раз менялся 8 дней назад, cleaner не должен агрессивно чистить проект только по старому target.

---

# 4. Activity State вместо одного магического score

Для safety лучше deterministic buckets.

Рекомендуемые defaults:

| State | Meaning |
|---|---|
| `ACTIVE_NOW` | процесс использует project/candidate или запись была < 15 min |
| `HOT` | meaningful source activity < 24 h |
| `WARM` | 1–3 days |
| `COOL` | 3–7 days |
| `COLD` | 7–30 days |
| `DORMANT` | 30–180 days |
| `ARCHIVAL` | > 180 days |

Порог **3–5 дней**, который нужен для реальной ежедневной работы, таким образом становится нормальной частью UX.

---

## 4.1. Рекомендованная политика по умолчанию

### `< 1 day`

- не предлагать удаление whole build directories;
- разрешать только очевидный temp/old incremental мусор;
- активные процессы — hard block.

### `1–3 days`

- `Light cleanup`;
- clean stale temp/incremental only;
- whole `target/node_modules` обычно не выбирать автоматически.

### `3–7 days`

- `Balanced cleanup`;
- можно рекомендовать удаление больших regenerable directories;
- например `target > 3 GB`;
- всегда показывать rebuild cost.

### `7–30 days`

- whole generated directories можно рекомендовать активнее;
- `target`, `node_modules`, `.venv`, old benchmark output.

### `30+ days`

- aggressive generated cleanup;
- IDE/project caches;
- old package state;
- AI sessions — только review/archive.

### `180+ days`

- показывать проект как archival candidate;
- **никогда автоматически не удалять source project**.

---

# 5. Project Heat signals

## P0 — обязательные

### 5.1. Latest meaningful source modification

Для project root:

```text
latest_source_write
source_files_changed_last_24h
source_files_changed_last_3d
source_files_changed_last_7d
```

Это намного сильнее обычного `last Git commit`.

---

## 5.2. Git state

Использовать `weavatrix-git`.

Проверять:

- staged changes;
- tracked modified/deleted files;
- untracked files;
- ignored-only state;
- branch/HEAD;
- last commit;
- worktree relationship;
- linked worktrees.

### Необходимое расширение `weavatrix-git`

Сейчас `weavatrix-git::Repository::status()` работает по tracked/index state, но не строит полный untracked/ignored contract.

Для cleaner добавить reusable API:

```rust
pub struct WorktreeSafety {
    pub tracked_dirty: bool,
    pub staged_dirty: bool,
    pub untracked_count: u64,
    pub ignored_count: u64,
    pub submodule_unknown: bool,
    pub worktree_kind: WorktreeKind,
    pub evidence: Vec<WorktreeEvidence>,
}

pub enum WorktreeSafetyLevel {
    Clean,
    IgnoredOnly,
    HasUntracked,
    DirtyTracked,
    Unknown,
}
```

Это будет полезно не только cleaner'у, но и Weavatrix agent/worktree ecosystem.

---

## 5.3. Active process detection

Использовать `sysinfo`.

Проверять:

```text
process.cwd()
process.exe()
process.cmd()
```

### Hard blocker

Если:

```text
process.cwd() is inside project root
```

или:

```text
process.cwd() is inside candidate root
```

### High-confidence blocker

Если command line явно содержит canonical project/target path и процесс относится к:

- cargo
- rustc
- rust-analyzer
- node
- bun
- npm
- pnpm
- yarn
- vite
- next
- python
- uv
- pytest
- java
- gradle
- dotnet
- go
- cmake
- ninja
- clang
- gcc

### Open files

Если платформа позволяет получить open files — использовать только как дополнительный evidence.

Не делать систему зависимой от этого: права и OS APIs отличаются.

---

## 5.4. Recent filesystem writes

Перед удалением candidate:

```text
if newest_write < safety_window:
    BLOCK
```

Рекомендуемый safety window:

```text
ACTIVE_NOW window = 15 min
```

Для build directories можно сделать configurable 5–60 min.

---

## 5.5. AI tool activity

Позже:

- Claude Code session по project path;
- Codex session;
- Cursor workspace;
- agent worktree;
- Grand Tab / MCP agent session при наличии локального evidence.

Если project использовался AI-агентом 10 минут назад, он должен быть `ACTIVE_NOW`, даже если source mtime ещё не изменился.

---

# 6. User intent: cleaner должен учиться не через AI, а через policy

На каждом candidate:

```text
Clean now
Keep
Never clean
Always clean when cold
Clean only if > X GB
Clean after N days
Archive instead of delete
Ask every time
```

На project:

```text
Pin project
Mark as active
Mark as dormant
Never clean dependencies
Allow generated cleanup
```

На category:

```text
Rust target:
  [x] suggest after 3 days
  [x] auto-select after 7 days
  [ ] scheduled cleanup

Downloads:
  [ ] auto-select anything
  [x] suggest installers > 30 days
```

---

# 7. Smart selection — «что бы cleaner сам удалил»

Это отдельный слой над candidates.

## 7.1. Suggested Selection

В Review screen cleaner автоматически формирует рекомендуемый набор.

Но:

- SAFE generated/cache candidates можно pre-select;
- REVIEW user data **не pre-select по умолчанию**;
- BLOCKED никогда нельзя выбрать без изменения причины block.

Пример:

```text
[x] SAFE      target/incremental     4.2 GB
[x] SAFE      old target             9.8 GB
[x] SAFE      npm cache              2.7 GB
[ ] REVIEW    Downloads/archive.zip  6.0 GB
[ ] REVIEW    Claude history         3.1 GB
[-] BLOCKED   active target         18.0 GB
```

---

## 7.2. «Free X GB»

Очень сильная UX feature.

Пользователь задаёт:

```text
Free at least: 30 GB
```

Planner выбирает минимально болезненный набор.

Сортировка:

1. hard safety;
2. user policies;
3. rebuild cost;
4. activity state;
5. reclaimable physical bytes;
6. category preferences.

Псевдологика:

```text
exclude BLOCKED

rank by:
  risk ascending
  rebuild_cost ascending
  activity colder first
  reclaimable_bytes descending

take until requested bytes reached
```

Никакого LLM.

---

# 8. Candidate model

Рекомендуемый public contract:

```rust
pub struct Candidate {
    pub id: CandidateId,
    pub kind: CandidateKind,
    pub owner: CandidateOwner,

    pub path: PathBuf,

    pub logical_bytes: u64,
    pub allocated_bytes: Option<u64>,
    pub file_count: u64,

    pub activity: ActivityEvidence,
    pub safety: SafetyAssessment,
    pub rebuild: RebuildAssessment,

    pub deletion: DeletionStrategy,
    pub evidence: Vec<Evidence>,

    pub user_policy: UserPolicy,
}
```

---

## 8.1. CandidateKind

```rust
pub enum CandidateKind {
    BuildArtifact,
    DependencyTree,
    PackageCache,
    ToolCache,
    IdeCache,
    AiSession,
    AiCache,
    AgentWorktree,
    TempFile,
    TempDirectory,
    CrashDump,
    Log,
    BrowserCache,
    OldInstaller,
    OldArchive,
    LargeFile,
    EmptyDirectory,
    DuplicateFile,
    ContainerResource,
    SimulatorData,
    MlModelCache,
}
```

---

## 8.2. CandidateOwner

```rust
pub enum CandidateOwner {
    Project(ProjectId),
    Tool(ToolId),
    Application(AppId),
    System,
    User,
}
```

---

# 9. SafetyAssessment

```rust
pub struct SafetyAssessment {
    pub level: SafetyLevel,
    pub blockers: Vec<Blocker>,
    pub warnings: Vec<Warning>,
    pub confidence: Confidence,
}
```

### SafetyLevel

```text
SAFE
LOW_RISK
REVIEW
DANGEROUS
BLOCKED
```

### Blockers

Примеры:

```text
ActiveProcess
RecentWrite
DirtyTrackedFiles
UntrackedFiles
UnknownGitState
SymlinkEscape
ReparsePointEscape
PermissionBoundary
MountedFilesystemBoundary
CandidateChangedAfterPlan
ProtectedPath
UserPinned
SharedBuildDirectoryInUse
```

---

# 10. Rebuild cost

Очень полезная фича, которой не хватает обычным cleaners.

```text
NONE
LOW
MEDIUM
HIGH
VERY_HIGH
UNKNOWN
```

Сигналы:

### Rust

- размер target;
- release/debug;
- количество crates;
- native deps/build scripts;
- project history;
- optionally previous observed build duration later.

### Node

- dependency count;
- package manager;
- local package store;
- lock file;
- postinstall/native modules.

### Python

- `.venv`;
- package count;
- native packages;
- uv/pip cache availability.

### Xcode / Android

- DerivedData rebuild;
- simulator data — другая стоимость;
- archives — потенциально пользовательские данные, не cache.

---

# 11. Фактический disk usage

`metadata.len()` — это не всегда реально освобождаемое место.

Причины:

- sparse files;
- filesystem compression;
- hard links;
- APFS/CoW;
- deduplication;
- reflinks.

Поэтому модель должна различать:

```text
logical_bytes
allocated_bytes
estimated_reclaimable_bytes
actual_reclaimed_bytes
```

---

## 11.1. Hard link accounting

`weavatrix-scan` уже имеет native `FileIdentity`:

- Unix: device + inode;
- Windows: volume + file identity.

Использовать identity для:

- не считать hard-linked data несколько раз;
- защищаться от path replacement race;
- группировать физические объекты.

---

## 11.2. Preview vs actual

До cleanup:

```text
Estimated reclaim: 19.2 GB
```

После:

```text
Planned logical:      21.7 GB
Estimated physical:   19.2 GB
Filesystem free gain: 18.9 GB
```

Последняя цифра — самая честная.

---

# 12. Очень важный момент: Trash не всегда освобождает место

Конкуренты любят писать «safe delete to Trash».

Но если 40 GB `target` переместить в Trash **на том же volume**, свободного места может почти не прибавиться.

Поэтому стратегии должны отличаться.

```rust
pub enum DeletionStrategy {
    PermanentGenerated,
    Trash,
    NativeTool,
    Archive,
    Truncate,
    InspectOnly,
}
```

### Defaults

**Generated/regenerable:**

```text
PermanentGenerated
```

после review/revalidation.

**User files:**

```text
Trash
```

**AI history:**

```text
Archive or Trash
```

**Docker/package manager:**

```text
NativeTool/API
```

---

## 12.1. `trash` crate

Для первого production prototype можно изолировать crate `trash` за `TrashBackend`.

Он поддерживает Windows Recycle Bin, macOS Trash и FreeDesktop Trash.

Но у текущей Linux/FreeBSD реализации документация отдельно отмечает потенциальную проблему с thread-unsafe libc mount APIs, закрытую внутренним Mutex.

Поэтому:

```text
sweeploom-platform::TrashBackend
```

должен быть abstraction boundary.

Не размазывать `trash::delete()` по business logic.

Позже backend можно заменить без изменения planner/executor.

---

# 13. Что именно чистим: Developer mode

## P0

### Rust

- `target`;
- incremental;
- stale `deps` variants;
- fingerprints;
- build-script caches;
- temp;
- old benchmark outputs;
- Criterion reports;
- coverage;
- custom `CARGO_TARGET_DIR`;
- workspace shared target;
- Cargo home cache where safe.

### Node / Bun

- `node_modules`;
- `.next`;
- `.nuxt`;
- `.turbo`;
- `.vite`;
- `.parcel-cache`;
- `dist`;
- build caches;
- npm/yarn/pnpm/bun caches.

### Python

- `.venv`;
- `venv`;
- `__pycache__`;
- `.pytest_cache`;
- `.mypy_cache`;
- `.ruff_cache`;
- pip cache;
- uv cache;
- build/dist artifacts.

### Go

- build cache;
- module cache;
- temporary test/build artifacts.

### .NET

- `bin`;
- `obj`;
- test results;
- NuGet caches where safe.

---

## P1

### JVM

- Gradle project caches;
- Gradle user cache;
- Maven cache review;
- `target`;
- `build`.

### Swift/Xcode

- DerivedData;
- DeviceSupport;
- simulator caches;
- `.build`;
- SPM cache;
- CocoaPods cache.

**Archives не считать обычным cache.**

### Android

- Gradle;
- build directories;
- emulator snapshots/caches;
- obsolete SDK pieces только через explicit review/native tooling.

### C/C++

- CMake build;
- Ninja build;
- Meson;
- generated object directories;
- compilation caches.

---

# 14. Rust analyzer должен быть лучше простого `cargo clean`

Это хороший первый showcase продукта.

## Modes

### Light

Удалить:

- temp;
- stale incremental;
- очевидно superseded intermediates.

Минимальная rebuild cost.

### Balanced

Дополнительно:

- old hashed deps variants;
- fingerprints;
- stale build script outputs;
- old intermediates.

### Profile

```text
old debug
old release
custom profiles
cross targets
```

### Full

Удалить весь target.

---

## 14.1. Обязательно понимать

- workspace root;
- workspace members;
- shared `target`;
- `CARGO_TARGET_DIR`;
- Cargo config target dir;
- build directory configuration;
- cross compilation targets;
- profiles;
- active cargo/rustc processes.

---

## 14.2. Active build = hard block

У `cargo-reclaim` правильная идея:

> нельзя безопасно выдёргивать supposedly stale hashed artifacts во время активного Cargo build.

В `sweeploom`:

```text
active cargo/rustc against target
    => BLOCK candidate
```

Даже для smart trim.

---

# 15. AI / coding-agent cleaner

Это может стать второй killer feature после Project Heat.

## 15.1. Claude Code

Раздел:

```text
Claude Code
  Sessions
  Tool results
  File history
  Plans
  Debug/cache
```

Показывать:

```text
Project
Session age
Size
Last accessed/modified
Searchable
Retention-managed?
```

Actions:

```text
Keep
Archive
Trash
Use Claude-native purge if supported
Change retention setting
```

---

## 15.2. Codex

С internal session storage действовать намного осторожнее.

Policy:

```text
official/supported cleanup API exists
    => use it

supported CLI operation exists
    => use it

format known but undocumented
    => inspect/archive only

raw delete of internal DB/session relation
    => never default
```

Причина: session metadata и payload могут жить в нескольких связанных местах.

---

## 15.3. Weavatrix Search

`weavatrix-search` не нужен для обычного disk scan.

Он нужен позже именно здесь:

```text
Search old conversations before cleaning:
  "SDPA benchmark"
  "edge analytics"
  "scheduler"
```

Потом:

```text
Keep matching sessions
Archive matching sessions
Clean the rest
```

Это очень сильная интеграция.

---

# 16. Обычный disk cleaner — не только разработка

Developer intelligence остаётся differentiator, но приложение должно быть полезно и как обычный cleaner.

---

## P0 General

### Temporary files

- OS/user temp;
- application temp;
- `.tmp`;
- `.temp`;
- partial downloads;
- crash temp.

### Crash dumps

- user crash dumps;
- application crash reports;
- old dumps.

### Logs

- old user/app logs;
- truncate active logs вместо blind delete, где это требуется.

### Large Files Explorer

Не auto-delete.

Показывать:

```text
size
age
type
location
last modified
risk
```

### Downloads cleanup

Отдельно:

- old `.exe`;
- `.msi`;
- `.dmg`;
- `.pkg`;
- `.deb`;
- `.rpm`;
- archives;
- duplicate installers.

Никогда не считать любой старый файл в Downloads мусором.

---

## P1 General

### Application caches

- browser cache;
- Electron app caches;
- IDE caches;
- thumbnail caches.

### Empty directories/files

Как inspection category.

### Old installers

Smart suggestions, но `REVIEW`.

### Old archives

`.zip`, `.7z`, `.tar.gz`, `.rar`, disk images.

Только `REVIEW`.

---

## P2 General

### Duplicate files

Не нужно пытаться победить Czkawka в первом релизе.

Если делать:

```text
group by size
-> partial hash
-> full BLAKE3
-> user selection strategy
```

Никакого blind duplicate deletion.

### Similar media

Не относится к core thesis.

Позже или не делать вообще.

---

# 17. System cleaning по платформам

## Windows

### User-level P0

- `%TEMP%`;
- `%LOCALAPPDATA%\Temp`;
- CrashDumps;
- application caches;
- IDE caches;
- developer caches;
- Downloads review.

### P1/P2

- Windows Update cache;
- Delivery Optimization;
- system logs;
- thumbnails;
- component/system areas.

Только через Windows-specific adapters и elevation/capability check.

Не начинать с registry cleaner.

---

## macOS

### P0

- `~/Library/Caches`;
- `~/Library/Logs`;
- temp;
- Xcode;
- simulator caches;
- Homebrew cache;
- developer tools.

### Protected

- system-managed paths;
- user documents;
- Photos;
- Time Machine data;
- unknown APFS snapshots.

---

## Linux

### P0

- `$XDG_CACHE_HOME` / `~/.cache`;
- user temp;
- app logs;
- developer caches;
- Downloads review.

### P1

Package managers через native operations:

- apt;
- dnf;
- pacman;
- snap/flatpak where applicable.

Не удалять package DB вручную.

---

# 18. Rule architecture

Не хардкодить 300 путей в Rust.

Должно быть два класса cleaners.

---

## 18.1. Declarative rules

Например TOML:

```toml
schema = 1

[[cleaner]]
id = "vite-cache"
label = "Vite cache"
category = "build-cache"
risk = "safe"
strategy = "permanent-generated"

markers = ["package.json"]
paths = ["node_modules/.vite", ".vite"]

[cleaner.activity]
use_project_heat = true
suggest_after = "3d"
auto_select_after = "7d"
```

System example:

```toml
[[cleaner]]
id = "windows-user-temp"
category = "temp"
platforms = ["windows"]
roots = ["user-temp"]
match = "**/*"
risk = "safe"
strategy = "permanent-generated"
min_age = "1d"
```

---

## 18.2. Semantic analyzers in Rust

Для:

- Cargo;
- Git;
- Docker;
- Xcode;
- Claude;
- Codex;
- package managers;
- shared build directories.

Интерфейс:

```rust
pub trait Analyzer {
    fn id(&self) -> AnalyzerId;
    fn discover(&self, ctx: &DiscoveryContext, sink: &mut dyn CandidateSink);
    fn revalidate(&self, candidate: &Candidate) -> ValidationResult;
}
```

---

# 19. Что переиспользовать из Weavatrix

## 19.1. `weavatrix-scan` — главный фундамент

Текущая версия при исследовании: `0.4.6`.

Уже есть:

- bounded traversal;
- parallel walker;
- deterministic manifests;
- metadata;
- file sizes;
- `FileVersion`;
- `modified_ns`;
- native `FileIdentity`;
- multi-root;
- cancellation;
- same-filesystem boundary;
- symlink policy;
- compact scan;
- streaming sink;
- incremental/watch support;
- typed skip/error evidence.

Это основное filesystem engine.

---

## 19.2. Важное разделение Scan modes

### Artifact discovery

Нужно **не уважать project `.gitignore`**, иначе cleaner не увидит:

```text
target
node_modules
.venv
dist
```

Текущий `weavatrix-scan` по умолчанию имеет standard skips для многих именно таких directories.

Поэтому artifact discovery:

```text
metadata only
standard skips disabled
repository ignore sources disabled
hidden enabled where needed
no content hashing
streaming/no huge retained manifest
classifier-based pruning
```

### Source Heat

Наоборот:

```text
repository ignore enabled
standard skips enabled
metadata only
meaningful source selection
```

Это убирает generated files из source-activity signal.

---

# 20. Небольшие улучшения `weavatrix-scan`

Не превращать Scan в cleaner-specific crate.

Но два generic API улучшения оправданы.

## 20.1. `IgnorePolicy::none()`

Добавить:

```rust
pub const fn none() -> Self
```

вместо ручного выключения каждого источника.

Полезно любому generic filesystem consumer.

---

## 20.2. Builder для StandardSkips

Добавить:

```rust
ScanOptions::with_standard_skips(StandardSkips)
```

Сейчас поле public, но builder сделает contract явным.

---

## 20.3. Directory aggregation сначала оставить в `sweeploom`

Не нужно сразу добавлять `DirectoryAggregate` в Scan.

Сначала:

```text
ScanSink / streaming traversal
    -> sweeploom-core aggregator
```

Если API окажется generic и полезным минимум двум продуктам — тогда вынести в Scan.

---

# 21. `weavatrix-git`

Текущая версия: `0.3.1`.

Переиспользовать:

- direct in-process repository read;
- HEAD;
- history;
- snapshots;
- tracked status;
- worktree layouts;
- commit evidence.

### Доработать

```text
untracked
ignored
worktree safety summary
submodule safety status
```

Cleaner — хороший реальный consumer для этого API.

---

# 22. `weavatrix-search`

Текущая версия: `0.3.1`.

**Не включать в MVP scan path.**

Подключить feature-gated позже:

```text
sweeploom-ai-search
```

Для:

- Claude;
- Codex;
- Cursor/agent histories;
- search-before-delete.

Это сохранит основной binary меньше.

---

# 23. `weavatrix-worktree`

Не использовать как bulk delete executor.

Его transaction model хорош для десятков файлов, но cleaner может удалять сотни тысяч/миллионы entries.

Переиспользовать концепции:

- dry run;
- immutable plan;
- prepare/commit boundary;
- path confinement;
- CAS/revalidation;
- journal;
- receipts;
- recovery semantics.

Но написать отдельный `sweeploom-exec`.

---

# 24. `weavatrix-benchmarks`

Переиспользовать methodology:

- deterministic fixtures;
- independent runs;
- warmups;
- medians;
- exact parity;
- environment recording;
- generated results;
- no hand-written benchmark numbers.

Добавить отдельный suite для `sweeploom`.

---

# 25. Что НЕ нужно тащить

Не делать cleaner зависимым от:

- `weavatrix-graph`;
- `weavatrix-parse`;
- `weavatrix-memory`;
- `weavatrix-clone`;
- LLM stack.

Это увеличит binary, compile time и surface без пользы для MVP.

---

# 26. GUI: окончательный выбор — egui + eframe

Решение:

```text
egui
eframe
egui_extras
```

Почему:

- полностью Rust-native application stack;
- MIT OR Apache-2.0;
- очень популярный ecosystem;
- быстро писать;
- cross-platform;
- не нужен Electron;
- не нужен Tauri/WebView;
- удобно строить developer-oriented dense UI;
- `egui_extras::TableBuilder`;
- virtualized `TableBody::rows` создаёт только visible rows;
- хороший fit для live streaming scan.

---

# 27. Важный нюанс egui version/MSRV

На 2026-08-25:

```text
egui/eframe 0.36.1
Rust MSRV 1.95
```

`weavatrix-scan/git/search` сейчас держат Rust `1.88`.

Это **не проблема**.

Не поднимать MSRV core libraries.

Структура:

```text
sweeploom-core       MSRV >= 1.88 where possible
sweeploom-rules      MSRV >= 1.88
sweeploom-exec       MSRV >= 1.88 where possible
sweeploom-platform   chosen separately
sweeploom-gui        Rust >= 1.95
```

Новый rustc спокойно скомпилирует зависимости с MSRV 1.88.

---

# 28. eframe renderer: выбрать `glow` для compact desktop app

`eframe` по умолчанию включает `wgpu`.

Но его документация прямо отмечает, что переключение с `wgpu` на `glow` может заметно уменьшить binary size.

Для cleaner:

- сложного GPU rendering нет;
- главная нагрузка — filesystem;
- UI — таблицы, badges, charts;
- `glow` достаточно.

Рекомендация:

```toml
eframe = {
    version = "0.36",
    default-features = false,
    features = ["glow", "persistence", "accesskit"]
}

egui = "0.36"
egui_extras = "0.36"
```

Для Linux включить нужные `x11/wayland` features.

Если позже profiling покажет причину перейти на `wgpu` — renderer можно заменить без переписывания core.

---

# 29. GUI architecture

```text
+-------------------------------------------------------------+
| Disk: 71% used | Reclaimable: 84 GB | Safe now: 42 GB      |
+----------------+--------------------------------------------+
| Overview       | Filters / Smart Plan                       |
| Projects       +--------------------------------------------+
| AI             | [x] Item      Size Heat Risk Impact        |
| Caches         | [x] ...                                ... |
| System         |                                            |
| Large Files    |                                            |
| Downloads      |                                            |
| Rules          |                                            |
| Settings       |                                            |
+----------------+--------------------------------------------+
| Selected: 31 items / 47.3 GB    [Review] [Clean selected]  |
+-------------------------------------------------------------+
```

---

# 30. Projects screen

Основная таблица:

| Select | Project | Reclaim | Source Activity | Artifact Activity | Git | Process | Rebuild | Recommendation |
|---|---|---:|---|---|---|---|---|---|

Например:

```text
[x] weavatrix-search   18.7 GB   5d   4d   clean   none   medium   CLEAN
[ ] kablay             3.1 GB    2h   1h   dirty   vite   low      ACTIVE
[x] old-parser         9.4 GB   41d  40d   clean   none   high     CLEAN
```

---

# 31. Detail drawer

При выборе row:

```text
PROJECT
  /Users/me/src/weavatrix-search

ACTIVITY
  last source write     5d 7h
  files changed 3d      0
  files changed 7d      19
  last Git commit       6d
  build activity        4d 22h
  active processes      none

STORAGE
  target                18.7 GB
    incremental          4.1 GB
    deps                 9.2 GB
    build                2.8 GB
    other                2.6 GB

PLAN
  [x] incremental
  [x] stale deps
  [ ] whole target

WHY
  ✓ Generated by Cargo
  ✓ Git tracked state clean
  ✓ No untracked user files
  ✓ No active process
  ✓ No source writes > 5 days
```

---

# 32. Performance architecture GUI ↔ scanner

Не отправлять UI event на каждый файл.

Background worker:

```text
scan
 -> aggregate
 -> batch 64/128 candidates
 -> bounded channel
 -> UI snapshot
```

UI:

```text
drain batches
sort/filter immutable candidate view
request repaint only when batch arrives
```

Target:

```text
UI updates: 5–15 Hz during scan
```

Не 100,000 repaints/sec.

---

# 33. egui large tables

Использовать:

```text
egui_extras::TableBuilder
TableBody::rows
```

`rows()` виртуализирует list и рендерит только visible rows.

Не создавать widget state для каждого найденного файла.

`CandidateId -> selection state` хранить отдельно.

---

# 34. Workspace structure

Рекомендуемая структура:

```text
SweepLoom/
├─ Cargo.toml
├─ crates/
│  ├─ sweeploom-core/
│  ├─ sweeploom-rules/
│  ├─ sweeploom-exec/
│  ├─ sweeploom-platform/
│  ├─ sweeploom-dev/
│  ├─ sweeploom-ai/
│  ├─ sweeploom-general/
│  └─ sweeploom-cli/
├─ apps/
│  └─ sweeploom-gui/
├─ rules/
│  ├─ common/
│  ├─ windows/
│  ├─ macos/
│  └─ linux/
├─ fixtures/
├─ benchmarks/
└─ docs/
```

---

# 35. `sweeploom-core`

Не знает об egui.

Содержит:

```text
Candidate
Evidence
ActivityEvidence
SafetyAssessment
RebuildAssessment
CleanPlan
CleanPlanEntry
ExecutionReport
Receipt
Policy
```

---

# 36. `sweeploom-rules`

- declarative TOML schema;
- validation;
- platform variables;
- marker detection;
- category mapping;
- age policy;
- exclusions.

Правила — data, не arbitrary code.

---

# 37. `sweeploom-dev`

Semantic analyzers:

```text
Rust
Node/Bun
Python
Go
.NET
JVM
Swift/Xcode
Android
C/C++
```

---

# 38. `sweeploom-ai`

Feature-gated analyzers:

```text
Claude
Codex
Cursor
other agent systems
```

`weavatrix-search` подключать только здесь.

---

# 39. `sweeploom-platform`

Всё OS-specific:

```text
standard paths
process discovery
available/free space
allocated size
trash
elevation
system cleaners
filesystem capabilities
```

Никаких `cfg(target_os)` по всему core.

---

# 40. Cross-platform user directories и лицензии

Так как одна из причин выбора egui — избежать лишних licensing headaches, dependency graph тоже стоит держать аккуратным.

Вместо автоматического выбора `dirs/directories` можно рассмотреть:

```text
etcetera
+
userdirs
```

На момент исследования `userdirs` специально позиционируется как cross-platform user-facing directories crate без copyleft dependencies, а `etcetera` покрывает app config/data/cache conventions.

В любом случае добавить CI:

```text
cargo deny check licenses
```

И зафиксировать whitelist:

```text
MIT
Apache-2.0
BSD-2-Clause
BSD-3-Clause
ISC
Zlib
Unicode-3.0
```

Отдельно review любых исключений.

---

# 41. Execution model

Pipeline:

```text
DISCOVER
  ↓
CLASSIFY
  ↓
ACTIVITY
  ↓
SAFETY
  ↓
RECOMMEND
  ↓
CLEAN PLAN
  ↓
USER REVIEW
  ↓
REVALIDATE
  ↓
EXECUTE
  ↓
VERIFY
  ↓
RECEIPT
```

---

# 42. CleanPlan

Plan должен быть immutable snapshot.

```rust
pub struct CleanPlan {
    pub version: u32,
    pub id: PlanId,
    pub created_at: SystemTime,
    pub entries: Vec<CleanPlanEntry>,
    pub requested_free_bytes: Option<u64>,
    pub estimated_reclaimable_bytes: u64,
}
```

Entry:

```rust
pub struct CleanPlanEntry {
    pub candidate_id: CandidateId,
    pub path: PathBuf,
    pub expected_identity: Option<FileIdentity>,
    pub expected_latest_write: Option<SystemTime>,
    pub expected_bytes: u64,
    pub strategy: DeletionStrategy,
    pub required_safety: Vec<SafetyPrecondition>,
}
```

---

# 43. Revalidation непосредственно перед delete

Это обязательная часть продукта.

Проверить заново:

- path;
- file identity;
- symlink/reparse state;
- filesystem boundary;
- recent writes;
- active process;
- Git state;
- candidate type markers;
- size/count/revision where applicable.

Если candidate изменился:

```text
SKIPPED_CHANGED
```

Не «ну всё равно удалим».

---

# 44. Race protection

Типичная атака/ошибка:

```text
scan /a/target
user approves
/a/target replaced with symlink to /important
cleaner deletes
```

Должно быть невозможно.

Правила:

- `symlink_metadata`, не blind follow;
- no symlink/reparse traversal;
- identity verification;
- root confinement;
- same-filesystem policy;
- path alias checks;
- revalidation после parent traversal;
- fail closed.

---

# 45. Bulk deletion

Не использовать `weavatrix-worktree` per-file transactions.

Для regenerable directory:

```text
validate root
validate candidate invariants
bounded bottom-up delete
record failures
verify result
```

### Ограниченный parallelism

Не начинать с максимального параллельного `remove_file`.

Диски и Windows filesystem могут стать медленнее.

Benchmark:

```text
1 / 2 / 4 / 8 workers
SSD
HDD
NTFS
APFS
ext4
```

Выбрать adaptive default только после измерений.

---

# 46. Receipts

После cleanup:

```json
{
  "plan": "...",
  "started": "...",
  "finished": "...",
  "selected_logical_bytes": 21400000000,
  "estimated_physical_bytes": 19000000000,
  "actual_free_space_delta": 18700000000,
  "deleted": 29,
  "skipped_changed": 2,
  "failed": 1
}
```

Хранить bounded history.

---

# 47. Optional scheduling

Не P0, но сильная feature.

Примеры:

```text
Clean Rust generated data when:
  project cold > 7d
  AND target > 5 GB

Keep at least:
  80 GB free

Never:
  pinned projects
  dirty repos
  active projects
```

Сначала:

```text
manual run + saved policy
```

Потом:

- Windows Task Scheduler;
- macOS launchd;
- Linux systemd user timer.

Не нужен resident daemon в первой версии.

---

# 48. Конкурентный анализ

## 48.1. null-e

Сейчас это самый близкий прямой конкурент.

Плюсы:

- Rust core;
- developer cleanup;
- Node/Rust/Python/etc.;
- Docker;
- Xcode;
- Android;
- ML;
- IDE;
- Git protection;
- stale projects;
- duplicates;
- TUI;
- GUI;
- Windows/macOS/Linux;
- trash;
- configuration.

Текущий workspace `0.4.3` использует:

```text
jwalk
walkdir
ignore
gix
trash
rayon
Tauri 2
```

### Где можно быть сильнее

1. **Filesystem engine**

У нас уже есть `weavatrix-scan` с bounded deterministic evidence.

2. **Project activity**

В inspected `null-e` stale analyzer:

- при `.git` предпочитается `git log -1`;
- filesystem fallback проверяет mtime только нескольких key files.

Это слабее `Source Heat`.

3. **Pure Rust UI**

У `null-e` GUI — Tauri 2.

У нас:

```text
egui + eframe
```

без WebView frontend.

4. **Rebuild-cost-aware planning**

Делать first-class concept.

5. **Free X GB**

Оптимизировать cleanup под требуемый объём.

6. **AI history search before delete**

Через `weavatrix-search`.

---

# 49. Kondo

Сильные стороны:

- простой;
- Rust;
- GUI + CLI;
- project structures;
- age filter;
- очень понятный workflow.

Сам проект прямо предупреждает, что по смыслу это близко к:

```text
rm -rf with a prompt
```

### Что взять

- максимально простой review workflow;
- `older than` filter;
- не перегружать основной экран.

### Что сделать сильнее

- project heat;
- Git/worktree safety;
- active processes;
- partial cleanup;
- rebuild cost;
- general disk;
- AI data.

---

# 50. cargo-reclaim

Самый полезный reference именно для Rust cleanup semantics.

Есть:

- target discovery;
- partial cleanup;
- stale incremental;
- stale deps;
- fingerprint/build-script cleanup;
- recent-write windows;
- active build protection;
- persisted plans;
- apply-time revalidation;
- scheduler;
- Cargo home.

### Что взять как идею

- smart trim ≠ whole target delete;
- apply-time revalidation;
- active build hard block;
- saved plan;
- policy modes.

### Что сделать шире

`sweeploom` работает не только с Cargo, поэтому Cargo analyzer — один plugin/analyzer внутри общего planner.

---

# 51. cargo-clean-all

Полезный UX reference:

- finds Rust projects recursively;
- interactive selection;
- exclude projects compiled in last N days;
- size threshold.

Это подтверждает, что сценарий:

```text
не чистить проект, который использовался последние 3–5 дней
```

реально нужен.

Но у нас это будет не single age number, а Project Heat.

---

# 52. BleachBit

Stable 6.0.2 в июле 2026 уже добавил cleaners для:

- VS Code;
- Codium;
- Cursor;
- Windsurf;
- Devin;
- Claude Code;
- developer deep scan:
  - venv
  - __pycache__
  - node_modules
  - .angular

Это означает:

> developer cleanup уже становится частью general cleaner market.

### Что взять

Главная архитектурная идея BleachBit:

```text
CleanerML / declarative cleaner rules
```

Нам нужен свой typed TOML schema.

### Где отличаться

BleachBit понимает applications/files.

SweepLoom должен понимать:

```text
project lifecycle
source activity
build activity
Git state
agent activity
rebuild cost
```

---

# 53. Czkawka / Krokiet

Очень сильный Rust general-file competitor.

Есть:

- duplicate files;
- empty folders;
- big files;
- temporary files;
- similar images/videos;
- invalid symlinks;
- broken files;
- bad extensions;
- previews;
- protected/reference directories.

### Что взять

- reference/protected paths;
- explicit selection;
- scalable large-file inspection;
- duplicate selection strategies.

### Не пытаться копировать в MVP

- similar images;
- similar videos;
- music similarity;
- media optimizer.

---

# 54. Главная позиция SweepLoom

```text
BleachBit:
  understands applications

Czkawka:
  understands files

Kondo:
  understands build directories

cargo-reclaim:
  deeply understands Cargo artifacts

null-e:
  understands developer junk categories

SweepLoom:
  should understand the developer workspace lifecycle
```

---


# 54A. Live Resources — обязательное дополнение к MVP

Эта секция имеет приоритет наравне со Storage MVP.

Обязательно:

```text
process snapshots
raw process tree
logical session grouping
project attribution
Claude/Codex/MCP/dev-server signatures
RSS/CPU/I/O sampling
observed activity history
listening ports where available
graceful terminate
terminate subtree/session
system-process protection
```

Browser baseline:

```text
native browser totals
optional companion
lastAccessed
Discard
Bookmark + Close
```

Folder Inspector baseline:

```text
tree breakdown
largest children
logical size
allocated size where available
activity breakdown
category breakdown
```


# 55. MVP — что должно войти обязательно

## Scan

- home/project roots;
- streaming progress;
- size aggregation;
- source/artifact heat.

## Developer

- Rust;
- Node/Bun;
- Python.

## Safety

- Git tracked;
- untracked;
- recent writes;
- active processes;
- symlink/reparse protection;
- revalidation.

## Live Sessions / Memory

- process tree + logical session grouping;
- RAM / CPU / disk-I/O sampling;
- project and terminal attribution;
- Claude Code / Codex / MCP / dev-server detection;
- stale-session recommendations;
- graceful terminate → force terminate fallback;
- listening ports and network connections;
- bounded local resource history.

## Browser

- browser process totals without extension;
- optional Chromium/Firefox companion;
- tab title/URL/lastAccessed/pinned/audible/discarded state;
- safe `Discard`;
- transactional `Bookmark + Close`.

## General

- user temp;
- app caches;
- crash dumps;
- logs;
- large files;
- Downloads old installers review.

## UX

- egui;
- table;
- checkboxes;
- recommendation;
- reason/evidence;
- detail panel;
- `Free X GB`.

---

# 56. MVP не должен включать

- registry cleaner;
- RAM optimizer;
- «PC boost»;
- secure erase/free-space wipe;
- similar photos;
- video duplicate AI;
- uninstall manager;
- package uninstall;
- full Docker UI;
- system service running permanently;
- LLM.

---

# 57. Этап разработки 0 — repository skeleton

Создать:

```text
Weavatrix/sweeploom
```

Workspace crates из раздела выше.

### Acceptance

- CI Windows/macOS/Linux;
- formatter;
- clippy;
- cargo test;
- cargo deny;
- empty eframe app;
- sweeploom-core без GUI dependency.

---

# 58. Этап 1 — filesystem inventory

Подключить:

```text
weavatrix-scan 0.4.x
```

Реализовать:

- multi-root discovery;
- streaming aggregation;
- project marker discovery;
- generated directory classifier;
- general temp classifier;
- progress/cancellation.

### Acceptance

- 1M-file fixture;
- bounded RAM;
- cancellation < 250 ms reaction target;
- no symlink escape;
- exact byte/count parity.

---

# 59. Этап 2 — Project Heat

Реализовать:

```text
SourceActivityAnalyzer
ArtifactActivityAnalyzer
ProcessActivityAnalyzer
```

Buckets:

```text
ACTIVE_NOW
HOT
WARM
COOL
COLD
DORMANT
ARCHIVAL
```

### Acceptance

Fixture:

```text
project A source touched 1 min ago
project B source touched 3d ago
project C source touched 5d ago
project D generated touched today, source 30d ago
```

UI/result должен корректно различить все 4.

---

# 60. Этап 3 — Git safety

Расширить `weavatrix-git`.

Реализовать:

```text
WorktreeSafety
untracked
ignored
```

### Acceptance

Cases:

- clean;
- staged;
- modified;
- deleted;
- untracked;
- ignored-only;
- linked worktree;
- bare repo;
- submodule unknown.

Ни один dirty/untracked case не должен попасть в auto-selected dangerous cleanup.

---

# 61. Этап 4 — CleanPlan + executor

Реализовать:

```text
plan
review
revalidate
execute
receipt
```

### Fault injection

Во время plan/apply:

- create file;
- replace directory;
- create symlink;
- modify source;
- start process;
- change Git state.

Ожидаемый результат:

```text
SKIP / BLOCK
```

а не delete.

---

# 62. Этап 5 — Rust analyzer

Первый flagship analyzer.

Поддержать:

- Cargo workspace;
- target dirs;
- shared target;
- custom target;
- active build;
- light/balanced/full;
- rebuild assessment.

Сравнить с:

```text
cargo-reclaim
cargo-clean-all
cargo clean
```

---

# 63. Этап 6 — egui production UI

После core safety.

### Screens

1. Overview
2. Projects
3. General
4. AI
5. Large Files
6. Rules
7. History
8. Settings

### Acceptance

- 100k candidate table smooth scroll;
- scan does not block UI;
- live sorting;
- search/filter;
- keyboard navigation;
- cancel scan;
- details update < frame;
- dark/light.

---

# 64. Этап 7 — Node/Bun + Python

### Node

- node_modules;
- package manager;
- build caches;
- lock file;
- project heat.

### Python

- venv;
- cache;
- package cache;
- project heat.

### Acceptance

Nested monorepos:

```text
repo/
  apps/a/
  apps/b/
  packages/c/
```

не должны давать хаотичные duplicate candidates.

---

# 65. Этап 8 — General Cleaner P0

Реализовать:

```text
temp
crash dumps
logs
user caches
large files
downloads installers
empty dirs/files
```

### Правило

User-owned content:

```text
REVIEW, not automatic delete
```

---

# 66. Этап 9 — AI cleaner

### Claude

Сначала supported/documented storage.

### Codex

Inspect-first.

### Search

Feature-gated `weavatrix-search`.

### Acceptance

- search session text;
- pin;
- archive;
- clean unpinned old sessions;
- project-linked age.

---

# 67. Этап 10 — JVM/.NET/Go/Xcode/Android

После стабилизации core.

Не добавлять 50 categories до того, как basic safety proven.

---

# 68. Этап 11 — Smart policies / scheduler

```text
Keep 80 GB free
Clean cold projects weekly
Never touch pinned projects
```

Schedule только после хорошего execution history.

---

# 69. Этап 12 — Advanced general tools

Опционально:

- duplicate files;
- cache visualization;
- treemap;
- browser cleanup;
- package manager cleanup;
- Docker;
- ML models.

---

# 70. UI recommendation details

## Header cards

```text
Used
Reclaimable
Safe now
Review
```

Не показывать одну огромную «Junk: 84GB» цифру.

---

## Activity badges

```text
ACTIVE  red
HOT     orange
WARM    yellow
COOL    neutral
COLD    blue
DORMANT gray
```

Цвета — UI detail, но state должен существовать в core.

---

## Risk badge

Не смешивать с activity:

```text
SAFE
LOW RISK
REVIEW
BLOCKED
```

---

# 71. Filters

Обязательные:

```text
> 1 GB
> 5 GB
Cold > 3d
Cold > 5d
Cold > 7d
Only safe
Only projects
Only AI
Only user files
Selected
Blocked
```

---

# 72. Sort

```text
Recommended
Size
Last source activity
Last artifact activity
Rebuild cost
Risk
Path
```

---

# 73. Search

По:

```text
project name
path
category
tool
evidence
```

---

# 74. Manual include/exclude roots

Как у Czkawka:

```text
Include roots
Protected roots
Excluded roots
```

Protected root:

```text
can scan
cannot delete
```

Очень полезно для:

- backups;
- work;
- family photos;
- mounted disks.

---

# 75. Presets

```text
Quick Safe
Developer
Deep
Disk Pressure
Before Backup
Custom
```

### Quick Safe

Только high-confidence cache/temp.

### Developer

Projects + tool caches.

### Deep

Большие/old user files тоже, но review.

### Disk Pressure

`Free X GB`.

---

# 76. Rule packs

Built-in rules versioned вместе с app.

Позже:

```text
community rule packs
```

Но:

- declarative only;
- no shell commands из downloaded rule;
- signature/checksum;
- schema validation;
- dangerous action types запрещены.

---

# 77. Native tool adapters

Некоторые вещи лучше чистить не filesystem delete.

Примеры:

```text
Docker
package managers
Claude supported purge
Cargo commands where appropriate
```

Правило:

> если tool имеет стабильный cleanup/prune API, предпочесть его private filesystem format.

---

# 78. CLI

GUI — основная фича, но CLI нужен обязательно.

```text
sweeploom scan
sweeploom projects
sweeploom plan
sweeploom apply
sweeploom free 50GB
sweeploom explain <id>
sweeploom history
```

Machine output:

```text
--json
--jsonl
```

Полезно для:

- scripts;
- CI workstations;
- agents;
- MCP в будущем.

---

# 79. MCP — не P0

Позже можно сделать read-only tools:

```text
disk_inventory
cleanup_candidates
explain_candidate
build_cleanup_plan
```

Destructive `apply` через MCP — только с очень явной permission model.

Не начинать с этого.

---

# 80. Benchmark plan

## Scan

Fixtures:

```text
20k
200k
1M
5M entries
```

Metrics:

```text
time
peak RSS
files/sec
dirs/sec
cancellation latency
```

---

## Competitors

Сравнить:

- null-e;
- Kondo;
- dua/dust для generic scan where contract comparable;
- cargo-reclaim для Rust analysis.

Только contract-equivalent rows.

---

# 81. Project Heat benchmark/correctness

Не latency, а accuracy fixtures.

Например:

```text
last commit 60d
source write today
```

Ожидаем:

```text
HOT
```

Это как раз случай, где простой Git-last-commit stale detector слаб.

---

# 82. Destructive safety suite

Это должно стать одной из сильнейших частей проекта.

Cases:

### Path

- `..`;
- symlink;
- junction;
- reparse point;
- alias;
- case-change;
- Windows reserved paths;
- long paths;
- non-UTF8 Unix path.

### Filesystem

- permission denied;
- read-only;
- hard links;
- sparse file;
- mount boundary;
- disconnected drive;
- network share;
- file disappears.

### Race

- replace candidate;
- new file after plan;
- source changes;
- active process starts;
- Git becomes dirty;
- file identity changes.

---

# 83. Developer fixtures

## Rust

- workspace;
- shared target;
- linked worktree;
- custom target dir;
- build active;
- dirty source;
- untracked source;
- ignored target.

## Node

- npm;
- pnpm symlinks;
- Yarn;
- Bun;
- monorepo;
- nested node_modules.

## Python

- venv;
- uv;
- pip;
- editable installs.

---

# 84. Cross-platform CI

Минимум:

```text
Windows x64
macOS arm64
Linux x64
```

Позже:

```text
Windows arm64
macOS x64
Linux arm64
```

---

# 85. Packaging

### Windows

- portable `.exe`;
- installer позже.

### macOS

- `.app`;
- signed/notarized release если product distribution.

### Linux

- AppImage;
- tarball;
- distro packages позже.

---

# 86. Binary-size discipline

Поскольку одна из целей — уйти от Electron/Tauri-style overhead:

- `eframe` default features off;
- `glow` first;
- `panic = "abort"` release;
- LTO;
- strip;
- feature-gate AI/archive extras;
- не тянуть Tokio без реальной нужды;
- не тянуть `weavatrix-search` в base build;
- не тянуть media codecs.

Проверять binary size в CI.

---

# 87. Runtime architecture без Tokio в MVP

Не нужен async runtime ради filesystem scan.

Можно:

```text
std::thread
+
bounded crossbeam channel
+
Weavatrix parallel runtime
```

Отдельный async runtime добавлять только если появятся реальные network/API adapters.

---

# 88. Suggested dependencies

Не окончательный lockfile, а направление:

```toml
weavatrix-scan
weavatrix-git

serde
serde_json
thiserror

eframe
egui
egui_extras

crossbeam-channel
sysinfo
fs2

etcetera
userdirs

trash # isolated backend, review/audit
```

Optional:

```text
weavatrix-search
notify
blake3      # duplicate mode later
```

---

# 89. Dependency rules

Для core:

```text
no GUI dependency
no network
no shell execution by declarative rules
no dynamic plugin code
```

Для semantic adapters:

```text
native command allowed only by explicit adapter
command + args typed, never generated string shell
```

Никогда:

```rust
Command::new("sh").arg("-c").arg(rule_string)
```

из community rules.

---

# 90. Telemetry

По умолчанию:

```text
none
```

Cleaner видит очень чувствительную информацию о filesystem.

Если когда-нибудь появится crash reporting:

- explicit opt-in;
- path redaction;
- no filenames by default.

---

# 91. Privacy positioning

Хороший product advantage:

```text
Local-first
No account required
No cloud scan
No file upload
No AI required
```

Очень логично для developer utility.

---

# 92. Что сделать в первую очередь — конкретный порядок

Если начинать писать прямо сейчас:

## 1 — Repository + boundaries

Создать:

```text
Weavatrix/sweeploom
```

Сразу разделить:

```text
sweeploom-core
sweeploom-storage
sweeploom-process
sweeploom-session
sweeploom-platform
sweeploom-gui
```

## 2 — egui shell

Поднять empty `egui/eframe` app:

```text
Overview
Storage
Sessions
Projects
Explorer
```

Browser/AI tabs можно пока оставить placeholders.

## 3 — Weavatrix Scan

Подключить `weavatrix-scan`.

Добавить generic improvements:

```text
IgnorePolicy::none()
with_standard_skips(...)
```

## 4 — Storage inventory

Сделать:

```text
streaming project discovery
artifact inventory
directory aggregation
physical/logical size model
```

## 5 — Folder Inspector baseline

До сложных cleaner rules уже дать:

```text
tree
largest children
logical bytes
allocated bytes
activity
category
```

Это сразу делает SweepLoom полезным как disk explorer.

## 6 — Project Heat

Сделать:

```text
ActivityEvidence
Source Heat
Artifact Heat
```

И buckets:

```text
ACTIVE_NOW
HOT
WARM
COOL
COLD
DORMANT
ARCHIVAL
```

## 7 — Process snapshot engine

Подключить `sysinfo`.

Собирать:

```text
PID + start time
PPID
cwd
exe
command (redacted)
RSS
CPU
accumulated CPU
disk I/O
runtime
```

## 8 — Raw Process Tree

Сделать обычное дерево процессов с virtualized egui table.

До recommendations пользователь уже должен иметь удобный Task-Manager-like view.

## 9 — Logical Session grouping

Добавить:

```text
project attribution
terminal/session attribution
Claude/Codex signatures
MCP detection
dev-server detection
listening port evidence
```

## 10 — Resource history

Добавить bounded sampling:

```text
CPU
RAM
disk I/O
connections
```

Собственная history начинается только с момента наблюдения.

## 11 — Safe termination

Реализовать:

```text
graceful stop
wait
terminate
force escalation
terminate subtree
terminate logical session
critical-process protection
```

## 12 — Git safety

Расширить `weavatrix-git` до:

```text
WorktreeSafety
untracked
ignored
```

Связать Git state и со storage, и с live-session recommendation.

## 13 — CleanPlan

Сделать:

```text
immutable plan
review
revalidate
execute
verify
receipt
```

## 14 — Rust analyzer

Первый flagship storage analyzer:

```text
Cargo workspace
target
shared/custom target
active build
Light/Balanced/Full
```

## 15 — Smart recommendations

Сделать одновременно:

```text
Suggested Selection
Free X GB
Free X GB RAM
Reduce CPU Load
```

## 16 — Node/Bun + Python

Добавить disk analyzers и dev-service detectors.

Это важно: storage и process detection должны развиваться вместе.

## 17 — Browser companion

Сделать optional Chromium/Firefox WebExtension:

```text
lastAccessed
pinned
audible
discarded
Discard
Bookmark + Close
Native Messaging
```

## 18 — General cleaner

Добавить:

```text
temp
cache
logs
crash dumps
large files
Downloads review
```

## 19 — Network deep attribution

Baseline connections/listeners можно сделать раньше.

Здесь добавить:

```text
Windows ETW byte history
Linux optional advanced accounting
macOS best-effort enhanced backend
```

## 20 — AI storage/history

Claude/Codex disk data:

```text
inspect
archive
search-before-delete
native cleanup where supported
```

Optional `weavatrix-search`.

---

# 93. Что должно быть в первом реально публикуемом release

Не 100 cleaners.

Первый SweepLoom должен показать **всю уникальную идею продукта**.

## Storage

```text
Rust
Node/Bun
Python
Temp
IDE/tool cache
Folder Inspector
```

## Projects

```text
Source Heat
Artifact Heat
Git Safety
Rebuild Cost
```

## Sessions

```text
Raw Process Tree
Logical Session Tree
Project attribution
Claude/Codex/MCP/dev-server detection
RAM
CPU
disk I/O
observed history
listening ports/connections baseline
graceful terminate
terminate session
```

## Browser

Минимум companion preview или уже production extension:

```text
lastAccessed
Discard
Bookmark + Close
```

## Safety

```text
explicit selection
system-process protection
CleanPlan
revalidation
receipts
post-action measured reclaim
```

Лучше сделать 8 categories великолепно, чем 100 shallow cache patterns.

---

# 94. Самая важная конкурентная фича

После расширения scope самая сильная идея уже не только Project Heat.

> **SweepLoom строит workspace model поверх filesystem и OS process tree.**

Он может показать:

```text
Project: Kablay

DISK
  target/node_modules/cache     14.3 GB

LIVE
  Claude session                 5.8 GB RAM
  Vite server                    1.2 GB RAM
  2 listening ports
  CPU idle 6h

BROWSER
  8 related cold tabs
```

И объяснить:

```text
что сейчас активно
что забыто
что regenerable
что protected
что можно убрать
```

Обычный cleaner не видит live sessions.
Обычный Task Manager не понимает projects.
Обычный tab manager не понимает machine pressure.
Обычный disk analyzer не понимает rebuild/safety.

**SweepLoom должен связать эти слои.**

---

# 95. Вторая killer feature

> **Reclaim a requested amount with minimum developer pain.**

Не только:

```text
Free 50 GB
```

Но и:

```text
Free 12 GB RAM
Reduce background CPU below 10%
Quiet my workstation
```

Planner выбирает минимально болезненный набор, но user всегда контролирует destructive/live actions.

---

# 96. Третья killer feature

> **Safe browser memory reclaim based on last real access.**

Не просто «Chrome использует 9 GB».

SweepLoom предлагает:

```text
37 tabs not accessed > 14d

[x] Discard 29
[ ] Bookmark + Close 8
[-] Protect pinned/audible/current
```

`Discard` — preferred action, потому что вкладка остаётся доступной.

---

# 96A. Четвёртая killer feature

> **AI workspace cleanup with search-before-delete.**

Не просто удалить `.claude` / `.codex`, а понять:

- к какому project относится;
- насколько старая session;
- можно ли её найти через search;
- pinned ли она;
- поддерживает ли tool native purge;
- можно ли архивировать.

И live AI processes должны быть связаны с их disk histories, когда attribution доказуема.

---

# 97. Финальное позиционирование

Не:

```text
Fast Rust disk cleaner
```

И не:

```text
RAM booster
```

Сильнее:

> **SweepLoom is a developer-aware workstation resource manager for reclaiming disk, memory, and background compute without losing your work.**

Более коротко:

> **Reclaim your workstation without losing your workspace.**

Developer-specific promise:

> **SweepLoom understands projects, build artifacts, terminals, AI agents, processes, and browser tabs as one workspace.**

---

# 98. Почему проект имеет смысл именно с текущим Weavatrix stack

У большинства конкурентов сложная часть начинается с filesystem traversal, Git integration и safety evidence.

У Weavatrix это уже частично решено:

- `weavatrix-scan` — быстрый evidence-rich filesystem scanner;
- `weavatrix-git` — in-process Git evidence;
- `weavatrix-search` — later searchable AI history;
- `weavatrix-worktree` — проверенные идеи plan/revalidation/recovery;
- `weavatrix-benchmarks` — correctness-gated performance discipline.

Поэтому реальная новая работа концентрируется там, где и должен быть product value:

```text
activity intelligence
cleanup rules
rebuild semantics
safe bulk execution
UI
```

А не на переписывании очередного walker.

---

# 99. Research notes / источники

## Weavatrix

- Weavatrix Scan: https://github.com/Weavatrix/weavatrix-scan
- Weavatrix Git: https://github.com/Weavatrix/weavatrix-git
- Weavatrix Search: https://github.com/Weavatrix/weavatrix-search
- Weavatrix Worktree: https://github.com/Weavatrix/weavatrix-worktree
- Weavatrix Benchmarks: https://github.com/Weavatrix/weavatrix-benchmarks

Исследованные текущие версии на 2026-08-25:

```text
weavatrix-scan   0.4.6 / Rust 1.88
weavatrix-git    0.3.1 / Rust 1.88
weavatrix-search 0.3.1 / Rust 1.88
```

## GUI

- egui: https://github.com/emilk/egui
- eframe: https://github.com/emilk/egui/tree/main/crates/eframe
- egui_extras TableBuilder:
  https://docs.rs/egui_extras/latest/egui_extras/struct.TableBuilder.html
- TableBody virtualization:
  https://docs.rs/egui_extras/latest/egui_extras/struct.TableBody.html

На момент исследования:

```text
egui/eframe 0.36.1
MSRV Rust 1.95
MIT OR Apache-2.0
```

## Developer cleaner competitors

- null-e: https://github.com/us/null-e
- Kondo: https://github.com/tbillington/kondo
- cargo-reclaim: https://github.com/ml-rust/cargo-reclaim
- cargo-clean-all: https://github.com/dnlmlr/cargo-clean-all

## General cleaners

- BleachBit 6.0.2:
  https://www.bleachbit.org/news/bleachbit-602
- CleanerML:
  https://docs.bleachbit.org/cml/cleanerml.html
- Czkawka/Krokiet:
  https://github.com/qarmin/czkawka
- Krokiet instructions:
  https://github.com/qarmin/czkawka/blob/master/instructions/Instruction_Krokiet.md

## Supporting Rust crates / APIs

- sysinfo Process:
  https://docs.rs/sysinfo/latest/sysinfo/struct.Process.html
- trash:
  https://docs.rs/trash/latest/trash/
- etcetera:
  https://docs.rs/etcetera/latest/etcetera/
- userdirs:
  https://docs.rs/userdirs/latest/userdirs/

---


# 99A. Обновлённый приоритет после Memory / Sessions / Browser исследования

После расширения scope приоритет продукта меняется:

```text
1. Safety/data model
2. Fast storage inventory
3. Project Heat
4. Live Sessions
5. Safe process termination
6. Folder Inspector
7. Rust/Node/Python storage analyzers
8. Browser Discard + Bookmark/Close
9. General disk cleanup
10. Per-process network deep attribution
11. AI history search
12. Advanced duplicates/media/system cleanup
```

Причина:

**Live Sessions сильнее отличают SweepLoom от обычного cleaner рынка, чем добавление ещё 50 cache rules.**

Не откладывать Sessions до далёкого roadmap.


# 100. Decision summary

Фиксируем:

```text
Product:               SweepLoom
Domain:                sweeploom.com
Brand:                 SweepLoom by Weavatrix

Language:              Rust
GUI:                   egui + eframe + egui_extras
Renderer first choice: glow

Storage engine:        weavatrix-scan
Git intelligence:      weavatrix-git
AI-history search:     weavatrix-search later/optional

Process metrics:       sysinfo baseline
Process grouping:      custom SweepLoom session engine
Network:               platform adapters
Browser:               optional WebExtension + Native Messaging
History:               bounded local time series

Bulk disk executor:    sweeploom-exec
Rules:                 declarative TOML + semantic Rust analyzers

Primary surfaces:
  Storage
  Sessions
  Projects
  Browser
  Explorer

Main differentiator:
  Workspace model across disk + processes + projects + AI agents + browser

General cleaning:      yes
Project Heat:          yes
Process/session heat:  yes
RAM reclaim planner:   yes
CPU pressure planner:  yes
Network activity:      yes, capability-gated
Folder inspector:      yes
Browser safe discard:  yes
Auto recommendation:   yes
User pin/keep policy:  yes

Cross-platform:
  Windows
  macOS
  Linux

Electron/Tauri:        no
LLM required:          no
Cloud required:        no
Telemetry default:     none
```

## Первый milestone

Не ограничивать его только Rust target cleanup.

Первый architecture milestone:

> **SweepLoom сканирует projects и running processes одновременно, связывает process trees с project roots, показывает disk footprint + Source/Artifact Heat + RAM/CPU, распознаёт хотя бы Claude/Codex/Node/Cargo sessions, позволяет открыть Folder Inspector и безопасно завершить выбранную stale session либо очистить generated artifacts после revalidation.**

Конкретный demo:

```text
SweepLoom project
  target           18.7 GB disk
  source heat      COLD / 5d
  Git              clean

Claude session
  6.4 GB RAM
  idle 9h
  ports 5173, 9229

Actions:
  [Clean target]
  [Terminate Claude session]
  [Inspect folder]
```

Если этот milestone работает качественно, остальной продукт расширяется analyzers/adapters, а не требует менять фундамент.

