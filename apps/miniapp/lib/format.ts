/**
 * MAIRON formatting helpers (shared across the mini-app tabs).
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
export const ESCROW_STATUS: Record<string, { label: string; className: string }> = {
  CREATED: { label: "created", className: "bg-zinc-500/15 text-zinc-300 ring-zinc-400/30" },
  FUNDED: { label: "funded", className: "bg-royal-500/15 text-royal-400 ring-royal-400/40" },
  DELIVERED: { label: "delivered", className: "bg-gold-400/15 text-gold-400 ring-gold-400/40" },
  RELEASED: { label: "released", className: "bg-emerald-500/15 text-emerald-400 ring-emerald-400/40" },
  DISPUTED: { label: "disputed", className: "bg-red-500/15 text-red-400 ring-red-400/40" },
  REFUNDED: { label: "refunded", className: "bg-amber-500/15 text-amber-400 ring-amber-400/40" },
  CANCELLED: { label: "cancelled", className: "bg-zinc-500/15 text-zinc-400 ring-zinc-400/30" },
  EXPIRED: { label: "expired", className: "bg-zinc-600/15 text-zinc-500 ring-zinc-500/30" },
};

/** Label and color for deposit statuses. */
export const DEPOSIT_STATUS: Record<string, { label: string; className: string }> = {
  PENDING: { label: "awaiting admin confirmation", className: "bg-gold-400/15 text-gold-400 ring-gold-400/40" },
  CONFIRMED: { label: "confirmed — Din minted", className: "bg-emerald-500/15 text-emerald-400 ring-emerald-400/40" },
  REJECTED: { label: "rejected", className: "bg-red-500/15 text-red-400 ring-red-400/40" },
};
