import path from "node:path";
import type { NextConfig } from "next";

// Static export: `next build` writes the whole site to ./out.
// trailingSlash makes /about/ resolve to out/about/index.html on Workers
// static assets, GitHub Pages and Cloudflare Pages alike.
const nextConfig: NextConfig = {
  output: "export",
  trailingSlash: true,
  images: { unoptimized: true },
  poweredByHeader: false,
  reactStrictMode: true,
  // The site lives inside the Rust repo; its own node_modules are here.
  turbopack: { root: path.resolve(process.cwd()) },
};

export default nextConfig;
