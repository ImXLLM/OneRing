-- 0002_m3_sessions — M3 identity layer (additive on the v3 schema — M2 untouched)
--
-- Conventions:
--  * PascalCase naming with quotes — same convention as schema.prisma for dual-run.
--  * DATETIME is an ISO-8601 UTC string with a Z suffix (EPOCH_ISO convention).
--  * tables are IF NOT EXISTS — idempotent on any database.
--  * the user balance is still derived from Posting (ADR-003) — the User table is
--    identity, not money; the ledger account lives in the accountId column (`citizen:{telegramId}`).

CREATE TABLE IF NOT EXISTS "User" (
    "id" TEXT NOT NULL PRIMARY KEY,
    -- numeric Telegram id stored as a string — guards the 2^53−1 limit (never float)
    "telegramId" TEXT NOT NULL,
    "username" TEXT,
    "tonAddress" TEXT,
    -- linked double-entry ledger account (kind=User) — citizen: prefix
    "accountId" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- one Telegram identity = one citizen — re-registration finds the same row
CREATE UNIQUE INDEX IF NOT EXISTS "User_telegramId_key" ON "User" ("telegramId");
CREATE INDEX IF NOT EXISTS "User_accountId_idx" ON "User" ("accountId");

CREATE TABLE IF NOT EXISTS "Session" (
    -- opaque 64-hex token — hashable, but stored as plaintext for the short-lived v3
    -- (default TTL 7 days; the sqlite file is a filesystem-guarded custody file)
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
    -- user proof — the admin verifies it before minting (article 3)
    "txHash" TEXT NOT NULL,
    "walletAddress" TEXT NOT NULL,
    -- TON amount as a decimal string — never float
    "amountTon" TEXT NOT NULL,
    -- PENDING / CONFIRMED / REJECTED
    "status" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL,
    "confirmedAt" DATETIME,
    "confirmedBy" TEXT
);

-- the same on-chain hash never deposits twice (phase-2 replay lesson)
CREATE UNIQUE INDEX IF NOT EXISTS "TonDeposit_txHash_key" ON "TonDeposit" ("txHash");
CREATE INDEX IF NOT EXISTS "TonDeposit_userId_idx" ON "TonDeposit" ("userId");
CREATE INDEX IF NOT EXISTS "TonDeposit_status_idx" ON "TonDeposit" ("status");
