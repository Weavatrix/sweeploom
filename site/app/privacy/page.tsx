import Link from "next/link";
import { LegalPage } from "@/components/LegalPage";
import { pageMeta } from "@/lib/meta";
import { SITE, docsUrl, repoFile } from "@/lib/site";

export const metadata = pageMeta({
  title: "Privacy Policy",
  description:
    "SweepLoom is local-first: no account, no telemetry, no update checks. The CLI and MCP server make no network requests. sweeploom.com has no analytics, no cookies and no trackers.",
  path: "/privacy/",
});

export default function PrivacyPage() {
  return (
    <LegalPage
      title="Privacy Policy"
      effective="2026-10-10"
      current="/privacy/"
      intro={
        <p>
          SweepLoom runs on your computer and sends nothing to us. This website has no analytics, sets no cookies and loads
          nothing from third parties.
        </p>
      }
    >
      <h2 id="short-version">The short version</h2>
      <ul>
        <li>
          <strong>No account, no telemetry.</strong> SweepLoom does not collect analytics, crash reports or usage data, and
          does not check for updates over the network.
        </li>
        <li>
          <strong>The CLI and MCP server make no network requests.</strong> Their dependency tree carries no HTTP client.
        </li>
        <li>
          <strong>The desktop app&apos;s only network use</strong> is a local Ollama server at <code>127.0.0.1:11434</code>, if
          one is running, to list and remove models.
        </li>
        <li>
          <strong>This website</strong> is a static site with no analytics, no cookies, no trackers and self-hosted fonts.
        </li>
      </ul>

      <h2 id="software">The SweepLoom software</h2>
      <p>
        This covers the <code>sweeploom</code> CLI, the MCP server, the Rust library, the desktop app, the Codex plugin and
        the browser companion extension. They are built by Sergii Ziborov at Weavatrix (“we”), and none of them sends data to
        us. We operate no server they talk to.
      </p>

      <h3 id="what-it-reads">What it reads on your computer</h3>
      <p>SweepLoom works mostly from metadata:</p>
      <ul>
        <li>
          <strong>Files and folders:</strong> names, sizes (allocated blocks), file counts and timestamps, in the folders you
          scan and in known developer, cache and AI tool locations.
        </li>
        <li>
          <strong>Processes:</strong> names, IDs and start times, parent relationships, CPU and memory use, working
          directories, and, where the operating system exposes it, network connection state. Command-line arguments are
          redacted (for example <code>--token</code> values and credentials in URLs) before they are shown or stored.
        </li>
        <li>
          <strong>Projects:</strong> project kinds and Git status, to tell live source from rebuildable output.
        </li>
        <li>
          <strong>AI tool folders:</strong> classified by file name only. SweepLoom does not open secret files such as{" "}
          <code>.credentials.json</code> or SQLite databases.
        </li>
        <li>
          <strong>Local tools you already use:</strong> cleanup in the desktop app can query and run <code>docker</code>,{" "}
          <code>xcrun simctl</code>, <code>rustup</code> and the local Ollama API. Those tools follow their own behavior and
          terms.
        </li>
      </ul>

      <h3 id="what-it-stores">What it stores, and where</h3>
      <p>
        Settings, scan history, the last browser companion snapshot and the Later shelf of saved tab titles and URLs are
        stored in SweepLoom&apos;s local config and data directories on your computer. Nothing is uploaded. You can delete
        them at any time.
      </p>

      <h3 id="browser-companion">Browser companion</h3>
      <p>
        The optional extension exchanges messages only with a local SweepLoom process through the browser&apos;s native
        messaging channel. It makes no network requests. Per tab it reports the tab and window id, title, URL with the query
        string, fragment and user info removed, <code>lastAccessed</code>, and the pinned, audible, discarded and incognito
        flags. Details:{" "}
        <a href={docsUrl("BROWSER.md")}>docs/BROWSER.md</a>.
      </p>

      <h3 id="agents">When you connect an AI agent</h3>
      <p>
        This is the one path by which SweepLoom output can leave your machine, and it is the agent&apos;s, not ours. When an
        MCP host or Codex calls SweepLoom&apos;s tools, the results (paths, sizes, process and project details) enter that
        agent&apos;s context and may be sent to the agent&apos;s model provider under that provider&apos;s terms. Choose what
        you ask your agent to inspect accordingly.
      </p>

      <h3 id="building">Building from source</h3>
      <p>
        Installing SweepLoom uses Git and Cargo, which download source from GitHub and dependencies from crates.io under
        those services&apos; own policies. That happens at install time, not when SweepLoom runs.
      </p>

      <h2 id="website">This website</h2>
      <ul>
        <li>
          <strong>No analytics, no trackers, no cookies.</strong> There is no analytics script, pixel, or third-party embed on
          any page.
        </li>
        <li>
          <strong>Fonts are self-hosted.</strong> They are bundled at build time and served from this domain, so loading a page
          makes no request to a font provider.
        </li>
        <li>
          <strong>Your theme choice</strong> (auto, light or dark) is saved in your browser&apos;s local storage under{" "}
          <code>sweeploom-theme</code>. It never leaves your browser, and you can clear it with your site data.
        </li>
        <li>
          <strong>Hosting.</strong> The site is served as static files by Cloudflare. Like any host, Cloudflare processes basic
          request data such as IP address and user agent to deliver and protect the pages, under its own privacy policy. We do
          not receive or keep visitor logs for analytics.
        </li>
        <li>
          <strong>Links.</strong> Links to GitHub and other sites take you to services with their own policies.
        </li>
      </ul>

      <h2 id="children">Children</h2>
      <p>SweepLoom is a developer tool, and this website does not knowingly collect information from anyone, including children.</p>

      <h2 id="changes">Changes</h2>
      <p>
        If this policy changes, the effective date above changes with it, and the history is kept in the repository. The
        privacy section of the <Link href="/terms/">Terms of Use</Link> (from{" "}
        <a href={repoFile("TERMS.md")}>TERMS.md</a>) says the same thing in fewer words.
      </p>

      <h2 id="contact">Contact</h2>
      <p>
        Questions go through <a href={SITE.issues}>GitHub issues</a>. For a security problem, open an issue that asks for a
        private contact and leave out exploit details and secrets.
      </p>
    </LegalPage>
  );
}
