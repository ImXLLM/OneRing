//! Wire contracts of Wave 10 (Commit #30) — the sovereign founder war room and
//! the real-time "Day 0 → Day 1" simulation engine.
//!
//! On-wire DTOs for the four new endpoints:
//! - `GET  /api/v1/war-room/state` — the complete Day-0 state + the
//!   canonical 24-tick scenario ([`WarRoomStateResponse`]);
//! - `POST /api/v1/war-room/step` — single-step causal advance
//!   ([`WarRoomStepRequest`], [`WarRoomStepResponse`]);
//! - `POST /api/v1/war-room/directive` — the boss's tactical directive + three
//!   in-character replies ([`WarRoomDirectiveRequest`], [`WarRoomDirectiveResponse`]);
//! - `GET  /api/v1/war-room/explain-simple` — the "0-to-100" explanation for
//!   zero-literacy audiences ([`WarRoomExplainResponse`]).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Wire view of a co-founder persona.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FounderView {
    /// Wire ID — `elon_musk` | `cz_binance` | `jeff_bezos` | `chairman`.
    pub persona_id: String,
    /// X handle.
    pub handle: String,
    /// Display name — English.
    pub display_name_en: String,
    /// Display name — Persian.
    pub display_name_fa: String,
    /// Mission — English.
    pub mandate_en: String,
    /// Mission — Persian.
    pub mandate_fa: String,
    /// Accent color as hex.
    pub accent_hex: String,
    /// 24/7 network member?
    pub is_co_founder: bool,
}

/// Wire view of the org wallet ledger — every honest Day-0 figure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WalletLedgerView {
    /// Crypto seed (`µUSDC`) — `33_000_000`.
    pub crypto_seed_micros: i64,
    /// Cash seed (cents) — `1_500`.
    pub cash_seed_cents: i64,
    /// Locked paper reserve (cents) — `35_000`.
    pub paper_reserve_locked_cents: i64,
    /// Platform net revenue (`µUSDC`) — starting from zero.
    pub earned_revenue_micros: i64,
    /// 80% creator payouts (`µUSDC`).
    pub creator_payouts_micros: i64,
    /// Affiliate payouts (`µUSDC`).
    pub affiliate_payouts_micros: i64,
    /// Marketplace tax bucket (`µUSDC`).
    pub stream_marketplace_tax_micros: i64,
    /// Affiliate routing bucket (`µUSDC`).
    pub stream_affiliate_routing_micros: i64,
    /// Featured boost bucket (`µUSDC`).
    pub stream_featured_boost_micros: i64,
    /// Settlement variance bucket (`µUSDC`).
    pub stream_settlement_spread_micros: i64,
    /// Active users.
    pub active_users: i64,
    /// Visit funnel.
    pub funnel_visits: i64,
    /// Signup funnel.
    pub funnel_signups: i64,
    /// Purchase funnel.
    pub funnel_purchases: i64,
    /// Revenue as a dollar string — `$9.55`.
    pub earned_revenue_usd: String,
    /// Crypto seed as a dollar string — `$33.00`.
    pub crypto_seed_usd: String,
    /// Cash seed as a dollar string — `$15.00`.
    pub cash_seed_usd: String,
    /// Paper reserve as a dollar string — `$350.00`.
    pub paper_reserve_usd: String,
}

/// Wire view of one simulation tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomTickView {
    /// Tick number.
    pub tick_index: i64,
    /// Simulation seconds.
    pub simulation_second: i64,
    /// Speaker ID.
    pub speaker_id: String,
    /// Speaker display name — English.
    pub speaker_name_en: String,
    /// Speaker display name — Persian.
    pub speaker_name_fa: String,
    /// Speaker accent color.
    pub speaker_accent_hex: String,
    /// Persian message.
    pub message_fa: String,
    /// English message.
    pub message_en: String,
    /// Terminal command — if executed.
    pub terminal_cmd: Option<String>,
    /// Social post as JSON — if published.
    pub social_post_json: Option<String>,
    /// Treasury delta (`µUSDC`).
    pub wallet_delta_usdc_micros: i64,
    /// Wallet total after the tick (`µUSDC`).
    pub wallet_total_usdc_micros: i64,
    /// Active stage.
    pub active_pipeline_stage: String,
    /// Active revenue stream.
    pub revenue_stream: String,
    /// Gross volume (`µUSDC`).
    pub sale_volume_micros: i64,
    /// Creator payout (`µUSDC`).
    pub creator_payout_micros: i64,
    /// Referred sale?
    pub referred: bool,
}

/// Response of `GET /api/v1/war-room/state` — the complete war-room state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomStateResponse {
    /// War-room signature.
    pub signature: String,
    /// Target repository.
    pub github_repo: String,
    /// Short hash of the flagship metric.
    pub github_commit_short: String,
    /// Base chain ID.
    pub base_chain_id: i64,
    /// Base-native USDC contract.
    pub base_usdc_contract: String,
    /// Telegram Stars symbol.
    pub telegram_stars_currency: String,
    /// Simulation clock — elapsed seconds.
    pub elapsed_seconds: i64,
    /// Next-tick pointer.
    pub tick_cursor: i64,
    /// Progress percentage.
    pub progress_percent: i64,
    /// Autoplay enabled?
    pub playing: bool,
    /// Speed multiplier — 1 | 2 | 5.
    pub speed_multiplier: i64,
    /// Org wallet ledger.
    pub wallet: WalletLedgerView,
    /// War-room personas.
    pub founders: Vec<FounderView>,
    /// The canonical 24-tick scenario — the single source of truth for the HTML app.
    pub scenario: Vec<WarRoomTickView>,
    /// Charter constants for the UI (bps).
    pub charter_unreferred_take_bps: i64,
    /// `8000 bps` perpetual creator share.
    pub charter_creator_bps: i64,
    /// Anti-jump cap (seconds).
    pub max_tick_delta_seconds: i64,
    /// The honest Day-1 revenue total (`µUSDC`) — `9_550_000`.
    pub day1_total_earned_micros: i64,
}

/// Request of `POST /api/v1/war-room/step`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomStepRequest {
    /// Ticks executed so far — `0..=24`.
    pub tick_cursor: i64,
    /// Selected speed multiplier — only 1, 2, and 5.
    pub speed_multiplier: i64,
}

/// Response of `POST /api/v1/war-room/step` — the executed tick + fresh state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomStepResponse {
    /// The executed tick — full.
    pub executed: WarRoomTickView,
    /// Tick pointer after execution.
    pub tick_cursor: i64,
    /// Progress percentage.
    pub progress_percent: i64,
    /// Simulation seconds.
    pub elapsed_seconds: i64,
    /// Wallet ledger after execution.
    pub wallet: WalletLedgerView,
    /// Is the scenario finished?
    pub scenario_exhausted: bool,
}

/// Request of `POST /api/v1/war-room/directive`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomDirectiveRequest {
    /// Boss directive text — Persian or English, 2000-character cap.
    pub directive_text: String,
    /// Reply locale — `fa` | `en`.
    pub locale: String,
    /// Tick active at injection time (for causal logging).
    pub triggered_tick: i64,
}

/// One in-character reply on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FounderReplyView {
    /// Responder ID.
    pub speaker_id: String,
    /// English name.
    pub speaker_name_en: String,
    /// Persian name.
    pub speaker_name_fa: String,
    /// Accent color.
    pub accent_hex: String,
    /// Persian reply.
    pub message_fa: String,
    /// English reply.
    pub message_en: String,
    /// Suggested terminal command.
    pub terminal_cmd: Option<String>,
    /// Suggested social post as JSON.
    pub social_post_json: Option<String>,
}

/// Response of `POST /api/v1/war-room/directive`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomDirectiveResponse {
    /// Directive text — after sanitization.
    pub directive_text: String,
    /// Detected intent.
    pub intent_id: String,
    /// Persian intent label.
    pub intent_label_fa: String,
    /// English intent label.
    pub intent_label_en: String,
    /// Three in-character replies: Elon, CZ, Jeff.
    pub replies: Vec<FounderReplyView>,
    /// Tick active at injection time.
    pub triggered_tick: i64,
    /// Simulation seconds.
    pub simulation_second: i64,
}

/// Response of `GET /api/v1/war-room/explain-simple` — the "0-to-100" explanation for zero-literacy audiences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomExplainResponse {
    /// War-room signature.
    pub signature: String,
    /// Bundle locale — `fa` | `en`.
    pub locale: String,
    /// Headline — Persian.
    pub headline_fa: String,
    /// Headline — English.
    pub headline_en: String,
    /// Money charter — Persian.
    pub charter_fa: String,
    /// Money charter — English.
    pub charter_en: String,
    /// Four steps — IDs in alphabetical order.
    pub step_ids: Vec<String>,
    /// Persian title of each step.
    pub step_titles_fa: Vec<String>,
    /// English title of each step.
    pub step_titles_en: Vec<String>,
    /// Persian body of each step.
    pub step_bodies_fa: Vec<String>,
    /// English body of each step.
    pub step_bodies_en: Vec<String>,
}

/// Response of `POST /api/v1/war-room/reset` — causal rollback to Day 0.
///
/// Recorded ticks and boss directives are cleared from the command ledger so that
/// `POST /war-room/step` can be re-run without colliding with the end of the scenario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomResetResponse {
    /// War-room signature.
    pub signature: String,
    /// Number of cleared ticks.
    pub deleted_ticks: i64,
    /// Number of cleared boss directives.
    pub deleted_directives: i64,
    /// Tick pointer after reset — always zero.
    pub tick_cursor: i64,
    /// Progress percentage after reset — always zero.
    pub progress_percent: i64,
    /// Persian message.
    pub message_fa: String,
    /// English message.
    pub message_en: String,
}
