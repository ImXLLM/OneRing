-- 0001_init — complete v3 schema — one-to-one mirror of back/server/prisma/schema.prisma
-- (table, column and index names are kept "exactly" identical so the phase-8 dual-run
--  can sit on the very SQLite file Prisma produced, without any conversion.)
--  * every CREATE is IF NOT EXISTS: creates on a fresh database, no-ops on a database
--    already built by Prisma (idempotent).
--  * DATETIMEs are ISO-8601 UTC strings with a Z suffix (domain convention: EPOCH_ISO).
--  * ADR-003: balances are not stored — "Posting" is the single source of truth and the
--    balance is always derived with SUM(amount).

CREATE TABLE IF NOT EXISTS "Account" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "kind" TEXT NOT NULL,
    "label" TEXT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "Tx" (
    "id" TEXT NOT NULL PRIMARY KEY,
    -- idempotency-key uniqueness constraint — guarantees that "a retry race never
    -- moves money twice" (ADR-003 + phase 2)
    "idempotencyKey" TEXT,
    "memo" TEXT,
    "at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS "Tx_idempotencyKey_key" ON "Tx" ("idempotencyKey");

CREATE TABLE IF NOT EXISTS "Posting" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "txId" TEXT NOT NULL,
    "accountId" TEXT NOT NULL,
    "amount" INTEGER NOT NULL,
    CONSTRAINT "Posting_txId_fkey" FOREIGN KEY ("txId") REFERENCES "Tx" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "Posting_accountId_fkey" FOREIGN KEY ("accountId") REFERENCES "Account" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS "Posting_accountId_idx" ON "Posting" ("accountId");
CREATE INDEX IF NOT EXISTS "Posting_txId_idx" ON "Posting" ("txId");

CREATE TABLE IF NOT EXISTS "Escrow" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "buyerId" TEXT NOT NULL,
    "sellerId" TEXT NOT NULL,
    "amount" INTEGER NOT NULL,
    "productRef" TEXT,
    "status" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL,
    "fundedTxId" TEXT,
    "settleTxId" TEXT,
    "disputeReason" TEXT,
    "deliveryRef" TEXT,
    "expiresAt" DATETIME
);

CREATE INDEX IF NOT EXISTS "Escrow_status_idx" ON "Escrow" ("status");

CREATE TABLE IF NOT EXISTS "OutboxEvent" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "kind" TEXT NOT NULL,
    -- JSON-encoded DomainEvent — same wire shape as JSON.stringify in TS
    "payload" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "published" BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS "AuditLog" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "actor" TEXT NOT NULL,
    "action" TEXT NOT NULL,
    "target" TEXT,
    "meta" TEXT,
    "at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "HitlRequest" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "kind" TEXT NOT NULL,
    "escrowId" TEXT NOT NULL,
    "amount" INTEGER NOT NULL,
    "requesterId" TEXT NOT NULL,
    "note" TEXT,
    "status" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "expiresAt" DATETIME NOT NULL,
    "decidedBy" TEXT,
    "decidedAt" DATETIME,
    "decisionNote" TEXT
);

CREATE INDEX IF NOT EXISTS "HitlRequest_status_idx" ON "HitlRequest" ("status");
CREATE INDEX IF NOT EXISTS "HitlRequest_escrowId_idx" ON "HitlRequest" ("escrowId");

CREATE TABLE IF NOT EXISTS "AiUsage" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    "agent" TEXT NOT NULL,
    "provider" TEXT NOT NULL,
    "model" TEXT NOT NULL,
    "kind" TEXT NOT NULL,
    "promptTokens" INTEGER NOT NULL,
    "completionTokens" INTEGER NOT NULL,
    "totalTokens" INTEGER NOT NULL,
    "outcome" TEXT NOT NULL,
    "at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS "AiUsage_userId_at_idx" ON "AiUsage" ("userId", "at");

CREATE TABLE IF NOT EXISTS "Product" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "sellerId" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "priceDin" INTEGER NOT NULL,
    "category" TEXT,
    -- legacy lesson: files on the Railway ephemeral disk were lost — delivery content
    -- is now stored inside the row itself.
    "deliveryPayload" TEXT,
    "active" BOOLEAN NOT NULL DEFAULT true,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS "Product_active_idx" ON "Product" ("active");
CREATE INDEX IF NOT EXISTS "Product_sellerId_idx" ON "Product" ("sellerId");

CREATE TABLE IF NOT EXISTS "KbEntry" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "title" TEXT NOT NULL,
    "body" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL
);

CREATE INDEX IF NOT EXISTS "KbEntry_title_idx" ON "KbEntry" ("title");
