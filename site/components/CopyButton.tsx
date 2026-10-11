"use client";

import { useState } from "react";

export function CopyButton({ text, label = "Copy" }: { text: string; label?: string }) {
  const [state, setState] = useState<"idle" | "done" | "failed">("idle");

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      setState("done");
    } catch {
      setState("failed");
    }
    window.setTimeout(() => setState("idle"), 1600);
  }

  return (
    <button
      type="button"
      onClick={copy}
      className="ml-auto inline-flex h-7 items-center gap-1.5 rounded-md border border-[var(--term-border)] px-2 font-mono text-[0.68rem] tracking-wide text-[var(--term-dim)] transition-colors hover:border-[var(--term-gold)] hover:text-[var(--term-fg)]"
      aria-label={`${label} to clipboard`}
    >
      <span aria-live="polite">{state === "done" ? "Copied" : state === "failed" ? "Copy failed" : label}</span>
    </button>
  );
}
