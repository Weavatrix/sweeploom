import Link from "next/link";
import { PageHeader } from "@/components/Section";
import { icons } from "@/components/Icons";
import { formatDate, getAllPosts } from "@/lib/blog";
import { pageMeta } from "@/lib/meta";

export const metadata = pageMeta({
  title: "Blog",
  description:
    "Notes from building SweepLoom: live agent sessions, safe disk cleanup on developer machines, Xcode and iOS Simulator storage, and the always-on prompt tax.",
  path: "/blog/",
});

export default function BlogIndex() {
  const posts = getAllPosts();
  return (
    <>
      <PageHeader
        eyebrow="Blog"
        title="Notes on reclaiming space without losing work."
        lead="How SweepLoom decides what is safe, what it measured, and where the numbers stop meaning what they seem to."
      >
        <a href="/rss.xml" className="btn btn-ghost">
          <icons.rss size={16} /> Subscribe via RSS
        </a>
      </PageHeader>
      <div className="wrap py-14">
        <ol className="grid gap-5">
          {posts.map((post) => (
            <li key={post.slug}>
              <article className="card card-hover relative p-6 sm:p-8">
                <div className="flex flex-wrap items-center gap-x-3 gap-y-2 font-mono text-xs text-faint">
                  <time dateTime={post.date}>{formatDate(post.date)}</time>
                  <span aria-hidden="true">·</span>
                  <span>{post.readingMinutes} min read</span>
                </div>
                <h2 className="mt-3 max-w-3xl font-display text-2xl leading-snug font-semibold tracking-tight text-balance">
                  <Link href={`/blog/${post.slug}/`} className="after:absolute after:inset-0 after:rounded-2xl">
                    {post.title}
                  </Link>
                </h2>
                <p className="mt-3 max-w-3xl leading-relaxed text-muted">{post.summary}</p>
                {post.tags.length ? (
                  <ul className="mt-5 flex flex-wrap gap-2" aria-label="Tags">
                    {post.tags.map((tag) => (
                      <li key={tag} className="chip">
                        #{tag}
                      </li>
                    ))}
                  </ul>
                ) : null}
              </article>
            </li>
          ))}
        </ol>
      </div>
    </>
  );
}
