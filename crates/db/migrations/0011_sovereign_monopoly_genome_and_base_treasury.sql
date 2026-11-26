-- Migration 0011 — sovereign monopoly, the four-model genome, and the Base treasury
-- (Wave 7 — Step 4/4)
--
-- the `sovereign_treasury_splits` table: the Day-1 atomic-split ledger bill —
-- the output of `compute_day_one_atomic_split` and `compute_instant_boost_split`
-- (charter streams 1 to 4) is persisted before hitting the wire.
--
-- zero-sum principle: `kia_treasury_micros + referrer_micros + creator_net_micros
-- == gross_amount_micros` — an unbalanced row is never inserted (the
-- `zero_sum_verified` gate in the db layer).

CREATE TABLE IF NOT EXISTS "sovereign_treasury_splits" (
    "id" TEXT PRIMARY KEY,
    "buyer_user_id" TEXT NOT NULL,
    "creator_user_id" TEXT NOT NULL,
    "referrer_user_id" TEXT,
    "settlement_asset" TEXT NOT NULL,
    "gross_amount_micros" INTEGER NOT NULL,
    "kia_treasury_micros" INTEGER NOT NULL,
    "referrer_micros" INTEGER NOT NULL,
    "creator_net_micros" INTEGER NOT NULL,
    "effective_platform_bps" INTEGER NOT NULL,
    "zero_sum_verified" INTEGER NOT NULL DEFAULT 1,
    "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS "idx_sovereign_splits_created_at"
    ON "sovereign_treasury_splits" ("created_at" DESC);
CREATE INDEX IF NOT EXISTS "idx_sovereign_splits_creator"
    ON "sovereign_treasury_splits" ("creator_user_id");
