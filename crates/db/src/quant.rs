//! Causal quant engine — persistence for `quant_strategies` strategies.
//! (Wave 6 — Step 3/4).
//!
//! Three deterministic operations on migration `0010`: insert a Mairon-gate-green strategy,
//! list reads and a total count — with audit tracking on every insert. No ledger
//! table is touched.

use sqlx::Row;

use crate::error::DbResult;
use crate::store::V3Store;

/// Quant strategy row — the same columns as `quant_strategies` (migration 0010).
/// Strategy id (`vibe:<hash>`) lives inside `strategy_ast_json`.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantStrategyRow {
    /// Row id (`quant:<uuid>`).
    pub id: String,
    /// Builder account id.
    pub creator_user_id: String,
    /// Human-readable name.
    pub name: String,
    /// Symbol (`BTC-USDT`).
    pub symbol: String,
    /// Timeframe (`1h`).
    pub timeframe: String,
    /// Raw Vibe prompt.
    pub vibe_prompt: String,
    /// Typed AST as JSON.
    pub strategy_ast_json: String,
    /// Institutional metrics as JSON.
    pub metrics_json: String,
    /// Annualized Sharpe.
    pub sharpe_ratio: f64,
    /// Maximum drawdown (percent).
    pub max_drawdown_pct: f64,
    /// Mairon gate seal.
    pub mairon_approved: bool,
    /// Claims ledger hash.
    pub claim_verification_hash: String,
    /// The forged `SKILL.md`.
    pub skill_md: String,
    /// Generation timestamp (ISO-8601).
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
    /// Insert a quant strategy — only Mairon-gate-green output; deterministic and
    /// idempotent on `id`.
    ///
    /// # Errors
    /// SQL error or an audit error after the insert.
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

    /// Persisted strategy list — newest first, capped at `limit`.
    ///
    /// # Errors
    /// SQL error.
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

    /// Single-row read — unknown → `None`.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_quant_strategy(&self, id: &str) -> DbResult<Option<QuantStrategyRow>> {
        let row = sqlx::query(crate::sqlsafe::audited(format!(
            r#"SELECT {QUANT_COLS} FROM "quant_strategies" WHERE "id"=?"#
        )))
        .bind(id)
        .fetch_optional(self.pool_ref())
        .await?;
        Ok(row.as_ref().map(quant_row_from))
    }

    /// Total strategy count — the handler test gate.
    ///
    /// # Errors
    /// SQL error.
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
            name: "\u{631}\u{648}\u{646}\u{62f}-\u{67e}\u{6cc}\u{648}\u{650} \u{6a9}\u{627}\u{646}\u{648}\u{646}\u{6cc}\u{6a9}".to_owned(),
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
        assert_eq!(listed[0].id, "quant:b", "\u{646}\u{648}\u{62a}\u{631}\u{6cc}\u{646} \u{627}\u{648}\u{644}");
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
