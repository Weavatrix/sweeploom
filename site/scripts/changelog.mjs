// Regenerates content/changelog.json from the SweepLoom repository history.
// Run from site/: `npm run changelog`. The JSON is committed so builds do not
// depend on a full (non-shallow) clone being available in CI.
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "..", "..");
const out = path.resolve(here, "..", "content", "changelog.json");

const SEP = "\u001f";
const raw = execFileSync(
  "git",
  ["-C", repo, "log", "--no-merges", `--format=%h${SEP}%ad${SEP}%s`, "--date=short"],
  { encoding: "utf8" },
);

const commits = raw
  .split("\n")
  .filter(Boolean)
  .map((line) => {
    const [hash, date, subject] = line.split(SEP);
    return { hash, date, subject: subject.replace(/\.$/, "") };
  });

writeFileSync(out, JSON.stringify({ generated: new Date().toISOString().slice(0, 10), commits }, null, 2) + "\n");
console.log(`wrote ${commits.length} commits to ${path.relative(process.cwd(), out)}`);
