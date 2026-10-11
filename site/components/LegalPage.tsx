import type { ReactNode } from "react";
import Link from "next/link";
import { formatDate } from "@/lib/blog";

const LEGAL = [
  { href: "/terms/", label: "Terms of Use" },
  { href: "/privacy/", label: "Privacy Policy" },
  { href: "/license/", label: "License" },
];

export function LegalPage({
  title,
  effective,
  intro,
  current,
  children,
}: {
  title: string;
  effective: string;
  intro?: ReactNode;
  current: string;
  children: ReactNode;
}) {
  return (
    <>
      <div className="border-b border-line">
        <div className="wrap py-14 sm:py-16">
          <p className="eyebrow">Legal</p>
          <h1 className="mt-3 font-display text-4xl font-semibold tracking-[-0.025em] sm:text-5xl">{title}</h1>
          <p className="mt-4 text-sm text-muted">
            Effective <time dateTime={effective}>{formatDate(effective)}</time>
          </p>
          {intro ? <div className="mt-5 max-w-2xl text-lg leading-relaxed text-muted">{intro}</div> : null}
        </div>
      </div>
      <div className="wrap grid gap-12 py-12 lg:grid-cols-[200px_minmax(0,1fr)]">
        <nav aria-label="Legal pages" className="lg:sticky lg:top-24 lg:self-start">
          <ul className="flex flex-wrap gap-2 lg:grid lg:gap-1">
            {LEGAL.map((item) => (
              <li key={item.href}>
                <Link
                  href={item.href}
                  aria-current={item.href === current ? "page" : undefined}
                  className={
                    "block rounded-lg px-3 py-2 text-sm transition-colors " +
                    (item.href === current ? "bg-panel text-text" : "text-muted hover:bg-panel hover:text-text")
                  }
                >
                  {item.label}
                </Link>
              </li>
            ))}
          </ul>
        </nav>
        <div className="prose min-w-0">{children}</div>
      </div>
    </>
  );
}
