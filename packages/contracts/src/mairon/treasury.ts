import { z } from "zod";

/**
 * MAIRON — Base-8453 Sovereign Treasury & Omni-Model Genome wire contracts.
 *
 * TypeScript mirror of `crates/contracts/src/monopoly.rs`: the four-stream
 * Day-1 revenue charter, the Base Mainnet settlement layer, the capital
 * reserve partition, and the 5-category x 9-slot Omni-Model Genome with both
 * selection modes (`ManualOverride` and `AgentAutoDecision`).
 */

// ── Canonical settlement constants ─────────────────────────────────────────────

/** Base Mainnet — the sole sovereign settlement chain. */
export const BASE_CHAIN_ID = 8453 as const;

/** Base-native USDC contract address. */
export const BASE_USDC_CONTRACT = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913" as const;

/** The sovereign platform token symbol. */
export const KIA_TOKEN_SYMBOL = "$0xKIA" as const;

/** Telegram Stars currency code — the instant-settlement rail. */
export const TELEGRAM_STARS_CURRENCY = "XTR" as const;

// ── Four-stream revenue charter ────────────────────────────────────────────────

/** Platform take on an unreferred sale — `2000 bps` = 20%. */
export const PLATFORM_TAKE_BPS_UNREFERRED = 2_000 as const;

/** Platform take on a referred sale — `1000 bps` = 10% (the affiliate earns the other 10%). */
export const PLATFORM_TAKE_BPS_REFERRED = 1_000 as const;

/** Perpetual creator share — `8000 bps` = 80%. */
export const CREATOR_PAYOUT_BPS = 8_000 as const;

/** Affiliate share on a referred sale — `1000 bps` = 10%. */
export const AFFILIATE_BPS_REFERRED = 1_000 as const;

/** Instant boosts route 0% to the creator — Stream 3 is a 100%-margin treasury purchase. */
export const INSTANT_BOOST_CREATOR_BPS = 0 as const;

/** Instant boosts route 100% (10000 bps) to the MAIRON Sovereign Treasury. */
export const INSTANT_BOOST_TREASURY_BPS = 10_000 as const;

/** Instant-settlement spread on Base USDC and Telegram Stars — `150 bps` = 1.5%. */
export const SETTLEMENT_SPREAD_BPS = 150 as const;

// ── Capital reserve partition ──────────────────────────────────────────────────

/** Operational crypto seed — `$33.00` = 33,000,000 micro-USDC. */
export const CRYPTO_SEED_MICROS = 33_000_000 as const;

/** Cash seed — `$15.00` = 1,500 cents. */
export const CASH_SEED_CENTS = 1_500 as const;

/** Locked cold paper reserve — `$350.00` = 35,000 cents. */
export const PAPER_RESERVE_LOCKED_CENTS = 35_000 as const;

/** Honest Day-1 earned revenue — `$9.55` = 9,550,000 micro-USDC. */
export const DAY1_TOTAL_EARNED_MICROS = 9_550_000 as const;

/** Day-1 wallet total on the crypto rail — `$42.55` = 42,550,000 micro-USDC. */
export const DAY1_WALLET_TOTAL_MICROS = 42_550_000 as const;

// ── Omni-Model Genome layout ───────────────────────────────────────────────────

/** Genome categories (each holds exactly 9 slots). */
export const GENOME_CATEGORIES = [
  "flash-agentic-execution",
  "web-synthesis",
  "deep-research",
  "code-supremacy",
  "sovereign-judgment",
] as const;
export type GenomeCategoryT = (typeof GENOME_CATEGORIES)[number];

/** Slots per category. */
export const GENOME_SLOTS_PER_CATEGORY = 9 as const;

/** Total genome models — 5 categories x 9 slots = 45. */
export const GENOME_TOTAL_MODELS = 45 as const;

/** The two genome selection modes. */
export const GENOME_SELECTION_MODES = ["ManualOverride", "AgentAutoDecision"] as const;
export type GenomeSelectionModeT = (typeof GENOME_SELECTION_MODES)[number];

// ── Direct mirrors of crates/contracts/src/monopoly.rs ─────────────────────────

/** Mirror of `MonopolyGenomeModelView`. */
export const MonopolyGenomeModelView = z.object({
  modelId: z.string(),
  aliases: z.array(z.string()),
  genomeDir: z.string(),
  specialty: z.string(),
  canonicalFiles: z.array(z.string()),
  filesVerified: z.number().int().min(0).max(9),
  isComplete: z.boolean(),
  trinityAvatar: z.string(),
  verifiedAgainst: z.string(),
});
export type MonopolyGenomeModelViewT = z.infer<typeof MonopolyGenomeModelView>;

/** Mirror of `MonopolyRoutingRuleView`. */
export const MonopolyRoutingRuleView = z.object({
  domain: z.string(),
  championModelId: z.string(),
  trinityAvatar: z.string(),
});
export type MonopolyRoutingRuleViewT = z.infer<typeof MonopolyRoutingRuleView>;

/** Mirror of `MonopolyGenomeMatrixResponse`. */
export const MonopolyGenomeMatrixResponse = z.object({
  signature: z.string(),
  models: z.array(MonopolyGenomeModelView),
  routingRules: z.array(MonopolyRoutingRuleView),
  totalFilesVerified: z.number().int().min(0),
  expectedFileCount: z.number().int().min(0),
  selectionModes: z.array(z.string()),
});
export type MonopolyGenomeMatrixResponseT = z.infer<typeof MonopolyGenomeMatrixResponse>;

/** Mirror of `MonopolyRouteModelRequest`. */
export const MonopolyRouteModelRequest = z.object({
  taskPrompt: z.string(),
  agentMode: z.boolean(),
  manualModelOverride: z.string().nullable(),
});
export type MonopolyRouteModelRequestT = z.infer<typeof MonopolyRouteModelRequest>;

/** Mirror of `MonopolyRouteModelResponse`. */
export const MonopolyRouteModelResponse = z.object({
  selectedModel: z.string(),
  selectionMode: z.enum(["manual_override", "agent_auto_decision"]),
  confidenceScore: z.number(),
  decisionRationale: z.string(),
  routedDomain: z.string(),
  trinityAvatar: z.string(),
  distilledSystemPrompt: z.string(),
  genomeDir: z.string(),
  filesVerified: z.number().int(),
  expectedFiles: z.number().int(),
});
export type MonopolyRouteModelResponseT = z.infer<typeof MonopolyRouteModelResponse>;

/** Mirror of `MonopolyCharterStreamView`. */
export const MonopolyCharterStreamView = z.object({
  stream: z.number().int().min(1).max(4),
  name: z.string(),
  bps: z.number().int(),
  destinationAccount: z.string(),
  note: z.string(),
});
export type MonopolyCharterStreamViewT = z.infer<typeof MonopolyCharterStreamView>;

/** Mirror of `MonopolyAuditPoint`. */
export const MonopolyAuditPoint = z.object({
  key: z.string(),
  value: z.string(),
});
export type MonopolyAuditPointT = z.infer<typeof MonopolyAuditPoint>;

/** Mirror of `MonopolyContractAuditView`. */
export const MonopolyContractAuditView = z.object({
  contractName: z.string(),
  sourcePath: z.string(),
  solidityVersion: z.string(),
  auditPoints: z.array(MonopolyAuditPoint),
});
export type MonopolyContractAuditViewT = z.infer<typeof MonopolyContractAuditView>;

/** Mirror of `MonopolyCharterAndBaseResponse`. */
export const MonopolyCharterAndBaseResponse = z.object({
  signature: z.string(),
  charterMotto: z.string(),
  streams: z.array(MonopolyCharterStreamView),
  baseChainId: z.number().int(),
  baseRpcUrl: z.string(),
  baseBlockExplorer: z.string(),
  baseUsdcContract: z.string(),
  tokenAudit: MonopolyContractAuditView,
  escrowAudit: MonopolyContractAuditView,
});
export type MonopolyCharterAndBaseResponseT = z.infer<typeof MonopolyCharterAndBaseResponse>;

/** Mirror of `MonopolySimulateSplitRequest`. */
export const MonopolySimulateSplitRequest = z.object({
  scenario: z.enum(["marketplace_sale", "instant_boost"]),
  settlementAsset: z.enum([
    "telegram-stars-xtr",
    "base-native-eth",
    "base-native-usdc",
    "zero-x-kia-base-token",
    "internal-micro-kian",
  ]),
  grossAmountMicros: z.number().int().min(0),
  buyerUserId: z.string(),
  creatorUserId: z.string(),
  referrerUserId: z.string().nullable(),
  affiliateBps: z.number().int().min(1_000).max(5_000),
  boostTier: z.string().nullable(),
  recipientAddress: z.string().nullable(),
});
export type MonopolySimulateSplitRequestT = z.infer<typeof MonopolySimulateSplitRequest>;

/** Mirror of `MonopolySplitLegView`. */
export const MonopolySplitLegView = z.object({
  legKind: z.string(),
  accountId: z.string(),
  amountMicros: z.number().int(),
  bps: z.number().int(),
  note: z.string(),
});
export type MonopolySplitLegViewT = z.infer<typeof MonopolySplitLegView>;

/** Mirror of `MonopolySimulateSplitResponse`. */
export const MonopolySimulateSplitResponse = z.object({
  splitId: z.string(),
  scenario: z.string(),
  settlementAsset: z.string(),
  grossAmountMicros: z.number().int(),
  kiaTreasuryMicros: z.number().int(),
  referrerMicros: z.number().int(),
  creatorNetMicros: z.number().int(),
  effectivePlatformBps: z.number().int(),
  legs: z.array(MonopolySplitLegView),
  zeroSumProof: z.string(),
  persisted: z.boolean(),
  eip681PaymentUri: z.string().nullable(),
  orchestratorStatus: z.string(),
});
export type MonopolySimulateSplitResponseT = z.infer<typeof MonopolySimulateSplitResponse>;

// ── Omni-Model Genome slot registry ────────────────────────────────────────────

/**
 * OmniModelGenomeSlot — one administrative routing cell of the 45-model
 * registry, in either selection mode.
 */
export const OmniModelGenomeSlot = z.object({
  categoryId: z.enum([
    "flash-agentic-execution",
    "web-synthesis",
    "deep-research",
    "code-supremacy",
    "sovereign-judgment",
  ]),
  slotIndex: z.number().int().min(1).max(9),
  modelId: z.string().min(1).max(96),
  selectionMode: z.enum(["ManualOverride", "AgentAutoDecision"]),
  routedAvatar: z.enum(["alpha", "logic_prime", "mairon"]),
  filesVerified: z.number().int().min(0).max(9),
  enabled: z.boolean(),
});
export type OmniModelGenomeSlotT = z.infer<typeof OmniModelGenomeSlot>;
