//! هندلرهای کوانتِ علّی — هستهٔ `ccxt` و موتور `Vibe-Trading`
//! (Wave 6 — Step 3/4).
//!
//! سه مسیر additive:
//! - `GET /quant/venues` — ماتریسِ ۸-مکانیِ بی‌کلید + خودآزمونِ تیکِ
//!   `Precise` (درمانِ #1 سیسی‌اکس‌تا) + نشانِ گپِ ترتیبِ `OrderBook`
//!   (درمانِ #2) — عمومی و قطعی؛
//! - `POST /quant/vibe-backtest` — DAG توپولوژیکِ ۵گرهی روی پرامپتِ
//!   طبیعیِ دوزبانه: مولدِ قطعیِ بازار → بک‌تسترِ اکیداً علّی
//!   (`&candles[0..=t]` → `open(t+1)`) → دفترِ ادعای عددی → گیتِ مایرون →
//!   ماندگاری در `quant_strategies`؛
//! - `GET /quant/strategies` — فهرستِ استراتژی‌هایِ ماندگار.

use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;

use kian_contracts::{
    QuantGateView, QuantMetricsView, QuantStrategyView, QuantStrategiesResponse,
    QuantSwarmNodeView, QuantTickSelftestView, QuantVibeBacktestRequest,
    QuantVibeBacktestResponse, QuantVenueEndpointView, QuantVenueView, QuantVenuesResponse,
};
use kian_db::now_iso;
use kian_hermes::vibe_trading_engine::{
    run_vibe_quant_swarm_dag, SWARM_DAG_DEFAULT_BARS, VIBE_QUANT_SIGNATURE,
};
use kian_wallet::ccxt_core::{
    keyless_public_market_matrix, orderbook_gap_guard_probe, precise_tick_selftest_rows,
    CCXT_CORE_SIGNATURE,
};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::require_session;
use crate::state::AppState;

/// سقفِ اندازهٔ پرامپتِ Vibe روی سیم — همان ۸ کیلوبایتِ استودیو.
const VIBE_PROMPT_MAX_BYTES: usize = 8_192;

/// نگاشتِ مکانِ بی‌کلید به نمای وایر.
#[must_use]
pub fn venue_view(
    v: &kian_wallet::ccxt_core::KeylessMarketVenue,
) -> QuantVenueView {
    QuantVenueView {
        id: v.id.to_owned(),
        display_name: v.display_name.to_owned(),
        venue_kind: v.venue_kind.to_owned(),
        base_url: v.base_url.to_owned(),
        auth: v.auth.to_owned(),
        endpoints: v
            .endpoints
            .iter()
            .map(|e| QuantVenueEndpointView {
                purpose: e.purpose.to_owned(),
                method: e.method.to_owned(),
                path: e.path.to_owned(),
                weight: e.weight,
            })
            .collect(),
        note: v.note.to_owned(),
    }
}

/// نگاشتِ ردیفِ خودآزمونِ تیک به نمای وایر.
#[must_use]
pub fn tick_selftest_view(
    r: &kian_wallet::ccxt_core::TickSelftestRow,
) -> QuantTickSelftestView {
    QuantTickSelftestView {
        input: r.input.to_owned(),
        tick: r.tick.to_owned(),
        output: r.output.clone(),
    }
}

/// نگاشتِ تلمتریِ گرهٔ DAG به نمای وایر.
#[must_use]
pub fn node_view(t: &kian_hermes::vibe_trading_engine::SwarmNodeTelemetry) -> QuantSwarmNodeView {
    QuantSwarmNodeView {
        index: u32::try_from(t.index).unwrap_or(5),
        name: t.name.clone(),
        status: t.status.clone(),
        summary: t.summary.clone(),
        deterministic_ticks: t.deterministic_ticks,
    }
}

/// نگاشتِ گزارشِ بک‌تست به نمایِ سنجه‌ها.
#[must_use]
pub fn metrics_view(
    r: &kian_hermes::vibe_trading_engine::CausalBacktestReport,
) -> QuantMetricsView {
    QuantMetricsView {
        causal_slice_proof: r.causal_slice_proof,
        sharpe_ratio: r.sharpe_ratio,
        sortino_ratio: r.sortino_ratio,
        max_drawdown_pct: r.max_drawdown_pct,
        total_return_pct: r.total_return_pct,
        cagr_pct: r.cagr_pct,
        calmar_ratio: r.calmar_ratio,
        win_rate_pct: r.win_rate_pct,
        profit_factor: r.profit_factor,
        omega_ratio: r.omega_ratio,
        tail_ratio: r.tail_ratio,
        skewness: r.skewness,
        kurtosis: r.kurtosis,
        turnover_ratio: r.turnover_ratio,
        total_trades: u32::try_from(r.total_trades).unwrap_or(u32::MAX),
        bars: u32::try_from(r.bars).unwrap_or(u32::MAX),
        final_equity: r.final_equity,
        liquidated: r.liquidated,
        kill_switch_tripped: r.kill_switch_tripped,
        claim_verification_hash: r.claim_verification_hash.clone(),
    }
}

/// `GET /quant/venues` — ماتریسِ بازارِ بی‌کلید + شواهدِ درمان‌ها (عمومی).
///
/// # Errors
/// رخ نمی‌دهد — خروجیِ قطعیِ کورهای خالص.
pub async fn venues() -> Json<QuantVenuesResponse> {
    let matrix = keyless_public_market_matrix();
    let count = matrix.len();
    Json(QuantVenuesResponse {
        venues: matrix.iter().map(venue_view).collect(),
        tick_selftest: precise_tick_selftest_rows()
            .iter()
            .map(tick_selftest_view)
            .collect(),
        orderbook_gap_guard: orderbook_gap_guard_probe().iter().map(|s| (*s).to_owned()).collect(),
        signature: format!("{CCXT_CORE_SIGNATURE}-venues-{count}"),
    })
}

/// `POST /quant/vibe-backtest` — اجرایِ کاملِ DAG کوانت (نشستِ شهروند).
///
/// خط لوله: مولدِ قطعیِ بازار → تجزیه‌گرِ دوزبانهٔ Vibe (صفر اجرای پویا)
/// → بک‌تسترِ اکیداً علّی → دفترِ ادعای عددی → گیتِ مایرون (+ گذرِ قطعیِ
/// تنظیمِ رژیم) → ماندگاری در `quant_strategies` → پاسخِ کامل + `SKILL.md`.
///
/// # Errors
/// ۴۰۱ نشستِ غایب؛ ۴۰۰ بدنهٔ نامعتبر / پرامپتِ خالی / طولِ بیش از سقف.
#[allow(clippy::too_many_lines)]
pub async fn vibe_backtest(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<QuantVibeBacktestResponse>> {
    let user = require_session(&state, &headers).await?;
    let req: QuantVibeBacktestRequest = parse_body(&body)?;

    let prompt = req.vibe_prompt.trim().to_owned();
    if prompt.is_empty() || prompt.len() > VIBE_PROMPT_MAX_BYTES {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }
    let symbol = req.symbol.clone().unwrap_or_else(|| "BTC-USDT".to_owned());
    let timeframe = req.timeframe.clone().unwrap_or_else(|| "1h".to_owned());
    let default_bars = u32::try_from(SWARM_DAG_DEFAULT_BARS).unwrap_or(720);
    let bars = u64::from(req.bars.unwrap_or(default_bars));
    let seed = req.seed.unwrap_or(2026);

    let dag = run_vibe_quant_swarm_dag(&prompt, &symbol, &timeframe, usize::try_from(bars).unwrap_or(SWARM_DAG_DEFAULT_BARS), seed);

    // ماندگاری — هر اجرا یک ردیف؛ پرچمِ گیتِ مایرون صادقانه ثبت می‌شود.
    let row_id = format!("quant:{}", uuid::Uuid::now_v7());
    let ast_json = serde_json::to_string(&dag.ast)
        .map_err(|_| ApiFailure::from(kian_contracts::HttpFailure::invalid_request(1)))?;
    let metrics_json = serde_json::to_string(&dag.backtest)
        .map_err(|_| ApiFailure::from(kian_contracts::HttpFailure::invalid_request(1)))?;
    let row = kian_db::quant::QuantStrategyRow {
        id: row_id.clone(),
        creator_user_id: user.id.clone(),
        name: dag.ast.name.clone(),
        symbol: dag.ast.symbol.clone(),
        timeframe: dag.ast.timeframe.clone(),
        vibe_prompt: prompt,
        strategy_ast_json: ast_json,
        metrics_json,
        sharpe_ratio: dag.backtest.sharpe_ratio,
        max_drawdown_pct: dag.backtest.max_drawdown_pct,
        mairon_approved: dag.gate.approved,
        claim_verification_hash: dag.backtest.claim_verification_hash.clone(),
        skill_md: dag.skill_md.clone(),
        created_at: now_iso(),
    };
    state.store.create_quant_strategy(&row).await?;

    let ast_wire = serde_json::to_value(&dag.ast)
        .map_err(|_| ApiFailure::from(kian_contracts::HttpFailure::invalid_request(1)))?;
    Ok(Json(QuantVibeBacktestResponse {
        strategy_id: dag.ast.strategy_id.clone(),
        strategy_ast_json: ast_wire,
        metrics: metrics_view(&dag.backtest),
        nodes: dag.nodes.iter().map(node_view).collect(),
        gate: QuantGateView {
            approved: dag.gate.approved,
            reason: dag.gate.reason.clone(),
            regime_boost_applied: dag.gate.regime_boost_applied,
        },
        skill_md: dag.skill_md.clone(),
        persisted_id: row_id,
        signature: VIBE_QUANT_SIGNATURE.to_owned(),
    }))
}

/// نگاشتِ ردیفِ ماندگار به نمای وایر — شناسهٔ استراتژی از AST JSON.
#[must_use]
pub fn strategy_view(row: &kian_db::quant::QuantStrategyRow) -> QuantStrategyView {
    let strategy_id = serde_json::from_str::<serde_json::Value>(&row.strategy_ast_json)
        .ok()
        .and_then(|v| v.get("strategy_id").and_then(serde_json::Value::as_str).map(str::to_owned))
        .unwrap_or_default();
    QuantStrategyView {
        id: row.id.clone(),
        strategy_id,
        creator_user_id: row.creator_user_id.clone(),
        name: row.name.clone(),
        symbol: row.symbol.clone(),
        timeframe: row.timeframe.clone(),
        sharpe_ratio: row.sharpe_ratio,
        max_drawdown_pct: row.max_drawdown_pct,
        mairon_approved: row.mairon_approved,
        claim_verification_hash: row.claim_verification_hash.clone(),
        created_at: row.created_at.clone(),
    }
}

/// `GET /quant/strategies` — فهرستِ استراتژی‌هایِ ماندگار (عمومی).
///
/// # Errors
/// ۵۰۰ خطای SQL.
pub async fn strategies(State(state): State<Arc<AppState>>) -> ApiResult<Json<QuantStrategiesResponse>> {
    let rows = state.store.list_quant_strategies(100).await?;
    let count = rows.len();
    Ok(Json(QuantStrategiesResponse {
        strategies: rows.iter().map(strategy_view).collect(),
        count: u32::try_from(count).unwrap_or(u32::MAX),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn venue_mapping_preserves_all_fields() {
        let matrix = keyless_public_market_matrix();
        assert_eq!(matrix.len(), 8);
        let hl = matrix.iter().find(|v| v.id == "hyperliquid-dex").unwrap();
        let view = venue_view(hl);
        assert_eq!(view.endpoints.len(), 4);
        assert_eq!(view.endpoints[0].weight, 2);
        assert_eq!(view.endpoints[2].weight, 4);
        assert_eq!(view.auth, "none");
    }

    #[test]
    fn tick_selftest_and_gap_guard_reach_the_wire() {
        let rows = precise_tick_selftest_rows();
        let views: Vec<QuantTickSelftestView> = rows.iter().map(tick_selftest_view).collect();
        assert_eq!(views.len(), 5);
        assert_eq!(views[0].output, "0.2857");
        let guard = orderbook_gap_guard_probe();
        assert_eq!(guard, ["Applied", "IgnoredStale", "SequenceGapDetected"]);
    }

    #[test]
    fn metrics_view_carries_claim_hash() {
        use kian_hermes::vibe_trading_engine::{
            compile_vibe_to_strategy_ast, CausalBacktestEngine,
        };
        let ast = compile_vibe_to_strategy_ast("buy rsi below 45", "BTC-USDT", "1h");
        let candles = kian_hermes::vibe_trading_engine::generate_deterministic_ohlcv("BTC-USDT", 360, 7);
        let report = CausalBacktestEngine::default().run(&ast, &candles);
        let view = metrics_view(&report);
        assert!(view.causal_slice_proof);
        assert_eq!(view.claim_verification_hash, report.claim_verification_hash);
    }
}
