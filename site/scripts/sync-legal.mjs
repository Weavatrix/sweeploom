// Copies the repository's TERMS.md into content/legal/ so the site's Terms
// page always renders the same text as the repo. Runs before every build
// (npm "prebuild"). When the site is built outside the repo, the committed
// copy is used as-is.
import { copyFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const source = path.resolve(here, "..", "..", "TERMS.md");
const target = path.resolve(here, "..", "content", "legal", "TERMS.md");

if (existsSync(source)) {
  copyFileSync(source, target);
  console.log("sync-legal: copied ../TERMS.md");
} else {
  console.log("sync-legal: ../TERMS.md not found; using committed copy");
}
