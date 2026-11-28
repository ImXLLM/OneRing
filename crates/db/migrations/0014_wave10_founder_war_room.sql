-- Migration 0014 — Wave 10 (Commit #30): the sovereign founder war room +
-- the real-time "Day 0 → Day 1" simulation engine
--
-- two additive tables:
-- - `FounderWarRoomTick`: each of the 24 Day-0 operational steps — speaker, bilingual
--   message (Persian/English), terminal command, social post, wallet delta and total
--   (µUSDC), pipeline stage, and active revenue stream;
-- - `ChairmanDirectiveLog`: every tactical directive from the chairman `0xKIA` + the
--   in-persona replies of the three co-founders (Elon/CZ/Jeff) in both languages.
--
-- no double-entry ledger table is touched — the `Posting` zero-sum stays untouched.

CREATE TABLE IF NOT EXISTS "FounderWarRoomTick" (
    "id" TEXT PRIMARY KEY,
    -- schema-level guards: the canonical scenario is exactly 24 ticks
    "tick_index" INTEGER NOT NULL CHECK ("tick_index" BETWEEN 0 AND 23),
    "simulation_second" INTEGER NOT NULL,
    "speaker_id" TEXT NOT NULL,
    "speaker_name" TEXT NOT NULL,
    "message_fa" TEXT NOT NULL,
    "message_en" TEXT NOT NULL,
    "terminal_cmd" TEXT,
    "social_post_json" TEXT,
    "wallet_delta_usdc_micros" INTEGER NOT NULL DEFAULT 0,
    -- schema-level guard: the sovereign wallet never goes negative
    "wallet_total_usdc_micros" INTEGER NOT NULL DEFAULT 0 CHECK ("wallet_total_usdc_micros" >= 0),
    "active_pipeline_stage" TEXT NOT NULL,
    "created_at" TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS "idx_war_room_tick_order"
    ON "FounderWarRoomTick"("tick_index");

CREATE TABLE IF NOT EXISTS "ChairmanDirectiveLog" (
    "id" TEXT PRIMARY KEY,
    "directive_text" TEXT NOT NULL,
    "target_founder" TEXT NOT NULL,
    "elon_response" TEXT NOT NULL,
    "cz_response" TEXT NOT NULL,
    "bezos_response" TEXT NOT NULL,
    "triggered_tick" INTEGER NOT NULL DEFAULT 0,
    "created_at" TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS "idx_chairman_directive_time"
    ON "ChairmanDirectiveLog"("created_at");

-- ── named war-room state + tick-event ledger (Gate 5 remediation) ─────────────
-- `WarRoomState`: the singleton live state of the Day 0 → Day 1 simulation.
-- `WarRoomTickEvent`: the append-only causal ledger of executed ticks — every row
-- carries the wallet total after the tick so the ledger is replayable.
-- Both tables keep the same schema-level guards as `FounderWarRoomTick`.

CREATE TABLE IF NOT EXISTS "WarRoomState" (
    "id" TEXT PRIMARY KEY,
    -- schema-level guard: the canonical scenario is exactly 24 ticks
    "tick_index" INTEGER NOT NULL DEFAULT 0 CHECK (tick_index BETWEEN 0 AND 23),
    "elapsed_seconds" INTEGER NOT NULL DEFAULT 0,
    "tick_cursor" INTEGER NOT NULL DEFAULT 0 CHECK (tick_cursor BETWEEN 0 AND 23),
    -- schema-level guard: the sovereign wallet never goes negative
    "wallet_total_usdc_micros" INTEGER NOT NULL DEFAULT 0 CHECK (wallet_total_usdc_micros >= 0),
    "earned_revenue_micros" INTEGER NOT NULL DEFAULT 0,
    "creator_payouts_micros" INTEGER NOT NULL DEFAULT 0,
    "affiliate_payouts_micros" INTEGER NOT NULL DEFAULT 0,
    "active_users" INTEGER NOT NULL DEFAULT 0,
    "locale" TEXT NOT NULL DEFAULT 'en',
    "updated_at" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "WarRoomTickEvent" (
    "id" TEXT PRIMARY KEY,
    -- schema-level guard: the canonical scenario is exactly 24 ticks
    "tick_index" INTEGER NOT NULL CHECK (tick_index BETWEEN 0 AND 23),
    "simulation_second" INTEGER NOT NULL,
    "speaker_id" TEXT NOT NULL,
    "revenue_stream" TEXT NOT NULL DEFAULT 'None',
    "wallet_delta_usdc_micros" INTEGER NOT NULL DEFAULT 0,
    -- schema-level guard: the sovereign wallet never goes negative
    "wallet_total_usdc_micros" INTEGER NOT NULL DEFAULT 0 CHECK (wallet_total_usdc_micros >= 0),
    "message_en" TEXT NOT NULL,
    "created_at" TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS "idx_war_room_tick_event_order"
    ON "WarRoomTickEvent"("tick_index");
