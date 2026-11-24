-- 0002_m3_sessions — لایهٔ هویت M3 (additive روی اسکیمای v3 — M2 دست نمی‌خورد)
--
-- قراردادها:
--  * نام‌گذاری PascalCase با کوتیشن — همان قرارداد schema.prisma برای dual-run.
--  * DATETIME رشتهٔ ISO-8601 UTC با پسوند Z (قرارداد EPOCH_ISO).
--  * جدول‌ها IF NOT EXISTS — idempotent روی هر دیتابیسی.
--  * تراز کاربر همچنان از Posting مشتق می‌شود (ADR-003) — جدول User «هویت» است
--    نه پول؛ حساب دفتر در ستون accountId زندگی می‌کند (`citizen:{telegramId}`).

CREATE TABLE IF NOT EXISTS "User" (
    "id" TEXT NOT NULL PRIMARY KEY,
    -- شناسهٔ عددی تلگرام به‌صورت رشته — حفاظ 2^53−1 (هرگز float)
    "telegramId" TEXT NOT NULL,
    "username" TEXT,
    "tonAddress" TEXT,
    -- حساب دفتر دوطرفهٔ متصل (kind=User) — پیشوند citizen:
    "accountId" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- یک هویت تلگرام = یک شهروند — ثبت‌نام دوباره همان ردیف را پیدا می‌کند
CREATE UNIQUE INDEX IF NOT EXISTS "User_telegramId_key" ON "User" ("telegramId");
CREATE INDEX IF NOT EXISTS "User_accountId_idx" ON "User" ("accountId");

CREATE TABLE IF NOT EXISTS "Session" (
    -- توکن اپاک ۶۴ hex — هش‌شدنی، اما برای v3 کوتاه‌مدت به‌صورت متن ذخیره می‌شود
    -- (TTL پیش‌فرض ۷ روز؛ فایل sqlite حضانت‌شدهٔ فایل‌سیستمی است)
    "token" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL,
    "expiresAt" DATETIME NOT NULL,
    CONSTRAINT "Session_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS "Session_userId_idx" ON "Session" ("userId");
CREATE INDEX IF NOT EXISTS "Session_expiresAt_idx" ON "Session" ("expiresAt");

CREATE TABLE IF NOT EXISTS "TonDeposit" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    -- اثبات کاربر — ادمین قبل از ضرب مادهٔ ۳ آن را راستی‌آزمایی می‌کند
    "txHash" TEXT NOT NULL,
    "walletAddress" TEXT NOT NULL,
    -- مبلغ TON رشتهٔ دهدهی — هرگز float
    "amountTon" TEXT NOT NULL,
    -- PENDING / CONFIRMED / REJECTED
    "status" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL,
    "confirmedAt" DATETIME,
    "confirmedBy" TEXT
);

-- همان هش زنجیره هرگز دو واریز نمی‌شود (درس replay فاز ۲)
CREATE UNIQUE INDEX IF NOT EXISTS "TonDeposit_txHash_key" ON "TonDeposit" ("txHash");
CREATE INDEX IF NOT EXISTS "TonDeposit_userId_idx" ON "TonDeposit" ("userId");
CREATE INDEX IF NOT EXISTS "TonDeposit_status_idx" ON "TonDeposit" ("status");
