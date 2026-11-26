import { z } from "zod";

/**
 * MAIRON wire contracts — a TypeScript mirror of the kernel's crates/contracts
 * computations, plus the MAIRON domain modules under `./mairon/`.
 *
 * Source of truth: Rust (the crates/contracts crate with utoipa + golden tests)
 * -> here only a Zod mirror for web/mini-app/admin clients. Wire rules:
 * camelCase, explicit null for nullable fields, integers up to 2^53-1 (Din) —
 * same as legacy so dual-run works without conversion.
 */

// ── MAIRON domain modules (ledger, trinity, arda, super-studio, supremacy,
//    quant, treasury, neural-hud, war-room, world state) ──────────────────────
export * from "./mairon/ledger";
export * from "./mairon/trinity";
export * from "./mairon/arda";
export * from "./mairon/superstudio";
export * from "./mairon/quant";
export * from "./mairon/treasury";
export * from "./mairon/neuralHud";
export * from "./mairon/warRoom";
export * from "./mairon/worldState";

// ── Legacy-compatible core schemas (marketplace, escrow, wallet, builder) ─────

/** Din — the integer money unit; cap 2^53−1 (the Number.MAX_SAFE_INTEGER boundary deliberately preserved) */
export const DinAmount = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);

/** The eight escrow state-machine states — one-to-one with `kian_domain::EscrowStatus` (SCREAMING_SNAKE_CASE) */
export const EscrowStatus = z.enum([
  "CREATED",
  "FUNDED",
  "DELIVERED",
  "RELEASED",
  "DISPUTED",
  "REFUNDED",
  "CANCELLED",
  "EXPIRED",
]);

/** Escrow wire view — mirror of `EscrowView` in crates/contracts/src/escrows.rs */
export const EscrowView = z.object({
  id: z.string(),
  buyerId: z.string(),
  sellerId: z.string(),
  amount: DinAmount,
  productRef: z.string().nullable(),
  status: EscrowStatus,
  createdAt: z.string(),
  fundedTxId: z.string().nullable(),
  settleTxId: z.string().nullable(),
  disputeReason: z.string().nullable(),
  deliveryRef: z.string().nullable(),
  expiresAt: z.string().nullable(),
});

/** Standard escrow-action response — mirror of `EscrowActionResponse` */
export const EscrowActionResponse = z.object({
  escrow: EscrowView,
  txId: z.string().nullable(),
  replayed: z.boolean(),
});

/** Account view — balance derived from `Posting` (ADR-003: balances are not stored) */
export const AccountView = z.object({
  id: z.string(),
  kind: z.string(),
  label: z.string().nullable(),
  balance: z.number().int(),
  createdAt: z.string(),
});

/** Uniform error body — always `{ error: { code, message, meta? } }` */
export const ErrorEnvelope = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
    meta: z.record(z.string(), z.union([z.string(), z.number()])).optional(),
  }),
});

/** Economy summary — `GET /api/v1/economy` (Article 6 transparency; public and read-only) */
export const EconomySummary = z.object({
  currency: z.literal("DNX"),
  symbol: z.literal("Đ"),
  circulatingSupply: DinAmount,
  treasuryBalance: DinAmount,
  escrowLocked: DinAmount,
  mintedAllTime: DinAmount,
  burnedAllTime: DinAmount,
  mintedThisYear: DinAmount,
  /** Charter annual mint cap — Article 3 */
  yearlyMintCap: DinAmount,
  genesisSupply: DinAmount,
  trialBalanceSum: z.number().int(),
  balanced: z.boolean(),
  accountCount: z.number().int(),
  openEscrows: z.number().int(),
});

/** Liveness — `GET /api/v1/health` (a holdover from the legacy phase; bot/ai honestly disabled in M2) */
export const HealthResponse = z.object({
  status: z.literal("ok"),
  service: z.literal("kian-kernel-server"),
  version: z.string(),
  llmConfigured: z.boolean(),
  llmModel: z.string().nullable(),
  db: z.string().optional(),
  bot: z.string().optional(),
  ai: z.string().optional(),
  uptime: z.number().optional(),
});

// ---------- Requests ----------

export const CreateAccountRequest = z.object({
  id: z.string().min(1).max(120),
  label: z.string().max(120).optional(),
});

export const MintRequest = z.object({
  accountId: z.string().min(1),
  amount: DinAmount,
  memo: z.string().max(200).optional(),
  idempotencyKey: z.string().max(160).optional(),
});

export const CreateEscrowRequest = z.object({
  id: z.string().max(120).optional(),
  buyerId: z.string().min(1),
  sellerId: z.string().min(1),
  amount: DinAmount,
  productRef: z.string().max(128).optional(),
  /** TTL in hours (1 to 720) */
  ttlHours: z.number().int().min(1).max(720).optional(),
});

export const EscrowActionRequest = z.object({
  idempotencyKey: z.string().max(160).optional(),
});

// ---------- Marketplace, HTML artifact, referral, and delivery (phase 3) ----------

/** Product delivery content size cap (150 kilobytes for interactive HTML) */
export const MAX_DELIVERY_PAYLOAD_BYTES = 150_000;

/** Public HTML preview size cap before purchase (50 kilobytes) */
export const MAX_PREVIEW_HTML_BYTES = 50_000;

/** Default rate and cap of the affiliate commission, in basis points */
export const DEFAULT_AFFILIATE_BPS = 1_000;
export const MAX_AFFILIATE_BPS = 5_000;

/** Public product view — mirror of `ProductView` in crates/contracts/src/market.rs */
export const ProductView = z.object({
  id: z.string(),
  sellerId: z.string(),
  title: z.string(),
  description: z.string(),
  priceDin: DinAmount,
  category: z.string().nullable(),
  previewHtml: z.string(),
  affiliateBps: z.number().int().min(0).max(MAX_AFFILIATE_BPS),
  artifactKind: z.string(),
  active: z.boolean(),
  createdAt: z.string(),
});

export const ProductListResponse = z.object({
  items: z.array(ProductView),
});

export const CreateProductRequest = z.object({
  title: z.string().min(1).max(120),
  description: z.string().min(1).max(2000),
  priceDin: DinAmount,
  category: z.string().max(40).optional(),
  previewHtml: z.string().optional(),
  affiliateBps: z.number().int().min(0).max(MAX_AFFILIATE_BPS).optional(),
  artifactKind: z.string().max(32).optional(),
  deliveryPayload: z.string().optional(),
});

export const PurchaseRequest = z.object({
  productId: z.string().min(1).max(128),
  referrer: z.string().max(64).optional(),
  idempotencyKey: z.string().max(160).optional(),
});

export const PurchaseResponse = z.object({
  escrow: EscrowView,
  txId: z.string().nullable(),
  replayed: z.boolean(),
});

export const MyMarketResponse = z.object({
  products: z.array(ProductView),
  orders: z.array(EscrowView),
});

/** Full product content delivery response — mirror of `EscrowDeliveryView` */
export const EscrowDeliveryView = z.object({
  escrowId: z.string(),
  productId: z.string(),
  title: z.string(),
  artifactKind: z.string(),
  state: EscrowStatus,
  deliveryPayload: z.string(),
});

export const DisputeEscrowRequest = z.object({
  reason: z.string().min(1).max(500),
  idempotencyKey: z.string().max(160).optional(),
});

// ---------- AI-powered HTML product-builder studio (phase 4) ----------

export const BuilderMessageView = z.object({
  id: z.number().int(),
  role: z.string(),
  content: z.string(),
  createdAt: z.number().int(),
});

export const BuilderSessionSummary = z.object({
  id: z.string(),
  stage: z.string(),
  title: z.string(),
  description: z.string(),
  suggestedPrice: z.number().int(),
  affiliateBps: z.number().int().min(0).max(MAX_AFFILIATE_BPS),
  hasHtml: z.boolean(),
  publishedProductId: z.string().nullable(),
  createdAt: z.number().int(),
  updatedAt: z.number().int(),
});

export const BuilderSessionView = z.object({
  id: z.string(),
  stage: z.string(),
  title: z.string(),
  description: z.string(),
  suggestedPrice: z.number().int(),
  affiliateBps: z.number().int().min(0).max(MAX_AFFILIATE_BPS),
  generatedHtml: z.string(),
  previewHtml: z.string(),
  publishedProductId: z.string().nullable(),
  messages: z.array(BuilderMessageView),
  createdAt: z.number().int(),
  updatedAt: z.number().int(),
});

export const BuilderSessionListResponse = z.object({
  items: z.array(BuilderSessionSummary),
});

export const CreateBuilderSessionRequest = z.object({
  initialIdea: z.string().max(120).optional(),
});

export const BuilderChatRequest = z.object({
  message: z.string().min(1).max(4000),
  forceGenerate: z.boolean().optional(),
});

export const BuilderChatResponse = z.object({
  reply: z.string(),
  session: BuilderSessionView,
  usedFallback: z.boolean(),
  provider: z.string().nullable(),
  totalTokens: z.number().int(),
});

export const PublishBuilderRequest = z.object({
  title: z.string().min(1).max(120).optional(),
  description: z.string().min(1).max(2000).optional(),
  priceDin: DinAmount.optional(),
  affiliateBps: z.number().int().min(0).max(MAX_AFFILIATE_BPS).optional(),
  category: z.string().max(40).optional(),
});

export const PublishBuilderResponse = z.object({
  product: ProductView,
  sessionId: z.string(),
  shareStartParam: z.string(),
  alreadyPublished: z.boolean(),
});

export type EscrowStatusT = z.infer<typeof EscrowStatus>;
export type EscrowViewT = z.infer<typeof EscrowView>;
export type EscrowActionResponseT = z.infer<typeof EscrowActionResponse>;
export type AccountViewT = z.infer<typeof AccountView>;
export type ErrorEnvelopeT = z.infer<typeof ErrorEnvelope>;
export type EconomySummaryT = z.infer<typeof EconomySummary>;
export type HealthResponseT = z.infer<typeof HealthResponse>;
export type CreateEscrowRequestT = z.infer<typeof CreateEscrowRequest>;
export type ProductViewT = z.infer<typeof ProductView>;
export type CreateProductRequestT = z.infer<typeof CreateProductRequest>;
export type PurchaseRequestT = z.infer<typeof PurchaseRequest>;
export type PurchaseResponseT = z.infer<typeof PurchaseResponse>;
export type MyMarketResponseT = z.infer<typeof MyMarketResponse>;
export type EscrowDeliveryViewT = z.infer<typeof EscrowDeliveryView>;
export type DisputeEscrowRequestT = z.infer<typeof DisputeEscrowRequest>;
export type BuilderMessageViewT = z.infer<typeof BuilderMessageView>;
export type BuilderSessionSummaryT = z.infer<typeof BuilderSessionSummary>;
export type BuilderSessionViewT = z.infer<typeof BuilderSessionView>;
export type BuilderSessionListResponseT = z.infer<typeof BuilderSessionListResponse>;
export type CreateBuilderSessionRequestT = z.infer<typeof CreateBuilderSessionRequest>;
export type BuilderChatRequestT = z.infer<typeof BuilderChatRequest>;
export type BuilderChatResponseT = z.infer<typeof BuilderChatResponse>;
export type PublishBuilderRequestT = z.infer<typeof PublishBuilderRequest>;
export type PublishBuilderResponseT = z.infer<typeof PublishBuilderResponse>;

/** Allowed escrow state-machine transitions — mirror of ADR-004 (UI only; truth lives in the kernel) */
export const ESCROW_TRANSITIONS: Record<EscrowStatusT, readonly EscrowStatusT[]> = {
  CREATED: ["FUNDED", "CANCELLED"],
  FUNDED: ["DELIVERED", "DISPUTED", "EXPIRED"],
  DELIVERED: ["RELEASED", "DISPUTED"],
  RELEASED: [],
  DISPUTED: ["REFUNDED", "RELEASED"],
  REFUNDED: [],
  CANCELLED: [],
  EXPIRED: [],
};

/** M2 money-kernel endpoints — client reference (all admin except health/economy) */
export const M2_ENDPOINTS = {
  health: "GET /api/v1/health",
  economy: "GET /api/v1/economy",
  createAccount: "POST /api/v1/accounts",
  listAccounts: "GET /api/v1/accounts",
  balance: "GET /api/v1/accounts/{id}/balance",
  createEscrow: "POST /api/v1/escrows",
  listEscrows: "GET /api/v1/escrows",
  getEscrow: "GET /api/v1/escrows/{id}",
  fund: "POST /api/v1/escrows/{id}/fund",
  deliver: "POST /api/v1/escrows/{id}/deliver",
  release: "POST /api/v1/escrows/{id}/release",
  dispute: "POST /api/v1/escrows/{id}/dispute",
  refund: "POST /api/v1/escrows/{id}/refund",
  cancel: "POST /api/v1/escrows/{id}/cancel",
  expire: "POST /api/v1/escrows/{id}/expire",
  mint: "POST /api/v1/wallet/mint",
  burn: "POST /api/v1/wallet/burn",
} as const;

/** Admin auth header key — fail-closed (server without a key = admin endpoints off) */
export const KIAN_KEY_HEADER = "X-KIAN-Key";

// ── Phase 6 — crypto withdrawal requests (TON / USDT on TON, TRX, SOL, BSC) ───────

export const withdrawalChainSchema = z.enum(["TON", "TRX", "SOL", "BSC"]);
export type WithdrawalChainT = z.infer<typeof withdrawalChainSchema>;

export const withdrawalAssetSchema = z.enum(["TON", "USDT"]);
export type WithdrawalAssetT = z.infer<typeof withdrawalAssetSchema>;

export const withdrawalStatusSchema = z.enum(["PENDING", "CONFIRMED", "REJECTED"]);
export type WithdrawalStatusT = z.infer<typeof withdrawalStatusSchema>;

export const createWithdrawalRequestSchema = z.object({
  chain: withdrawalChainSchema,
  asset: withdrawalAssetSchema,
  destinationAddress: z.string().min(8).max(160),
  amountDin: z.number().int().min(1),
});
export type CreateWithdrawalRequestT = z.infer<typeof createWithdrawalRequestSchema>;

export const withdrawalRequestViewSchema = z.object({
  id: z.string().min(1),
  accountId: z.string().min(1),
  chain: withdrawalChainSchema,
  asset: withdrawalAssetSchema,
  destinationAddress: z.string().min(1),
  amountDin: z.number().int().min(1),
  txHash: z.string().nullable(),
  status: withdrawalStatusSchema,
  createdAt: z.number().int(),
  resolvedAt: z.number().int().nullable(),
});
export type WithdrawalRequestViewT = z.infer<typeof withdrawalRequestViewSchema>;

export const withdrawalListResponseSchema = z.object({
  items: z.array(withdrawalRequestViewSchema),
});
export type WithdrawalListResponseT = z.infer<typeof withdrawalListResponseSchema>;

// ── Phase M3 — identity, session, and TON deposits (mirror of crates/contracts/src/auth.rs) ─────

/** Telegram authentication request — the raw initData string, untouched. */
export const telegramAuthRequestSchema = z.object({
  initData: z.string().min(1),
});
export type TelegramAuthRequestT = z.infer<typeof telegramAuthRequestSchema>;

/** User view — `username` is optional (a missing key on the wire, not null). */
export const userViewSchema = z.object({
  id: z.string(),
  telegramId: z.string(),
  username: z.string().optional(),
  accountId: z.string(),
  tonAddress: z.string().nullable(),
  createdAt: z.string(),
});
export type UserViewT = z.infer<typeof userViewSchema>;

export const authResponseSchema = z.object({
  token: z.string(),
  expiresAt: z.string(),
  user: userViewSchema,
});
export type AuthResponseT = z.infer<typeof authResponseSchema>;

// ── Development/demo mode — one-click login and faucet (KIAN_DEV_MODE=1 only) ──────

/**
 * Demo/developer login request — mirror of `DevLoginRequest` in crates/contracts/src/auth.rs.
 * All fields except tgUserId are optional with sensible defaults; tgUserId is digits only.
 */
export const devLoginRequestSchema = z.object({
  tgUserId: z.string().regex(/^[0-9]{1,20}$/),
  username: z.string().min(1).max(64).optional(),
  firstName: z.string().min(1).max(120).optional(),
  startParam: z.string().max(160).nullable().optional(),
});
export type DevLoginRequestT = z.infer<typeof devLoginRequestSchema>;

/** Personas ready for local testing — aligned with loginAsDemoPersona in the mini-app. */
export const DEMO_PERSONAS = {
  creator: { tgUserId: "1001", username: "creator_demo", firstName: "Creator (demo persona)" },
  buyer: { tgUserId: "2002", username: "buyer_demo", firstName: "Buyer (demo persona)" },
  affiliate: { tgUserId: "3003", username: "affiliate_demo", firstName: "Affiliate (demo persona)" },
} as const;
export type DemoPersona = keyof typeof DEMO_PERSONAS;

/** Telegram IDs of the demo personas — for loading the active persona in the UI. */
export const DEMO_PERSONA_TG_IDS: readonly string[] = Object.values(DEMO_PERSONAS).map(
  (p) => p.tgUserId,
);

/** Default faucet amount — 5,000 KIAN in micro. */
export const FAUCET_DEFAULT_MICRO = 5_000_000_000;
/** Per-withdrawal faucet cap — 50,000 KIAN in micro. */
export const FAUCET_MAX_MICRO = 50_000_000_000;

/** Experimental faucet request — mirror of `FaucetRequest` in crates/contracts. */
export const faucetRequestSchema = z.object({
  amountMicro: z.number().int().min(1).max(FAUCET_MAX_MICRO).optional(),
});
export type FaucetRequestT = z.infer<typeof faucetRequestSchema>;

/** Wallet history row — `memo` is always on the wire (explicit null). */
export const walletTxViewSchema = z.object({
  txId: z.string(),
  amount: z.number().int(),
  memo: z.string().nullable(),
  at: z.string(),
});
export type WalletTxViewT = z.infer<typeof walletTxViewSchema>;

/** `meResponseSchema` — balance derived from the double-entry ledger. */
export const meResponseSchema = z.object({
  user: userViewSchema,
  balance: DinAmount,
  escrowLocked: DinAmount,
  recentTxs: z.array(walletTxViewSchema),
});
export type MeResponseT = z.infer<typeof meResponseSchema>;

export const setTonAddressRequestSchema = z.object({
  address: z.string().min(8).max(120),
});
export type SetTonAddressRequestT = z.infer<typeof setTonAddressRequestSchema>;

/** TON deposit-proof registration request — amount as a decimal string (never float). */
export const tonDepositCreateRequestSchema = z.object({
  txHash: z.string().min(8).max(128),
  walletAddress: z.string().min(8).max(120),
  amountTon: z.string().min(1).max(32),
});
export type TonDepositCreateRequestT = z.infer<typeof tonDepositCreateRequestSchema>;

export const tonDepositViewSchema = z.object({
  id: z.string(),
  txHash: z.string(),
  walletAddress: z.string(),
  amountTon: z.string(),
  status: z.enum(["PENDING", "CONFIRMED", "REJECTED"]),
  createdAt: z.string(),
  confirmedAt: z.string().nullable(),
});
export type TonDepositViewT = z.infer<typeof tonDepositViewSchema>;

export const tonDepositListResponseSchema = z.object({
  items: z.array(tonDepositViewSchema),
});
export type TonDepositListResponseT = z.infer<typeof tonDepositListResponseSchema>;

// ── Phase M2 — wallet transfer and burn responses (mirror of crates/contracts/src/wallet.rs) ──

export const burnRequestSchema = z.object({
  accountId: z.string().min(1),
  amount: DinAmount,
  memo: z.string().max(200).optional(),
  idempotencyKey: z.string().max(160).optional(),
});
export type BurnRequestT = z.infer<typeof burnRequestSchema>;

export const walletMoveResponseSchema = z.object({
  txId: z.string(),
  accountId: z.string(),
  balance: z.number().int(),
  replayed: z.boolean(),
});
export type WalletMoveResponseT = z.infer<typeof walletMoveResponseSchema>;

/** Admin approval request for a withdrawal payment. */
export const confirmWithdrawalRequestSchema = z.object({
  txHash: z.string().min(8).max(128),
});
export type ConfirmWithdrawalRequestT = z.infer<typeof confirmWithdrawalRequestSchema>;
