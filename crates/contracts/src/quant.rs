//! Causal quant contracts — the `ccxt` core and the `Vibe-Trading` engine (Wave 6 —
//! Step 3/4).
//!
//! three DTOs on the wire:
//! - [`QuantVenuesResponse`] — the keyless 8-venue matrix + the `Precise` tick
//!   self-test (ccxt's forensic remedy #1) + the `OrderBook` sequence-gap badge
//!   (remedy #2);
//! - [`QuantVibeBacktestRequest`] / [`QuantVibeBacktestResponse`] — running the
//!   5-node topological DAG on a bilingual natural prompt with a strictly
//!   causal backtester, a claims ledger, the Mairon gate, and `SKILL.md` forging;
//! - [`QuantStrategiesResponse`] — the list of `quant_strategies`
//!   `quant_strategies`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A venue endpoint on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVenueEndpointView {
    /// Use-case purpose (`ticker24h`, `depth`, …).
    pub purpose: String,
    /// HTTP method.
    pub method: String,
    /// Path or sample body.
    pub path: String,
    /// Declared rate-limit weight (0 = undeclared).
    pub weight: u32,
}

/// A keyless venue on the wire — zero API keys, zero KYC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVenueView {
    /// Stable id (`binance-spot`, …).
    pub id: String,
    /// Display name.
    pub display_name: String,
    /// Venue kind (`cex-spot` | `cex-linear-perp` | `dex-perp` | `dex-amm`).
    pub venue_kind: String,
    /// URL base.
    pub base_url: String,
    /// Required auth — always `none`.
    pub auth: String,
    /// Endpoints.
    pub endpoints: Vec<QuantVenueEndpointView>,
    /// Short note.
    pub note: String,
}

/// One `Precise` tick self-test row — canonical input/tick/output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantTickSelftestView {
    /// Raw input.
    pub input: String,
    /// Raw tick.
    pub tick: String,
    /// Canonical output.
    pub output: String,
}

/// `GET /quant/venues` — the keyless market matrix + evidence of the two forensic remedies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVenuesResponse {
    /// 8 zero-key venues.
    pub venues: Vec<QuantVenueView>,
    /// Tick self-test — 100% `Precise`, zero float drift.
    pub tick_selftest: Vec<QuantTickSelftestView>,
    /// Sequence-gap badge — `[Applied, IgnoredStale, SequenceGapDetected]`.
    pub orderbook_gap_guard: Vec<String>,
    /// Blind signature.
    pub signature: String,
}

/// `POST /quant/vibe-backtest` — request to run the quant DAG.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVibeBacktestRequest {
    /// Natural prompt — Persian or English.
    pub vibe_prompt: String,
    /// Symbol — default `BTC-USDT`.
    pub symbol: Option<String>,
    /// Timeframe — default `1h`.
    pub timeframe: Option<String>,
    /// Number of deterministically generated candles — default 720.
    pub bars: Option<u32>,
    /// Deterministic seed — default 2026.
    pub seed: Option<u64>,
}

/// Telemetry of one DAG node on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantSwarmNodeView {
    /// 1-based index.
    pub index: u32,
    /// The node's canonical name.
    pub name: String,
    /// Status.
    pub status: String,
    /// Deterministic summary.
    pub summary: String,
    /// Deterministic tick.
    pub deterministic_ticks: u64,
}

/// The backtest report's institutional metrics on the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantMetricsView {
    /// Causal proof — always `true`.
    pub causal_slice_proof: bool,
    /// Annualized Sharpe.
    pub sharpe_ratio: f64,
    /// Sortino.
    pub sortino_ratio: f64,
    /// Maximum drawdown (percent, negative).
    pub max_drawdown_pct: f64,
    /// Total return (percent).
    pub total_return_pct: f64,
    /// Annualized compound return (percent).
    pub cagr_pct: f64,
    /// Calmar.
    pub calmar_ratio: f64,
    /// Win rate (percent).
    pub win_rate_pct: f64,
    /// Profit factor.
    pub profit_factor: f64,
    /// Omega.
    pub omega_ratio: f64,
    /// Tail ratio.
    pub tail_ratio: f64,
    /// Skewness.
    pub skewness: f64,
    /// Excess kurtosis.
    pub kurtosis: f64,
    /// Capital turnover.
    pub turnover_ratio: f64,
    /// Number of trades.
    pub total_trades: u32,
    /// Number of candles.
    pub bars: u32,
    /// Final equity.
    pub final_equity: f64,
    /// Forced liquidation?
    pub liquidated: bool,
    /// Halt key?
    pub kill_switch_tripped: bool,
    /// Claims ledger hash.
    pub claim_verification_hash: String,
}

/// The Mairon gate verdict on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantGateView {
    /// Marketplace sale approval.
    pub approved: bool,
    /// Honest reason.
    pub reason: String,
    /// Was the deterministic regime boost applied?
    pub regime_boost_applied: bool,
}

/// `POST /quant/vibe-backtest` — the complete DAG response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVibeBacktestResponse {
    /// Strategy id (`vibe:<hash>`).
    pub strategy_id: String,
    /// Typed AST — full JSON round-trip.
    pub strategy_ast_json: serde_json::Value,
    /// Institutional metrics.
    pub metrics: QuantMetricsView,
    /// Telemetry of the 5 nodes in topological order.
    pub nodes: Vec<QuantSwarmNodeView>,
    /// The Mairon gate verdict.
    pub gate: QuantGateView,
    /// The forged `SKILL.md`.
    pub skill_md: String,
    /// Persisted row id (`quant:<uuid>`) — after insert.
    pub persisted_id: String,
    /// Signature of the causal quant engine.
    pub signature: String,
}

/// A persisted strategy on the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantStrategyView {
    /// Row id (`quant:<uuid>`).
    pub id: String,
    /// Strategy id (`vibe:<hash>`).
    pub strategy_id: String,
    /// Creator.
    pub creator_user_id: String,
    /// Human-readable name.
    pub name: String,
    /// Symbol.
    pub symbol: String,
    /// Timeframe.
    pub timeframe: String,
    /// Sharpe.
    pub sharpe_ratio: f64,
    /// Drawdown.
    pub max_drawdown_pct: f64,
    /// Mairon gate stamp.
    pub mairon_approved: bool,
    /// Claims hash.
    pub claim_verification_hash: String,
    /// Creation timestamp.
    pub created_at: String,
}

/// `GET /quant/strategies` — the list of persisted strategies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantStrategiesResponse {
    /// Rows in newest-first order.
    pub strategies: Vec<QuantStrategyView>,
    /// Total count.
    pub count: u32,
}
