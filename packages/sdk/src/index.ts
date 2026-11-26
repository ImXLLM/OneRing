/**
 * MAIRON SDK — framework-free kernel client, shared between the Web OS, the
 * admin console, the mini-app, and any TS client.
 *
 * Contracts mirror crates/contracts exactly:
 * - every session-bound request sends `Authorization: Bearer`.
 * - the error envelope is always `{ error: { code, message, meta? } }` → `ApiError`.
 * - identity is never constructed here: only the raw Telegram initData string goes to the kernel
 *   where the official HMAC is verified (fail-closed).
 *
 * The sovereign `MaironSdkClient` (domain namespaces + `syncWorldState`) lives
 * in `./mairon-client`; its legacy aliases `KianSdkClient` / `createKianSdk`
 * are re-exported so existing imports never break.
 */

import { typedRequest } from "./core";

export { ApiError, resolveDefaultApiBase, qs } from "./core";
export {
  MaironSdkClient,
  createMaironSdk,
  createKianSdk,
  KianSdkClient,
} from "./mairon-client";
export type { MaironSdkOptions } from "./mairon-client";

// ── Wire types ────────────────────────────────────────────────────────────

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
  amount: number; // Din as an integer (signed — negative means owed)
  /** The kernel always sends `memo` on the wire (explicit null) — mirror of WalletTxView in crates/contracts */
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

/** Wallet move response — mirror of WalletMoveResponse in crates/contracts. */
export type WalletMoveResponse = {
  txId: string;
  accountId: string;
  balance: number;
  replayed: boolean;
};

/** Demo/dev login request — only when the kernel is booted with KIAN_DEV_MODE=1. */
export type DevLoginInput = {
  tgUserId: string;
  username?: string;
  firstName?: string;
  startParam?: string | null;
};

/** Personas ready for tonight’s test — aligned with DEMO_PERSONAS in @kian/contracts. */
export const DEMO_PERSONAS = {
  creator: { tgUserId: "1001", username: "creator_demo", firstName: "Creator (demo persona)" },
  buyer: { tgUserId: "2002", username: "buyer_demo", firstName: "Buyer (demo persona)" },
  affiliate: { tgUserId: "3003", username: "affiliate_demo", firstName: "Affiliate (demo persona)" },
} as const;

export type DemoPersona = keyof typeof DEMO_PERSONAS;

/** Telegram ids of the demo personas — for detecting a "demo user" in the UI. */
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
  /** All nullable Rust fields arrive explicitly on the wire (no skip) — undefined never occurs */
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

/** National economy summary — GET /economy (public, read-only, Article 6).
 *
 * Mirror of `EconomySummary` in crates/contracts/src/economy.rs with `#[serde(rename_all = "camelCase")]`
 * - field names exactly matching the Zod schema in packages/contracts.
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

// ── Errors ──────────────────────────────────────────────────────────────────────

// `ApiError` is defined once in `./core` and re-exported above.

// ── Request engine ────────────────────────────────────────────────────────────

const DEFAULT_BASE = resolveDefaultBase();

/** Legacy-compatible base resolution (kept for the flat client below). */
function resolveDefaultBase(): string {
  if (typeof process !== "undefined" && process.env?.NEXT_PUBLIC_KIAN_API) {
    return process.env.NEXT_PUBLIC_KIAN_API;
  }
  if (typeof window !== "undefined" && typeof window.location !== "undefined") {
    return `${window.location.origin}/api/v1`;
  }
  return "http://127.0.0.1:8088/api/v1";
}

async function request<T>(
  base: string,
  path: string,
  init: RequestInit & { token?: string | null } = {},
): Promise<T> {
  return typedRequest<T>(base, path, init);
}

// ── Client ───────────────────────────────────────────────────────────────────

export type KianClientOptions = {
  /** Kernel API base — defaults to `NEXT_PUBLIC_KIAN_API` or local 8088. */
  baseUrl?: string;
};

export function createKianClient(options: KianClientOptions = {}) {
  const base = (options.baseUrl ?? DEFAULT_BASE).replace(/\/$/, "");

  return {
    baseUrl: base,

    // M3 — session and identity
    authTelegram: (initData: string) =>
      request<AuthResponse>(base, "/auth/telegram", {
        method: "POST",
        body: JSON.stringify({ initData }),
      }),

    // dev mode — one-click persona login for full testing in the browser (403 in production)
    devLogin: (input: DevLoginInput) =>
      request<AuthResponse>(base, "/auth/dev-login", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    // dev mode — demo credit faucet (5,000 KIAN default; 50,000 KIAN cap)
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

    // public — national economy
    economy: () => request<EconomySummary>(base, "/economy", {}),

    // M3+ — market, preview, referrals, HTML delivery and buyer disputes
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

    // Phase 4 — the AI HTML product builder studio
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

    // M3 — wallet and TON deposits
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

    // Phase 6 — crypto withdrawal requests (TON / USDT)
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

/** Default client — the same env path the web app uses. */
export const api = createKianClient();

/** Default MAIRON SDK client — the canonical entry point for new code. */
export const mairon = createMaironSdk();

// ── Telegram — extracting initData (two official paths) ────────────────────────────────

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

/** Prepare the Telegram WebApp — ready + expand + royal theme (harmless in a browser). */
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
    /* Older Telegram versions — irrelevant */
  }
}

/** Raw initData string from either official path — `null` means outside Telegram. */
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

/** For friendly display only — real identity comes solely from kernel HMAC verification. */
export function peekTelegramName(): string | null {
  if (typeof window === "undefined") return null;
  return window.Telegram?.WebApp?.initDataUnsafe?.user?.username ?? null;
}

// ── Session — bearer storage (localStorage) ──────────────────────────────────────

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
    /* Incognito mode — session kept in memory only */
  }
}

export function clearToken(): void {
  try {
    window.localStorage.removeItem(SESSION_KEY);
  } catch {
    /* Irrelevant */
  }
}
