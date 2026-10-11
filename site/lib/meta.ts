import type { Metadata } from "next";
import { SITE } from "./site";

/* Per-page metadata. Next merges `openGraph` shallowly, so every page sets
   the full object here, image included, instead of inheriting half of it. */

export const DEFAULT_OG = { url: "/og.png", width: 1200, height: 630, alt: `${SITE.name}: ${SITE.tagline}` };

export function pageMeta({
  title,
  description,
  path,
  image,
  type = "website",
  publishedTime,
  tags,
}: {
  title: string;
  description: string;
  path: string;
  image?: { url: string; width: number; height: number; alt: string };
  type?: "website" | "article";
  publishedTime?: string;
  tags?: string[];
}): Metadata {
  const img = image ?? DEFAULT_OG;
  return {
    title,
    description,
    alternates: { canonical: path },
    openGraph: {
      type,
      siteName: SITE.name,
      url: path,
      title: `${title} · ${SITE.name}`,
      description,
      images: [img],
      locale: "en_US",
      ...(type === "article" ? { publishedTime, authors: [SITE.author], tags } : {}),
    },
    twitter: {
      card: "summary_large_image",
      title: `${title} · ${SITE.name}`,
      description,
      images: [img.url],
    },
  };
}
