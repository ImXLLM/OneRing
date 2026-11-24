-- 0005_builder_referral — پیش‌نمایش و آرتیفکت HTML، پورسانت رفرال اسکرو و جلسات سازندهٔ محصول با AI
--
-- قواعد:
--  * ستون‌های جدید روی "Product" و "Escrow" دارای مقدار پیش‌فرض ایمن هستند تا
--    دیتابیس‌های قدیمی و مسیر مهاجرت لگاسی بدون شکست ارتقا یابند.
--  * جدول‌های BuilderSession و BuilderMessage با IF NOT EXISTS برای فاز ۴ ساخته می‌شوند.

-- ارتقای جدول Product: پیش‌نمایش HTML، درصد پورسانت معرف و نوع آرتیفکت
ALTER TABLE "Product" ADD COLUMN "previewHtml" TEXT NOT NULL DEFAULT '';
ALTER TABLE "Product" ADD COLUMN "affiliateBps" INTEGER NOT NULL DEFAULT 1000;
ALTER TABLE "Product" ADD COLUMN "artifactKind" TEXT NOT NULL DEFAULT 'html';

-- ارتقای جدول Escrow: حساب معرف و درصد پورسانت ثبت‌شده در زمان خرید
ALTER TABLE "Escrow" ADD COLUMN "referrer" TEXT;
ALTER TABLE "Escrow" ADD COLUMN "affiliateBps" INTEGER NOT NULL DEFAULT 0;

-- جلسات ساخت محصول تعاملی با AI (آماده‌سازی برای فاز ۴)
CREATE TABLE IF NOT EXISTS "BuilderSession" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "accountId" TEXT NOT NULL,
    "stage" TEXT NOT NULL DEFAULT 'discovery',
    "title" TEXT NOT NULL DEFAULT '',
    "description" TEXT NOT NULL DEFAULT '',
    "suggestedPrice" INTEGER NOT NULL DEFAULT 100,
    "affiliateBps" INTEGER NOT NULL DEFAULT 1000,
    "generatedHtml" TEXT NOT NULL DEFAULT '',
    "publishedProductId" TEXT,
    "createdAt" INTEGER NOT NULL,
    "updatedAt" INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS "BuilderSession_accountId_updatedAt_idx"
    ON "BuilderSession"("accountId", "updatedAt");

-- پیام‌های گفتگوی چندمرحله‌ای سازندهٔ محصول
CREATE TABLE IF NOT EXISTS "BuilderMessage" (
    "id" INTEGER PRIMARY KEY AUTOINCREMENT,
    "sessionId" TEXT NOT NULL,
    "role" TEXT NOT NULL,
    "content" TEXT NOT NULL,
    "createdAt" INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS "BuilderMessage_sessionId_id_idx"
    ON "BuilderMessage"("sessionId", "id");

-- درخواست‌های برداشت کریپتو (فاز ۶ — قفل/سوزاندن اتمی Din در لحظهٔ ثبت برای ضد دابل‌اسپند)
CREATE TABLE IF NOT EXISTS "WithdrawalRequest" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "accountId" TEXT NOT NULL,
    "chain" TEXT NOT NULL,
    "asset" TEXT NOT NULL,
    "destinationAddress" TEXT NOT NULL,
    "amountDin" INTEGER NOT NULL,
    "txHash" TEXT,
    "status" TEXT NOT NULL DEFAULT 'PENDING',
    "createdAt" INTEGER NOT NULL,
    "resolvedAt" INTEGER
);

CREATE INDEX IF NOT EXISTS "WithdrawalRequest_accountId_createdAt_idx"
    ON "WithdrawalRequest"("accountId", "createdAt");

CREATE INDEX IF NOT EXISTS "WithdrawalRequest_status_createdAt_idx"
    ON "WithdrawalRequest"("status", "createdAt");
