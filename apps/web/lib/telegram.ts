/**
 * KIAN M3 — Telegram initData extraction.
 *
 * The two official paths:
 * 1. Inside the Telegram app: `window.Telegram.WebApp.initData` — the raw signed string.
 * 2. Opening in a browser (the mini-app direct link): Telegram sends the same string in the
 *    URL fragment as `#tgWebAppData=...`.
 *
 * No identity is fabricated here — the raw string goes to the kernel so the official
 * HMAC is verified there (identity never comes from the body).
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

/** Prepare the WebApp inside Telegram — ready + expand (harmless in a browser). */
export function setupTelegramWebApp(): void {
  if (typeof window === "undefined") return;
  const app = window.Telegram?.WebApp;
  if (!app) return;
  try {
    app.ready?.();
    app.expand?.();
  } catch {
    /* Old versions — irrelevant */
  }
}

/** The raw initData string from either official path — `null` means outside Telegram. */
export function extractInitData(): string | null {
  if (typeof window === "undefined") return null;
  const fromApp = window.Telegram?.WebApp?.initData;
  if (fromApp && fromApp.trim().length > 0) return fromApp;

  // browser path: `#tgWebAppData=user%3D...%26hash%3D...`
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

/** For friendly display only — real identity comes only from kernel HMAC verification. */
export function peekTelegramName(): string | null {
  if (typeof window === "undefined") return null;
  const user = window.Telegram?.WebApp?.initDataUnsafe?.user;
  return user?.username ?? null;
}
