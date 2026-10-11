import { ImageResponse } from "next/og";
import { OgCard } from "@/lib/og";

// Rendered once at build time into out/og.png (a real .png, so static hosts
// send image/png).
export const dynamic = "force-static";

export function GET() {
  return new ImageResponse(
    <OgCard
      kicker="A Weavatrix product · local-first · MPL-2.0"
      title="Reclaim your workstation without losing your workspace."
      footer="Disk · live sessions · projects · browser · AI context"
    />,
    { width: 1200, height: 630 },
  );
}
