/**
 * KIAN M3 — formatting and status labels (shared between wallet and market).
 */

/** Integer Din amount with thousands separators + the Đ symbol. */
export function formatDin(v: number): string {
  return `${v.toLocaleString("fa-IR")} Đ`;
}

/** Short Persian time string from an ISO timestamp. */
export function formatWhen(iso: string): string {
  try {
    return new Date(iso).toLocaleString("fa-IR", {
      dateStyle: "short",
      timeStyle: "short",
    });
  } catch {
    return iso;
  }
}

/** Label and color for the 8-state escrow machine statuses (Article 8). */
export const ESCROW_STATUS: Record<string, { fa: string; className: string }> = {
  CREATED: { fa: "\u0627\u06cc\u062c\u0627\u062f\u0634\u062f\u0647", className: "bg-zinc-500/15 text-zinc-300 ring-zinc-400/30" },
  FUNDED: { fa: "\u062a\u0623\u0645\u06cc\u0646‌\u0634\u062f\u0647", className: "bg-royal-500/15 text-royal-400 ring-royal-400/40" },
  DELIVERED: { fa: "\u062a\u062d\u0648\u06cc\u0644‌\u062f\u0627\u062f\u0647‌\u0634\u062f\u0647", className: "bg-gold-400/15 text-gold-400 ring-gold-400/40" },
  RELEASED: { fa: "\u0622\u0632\u0627\u062f\u0634\u062f\u0647", className: "bg-emerald-500/15 text-emerald-400 ring-emerald-400/40" },
  DISPUTED: { fa: "\u062f\u0631 \u0627\u062e\u062a\u0644\u0627\u0641", className: "bg-red-500/15 text-red-400 ring-red-400/40" },
  REFUNDED: { fa: "\u0628\u0627\u0632\u067e\u0631\u062f\u0627\u062e\u062a‌\u0634\u062f\u0647", className: "bg-amber-500/15 text-amber-400 ring-amber-400/40" },
  CANCELLED: { fa: "\u0644\u063a\u0648\u0634\u062f\u0647", className: "bg-zinc-500/15 text-zinc-400 ring-zinc-400/30" },
  EXPIRED: { fa: "\u0645\u0646\u0642\u0636\u06cc", className: "bg-zinc-600/15 text-zinc-500 ring-zinc-500/30" },
};

/** Label and color for TON deposit statuses. */
export const DEPOSIT_STATUS: Record<string, { fa: string; className: string }> = {
  PENDING: { fa: "\u062f\u0631 \u0627\u0646\u062a\u0638\u0627\u0631 \u062a\u0623\u06cc\u06cc\u062f \u0627\u062f\u0645\u06cc\u0646", className: "bg-gold-400/15 text-gold-400 ring-gold-400/40" },
  CONFIRMED: { fa: "\u062a\u0623\u06cc\u06cc\u062f\u0634\u062f\u0647 — \u062f\u06cc\u06cc\u0646 \u0636\u0631\u0628 \u0634\u062f", className: "bg-emerald-500/15 text-emerald-400 ring-emerald-400/40" },
  REJECTED: { fa: "\u0631\u062f\u0634\u062f\u0647", className: "bg-red-500/15 text-red-400 ring-red-400/40" },
};
