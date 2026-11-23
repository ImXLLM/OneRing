-- 0003_m4_brain — لایهٔ مغز M4 (additive روی اسکیمای v3 — M2/M3 دست نمی‌خورد)
--
-- قراردادها همان 0002:
--  * نام‌گذاری PascalCase با کوتیشن.
--  * DATETIME رشتهٔ ISO-8601 UTC با پسوند Z.
--  * جدول‌ها IF NOT EXISTS — idempotent روی هر دیتابیسی.
--  * هیچ پولی اینجا زندگی نمی‌کند — تراز فقط از Posting مشتق می‌شود (ADR-003)؛
--    جدول‌های این فایل حافظه/شواهد اجرای مغز هستند نه دفتر.

-- ۱۶ مهارت هرمس — seed از crates/hermes (منبع حقیقت داده، نه کد)
CREATE TABLE IF NOT EXISTS "HermesSkill" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "title" TEXT NOT NULL,
    "category" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    -- kernel = اجرای قطعی روی کرنل؛ llm = قالب پرامپت + گیت‌وی AI
    "kind" TEXT NOT NULL,
    -- برآورد توکن برای پاکت بودجه — مهارت kernel صفر
    "estimateTokens" INTEGER NOT NULL,
    "active" BOOLEAN NOT NULL DEFAULT true,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS "HermesSkill_active_idx" ON "HermesSkill" ("active");

-- هر اجرای مهارت یک ردیف شاهد — مبنای امتیاز هویتی و دنبالهٔ ادمین
CREATE TABLE IF NOT EXISTS "HermesRun" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    "skillId" TEXT NOT NULL,
    -- ورودی کاربر پس از سپر (اگر REDACTED، متن پاک‌شده ذخیره می‌شود؛ راز هرگز)
    "input" TEXT NOT NULL,
    -- خروجی مهارت — برای ران مسدودشده رشتهٔ خالی
    "output" TEXT NOT NULL,
    -- ALLOW / REDACTED / BLOCKED
    "shield" TEXT NOT NULL,
    -- hermes.ok / hermes.blocked / hermes.failed
    "outcome" TEXT NOT NULL,
    "provider" TEXT,
    "model" TEXT,
    "promptTokens" INTEGER NOT NULL DEFAULT 0,
    "completionTokens" INTEGER NOT NULL DEFAULT 0,
    "totalTokens" INTEGER NOT NULL DEFAULT 0,
    "createdAt" DATETIME NOT NULL
);

CREATE INDEX IF NOT EXISTS "HermesRun_userId_createdAt_idx" ON "HermesRun" ("userId", "createdAt");
CREATE INDEX IF NOT EXISTS "HermesRun_outcome_idx" ON "HermesRun" ("outcome");

-- حافظهٔ ۷لایهٔ هرمس — هر یادداشت فقط در یک لایه؛ حذف مستقیم ندارد (audit-friendly)
CREATE TABLE IF NOT EXISTS "MemoryNote" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    -- working / episodic / semantic / procedural / identity / financial / social
    "layer" TEXT NOT NULL,
    "content" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL
);

CREATE INDEX IF NOT EXISTS "MemoryNote_userId_layer_idx" ON "MemoryNote" ("userId", "layer");

-- ناوگان ۷ نقش — ردیف ایجنت‌ها؛ وضعیت صادقانه (active/registered/disabled)
CREATE TABLE IF NOT EXISTS "Agent" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "name" TEXT NOT NULL,
    "role" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS "Agent_role_key" ON "Agent" ("role");

-- پیام‌های A2A — صندوق ورودی هر ایجنت؛ گیت‌وی فقط ادمین
CREATE TABLE IF NOT EXISTS "A2aMessage" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "fromAgent" TEXT NOT NULL,
    "toAgent" TEXT NOT NULL,
    -- handoff / query / result
    "kind" TEXT NOT NULL,
    "body" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL
);

CREATE INDEX IF NOT EXISTS "A2aMessage_toAgent_idx" ON "A2aMessage" ("toAgent", "createdAt");
