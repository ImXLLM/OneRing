-- 0008_arda_locales_and_skill_forge — ARDA's 24-hour planetary localization
-- on the authentic Fishkal architecture + the Mairon skill forge (Wave 4 — Step 1/4).
--
-- Rules (same convention as 0005/0007):
--  * new columns on "Product" carry safe defaults so legacy databases and the
--    legacy migration path upgrade without failure; all 20 existing products
--    are preserved with the default origin 'en' and the default band ["en","fa","ar"].
--  * "skillMd" / "skillJson" are nullable — the Mairon skill bundle is forged and
--    sealed only after passing the REA >= 85 gate (two-key gate preserved).
--  * "ardaLastLocalizedAt" is the timestamp of ARDA's 24-hour cron tick.

-- the ARDA localization seal on the product
ALTER TABLE "Product" ADD COLUMN "originLocale" TEXT NOT NULL DEFAULT 'en';
ALTER TABLE "Product" ADD COLUMN "supportedLocalesJson" TEXT NOT NULL DEFAULT '["en","fa","ar"]';
ALTER TABLE "Product" ADD COLUMN "skillMd" TEXT;
ALTER TABLE "Product" ADD COLUMN "skillJson" TEXT;
ALTER TABLE "Product" ADD COLUMN "ardaLastLocalizedAt" TEXT;

CREATE INDEX IF NOT EXISTS "Product_originLocale_idx" ON "Product"("originLocale");
