import { ImageResponse } from "next/og";
import { OgCard } from "@/lib/og";
import { formatDate, getAllPosts } from "@/lib/blog";

export const dynamic = "force-static";
export const dynamicParams = false;

export function generateStaticParams() {
  return getAllPosts().map((post) => ({ slug: post.slug }));
}

export async function GET(_request: Request, { params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const post = getAllPosts().find((p) => p.slug === slug);
  return new ImageResponse(
    <OgCard
      kicker={`Blog · ${post ? formatDate(post.date) : ""}`}
      title={post?.title ?? "SweepLoom blog"}
      footer="A Weavatrix product"
    />,
    { width: 1200, height: 630 },
  );
}
