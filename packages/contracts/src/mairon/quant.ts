import { z } from "zod";

/**
 * MAIRON — Mairon Quant & CCXT execution wire contracts.
 *
 * TypeScript mirror of `crates/contracts/src/quant.rs`: fixed-point decimals,
 * tick-size desks, the typed strategy AST, the causal backtest report with its
 * `Sharpe >= 1.20` live-deployment gate and SHA-256 claim hash, and the eight
 * keyless venue descriptors (zero API keys, zero KYC).
 */

// ── Institutional framing ─────────────────────────────────────────────────────

/** The single live-deployment gate — a strategy trades real capital only above this Sharpe. */
export const QUANT_LIVE_SHARPE_GATE = 1.2 as const;

/** Micro-lot scale of `FixedPointI128` — six decimal places, integer math only. */
export const FIXED_POINT_SCALE_MICROS = 1_000_000 as const;

/**
 * FixedPointI128 — a 128-bit-range fixed-point decimal carried on the wire as
 * a signed micro-integer plus its display string (never a float).
 */
export const FixedPointI128 = z.object({
  micros: z.number().int(),
  /** Canonical display form, e.g. `"12.500000"`. */
  display: z.string().regex(/^-?\d+\.\d{6}$/),
  scale: z.literal(6),
});
export type FixedPointI128T = z.infer<typeof FixedPointI128>;

/** TickSize — the venue's minimum price increment for one symbol. */
export const TickSize = z.object({
  venueId: z.string().min(1).max(64),
  symbol: z.string().min(1).max(32),
  tick: FixedPointI128,
  lotSize: FixedPointI128,
});
export type TickSizeT = z.infer<typeof TickSize>;

// ── Typed strategy AST ─────────────────────────────────────────────────────────

/** AST node kinds of the vibe-trading strategy language. */
export const STRATEGY_AST_NODE_KINDS = [
  "program", "signal", "candles_slice", "indicator", "comparison",
  "logic", "action", "risk",
] as const;
export type StrategyAstNodeKindT = (typeof STRATEGY_AST_NODE_KINDS)[number];

/**
 * StrategyAst — the typed, JSON round-trippable strategy tree the engine
 * parses from the natural vibe prompt. Every causal read is bounded to
 * `&candles[0..=t]` by construction.
 */
export const StrategyAst = z.object({
  kind: z.enum([
    "program", "signal", "candles_slice", "indicator", "comparison",
    "logic", "action", "risk",
  ]),
  label: z.string().min(1).max(64),
  params: z.record(z.string(), z.union([z.string(), z.number(), z.boolean()])),
  children: z.array(z.lazy(() => StrategyAst)),
});
export type StrategyAstT = z.infer<typeof StrategyAst>;

// ── Backtest ───────────────────────────────────────────────────────────────────

/**
 * BacktestCausalReport — the institutional verdict on one backtest: the
 * causal-slice proof, the fourteen metrics, the `Sharpe >= 1.20` live gate,
 * and the SHA-256 claim-verification hash.
 */
export const BacktestCausalReport = z.object({
  strategyId: z.string().min(1).max(128),
  causalSliceProof: z.literal(true),
  sharpeRatio: z.number(),
  sortinoRatio: z.number(),
  maxDrawdownPct: z.number(),
  totalReturnPct: z.number(),
  cagrPct: z.number(),
  calmarRatio: z.number(),
  winRatePct: z.number(),
  profitFactor: z.number(),
  omegaRatio: z.number(),
  tailRatio: z.number(),
  skewness: z.number(),
  kurtosis: z.number(),
  turnoverRatio: z.number(),
  totalTrades: z.number().int().min(0),
  bars: z.number().int().min(0),
  finalEquity: z.number(),
  liquidated: z.boolean(),
  killSwitchTripped: z.boolean(),
  claimVerificationHash: z.string().min(16).max(128),
  liveGate: z.object({
    thresholdSharpe: z.literal(1.2),
    passed: z.boolean(),
  }),
});
export type BacktestCausalReportT = z.infer<typeof BacktestCausalReport>;

// ── Direct mirrors of crates/contracts/src/quant.rs ────────────────────────────

/** Mirror of `QuantVenueEndpointView`. */
export const QuantVenueEndpointView = z.object({
  purpose: z.string(),
  method: z.string(),
  path: z.string(),
  weight: z.number().int().min(0),
});
export type QuantVenueEndpointViewT = z.infer<typeof QuantVenueEndpointView>;

/** Mirror of `QuantVenueView` — one keyless venue descriptor. */
export const QuantVenueView = z.object({
  id: z.string(),
  displayName: z.string(),
  venueKind: z.enum(["cex-spot", "cex-linear-perp", "dex-perp", "dex-amm"]),
  baseUrl: z.string(),
  auth: z.literal("none"),
  endpoints: z.array(QuantVenueEndpointView),
  note: z.string(),
});
export type QuantVenueViewT = z.infer<typeof QuantVenueView>;

/** Mirror of `QuantTickSelftestView`. */
export const QuantTickSelftestView = z.object({
  input: z.string(),
  tick: z.string(),
  output: z.string(),
});
export type QuantTickSelftestViewT = z.infer<typeof QuantTickSelftestView>;

/** Mirror of `QuantVenuesResponse` — the 8-venue matrix + forensic remedies. */
export const QuantVenuesResponse = z.object({
  venues: z.array(QuantVenueView),
  tickSelftest: z.array(QuantTickSelftestView),
  orderbookGapGuard: z.array(z.string()),
  signature: z.string(),
});
export type QuantVenuesResponseT = z.infer<typeof QuantVenuesResponse>;

/** Mirror of `QuantVibeBacktestRequest`. */
export const QuantVibeBacktestRequest = z.object({
  vibePrompt: z.string(),
  symbol: z.string().optional(),
  timeframe: z.string().optional(),
  bars: z.number().int().min(50).max(5000).optional(),
  seed: z.number().int().min(0).optional(),
});
export type QuantVibeBacktestRequestT = z.infer<typeof QuantVibeBacktestRequest>;

/** Mirror of `QuantSwarmNodeView`. */
export const QuantSwarmNodeView = z.object({
  index: z.number().int().min(1),
  name: z.string(),
  status: z.string(),
  summary: z.string(),
  deterministicTicks: z.number().int().min(0),
});
export type QuantSwarmNodeViewT = z.infer<typeof QuantSwarmNodeView>;

/** Mirror of `QuantMetricsView`. */
export const QuantMetricsView = z.object({
  causalSliceProof: z.boolean(),
  sharpeRatio: z.number(),
  sortinoRatio: z.number(),
  maxDrawdownPct: z.number(),
  totalReturnPct: z.number(),
  cagrPct: z.number(),
  calmarRatio: z.number(),
  winRatePct: z.number(),
  profitFactor: z.number(),
  omegaRatio: z.number(),
  tailRatio: z.number(),
  skewness: z.number(),
  kurtosis: z.number(),
  turnoverRatio: z.number(),
  totalTrades: z.number().int(),
  bars: z.number().int(),
  finalEquity: z.number(),
  liquidated: z.boolean(),
  killSwitchTripped: z.boolean(),
  claimVerificationHash: z.string(),
});
export type QuantMetricsViewT = z.infer<typeof QuantMetricsView>;

/** Mirror of `QuantGateView`. */
export const QuantGateView = z.object({
  approved: z.boolean(),
  reason: z.string(),
  regimeBoostApplied: z.boolean(),
});
export type QuantGateViewT = z.infer<typeof QuantGateView>;

/** Mirror of `QuantVibeBacktestResponse`. */
export const QuantVibeBacktestResponse = z.object({
  strategyId: z.string(),
  strategyAstJson: z.unknown(),
  metrics: QuantMetricsView,
  nodes: z.array(QuantSwarmNodeView),
  gate: QuantGateView,
  skillMd: z.string(),
  persistedId: z.string(),
  signature: z.string(),
});
export type QuantVibeBacktestResponseT = z.infer<typeof QuantVibeBacktestResponse>;

/** Mirror of `QuantStrategyView`. */
export const QuantStrategyView = z.object({
  id: z.string(),
  strategyId: z.string(),
  creatorUserId: z.string(),
  name: z.string(),
  symbol: z.string(),
  timeframe: z.string(),
  sharpeRatio: z.number(),
  maxDrawdownPct: z.number(),
  maironApproved: z.boolean(),
  claimVerificationHash: z.string(),
  createdAt: z.string(),
});
export type QuantStrategyViewT = z.infer<typeof QuantStrategyView>;

/** Mirror of `QuantStrategiesResponse`. */
export const QuantStrategiesResponse = z.object({
  strategies: z.array(QuantStrategyView),
  count: z.number().int(),
});
export type QuantStrategiesResponseT = z.infer<typeof QuantStrategiesResponse>;
