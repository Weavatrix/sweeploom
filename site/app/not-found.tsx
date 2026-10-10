import Link from "next/link";
import type { Metadata } from "next";
import { Mark } from "@/components/Logo";

export const metadata: Metadata = {
  title: "Page not found",
  robots: { index: false, follow: true },
};

export default function NotFound() {
  return (
    <div className="relative overflow-hidden">
      <div className="weave-bg pointer-events-none absolute inset-0 opacity-70" aria-hidden="true" />
      <div className="wrap relative flex min-h-[60vh] flex-col items-start justify-center py-24">
        <Mark size={56} />
        <p className="eyebrow mt-8">404 · not found</p>
        <h1 className="mt-3 max-w-2xl font-display text-4xl font-semibold tracking-tight text-balance sm:text-5xl">
          This path was swept. Unlike <code className="font-mono text-[0.85em] text-accent">target/</code>, it will not
          rebuild itself.
        </h1>
        <p className="mt-5 max-w-xl text-lg text-muted">
          The page you asked for does not exist, or it moved. Nothing else was deleted.
        </p>
        <div className="mt-8 flex flex-wrap gap-3">
          <Link href="/" className="btn btn-primary">
            Back to home
          </Link>
          <Link href="/docs/" className="btn btn-ghost">
            Docs
          </Link>
          <Link href="/blog/" className="btn btn-ghost">
            Blog
          </Link>
        </div>
      </div>
    </div>
  );
}
