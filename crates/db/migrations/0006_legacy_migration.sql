-- 0006_legacy_migration — dropagent legacy identity columns + the 30-day campaign referral lineage table
--
-- Context: migrating the live dropagent community (466 users, 247 referral links) to KIAN v2.
-- Rules are the same as 0002/0005:
--  * ALTERs add nullable columns without a default so that old rows stay
--    untouched (in SQLite, ADD COLUMN with a NULL default is safe).
--  * tables are IF NOT EXISTS — idempotent on any database.
--  * identity never knows about "money" — the balance is derived only from Posting (ADR-003).

-- legacy profile first name (Telegram first_name) — display only
ALTER TABLE "User" ADD COLUMN "firstName" TEXT;

-- the citizen's direct referrer — the referrer's citizen account id (`citizen:{tg}`) or NULL
ALTER TABLE "User" ADD COLUMN "referredBy" TEXT;

-- complete, deterministic archive of the 247 referral links of the 30-day X campaign — one legacy link per row
-- (tgUserId = the referred user's numeric Telegram id as a string; never float)
CREATE TABLE IF NOT EXISTS "ReferralLineage" (
    "tgUserId" TEXT NOT NULL PRIMARY KEY,
    "referredByTgId" TEXT NOT NULL,
    "legacyCreatedAt" DATETIME
);
