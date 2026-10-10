import fs from "node:fs";
import path from "node:path";
import matter from "gray-matter";
import { markdownToHtml } from "./markdown";

const BLOG_DIR = path.join(process.cwd(), "content", "blog");

export type PostMeta = {
  slug: string;
  title: string;
  date: string;
  summary: string;
  tags: string[];
  author: string;
  readingMinutes: number;
};

export type Post = PostMeta & { html: string };

function slugOf(file: string) {
  return file.replace(/\.mdx?$/, "");
}

function toDateString(value: unknown): string {
  if (value instanceof Date) return value.toISOString().slice(0, 10);
  if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}$/.test(value)) return value;
  throw new Error(`blog: frontmatter date must be YYYY-MM-DD, got ${String(value)}`);
}

function readSource(slug: string) {
  for (const ext of [".md", ".mdx"]) {
    const file = path.join(BLOG_DIR, slug + ext);
    if (fs.existsSync(file)) return fs.readFileSync(file, "utf8");
  }
  throw new Error(`blog: no post named ${slug}`);
}

function metaFrom(slug: string, source: string): PostMeta & { body: string } {
  const { data, content } = matter(source);
  for (const key of ["title", "date", "summary"]) {
    if (!data[key]) throw new Error(`blog: ${slug} is missing frontmatter "${key}"`);
  }
  const words = content.trim().split(/\s+/).length;
  return {
    slug,
    title: String(data.title),
    date: toDateString(data.date),
    summary: String(data.summary),
    tags: Array.isArray(data.tags) ? data.tags.map(String) : [],
    author: data.author ? String(data.author) : "Sergii Ziborov",
    readingMinutes: Math.max(1, Math.round(words / 220)),
    body: content,
  };
}

export function getAllPosts(): PostMeta[] {
  if (!fs.existsSync(BLOG_DIR)) return [];
  return fs
    .readdirSync(BLOG_DIR)
    .filter((file) => /\.mdx?$/.test(file))
    .map((file) => {
      const slug = slugOf(file);
      const { body: _body, ...meta } = metaFrom(slug, readSource(slug));
      return meta;
    })
    .sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : a.slug.localeCompare(b.slug)));
}

export async function getPost(slug: string): Promise<Post> {
  const { body, ...meta } = metaFrom(slug, readSource(slug));
  return { ...meta, html: await markdownToHtml(body) };
}

export function formatDate(date: string) {
  return new Date(`${date}T00:00:00Z`).toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
    timeZone: "UTC",
  });
}
