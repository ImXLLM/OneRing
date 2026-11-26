/**
 * Zero-dependency Telegram Mini App helpers for apps/miniapp/lib/kian-tma-bridge.ts.
 * Safe to import during Next.js SSR: window/document are only read inside calls.
 */

export type HapticImpactStyle = "light" | "medium" | "heavy" | "rigid" | "soft";
export type HapticNotificationType = "error" | "success" | "warning";
export type Insets = { top: number; right: number; bottom: number; left: number };
type SafeAreaSide = keyof Insets;

export interface TelegramWebAppLike {
  disableVerticalSwipes?: () => void;
  enableVerticalSwipes?: () => void;
  HapticFeedback?: {
    impactOccurred?: (style: HapticImpactStyle) => void;
    notificationOccurred?: (type: HapticNotificationType) => void;
    selectionChanged?: () => void;
  };
  safeAreaInset?: Partial<Insets>;
  contentSafeAreaInset?: Partial<Insets>;
  onEvent?: (eventType: string, handler: () => void) => void;
  offEvent?: (eventType: string, handler: () => void) => void;
  ready?: () => void;
  expand?: () => void;
  platform?: string;
  version?: string;
}

export interface TelegramWindowLike extends Window {
  Telegram?: { WebApp?: TelegramWebAppLike };
}

// Note: the global `window.Telegram` is already declared in `@kian/sdk` with the `TelegramWebApp`
// type; we do not declare it globally again here to avoid TS2717 — internal access
// goes through a safe cast to `TelegramWindowLike`, and consumers must use this very bridge
// for haptic/swipe/safe-area, not raw `window`.

function getWebApp(): TelegramWebAppLike | null {
  if (typeof window === "undefined") return null;
  return (window as TelegramWindowLike).Telegram?.WebApp ?? null;
}

/**
 * KIAN-facing boolean wrapper. `false` disables Telegram's vertical swipe-to-close
 * gesture (use only while it conflicts with an in-app gesture); `true` restores it.
 */
export function setSwipeBehavior(
  allowVerticalSwipes: boolean,
  app: TelegramWebAppLike | null = getWebApp(),
): void {
  if (!app) return;
  try {
    if (allowVerticalSwipes) app.enableVerticalSwipes?.();
    else app.disableVerticalSwipes?.();
  } catch {
    // Unsupported/old Telegram clients should not prevent the Mini App from working.
  }
}

function safeHaptic(action: (() => void) | undefined): void {
  try {
    action?.();
  } catch {
    // Haptics are optional platform feedback, never a functional dependency.
  }
}

export const haptics = {
  impact(style: HapticImpactStyle = "light"): void {
    const app = getWebApp();
    safeHaptic(() => app?.HapticFeedback?.impactOccurred?.(style));
  },
  selection(): void {
    const app = getWebApp();
    safeHaptic(() => app?.HapticFeedback?.selectionChanged?.());
  },
  success(): void {
    const app = getWebApp();
    safeHaptic(() => app?.HapticFeedback?.notificationOccurred?.("success"));
  },
  warning(): void {
    const app = getWebApp();
    safeHaptic(() => app?.HapticFeedback?.notificationOccurred?.("warning"));
  },
  error(): void {
    const app = getWebApp();
    safeHaptic(() => app?.HapticFeedback?.notificationOccurred?.("error"));
  },
} as const;

const SIDES: readonly SafeAreaSide[] = ["top", "right", "bottom", "left"];

function cssInset(
  insets: Partial<Insets> | undefined,
  side: SafeAreaSide,
  useDeviceFallback: boolean,
): string {
  const value = insets?.[side];
  if (typeof value === "number" && Number.isFinite(value) && value >= 0) {
    return `${value}px`;
  }
  return useDeviceFallback ? `env(safe-area-inset-${side}, 0px)` : "0px";
}

/**
 * Sync Telegram safe-area and content-safe-area insets into KIAN CSS variables.
 * Returns an unsubscribe callback. Includes env() fallbacks for browser previews.
 */
export function syncSafeArea(
  root?: HTMLElement,
  app: TelegramWebAppLike | null = getWebApp(),
): () => void {
  if (typeof document === "undefined") return () => undefined;
  const element = root ?? document.documentElement;
  const update = (): void => {
    for (const side of SIDES) {
      element.style.setProperty(
        `--kian-safe-area-${side}`,
        cssInset(app?.safeAreaInset, side, true),
      );
      element.style.setProperty(
        `--kian-content-safe-area-${side}`,
        cssInset(app?.contentSafeAreaInset ?? app?.safeAreaInset, side, true),
      );
    }
  };

  const onSafeAreaChanged = (): void => update();
  const onContentSafeAreaChanged = (): void => update();
  const onViewportChanged = (): void => update();
  app?.onEvent?.("safeAreaChanged", onSafeAreaChanged);
  app?.onEvent?.("contentSafeAreaChanged", onContentSafeAreaChanged);
  app?.onEvent?.("viewportChanged", onViewportChanged);
  if (typeof window !== "undefined") window.addEventListener("resize", update, { passive: true });
  update();

  return (): void => {
    app?.offEvent?.("safeAreaChanged", onSafeAreaChanged);
    app?.offEvent?.("contentSafeAreaChanged", onContentSafeAreaChanged);
    app?.offEvent?.("viewportChanged", onViewportChanged);
    if (typeof window !== "undefined") window.removeEventListener("resize", update);
  };
}

/** Call once from a client component after Telegram's WebApp object is available. */
export function initKianTmaBridge(root?: HTMLElement): () => void {
  const app = getWebApp();
  app?.ready?.();
  setSwipeBehavior(false, app);
  return syncSafeArea(root, app);
}

function positiveDecimal(value: string | number, label: string): string {
  if (typeof value === "number" && (!Number.isSafeInteger(value) || value <= 0)) {
    throw new TypeError(`${label} must be a safe positive integer; pass large IDs as strings`);
  }
  const text = String(value);
  if (!/^[1-9][0-9]*$/.test(text)) {
    throw new TypeError(`${label} must be a positive decimal integer`);
  }
  return text;
}

/** Build a t.me deep link that opens the bot's main Mini App with startapp data. */
export function buildStartAppLink(botUsername: string, startParam: string): string {
  const username = botUsername.trim().replace(/^@+/, "");
  if (!/^[A-Za-z][A-Za-z0-9_]{4,31}$/.test(username)) {
    throw new TypeError("botUsername must be a Telegram bot username (without @)");
  }
  if (!startParam || startParam.length > 512) {
    throw new TypeError("startParam must contain 1–512 characters");
  }
  return `https://t.me/${username}?startapp=${encodeURIComponent(startParam)}`;
}

/**
 * Viral product link: the opener receives product context and the inviter's
 * Telegram ID in the startapp payload `p_<productId>_r_<refTgId>`.
 * The backend must still validate/refuse referral claims; this URL is not auth.
 */
export function buildViralStartAppLink(
  botUsername: string,
  productId: string | number,
  refTgId: string | number,
): string {
  const product = positiveDecimal(productId, "productId");
  const referrer = positiveDecimal(refTgId, "refTgId");
  return buildStartAppLink(botUsername, `p_${product}_r_${referrer}`);
}

/** Build a Telegram share-sheet URL around a referral Mini App deep link. */
export function buildViralShareLink(
  botUsername: string,
  productId: string | number,
  refTgId: string | number,
  shareText: string,
): string {
  const appLink = buildViralStartAppLink(botUsername, productId, refTgId);
  const params = new URLSearchParams({ url: appLink, text: shareText });
  return `https://t.me/share/url?${params.toString()}`;
}
