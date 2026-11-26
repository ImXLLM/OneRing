-- 0010_quant_vibe_trading_and_ccxt_feeds — the causal `Vibe-Trading` quant engine
-- and the `ccxt` core (Wave 6 — Step 3/4).
--
-- Rules (same convention as 0005/0007/0008/0009):
--  * the new `quant_strategies` table persists every typed AST strategy with institutional
--    metrics, the claims-ledger hash, and the Mairon gate seal; no ledger
--    table is touched so that every invariant (User=466, Product=20,
--    Posting SUM=0) stays untouched.
--  * `strategy_ast_json` — a typed AST with zero dynamic execution (remedy #1).
--  * `mairon_approved` defaults to 1 — only gate-green rows are inserted.
--  * `sharpe_ratio` / `max_drawdown_pct` are direct-query columns for
--    quant storefront ranking.

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
