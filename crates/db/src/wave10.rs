//! ماندگاریِ Wave 10 (Commit #30) — اتاق جنگِ بنیان‌گذار روی مهاجرتِ `0014`:
//!
//! - [`FounderWarRoomTickRow`]: درج/فهرستِ گام‌هایِ عملیاتیِ روزِ صفر؛
//! - [`ChairmanDirectiveLogRow`]: درج/فهرستِ دستورهایِ رئیس + پاسخِ سه شریک.
//!
//! هیچ جدولِ دفتریِ دوطرفه لمس نمی‌شود — این‌ها «دفترِ فرمان و تلمتری» است نه
//! خودِ پول؛ صفرجمعِ `Posting` دست‌نخورده می‌ماند.

use sqlx::Row;

use crate::error::DbResult;
use crate::store::V3Store;

/// ردیفِ گامِ عملیاتیِ اتاقِ جنگ — همان ستون‌هایِ `FounderWarRoomTick`.
#[derive(Debug, Clone, PartialEq)]
pub struct FounderWarRoomTickRow {
    /// شناسهٔ قطعی — `warroom:{tick:04}`.
    pub id: String,
    /// شمارهٔ تیک — `0..24`.
    pub tick_index: i64,
    /// ثانیهٔ شبیه‌سازی از `Day 0, 00:00:00`.
    pub simulation_second: i64,
    /// شناسهٔ سخنران — `elon_musk` | `cz_binance` | `jeff_bezos`.
    pub speaker_id: String,
    /// نامِ نمایشیِ سخنران.
    pub speaker_name: String,
    /// پیامِ فارسی.
    pub message_fa: String,
    /// پیامِ انگلیسی.
    pub message_en: String,
    /// فرمانِ ترمینال — در صورتِ اجرا.
    pub terminal_cmd: Option<String>,
    /// پستِ اجتماعی به JSON — در صورتِ انتشار.
    pub social_post_json: Option<String>,
    /// دلتایِ خزانه (`µUSDC`).
    pub wallet_delta_usdc_micros: i64,
    /// جمعِ ولت پس از تیک (`µUSDC`).
    pub wallet_total_usdc_micros: i64,
    /// مرحلهٔ فعالِ خطِ لوله.
    pub active_pipeline_stage: String,
    /// مُهرِ زمانِ ISO.
    pub created_at: String,
}

/// ردیفِ دستورِ رئیس — همان ستون‌هایِ `ChairmanDirectiveLog`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChairmanDirectiveLogRow {
    /// شناسهٔ قطعی — `directive:{ulid}`.
    pub id: String,
    /// متنِ خامِ دستور.
    pub directive_text: String,
    /// قصدِ طبقه‌بندی‌شده (ذخیره در ستونِ `target_founder` به‌عنوانِ برچسبِ
    /// هدفِ تاکتیکی — `intent:{intent_id}`).
    pub target_founder: String,
    /// پاسخِ ایلان — فارسی و انگلیسی جدا با `---`.
    pub elon_response: String,
    /// پاسخِ CZ.
    pub cz_response: String,
    /// پاسخِ جف.
    pub bezos_response: String,
    /// تیکِ فعالِ زمانِ تزریق.
    pub triggered_tick: i64,
    /// مُهرِ زمانِ ISO.
    pub created_at: String,
}

impl V3Store {
    /// درجِ یک گامِ عملیاتی — ایدمپوتنت روی `id`.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn insert_war_room_tick(&self, row: &FounderWarRoomTickRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "FounderWarRoomTick"
               ("id","tick_index","simulation_second","speaker_id","speaker_name",
                "message_fa","message_en","terminal_cmd","social_post_json",
                "wallet_delta_usdc_micros","wallet_total_usdc_micros",
                "active_pipeline_stage","created_at")
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)
               ON CONFLICT("id") DO NOTHING"#,
        )
        .bind(&row.id)
        .bind(row.tick_index)
        .bind(row.simulation_second)
        .bind(&row.speaker_id)
        .bind(&row.speaker_name)
        .bind(&row.message_fa)
        .bind(&row.message_en)
        .bind(&row.terminal_cmd)
        .bind(&row.social_post_json)
        .bind(row.wallet_delta_usdc_micros)
        .bind(row.wallet_total_usdc_micros)
        .bind(&row.active_pipeline_stage)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// فهرستِ گام‌ها به ترتیبِ تیک — سقفِ `limit` ردیف.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_war_room_ticks(&self, limit: u32) -> DbResult<Vec<FounderWarRoomTickRow>> {
        let cap = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"SELECT "id","tick_index","simulation_second","speaker_id","speaker_name",
                      "message_fa","message_en","terminal_cmd","social_post_json",
                      "wallet_delta_usdc_micros","wallet_total_usdc_micros",
                      "active_pipeline_stage","created_at"
               FROM "FounderWarRoomTick"
               ORDER BY "tick_index" ASC
               LIMIT ?"#,
        )
        .bind(cap)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows
            .iter()
            .map(|r| FounderWarRoomTickRow {
                id: r.get("id"),
                tick_index: r.get("tick_index"),
                simulation_second: r.get("simulation_second"),
                speaker_id: r.get("speaker_id"),
                speaker_name: r.get("speaker_name"),
                message_fa: r.get("message_fa"),
                message_en: r.get("message_en"),
                terminal_cmd: r.get("terminal_cmd"),
                social_post_json: r.get("social_post_json"),
                wallet_delta_usdc_micros: r.get("wallet_delta_usdc_micros"),
                wallet_total_usdc_micros: r.get("wallet_total_usdc_micros"),
                active_pipeline_stage: r.get("active_pipeline_stage"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    /// درجِ دستورِ رئیس — ایدمپوتنت روی `id`.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn insert_chairman_directive(
        &self,
        row: &ChairmanDirectiveLogRow,
    ) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "ChairmanDirectiveLog"
               ("id","directive_text","target_founder","elon_response",
                "cz_response","bezos_response","triggered_tick","created_at")
               VALUES (?,?,?,?,?,?,?,?)
               ON CONFLICT("id") DO NOTHING"#,
        )
        .bind(&row.id)
        .bind(&row.directive_text)
        .bind(&row.target_founder)
        .bind(&row.elon_response)
        .bind(&row.cz_response)
        .bind(&row.bezos_response)
        .bind(row.triggered_tick)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// فهرستِ دستورهایِ رئیس — تازه‌ترین‌ها اول، سقفِ `limit` ردیف.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_chairman_directives(
        &self,
        limit: u32,
    ) -> DbResult<Vec<ChairmanDirectiveLogRow>> {
        let cap = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"SELECT "id","directive_text","target_founder","elon_response",
                      "cz_response","bezos_response","triggered_tick","created_at"
               FROM "ChairmanDirectiveLog"
               ORDER BY "created_at" DESC, "id" DESC
               LIMIT ?"#,
        )
        .bind(cap)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows
            .iter()
            .map(|r| ChairmanDirectiveLogRow {
                id: r.get("id"),
                directive_text: r.get("directive_text"),
                target_founder: r.get("target_founder"),
                elon_response: r.get("elon_response"),
                cz_response: r.get("cz_response"),
                bezos_response: r.get("bezos_response"),
                triggered_tick: r.get("triggered_tick"),
                created_at: r.get("created_at"),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// درج و فهرستِ کاملِ گام‌ها — ایدمپوتنت و مرتب.
    #[tokio::test]
    async fn war_room_ticks_roundtrip_idempotent() {
        let store = V3Store::connect_in_memory().await.expect("store");
        store.migrate().await.expect("migrate");
        let row = FounderWarRoomTickRow {
            id: "warroom:0008".to_owned(),
            tick_index: 8,
            simulation_second: 28_800,
            speaker_id: "jeff_bezos".to_owned(),
            speaker_name: "Jeff Bezos".to_owned(),
            message_fa: "اولین فروش واقعی".to_owned(),
            message_en: "first real sale".to_owned(),
            terminal_cmd: None,
            social_post_json: None,
            wallet_delta_usdc_micros: 600_000,
            wallet_total_usdc_micros: 33_600_000,
            active_pipeline_stage: "revenue_flow".to_owned(),
            created_at: "2026-11-25T08:00:00.000Z".to_owned(),
        };
        store.insert_war_room_tick(&row).await.expect("insert");
        store.insert_war_room_tick(&row).await.expect("idempotent insert");
        let rows = store.list_war_room_ticks(100).await.expect("list");
        assert_eq!(rows.len(), 1, "ON CONFLICT DO NOTHING keeps one row");
        assert_eq!(rows[0], row);
    }

    /// دستورِ رئیس — درج، مرتب‌سازیِ نزولی و گردشِ کامل.
    #[tokio::test]
    async fn chairman_directives_roundtrip_newest_first() {
        let store = V3Store::connect_in_memory().await.expect("store");
        store.migrate().await.expect("migrate");
        for (id, created) in [
            ("directive:a", "2026-11-25T09:00:00.000Z"),
            ("directive:b", "2026-11-25T10:00:00.000Z"),
        ] {
            store
                .insert_chairman_directive(&ChairmanDirectiveLogRow {
                    id: id.to_owned(),
                    directive_text: "گزارش ولت".to_owned(),
                    target_founder: "intent:status_report".to_owned(),
                    elon_response: "FA---EN".to_owned(),
                    cz_response: "FA---EN".to_owned(),
                    bezos_response: "FA---EN".to_owned(),
                    triggered_tick: 3,
                    created_at: created.to_owned(),
                })
                .await
                .expect("insert");
        }
        let rows = store.list_chairman_directives(10).await.expect("list");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "directive:b", "newest first");
        assert_eq!(rows[0].target_founder, "intent:status_report");
    }
}
