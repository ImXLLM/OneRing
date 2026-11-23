import { z } from "zod";

/**
 * قراردادهای وایر KIAN — قرینهٔ TypeScript محاسبات crates/domain و crates/contracts.
 *
 * منبع حقیقت: Rust (کامیون crates/contracts با utoipa + گلدن‌تست‌ها) → اینجا فقط
 * mirror با Zod برای کلاینت‌های وب/مینی‌اپ. قواعد سیم: camelCase، null صریح برای
 * nullable، صحیح تا 2^53−1 (Din) — همان legacy تا dual-run بدون تبدیل کار کند.
 */

/** Din — واحد پول عدد صحیح؛ سقف 2^53−1 (مرز Number.MAX_SAFE_INTEGER عمداً حفظ شده) */
export const DinAmount = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);

/** هشت حالت ماشین امانت — یک‌به‌یک با `kian_domain::EscrowStatus` (SCREAMING_SNAKE_CASE) */
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

/** نمای امانت روی سیم — قرینهٔ `EscrowView` در crates/contracts/src/escrows.rs */
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

/** پاسخ استاندارد اقدام امانت — قرینهٔ `EscrowActionResponse` */
export const EscrowActionResponse = z.object({
  escrow: EscrowView,
  txId: z.string().nullable(),
  replayed: z.boolean(),
});

/** نمای حساب — تراز مشتق از `Posting` (ADR-003: ترازها ذخیره نمی‌شوند) */
export const AccountView = z.object({
  id: z.string(),
  kind: z.string(),
  label: z.string().nullable(),
  balance: z.number().int(),
  createdAt: z.string(),
});

/** بدنهٔ خطای یکدست — همیشه `{ error: { code, message, meta? } }` */
export const ErrorEnvelope = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
    meta: z.record(z.string(), z.union([z.string(), z.number()])).optional(),
  }),
});

/** خلاصهٔ اقتصاد — `GET /api/v1/economy` (شفافیت مادهٔ ۶؛ عمومی و فقط‌خواندنی) */
export const EconomySummary = z.object({
  currency: z.literal("DNX"),
  symbol: z.literal("Đ"),
  circulatingSupply: DinAmount,
  treasuryBalance: DinAmount,
  escrowLocked: DinAmount,
  mintedAllTime: DinAmount,
  burnedAllTime: DinAmount,
  mintedThisYear: DinAmount,
  /** سقف ضرب سالانهٔ منشور — مادهٔ ۳ */
  yearlyMintCap: DinAmount,
  genesisSupply: DinAmount,
  trialBalanceSum: z.number().int(),
  balanced: z.boolean(),
  accountCount: z.number().int(),
  openEscrows: z.number().int(),
});

/** لایوهنس — `GET /api/v1/health` (کمی از فاز legacy؛ bot/ai در M2 صادقانه disabled) */
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

// ---------- درخواست‌ها ----------

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
  /** TTL ساعت (۱ تا ۷۲۰) */
  ttlHours: z.number().int().min(1).max(720).optional(),
});

export const EscrowActionRequest = z.object({
  idempotencyKey: z.string().max(160).optional(),
});

// ---------- بازار، آرتیفکت HTML، رفرال و تحویل (فاز ۳) ----------

/** سقف حجم محتوای تحویل محصول (۱۵۰ کیلوبایت برای HTML تعاملی) */
export const MAX_DELIVERY_PAYLOAD_BYTES = 150_000;

/** سقف حجم پیش‌نمایش عمومی HTML پیش از خرید (۵۰ کیلوبایت) */
export const MAX_PREVIEW_HTML_BYTES = 50_000;

/** نرخ پیش‌فرض و سقف پورسانت معرف بر حسب نقاط پایه */
export const DEFAULT_AFFILIATE_BPS = 1_000;
export const MAX_AFFILIATE_BPS = 5_000;

/** نمای عمومی محصول — قرینهٔ `ProductView` در crates/contracts/src/market.rs */
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

/** پاسخ تحویل محتوای کامل محصول — قرینهٔ `EscrowDeliveryView` */
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

// ---------- استودیوی ساخت محصول HTML با AI (فاز ۴) ----------

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

/** گذارهای مجاز ماشین امانت — قرینهٔ ADR-004 (فقط برای UI؛ حقیقت سمت کرنل است) */
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

/** اندپوینت‌های کرنل پول M2 — مرجع کلاینت (همه ادمین به‌جز health/economy) */
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

/** کلید هدر احراز ادمین — fail-closed (سرور بدون کلید = اندپوینت‌های ادمین خاموش) */
export const KIAN_KEY_HEADER = "X-KIAN-Key";

// ── فاز ۶ — درخواست برداشت کریپتویی (TON / USDT روی TON, TRX, SOL, BSC) ───────

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

// ── فاز M3 — هویت، نشست و واریز TON (قرینهٔ crates/contracts/src/auth.rs) ─────

/** درخواست احراز تلگرام — رشتهٔ خام initData بدون دست‌کاری. */
export const telegramAuthRequestSchema = z.object({
  initData: z.string().min(1),
});
export type TelegramAuthRequestT = z.infer<typeof telegramAuthRequestSchema>;

/** نمای کاربر — `username` اختیاری است (کلید غایب روی سیم، نه null). */
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

/** ردیف تاریخچهٔ کیف — `memo` همیشه روی سیم است (null صریح). */
export const walletTxViewSchema = z.object({
  txId: z.string(),
  amount: z.number().int(),
  memo: z.string().nullable(),
  at: z.string(),
});
export type WalletTxViewT = z.infer<typeof walletTxViewSchema>;

/** `meResponseSchema` — تراز مشتق از دفتر دوطرفه. */
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

/** درخواست ثبت اثبات واریز TON — مبلغ رشتهٔ دهدهی (هرگز float). */
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

// ── فاز M2 — پاسخ جابه‌جایی کیف و سوزاندن (قرینهٔ crates/contracts/src/wallet.rs) ──

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

/** درخواست تأیید پرداخت برداشت توسط ادمین. */
export const confirmWithdrawalRequestSchema = z.object({
  txHash: z.string().min(8).max(128),
});
export type ConfirmWithdrawalRequestT = z.infer<typeof confirmWithdrawalRequestSchema>;
