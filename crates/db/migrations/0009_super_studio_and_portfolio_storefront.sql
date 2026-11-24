-- 0009_super_studio_and_portfolio_storefront — استودیوی ادراک‌سیارِ
-- سه‌لایه (Age 9 ↔ Age 28)، ابراستودیوی ۹-مدالیتیِ صفر-هزینه و ویترینِ
-- سازندهٔ پورتفولیو (Wave 5 — Step 2/4).
--
-- قواعد (همان قرارداد 0005/0007/0008):
--  * جدولِ تازهٔ `studio_creations` — هر آفرینشِ استودیو با لایهٔ شناختیِ
--    مورف‌شده، مدالیتیِ مولد، کارتِ ویترینِ JSON و قیمت‌گذاریِ قطعی ماندگار
--    می‌شود؛ هیچ جدولِ موجود لمس نمی‌شود تا همهٔ ناورنی‌های دفتر
--    (User=466، Product=20، Posting SUM=0) دست‌نخورده بمانند.
--  * `rea_score` پیش‌فرضِ ۹۲ — بالای گیتِ فورجِ مایرون (>= 85).
--  * `price_xtr_stars` پیش‌فرضِ ۵۰ و `price_ukian` پیش‌فرضِ ۱٬۰۰۰٬۰۰۰ —
--    قفلِ قیمتِ یک‌کلیکیِ ToySpark.

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
