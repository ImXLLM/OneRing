-- مهاجرت 0011 — انحصارِ حاکم، ژنومِ چهارمدلی و خزانهٔ Base
-- (Wave 7 — Step 4/4)
--
-- جدولِ `sovereign_treasury_splits`: لایحهٔ دفتریِ تقسیمِ اتمیِ روزِ اول —
-- خروجیِ `compute_day_one_atomic_split` و `compute_instant_boost_split`
-- (جریان‌هایِ ۱ تا ۴ منشور) قبل از انتشار روی وایر ماندگار می‌شود.
--
-- اصلِ صفرجمع: `kia_treasury_micros + referrer_micros + creator_net_micros
-- == gross_amount_micros` — ردیفِ نامتوازن هرگز درج نمی‌شود (گیتِ
-- `zero_sum_verified` در لایهٔ db).

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
