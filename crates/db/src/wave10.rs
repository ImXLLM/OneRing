//! Wave 10 persistence (Commit #30) — the founder war room on migration `0014`:
//!
//! - [`FounderWarRoomTickRow`]: insert/list day-zero operational steps;
//! - [`ChairmanDirectiveLogRow`]: insert/list chairman directives + the three partners' replies.
//!
//! No double-entry ledger table is touched — these are the "command and telemetry ledger",
//! not money itself; the `Posting` zero-sum stays intact.

use sqlx::Row;

use crate::error::DbResult;
use crate::store::V3Store;

/// War-room operational step row — the `FounderWarRoomTick` columns.
#[derive(Debug, Clone, PartialEq)]
pub struct FounderWarRoomTickRow {
    /// Deterministic ID — `warroom:{tick:04}`.
    pub id: String,
    /// Tick number — `0..24`.
    pub tick_index: i64,
    /// Simulation seconds from `Day 0, 00:00:00`.
    pub simulation_second: i64,
    /// Speaker ID — `elon_musk` | `cz_binance` | `jeff_bezos`.
    pub speaker_id: String,
    /// Speaker display name.
    pub speaker_name: String,
    /// Persian message.
    pub message_fa: String,
    /// English message.
    pub message_en: String,
    /// Terminal command — when executed.
    pub terminal_cmd: Option<String>,
    /// Social post as JSON — when published.
    pub social_post_json: Option<String>,
    /// Treasury delta (`µUSDC`).
    pub wallet_delta_usdc_micros: i64,
    /// Wallet total after the tick (`µUSDC`).
    pub wallet_total_usdc_micros: i64,
    /// Active pipeline stage.
    pub active_pipeline_stage: String,
    /// ISO timestamp.
    pub created_at: String,
}

/// Chairman directive row — the `ChairmanDirectiveLog` columns.
#[derive(Debug, Clone, PartialEq)]
pub struct ChairmanDirectiveLogRow {
    /// Deterministic ID — `directive:{ulid}`.
    pub id: String,
    /// Raw directive text.
    pub directive_text: String,
    /// Classified intent (stored in the `target_founder` column as the
    /// tactical-target label — `intent:{intent_id}`).
    pub target_founder: String,
    /// Elon's reply — Persian and English separated by `---`.
    pub elon_response: String,
    /// CZ's reply.
    pub cz_response: String,
    /// Jeff's reply.
    pub bezos_response: String,
    /// Active tick at injection time.
    pub triggered_tick: i64,
    /// ISO timestamp.
    pub created_at: String,
}

impl V3Store {
    /// Insert one operational step — idempotent on `id`.
    ///
    /// # Errors
    /// SQL error.
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

    /// List the steps in tick order — capped at `limit` rows.
    ///
    /// # Errors
    /// SQL error.
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

    /// Insert a chairman directive — idempotent on `id`.
    ///
    /// # Errors
    /// SQL error.
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

    /// List chairman directives — newest first, capped at `limit` rows.
    ///
    /// # Errors
    /// SQL error.
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

    /// Admin war-room reset to day zero — clears all recorded ticks
    /// and the chairman-directive log in one transaction; `POST /war-room/step` afterwards
    /// can be replayed without hitting scenario exhaustion.
    ///
    /// # Errors
    /// SQL error.
    pub async fn reset_war_room_state(&self) -> DbResult<(u64, u64)> {
        let mut tx = self.pool_ref().begin().await?;
        let ticks = sqlx::query(r#"DELETE FROM "FounderWarRoomTick""#)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        let directives = sqlx::query(r#"DELETE FROM "ChairmanDirectiveLog""#)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        tx.commit().await?;
        Ok((ticks, directives))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Insert and fully list the steps — idempotent and ordered.
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
            message_fa: "\u{627}\u{648}\u{644}\u{6cc}\u{646} \u{641}\u{631}\u{648}\u{634} \u{648}\u{627}\u{642}\u{639}\u{6cc}".to_owned(),
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

    /// Chairman directive — insert, descending sort, and a full round trip.
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
                    directive_text: "\u{6af}\u{632}\u{627}\u{631}\u{634} \u{648}\u{644}\u{62a}".to_owned(),
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
