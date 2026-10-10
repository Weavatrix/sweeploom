import Link from "next/link";
import { notFound } from "next/navigation";
import { formatDate, getAllPosts, getPost } from "@/lib/blog";
import { pageMeta } from "@/lib/meta";
import { SITE } from "@/lib/site";

type Params = { slug: string };

export const dynamicParams = false;

export function generateStaticParams(): Params[] {
  return getAllPosts().map((post) => ({ slug: post.slug }));
}

export async function generateMetadata({ params }: { params: Promise<Params> }) {
  const { slug } = await params;
  const post = getAllPosts().find((p) => p.slug === slug);
  if (!post) return {};
  return pageMeta({
    title: post.title,
    description: post.summary,
    path: `/blog/${post.slug}/`,
    type: "article",
    publishedTime: post.date,
    tags: post.tags,
    image: { url: `/blog/${post.slug}/og.png`, width: 1200, height: 630, alt: post.title },
  });
}

export default async function BlogPost({ params }: { params: Promise<Params> }) {
  const { slug } = await params;
  const all = getAllPosts();
  const index = all.findIndex((p) => p.slug === slug);
  if (index === -1) notFound();
  const post = await getPost(slug);
  const newer = index > 0 ? all[index - 1] : null;
  const older = index < all.length - 1 ? all[index + 1] : null;

  const jsonLd = {
    "@context": "https://schema.org",
    "@type": "BlogPosting",
    headline: post.title,
    description: post.summary,
    datePublished: post.date,
    dateModified: post.date,
    author: { "@type": "Person", name: post.author },
    publisher: { "@type": "Organization", name: SITE.company, url: SITE.companyUrl, logo: { "@type": "ImageObject", url: `${SITE.url}/mark-512.png` } },
    mainEntityOfPage: `${SITE.url}/blog/${post.slug}/`,
    image: `${SITE.url}/blog/${post.slug}/og.png`,
    keywords: post.tags.join(", "),
  };

  return (
    <article>
      <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }} />
      <header className="relative overflow-hidden border-b border-line">
        <div className="weave-bg pointer-events-none absolute inset-0 opacity-60" aria-hidden="true" />
        <div className="wrap relative py-14 sm:py-20">
          <div className="mx-auto max-w-[42.5rem]">
          <nav aria-label="Breadcrumb" className="font-mono text-xs text-faint">
            <ol className="flex items-center gap-2">
              <li>
                <Link href="/blog/" className="hover:text-text">
                  Blog
                </Link>
              </li>
              <li aria-hidden="true">/</li>
              <li aria-current="page" className="truncate text-muted">
                {post.slug}
              </li>
            </ol>
          </nav>
          <h1 className="rise mt-5 font-display text-[2rem] leading-[1.1] font-semibold tracking-[-0.025em] text-balance sm:text-[2.8rem]">
            {post.title}
          </h1>
          <p className="rise mt-5 text-lg leading-relaxed text-pretty text-muted" style={{ animationDelay: "80ms" }}>
            {post.summary}
          </p>
          <div className="mt-6 flex flex-wrap items-center gap-x-3 gap-y-2 text-sm text-muted">
            <span>{post.author}</span>
            <span aria-hidden="true">·</span>
            <time dateTime={post.date}>{formatDate(post.date)}</time>
            <span aria-hidden="true">·</span>
            <span>{post.readingMinutes} min read</span>
          </div>
          {post.tags.length ? (
            <ul className="mt-5 flex flex-wrap gap-2" aria-label="Tags">
              {post.tags.map((tag) => (
                <li key={tag} className="chip">
                  #{tag}
                </li>
              ))}
            </ul>
          ) : null}
          </div>
        </div>
      </header>

      <div className="wrap py-12 sm:py-16">
        <div className="prose mx-auto" dangerouslySetInnerHTML={{ __html: post.html }} />

        <aside className="mx-auto mt-16 max-w-[42.5rem] rounded-2xl border border-line bg-panel p-6">
          <p className="font-display text-lg font-semibold">Try it on your own machine</p>
          <p className="mt-2 text-[0.95rem] leading-relaxed text-muted">
            SweepLoom is open source and local-first. Install from source, run <code className="inline-code">sweeploom clean .</code>,
            and nothing is deleted until you say so.
          </p>
          <div className="mt-4 flex flex-wrap gap-3">
            <Link href="/download/" className="btn btn-primary">
              Install SweepLoom
            </Link>
            <a href={SITE.newIssue} className="btn btn-ghost">
              Discuss on GitHub
            </a>
          </div>
        </aside>

        <nav aria-label="More posts" className="mx-auto mt-10 grid max-w-[42.5rem] gap-4 sm:grid-cols-2">
          {older ? (
            <Link href={`/blog/${older.slug}/`} className="card card-hover p-5">
              <span className="font-mono text-xs text-faint">← Older</span>
              <span className="mt-2 block font-medium leading-snug">{older.title}</span>
            </Link>
          ) : (
            <span />
          )}
          {newer ? (
            <Link href={`/blog/${newer.slug}/`} className="card card-hover p-5 sm:text-right">
              <span className="font-mono text-xs text-faint">Newer →</span>
              <span className="mt-2 block font-medium leading-snug">{newer.title}</span>
            </Link>
          ) : null}
        </nav>
      </div>
    </article>
  );
}
