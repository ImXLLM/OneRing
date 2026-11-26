//! Outbox — the bridge from domain events to later consumers (the phase 5 bot).
//!
//! The transactional outbox pattern: events are inserted in the same transaction as the
//! money-state change — "happened but unannounced" is impossible. Dispatch happens later by
//! reading `published = 0` and marking them done.

use crate::error::DbResult;
use crate::store::{insert_audit, insert_outbox_events, OutboxRow};
use kian_domain::DomainEvent;
use sqlx::Row;

/// Insert events into the outbox — called inside the current transaction.
///
/// # Errors
/// SQL error.
pub(crate) async fn enqueue(
    exec: &mut sqlx::SqliteConnection,
    events: &[DomainEvent],
) -> DbResult<()> {
    insert_outbox_events(exec, events).await
}

/// One audit row — called inside the current transaction.
///
/// # Errors
/// SQL error.
pub(crate) async fn audit(
    exec: &mut sqlx::SqliteConnection,
    actor: &str,
    action: &str,
    target: Option<&str>,
    meta: Option<&str>,
) -> DbResult<()> {
    insert_audit(exec, actor, action, target, meta).await
}

impl crate::store::V3Store {
    /// Events awaiting dispatch in insertion order (rowid — the real production order).
    ///
    /// # Errors
    /// SQL error.
    pub async fn pending_outbox(&self, limit: i64) -> DbResult<Vec<OutboxRow>> {
        let mut out = Vec::new();
        for r in sqlx::query(
            r#"SELECT "id", "kind", "payload", "createdAt", "published" FROM "OutboxEvent"
               WHERE "published" = 0 ORDER BY "rowid" ASC LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(self.pool_ref())
        .await?
        {
            out.push(OutboxRow {
                id: r.get("id"),
                kind: r.get("kind"),
                payload: r.get("payload"),
                created_at: r.get("createdAt"),
                published: r.get::<i64, _>("published") != 0,
            });
        }
        Ok(out)
    }

    /// Mark events as dispatched — returns the number of changed rows.
    ///
    /// # Errors
    /// SQL error.
    pub async fn mark_outbox_published(&self, ids: &[String]) -> DbResult<u64> {
        let mut n = 0;
        let mut tx = self.pool_ref().begin().await?;
        for id in ids {
            n += sqlx::query(r#"UPDATE "OutboxEvent" SET "published" = 1 WHERE "id" = ?"#)
                .bind(id)
                .execute(&mut *tx)
                .await?
                .rows_affected();
        }
        tx.commit().await?;
        Ok(n)
    }

    /// Standalone audit record (outside `apply`) — e.g. `legacy.adjustment`.
    ///
    /// # Errors
    /// SQL error.
    pub async fn audit(
        &self,
        actor: &str,
        action: &str,
        target: Option<&str>,
        meta: Option<&str>,
    ) -> DbResult<()> {
        let mut tx = self.pool_ref().begin().await?;
        insert_audit(&mut tx, actor, action, target, meta).await?;
        tx.commit().await?;
        Ok(())
    }
}
