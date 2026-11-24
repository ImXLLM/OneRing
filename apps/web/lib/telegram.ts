/**
 * KIAN M3 — استخراج initData تلگرام.
 *
 * دو مسیر رسمی:
 * 1. داخل اپ تلگرام: `window.Telegram.WebApp.initData` — رشتهٔ خام امضاشده.
 * 2. باز شدن در مرورگر (لینک مستقیم مینی‌اپ): تلگرام همان رشته را در
 *    URL fragment به‌شکل `#tgWebAppData=...` می‌فرستد.
 *
 * هیچ هویتی اینجا ساخته نمی‌شود — فقط رشتهٔ خام به کرنل می‌رود تا HMAC
 * رسمی همان‌جا تأیید شود (هویت هرگز از بدنه نمی‌آید).
 */

type TelegramWebApp = {
  initData?: string;
  initDataUnsafe?: { user?: { id: number; username?: string } };
  ready?: () => void;
  expand?: () => void;
};

declare global {
  interface Window {
    Telegram?: { WebApp?: TelegramWebApp };
  }
}

/** آماده‌سازی WebApp داخل تلگرام — ready + expand (بی‌ضرر در مرورگر). */
export function setupTelegramWebApp(): void {
  if (typeof window === "undefined") return;
  const app = window.Telegram?.WebApp;
  if (!app) return;
  try {
    app.ready?.();
    app.expand?.();
  } catch {
    /* نسخه‌های قدیمی — بی‌اهمیت */
  }
}

/** رشتهٔ خام initData از هر مسیر رسمی — `null` یعنی خارج از تلگرام. */
export function extractInitData(): string | null {
  if (typeof window === "undefined") return null;
  const fromApp = window.Telegram?.WebApp?.initData;
  if (fromApp && fromApp.trim().length > 0) return fromApp;

  // مسیر مرورگر: `#tgWebAppData=user%3D...%26hash%3D...`
  const fragment = window.location.hash.replace(/^#/, "");
  const match = /(?:^|&)tgWebAppData=([^&]+)/.exec(fragment);
  if (match) {
    try {
      const decoded = decodeURIComponent(match[1]);
      if (decoded.includes("hash=")) return decoded;
    } catch {
      return null;
    }
  }
  return null;
}

/** فقط برای نمایشِ دوستانه — هویت واقعی فقط از تأیید HMAC کرنل می‌آید. */
export function peekTelegramName(): string | null {
  if (typeof window === "undefined") return null;
  const user = window.Telegram?.WebApp?.initDataUnsafe?.user;
  return user?.username ?? null;
}
