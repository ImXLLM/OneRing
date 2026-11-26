# Creator-Brain DNA 15: The `ccxt` Quant Core, the Causal `Vibe-Trading` Quant Engine, and the Sovereign Mairon Gate

> `KIA-BRAIN-OS/15_CCXT_AND_VIBE_TRADING_QUANT_ENGINE.md` — Wave 6 (Step 3/4) of the `0xKIA` kernel
> Execution tag: `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]`
> Engines: `crates/wallet/src/ccxt_core.rs` · `crates/hermes/src/vibe_trading_engine.rs`
> Migration: `0010_quant_vibe_trading_and_ccxt_feeds.sql` · Wire: `GET/POST /api/v1/quant/*`

---

## 1. Executive Summary

DNA 15 mounts two deterministic cores onto the kernel and seals both behind the Mairon gate:

1. **A pure-Rust `ccxt/ccxt` core** (`kian-wallet::ccxt_core`) — `Precise` with an `i128`
   factor and the compile-time power table `POW10[38]`, `decimal_to_precision` with three
   counting modes, the dual-band `CcxtThrottler` limiter (leaky bucket + `VecDeque` sliding
   window at $O(1)$), an L2 order book with binary search, the defensive `SafeExtractor`,
   and error classification with `is_retryable`/exponential backoff — plus the **complete
   8-venue keyless, zero-KYC vertical matrix**.
2. **The `HKUDS/Vibe-Trading` causal quant engine** (`kian_hermes::vibe_trading_engine`) —
   a typed, Turing-incomplete AST, a bilingual Persian/English parser, a strictly causal
   backtester with a `+1 candle` fill, full institutional metrics, a numeric claim ledger
   locked with `SHA-256`, a risk-guard hard kill switch, a 5-node topological DAG, and the
   Mairon gate with deterministic regime-tuning passes.

The kernel contract is preserved: zero `unsafe`, zero `Clippy` warnings (`-D warnings`),
full determinism (same prompt and seed → same AST, same report, same hash), and **100%
float-free decimal arithmetic on price/amount** — floats live only at pure statistical
boundaries (`bps`, `VWAP`, metrics).

## 2. The `ccxt` Core — Two Adjudicated Fixes

### Fix #1 — 100% `Precise` Price Ticks (`Round` too!)

In the reference `number.ts:7–243`, the `TICK_SIZE` mode with `ROUND` falls back to the
floating-point `%` path and suffers `IEEE-754` drift on ticks such as `0.0001` and `0.05`.
In the Rust port, `TickSize` is applied for **both** the `Round`/`Truncate` modes with pure
integer division/remainder: `q = val / tick`, round `q` by the exact comparison
`|rem| ≥ |tick| − |rem|`, then `q_rounded × tick`.
The wired self-test (`GET /quant/venues`) returns five deterministic witness rows:

| Input | Tick | Output |
| --- | --- | --- |
| `0.2857` | `0.0001` | `0.2857` |
| `0.15` | `0.05` | `0.15` |
| `0.125` | `0.05` | `0.15` (round-half-away-from-zero) |
| `1.0049` | `0.05` | `1` |
| `123.456` | `0.0001` | `123.456` |

### Fix #2 — The `OrderBook` Sequence-Gap Guard

The reference `OrderBook.ts:96–136` drops stale deltas (`nonce ≤ current`) but **silently**
accepts gaps (`incoming > current + 1`), corrupting the L2 book. The Rust `apply_delta` is
tri-state: `IgnoredStale` / `SequenceGapDetected { expected, received }` (with no level
mutation at all) / `Applied { nonce }` — the last is the signal for an immediate
REST-snapshot resync.
The wired probe `[Applied, IgnoredStale, SequenceGapDetected]` is always green.

### The Keyless Market Matrix (8 Venues — Arena-Verified)

| Identifier | Kind | Endpoints |
| --- | --- | --- |
| `binance-spot` | cex-spot | `/api/v3/ticker/24hr` · `/depth` · `/klines` |
| `binance-linear-usdtm` | cex-linear-perp | `/fapi/v1/premiumIndex` · `/fundingRate` · `/openInterest` |
| `bybit-v5` | cex-linear-perp | `/v5/market/tickers` · `/orderbook` · `/kline` · `/funding/history` |
| `okx-v5` | cex-linear-perp | `/api/v5/market/tickers` · `/books` · `/candles` · `/public/funding-rate` |
| `kraken-spot` | cex-spot | `/0/public/Ticker` · `/Depth` · `/OHLC` |
| `coinbase-exchange` | cex-spot | `/products/{id}/ticker` · `/book?level=2` · `/candles` |
| `hyperliquid-dex` | dex-perp | `POST /info` with `allMids`(2) · `l2Book`(2) · `candleSnapshot`(4) · `metaAndAssetCtxs` |
| `base-mainnet-dex` | dex-amm | Keyless RPC on Chain ID `8453` + pool quotes |

## 3. The Four Adjudicated Fixes of `Vibe-Trading`

1. **A Turing-incomplete `StrategyAst`** — replaces `importlib`/arbitrary Python execution
   (`runner.py:L1541–1568`). Every strategy is a serializable AST built from `IndicatorSpec`
   (`Sma/Ema/Rsi/BollingerBands/Atr/ZScore/Momentum`) and `SignalRule`
   (`Above/Below/CrossesAbove/CrossesBelow/PriceAboveSma/PriceBelowSma/AnyOf`) —
   **zero dynamic execution, zero `eval`**.
2. **A strictly causal backtester** — the evaluator receives only `&candles[0..=t]` (seeing
   the future is out of reach of the signature); a signal on `close(t)` fills at `open(t+1)`
   (`base.py:L343–348`) with a contractual friction floor (fee ≥ 10 bps, slippage ≥ 5 bps),
   buy sizing via a 32-step binary search (`qty·fill + fee ≤ cash`), a 10% maintenance
   margin, and a zeroed final snapshot at `timestamp + 1ms`.
3. **`VerifiedNumericClaimLedger`** — the report's five numeric claims (Sharpe, drawdown,
   return, win rate, turnover) are locked into `SHA-256` in canonical `%.9f` form;
   `verify_numeric_claims` burns any tampering — replacing the textual
   anti-hallucination rules (`context.py:L35–74`).
4. **A `PortfolioRiskGuard` hard kill switch** — leverage ≤ 5.0, order ≤ 25% of equity,
   daily loss ≤ 8%, a 20% drawdown kill switch — enforced pre-order and at runtime,
   fail-closed.

## 4. The Bilingual Vibe Parser

Persian/Arabic digits are normalized to Latin digits (e.g. the Persian numeral for 30
becomes `30`), the half-space (ZWNJ) becomes a space, and a **direction-aware** mapping is
applied: for longs, "below the threshold" is entry and "above the threshold" is exit; for
shorts, the reverse. Both reference prompts compile correctly:

- *"Buy when RSI comes under 30 and the trend is bullish, sell above 70"* → entry
  `Rsi(14)<30` + the `PriceAboveSma(50)` trend filter; exit `Rsi(14)>70`.
- *"Momentum breakout with EMA 12/26 crossover and RSI filter on BTC-USDT"* → entry
  `Momentum(10)>0` ∧ the golden `EMA12>EMA26` cross ∧ `Rsi<70`; a mirrored exit.

Stop-loss/take-profit/leverage also come from the prompt (`stop loss 1.5%` → `150bps`,
`leverage 4` → `4.0` capped by the guard's 5.0 ceiling); a signal-less prompt falls back to
the canonical `EMA 12/26 + RSI<70` strategy so the DAG always yields a valid AST.

## 5. The 5-Node Topological DAG and the Mairon Gate

`1. MarketDataScout → 2. FactorFeatureMiner → 3. StrategyAstSynthesizer →
4. CausalBacktestVerifier → 5. MaironQuantRiskJudge` — with a deterministic AR(1)-around-trend
market generator (a 0.30 pull toward `trend × band ±5%`, three-uniform Irwin-Hall noise; zero
network, full replay). The Mairon gate turns green only when **all five** conditions hold:
`sharpe ≥ 1.20` ∧ `|maxDD| ≤ 20%` ∧ `trades ≥ 5` ∧ `causal_slice_proof` ∧
`claim_ledger_verified` — and on failure, up to four deterministic regime-tuning variants
(thresholds ±5, removal of a redundant rule, a later stop, trend confirmation) are tried;
`regime_boost_applied` surfaces truthfully over the wire.

## 6. Persistence and Wire

The migration `0010_quant_vibe_trading_and_ccxt_feeds.sql` creates the `quant_strategies`
table (`id`, `creator_user_id`, `name`, `symbol`, `timeframe`, `vibe_prompt`,
`strategy_ast_json`, `metrics_json`, `sharpe_ratio`, `max_drawdown_pct`,
`mairon_approved`, `claim_verification_hash`, `skill_md`, `created_at` with `strftime` +
two creator/symbol indexes) — **no ledger table is touched**, and every invariant
(User=466, Product=20, Posting SUM=0) stays intact. Wire:

| Path | Contract | Guard |
| --- | --- | --- |
| `GET /api/v1/quant/venues` | `QuantVenuesResponse` — 8 venues + tick self-test + gap badge | public |
| `POST /api/v1/quant/vibe-backtest` | `QuantVibeBacktestResponse` — AST + metrics + 5 nodes + gate + `SKILL.md` | citizen session |
| `GET /api/v1/quant/strategies` | `QuantStrategiesResponse` — persisted rows, newest first | public |

## 7. Tests and the Execution Seal

44 new tests (29 for the `ccxt` core + 13 for the quant engine + 2 for persistence + 3 for
handler wire) on top of the Wave 5 suite of 607 tests — **zero `Clippy` warnings**,
`KbEntry` = **330** with the idempotent `kb:dna-15-ccxt-vibe-quant` injection.

`[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]`
