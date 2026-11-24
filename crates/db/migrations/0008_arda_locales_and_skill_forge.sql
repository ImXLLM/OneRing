-- 0008_arda_locales_and_skill_forge — ARDA (آردا) بومی‌سازیِ سیاره‌ایِ ۲۴ساعته
-- روی معماری اصیل Fishkal + فورجِ مهارتِ مایرون (Wave 4 — Step 1/4).
--
-- قواعد (همان قرارداد 0005/0007):
--  * ستون‌های جدید روی "Product" مقدار پیش‌فرضِ ایمن دارند تا دیتابیس‌های
--    قدیمی و مسیر مهاجرت لگاسی بدون شکست ارتقا یابند؛ هر ۲۰ محصولِ موجود
--    با مبدأ پیش‌فرض 'en' و نوارِ پیش‌فرض ["en","fa","ar"] حفظ می‌شوند.
--  * "skillMd" / "skillJson" نال‌پذیرند — بستهٔ مهارتِ مایرون فقط پس از
--    عبور از گیتِ REA >= 85 فورج و مُهر می‌شود (دروازهٔ دو-کلیدی محفوظ).
--  * "ardaLastLocalizedAt" مُهرِ زمانِ تیکِ کرونِ ۲۴ساعتهٔ ARDA است.

-- مُهر بومی‌سازی ARDA روی محصول
ALTER TABLE "Product" ADD COLUMN "originLocale" TEXT NOT NULL DEFAULT 'en';
ALTER TABLE "Product" ADD COLUMN "supportedLocalesJson" TEXT NOT NULL DEFAULT '["en","fa","ar"]';
ALTER TABLE "Product" ADD COLUMN "skillMd" TEXT;
ALTER TABLE "Product" ADD COLUMN "skillJson" TEXT;
ALTER TABLE "Product" ADD COLUMN "ardaLastLocalizedAt" TEXT;

CREATE INDEX IF NOT EXISTS "Product_originLocale_idx" ON "Product"("originLocale");
