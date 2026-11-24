-- 0004_m5_nation_wallet — حاکمیت (پارلمان HITL) + کیف پول USDT چهارزنجیره
--
-- قراردادها همان 0002/0003:
--  * نام‌گذاری PascalCase با کوتیشن.
--  * DATETIME رشتهٔ ISO-8601 UTC با پسوند Z.
--  * جدول‌ها IF NOT EXISTS — idempotent روی هر دیتابیسی.
--  * هیچ پولی اینجا زندگی نمی‌کند — تراز فقط از Posting مشتق می‌شود (ADR-003)؛
--    لایحه/رأی شواهد حاکمیتی‌اند و واریز USDT اثبات کاربر تا راستی‌آزمایی ادمین.

-- لایحه‌های پارلمان — گذارها در kian-nation، شمارش‌ها پس از بستن اینجا ثبت می‌شوند
CREATE TABLE IF NOT EXISTS "Bill" (
    "id" TEXT NOT NULL PRIMARY KEY,
    -- escrowRelease / treasuryMove
    "kind" TEXT NOT NULL,
    -- شناسهٔ اسکرو یا حساب مقصد
    "subject" TEXT NOT NULL,
    "amountDin" INTEGER NOT NULL DEFAULT 0,
    -- release / refund — فقط escrowRelease
    "resolution" TEXT,
    "title" TEXT NOT NULL,
    -- OPEN / PASSED / REJECTED / EXECUTED
    "status" TEXT NOT NULL DEFAULT 'OPEN',
    -- passed / quorum_not_met / majority_no / human_approval_missing
    "decisionReason" TEXT,
    "openedAt" DATETIME NOT NULL,
    "closedAt" DATETIME,
    "executedAt" DATETIME,
    -- مادهٔ ۷ — مبلغ ≥ ۵۰,۰۰۰ Đ رأی انسانی می‌خواهد (قطعی در زمان باز شدن)
    "hitlRequired" BOOLEAN NOT NULL DEFAULT false,
    -- شمارش قطعی پس از بستن — شفافیت مادهٔ ۶
    "votesYes" INTEGER NOT NULL DEFAULT 0,
    "votesNo" INTEGER NOT NULL DEFAULT 0,
    "humanYes" INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS "Bill_status_idx" ON "Bill" ("status", "openedAt");

-- رأی‌ها — هر رأی‌دهنده یک‌بار به ازای هر لایحه (قید UNIQUE؛ گیت درون kian-nation)
CREATE TABLE IF NOT EXISTS "BillVote" (
    "billId" TEXT NOT NULL,
    -- citizen:… (نشست انسانی) یا agent:… (A2A)
    "voter" TEXT NOT NULL,
    -- انسان؟ — گیت مادهٔ ۷ روی همین ستون بسته می‌شود
    "isHuman" BOOLEAN NOT NULL DEFAULT false,
    "approve" BOOLEAN NOT NULL,
    "createdAt" DATETIME NOT NULL,
    PRIMARY KEY ("billId", "voter"),
    FOREIGN KEY ("billId") REFERENCES "Bill"("id") ON DELETE CASCADE
);

-- آدرس واریز کاربر به ازای هر زنجیره — هر (کاربر، زنجیره) یک آدرس
CREATE TABLE IF NOT EXISTS "WalletAddress" (
    "userId" TEXT NOT NULL,
    -- TRX / SOL / TON / BSC
    "chain" TEXT NOT NULL,
    -- سمت سرور با چک‌سام رسمی زنجیره اعتبارسنجی شده (kian-wallet)
    "address" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL,
    PRIMARY KEY ("userId", "chain")
);

CREATE INDEX IF NOT EXISTS "WalletAddress_address_idx" ON "WalletAddress" ("address");

-- واریز USDT — اثبات کاربر؛ همان قرارداد TonDeposit با چهار زنجیره
CREATE TABLE IF NOT EXISTS "UsdtDeposit" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    "chain" TEXT NOT NULL,
    -- هش تراکنش زنجیره — UNIQUE روی (chain, txHash): «همان هش دوبار ضرب نمی‌شود»
    "txHash" TEXT NOT NULL,
    "fromAddress" TEXT NOT NULL,
    -- مقیاس micro-USDT (۱۰⁻⁶) — صحیح، بدون float
    "amountMicro" INTEGER NOT NULL,
    -- رشتهٔ دهدهی اعلامی کاربر (≤۶ اعشار) — برای نمایش صادقانه
    "amountUsdt" TEXT NOT NULL,
    -- PENDING / CONFIRMED / REJECTED
    "status" TEXT NOT NULL DEFAULT 'PENDING',
    "createdAt" DATETIME NOT NULL,
    "confirmedAt" DATETIME,
    "confirmedBy" TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS "UsdtDeposit_chain_txHash_key" ON "UsdtDeposit" ("chain", "txHash");
CREATE INDEX IF NOT EXISTS "UsdtDeposit_userId_createdAt_idx" ON "UsdtDeposit" ("userId", "createdAt");
CREATE INDEX IF NOT EXISTS "UsdtDeposit_status_idx" ON "UsdtDeposit" ("status");
