/**
 * MAIRON formatting helpers and status labels (shared between wallet and market).
 */

/** Integer Din amount with thousands separators + the Đ symbol. */
export function formatDin(v: number): string {
  return `${v.toLocaleString("en-US")} Đ`;
}

/** Short international time string from an ISO timestamp. */
export function formatWhen(iso: string): string {
  try {
    return new Date(iso).toLocaleString("en-US", {
      dateStyle: "short",
      timeStyle: "short",
    });
  } catch {
    return iso;
  }
}

/** Label and color for the 8-state escrow machine statuses (Article 8). */
export const ESCROW_STATUS: Record<string, { fa: string; className: string }> = {
  CREATED: { fa: "created", className: "bg-zinc-500/15 text-zinc-300 ring-zinc-400/30" },
  FUNDED: { fa: "funded", className: "bg-royal-500/15 text-royal-400 ring-royal-400/40" },
  DELIVERED: { fa: "delivered", className: "bg-gold-400/15 text-gold-400 ring-gold-400/40" },
  RELEASED: { fa: "released", className: "bg-emerald-500/15 text-emerald-400 ring-emerald-400/40" },
  DISPUTED: { fa: "disputed", className: "bg-red-500/15 text-red-400 ring-red-400/40" },
  REFUNDED: { fa: "refunded", className: "bg-amber-500/15 text-amber-400 ring-amber-400/40" },
  CANCELLED: { fa: "cancelled", className: "bg-zinc-500/15 text-zinc-400 ring-zinc-400/30" },
  EXPIRED: { fa: "expired", className: "bg-zinc-600/15 text-zinc-500 ring-zinc-500/30" },
};

/** Label and color for deposit statuses. */
export const DEPOSIT_STATUS: Record<string, { fa: string; className: string }> = {
  PENDING: { fa: "awaiting admin confirmation", className: "bg-gold-400/15 text-gold-400 ring-gold-400/40" },
  CONFIRMED: { fa: "confirmed — Din minted", className: "bg-emerald-500/15 text-emerald-400 ring-emerald-400/40" },
  REJECTED: { fa: "rejected", className: "bg-red-500/15 text-red-400 ring-red-400/40" },
};
