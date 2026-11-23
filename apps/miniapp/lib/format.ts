/**
 * KIAN M3 — قالب‌بندی و برچسب‌های وضعیت (مشترک بین ولت و بازار).
 */

/** عدد صحیح Din با جداکنندهٔ هزارگان + نماد Đ. */
export function formatDin(v: number): string {
  return `${v.toLocaleString("fa-IR")} Đ`;
}

/** زمان کوتاه فارسی از ISO. */
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

/** برچسب و رنگ وضعیت‌های ماشین ۸ حالتهٔ امانت (مادهٔ ۸). */
export const ESCROW_STATUS: Record<string, { fa: string; className: string }> = {
  CREATED: { fa: "ایجادشده", className: "bg-zinc-500/15 text-zinc-300 ring-zinc-400/30" },
  FUNDED: { fa: "تأمین‌شده", className: "bg-royal-500/15 text-royal-400 ring-royal-400/40" },
  DELIVERED: { fa: "تحویل‌داده‌شده", className: "bg-gold-400/15 text-gold-400 ring-gold-400/40" },
  RELEASED: { fa: "آزادشده", className: "bg-emerald-500/15 text-emerald-400 ring-emerald-400/40" },
  DISPUTED: { fa: "در اختلاف", className: "bg-red-500/15 text-red-400 ring-red-400/40" },
  REFUNDED: { fa: "بازپرداخت‌شده", className: "bg-amber-500/15 text-amber-400 ring-amber-400/40" },
  CANCELLED: { fa: "لغوشده", className: "bg-zinc-500/15 text-zinc-400 ring-zinc-400/30" },
  EXPIRED: { fa: "منقضی", className: "bg-zinc-600/15 text-zinc-500 ring-zinc-500/30" },
};

/** برچسب و رنگ وضعیت واریز TON. */
export const DEPOSIT_STATUS: Record<string, { fa: string; className: string }> = {
  PENDING: { fa: "در انتظار تأیید ادمین", className: "bg-gold-400/15 text-gold-400 ring-gold-400/40" },
  CONFIRMED: { fa: "تأییدشده — دیین ضرب شد", className: "bg-emerald-500/15 text-emerald-400 ring-emerald-400/40" },
  REJECTED: { fa: "ردشده", className: "bg-red-500/15 text-red-400 ring-red-400/40" },
};
