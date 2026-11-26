/**
 * Tracking Telegram viral links and storing the referrer id (Phase 5).
 *
 * Standard sale and referral link format:
 * - `p_<productId>_r_<refTgId>` (aligned with `shareStartParam` in the Rust kernel)
 * - `p_<productId>` (direct product link without a referrer)
 * - `?startapp=...` or `?tgWebAppStartParam=...` or `?ref=<refTgId>`
 */

import { buildViralShareLink } from "@/lib/kian-tma-bridge";

export const ACTIVE_REFERRER_KEY = "kian_active_referrer";
const REDIRECTED_PRODUCT_KEY = "kian_viral_redirected_product";

export type ParsedViralParam = {
  productId: string | null;
  referrer: string | null;
};

/** Extract the product id and numeric referrer id from the `start_param` / `startapp` parameter. */
export function parseViralStartParam(
  raw: string | null | undefined,
): ParsedViralParam {
  if (!raw) return { productId: null, referrer: null };
  let decoded = raw.trim();
  try {
    decoded = decodeURIComponent(decoded).trim();
  } catch {
    /* The raw string is preserved */
  }
  if (!decoded) return { productId: null, referrer: null };

  // primary pattern: p_<productId>_r_<refTgId>
  const fullMatch = /^p_(.+)_r_([0-9]+)$/.exec(decoded);
  if (fullMatch) {
    return { productId: fullMatch[1], referrer: fullMatch[2] };
  }

  // direct product pattern: p_<productId>
  const prodOnly = /^p_(.+)$/.exec(decoded);
  if (prodOnly) {
    return { productId: prodOnly[1], referrer: null };
  }

  // referrer-only pattern: r_<refTgId> or ref_<refTgId>
  const refOnly = /^(?:r|ref)_([0-9]+)$/.exec(decoded);
  if (refOnly) {
    return { productId: null, referrer: refOnly[1] };
  }

  return { productId: null, referrer: null };
}

/** Normalize the numeric referrer id (digits only or `citizen:<digits>`). */
export function normalizeReferrerId(raw: string | null | undefined): string | null {
  if (!raw) return null;
  const trimmed = raw.trim();
  if (/^[0-9]+$/.test(trimmed)) return trimmed;
  const citizenMatch = /^citizen:([0-9]+)$/.exec(trimmed);
  if (citizenMatch) return citizenMatch[1];
  return null;
}

/** Store the active referrer in sessionStorage and localStorage for automatic submission at purchase. */
export function saveActiveReferrer(ref: string | null): void {
  if (typeof window === "undefined") return;
  const clean = normalizeReferrerId(ref);
  if (!clean) return;
  try {
    window.sessionStorage.setItem(ACTIVE_REFERRER_KEY, clean);
  } catch {
    /* Browser-restricted mode */
  }
  try {
    window.localStorage.setItem(ACTIVE_REFERRER_KEY, clean);
  } catch {
    /* Incognito mode */
  }
}

/** Read the active referrer (sessionStorage first, then localStorage). */
export function getActiveReferrer(): string | null {
  if (typeof window === "undefined") return null;
  try {
    const s = normalizeReferrerId(window.sessionStorage.getItem(ACTIVE_REFERRER_KEY));
    if (s) return s;
  } catch {
    /* Ignored */
  }
  try {
    return normalizeReferrerId(window.localStorage.getItem(ACTIVE_REFERRER_KEY));
  } catch {
    return null;
  }
}

/**
 * Read the entry parameter from Telegram (`initDataUnsafe.start_param`) or the URL
 * (`?startapp=`, `?tgWebAppStartParam=`, `?ref=`, `?product=`) and store the referrer.
 */
export function captureInboundViralLink(): ParsedViralParam {
  if (typeof window === "undefined") {
    return { productId: null, referrer: null };
  }

  const params = new URLSearchParams(window.location.search);
  const tgWindow = window as Window & {
    Telegram?: { WebApp?: { initDataUnsafe?: { start_param?: string } } };
  };
  const rawStart =
    tgWindow.Telegram?.WebApp?.initDataUnsafe?.start_param ??
    params.get("startapp") ??
    params.get("tgWebAppStartParam");

  const parsed = parseViralStartParam(rawStart);
  const queryRef = normalizeReferrerId(params.get("ref"));
  const queryProduct = params.get("product")?.trim() || null;

  const referrer = parsed.referrer ?? queryRef;
  const productId = parsed.productId ?? queryProduct;

  if (referrer) {
    saveActiveReferrer(referrer);
  }

  return { productId, referrer };
}

/** Prevent a duplicate redirect for one product within the same browser session. */
export function consumePendingProductRedirect(productId: string | null): string | null {
  if (typeof window === "undefined" || !productId) return null;
  try {
    const prev = window.sessionStorage.getItem(REDIRECTED_PRODUCT_KEY);
    if (prev === productId) return null;
    window.sessionStorage.setItem(REDIRECTED_PRODUCT_KEY, productId);
    return productId;
  } catch {
    return productId;
  }
}

/** Build ready-made viral and referral Telegram links for any product. */
export function buildTelegramViralLink(
  productId: string,
  referrerTgId: string,
): { startParam: string; telegramUrl: string; webUrl: string } {
  const startParam = `p_${productId}_r_${referrerTgId}`;
  const botUsername = (
    (typeof process !== "undefined" &&
      process.env?.NEXT_PUBLIC_TELEGRAM_BOT_USERNAME) ||
    "KianAgentBot"
  ).replace(/^@/, "");
  const telegramUrl = `https://t.me/${botUsername}?startapp=${encodeURIComponent(startParam)}`;
  const origin =
    typeof window !== "undefined" ? window.location.origin : "";
  const webUrl = `${origin}/market?startapp=${encodeURIComponent(startParam)}`;
  return { startParam, telegramUrl, webUrl };
}

/** International display of the commission percent from basis points (e.g. `1000` -> `10%`). */
export function formatAffiliatePercent(bps: number): string {
  const pct = bps / 100;
  return `${pct.toLocaleString("en-US", { maximumFractionDigits: 1 })}%`;
}

/**
 * One-click product viral links — aligned with the `kian-tma-bridge.ts` adapter
 * and the standard Rust kernel format: `p_<productId>_r_<refTgId>`.
 *
 * * `startParam` — raw parameter for the kernel’s `shareStartParam`
 * * `telegramUrl` — direct `t.me/<bot>?startapp=…` link (automatic login with the payload)
 * * `webUrl` — mini-app web link with the same payload
 * * `shareUrl` — Telegram share sheet (`t.me/share/url?…`) with suggested text
 */
export function buildViralShareLinks(
  productId: string,
  referrerTgId: string,
  shareText?: string,
): {
  startParam: string;
  telegramUrl: string;
  webUrl: string;
  shareUrl: string;
} {
  const base = buildTelegramViralLink(productId, referrerTgId);
  const botUsername = (
    (typeof process !== "undefined" &&
      process.env?.NEXT_PUBLIC_TELEGRAM_BOT_USERNAME) ||
    "KianAgentBot"
  ).replace(/^@/, "");
  const text =
    shareText ??
    shareText ?? "I built this tool on the MAIRON market — open it in one tap and earn a referral reward on every purchase.";
  const shareUrl = buildViralShareLink(botUsername, productId, referrerTgId, text);
  return { startParam: base.startParam, telegramUrl: base.telegramUrl, webUrl: base.webUrl, shareUrl };
}
