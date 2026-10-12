"use client";

import { useEffect } from "react";
import { copyText } from "@/lib/clipboard";

/* Makes the Copy buttons that lib/markdown.ts renders into blog posts work.
   One delegated listener; the buttons stay hidden until it is attached, so
   a visitor without JavaScript never sees a dead control. Like CodeBlock,
   a block with "$ " prompt lines copies only the commands. */

function textFor(pre: HTMLElement) {
  const lines = (pre.textContent ?? "").replace(/\n$/, "").split("\n");
  const commands = lines.filter((line) => line.startsWith("$ ")).map((line) => line.slice(2).replace(/\s{2,}#.*$/, ""));
  return commands.length ? commands.join("\n") : lines.join("\n");
}

export function CodeCopy({ rootId }: { rootId: string }) {
  useEffect(() => {
    const root = document.getElementById(rootId);
    if (!root) return;
    root.setAttribute("data-copy-ready", "");

    async function onClick(event: MouseEvent) {
      const button = (event.target as HTMLElement | null)?.closest<HTMLButtonElement>(".code-copy");
      if (!button || !root?.contains(button)) return;
      const pre = button.closest(".code-wrap")?.querySelector("pre");
      if (!pre) return;
      button.textContent = (await copyText(textFor(pre))) ? "Copied" : "Copy failed";
      window.setTimeout(() => {
        button.textContent = "Copy";
      }, 1600);
    }

    root.addEventListener("click", onClick);
    return () => {
      root.removeEventListener("click", onClick);
      root.removeAttribute("data-copy-ready");
    };
  }, [rootId]);

  return null;
}
