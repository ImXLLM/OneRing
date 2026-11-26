"use client";

import { cn } from "@/lib/utils";

/**
 * PerspectiveSwitcherBar — the lossless perspective switcher.
 *
 * One bar, one active perspective, zero data loss: the parent keeps the
 * `MaironWorldState` (single authoritative store) mounted; this bar only
 * reports the requested perspective id. Switching never unmounts the store.
 */
export interface PerspectiveSwitcherBarProps {
  /** Perspective ids, in display order. */
  perspectives: readonly string[];
  /** The active perspective id (controlled). */
  active: string;
  /** Perspective change callback. */
  onChange: (perspective: string) => void;
  /** Compact rendering (icon-less, smaller paddings). */
  compact?: boolean;
  className?: string;
}

const LABELS: Record<string, string> = {
  toyspark: "ToySpark 64px",
  "creator-studio": "Creator Studio",
  "pro-terminal": "Pro Terminal",
  quant: "Quant Desk",
  treasury: "Treasury",
  "neural-hud": "Neural HUD",
  "war-room": "War-Room",
  forge: "Forge",
  desk: "Pocket Desk",
  wallet: "Sovereign Wallet",
};

export function PerspectiveSwitcherBar({
  perspectives,
  active,
  onChange,
  compact = false,
  className,
}: PerspectiveSwitcherBarProps) {
  return (
    <div
      data-slot="perspective-switcher-bar"
      role="tablist"
      aria-label="MAIRON perspective switcher"
      className={cn(
        "inline-flex flex-wrap items-center gap-1 rounded-xl border border-zinc-700/80 bg-zinc-900/80 p-1",
        className,
      )}
    >
      {perspectives.map((p) => {
        const isActive = p === active;
        return (
          <button
            key={p}
            type="button"
            role="tab"
            aria-selected={isActive}
            onClick={() => onChange(p)}
            className={cn(
              "rounded-lg px-3 py-1.5 text-xs font-semibold transition",
              isActive
                ? "bg-amber-400/15 text-amber-200 shadow-[inset_0_0_0_1px_rgba(251,191,36,0.35)]"
                : "text-zinc-400 hover:bg-zinc-800/80 hover:text-zinc-200",
              compact && "px-2 py-1 text-[11px]",
            )}
          >
            {LABELS[p] ?? p}
          </button>
        );
      })}
    </div>
  );
}
