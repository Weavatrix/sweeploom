/* SweepLoom mark, redrawn from apps/sweeploom-gui/src/mark.rs:
   a gold ring (r 0.68, width 0.2), a warp thread tilted by 0.15, a weft
   thread tilted by -0.12 (both width 0.18, inside r 0.62) and an ink hub
   (r 0.12), on a 2x2 field scaled to 64 units. */

export const MARK_GOLD = "#C48C40";
export const MARK_INK = "#5C3E18";

export function MarkPaths({ gold = MARK_GOLD, ink = MARK_INK }: { gold?: string; ink?: string }) {
  return (
    <>
      <circle cx="32" cy="32" r="21.76" fill="none" stroke={gold} strokeWidth="6.4" />
      <line x1="28.87" y1="11.11" x2="35.13" y2="52.89" stroke={gold} strokeWidth="5.76" />
      <line x1="11.03" y1="34.52" x2="52.97" y2="29.48" stroke={gold} strokeWidth="5.76" />
      <circle cx="32" cy="32" r="3.84" fill={ink} />
    </>
  );
}

export function Mark({ size = 28, className, title }: { size?: number; className?: string; title?: string }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      className={className}
      role={title ? "img" : undefined}
      aria-hidden={title ? undefined : true}
      aria-label={title}
      focusable="false"
    >
      <MarkPaths />
    </svg>
  );
}

export function Wordmark({ compact = false }: { compact?: boolean }) {
  return (
    <span className="inline-flex items-center gap-2.5">
      <Mark size={compact ? 26 : 30} />
      <span className="font-display text-[1.12rem] font-semibold tracking-[-0.01em] text-text">
        Sweep<span className="text-accent">Loom</span>
      </span>
    </span>
  );
}
