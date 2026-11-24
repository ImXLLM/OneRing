/**
 * KIAN SDK — کلاینت فریم‌ورک-آزادِ کرنل، مشترک بین مینی‌اپ و هر کلاینت TS.
 *
 * قراردادها دقیقاً قرینهٔ crates/contracts است:
 * - هر درخواست نشست‌دار `Authorization: Bearer` می‌فرستد.
 * - پاکت خطا همیشه `{ error: { code, message, meta? } }` است → `ApiError`.
 * - هویت هرگز اینجا ساخته نمی‌شود: فقط رشتهٔ خام initData تلگرام به کرنل
 *   می‌رود تا HMAC رسمی همان‌جا تأیید شود (fail-closed).
 */

// ── تایپ‌های وایر ────────────────────────────────────────────────────────────

export type UserView = {
  id: string;
  telegramId: string;
  username?: string;
  accountId: string;
  tonAddress: string | null;
  createdAt: string;
};

export type AuthResponse = {
  token: string;
  expiresAt: string;
  user: UserView;
};

export type WalletTxView = {
  txId: string;
  amount: number; // Din به‌صورت عدد صحیح (علامت‌دار — بدهکار منفی)
  /** کرنل همیشه `memo` را روی سیم می‌فرستد (null صریح) — قرینهٔ WalletTxView در crates/contracts */
  memo: string | null;
  at: string;
};

export type MeResponse = {
  user: UserView;
  balance: number;
  escrowLocked: number;
  recentTxs: WalletTxView[];
};

export type WithdrawalChain = "TON" | "TRX" | "SOL" | "BSC";
export type WithdrawalAsset = "TON" | "USDT";
export type WithdrawalStatus = "PENDING" | "CONFIRMED" | "REJECTED";

export type WithdrawalRequestView = {
  id: string;
  accountId: string;
  chain: WithdrawalChain;
  asset: WithdrawalAsset;
  destinationAddress: string;
  amountDin: number;
  txHash: string | null;
  status: WithdrawalStatus;
  createdAt: number;
  resolvedAt: number | null;
};

export type CreateWithdrawalInput = {
  chain: WithdrawalChain;
  asset: WithdrawalAsset;
  destinationAddress: string;
  amountDin: number;
};

/** پاسخ جابه‌جایی کیف — قرینهٔ WalletMoveResponse در crates/contracts. */
export type WalletMoveResponse = {
  txId: string;
  accountId: string;
  balance: number;
  replayed: boolean;
};

/** درخواست ورود دمو/توسعه — فقط وقتی کرنل با KIAN_DEV_MODE=1 بالا آمده باشد. */
export type DevLoginInput = {
  tgUserId: string;
  username?: string;
  firstName?: string;
  startParam?: string | null;
};

/** پرسوناهای آمادهٔ تستِ امشب — هم‌تراز با DEMO_PERSONAS در @kian/contracts. */
export const DEMO_PERSONAS = {
  creator: { tgUserId: "1001", username: "creator_demo", firstName: "سازنده (Creator)" },
  buyer: { tgUserId: "2002", username: "buyer_demo", firstName: "خریدار (Buyer)" },
  affiliate: { tgUserId: "3003", username: "affiliate_demo", firstName: "بازاریاب (Affiliate)" },
} as const;

export type DemoPersona = keyof typeof DEMO_PERSONAS;

/** شناسه‌های تلگرامی پرسوناهای دمو — برای تشخیص «کاربرِ دمو» در UI. */
export const DEMO_PERSONA_TG_IDS: readonly string[] = Object.values(DEMO_PERSONAS).map(
  (p) => p.tgUserId,
);

export type ProductView = {
  id: string;
  sellerId: string;
  title: string;
  description: string;
  priceDin: number;
  category: string | null;
  previewHtml: string;
  affiliateBps: number;
  artifactKind: string;
  active: boolean;
  createdAt: string;
};

export type CreateProductInput = {
  title: string;
  description: string;
  priceDin: number;
  category?: string;
  previewHtml?: string;
  affiliateBps?: number;
  artifactKind?: string;
  deliveryPayload?: string;
};

export type EscrowView = {
  id: string;
  buyerId: string;
  sellerId: string;
  amount: number;
  productRef: string | null;
  status: string;
  createdAt: string;
  /** همهٔ فیلدهای nullable در Rust صریح روی سیم می‌آیند (بدون skip) — undefined هرگز رخ نمی‌دهد */
  fundedTxId: string | null;
  settleTxId: string | null;
  disputeReason: string | null;
  deliveryRef: string | null;
  expiresAt: string | null;
};

export type EscrowActionResponse = {
  escrow: EscrowView;
  txId: string | null;
  replayed: boolean;
};

export type PurchaseOptions = {
  referrer?: string;
  idempotencyKey?: string;
};

export type PurchaseResponse = {
  escrow: EscrowView;
  txId: string | null;
  replayed: boolean;
};

export type MyMarketResponse = {
  products: ProductView[];
  orders: EscrowView[];
};

export type EscrowDeliveryView = {
  escrowId: string;
  productId: string;
  title: string;
  artifactKind: string;
  state: string;
  deliveryPayload: string;
};

export type BuilderMessageView = {
  id: number;
  role: string;
  content: string;
  createdAt: number;
};

export type BuilderSessionSummary = {
  id: string;
  stage: string;
  title: string;
  description: string;
  suggestedPrice: number;
  affiliateBps: number;
  hasHtml: boolean;
  publishedProductId: string | null;
  createdAt: number;
  updatedAt: number;
};

export type BuilderSessionView = {
  id: string;
  stage: string;
  title: string;
  description: string;
  suggestedPrice: number;
  affiliateBps: number;
  generatedHtml: string;
  previewHtml: string;
  publishedProductId: string | null;
  messages: BuilderMessageView[];
  createdAt: number;
  updatedAt: number;
};

export type BuilderSessionListResponse = {
  items: BuilderSessionSummary[];
};

export type BuilderChatResponse = {
  reply: string;
  session: BuilderSessionView;
  usedFallback: boolean;
  provider: string | null;
  totalTokens: number;
};

export type PublishBuilderOverrides = {
  title?: string;
  description?: string;
  priceDin?: number;
  affiliateBps?: number;
  category?: string;
};

export type PublishBuilderResponse = {
  product: ProductView;
  sessionId: string;
  shareStartParam: string;
  alreadyPublished: boolean;
};

export type TonDepositView = {
  id: string;
  txHash: string;
  walletAddress: string;
  amountTon: string;
  status: "PENDING" | "CONFIRMED" | "REJECTED";
  createdAt: string;
  confirmedAt: string | null;
};

/** خلاصهٔ اقتصاد ملی — GET /economy (عمومی، فقط‌خواندنی، مادهٔ ۶).
 *
 * قرینهٔ `EconomySummary` در crates/contracts/src/economy.rs با `#[serde(rename_all = "camelCase")]`
 * — دقیقاً هم‌نام با اسکیمای Zod در packages/contracts.
 */
export type EconomySummary = {
  currency: "DNX";
  symbol: "Đ";
  circulatingSupply: number;
  treasuryBalance: number;
  escrowLocked: number;
  mintedAllTime: number;
  burnedAllTime: number;
  mintedThisYear: number;
  yearlyMintCap: number;
  genesisSupply: number;
  trialBalanceSum: number;
  balanced: boolean;
  accountCount: number;
  openEscrows: number;
};

// ── خطا ──────────────────────────────────────────────────────────────────────

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
  }
}

// ── موتور درخواست ────────────────────────────────────────────────────────────

const DEFAULT_BASE =
  typeof process !== "undefined" && process.env?.NEXT_PUBLIC_KIAN_API
    ? process.env.NEXT_PUBLIC_KIAN_API
    : "http://127.0.0.1:8088/api/v1";

async function request<T>(
  base: string,
  path: string,
  init: RequestInit & { token?: string | null } = {},
): Promise<T> {
  const { token, ...rest } = init;
  const headers = new Headers(rest.headers);
  if (rest.body) headers.set("content-type", "application/json");
  if (token) headers.set("authorization", `Bearer ${token}`);
  const res = await fetch(`${base}${path}`, { ...rest, headers });
  if (res.status === 204) return undefined as T;
  const body: unknown = await res.json().catch(() => ({}));
  if (!res.ok) {
    const envelope = (body as { error?: { code?: string; message?: string } })
      .error;
    throw new ApiError(
      res.status,
      envelope?.code ?? "UNKNOWN",
      envelope?.message ?? "پاسخ ناشناخته از کرنل",
    );
  }
  return body as T;
}

// ── کلاینت ───────────────────────────────────────────────────────────────────

export type KianClientOptions = {
  /** پایهٔ API کرنل — پیش‌فرض `NEXT_PUBLIC_KIAN_API` یا لوکال 8088. */
  baseUrl?: string;
};

export function createKianClient(options: KianClientOptions = {}) {
  const base = (options.baseUrl ?? DEFAULT_BASE).replace(/\/$/, "");

  return {
    baseUrl: base,

    // M3 — نشست و هویت
    authTelegram: (initData: string) =>
      request<AuthResponse>(base, "/auth/telegram", {
        method: "POST",
        body: JSON.stringify({ initData }),
      }),

    // حالت dev — ورود یک‌کلیکی پرسونا برای تستِ کامل در مرورگر (۴۰۳ در تولید)
    devLogin: (input: DevLoginInput) =>
      request<AuthResponse>(base, "/auth/dev-login", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    // حالت dev — شیر اعتبار آزمایشی (۵٬۰۰۰ KIAN پیش‌فرض؛ سقف ۵۰٬۰۰۰ KIAN)
    claimDemoFaucet: (token: string, amountMicro?: number) =>
      request<WalletMoveResponse>(base, "/wallet/faucet", {
        method: "POST",
        token,
        body: JSON.stringify(
          amountMicro && amountMicro > 0 ? { amountMicro } : {},
        ),
      }),

    me: (token: string) => request<MeResponse>(base, "/me", { token }),

    logout: (token: string) =>
      request<void>(base, "/auth/logout", { method: "POST", token }),

    setTonAddress: (token: string, address: string) =>
      request<UserView>(base, "/me/ton-address", {
        method: "POST",
        token,
        body: JSON.stringify({ address }),
      }),

    // عمومی — اقتصاد ملی
    economy: () => request<EconomySummary>(base, "/economy", {}),

    // M3+ — بازار، پیش‌نمایش، رفرال، تحویل HTML و اختلاف خریدار
    listProducts: () =>
      request<{ items: ProductView[] }>(base, "/market/products", {}),

    getProduct: (id: string) =>
      request<ProductView>(
        base,
        `/market/products/${encodeURIComponent(id)}`,
        {},
      ),

    createProduct: (token: string, input: CreateProductInput) =>
      request<ProductView>(base, "/market/products", {
        method: "POST",
        token,
        body: JSON.stringify(input),
      }),

    purchase: (token: string, productId: string, options?: PurchaseOptions) =>
      request<PurchaseResponse>(base, "/market/purchase", {
        method: "POST",
        token,
        body: JSON.stringify({
          productId,
          ...(options?.referrer ? { referrer: options.referrer } : {}),
          ...(options?.idempotencyKey
            ? { idempotencyKey: options.idempotencyKey }
            : {}),
        }),
      }),

    purchaseProduct: (
      token: string,
      productId: string,
      options?: PurchaseOptions,
    ) =>
      request<PurchaseResponse>(base, "/market/purchase", {
        method: "POST",
        token,
        body: JSON.stringify({
          productId,
          ...(options?.referrer ? { referrer: options.referrer } : {}),
          ...(options?.idempotencyKey
            ? { idempotencyKey: options.idempotencyKey }
            : {}),
        }),
      }),

    myMarket: (token: string) =>
      request<MyMarketResponse>(base, "/market/my", { token }),

    getEscrowDelivery: (token: string, escrowId: string) =>
      request<EscrowDeliveryView>(
        base,
        `/market/escrows/${encodeURIComponent(escrowId)}/delivery`,
        { token },
      ),

    deliver: (token: string, escrowId: string) =>
      request<EscrowActionResponse>(
        base,
        `/market/escrows/${encodeURIComponent(escrowId)}/deliver`,
        {
          method: "POST",
          token,
          body: "{}",
        },
      ),

    release: (token: string, escrowId: string) =>
      request<EscrowActionResponse>(
        base,
        `/market/escrows/${encodeURIComponent(escrowId)}/release`,
        {
          method: "POST",
          token,
          body: "{}",
        },
      ),

    disputeEscrow: (
      token: string,
      escrowId: string,
      reason: string,
      idempotencyKey?: string,
    ) =>
      request<EscrowActionResponse>(
        base,
        `/market/escrows/${encodeURIComponent(escrowId)}/dispute`,
        {
          method: "POST",
          token,
          body: JSON.stringify({
            reason,
            ...(idempotencyKey ? { idempotencyKey } : {}),
          }),
        },
      ),

    // فاز ۴ — استودیوی ساخت محصول HTML با هوش مصنوعی
    createBuilderSession: (token: string, initialIdea?: string) =>
      request<BuilderSessionView>(base, "/builder/sessions", {
        method: "POST",
        token,
        body: JSON.stringify(initialIdea ? { initialIdea } : {}),
      }),

    listBuilderSessions: (token: string) =>
      request<BuilderSessionListResponse>(base, "/builder/sessions", { token }),

    getBuilderSession: (token: string, id: string) =>
      request<BuilderSessionView>(
        base,
        `/builder/sessions/${encodeURIComponent(id)}`,
        { token },
      ),

    chatBuilderSession: (
      token: string,
      id: string,
      message: string,
      forceGenerate?: boolean,
    ) =>
      request<BuilderChatResponse>(
        base,
        `/builder/sessions/${encodeURIComponent(id)}/chat`,
        {
          method: "POST",
          token,
          body: JSON.stringify({
            message,
            ...(forceGenerate !== undefined ? { forceGenerate } : {}),
          }),
        },
      ),

    publishBuilderSession: (
      token: string,
      id: string,
      overrides?: PublishBuilderOverrides,
    ) =>
      request<PublishBuilderResponse>(
        base,
        `/builder/sessions/${encodeURIComponent(id)}/publish`,
        {
          method: "POST",
          token,
          body: JSON.stringify(overrides ?? {}),
        },
      ),

    // M3 — ولت و واریز TON
    createDeposit: (
      token: string,
      input: { txHash: string; walletAddress: string; amountTon: string },
    ) =>
      request<TonDepositView>(base, "/wallet/ton-deposits", {
        method: "POST",
        token,
        body: JSON.stringify(input),
      }),

    myDeposits: (token: string) =>
      request<{ items: TonDepositView[] }>(base, "/wallet/ton-deposits", {
        token,
      }),

    // فاز ۶ — درخواست برداشت کریپتویی (TON / USDT)
    createWithdrawal: (token: string, input: CreateWithdrawalInput) =>
      request<WithdrawalRequestView>(base, "/wallet/withdrawals", {
        method: "POST",
        token,
        body: JSON.stringify(input),
      }),

    myWithdrawals: (token: string) =>
      request<{ items: WithdrawalRequestView[] }>(base, "/wallet/withdrawals", {
        token,
      }),
  };
}

/** کلاینت پیش‌فرض — همان مسیر env که وب استفاده می‌کند. */
export const api = createKianClient();

// ── تلگرام — استخراج initData (دو مسیر رسمی) ────────────────────────────────

type TelegramWebApp = {
  initData?: string;
  initDataUnsafe?: {
    user?: { id: number; username?: string };
    start_param?: string;
  };
  ready?: () => void;
  expand?: () => void;
  setHeaderColor?: (color: string) => void;
  setBackgroundColor?: (color: string) => void;
};

declare global {
  interface Window {
    Telegram?: { WebApp?: TelegramWebApp };
  }
}

/** آماده‌سازی WebApp داخل تلگرام — ready + expand + تم سلطنتی (بی‌ضرر در مرورگر). */
export function setupTelegramWebApp(): void {
  if (typeof window === "undefined") return;
  const app = window.Telegram?.WebApp;
  if (!app) return;
  try {
    app.ready?.();
    app.expand?.();
    app.setHeaderColor?.("#0b0a12");
    app.setBackgroundColor?.("#0b0a12");
  } catch {
    /* نسخه‌های قدیمی تلگرام — بی‌اهمیت */
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
  return window.Telegram?.WebApp?.initDataUnsafe?.user?.username ?? null;
}

// ── نشست — ذخیرهٔ bearer (localStorage) ──────────────────────────────────────

export const SESSION_KEY = "kian.session";

export function loadToken(): string | null {
  try {
    return window.localStorage.getItem(SESSION_KEY);
  } catch {
    return null;
  }
}

export function saveToken(token: string): void {
  try {
    window.localStorage.setItem(SESSION_KEY, token);
  } catch {
    /* حالت incognito — نشست فقط در حافظه */
  }
}

export function clearToken(): void {
  try {
    window.localStorage.removeItem(SESSION_KEY);
  } catch {
    /* بی‌اهمیت */
  }
}
