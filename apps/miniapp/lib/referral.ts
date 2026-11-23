/**
 * ردیابی لینک‌های وایرال تلگرام و ذخیرهٔ شناسهٔ معرف (فاز ۵).
 *
 * فرمت استاندارد لینک فروش و رفرال:
 * - `p_<productId>_r_<refTgId>` (هم‌تراز با `shareStartParam` در کرنل Rust)
 * - `p_<productId>` (لینک مستقیم محصول بدون معرف)
 * - `?startapp=...` یا `?tgWebAppStartParam=...` یا `?ref=<refTgId>`
 */

export const ACTIVE_REFERRER_KEY = "kian_active_referrer";
const REDIRECTED_PRODUCT_KEY = "kian_viral_redirected_product";

export type ParsedViralParam = {
  productId: string | null;
  referrer: string | null;
};

/** استخراج شناسهٔ محصول و شناسهٔ عددی معرف از پارامتر `start_param` / `startapp`. */
export function parseViralStartParam(
  raw: string | null | undefined,
): ParsedViralParam {
  if (!raw) return { productId: null, referrer: null };
  let decoded = raw.trim();
  try {
    decoded = decodeURIComponent(decoded).trim();
  } catch {
    /* رشتهٔ خام حفظ می‌شود */
  }
  if (!decoded) return { productId: null, referrer: null };

  // الگوی اصلی: p_<productId>_r_<refTgId>
  const fullMatch = /^p_(.+)_r_([0-9]+)$/.exec(decoded);
  if (fullMatch) {
    return { productId: fullMatch[1], referrer: fullMatch[2] };
  }

  // الگوی مستقیم محصول: p_<productId>
  const prodOnly = /^p_(.+)$/.exec(decoded);
  if (prodOnly) {
    return { productId: prodOnly[1], referrer: null };
  }

  // الگوی صرفاً معرف: r_<refTgId> یا ref_<refTgId>
  const refOnly = /^(?:r|ref)_([0-9]+)$/.exec(decoded);
  if (refOnly) {
    return { productId: null, referrer: refOnly[1] };
  }

  return { productId: null, referrer: null };
}

/** نرمال‌سازی شناسهٔ عددی معرف (فقط ارقام یا `citizen:<digits>`). */
export function normalizeReferrerId(raw: string | null | undefined): string | null {
  if (!raw) return null;
  const trimmed = raw.trim();
  if (/^[0-9]+$/.test(trimmed)) return trimmed;
  const citizenMatch = /^citizen:([0-9]+)$/.exec(trimmed);
  if (citizenMatch) return citizenMatch[1];
  return null;
}

/** ذخیرهٔ معرف فعال در sessionStorage و localStorage برای ارسال خودکار هنگام خرید. */
export function saveActiveReferrer(ref: string | null): void {
  if (typeof window === "undefined") return;
  const clean = normalizeReferrerId(ref);
  if (!clean) return;
  try {
    window.sessionStorage.setItem(ACTIVE_REFERRER_KEY, clean);
  } catch {
    /* حالت محدود مرورگر */
  }
  try {
    window.localStorage.setItem(ACTIVE_REFERRER_KEY, clean);
  } catch {
    /* حالت incognito */
  }
}

/** خواندن معرف فعال (اولویت با sessionStorage، سپس localStorage). */
export function getActiveReferrer(): string | null {
  if (typeof window === "undefined") return null;
  try {
    const s = normalizeReferrerId(window.sessionStorage.getItem(ACTIVE_REFERRER_KEY));
    if (s) return s;
  } catch {
    /* نادیده */
  }
  try {
    return normalizeReferrerId(window.localStorage.getItem(ACTIVE_REFERRER_KEY));
  } catch {
    return null;
  }
}

/**
 * خواندن پارامتر ورود از تلگرام (`initDataUnsafe.start_param`) یا URL
 * (`?startapp=`، `?tgWebAppStartParam=`، `?ref=`، `?product=`) و ذخیرهٔ معرف.
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

/** جلوگیری از ریدایرکت تکراری برای یک محصول در همان نشست مرورگر. */
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

/** ساخت لینک آمادهٔ وایرال و رفرال تلگرام برای هر محصول. */
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

/** نمایش فارسی درصد پورسانت از نقاط پایه (مثلاً `1000` → `۱۰٪`). */
export function formatAffiliatePercent(bps: number): string {
  const pct = bps / 100;
  return `${pct.toLocaleString("fa-IR", { maximumFractionDigits: 1 })}٪`;
}
