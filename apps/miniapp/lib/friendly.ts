/**
 * ترجمهٔ کدهای خطای کرنل به فارسیِ انسانی، کوتاه و راهنما (UX لایهٔ نمایش).
 *
 * کدها از پاکت استاندارد `{ error: { code, message } }` کرنل Rust می‌آیند؛
 * اینجا فقط برای نمایش بازنویسی می‌شوند — هیچ منطق امنیتی اینجا نیست.
 */

import { ApiError } from "@kian/sdk";

const FRIENDLY_CODES: Record<string, string> = {
  ACCOUNT_OVERDRAFT:
    "موجودی کیف‌پول کافی نیست — اول کیف را شارژ کن یا با لینک رفرال درآمد بساز.",
  INSUFFICIENT_BALANCE:
    "موجودی کیف‌پول کافی نیست — اول کیف را شارژ کن یا با لینک رفرال درآمد بساز.",
  UNKNOWN_ACCOUNT: "این حساب پیدا نشد — شاید نشست قدیمی است؛ یک‌بار دیگر تلاش کن.",
  UNKNOWN_ESCROW: "این معامله پیدا نشد — شاید حذف شده یا نشست قدیمی است.",
  INVALID_STATE: "این معامله الان در این وضعیت قابل انجام نیست — صفحه را تازه کن.",
  REQUIRES_HUMAN_APPROVAL:
    "این مبلغ بزرگ است و به تأیید ادمین نیاز دارد — درخواستت در صف بررسی قرار گرفت.",
  SELF_PURCHASE: "نمی‌توانی محصول خودت را بخری.",
  RATE_LIMITED: "کمی آرام‌تر — چند لحظه بعد دوباره تلاش کن.",
  SESSION_REQUIRED: "نشستت منقضی شده — دوباره وارد شو.",
  SESSION_EXPIRED: "نشستت منقضی شده — دوباره وارد شو.",
  INVALID_INIT_DATA: "احراز هویت تلگرام پذیرفته نشد — مینی‌اپ را ببند و دوباره باز کن.",
  IDENTITY_DISABLED:
    "ورود با تلگرام فعلاً خاموش است — کمی بعد دوباره تلاش کن.",
  MINT_CAP_EXCEEDED:
    "سقف ضرب سالانهٔ پول پر شده — این محدودیت منشور است، نه مشکل حساب تو.",
  VALIDATION: "اطلاعات فرم کامل یا معتبر نیست — دوباره بررسی کن.",
  BODY_TOO_LARGE: "حجم محتوای ارسالی بیش از حد مجاز است (سقف HTML: ۱۵۰ کیلوبایت).",
  NETWORK: "اتصال به کرنل برقرار نشد — اینترنت یا VPN را چک کن و دوباره تلاش کن.",
  UNKNOWN: "یک خطای پیش‌بینی‌نشده رخ داد — دوباره تلاش کن؛ اگر تکرار شد به پشتیبانی خبر بده.",
};

/** پیام دوستانه برای هر خطای API یا شبکه — همیشه فارسیِ قابل‌فهم کاربر. */
export function friendlyError(err: unknown): string {
  if (err instanceof ApiError) {
    return FRIENDLY_CODES[err.code] ?? `${err.message || "خطای کرنل"} — دوباره تلاش کن.`;
  }
  if (err instanceof Error && err.message) {
    if (/fetch|network|Failed to fetch/i.test(err.message)) {
      return FRIENDLY_CODES.NETWORK;
    }
    return err.message;
  }
  return FRIENDLY_CODES.UNKNOWN;
}

/** عنوان کوتاه برای توست خطا بر اساس نوع خطا. */
export function friendlyErrorTitle(err: unknown): string {
  if (err instanceof ApiError && (err.code === "ACCOUNT_OVERDRAFT" || err.code === "INSUFFICIENT_BALANCE")) {
    return "موجودی کافی نیست";
  }
  if (err instanceof ApiError && (err.code === "SESSION_REQUIRED" || err.code === "SESSION_EXPIRED")) {
    return "نشست منقضی شد";
  }
  return "انجام نشد";
}
