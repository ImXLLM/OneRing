//! Mobile-perception studio — persistence of `studio_creations` (Wave 5 —
//! Step 2/4).
//!
//! Three deterministic operations on migration `0009`: insert a creation, single-row read, and
//! total count — with audit tracing on every insert. No ledger table is touched.

use sqlx::Row;

use crate::error::DbResult;
use crate::store::V3Store;

/// Studio creation row — the exact columns of `studio_creations` (migration 0009).
#[derive(Debug, Clone, PartialEq)]
pub struct StudioCreationRow {
    /// Creation ID (`studio:<uuid>`).
    pub id: String,
    /// Creator account ID.
    pub creator_user_id: String,
    /// Morphed cognitive layer (`ToySpark` | `CreatorStudio` | `ProTerminal`).
    pub cognitive_tier: String,
    /// Generative modality (`search` … `sfx`).
    pub primary_modality: String,
    /// Creator's raw prompt.
    pub prompt_text: String,
    /// Deterministic title.
    pub title: String,
    /// JSON storefront card (`StorefrontProjectCard`).
    pub storefront_card_json: String,
    /// REA score (`88..=97`).
    pub rea_score: i64,
    /// Price in micro-Kian.
    pub price_ukian: i64,
    /// Price in Telegram Stars.
    pub price_xtr_stars: i64,
    /// Generation timestamp (ISO-8601).
    pub created_at: String,
}

impl V3Store {
    /// Insert a studio creation — deterministic and idempotent on `id`.
    ///
    /// # Errors
    /// SQL error or audit error after the insert.
    pub async fn create_studio_creation(&self, row: &StudioCreationRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "studio_creations"
               ("id","creator_user_id","cognitive_tier","primary_modality","prompt_text",
                "title","storefront_card_json","rea_score","price_ukian","price_xtr_stars","created_at")
               VALUES (?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(&row.creator_user_id)
        .bind(&row.cognitive_tier)
        .bind(&row.primary_modality)
        .bind(&row.prompt_text)
        .bind(&row.title)
        .bind(&row.storefront_card_json)
        .bind(row.rea_score)
        .bind(row.price_ukian)
        .bind(row.price_xtr_stars)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext("server", "studio.generate", Some(row.id.as_str()), None)
            .await?;
        Ok(())
    }

    /// Single-row creation read — unknown → `None`.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_studio_creation(&self, id: &str) -> DbResult<Option<StudioCreationRow>> {
        let row = sqlx::query(
            r#"SELECT "id","creator_user_id","cognitive_tier","primary_modality","prompt_text",
                      "title","storefront_card_json","rea_score","price_ukian","price_xtr_stars","created_at"
               FROM "studio_creations" WHERE "id"=?"#,
        )
        .bind(id)
        .fetch_optional(self.pool_ref())
        .await?;
        Ok(row.map(studio_row_from))
    }

    /// Total creation count — the handler test gate.
    ///
    /// # Errors
    /// SQL error.
    pub async fn count_studio_creations(&self) -> DbResult<i64> {
        let row =
            sqlx::query(r#"SELECT COUNT(*) AS "n" FROM "studio_creations""#)
                .fetch_one(self.pool_ref())
                .await?;
        Ok(row.get::<i64, _>("n"))
    }
}

/// Map an SQL row to a domain row — same contract as `users.rs`/`nation.rs`.
#[allow(clippy::needless_pass_by_value)] // sqlx pattern: Row ownership comes from the fetch result and is consumed
fn studio_row_from(row: sqlx::sqlite::SqliteRow) -> StudioCreationRow {
    StudioCreationRow {
        id: row.get("id"),
        creator_user_id: row.get("creator_user_id"),
        cognitive_tier: row.get("cognitive_tier"),
        primary_modality: row.get("primary_modality"),
        prompt_text: row.get("prompt_text"),
        title: row.get("title"),
        storefront_card_json: row.get("storefront_card_json"),
        rea_score: row.get("rea_score"),
        price_ukian: row.get("price_ukian"),
        price_xtr_stars: row.get("price_xtr_stars"),
        created_at: row.get("created_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> V3Store {
        let s = V3Store::connect_in_memory().await.expect("connect");
        s.migrate().await.expect("migrate");
        s
    }

    fn sample(id: &str) -> StudioCreationRow {
        StudioCreationRow {
            id: id.to_owned(),
            creator_user_id: "user:tg:777777".to_owned(),
            cognitive_tier: "ToySpark".to_owned(),
            primary_modality: "workspace".to_owned(),
            prompt_text: "\u{6cc}\u{647} \u{62f}\u{627}\u{6cc}\u{646}\u{627}\u{633}\u{648}\u{631} \u{622}\u{628}\u{6cc} \u{6a9}\u{647} \u{62a}\u{648}\u{6cc} \u{641}\u{636}\u{627} \u{628}\u{633}\u{62a}\u{646}\u{6cc} \u{645}\u{6cc}‌\u{641}\u{631}\u{648}\u{634}\u{647}".to_owned(),
            title: "\u{645}\u{627}\u{62c}\u{631}\u{627}\u{6cc} \u{6cc}\u{6a9}‌\u{6a9}\u{644}\u{6cc}\u{6a9}\u{6cc}".to_owned(),
            storefront_card_json: "{\"slug\":\"toyspark-1\"}".to_owned(),
            rea_score: 92,
            price_ukian: 1_000_000,
            price_xtr_stars: 50,
            created_at: "2026-11-25T00:00:00Z".to_owned(),
        }
    }

    #[tokio::test]
    async fn studio_creation_roundtrip_and_count() {
        let s = store().await;
        assert_eq!(s.count_studio_creations().await.unwrap(), 0);
        s.create_studio_creation(&sample("studio:test-1")).await.unwrap();
        s.create_studio_creation(&sample("studio:test-2")).await.unwrap();
        assert_eq!(s.count_studio_creations().await.unwrap(), 2);
        let got = s.get_studio_creation("studio:test-1").await.unwrap().unwrap();
        assert_eq!(got.cognitive_tier, "ToySpark");
        assert_eq!(got.price_xtr_stars, 50);
        assert_eq!(got.rea_score, 92);
        assert!(s.get_studio_creation("studio:missing").await.unwrap().is_none());
    }
}
