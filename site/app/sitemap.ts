import type { MetadataRoute } from "next";
import { getAllPosts } from "@/lib/blog";
import { SITE } from "@/lib/site";

export const dynamic = "force-static";

const PAGES: { path: string; priority: number; changeFrequency: "weekly" | "monthly" | "yearly" }[] = [
  { path: "/", priority: 1, changeFrequency: "weekly" },
  { path: "/features/", priority: 0.9, changeFrequency: "monthly" },
  { path: "/download/", priority: 0.9, changeFrequency: "monthly" },
  { path: "/docs/", priority: 0.8, changeFrequency: "monthly" },
  { path: "/blog/", priority: 0.7, changeFrequency: "weekly" },
  { path: "/changelog/", priority: 0.6, changeFrequency: "weekly" },
  { path: "/about/", priority: 0.5, changeFrequency: "yearly" },
  { path: "/license/", priority: 0.3, changeFrequency: "yearly" },
  { path: "/privacy/", priority: 0.3, changeFrequency: "yearly" },
  { path: "/terms/", priority: 0.3, changeFrequency: "yearly" },
];

export default function sitemap(): MetadataRoute.Sitemap {
  const posts = getAllPosts();
  const latest = posts[0]?.date ?? "2026-10-10";
  return [
    ...PAGES.map((page) => ({
      url: `${SITE.url}${page.path}`,
      lastModified: page.path === "/" || page.path === "/blog/" ? latest : "2026-10-10",
      changeFrequency: page.changeFrequency,
      priority: page.priority,
    })),
    ...posts.map((post) => ({
      url: `${SITE.url}/blog/${post.slug}/`,
      lastModified: post.date,
      changeFrequency: "yearly" as const,
      priority: 0.6,
    })),
  ];
}
