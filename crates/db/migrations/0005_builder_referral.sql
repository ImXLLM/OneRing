-- 0005_builder_referral — HTML preview and artifact, escrow referral commission, and AI product-builder sessions
--
-- Rules:
--  * new columns on "Product" and "Escrow" carry safe defaults so that legacy
--    databases and the legacy migration path upgrade without failure.
--  * the BuilderSession and BuilderMessage tables are created with IF NOT EXISTS for phase 4.

-- Product table upgrade: HTML preview, referrer commission bps, and artifact kind
ALTER TABLE "Product" ADD COLUMN "previewHtml" TEXT NOT NULL DEFAULT '';
ALTER TABLE "Product" ADD COLUMN "affiliateBps" INTEGER NOT NULL DEFAULT 1000;
ALTER TABLE "Product" ADD COLUMN "artifactKind" TEXT NOT NULL DEFAULT 'html';

-- Escrow table upgrade: referrer account and commission bps recorded at purchase time
ALTER TABLE "Escrow" ADD COLUMN "referrer" TEXT;
ALTER TABLE "Escrow" ADD COLUMN "affiliateBps" INTEGER NOT NULL DEFAULT 0;

-- AI interactive product-builder sessions (preparation for phase 4)
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

-- multi-turn product-builder chat messages
CREATE TABLE IF NOT EXISTS "BuilderMessage" (
    "id" INTEGER PRIMARY KEY AUTOINCREMENT,
    "sessionId" TEXT NOT NULL,
    "role" TEXT NOT NULL,
    "content" TEXT NOT NULL,
    "createdAt" INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS "BuilderMessage_sessionId_id_idx"
    ON "BuilderMessage"("sessionId", "id");

-- crypto withdrawal requests (phase 6 — atomic Din lock/burn at record time to prevent double-spend)
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
