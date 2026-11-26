-- 0004_m5_nation_wallet — governance (HITL parliament) + four-chain USDT wallet
--
-- Conventions are the same as 0002/0003:
--  * PascalCase naming with quotes.
--  * DATETIME is an ISO-8601 UTC string with a Z suffix.
--  * tables are IF NOT EXISTS — idempotent on any database.
--  * no money lives here — the balance is derived only from Posting (ADR-003);
--    bills/votes are governance evidence and the USDT deposit is user proof until admin verification.

-- parliament bills — transitions live in kian-nation; tallies are recorded here after closing
CREATE TABLE IF NOT EXISTS "Bill" (
    "id" TEXT NOT NULL PRIMARY KEY,
    -- escrowRelease / treasuryMove
    "kind" TEXT NOT NULL,
    -- escrow id or destination account
    "subject" TEXT NOT NULL,
    "amountDin" INTEGER NOT NULL DEFAULT 0,
    -- release / refund — escrowRelease only
    "resolution" TEXT,
    "title" TEXT NOT NULL,
    -- OPEN / PASSED / REJECTED / EXECUTED
    "status" TEXT NOT NULL DEFAULT 'OPEN',
    -- passed / quorum_not_met / majority_no / human_approval_missing
    "decisionReason" TEXT,
    "openedAt" DATETIME NOT NULL,
    "closedAt" DATETIME,
    "executedAt" DATETIME,
    -- article 7 — amounts >= 50,000 Đ require a human vote (deterministic at open time)
    "hitlRequired" BOOLEAN NOT NULL DEFAULT false,
    -- deterministic tally after closing — article-6 transparency
    "votesYes" INTEGER NOT NULL DEFAULT 0,
    "votesNo" INTEGER NOT NULL DEFAULT 0,
    "humanYes" INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS "Bill_status_idx" ON "Bill" ("status", "openedAt");

-- votes — one vote per voter per bill (UNIQUE constraint; gating inside kian-nation)
CREATE TABLE IF NOT EXISTS "BillVote" (
    "billId" TEXT NOT NULL,
    -- citizen:… (human session) or agent:… (A2A)
    "voter" TEXT NOT NULL,
    -- human? — the article-7 gate closes on this column
    "isHuman" BOOLEAN NOT NULL DEFAULT false,
    "approve" BOOLEAN NOT NULL,
    "createdAt" DATETIME NOT NULL,
    PRIMARY KEY ("billId", "voter"),
    FOREIGN KEY ("billId") REFERENCES "Bill"("id") ON DELETE CASCADE
);

-- user deposit address per chain — one address per (user, chain)
CREATE TABLE IF NOT EXISTS "WalletAddress" (
    "userId" TEXT NOT NULL,
    -- TRX / SOL / TON / BSC
    "chain" TEXT NOT NULL,
    -- server-side validated with the chain's official checksum (kian-wallet)
    "address" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL,
    PRIMARY KEY ("userId", "chain")
);

CREATE INDEX IF NOT EXISTS "WalletAddress_address_idx" ON "WalletAddress" ("address");

-- USDT deposit — user proof; the TonDeposit contract extended to four chains
CREATE TABLE IF NOT EXISTS "UsdtDeposit" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    "chain" TEXT NOT NULL,
    -- on-chain tx hash — UNIQUE on (chain, txHash): "the same hash is never minted twice"
    "txHash" TEXT NOT NULL,
    "fromAddress" TEXT NOT NULL,
    -- micro-USDT scale (10⁻⁶) — integer, no float
    "amountMicro" INTEGER NOT NULL,
    -- user-declared decimal string (<= 6 decimals) — for honest display
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
