import { z } from "zod";

/**
 * MAIRON — Founder War-Room wire contracts.
 *
 * TypeScript mirror of `crates/contracts/src/wave10.rs`: the complete war-room
 * state snapshot with its 24 hourly causal ticks, the co-founder chat cast,
 * chairman directives, the honest Day-1 wallet ledger (`+$9.55` -> `$42.55`),
 * and the ELI5 explain-simple pipeline cards.
 */

// ── Canonical scenario constants ───────────────────────────────────────────────

/** Hourly causal ticks in the canonical scenario — `0..=23`. */
export const WAR_ROOM_TICK_COUNT = 24 as const;

/** Honest Day-1 earned revenue — `$9.55` = 9,550,000 micro-USDC. */
export const WAR_ROOM_DAY1_EARNED_MICROS = 9_550_000 as const;

/** Day-1 organization wallet total — `$42.55` = 42,550,000 micro-USDC. */
export const WAR_ROOM_DAY1_WALLET_MICROS = 42_550_000 as const;

/** Anti-jump cap — the largest clock delta a single step may consume, in seconds. */
export const WAR_ROOM_MAX_TICK_DELTA_SECONDS = 300 as const;

/** The five co-founder persona wire IDs. */
export const WAR_ROOM_PERSONA_IDS = ["chairman", "elon_musk", "cz_binance", "jeff_bezos"] as const;
export type WarRoomPersonaIdT = (typeof WAR_ROOM_PERSONA_IDS)[number];

// ── Direct mirrors of crates/contracts/src/wave10.rs ───────────────────────────

/** Mirror of `FounderView` — one war-room persona. */
export const FounderView = z.object({
  personaId: z.string(),
  handle: z.string(),
  displayNameEn: z.string(),
  displayNameFa: z.string(),
  mandateEn: z.string(),
  mandateFa: z.string(),
  accentHex: z.string(),
  isCoFounder: z.boolean(),
});
export type FounderViewT = z.infer<typeof FounderView>;

/** Mirror of `WalletLedgerView` — every honest Day-0 figure. */
export const WalletLedgerView = z.object({
  cryptoSeedMicros: z.number().int(),
  cashSeedCents: z.number().int(),
  paperReserveLockedCents: z.number().int(),
  earnedRevenueMicros: z.number().int(),
  creatorPayoutsMicros: z.number().int(),
  affiliatePayoutsMicros: z.number().int(),
  streamMarketplaceTaxMicros: z.number().int(),
  streamAffiliateRoutingMicros: z.number().int(),
  streamFeaturedBoostMicros: z.number().int(),
  streamSettlementSpreadMicros: z.number().int(),
  activeUsers: z.number().int(),
  funnelVisits: z.number().int(),
  funnelSignups: z.number().int(),
  funnelPurchases: z.number().int(),
  earnedRevenueUsd: z.string(),
  cryptoSeedUsd: z.string(),
  cashSeedUsd: z.string(),
  paperReserveUsd: z.string(),
});
export type WalletLedgerViewT = z.infer<typeof WalletLedgerView>;

/** Mirror of `WarRoomTickView` — one causal simulation tick. */
export const WarRoomTickView = z.object({
  tickIndex: z.number().int().min(0).max(23),
  simulationSecond: z.number().int().min(0),
  speakerId: z.string(),
  speakerNameEn: z.string(),
  speakerNameFa: z.string(),
  speakerAccentHex: z.string(),
  messageFa: z.string(),
  messageEn: z.string(),
  terminalCmd: z.string().nullable(),
  socialPostJson: z.string().nullable(),
  walletDeltaUsdcMicros: z.number().int(),
  walletTotalUsdcMicros: z.number().int(),
  activePipelineStage: z.string(),
  revenueStream: z.string(),
  saleVolumeMicros: z.number().int(),
  creatorPayoutMicros: z.number().int(),
  referred: z.boolean(),
});
export type WarRoomTickViewT = z.infer<typeof WarRoomTickView>;

/** Mirror of `WarRoomStateResponse` — the complete war-room snapshot. */
export const WarRoomStateResponse = z.object({
  signature: z.string(),
  githubRepo: z.string(),
  githubCommitShort: z.string(),
  baseChainId: z.number().int(),
  baseUsdcContract: z.string(),
  telegramStarsCurrency: z.string(),
  elapsedSeconds: z.number().int().min(0),
  tickCursor: z.number().int().min(0).max(24),
  progressPercent: z.number().int().min(0).max(100),
  playing: z.boolean(),
  speedMultiplier: z.union([z.literal(1), z.literal(2), z.literal(5)]),
  wallet: WalletLedgerView,
  founders: z.array(FounderView),
  scenario: z.array(WarRoomTickView).length(24),
  charterUnreferredTakeBps: z.number().int(),
  charterCreatorBps: z.number().int(),
  maxTickDeltaSeconds: z.number().int(),
  day1TotalEarnedMicros: z.number().int(),
});
export type WarRoomStateResponseT = z.infer<typeof WarRoomStateResponse>;

/** Mirror of `WarRoomStepRequest`. */
export const WarRoomStepRequest = z.object({
  tickCursor: z.number().int().min(0).max(24),
  speedMultiplier: z.union([z.literal(1), z.literal(2), z.literal(5)]),
});
export type WarRoomStepRequestT = z.infer<typeof WarRoomStepRequest>;

/** Mirror of `WarRoomStepResponse`. */
export const WarRoomStepResponse = z.object({
  executed: WarRoomTickView,
  tickCursor: z.number().int().min(0).max(24),
  progressPercent: z.number().int().min(0).max(100),
  elapsedSeconds: z.number().int().min(0),
  wallet: WalletLedgerView,
  scenarioExhausted: z.boolean(),
});
export type WarRoomStepResponseT = z.infer<typeof WarRoomStepResponse>;

/** Mirror of `WarRoomDirectiveRequest`. */
export const WarRoomDirectiveRequest = z.object({
  directiveText: z.string().min(1).max(2000),
  locale: z.enum(["fa", "en"]),
  triggeredTick: z.number().int().min(0).max(24),
});
export type WarRoomDirectiveRequestT = z.infer<typeof WarRoomDirectiveRequest>;

/** Mirror of `FounderReplyView`. */
export const FounderReplyView = z.object({
  speakerId: z.string(),
  speakerNameEn: z.string(),
  speakerNameFa: z.string(),
  accentHex: z.string(),
  messageFa: z.string(),
  messageEn: z.string(),
  terminalCmd: z.string().nullable(),
  socialPostJson: z.string().nullable(),
});
export type FounderReplyViewT = z.infer<typeof FounderReplyView>;

/** Mirror of `WarRoomDirectiveResponse`. */
export const WarRoomDirectiveResponse = z.object({
  directiveText: z.string(),
  intentId: z.string(),
  intentLabelFa: z.string(),
  intentLabelEn: z.string(),
  replies: z.array(FounderReplyView),
  triggeredTick: z.number().int(),
  simulationSecond: z.number().int(),
});
export type WarRoomDirectiveResponseT = z.infer<typeof WarRoomDirectiveResponse>;

/** Mirror of `WarRoomExplainResponse` — the "0 to 100" ELI5 bundle. */
export const WarRoomExplainResponse = z.object({
  signature: z.string(),
  locale: z.enum(["fa", "en"]),
  headlineFa: z.string(),
  headlineEn: z.string(),
  charterFa: z.string(),
  charterEn: z.string(),
  stepIds: z.array(z.string()),
  stepTitlesFa: z.array(z.string()),
  stepTitlesEn: z.array(z.string()),
  stepBodiesFa: z.array(z.string()),
  stepBodiesEn: z.array(z.string()),
});
export type WarRoomExplainResponseT = z.infer<typeof WarRoomExplainResponse>;

/** Mirror of `WarRoomResetResponse`. */
export const WarRoomResetResponse = z.object({
  signature: z.string(),
  deletedTicks: z.number().int().min(0),
  deletedDirectives: z.number().int().min(0),
  tickCursor: z.literal(0),
  progressPercent: z.literal(0),
  messageFa: z.string(),
  messageEn: z.string(),
});
export type WarRoomResetResponseT = z.infer<typeof WarRoomResetResponse>;

// ── ELI5 pipeline card framing ─────────────────────────────────────────────────

/** One ELI5 pipeline card rendered from the explain-simple bundle. */
export const WarRoomEli5Card = z.object({
  stepId: z.string(),
  titleEn: z.string(),
  bodyEn: z.string(),
  order: z.number().int().min(1),
});
export type WarRoomEli5CardT = z.infer<typeof WarRoomEli5Card>;
