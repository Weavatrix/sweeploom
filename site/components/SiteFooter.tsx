import Link from "next/link";
import { SITE } from "@/lib/site";
import { Wordmark } from "./Logo";

const COLUMNS = [
  {
    title: "Product",
    links: [
      { href: "/features/", label: "Features" },
      { href: "/download/", label: "Download" },
      { href: "/docs/", label: "Docs" },
      { href: "/changelog/", label: "Changelog" },
    ],
  },
  {
    title: "Resources",
    links: [
      { href: "/blog/", label: "Blog" },
      { href: "/rss.xml", label: "RSS feed" },
      { href: SITE.repo, label: "Source on GitHub" },
      { href: SITE.issues, label: "Issues & support" },
    ],
  },
  {
    title: "Company",
    links: [
      { href: "/about/", label: "About" },
      { href: SITE.companyUrl, label: "Weavatrix" },
      { href: "/license/", label: "License" },
      { href: "/privacy/", label: "Privacy" },
      { href: "/terms/", label: "Terms of Use" },
    ],
  },
];

function FooterLink({ href, label }: { href: string; label: string }) {
  const cls = "text-sm text-muted transition-colors hover:text-text";
  if (href.startsWith("http") || href.endsWith(".xml")) {
    return (
      <a href={href} className={cls}>
        {label}
      </a>
    );
  }
  return (
    <Link href={href} className={cls}>
      {label}
    </Link>
  );
}

export function SiteFooter() {
  return (
    <footer className="mt-24 border-t border-line bg-bg-2">
      <div className="wrap grid gap-10 py-14 md:grid-cols-[1.4fr_repeat(3,1fr)]">
        <div className="max-w-sm">
          <Wordmark compact />
          <p className="mt-4 text-sm leading-relaxed text-muted">
            Reclaim your workstation without losing your workspace. Local-first, gated apply, open source under
            MPL-2.0.
          </p>
          <p className="mt-4 text-sm text-muted">
            A{" "}
            <a href={SITE.companyUrl} className="link">
              Weavatrix
            </a>{" "}
            product.
          </p>
        </div>
        {COLUMNS.map((column) => (
          <nav key={column.title} aria-label={column.title}>
            <h2 className="eyebrow !text-faint">{column.title}</h2>
            <ul className="mt-4 grid gap-2.5">
              {column.links.map((link) => (
                <li key={link.href}>
                  <FooterLink {...link} />
                </li>
              ))}
            </ul>
          </nav>
        ))}
      </div>
      <div className="border-t border-line">
        <div className="wrap flex flex-col gap-2 py-6 text-xs text-faint sm:flex-row sm:items-center sm:justify-between">
          <p>© 2026 Weavatrix. SweepLoom is open source under the Mozilla Public License 2.0.</p>
          <p>No analytics. No cookies. No trackers.</p>
        </div>
      </div>
    </footer>
  );
}
