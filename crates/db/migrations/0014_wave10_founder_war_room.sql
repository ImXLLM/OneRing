-- مهاجرت 0014 — Wave 10 (Commit #30): اتاق جنگِ بنیان‌گذارِ حاکم +
-- موتورِ شبیه‌سازیِ واقعیِ زمانیِ «روزِ ۰ → روزِ ۱»
--
-- دو جدولِ additive:
-- - `FounderWarRoomTick`: هر گامِ عملیاتیِ ۲۴تاییِ روزِ صفر — سخنران، پیامِ
--   دوزبانه (فارسی/انگلیسی)، فرمانِ ترمینال، پستِ اجتماعی، دلتا و جمعِ
--   ولتِ سازمان (µUSDC)، مرحلهٔ خطِ لوله و جریانِ درآمدِ فعال؛
-- - `ChairmanDirectiveLog`: هر دستورِ تاکتیکیِ رئیسِ `0xKIA` + پاسخِ
--   در-شخصیتِ سه هم‌بنیان‌گذار (ایلان/CZ/جف) به هر دو زبان.
--
-- هیچ جدولِ دفتریِ دوطرفه‌ای لمس نمی‌شود — صفرجمعِ `Posting` دست‌نخورده می‌ماند.

CREATE TABLE IF NOT EXISTS "FounderWarRoomTick" (
    "id" TEXT PRIMARY KEY,
    "tick_index" INTEGER NOT NULL,
    "simulation_second" INTEGER NOT NULL,
    "speaker_id" TEXT NOT NULL,
    "speaker_name" TEXT NOT NULL,
    "message_fa" TEXT NOT NULL,
    "message_en" TEXT NOT NULL,
    "terminal_cmd" TEXT,
    "social_post_json" TEXT,
    "wallet_delta_usdc_micros" INTEGER NOT NULL DEFAULT 0,
    "wallet_total_usdc_micros" INTEGER NOT NULL DEFAULT 0,
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
