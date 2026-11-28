//! Pillar 1 — property-based tests for the Vibe-Trading causal quant engine (proptest).
//!
//! Deterministic properties (every case checked over randomly generated,
//! reproducible inputs):
//!
//! 1. **Causal backtest slice isolation** — the equity curve the engine records
//!    over the prefix `&candles[0..=cut]` is *identical* to the equity curve of
//!    the full run restricted to the same window: the strategy can never see
//!    `t+1..` (the borrowed `&candles[0..=t]` slice is the structural proof).
//! 2. **SHA-256 claims-ledger tamper evidence** — every report verifies against
//!    its own claim hash, and any tampered metric invalidates the hash.

use kian_hermes::vibe_trading_engine::{
    compile_vibe_to_strategy_ast, generate_deterministic_ohlcv, verify_numeric_claims,
    BacktestConfig, CausalBacktestEngine, PortfolioRiskGuard,
};
use proptest::prelude::*;

const REFERENCE_PROMPT: &str = "Momentum breakout with EMA 12/26 crossover and RSI filter on BTC-USDT";

fn engine() -> CausalBacktestEngine {
    CausalBacktestEngine {
        config: BacktestConfig::default(),
        guard: PortfolioRiskGuard::default(),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property 1 — slice isolation: running the engine on the full series and
    /// on a truncated prefix produces byte-identical equity over the shared
    /// window; the future bars cannot leak into the prefix result.
    #[test]
    fn causal_backtest_prefix_curve_is_slice_isolated(
        seed in 0u64..=100_000,
        extra_bars in 8usize..=40,
    ) {
        let ast = compile_vibe_to_strategy_ast(REFERENCE_PROMPT, "BTC-USDT", "1h");
        let warmup = ast.warmup_bars;
        // n guarantees a full warmup in both runs and a fill bar at `cut+1`.
        let n = warmup + 18 + extra_bars;
        let cut = warmup + 3 + (extra_bars % 8); // cut + 1 <= n - 1 always
        let candles = generate_deterministic_ohlcv("BTC-USDT", n, seed);
        prop_assert!(cut < n.saturating_sub(1), "the cut must keep a fill bar in the full run");

        let full = engine().run(&ast, &candles);
        let prefix = engine().run(&ast, &candles[0..=cut]);
        prop_assert!(full.causal_slice_proof, "the full run must carry the causal slice proof");
        prop_assert!(prefix.causal_slice_proof, "the prefix run must carry the causal slice proof");

        let k = full.equity_curve.len().min(prefix.equity_curve.len());
        prop_assert!(k >= 1, "at least one causal step must be shared");
        for i in 0..k {
            let a = full.equity_curve[i];
            let b = prefix.equity_curve[i];
            prop_assert!(
                (a - b).abs() <= 1e-9 * a.abs().max(1.0),
                "equity divergence at shared index {i}: full={a} prefix={b} — the future leaked into the past"
            );
        }
    }

    /// Property 2 — the claims ledger seals every generated report and rejects
    /// any tampered metric (SHA-256 anti-tampering lock).
    #[test]
    fn claims_ledger_is_tamper_evident(seed in 0u64..=100_000) {
        let ast = compile_vibe_to_strategy_ast(REFERENCE_PROMPT, "ETH-USDT", "1d");
        let candles = generate_deterministic_ohlcv("ETH-USDT", ast.warmup_bars + 24, seed);
        let report = engine().run(&ast, &candles);
        prop_assert!(
            verify_numeric_claims(&report),
            "a freshly forged report must verify against its own claim hash"
        );

        // Tamper one metric — the hash must invalidate.
        let mut tampered = report.clone();
        tampered.total_return_pct += 0.5;
        prop_assert!(
            !verify_numeric_claims(&tampered),
            "a tampered report must fail the claim-hash verification"
        );
    }
}
