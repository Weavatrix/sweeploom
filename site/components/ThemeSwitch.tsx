"use client";

import { useEffect, useState } from "react";

/* Same contract as weavatrix.com: auto | light | dark, stored in
   localStorage only (no cookie, nothing sent anywhere). The <head> bootstrap
   in app/layout.tsx stamps the attributes before first paint. */

import { THEME_KEY as KEY } from "@/lib/theme";
type Choice = "auto" | "light" | "dark";
const CHOICES: Choice[] = ["auto", "light", "dark"];

function read(): Choice {
  try {
    const value = localStorage.getItem(KEY);
    return value === "light" || value === "dark" ? value : "auto";
  } catch {
    return "auto";
  }
}

function resolve(choice: Choice): "light" | "dark" {
  if (choice !== "auto") return choice;
  return window.matchMedia?.("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

function paint(choice: Choice) {
  const root = document.documentElement;
  root.setAttribute("data-theme", choice);
  root.setAttribute("data-theme-resolved", resolve(choice));
}

const LABEL: Record<Choice, string> = {
  auto: "Match system theme",
  light: "Light theme",
  dark: "Dark theme",
};

function Icon({ choice }: { choice: Choice }) {
  const common = {
    width: 15,
    height: 15,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.8,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "aria-hidden": true,
  };
  if (choice === "light")
    return (
      <svg {...common}>
        <circle cx="12" cy="12" r="4" />
        <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
      </svg>
    );
  if (choice === "dark")
    return (
      <svg {...common}>
        <path d="M20.5 14.5A8.5 8.5 0 0 1 9.5 3.5a8.5 8.5 0 1 0 11 11Z" />
      </svg>
    );
  return (
    <svg {...common}>
      <rect x="3" y="4" width="18" height="12" rx="2" />
      <path d="M8 20h8M12 16v4" />
    </svg>
  );
}

export function ThemeSwitch() {
  const [choice, setChoice] = useState<Choice>("auto");

  useEffect(() => {
    setChoice(read());
    const media = window.matchMedia?.("(prefers-color-scheme: light)");
    const follow = () => {
      if (read() === "auto") paint("auto");
    };
    const sync = (event: StorageEvent) => {
      if (event.key === KEY) {
        const next = read();
        setChoice(next);
        paint(next);
      }
    };
    media?.addEventListener?.("change", follow);
    window.addEventListener("storage", sync);
    return () => {
      media?.removeEventListener?.("change", follow);
      window.removeEventListener("storage", sync);
    };
  }, []);

  function choose(next: Choice) {
    try {
      localStorage.setItem(KEY, next);
    } catch {
      /* private mode: the choice still applies to this page */
    }
    setChoice(next);
    paint(next);
  }

  return (
    <div
      role="group"
      aria-label="Color theme"
      className="inline-flex items-center gap-0.5 rounded-full border border-line bg-panel p-0.5"
    >
      {CHOICES.map((item) => (
        <button
          key={item}
          type="button"
          data-theme-choice={item}
          aria-pressed={choice === item}
          aria-label={LABEL[item]}
          title={LABEL[item]}
          onClick={() => choose(item)}
          className="inline-flex h-7 w-7 items-center justify-center rounded-full text-muted transition-colors hover:text-text"
        >
          <Icon choice={item} />
        </button>
      ))}
    </div>
  );
}

