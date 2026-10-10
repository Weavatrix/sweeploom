import { getAllPosts } from "@/lib/blog";
import { SITE } from "@/lib/site";

// Written once at build time to out/rss.xml.
export const dynamic = "force-static";

function esc(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

function rfc822(date: string) {
  return new Date(`${date}T09:00:00Z`).toUTCString();
}

export function GET() {
  const posts = getAllPosts();
  const items = posts
    .map((post) => {
      const url = `${SITE.url}/blog/${post.slug}/`;
      return `    <item>
      <title>${esc(post.title)}</title>
      <link>${url}</link>
      <guid isPermaLink="true">${url}</guid>
      <pubDate>${rfc822(post.date)}</pubDate>
      <description>${esc(post.summary)}</description>
${post.tags.map((tag) => `      <category>${esc(tag)}</category>`).join("\n")}
    </item>`;
    })
    .join("\n");

  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>SweepLoom blog</title>
    <link>${SITE.url}/blog/</link>
    <atom:link href="${SITE.url}/rss.xml" rel="self" type="application/rss+xml" />
    <description>${esc("Notes from building SweepLoom, a local-first workstation resource manager by Weavatrix.")}</description>
    <language>en</language>
    <lastBuildDate>${posts[0] ? rfc822(posts[0].date) : new Date().toUTCString()}</lastBuildDate>
${items}
  </channel>
</rss>
`;
  return new Response(xml, { headers: { "Content-Type": "application/rss+xml; charset=utf-8" } });
}
