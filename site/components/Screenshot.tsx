import fs from "node:fs";
import path from "node:path";

/* Screenshots live in public/screenshots/*.png and are added later. The site
   is a static export, so existence is decided at build time: a present file
   renders as an image, a missing one renders a labelled app-window
   placeholder instead of a broken image. Rebuild after adding files. */

const SIDEBAR: { section: string; items: string[] }[] = [
  { section: "LIVE", items: ["Overview", "Sessions", "History"] },
  { section: "DISK", items: ["Review", "Explorer", "Projects", "Cleanup", "Scan history"] },
  { section: "WORKSPACE", items: ["Browser", "AI"] },
  { section: "APP", items: ["Settings"] },
];

type Props = {
  file: string;
  alt: string;
  caption: string;
  screen: string;
  width?: number;
  height?: number;
};

function exists(file: string) {
  return fs.existsSync(path.join(process.cwd(), "public", "screenshots", file));
}

export function Screenshot({ file, alt, caption, screen, width = 1600, height = 1000 }: Props) {
  const present = exists(file);
  return (
    <figure className="group">
      <div className="card overflow-hidden p-0">
        {present ? (
          <img
            src={`/screenshots/${file}`}
            alt={alt}
            width={width}
            height={height}
            loading="lazy"
            decoding="async"
            className="block h-auto w-full"
          />
        ) : (
          <div
            role="img"
            aria-label={`${alt} (screenshot coming soon)`}
            className="relative grid aspect-[16/10] grid-cols-[34%_1fr] bg-bg-2 sm:grid-cols-[26%_1fr]"
          >
            <div className="border-r border-line bg-[var(--term-bg)] p-3 sm:p-4">
              <div className="mb-3 flex gap-1.5" aria-hidden="true">
                <i className="h-2 w-2 rounded-full bg-line-strong" />
                <i className="h-2 w-2 rounded-full bg-line-strong" />
                <i className="h-2 w-2 rounded-full bg-line-strong" />
              </div>
              {SIDEBAR.map((group) => (
                <div key={group.section} className="mb-2 sm:mb-3">
                  <div className="font-mono text-[0.5rem] tracking-[0.14em] text-[var(--term-dim)] sm:text-[0.58rem]">
                    {group.section}
                  </div>
                  <ul className="mt-1 grid gap-0.5">
                    {group.items.map((item) => (
                      <li
                        key={item}
                        className={
                          "truncate rounded px-1.5 py-0.5 text-[0.55rem] sm:text-[0.7rem] " +
                          (item === screen
                            ? "border-l-2 border-gold bg-[rgba(196,140,48,0.12)] text-[var(--term-fg)]"
                            : "text-[var(--term-dim)]")
                        }
                      >
                        {item}
                      </li>
                    ))}
                  </ul>
                </div>
              ))}
            </div>
            <div className="flex flex-col p-4 sm:p-6">
              <div className="h-3 w-1/3 rounded bg-line-strong/70" />
              <div className="mt-4 grid flex-1 grid-cols-2 gap-2 sm:gap-3">
                {[0, 1, 2, 3].map((i) => (
                  <div key={i} className="rounded-lg border border-line bg-panel" />
                ))}
              </div>
              <span className="mt-4 self-start font-mono text-[0.62rem] tracking-[0.12em] text-faint uppercase">
                Screenshot coming soon
              </span>
            </div>
          </div>
        )}
      </div>
      <figcaption className="mt-3 text-sm text-muted">
        <span className="font-medium text-text">{screen}.</span> {caption}
      </figcaption>
    </figure>
  );
}
