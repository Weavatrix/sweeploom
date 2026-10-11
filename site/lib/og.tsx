import { MARK_GOLD, MARK_INK } from "@/components/Logo";

/* Shared drawing for build-time OG and touch icons (next/og, Satori). */

export function MarkSvg({ size }: { size: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 64 64">
      <circle cx="32" cy="32" r="21.76" fill="none" stroke={MARK_GOLD} strokeWidth="6.4" />
      <line x1="28.87" y1="11.11" x2="35.13" y2="52.89" stroke={MARK_GOLD} strokeWidth="5.76" />
      <line x1="11.03" y1="34.52" x2="52.97" y2="29.48" stroke={MARK_GOLD} strokeWidth="5.76" />
      <circle cx="32" cy="32" r="3.84" fill={MARK_INK} />
    </svg>
  );
}

export function OgCard({ kicker, title, footer }: { kicker: string; title: string; footer: string }) {
  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
        padding: "64px 72px",
        background: "#0b0d10",
        backgroundImage:
          "radial-gradient(circle at 82% 18%, rgba(196,140,48,0.28), transparent 46%), linear-gradient(rgba(255,255,255,0.035) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.035) 1px, transparent 1px)",
        backgroundSize: "100% 100%, 44px 44px, 44px 44px",
        color: "#eceef1",
        fontFamily: "sans-serif",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 18 }}>
        <MarkSvg size={64} />
        <div style={{ display: "flex", fontSize: 40, fontWeight: 700 }}>SweepLoom</div>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
        <div style={{ display: "flex", fontSize: 22, letterSpacing: 3, textTransform: "uppercase", color: "#dcae5c" }}>{kicker}</div>
        <div style={{ display: "flex", fontSize: title.length > 60 ? 54 : 66, lineHeight: 1.1, fontWeight: 700, maxWidth: 1000 }}>
          {title}
        </div>
      </div>
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: 22, color: "#a0a6b0" }}>
        <span>{footer}</span>
        <span>sweeploom.com</span>
      </div>
    </div>
  );
}
