import type { ReactNode } from "react";

export function Section({
  id,
  eyebrow,
  title,
  lead,
  children,
  className,
}: {
  id?: string;
  eyebrow?: string;
  title: ReactNode;
  lead?: ReactNode;
  children?: ReactNode;
  className?: string;
}) {
  const headingId = id ? `${id}-title` : undefined;
  return (
    <section id={id} aria-labelledby={headingId} className={"py-16 sm:py-20 " + (className ?? "")}>
      <div className="wrap">
        <div className="max-w-2xl">
          {eyebrow ? <p className="eyebrow">{eyebrow}</p> : null}
          <h2 id={headingId} className="mt-3 font-display text-[1.75rem] leading-tight font-semibold tracking-[-0.02em] text-balance sm:text-[2.2rem]">
            {title}
          </h2>
          {lead ? <p className="mt-4 text-[1.05rem] leading-relaxed text-pretty text-muted">{lead}</p> : null}
        </div>
        {children ? <div className="mt-10">{children}</div> : null}
      </div>
    </section>
  );
}

export function PageHeader({ eyebrow, title, lead, children }: { eyebrow: string; title: ReactNode; lead?: ReactNode; children?: ReactNode }) {
  return (
    <div className="relative overflow-hidden border-b border-line">
      <div className="weave-bg pointer-events-none absolute inset-0 opacity-70" aria-hidden="true" />
      <div className="gold-glow pointer-events-none absolute -top-40 left-1/2 h-[28rem] w-[48rem] -translate-x-1/2" aria-hidden="true" />
      <div className="wrap relative py-14 sm:py-20">
        <p className="eyebrow rise">{eyebrow}</p>
        <h1 className="rise mt-3 max-w-3xl font-display text-[2.2rem] leading-[1.08] font-semibold tracking-[-0.025em] text-balance sm:text-5xl" style={{ animationDelay: "60ms" }}>
          {title}
        </h1>
        {lead ? (
          <p className="rise mt-5 max-w-2xl text-lg leading-relaxed text-pretty text-muted" style={{ animationDelay: "120ms" }}>
            {lead}
          </p>
        ) : null}
        {children ? <div className="rise mt-7" style={{ animationDelay: "180ms" }}>{children}</div> : null}
      </div>
    </div>
  );
}

export function Faq({ items }: { items: { q: string; a: ReactNode }[] }) {
  return (
    <div className="divide-y divide-line overflow-hidden rounded-2xl border border-line bg-panel">
      {items.map((item) => (
        <details key={item.q} className="group">
          <summary className="flex cursor-pointer list-none items-center justify-between gap-4 px-5 py-4 text-left font-medium transition-colors hover:bg-panel-2 sm:px-6 [&::-webkit-details-marker]:hidden">
            <span>{item.q}</span>
            <span
              aria-hidden="true"
              className="grid h-6 w-6 shrink-0 place-items-center rounded-full border border-line text-muted transition-transform group-open:rotate-45"
            >
              +
            </span>
          </summary>
          <div className="px-5 pb-5 text-[0.97rem] leading-relaxed text-muted sm:px-6 [&_a]:text-accent [&_a]:underline [&_code]:font-mono [&_code]:text-[0.86em] [&_code]:text-text">
            {item.a}
          </div>
        </details>
      ))}
    </div>
  );
}
