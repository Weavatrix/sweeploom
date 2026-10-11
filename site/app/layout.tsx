import type { Metadata, Viewport } from "next";
import { Inter, JetBrains_Mono, Space_Grotesk } from "next/font/google";
import { SiteHeader } from "@/components/SiteHeader";
import { SiteFooter } from "@/components/SiteFooter";
import { SITE } from "@/lib/site";
import { THEME_BOOTSTRAP } from "@/lib/theme";
import { DEFAULT_OG } from "@/lib/meta";
import "./globals.css";

// next/font downloads these at build time and serves them from /_next/static.
// No request reaches Google at runtime.
const inter = Inter({ subsets: ["latin"], variable: "--font-inter", display: "swap" });
const grotesk = Space_Grotesk({ subsets: ["latin"], variable: "--font-grotesk", display: "swap" });
const mono = JetBrains_Mono({ subsets: ["latin"], variable: "--font-jetbrains", display: "swap" });

export const metadata: Metadata = {
  metadataBase: new URL(SITE.url),
  title: {
    default: `${SITE.name} · ${SITE.tagline}`,
    template: `%s · ${SITE.name}`,
  },
  description: SITE.description,
  applicationName: SITE.name,
  authors: [{ name: SITE.author, url: SITE.companyGitHub }],
  creator: SITE.company,
  publisher: SITE.company,
  keywords: [
    "SweepLoom",
    "Weavatrix",
    "developer disk cleanup",
    "workstation resource manager",
    "MCP server",
    "Model Context Protocol",
    "coding agents",
    "Claude Code",
    "Codex",
    "Cursor",
    "Xcode DerivedData",
    "iOS Simulator cleanup",
    "Docker cleanup",
    "node_modules",
    "cargo target",
    "prompt tax",
    "local-first",
    "Rust",
  ],
  alternates: {
    canonical: "/",
    types: { "application/rss+xml": [{ url: "/rss.xml", title: "SweepLoom blog" }] },
  },
  openGraph: {
    type: "website",
    siteName: SITE.name,
    url: "/",
    title: `${SITE.name} · ${SITE.tagline}`,
    description: SITE.description,
    locale: "en_US",
    images: [DEFAULT_OG],
  },
  twitter: {
    card: "summary_large_image",
    title: `${SITE.name} · ${SITE.tagline}`,
    description: SITE.description,
    images: [DEFAULT_OG.url],
  },
  icons: {
    icon: [
      { url: "/icon.svg", type: "image/svg+xml" },
      { url: "/favicon.ico", sizes: "16x16 32x32 48x48" },
    ],
    apple: [{ url: "/apple-touch-icon.png", sizes: "180x180", type: "image/png" }],
  },
  robots: { index: true, follow: true },
  formatDetection: { telephone: false, email: false, address: false },
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: dark)", color: "#0b0d10" },
    { media: "(prefers-color-scheme: light)", color: "#f8f7f4" },
  ],
  colorScheme: "dark light",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="en"
      data-theme="auto"
      data-theme-resolved="dark"
      suppressHydrationWarning
      className={`${inter.variable} ${grotesk.variable} ${mono.variable}`}
    >
      <head>
        <script dangerouslySetInnerHTML={{ __html: THEME_BOOTSTRAP }} />
      </head>
      <body className="min-h-dvh">
        <a href="#main" className="skip-link">
          Skip to content
        </a>
        <SiteHeader />
        <main id="main" tabIndex={-1} className="outline-none">
          {children}
        </main>
        <SiteFooter />
      </body>
    </html>
  );
}
