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

/// Live war-room state row — the `WarRoomState` columns (migration `0014`).
#[derive(Debug, Clone, PartialEq)]
pub struct WarRoomStateRow {
    /// Named state id — `warroom:live` is the canonical singleton.
    pub id: String,
    /// Last executed tick — `0..24`.
    pub tick_index: i64,
    /// Simulated elapsed seconds from `Day 0, 00:00:00`.
    pub elapsed_seconds: i64,
    /// Replay cursor — `0..24`.
    pub tick_cursor: i64,
    /// Sovereign wallet total after the last tick (`µUSDC`, never negative).
    pub wallet_total_usdc_micros: i64,
    /// Net platform revenue so far (`µUSDC`).
    pub earned_revenue_micros: i64,
    /// Creator payouts so far (`µUSDC`).
    pub creator_payouts_micros: i64,
    /// Affiliate payouts so far (`µUSDC`).
    pub affiliate_payouts_micros: i64,
    /// Active users of the day.
    pub active_users: i64,
    /// Active locale — `en` canonical; `fa` arrives only via the patch file.
    pub locale: String,
    /// ISO timestamp of the last upsert.
    pub updated_at: String,
}

/// Append-only causal tick-event row — the `WarRoomTickEvent` columns
/// (migration `0014`); every row carries the wallet total after the tick so
/// the ledger stays replayable.
#[derive(Debug, Clone, PartialEq)]
pub struct WarRoomTickEventRow {
    /// Deterministic id — `warroom-event:{tick:04}`.
    pub id: String,
    /// Tick number — `0..24`.
    pub tick_index: i64,
    /// Simulation seconds from `Day 0, 00:00:00`.
    pub simulation_second: i64,
    /// Speaker wire id — `elon_musk` | `cz_binance` | `jeff_bezos`.
    pub speaker_id: String,
    /// Revenue stream wire id active in this tick.
    pub revenue_stream: String,
    /// Treasury delta in this tick (`µUSDC`).
    pub wallet_delta_usdc_micros: i64,
    /// Wallet total after this tick (`µUSDC`, never negative).
    pub wallet_total_usdc_micros: i64,
    /// English message — the canonical International English string.
    pub message_en: String,
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

    /// Admin war-room reset to day zero — clears all recorded ticks,
    /// the chairman-directive log, the append-only `WarRoomTickEvent` ledger and the
    /// live `WarRoomState` singleton in one transaction; `POST /war-room/step` afterwards
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
        // Gate 5 wiring — when the named war-room tables are present (migration 0014),
        // the causal tick-event ledger and the live state singleton reset with them.
        sqlx::query(r#"DELETE FROM "WarRoomTickEvent""#)
            .execute(&mut *tx)
            .await?;
        sqlx::query(r#"DELETE FROM "WarRoomState""#)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok((ticks, directives))
    }

    /// Upsert the live war-room state singleton (`WarRoomState`) — one row per
    /// named state (`warroom:live` is the canonical id). Tick boundaries and the
    /// non-negative wallet guard are enforced by the schema-level CHECKs.
    ///
    /// # Errors
    /// SQL error.
    pub async fn upsert_war_room_state(&self, row: &WarRoomStateRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "WarRoomState"
               ("id","tick_index","elapsed_seconds","tick_cursor","wallet_total_usdc_micros",
                "earned_revenue_micros","creator_payouts_micros","affiliate_payouts_micros",
                "active_users","locale","updated_at")
               VALUES (?,?,?,?,?,?,?,?,?,?,?)
               ON CONFLICT("id") DO UPDATE SET
                 "tick_index" = excluded."tick_index",
                 "elapsed_seconds" = excluded."elapsed_seconds",
                 "tick_cursor" = excluded."tick_cursor",
                 "wallet_total_usdc_micros" = excluded."wallet_total_usdc_micros",
                 "earned_revenue_micros" = excluded."earned_revenue_micros",
                 "creator_payouts_micros" = excluded."creator_payouts_micros",
                 "affiliate_payouts_micros" = excluded."affiliate_payouts_micros",
                 "active_users" = excluded."active_users",
                 "locale" = excluded."locale",
                 "updated_at" = excluded."updated_at""#,
        )
        .bind(&row.id)
        .bind(row.tick_index)
        .bind(row.elapsed_seconds)
        .bind(row.tick_cursor)
        .bind(row.wallet_total_usdc_micros)
        .bind(row.earned_revenue_micros)
        .bind(row.creator_payouts_micros)
        .bind(row.affiliate_payouts_micros)
        .bind(row.active_users)
        .bind(&row.locale)
        .bind(&row.updated_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// Read the live war-room state singleton by id — `None` before the first
    /// step/reset ever touched the named table (pure day zero).
    ///
    /// # Errors
    /// SQL error.
    pub async fn read_war_room_state(&self, id: &str) -> DbResult<Option<WarRoomStateRow>> {
        let row = sqlx::query(
            r#"SELECT "id","tick_index","elapsed_seconds","tick_cursor","wallet_total_usdc_micros",
                      "earned_revenue_micros","creator_payouts_micros","affiliate_payouts_micros",
                      "active_users","locale","updated_at"
               FROM "WarRoomState"
               WHERE "id" = ?"#,
        )
        .bind(id)
        .fetch_optional(self.pool_ref())
        .await?;
        Ok(row.map(|r| WarRoomStateRow {
            id: r.get("id"),
            tick_index: r.get("tick_index"),
            elapsed_seconds: r.get("elapsed_seconds"),
            tick_cursor: r.get("tick_cursor"),
            wallet_total_usdc_micros: r.get("wallet_total_usdc_micros"),
            earned_revenue_micros: r.get("earned_revenue_micros"),
            creator_payouts_micros: r.get("creator_payouts_micros"),
            affiliate_payouts_micros: r.get("affiliate_payouts_micros"),
            active_users: r.get("active_users"),
            locale: r.get("locale"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// Record one causal tick event into the append-only `WarRoomTickEvent`
    /// ledger — idempotent on `id` (the same tick replayed never double-writes).
    ///
    /// # Errors
    /// SQL error.
    pub async fn record_war_room_tick_event(&self, row: &WarRoomTickEventRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "WarRoomTickEvent"
               ("id","tick_index","simulation_second","speaker_id","revenue_stream",
                "wallet_delta_usdc_micros","wallet_total_usdc_micros","message_en","created_at")
               VALUES (?,?,?,?,?,?,?,?,?)
               ON CONFLICT("id") DO NOTHING"#,
        )
        .bind(&row.id)
        .bind(row.tick_index)
        .bind(row.simulation_second)
        .bind(&row.speaker_id)
        .bind(&row.revenue_stream)
        .bind(row.wallet_delta_usdc_micros)
        .bind(row.wallet_total_usdc_micros)
        .bind(&row.message_en)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// List the causal tick-event ledger in tick order — capped at `limit` rows.
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_war_room_tick_events(&self, limit: u32) -> DbResult<Vec<WarRoomTickEventRow>> {
        let cap = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"SELECT "id","tick_index","simulation_second","speaker_id","revenue_stream",
                      "wallet_delta_usdc_micros","wallet_total_usdc_micros","message_en","created_at"
               FROM "WarRoomTickEvent"
               ORDER BY "tick_index" ASC
               LIMIT ?"#,
        )
        .bind(cap)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows
            .iter()
            .map(|r| WarRoomTickEventRow {
                id: r.get("id"),
                tick_index: r.get("tick_index"),
                simulation_second: r.get("simulation_second"),
                speaker_id: r.get("speaker_id"),
                revenue_stream: r.get("revenue_stream"),
                wallet_delta_usdc_micros: r.get("wallet_delta_usdc_micros"),
                wallet_total_usdc_micros: r.get("wallet_total_usdc_micros"),
                message_en: r.get("message_en"),
                created_at: r.get("created_at"),
            })
            .collect())
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
            message_fa: "[FA-PATCH-SLOT] the first real sale".to_owned(),
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
                    directive_text: "[FA-PATCH-SLOT] wallet report".to_owned(),
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

    /// Gate 5 — the named `WarRoomState` singleton round-trips via upsert and
    /// the append-only `WarRoomTickEvent` ledger stays idempotent; reset clears
    /// all four war-room tables in one transaction.
    #[tokio::test]
    async fn war_room_named_state_and_tick_event_ledger_roundtrip() {
        let store = V3Store::connect_in_memory().await.expect("store");
        store.migrate().await.expect("migrate");

        // Before the first step, the live state is absent — pure day zero.
        let fresh = store.read_war_room_state("warroom:live").await.expect("read");
        assert!(fresh.is_none(), "no state row before the first step");

        let state_row = WarRoomStateRow {
            id: "warroom:live".to_owned(),
            tick_index: 1,
            elapsed_seconds: 28_800,
            tick_cursor: 2,
            wallet_total_usdc_micros: 33_600_000,
            earned_revenue_micros: 600_000,
            creator_payouts_micros: 2_400_000,
            affiliate_payouts_micros: 0,
            active_users: 1_527,
            locale: "en".to_owned(),
            updated_at: "2026-11-28T08:00:00.000Z".to_owned(),
        };
        store.upsert_war_room_state(&state_row).await.expect("upsert");
        store.upsert_war_room_state(&state_row).await.expect("idempotent upsert");
        let live = store
            .read_war_room_state("warroom:live")
            .await
            .expect("read")
            .expect("state row after upsert");
        assert_eq!(live, state_row, "the singleton must round-trip exactly");

        let event = WarRoomTickEventRow {
            id: "warroom-event:0001".to_owned(),
            tick_index: 1,
            simulation_second: 28_800,
            speaker_id: "cz_binance".to_owned(),
            revenue_stream: "marketplace_tax".to_owned(),
            wallet_delta_usdc_micros: 600_000,
            wallet_total_usdc_micros: 33_600_000,
            message_en: "full workspace suite: 826 passed, zero failures, zero clippy".to_owned(),
            created_at: "2026-11-28T08:00:00.000Z".to_owned(),
        };
        store.record_war_room_tick_event(&event).await.expect("record");
        store.record_war_room_tick_event(&event).await.expect("idempotent record");
        let events = store.list_war_room_tick_events(100).await.expect("list");
        assert_eq!(events.len(), 1, "ON CONFLICT DO NOTHING keeps one ledger row");
        assert_eq!(events[0], event);

        // Schema-level guards: out-of-range ticks and negative wallet totals are rejected.
        let bad_tick = WarRoomTickEventRow {
            id: "warroom-event:0024".to_owned(),
            tick_index: 24,
            simulation_second: 86_400,
            speaker_id: "cz_binance".to_owned(),
            revenue_stream: "none".to_owned(),
            wallet_delta_usdc_micros: 0,
            wallet_total_usdc_micros: 33_600_000,
            message_en: "out of the canonical 24-tick window".to_owned(),
            created_at: "2026-11-28T08:01:00.000Z".to_owned(),
        };
        assert!(
            store.record_war_room_tick_event(&bad_tick).await.is_err(),
            "tick_index 24 must violate the CHECK (tick_index BETWEEN 0 AND 23)"
        );
        let mut bad_wallet = event.clone();
        bad_wallet.id = "warroom-event:0002".to_owned();
        bad_wallet.wallet_total_usdc_micros = -1;
        assert!(
            store.record_war_room_tick_event(&bad_wallet).await.is_err(),
            "negative wallet totals must violate the CHECK (wallet_total_usdc_micros >= 0)"
        );

        // Reset — the causal ledger and the live singleton clear with the ticks.
        let (ticks, directives) = store.reset_war_room_state().await.expect("reset");
        assert_eq!((ticks, directives), (0, 0), "no persisted rows in this fixture");
        assert!(
            store.read_war_room_state("warroom:live").await.expect("read").is_none(),
            "reset must delete the WarRoomState singleton"
        );
        let events = store.list_war_room_tick_events(100).await.expect("list");
        assert!(events.is_empty(), "reset must delete the WarRoomTickEvent ledger");
    }

}
