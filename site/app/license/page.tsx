import Link from "next/link";
import { LegalPage } from "@/components/LegalPage";
import { pageMeta } from "@/lib/meta";
import { repoFile } from "@/lib/site";

export const metadata = pageMeta({
  title: "License",
  description:
    "SweepLoom is open source under the Mozilla Public License 2.0. What that means in practice, which components stay MIT, and how trademarks are handled.",
  path: "/license/",
});

export default function LicensePage() {
  return (
    <LegalPage
      title="License"
      effective="2026-10-10"
      current="/license/"
      intro={
        <p>
          SweepLoom is open source under the <strong className="text-text">Mozilla Public License 2.0</strong> (MPL-2.0).
        </p>
      }
    >
      <h2 id="summary">What MPL-2.0 means in practice</h2>
      <p>
        This is a plain-language summary, not legal advice and not a substitute for the license text. If they differ, the
        license wins.
      </p>
      <ul>
        <li>
          <strong>You can</strong> use SweepLoom for any purpose, including commercially, and copy, modify and distribute it.
        </li>
        <li>
          <strong>File-level copyleft.</strong> If you distribute a modified version of a SweepLoom source file, that file
          stays under MPL-2.0 and its source must be available. Your own files, combined in a “Larger Work”, can be under
          other terms.
        </li>
        <li>
          <strong>Patents.</strong> Contributors grant a patent license for their contributions, which ends for anyone who
          brings a patent claim over the software.
        </li>
        <li>
          <strong>No trademark rights.</strong> Section 2.3 of the license grants no rights in contributors&apos; names,
          trademarks or logos. You may refer to SweepLoom truthfully, for example “based on SweepLoom”.
        </li>
        <li>
          <strong>No warranty, limited liability.</strong> Sections 6 and 7: the software is provided “as is”.
        </li>
      </ul>

      <h2 id="text">Full text</h2>
      <p>
        The authoritative text is the <a href={repoFile("LICENSE")}>LICENSE</a> file in the repository. The same license is
        published by Mozilla at <a href="https://www.mozilla.org/en-US/MPL/2.0/">mozilla.org/MPL/2.0</a>.
      </p>

      <h2 id="boundary">License boundary</h2>
      <table>
        <thead>
          <tr>
            <th>Code</th>
            <th>License</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>SweepLoom (Weavatrix/sweeploom): CLI, MCP server, library, desktop app, companion extensions, Codex plugin</td>
            <td>MPL-2.0</td>
          </tr>
          <tr>
            <td>
              <code>weavatrix-scan</code>, <code>weavatrix-git</code>, <code>mcport</code>
            </td>
            <td>MIT, unchanged</td>
          </tr>
          <tr>
            <td>Other third-party dependencies</td>
            <td>Their own licenses, checked in CI with cargo-deny</td>
          </tr>
        </tbody>
      </table>
      <p>
        SweepLoom consumes the MIT Weavatrix libraries; it does not vendor, fork or relicense them.
      </p>

      <h2 id="site">This website</h2>
      <p>
        The source of sweeploom.com lives in the same repository under <code>site/</code> and is covered by the same MPL-2.0
        license. Product names and logos of third parties shown on this site belong to their owners; see the trademark
        section of the <Link href="/terms/">Terms of Use</Link>.
      </p>
    </LegalPage>
  );
}
