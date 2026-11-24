# DNA پانزدهم مغز خالق: هستهٔ کوانتِ `ccxt`، موتورِ کوانتِ علّیِ `Vibe-Trading` و گیتِ حاکمِ مایرون

> `KIA-BRAIN-OS/15_CCXT_AND_VIBE_TRADING_QUANT_ENGINE.md` — Wave 6 (Step 3/4) کرنل `0xKIA`
> برچسب اجرایی: `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]`
> موتورها: `crates/wallet/src/ccxt_core.rs` · `crates/hermes/src/vibe_trading_engine.rs`
> مهاجرت: `0010_quant_vibe_trading_and_ccxt_feeds.sql` · سیم: `GET/POST /api/v1/quant/*`

---

## ۱. چکیدهٔ اجرایی

DNA پانزدهم دو کورِ قطعی را به کرنل سوار می‌کند و هر دو را با گیتِ مایرون مهر می‌زند:

1. **هستهٔ خالص-راستِ `ccxt/ccxt`** (`kian-wallet::ccxt_core`) — `Precise` با ضریبِ
   `i128` و جدولِ توانِ کامپایل-زمانِ `POW10[38]`، `decimal_to_precision` با سه حالتِ
   شمارش، محدودسازِ دوگانهٔ `CcxtThrottler` (سطلِ سوراخ + پنجرهٔ لغزانِ `VecDeque` با
   $O(1)$)، کتابِ سفارشِ L2 با جست‌وجوی دودویی، استخراج‌گرِ دفاعیِ `SafeExtractor`
   و رده‌بندیِ خطا با `is_retryable`/backoff نمایی — به‌همراه **ماتریسِ کاملِ ۸
   مکانِ عمودیِ صفر-کلیدِ صفر-KYC**.
2. **موتورِ کوانتِ علّیِ `HKUDS/Vibe-Trading`** (`kian_hermes::vibe_trading_engine`) —
   AST تایپ‌دارِ ناتورینگ-ناقص، تجزیه‌گرِ دوزبانهٔ فارسی/انگلیسی، بک‌تسترِ اکیداً
   علّی با پرِ `+۱ کندل`، سنجه‌هایِ نهادیِ کامل، دفترِ ادعای عددیِ قفل‌شده با
   `SHA-256`، کلیدِ توقفِ سختِ گاردِ ریسک، DAG توپولوژیکِ ۵گرهی و گیتِ مایرون با
   گذرِ قطعیِ تنظیمِ رژیم.

قرارداد کرنل حفظ شده: صفر `unsafe`، صفر هشدارِ `Clippy` (`-D warnings`)، قطعیتِ کامل
(همان پرامپت و بذر → همان AST، همان گزارش، همان هش) و **صددرصد دهدهیِ بدونِ شناور روی
قیمت/مبلغ** — تنها شناورها در مرزهای آماریِ خالص (`bps`، `VWAP`، سنجه‌ها) زندگی می‌کنند.

## ۲. هستهٔ `ccxt` — دو درمانِ قضایی

### درمانِ #1 — تیکِ قیمتِ صددرصدِ `Precise` (`Round` هم!)

در مرجعِ `number.ts:7–243`، حالتِ `TICK_SIZE` با `ROUND` به فال‌بکِ شناورِ `%` می‌افتد و
روی تیک‌هایی مثل `0.0001` و `0.05` ریزشِ `IEEE-754` می‌دهد. در پورتِ راست، `TickSize`
برای **هر دو** حالتِ `Round`/`Truncate` با تقسیم/باقیماندهٔ صددرصدِ صحیح انجام می‌شود:
`q = val / tick`، گردِ `q` با مقایسهٔ دقیقِ `|rem| ≥ |tick| − |rem|`، سپس `q_rounded × tick`.
خودآزمونِ سیمی (`GET /quant/venues`) پنج ردیفِ شاهدِ قطعی برمی‌گرداند:

| ورودی | تیک | خروجی |
| --- | --- | --- |
| `0.2857` | `0.0001` | `0.2857` |
| `0.15` | `0.05` | `0.15` |
| `0.125` | `0.05` | `0.15` (نیم-دور-از-صفر) |
| `1.0049` | `0.05` | `1` |
| `123.456` | `0.0001` | `123.456` |

### درمانِ #2 — نگهبانِ گپِ ترتیبِ `OrderBook`

مرجعِ `OrderBook.ts:96–136` دلتایِ کهنه (`nonce ≤ current`) را می‌اندازد اما گپ
(`incoming > current + 1`) را **بی‌صدا** می‌پذیرد و کتابِ L2 را فاسد می‌کند. پورتِ راست
`apply_delta` سه‌وضعیتی است: `IgnoredStale` / `SequenceGapDetected { expected, received }`
(بدونِ هیچ جهشِ لِیول) / `Applied { nonce }` — دومی سیگنالِ resync فوریِ REST snapshot است.
پروبِ سیمی `[Applied, IgnoredStale, SequenceGapDetected]` همیشه سبز است.

### ماتریسِ بازارِ بی‌کلید (۸ مکان — تأییدشدهٔ Arena)

| شناسه | گونه | نقاطِ پایانی |
| --- | --- | --- |
| `binance-spot` | cex-spot | `/api/v3/ticker/24hr` · `/depth` · `/klines` |
| `binance-linear-usdtm` | cex-linear-perp | `/fapi/v1/premiumIndex` · `/fundingRate` · `/openInterest` |
| `bybit-v5` | cex-linear-perp | `/v5/market/tickers` · `/orderbook` · `/kline` · `/funding/history` |
| `okx-v5` | cex-linear-perp | `/api/v5/market/tickers` · `/books` · `/candles` · `/public/funding-rate` |
| `kraken-spot` | cex-spot | `/0/public/Ticker` · `/Depth` · `/OHLC` |
| `coinbase-exchange` | cex-spot | `/products/{id}/ticker` · `/book?level=2` · `/candles` |
| `hyperliquid-dex` | dex-perp | `POST /info` با `allMids`(2) · `l2Book`(2) · `candleSnapshot`(4) · `metaAndAssetCtxs` |
| `base-mainnet-dex` | dex-amm | RPC کلید-کمترِ Chain ID `8453` + کوتیشنِ استخر |

## ۳. چهار درمانِ قضاییِ `Vibe-Trading`

1. **`StrategyAst` ناتورینگ-ناقص** — جایگزینِ `importlib`/اجرایِ پایتونِ دلخواه
   (`runner.py:L1541–1568`). هر استراتژی یک AST سری‌شدنی از `IndicatorSpec`
   (`Sma/Ema/Rsi/BollingerBands/Atr/ZScore/Momentum`) و `SignalRule`
   (`Above/Below/CrossesAbove/CrossesBelow/PriceAboveSma/PriceBelowSma/AnyOf`) است —
   **صفر اجرای پویا، صفر `eval`**.
2. **بک‌تسترِ اکیداً علّی** — ارزیاب فقط `&candles[0..=t]` می‌گیرد (دیدنِ آینده از امضا
   بیرونِ دست است)؛ سیگنال روی `close(t)` در `open(t+1)` پر می‌شود (`base.py:L343–348`)
   با کفِ اصطکاکِ قراردادی (کارمزد ≥ ۱۰bps، احتراک ≥ ۵bps)، اندازه‌گیریِ خرید با
   جست‌وجوی دودوییِ ۳۲قدمی (`qty·fill + fee ≤ cash`)، هامشِ نگهداریِ ۱۰٪ و اسنپ‌شاتِ
   پایانیِ صفرشدهٔ `timestamp + 1ms`.
3. **`VerifiedNumericClaimLedger`** — پنج ادعایِ عددیِ گزارش (شارپ، دراودان، بازده،
   نرخِ برد، گردش) در قالبِ کانونیکِ `%.9f` به `SHA-256` قفل می‌شوند؛
   `verify_numeric_claims` هر دستکاری را می‌سوزاند — جایگزینِ قواعدِ متنیِ
   ضدهالوسیناسیون (`context.py:L35–74`).
4. **`PortfolioRiskGuard` کلیدِ توقفِ سخت** — اهرم ≤ ۵٫۰، سفارش ≤ ۲۵٪ حقوق، زیانِ
   روزانه ≤ ۸٪، کلیدِ توقفِ دراودانِ ۲۰٪ — پیش از سفارش و در runtime، fail-closed.

## ۴. تجزیه‌گرِ دوزبانهٔ Vibe

ارقام فارسی/عربی نرمال می‌شوند (`۳۰ → 30`)، نیم‌فاصله فضای می‌شود و نگاشتِ **جهت‌محور**
اعمال می‌گردد: برای لانگ «زیر آستانه» ورود و «بالای آستانه» خروج است؛ برای شورت برعکس.
هر دو الگویِ مرجع درست تدوین می‌شوند:

- *«وقتی RSI زیر ۳۰ آمد و روند صعودی بود بخر و بالای ۷۰ بفروش»* → ورود `Rsi(14)<30` +
  فیلترِ روندِ `PriceAboveSma(50)`؛ خروج `Rsi(14)>70`.
- *«Momentum breakout with EMA 12/26 crossover and RSI filter on BTC-USDT»* → ورود
  `Momentum(10)>0` ∧ طلاییِ `EMA12>EMA26` ∧ `Rsi<70`؛ خروجِ قرینه.

حدِ ضرر/سود/اهرم هم از پرامپت می‌آید (`حد ضرر ۱٫۵٪` → `150bps`، `اهرم ۴` → `4.0` با سقفِ
گاردِ ۵٫۰)؛ پرامپتِ بی-سیگنال به استراتژیِ کانونیکِ `EMA 12/26 + RSI<70` می‌افتد تا DAG
همیشه AST معتبر بدهد.

## ۵. DAG توپولوژیکِ ۵گرهی و گیتِ مایرون

`1. MarketDataScout → 2. FactorFeatureMiner → 3. StrategyAstSynthesizer →
4. CausalBacktestVerifier → 5. MaironQuantRiskJudge` — با مولدِ بازارِ قطعیِ
AR(1)-دور-روند (کششِ ۰٫۳۰ به `trend × باندِ ±۵٪`، نویزِ Irwin-Hall سه‌یکنواخت؛ صفر
شبکه، بازپخشِ کامل). گیتِ مایرون فقط با **هر پنج** شرط سبز می‌شود:
`sharpe ≥ 1.20` ∧ `|maxDD| ≤ 20%` ∧ `trades ≥ 5` ∧ `causal_slice_proof` ∧
`claim_ledger_verified` — و در شکست، تا چهار واریانتِ قطعیِ تنظیمِ رژیم
(آستانه‌ها ±۵، حذفِ قاعدهٔ زائد، توقفِ دیرتر، تأییدِ روند) امتحان می‌شوند؛
`regime_boost_applied` صادقانه روی سیم می‌آید.

## ۶. ماندگاری و سیم

مهاجرت `0010_quant_vibe_trading_and_ccxt_feeds.sql` جدولِ `quant_strategies` را می‌سازد
(`id`, `creator_user_id`, `name`, `symbol`, `timeframe`, `vibe_prompt`,
`strategy_ast_json`, `metrics_json`, `sharpe_ratio`, `max_drawdown_pct`,
`mairon_approved`, `claim_verification_hash`, `skill_md`, `created_at` با `strftime` +
دو نمایهٔ سازنده/نماد) — **هیچ جدولِ دفتری لمس نمی‌شود** و همهٔ ناورنی‌ها (User=466،
Product=20، Posting SUM=0) دست‌نخورده می‌مانند. سیم:

| مسیر | قرارداد | گارد |
| --- | --- | --- |
| `GET /api/v1/quant/venues` | `QuantVenuesResponse` — ۸ مکان + خودآزمونِ تیک + نشانِ گپ | عمومی |
| `POST /api/v1/quant/vibe-backtest` | `QuantVibeBacktestResponse` — AST + سنجه‌ها + ۵ گره + گیت + `SKILL.md` | نشستِ شهروند |
| `GET /api/v1/quant/strategies` | `QuantStrategiesResponse` — ماندگارها، نوترینِ اول | عمومی |

## ۷. آزمون‌ها و مُهرِ اجرا

۴۴ آزمونِ تازه (۲۹ هستهٔ `ccxt` + ۱۳ موتورِ کوانت + ۲ ماندگاری + ۳ سیمِ هندلر) روی
بدنهٔ ۶۰۷ آزمونیِ Wave 5 — **صفر هشدارِ `Clippy`**، `KbEntry` = **۳۳۰** با تزریقِ
ایدمپوتنتِ `kb:dna-15-ccxt-vibe-quant`.

`[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]`
