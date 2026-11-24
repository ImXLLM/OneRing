-- 0007_trinity_reviews — مُهر مایرون روی لیستینگ بازار + رأی ۵-ستارهٔ شهروندان
-- (Wave 3 — Sovereign Trinity Studio Pipeline)
--
-- قواعد:
--  * ستون‌های جدید روی "Product" مقدار پیش‌فرضِ ایمن دارند تا دیتابیس‌های قدیمی
--    و مسیر مهاجرت لگاسی بدون شکست ارتقا یابند (همان قرارداد 0005).
--  * جدول "ProductReview" با قفلِ یک‌رأییِ (محصول، رأی‌دهنده) در سطح UNIQUE —
--    سوریل نمی‌تواند امتیاز بسازد؛ پشتوانهٔ دیتابیسیِ قاعدهٔ دامنه.
--  * نمره و نشان مایرون فقط با دروازهٔ دو-کلیدی (حکم مایرون + تأیید سازنده)
--    روی "Product" می‌نشیند.

-- مُهر داوری مایرون روی محصول (دروازهٔ دو-کلیدی انتشار)
ALTER TABLE "Product" ADD COLUMN "maironScore" INTEGER NOT NULL DEFAULT 0;
ALTER TABLE "Product" ADD COLUMN "maironBadge" TEXT NOT NULL DEFAULT '';

-- رأی واقعی شهروندان — ستارهٔ ۱..=۵، نظر و وزنِ خریدِ تاییدشده
CREATE TABLE IF NOT EXISTS "ProductReview" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "productId" TEXT NOT NULL,
    "reviewerId" TEXT NOT NULL,
    "stars" INTEGER NOT NULL,
    "comment" TEXT NOT NULL,
    "verifiedPurchase" INTEGER NOT NULL DEFAULT 0,
    "maironScoreAtReview" INTEGER NOT NULL DEFAULT 0,
    "createdAt" TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS "ProductReview_product_reviewer_uniq"
    ON "ProductReview"("productId", "reviewerId");

CREATE INDEX IF NOT EXISTS "ProductReview_productId_idx"
    ON "ProductReview"("productId");
