-- 0003_m4_brain — M4 brain layer (additive on the v3 schema — M2/M3 untouched)
--
-- Conventions are the same as 0002:
--  * PascalCase naming with quotes.
--  * DATETIME is an ISO-8601 UTC string with a Z suffix.
--  * tables are IF NOT EXISTS — idempotent on any database.
--  * no money lives here — the balance is derived only from Posting (ADR-003);
--    the tables in this file hold brain memory/execution evidence, not the ledger.

-- 16 Hermes skills — seeded from crates/hermes (source-of-truth data, not code)
CREATE TABLE IF NOT EXISTS "HermesSkill" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "title" TEXT NOT NULL,
    "category" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    -- kernel = deterministic on-kernel execution; llm = prompt template + AI gateway
    "kind" TEXT NOT NULL,
    -- token estimate for the budget envelope — kernel skills are zero
    "estimateTokens" INTEGER NOT NULL,
    "active" BOOLEAN NOT NULL DEFAULT true,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS "HermesSkill_active_idx" ON "HermesSkill" ("active");

-- every skill run gets one evidence row — basis of the identity score and the admin trail
CREATE TABLE IF NOT EXISTS "HermesRun" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    "skillId" TEXT NOT NULL,
    -- user input after the shield (if REDACTED, the scrubbed text is stored; never secrets)
    "input" TEXT NOT NULL,
    -- skill output — empty string for a blocked run
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

-- the 7-layer Hermes memory — every note lives in exactly one layer; no direct deletes (audit-friendly)
CREATE TABLE IF NOT EXISTS "MemoryNote" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    -- working / episodic / semantic / procedural / identity / financial / social
    "layer" TEXT NOT NULL,
    "content" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL
);

CREATE INDEX IF NOT EXISTS "MemoryNote_userId_layer_idx" ON "MemoryNote" ("userId", "layer");

-- the 7-role fleet — agent roster; honest status (active/registered/disabled)
CREATE TABLE IF NOT EXISTS "Agent" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "name" TEXT NOT NULL,
    "role" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS "Agent_role_key" ON "Agent" ("role");

-- A2A messages — each agent's inbox; the gateway is admin-only
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
