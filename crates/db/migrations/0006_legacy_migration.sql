-- 0006_legacy_migration — ستون‌های هویت لگاسی dropagent + جدول نسبِ رفرال کمپین ۳۰روزه
--
-- زمینه: مهاجرت جامعهٔ زندهٔ dropagent (۴۶۶ کاربر، ۲۴۷ لینک رفرال) به KIAN v2.
-- قواعد همان 0002/0005:
--  * ALTERها ستون nullable با بدون مقدار پیش‌فرض افزوده می‌شوند تا ردیف‌های
--    قدیمی دست‌نخورده بمانند (در SQLite ADD COLUMN با NULL پیش‌فرض امن است).
--  * جدول‌ها IF NOT EXISTS — idempotent روی هر دیتابیسی.
--  * هویت «پول» نمی‌داند — تراز فقط از Posting مشتق می‌شود (ADR-003).

-- نام کوچک پروفایل لگاسی (first_name تلگرام) — فقط نمایشی
ALTER TABLE "User" ADD COLUMN "firstName" TEXT;

-- معرف مستقیم شهروند — شناسهٔ حساب شهروندی معرف (`citizen:{tg}`) یا NULL
ALTER TABLE "User" ADD COLUMN "referredBy" TEXT;

-- آرشیو کامل و قطعی ۲۴۷ لینک رفرال کمپین ۳۰روزه X — هر ردیف یک لینک لگاسی
-- (tgUserId = شناسهٔ عددی تلگرام کاربرِ معرف‌شده به‌صورت رشته؛ هرگز float)
CREATE TABLE IF NOT EXISTS "ReferralLineage" (
    "tgUserId" TEXT NOT NULL PRIMARY KEY,
    "referredByTgId" TEXT NOT NULL,
    "legacyCreatedAt" DATETIME
);
