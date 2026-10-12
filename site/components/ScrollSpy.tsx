"use client";

import { useEffect } from "react";

/* Marks the link to the section currently in view with aria-current="true".
   Progressive: without JavaScript the links still jump to their sections. */
export function ScrollSpy({ navId, ids }: { navId: string; ids: string[] }) {
  useEffect(() => {
    const nav = document.getElementById(navId);
    const sections = ids.map((id) => document.getElementById(id)).filter((el): el is HTMLElement => el !== null);
    if (!nav || !sections.length || !("IntersectionObserver" in window)) return;

    const visible = new Set<string>();
    const mark = () => {
      const current = ids.find((id) => visible.has(id));
      nav.querySelectorAll<HTMLAnchorElement>("a[href^='#']").forEach((link) => {
        if (current && link.getAttribute("href") === `#${current}`) link.setAttribute("aria-current", "true");
        else link.removeAttribute("aria-current");
      });
    };
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) visible.add(entry.target.id);
          else visible.delete(entry.target.id);
        }
        mark();
      },
      { rootMargin: "-20% 0px -65% 0px" },
    );
    sections.forEach((section) => observer.observe(section));
    return () => observer.disconnect();
  }, [navId, ids]);

  return null;
}
