/**
 * KIAN M3 — the kernel API client with a bearer session.
 *
 * The token is stored in localStorage under the `kian.session` key; every
 * sessioned request sends `Authorization: Bearer`. The error envelope is always
 * `{ error: { code, message, meta? } }` — exactly like the kernel contract.
 */

const API_BASE =
  process.env.NEXT_PUBLIC_KIAN_API ?? "http://127.0.0.1:8088/api/v1";

export const SESSION_KEY = "kian.session";

// ── Wire types (mirror of crates/contracts) ──────────────────────────────

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
  amount: number; // Din as an integer
  memo?: string;
  at: string;
};

export type MeResponse = {
  user: UserView;
  balance: number;
  escrowLocked: number;
  recentTxs: WalletTxView[];
};

export type ProductView = {
  id: string;
  sellerId: string;
  title: string;
  description: string;
  priceDin: number;
  category: string | null;
  active: boolean;
  createdAt: string;
};

export type EscrowView = {
  id: string;
  buyerId: string;
  sellerId: string;
  amount: number;
  productRef: string | null;
  status: string;
  createdAt: string;
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

export type TonDepositView = {
  id: string;
  txHash: string;
  walletAddress: string;
  amountTon: string;
  status: "PENDING" | "CONFIRMED" | "REJECTED";
  createdAt: string;
  confirmedAt: string | null;
};

// ── Errors ────────────────────────────────────────────────────────────────────

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

async function request<T>(
  path: string,
  init: RequestInit & { token?: string | null } = {},
): Promise<T> {
  const { token, ...rest } = init;
  const headers = new Headers(rest.headers);
  if (rest.body) headers.set("content-type", "application/json");
  if (token) headers.set("authorization", `Bearer ${token}`);
  const res = await fetch(`${API_BASE}${path}`, { ...rest, headers });
  if (res.status === 204) return undefined as T;
  const body: unknown = await res.json().catch(() => ({}));
  if (!res.ok) {
    const envelope = (body as { error?: { code?: string; message?: string } })
      .error;
    throw new ApiError(
      res.status,
      envelope?.code ?? "UNKNOWN",
      envelope?.message ?? "\u067e\u0627\u0633\u062e \u0646\u0627\u0634\u0646\u0627\u062e\u062a\u0647 \u0627\u0632 \u06a9\u0631\u0646\u0644",
    );
  }
  return body as T;
}

// ── Token ───────────────────────────────────────────────────────────────────

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
    /* Incognito mode — session in memory only */
  }
}

export function clearToken(): void {
  try {
    window.localStorage.removeItem(SESSION_KEY);
  } catch {
    /* Irrelevant */
  }
}

// ── M3 endpoints ────────────────────────────────────────────────────────

export const api = {
  authTelegram: (initData: string) =>
    request<AuthResponse>("/auth/telegram", {
      method: "POST",
      body: JSON.stringify({ initData }),
    }),

  me: (token: string) => request<MeResponse>("/me", { token }),

  logout: (token: string) =>
    request<void>("/auth/logout", { method: "POST", token }),

  setTonAddress: (token: string, address: string) =>
    request<UserView>("/me/ton-address", {
      method: "POST",
      token,
      body: JSON.stringify({ address }),
    }),

  listProducts: () =>
    request<{ items: ProductView[] }>("/market/products", {}),

  createProduct: (
    token: string,
    input: {
      title: string;
      description: string;
      priceDin: number;
      category?: string;
    },
  ) =>
    request<ProductView>("/market/products", {
      method: "POST",
      token,
      body: JSON.stringify(input),
    }),

  purchase: (token: string, productId: string) =>
    request<PurchaseResponse>("/market/purchase", {
      method: "POST",
      token,
      body: JSON.stringify({ productId }),
    }),

  myMarket: (token: string) => request<MyMarketResponse>("/market/my", { token }),

  deliver: (token: string, escrowId: string) =>
    request<EscrowView>(`/market/escrows/${escrowId}/deliver`, {
      method: "POST",
      token,
      body: "{}",
    }),

  release: (token: string, escrowId: string) =>
    request<EscrowView>(`/market/escrows/${escrowId}/release`, {
      method: "POST",
      token,
      body: "{}",
    }),

  createDeposit: (
    token: string,
    input: { txHash: string; walletAddress: string; amountTon: string },
  ) =>
    request<TonDepositView>("/wallet/ton-deposits", {
      method: "POST",
      token,
      body: JSON.stringify(input),
    }),

  myDeposits: (token: string) =>
    request<{ items: TonDepositView[] }>("/wallet/ton-deposits", { token }),
};
