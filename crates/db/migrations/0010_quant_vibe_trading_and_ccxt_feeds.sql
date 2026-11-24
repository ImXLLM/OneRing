-- 0010_quant_vibe_trading_and_ccxt_feeds — موتور کوانتِ علّیِ `Vibe-Trading`
-- و هستهٔ `ccxt` (Wave 6 — Step 3/4).
--
-- قواعد (همان قرارداد 0005/0007/0008/0009):
--  * جدولِ تازهٔ `quant_strategies` — هر استراتژیِ AST تایپ‌دار با سنجه‌های
--    نهادی، هشِ دفترِ ادعاها و مُهرِ گیتِ مایرون ماندگار می‌شود؛ هیچ جدولِ
--    دفتری لمس نمی‌شود تا همهٔ ناورنی‌ها (User=466، Product=20،
--    Posting SUM=0) دست‌نخورده بمانند.
--  * `strategy_ast_json` — AST تایپ‌دارِ صفر-اجرای-پویا (درمانِ قضاییِ #1).
--  * `mairon_approved` پیش‌فرضِ ۱ — فقط ردیف‌هایِ سبزِ گیت درج می‌شوند.
--  * `sharpe_ratio` / `max_drawdown_pct` ستون‌هایِ پرس‌وجوی مستقیم برای
--    رتبه‌بندیِ ویترینِ کوانت.

CREATE TABLE IF NOT EXISTS "quant_strategies" (
    "id" TEXT PRIMARY KEY,
    "creator_user_id" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "symbol" TEXT NOT NULL,
    "timeframe" TEXT NOT NULL,
    "vibe_prompt" TEXT NOT NULL,
    "strategy_ast_json" TEXT NOT NULL,
    "metrics_json" TEXT NOT NULL,
    "sharpe_ratio" REAL NOT NULL,
    "max_drawdown_pct" REAL NOT NULL,
    "mairon_approved" INTEGER NOT NULL DEFAULT 1,
    "claim_verification_hash" TEXT NOT NULL,
    "skill_md" TEXT NOT NULL,
    "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS "quant_strategies_creator_idx"
    ON "quant_strategies"("creator_user_id");
CREATE INDEX IF NOT EXISTS "quant_strategies_symbol_idx"
    ON "quant_strategies"("symbol");
