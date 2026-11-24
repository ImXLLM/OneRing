//! موتور کوانتِ علّی — ماندگاریِ استراتژی‌های `quant_strategies`
//! (Wave 6 — Step 3/4).
//!
//! سه عملیاتِ قطعی روی مهاجرت `0010`: درجِ استراتژیِ سبزِ گیتِ مایرون،
//! خوانشِ فهرست و شمارشِ کل — با ردیابیِ audit روی هر درج. هیچ جدولِ
//! دفتری لمس نمی‌شود.

use sqlx::Row;

use crate::error::DbResult;
use crate::store::V3Store;

/// ردیفِ استراتژی کوانت — همان ستون‌های `quant_strategies` (مهاجرت 0010).
/// شناسهٔ استراتژی (`vibe:<hash>`) درونِ `strategy_ast_json` زندگی می‌کند.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantStrategyRow {
    /// شناسهٔ ردیف (`quant:<uuid>`).
    pub id: String,
    /// شناسهٔ حسابِ سازنده.
    pub creator_user_id: String,
    /// نامِ انسانی.
    pub name: String,
    /// نماد (`BTC-USDT`).
    pub symbol: String,
    /// تایم‌فریم (`1h`).
    pub timeframe: String,
    /// پرامپتِ خامِ Vibe.
    pub vibe_prompt: String,
    /// AST تایپ‌دار به JSON.
    pub strategy_ast_json: String,
    /// سنجه‌هایِ نهادی به JSON.
    pub metrics_json: String,
    /// شارپِ سالانه‌شده.
    pub sharpe_ratio: f64,
    /// بیشینهٔ دراودان (درصد).
    pub max_drawdown_pct: f64,
    /// مُهرِ گیتِ مایرون.
    pub mairon_approved: bool,
    /// هشِ دفترِ ادعاها.
    pub claim_verification_hash: String,
    /// `SKILL.md` فورج‌شده.
    pub skill_md: String,
    /// مُهرِ زمانِ تولید (ISO-8601).
    pub created_at: String,
}

fn quant_row_from(r: &sqlx::sqlite::SqliteRow) -> QuantStrategyRow {
    QuantStrategyRow {
        id: r.get("id"),
        creator_user_id: r.get("creator_user_id"),
        name: r.get("name"),
        symbol: r.get("symbol"),
        timeframe: r.get("timeframe"),
        vibe_prompt: r.get("vibe_prompt"),
        strategy_ast_json: r.get("strategy_ast_json"),
        metrics_json: r.get("metrics_json"),
        sharpe_ratio: r.get::<f64, _>("sharpe_ratio"),
        max_drawdown_pct: r.get::<f64, _>("max_drawdown_pct"),
        mairon_approved: r.get::<i64, _>("mairon_approved") != 0,
        claim_verification_hash: r.get("claim_verification_hash"),
        skill_md: r.get("skill_md"),
        created_at: r.get("created_at"),
    }
}

const QUANT_COLS: &str = r#""id","creator_user_id","name","symbol","timeframe",
           "vibe_prompt","strategy_ast_json","metrics_json","sharpe_ratio","max_drawdown_pct",
           "mairon_approved","claim_verification_hash","skill_md","created_at""#;

impl V3Store {
    /// درجِ استراتژی کوانت — فقط خروجیِ سبزِ گیتِ مایرون؛ قطعی و
    /// ایدمپوتنت روی `id`.
    ///
    /// # Errors
    /// خطای SQL یا خطای audit پس از درج.
    pub async fn create_quant_strategy(&self, row: &QuantStrategyRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "quant_strategies"
               ("id","creator_user_id","name","symbol","timeframe",
                "vibe_prompt","strategy_ast_json","metrics_json","sharpe_ratio","max_drawdown_pct",
                "mairon_approved","claim_verification_hash","skill_md","created_at")
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(&row.creator_user_id)
        .bind(&row.name)
        .bind(&row.symbol)
        .bind(&row.timeframe)
        .bind(&row.vibe_prompt)
        .bind(&row.strategy_ast_json)
        .bind(&row.metrics_json)
        .bind(row.sharpe_ratio)
        .bind(row.max_drawdown_pct)
        .bind(i64::from(row.mairon_approved))
        .bind(&row.claim_verification_hash)
        .bind(&row.skill_md)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext("server", "quant.vibe-backtest", Some(row.id.as_str()), None)
            .await?;
        Ok(())
    }

    /// فهرستِ استراتژی‌هایِ ماندگار — نوترینِ اول، سقفِ `limit`.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_quant_strategies(&self, limit: u32) -> DbResult<Vec<QuantStrategyRow>> {
        let cap = i64::from(limit.clamp(1, 200));
        let sql = format!(
            r#"SELECT {QUANT_COLS} FROM "quant_strategies"
               ORDER BY "created_at" DESC, "id" DESC LIMIT ?"#
        );
        let rows = sqlx::query(crate::sqlsafe::audited(sql))
            .bind(cap)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.iter().map(quant_row_from).collect())
    }

    /// خوانشِ تک‌ردیفی — ناشناخته → `None`.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn get_quant_strategy(&self, id: &str) -> DbResult<Option<QuantStrategyRow>> {
        let row = sqlx::query(crate::sqlsafe::audited(format!(
            r#"SELECT {QUANT_COLS} FROM "quant_strategies" WHERE "id"=?"#
        )))
        .bind(id)
        .fetch_optional(self.pool_ref())
        .await?;
        Ok(row.as_ref().map(quant_row_from))
    }

    /// شمارشِ کلِ استراتژی‌ها — گیتِ آزمونِ هندلر.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn count_quant_strategies(&self) -> DbResult<i64> {
        let row = sqlx::query(crate::sqlsafe::audited(
            r#"SELECT COUNT(*) AS "c" FROM "quant_strategies""#.to_owned(),
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

    fn sample_row(id: &str) -> QuantStrategyRow {
        QuantStrategyRow {
            id: id.to_owned(),
            creator_user_id: "citizen:1".to_owned(),
            name: "روند-پیوِ کانونیک".to_owned(),
            symbol: "BTC-USDT".to_owned(),
            timeframe: "1h".to_owned(),
            vibe_prompt: "Momentum breakout with EMA 12/26 crossover".to_owned(),
            strategy_ast_json: r#"{"strategy_id":"vibe:abc123"}"#.to_owned(),
            metrics_json: r#"{"sharpeRatio":1.4}"#.to_owned(),
            sharpe_ratio: 1.4,
            max_drawdown_pct: -12.5,
            mairon_approved: true,
            claim_verification_hash: "deadbeef".to_owned(),
            skill_md: "---\nname: quant/x\n---\n".to_owned(),
            created_at: "2026-11-25T00:00:00Z".to_owned(),
        }
    }

    #[tokio::test]
    async fn quant_strategy_roundtrip_and_list_order() {
        let store = fresh_store().await;
        assert_eq!(store.count_quant_strategies().await.unwrap(), 0);
        store.create_quant_strategy(&sample_row("quant:a")).await.unwrap();
        store.create_quant_strategy(&sample_row("quant:b")).await.unwrap();
        assert_eq!(store.count_quant_strategies().await.unwrap(), 2);
        let listed = store.list_quant_strategies(10).await.unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, "quant:b", "نوترین اول");
        let got = store.get_quant_strategy("quant:a").await.unwrap().unwrap();
        assert!(got.mairon_approved);
        assert!((got.sharpe_ratio - 1.4).abs() < 1e-12);
        assert!(got.skill_md.starts_with("---"));
        assert!(store.get_quant_strategy("quant:missing").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn quant_list_respects_limit_cap() {
        let store = fresh_store().await;
        for i in 0..5 {
            store.create_quant_strategy(&sample_row(&format!("quant:{i}"))).await.unwrap();
        }
        let listed = store.list_quant_strategies(3).await.unwrap();
        assert_eq!(listed.len(), 3);
    }
}
