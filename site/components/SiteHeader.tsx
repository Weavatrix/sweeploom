"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import { NAV, SITE } from "@/lib/site";
import { Wordmark } from "./Logo";
import { ThemeSwitch } from "./ThemeSwitch";
import { GitHubIcon } from "./Icons";

function trim(path: string) {
  return path.length > 1 ? path.replace(/\/+$/, "") : path;
}

function isActive(pathname: string, href: string) {
  const here = trim(pathname);
  const target = trim(href);
  return here === target || here.startsWith(target + "/");
}

export function SiteHeader() {
  const pathname = usePathname() || "/";
  const [open, setOpen] = useState(false);

  useEffect(() => setOpen(false), [pathname]);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  return (
    <header className="sticky top-0 z-50 border-b border-line bg-[var(--header)] backdrop-blur-xl backdrop-saturate-150">
      <div className="wrap flex h-16 items-center justify-between gap-4">
        <Link href="/" className="shrink-0 rounded-lg" aria-label="SweepLoom home">
          <Wordmark />
        </Link>

        <nav aria-label="Primary" className="hidden lg:block">
          <ul className="flex items-center gap-1">
            {NAV.map((item) => {
              const active = isActive(pathname, item.href);
              return (
                <li key={item.href}>
                  <Link
                    href={item.href}
                    aria-current={active ? "page" : undefined}
                    className={
                      "rounded-lg px-3 py-2 text-[0.92rem] transition-colors " +
                      (active ? "text-text" : "text-muted hover:text-text")
                    }
                  >
                    {item.label}
                  </Link>
                </li>
              );
            })}
          </ul>
        </nav>

        <div className="flex items-center gap-2">
          <div className="hidden sm:block">
            <ThemeSwitch />
          </div>
          <a
            href={SITE.repo}
            className="hidden h-9 items-center gap-2 rounded-lg border border-line px-3 text-sm text-muted transition-colors hover:border-line-strong hover:text-text sm:inline-flex"
          >
            <GitHubIcon />
            <span>GitHub</span>
          </a>
          <button
            type="button"
            className="inline-flex h-10 w-10 items-center justify-center rounded-lg border border-line text-text lg:hidden"
            aria-expanded={open}
            aria-controls="mobile-nav"
            aria-label={open ? "Close menu" : "Open menu"}
            onClick={() => setOpen((value) => !value)}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" aria-hidden="true">
              {open ? <path d="M6 6l12 12M18 6L6 18" /> : <path d="M4 7h16M4 12h16M4 17h16" />}
            </svg>
          </button>
        </div>
      </div>

      <div id="mobile-nav" hidden={!open} className="border-t border-line bg-bg lg:hidden">
        <nav aria-label="Mobile" className="wrap py-3">
          <ul className="grid gap-1">
            {NAV.map((item) => {
              const active = isActive(pathname, item.href);
              return (
                <li key={item.href}>
                  <Link
                    href={item.href}
                    aria-current={active ? "page" : undefined}
                    className={
                      "block rounded-lg px-3 py-2.5 text-base " +
                      (active ? "bg-panel text-text" : "text-muted hover:bg-panel hover:text-text")
                    }
                  >
                    {item.label}
                  </Link>
                </li>
              );
            })}
            <li>
              <a href={SITE.repo} className="flex items-center gap-2 rounded-lg px-3 py-2.5 text-base text-muted hover:bg-panel hover:text-text">
                <GitHubIcon /> GitHub
              </a>
            </li>
          </ul>
          <div className="mt-3 flex items-center justify-between border-t border-line px-3 pt-3">
            <span className="text-sm text-muted">Theme</span>
            <ThemeSwitch />
          </div>
        </nav>
      </div>
    </header>
  );
}
