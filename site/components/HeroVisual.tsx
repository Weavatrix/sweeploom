/* The README's "The idea": Task Manager's flat list versus the tree a coding
   day actually is, with SweepLoom's verdict on each row. Illustrative, not a
   literal CLI transcript. */

type Row = {
  depth: 0 | 1;
  last?: boolean;
  label: string;
  note?: string;
  verdict?: { text: string; tone: "keep" | "optional" | "safe" | "park" | "archive" };
};

const ROWS: Row[] = [
  { depth: 0, label: "Cursor", note: "43 proc · 7.3 GB", verdict: { text: "Keep", tone: "keep" } },
  { depth: 1, label: "MCP", note: "attached", verdict: { text: "Keep", tone: "keep" } },
  { depth: 1, label: "terminal" },
  { depth: 1, last: true, label: "vite", note: "dev server" },
  { depth: 0, label: "Claude Code", note: "idle", verdict: { text: "Keep", tone: "keep" } },
  { depth: 0, label: "Codex", note: "cargo / rustc", verdict: { text: "Keep", tone: "keep" } },
  { depth: 0, label: "mcp-server", note: "no live agent parent", verdict: { text: "Optional", tone: "optional" } },
  { depth: 0, label: "~/.claude/history.jsonl", note: "archive · 0 tok", verdict: { text: "Inspect", tone: "archive" } },
  { depth: 0, label: "~/.claude/AGENTS.md", note: "always-on", verdict: { text: "Keep", tone: "keep" } },
  { depth: 0, label: "~/.claude/skills", note: "capped · idle 30d+", verdict: { text: "park?", tone: "park" } },
  { depth: 0, label: "app/target", note: "rebuild is cheap", verdict: { text: "SAFE", tone: "safe" } },
];

const TONE: Record<NonNullable<Row["verdict"]>["tone"], string> = {
  keep: "text-[var(--term-ok)] border-[color-mix(in_srgb,var(--term-ok)_40%,transparent)]",
  optional: "text-[var(--term-gold)] border-[color-mix(in_srgb,var(--term-gold)_45%,transparent)]",
  safe: "text-[var(--ink)] bg-[var(--gold)] border-transparent",
  park: "text-[var(--term-gold)] border-dashed border-[color-mix(in_srgb,var(--term-gold)_45%,transparent)]",
  archive: "text-[var(--term-dim)] border-[var(--term-border)]",
};

const TASK_MANAGER = ["node.exe", "node.exe", "node.exe", "claude.exe", "Cursor.exe", "msedge.exe"];

export function HeroVisual() {
  return (
    <div className="relative mx-auto w-full max-w-[580px] min-w-0 sm:pt-10 sm:pl-10 lg:mx-0 lg:justify-self-end">
      <div className="gold-glow pointer-events-none absolute -inset-16 opacity-80" aria-hidden="true" />

      {/* behind: what Task Manager shows */}
      <div
        className="term absolute top-0 left-0 hidden w-48 rotate-[-4deg] opacity-70 sm:block"
        aria-hidden="true"
      >
        <div className="term-bar">
          <span className="term-dots">
            <i />
            <i />
            <i />
          </span>
          <span>Task Manager</span>
        </div>
        <ul className="px-4 py-3 font-mono text-[0.72rem] leading-6 text-[var(--term-dim)]">
          {TASK_MANAGER.map((name, i) => (
            <li key={i}>{name}</li>
          ))}
        </ul>
      </div>

      {/* front: what SweepLoom shows */}
      <figure className="term relative">
        <div className="term-bar">
          <span className="term-dots" aria-hidden="true">
            <i />
            <i />
            <i />
          </span>
          <span>what a coding day actually looks like</span>
          <span className="ml-auto hidden items-center gap-1.5 sm:inline-flex">
            <i className="pulse-dot h-1.5 w-1.5 rounded-full bg-[var(--term-ok)]" aria-hidden="true" />
            local
          </span>
        </div>
        <ul className="px-3 py-3 font-mono text-[0.72rem] sm:px-4 sm:text-[0.78rem]">
          {ROWS.map((row, i) => (
            <li
              key={row.label + i}
              className="rise flex items-center gap-2 rounded-md px-1.5 py-[0.3rem] hover:bg-white/[0.03]"
              style={{ animationDelay: `${200 + i * 70}ms` }}
            >
              <span className="min-w-0 flex-1 truncate">
                {row.depth === 1 ? (
                  <span className="text-[var(--term-dim)]" aria-hidden="true">
                    {row.last ? "  └─ " : "  ├─ "}
                  </span>
                ) : null}
                <span className={row.depth === 0 ? "text-[var(--term-fg)]" : "text-[var(--term-fg)]/85"}>{row.label}</span>
                {row.note ? <span className="text-[var(--term-dim)]"> · {row.note}</span> : null}
              </span>
              {row.verdict ? (
                <span
                  className={
                    "shrink-0 rounded-full border px-2 py-[1px] text-[0.64rem] tracking-wide " + TONE[row.verdict.tone]
                  }
                >
                  {row.verdict.text}
                </span>
              ) : null}
            </li>
          ))}
        </ul>
        <figcaption className="border-t border-[var(--term-border)] px-4 py-2.5 font-mono text-[0.68rem] text-[var(--term-dim)]">
          Live agents stay Keep. Only SAFE generated rows can be applied.
        </figcaption>
      </figure>
    </div>
  );
}
