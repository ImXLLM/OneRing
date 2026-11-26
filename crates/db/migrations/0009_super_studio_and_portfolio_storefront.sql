-- 0009_super_studio_and_portfolio_storefront — the three-tier cognitive
-- studio (Age 9 ↔ Age 28), the 9-modality zero-cost super-studio, and the
-- portfolio-builder storefront (Wave 5 — Step 2/4).
--
-- Rules (same convention as 0005/0007/0008):
--  * the new `studio_creations` table persists every studio creation with its morphed
--    cognitive tier, generative modality, JSON storefront card, and deterministic pricing;
--    no existing table is touched so that every ledger invariant
--    (User=466, Product=20, Posting SUM=0) stays untouched.
--  * `rea_score` defaults to 92 — above the Mairon forge gate (>= 85).
--  * `price_xtr_stars` defaults to 50 and `price_ukian` defaults to 1,000,000 —
--    the ToySpark one-click price lock.

CREATE TABLE IF NOT EXISTS "studio_creations" (
    "id" TEXT PRIMARY KEY,
    "creator_user_id" TEXT NOT NULL,
    "cognitive_tier" TEXT NOT NULL DEFAULT 'ToySpark',
    "primary_modality" TEXT NOT NULL,
    "prompt_text" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "storefront_card_json" TEXT NOT NULL DEFAULT '{}',
    "rea_score" INTEGER NOT NULL DEFAULT 92,
    "price_ukian" INTEGER NOT NULL DEFAULT 1000000,
    "price_xtr_stars" INTEGER NOT NULL DEFAULT 50,
    "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS "studio_creations_creator_idx"
    ON "studio_creations"("creator_user_id");
CREATE INDEX IF NOT EXISTS "studio_creations_modality_idx"
    ON "studio_creations"("primary_modality");
