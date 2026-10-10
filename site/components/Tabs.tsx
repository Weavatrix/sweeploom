"use client";

import { useId, useRef, useState, type ReactNode, type KeyboardEvent } from "react";

/* WAI-ARIA tabs with automatic activation and arrow/Home/End keys.
   Panels are rendered on the server and passed in, so every panel is in the
   static HTML; inactive ones are `hidden`. */

export type TabItem = { id: string; label: string; hint?: string; content: ReactNode };

export function Tabs({ items, label }: { items: TabItem[]; label: string }) {
  const [active, setActive] = useState(0);
  const base = useId();
  const refs = useRef<(HTMLButtonElement | null)[]>([]);

  function onKey(event: KeyboardEvent<HTMLDivElement>) {
    const last = items.length - 1;
    let next = active;
    if (event.key === "ArrowRight") next = active === last ? 0 : active + 1;
    else if (event.key === "ArrowLeft") next = active === 0 ? last : active - 1;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = last;
    else return;
    event.preventDefault();
    setActive(next);
    refs.current[next]?.focus();
  }

  return (
    <div>
      <div
        role="tablist"
        aria-label={label}
        onKeyDown={onKey}
        className="inline-flex max-w-full gap-1 overflow-x-auto rounded-xl border border-line bg-bg-2 p-1"
      >
        {items.map((item, index) => (
          <button
            key={item.id}
            ref={(node) => {
              refs.current[index] = node;
            }}
            id={`${base}-tab-${item.id}`}
            role="tab"
            type="button"
            aria-selected={index === active}
            aria-controls={`${base}-panel-${item.id}`}
            tabIndex={index === active ? 0 : -1}
            onClick={() => setActive(index)}
            className="flex shrink-0 items-baseline gap-2 rounded-lg border border-transparent px-3.5 py-2 text-sm font-medium text-muted transition-colors hover:text-text sm:px-4"
          >
            {item.label}
            {item.hint ? <span className="hidden font-mono text-[0.68rem] text-faint sm:inline">{item.hint}</span> : null}
          </button>
        ))}
      </div>
      {items.map((item, index) => (
        <div
          key={item.id}
          id={`${base}-panel-${item.id}`}
          role="tabpanel"
          aria-labelledby={`${base}-tab-${item.id}`}
          hidden={index !== active}
          tabIndex={0}
          className="mt-6 rounded-xl focus-visible:outline-offset-4"
        >
          {item.content}
        </div>
      ))}
    </div>
  );
}
