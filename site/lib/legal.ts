import fs from "node:fs";
import path from "node:path";
import { markdownToHtml } from "./markdown";
import { SITE } from "./site";

/* Renders content/legal/TERMS.md (synced from the repo root before each
   build). The H1 and the effective-date line become page chrome; relative
   repository links are pointed at the site or GitHub. */
export async function getTerms() {
  const raw = fs.readFileSync(path.join(process.cwd(), "content", "legal", "TERMS.md"), "utf8");
  const effective = raw.match(/\*\*Effective date:\*\*\s*(\d{4}-\d{2}-\d{2})/)?.[1] ?? "2026-10-10";
  const body = raw
    .replace(/^# .*\n/, "")
    .replace(/^\*\*Effective date:\*\*.*\n/m, "")
    .replace(/\]\(LICENSE\)/g, "](/license/)")
    .replace(/\]\((docs\/[^)]+)\)/g, `](${SITE.repo}/blob/main/$1)`);
  return { effective, html: await markdownToHtml(body) };
}
