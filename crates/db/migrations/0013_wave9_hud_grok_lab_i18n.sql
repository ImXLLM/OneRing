-- Migration 0013 — Wave 9 (Commit #29): the neural HUD studio + the
-- 10-DNA lab + the Grok UX engine + the Day-1 i18n matrix
--
-- three additive tables:
-- - `UserLabTelemetryLog`: the 24-hour telemetry log of 10 DNA cohorts — the
--   source of truth for the admin dashboard and the RLHF/SFT/DPO export;
-- - `LocalePatchRegistry`: the state of the 20 `Arda` ecosystem locales — the two
--   Day-1 active ones (`fa`/`en` at 10000 bps) + 18 hot-patch-ready slots;
-- - `NeuralHudRenderJob`: neural HUD video render runs (preset,
--   aspect ratio/resolution/frame rate, status).
--
-- no double-entry ledger table is touched — the `Posting` zero-sum stays untouched.

CREATE TABLE IF NOT EXISTS "UserLabTelemetryLog" (
    "id" TEXT PRIMARY KEY,
    "dna_code" TEXT NOT NULL,
    "user_handle" TEXT NOT NULL,
    "hour_offset" INTEGER NOT NULL,
    "ux_tier" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "grok_persona" TEXT NOT NULL,
    "model_slug" TEXT NOT NULL,
    "routing_mode" TEXT NOT NULL,
    "prompt_text" TEXT NOT NULL,
    "thinking_trace_json" TEXT NOT NULL,
    "response_text" TEXT NOT NULL,
    "rea_score" INTEGER NOT NULL,
    "latency_ms" INTEGER NOT NULL,
    "usable_for_training" INTEGER NOT NULL DEFAULT 1,
    "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS "idx_lab_telemetry_dna"
    ON "UserLabTelemetryLog" ("dna_code", "hour_offset");

CREATE INDEX IF NOT EXISTS "idx_lab_telemetry_model"
    ON "UserLabTelemetryLog" ("model_slug");

CREATE TABLE IF NOT EXISTS "LocalePatchRegistry" (
    "locale_code" TEXT PRIMARY KEY,
    "direction" TEXT NOT NULL,
    "activation_state" TEXT NOT NULL,
    "completion_bps" INTEGER NOT NULL,
    "patch_version" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS "NeuralHudRenderJob" (
    "id" TEXT PRIMARY KEY,
    "preset_slug" TEXT NOT NULL,
    "aspect_ratio" TEXT NOT NULL,
    "resolution" TEXT NOT NULL,
    "fps" INTEGER NOT NULL,
    "duration_sec" INTEGER NOT NULL,
    "status" TEXT NOT NULL,
    "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS "idx_hud_render_job_created"
    ON "NeuralHudRenderJob" ("created_at" DESC);
