"use client";

import { badgeVariants } from "../components/badge";
import { cn } from "../lib/utils";

/**
 * MaironSealBadge — the sovereign verification seal.
 *
 * Renders the `MaironVerifiedSeal` gate as a compact badge: a seal id,
 * the verifying avatar and the verification tier. When the seal is not yet
 * granted the badge renders the honest "unsealed" state — never a fake
 * verified look.
 */
export interface MaironSealBadgeProps {
  /** The seal id, e.g. `seal:trinity:42` — omit when unsealed. */
  sealId?: string | null;
  /** The avatar that granted the seal (`mairon` by contract). */
  sealedBy?: string;
  /** Verification tier carried by the seal (`gold` | `silver` | `bronze`). */
  tier?: "gold" | "silver" | "bronze";
  /** The REA score backing the seal (>= 85 passes the gate). */
  reaScore?: number | null;
  className?: string;
}

const TIER_CLASS: Record<NonNullable<MaironSealBadgeProps["tier"]>, string> = {
  gold: "border-amber-400/60 bg-amber-400/10 text-amber-200",
  silver: "border-zinc-300/50 bg-zinc-300/10 text-zinc-100",
  bronze: "border-orange-400/50 bg-orange-400/10 text-orange-200",
};

export function MaironSealBadge({
  sealId,
  sealedBy = "mairon",
  tier = "gold",
  reaScore = null,
  className,
}: MaironSealBadgeProps) {
  const sealed = Boolean(sealId);
  return (
    <span
      data-slot="mairon-seal-badge"
      className={cn(
        badgeVariants({ variant: "outline" }),
        "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 font-mono text-[11px] tracking-wide",
        sealed ? TIER_CLASS[tier] : "border-zinc-700 bg-zinc-800/60 text-zinc-500",
        className,
      )}
    >
      <span aria-hidden>{sealed ? "\u2713" : "\u2715"}</span>
      <span className="font-semibold">{sealed ? "MAIRON VERIFIED" : "UNSEALED"}</span>
      {sealed && sealId ? (
        <span className="opacity-80">{sealId.length > 18 ? `${sealId.slice(0, 18)}…` : sealId}</span>
      ) : null}
      {sealed && typeof reaScore === "number" ? (
        <span className="opacity-80">REA {reaScore}</span>
      ) : null}
      {sealed ? <span className="sr-only">{`sealed by ${sealedBy}, tier ${tier}`}</span> : null}
    </span>
  );
}
