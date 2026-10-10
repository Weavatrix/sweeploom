import { ImageResponse } from "next/og";
import { MarkSvg } from "@/lib/og";

export const dynamic = "force-static";

export function GET() {
  return new ImageResponse(
    <div style={{ width: "100%", height: "100%", display: "flex", alignItems: "center", justifyContent: "center", background: "#111216" }}>
      <MarkSvg size={150} />
    </div>,
    { width: 180, height: 180 },
  );
}
