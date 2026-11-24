//! قراردادهای کوانتِ علّی — هستهٔ `ccxt` و موتور `Vibe-Trading` (Wave 6 —
//! Step 3/4).
//!
//! سه DTO روی وایر:
//! - [`QuantVenuesResponse`] — ماتریسِ ۸-مکانیِ بی‌کلید + خودآزمونِ تیکِ
//!   `Precise` (درمانِ قضاییِ #1 سیسی‌اکس‌تا) + نشانِ گپِ ترتیبِ `OrderBook`
//!   (درمانِ #2)؛
//! - [`QuantVibeBacktestRequest`] / [`QuantVibeBacktestResponse`] — اجرایِ
//!   DAG توپولوژیکِ ۵گرهی روی پرامپتِ طبیعیِ دوزبانه با بک‌تسترِ اکیداً
//!   علّی، دفترِ ادعاها، گیتِ مایرون و فورجِ `SKILL.md`؛
//! - [`QuantStrategiesResponse`] — فهرستِ استراتژی‌هایِ ماندگارِ
//!   `quant_strategies`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// یک نقطهٔ پایانیِ عمودی روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVenueEndpointView {
    /// هدفِ کاربردی (`ticker24h`, `depth`, …).
    pub purpose: String,
    /// متدِ HTTP.
    pub method: String,
    /// مسیر یا بدنهٔ نمونه.
    pub path: String,
    /// وزنِ نرخ‌سنجیِ اعلامی (۰ = اعلام‌نشده).
    pub weight: u32,
}

/// یک مکانِ بی‌کلید روی وایر — صفر API key، صفر KYC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVenueView {
    /// شناسهٔ پایدار (`binance-spot`, …).
    pub id: String,
    /// نام نمایشی.
    pub display_name: String,
    /// گونهٔ مکان (`cex-spot` | `cex-linear-perp` | `dex-perp` | `dex-amm`).
    pub venue_kind: String,
    /// پایهٔ URL.
    pub base_url: String,
    /// احرازِ لازم — همیشه `none`.
    pub auth: String,
    /// نقاطِ پایانی.
    pub endpoints: Vec<QuantVenueEndpointView>,
    /// یادداشتِ کوتاه.
    pub note: String,
}

/// یک ردیفِ خودآزمونِ تیکِ `Precise` — ورودی/تیک/خروجِ کانونیک.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantTickSelftestView {
    /// ورودی خام.
    pub input: String,
    /// تیک خام.
    pub tick: String,
    /// خروجی کانونیک.
    pub output: String,
}

/// `GET /quant/venues` — ماتریسِ بازارِ بی‌کلید + شواهدِ دو درمانِ قضایی.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVenuesResponse {
    /// ۸ مکانِ عمودیِ صفر-کلید.
    pub venues: Vec<QuantVenueView>,
    /// خودآزمونِ تیک — صددرصد `Precise`، صفر ریزشِ شناور.
    pub tick_selftest: Vec<QuantTickSelftestView>,
    /// نشانِ گپِ ترتیب — `[Applied, IgnoredStale, SequenceGapDetected]`.
    pub orderbook_gap_guard: Vec<String>,
    /// امضای کور.
    pub signature: String,
}

/// `POST /quant/vibe-backtest` — درخواستِ اجرایِ DAG کوانت.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVibeBacktestRequest {
    /// پرامپتِ طبیعی — فارسی یا انگلیسی.
    pub vibe_prompt: String,
    /// نماد — پیش‌فرض `BTC-USDT`.
    pub symbol: Option<String>,
    /// تایم‌فریم — پیش‌فرض `1h`.
    pub timeframe: Option<String>,
    /// تعدادِ کندل‌هایِ مولدِ قطعی — پیش‌فرض ۷۲۰.
    pub bars: Option<u32>,
    /// بذرِ قطعی — پیش‌فرض ۲۰۲۶.
    pub seed: Option<u64>,
}

/// تلمتریِ یک گرهٔ DAG روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantSwarmNodeView {
    /// اندیسِ ۱-پایه.
    pub index: u32,
    /// نامِ کانونیکِ گره.
    pub name: String,
    /// وضعیت.
    pub status: String,
    /// خلاصهٔ قطعی.
    pub summary: String,
    /// تیکِ قطعی.
    pub deterministic_ticks: u64,
}

/// سنجه‌هایِ نهادیِ گزارشِ بک‌تست روی وایر.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantMetricsView {
    /// اثباتِ علّی — همیشه `true`.
    pub causal_slice_proof: bool,
    /// شارپِ سالانه‌شده.
    pub sharpe_ratio: f64,
    /// سورتینو.
    pub sortino_ratio: f64,
    /// بیشینهٔ دراودان (درصد، منفی).
    pub max_drawdown_pct: f64,
    /// بازدهیِ کل (درصد).
    pub total_return_pct: f64,
    /// بازدهیِ سالانه‌شدهٔ مرکب (درصد).
    pub cagr_pct: f64,
    /// کالمر.
    pub calmar_ratio: f64,
    /// نرخِ برد (درصد).
    pub win_rate_pct: f64,
    /// نسبتِ سود.
    pub profit_factor: f64,
    /// امگا.
    pub omega_ratio: f64,
    /// نسبتِ دُم.
    pub tail_ratio: f64,
    /// چولگی.
    pub skewness: f64,
    /// کشیدگیِ اضافی.
    pub kurtosis: f64,
    /// گردشِ سرمایه.
    pub turnover_ratio: f64,
    /// تعدادِ معامله‌ها.
    pub total_trades: u32,
    /// تعدادِ کندل‌ها.
    pub bars: u32,
    /// حقوقِ پایانی.
    pub final_equity: f64,
    /// تسویهٔ اجباری؟
    pub liquidated: bool,
    /// کلیدِ توقف؟
    pub kill_switch_tripped: bool,
    /// هشِ دفترِ ادعاها.
    pub claim_verification_hash: String,
}

/// واپسِ گیتِ مایرون روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantGateView {
    /// تأییدِ فروش در مارکت‌پلیس.
    pub approved: bool,
    /// دلیلِ صادقانه.
    pub reason: String,
    /// گذرِ قطعیِ تنظیمِ رژیم به کار رفت؟
    pub regime_boost_applied: bool,
}

/// `POST /quant/vibe-backtest` — پاسخِ کاملِ DAG.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantVibeBacktestResponse {
    /// شناسهٔ استراتژی (`vibe:<hash>`).
    pub strategy_id: String,
    /// AST تایپ‌دار — گردشِ کاملِ JSON.
    pub strategy_ast_json: serde_json::Value,
    /// سنجه‌هایِ نهادی.
    pub metrics: QuantMetricsView,
    /// تلمتریِ ۵ گره به ترتیبِ توپولوژیک.
    pub nodes: Vec<QuantSwarmNodeView>,
    /// واپسِ گیتِ مایرون.
    pub gate: QuantGateView,
    /// `SKILL.md` فورج‌شده.
    pub skill_md: String,
    /// شناسهٔ ردیفِ ماندگار (`quant:<uuid>`) — پس از درج.
    pub persisted_id: String,
    /// امضای موتور کوانتِ علّی.
    pub signature: String,
}

/// یک استراتژیِ ماندگار روی وایر.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantStrategyView {
    /// شناسهٔ ردیف (`quant:<uuid>`).
    pub id: String,
    /// شناسهٔ استراتژی (`vibe:<hash>`).
    pub strategy_id: String,
    /// سازنده.
    pub creator_user_id: String,
    /// نامِ انسانی.
    pub name: String,
    /// نماد.
    pub symbol: String,
    /// تایم‌فریم.
    pub timeframe: String,
    /// شارپ.
    pub sharpe_ratio: f64,
    /// دراودان.
    pub max_drawdown_pct: f64,
    /// مُهرِ گیتِ مایرون.
    pub mairon_approved: bool,
    /// هشِ ادعاها.
    pub claim_verification_hash: String,
    /// مُهرِ زمانِ تولید.
    pub created_at: String,
}

/// `GET /quant/strategies` — فهرستِ استراتژی‌هایِ ماندگار.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantStrategiesResponse {
    /// ردیف‌ها به ترتیبِ نوترین.
    pub strategies: Vec<QuantStrategyView>,
    /// تعدادِ کل.
    pub count: u32,
}
