//! استودیوی ادراک‌سیار — ماندگاریِ آفرینش‌های `studio_creations` (Wave 5 —
//! Step 2/4).
//!
//! سه عملیاتِ قطعی روی مهاجرت `0009`: درجِ آفرینش، خوانشِ تک‌ردیفی و
//! شمارشِ کل — با ردیابیِ audit روی هر درج. هیچ جدولِ دفتری لمس نمی‌شود.

use sqlx::Row;

use crate::error::DbResult;
use crate::store::V3Store;

/// ردیفِ آفرینش استودیو — همان ستون‌های `studio_creations` (مهاجرت 0009).
#[derive(Debug, Clone, PartialEq)]
pub struct StudioCreationRow {
    /// شناسهٔ آفرینش (`studio:<uuid>`).
    pub id: String,
    /// شناسهٔ حسابِ سازنده.
    pub creator_user_id: String,
    /// لایهٔ شناختیِ مورف‌شده (`ToySpark` | `CreatorStudio` | `ProTerminal`).
    pub cognitive_tier: String,
    /// مدالیتیِ مولد (`search` … `sfx`).
    pub primary_modality: String,
    /// پرامپتِ خامِ سازنده.
    pub prompt_text: String,
    /// عنوانِ قطعی.
    pub title: String,
    /// کارتِ ویترینِ JSON (`StorefrontProjectCard`).
    pub storefront_card_json: String,
    /// نمرهٔ REA (`88..=97`).
    pub rea_score: i64,
    /// قیمت به میکرو-کیان.
    pub price_ukian: i64,
    /// قیمت به ستارهٔ تلگرام.
    pub price_xtr_stars: i64,
    /// مُهرِ زمانِ تولید (ISO-8601).
    pub created_at: String,
}

impl V3Store {
    /// درجِ آفرینش استودیو — قطعی و ایدمپوتنت روی `id`.
    ///
    /// # Errors
    /// خطای SQL یا خطای audit پس از درج.
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

    /// خوانشِ تک‌ردیفیِ آفرینش — ناشناخته → `None`.
    ///
    /// # Errors
    /// خطای SQL.
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

    /// شمارشِ کلِ آفرینش‌ها — گیتِ آزمونِ هندلر.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn count_studio_creations(&self) -> DbResult<i64> {
        let row =
            sqlx::query(r#"SELECT COUNT(*) AS "n" FROM "studio_creations""#)
                .fetch_one(self.pool_ref())
                .await?;
        Ok(row.get::<i64, _>("n"))
    }
}

/// نگاشتِ ردیفِ SQL به ردیفِ دامنه — همان قراردادِ `users.rs`/`nation.rs`.
#[allow(clippy::needless_pass_by_value)] // الگوی sqlx: مالکیت Row از نتیجهٔ fetch می‌آید و مصرف می‌شود
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
            prompt_text: "یه دایناسور آبی که توی فضا بستنی می‌فروشه".to_owned(),
            title: "ماجرای یک‌کلیکی".to_owned(),
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
