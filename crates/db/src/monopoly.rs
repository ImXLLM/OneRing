//! Sovereign treasury — persistence of day-one atomic splits on the table
//! `sovereign_treasury_splits` (Wave 7 — Step 4/4).
//!
//! Three deterministic operations on migration `0011`: insert a zero-sum split row, read the
//! list and count the total — with audit tracking on every insert. No double-entry
//! ledger table is touched; this table is the split "bill", not the money itself.
//! An unbalanced row (`zero_sum_verified == false`) is never inserted.

use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::store::V3Store;

/// Sovereign treasury split row — the same columns as `sovereign_treasury_splits`.
#[derive(Debug, Clone, PartialEq)]
pub struct SovereignTreasurySplitRow {
    /// Row id (`split:<hash>`).
    pub id: String,
    /// Buyer id.
    pub buyer_user_id: String,
    /// Builder id.
    pub creator_user_id: String,
    /// Referrer id — `None` means a direct referral-free sale (the 2000 bps rule).
    pub referrer_user_id: Option<String>,
    /// Settlement asset (`base-native-usdc`, ...).
    pub settlement_asset: String,
    /// Gross amount in micro units.
    pub gross_amount_micros: i64,
    /// Sovereign treasury micros.
    pub kia_treasury_micros: i64,
    /// Referrer micros.
    pub referrer_micros: i64,
    /// Builder net.
    pub creator_net_micros: i64,
    /// Effective platform share in bps.
    pub effective_platform_bps: i64,
    /// Zero-sum proof — inserting `false` is forbidden.
    pub zero_sum_verified: bool,
    /// ISO timestamp.
    pub created_at: String,
}

impl SovereignTreasurySplitRow {
    /// Structural zero-sum recheck — the gate before insert.
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.kia_treasury_micros + self.referrer_micros + self.creator_net_micros
            == self.gross_amount_micros
    }
}

const SPLIT_COLS: &str = r#""id","buyer_user_id","creator_user_id","referrer_user_id",
           "settlement_asset","gross_amount_micros","kia_treasury_micros","referrer_micros",
           "creator_net_micros","effective_platform_bps","zero_sum_verified","created_at""#;

fn split_row_from(r: &sqlx::sqlite::SqliteRow) -> SovereignTreasurySplitRow {
    let referrer: Option<String> = r
        .try_get::<Option<String>, _>("referrer_user_id")
        .ok()
        .flatten()
        .filter(|s| !s.is_empty());
    SovereignTreasurySplitRow {
        id: r.get("id"),
        buyer_user_id: r.get("buyer_user_id"),
        creator_user_id: r.get("creator_user_id"),
        referrer_user_id: referrer,
        settlement_asset: r.get("settlement_asset"),
        gross_amount_micros: r.get("gross_amount_micros"),
        kia_treasury_micros: r.get("kia_treasury_micros"),
        referrer_micros: r.get("referrer_micros"),
        creator_net_micros: r.get("creator_net_micros"),
        effective_platform_bps: r.get("effective_platform_bps"),
        zero_sum_verified: r.get::<i64, _>("zero_sum_verified") != 0,
        created_at: r.get("created_at"),
    }
}

impl V3Store {
    /// Insert a split row — zero-sum rows only; deterministic and idempotent on `id`.
    ///
    /// # Errors
    /// SQL error or an audit error after the insert; an unbalanced row is rejected with
    /// [`DbError::Corrupt`].
    pub async fn create_sovereign_split(&self, row: &SovereignTreasurySplitRow) -> DbResult<()> {
        if !row.is_balanced() || !row.zero_sum_verified {
            return Err(DbError::Corrupt(
                "sovereign split rejected: zero-sum ledger invariant violated".to_owned(),
            ));
        }
        sqlx::query(
            r#"INSERT INTO "sovereign_treasury_splits"
               ("id","buyer_user_id","creator_user_id","referrer_user_id",
                "settlement_asset","gross_amount_micros","kia_treasury_micros","referrer_micros",
                "creator_net_micros","effective_platform_bps","zero_sum_verified","created_at")
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(&row.buyer_user_id)
        .bind(&row.creator_user_id)
        .bind(row.referrer_user_id.as_deref().filter(|s| !s.is_empty()))
        .bind(&row.settlement_asset)
        .bind(row.gross_amount_micros)
        .bind(row.kia_treasury_micros)
        .bind(row.referrer_micros)
        .bind(row.creator_net_micros)
        .bind(row.effective_platform_bps)
        .bind(i64::from(row.zero_sum_verified))
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext("server", "monopoly.simulate-split", Some(row.id.as_str()), None)
            .await?;
        Ok(())
    }

    /// Persisted split list — newest first, capped at `limit`.
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_sovereign_splits(&self, limit: u32) -> DbResult<Vec<SovereignTreasurySplitRow>> {
        let cap = i64::from(limit.clamp(1, 200));
        let sql = format!(
            r#"SELECT {SPLIT_COLS} FROM "sovereign_treasury_splits"
               ORDER BY "created_at" DESC, "id" DESC LIMIT ?"#
        );
        let rows = sqlx::query(crate::sqlsafe::audited(sql))
            .bind(cap)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.iter().map(split_row_from).collect())
    }

    /// Total split count — the handler test gate.
    ///
    /// # Errors
    /// SQL error.
    pub async fn count_sovereign_splits(&self) -> DbResult<i64> {
        let row = sqlx::query(crate::sqlsafe::audited(
            r#"SELECT COUNT(*) AS "c" FROM "sovereign_treasury_splits""#.to_owned(),
        ))
        .fetch_one(self.pool_ref())
        .await?;
        Ok(row.get::<i64, _>("c"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::V3Store;

    async fn fresh_store() -> V3Store {
        let s = V3Store::connect_in_memory().await.unwrap();
        s.migrate().await.unwrap();
        s
    }

    fn sample_row(id: &str, referred: bool) -> SovereignTreasurySplitRow {
        // Sale of 100.00 USDC: with referrer 10/10/80, without 20/0/80.
        let (treasury_leg, affiliate_leg, platform_bps) = if referred {
            (10_000_000i64, 10_000_000i64, 2_000i64)
        } else {
            (20_000_000i64, 0i64, 2_000i64)
        };
        SovereignTreasurySplitRow {
            id: id.to_owned(),
            buyer_user_id: "user:buyer-1".to_owned(),
            creator_user_id: "user:creator-9".to_owned(),
            referrer_user_id: referred.then(|| "user:ref-7".to_owned()),
            settlement_asset: "base-native-usdc".to_owned(),
            gross_amount_micros: 100_000_000,
            kia_treasury_micros: treasury_leg,
            referrer_micros: affiliate_leg,
            creator_net_micros: 80_000_000,
            effective_platform_bps: platform_bps,
            zero_sum_verified: true,
            created_at: "2026-11-25T00:00:00Z".to_owned(),
        }
    }

    #[tokio::test]
    async fn sovereign_split_roundtrip_and_list_order() {
        let store = fresh_store().await;
        assert_eq!(store.count_sovereign_splits().await.unwrap(), 0);
        store.create_sovereign_split(&sample_row("split:a", true)).await.unwrap();
        store.create_sovereign_split(&sample_row("split:b", false)).await.unwrap();
        assert_eq!(store.count_sovereign_splits().await.unwrap(), 2);
        let listed = store.list_sovereign_splits(10).await.unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, "split:b", "\u{646}\u{648}\u{62a}\u{631}\u{6cc}\u{646} \u{627}\u{648}\u{644}");
        // Direct referral-free sale rule: treasury 20%, referrer zero, builder 80%.
        let direct = &listed[0];
        assert!(direct.referrer_user_id.is_none());
        assert_eq!(direct.kia_treasury_micros, 20_000_000);
        assert_eq!(direct.creator_net_micros, 80_000_000);
        let referred = store
            .list_sovereign_splits(10)
            .await
            .unwrap()
            .into_iter()
            .find(|r| r.id == "split:a")
            .unwrap();
        assert_eq!(referred.referrer_user_id.as_deref(), Some("user:ref-7"));
        assert_eq!(referred.kia_treasury_micros, 10_000_000);
        assert!(referred.is_balanced());
    }

    #[tokio::test]
    async fn unbalanced_split_is_rejected() {
        let store = fresh_store().await;
        let mut bad = sample_row("split:bad", true);
        bad.creator_net_micros = 70_000_000; // sum = 90M != 100M
        let err = store.create_sovereign_split(&bad).await.unwrap_err();
        assert!(err.to_string().contains("zero-sum"), "{err}");
        assert_eq!(store.count_sovereign_splits().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn sovereign_list_respects_limit_cap() {
        let store = fresh_store().await;
        for i in 0..5 {
            let id = if i % 2 == 0 { format!("split:{i}-r") } else { format!("split:{i}") };
            let row = sample_row(&id, i % 2 == 0);
            store.create_sovereign_split(&row).await.unwrap();
        }
        let listed = store.list_sovereign_splits(3).await.unwrap();
        assert_eq!(listed.len(), 3);
    }
}
