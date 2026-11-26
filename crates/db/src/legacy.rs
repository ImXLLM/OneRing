//! The legacy database reader — raw SQL over the recovered Railway file.
//!
//! The legacy tables (`users`, `products`, `transactions`) are not the v3 model; like
//! `migrate-legacy.ts` we read with raw queries and `CAST(... AS TEXT)` because
//! legacy IDs exceed Int32. The connection is read-only — the migration
//! never writes to the source.

use crate::error::{DbError, DbResult};
use crate::genesis::{LegacyProductRow, LegacyUserRow};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row,
};

/// The complete snapshot read from legacy — input to migration and the genesis plan.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySnapshot {
    /// All users (e.g. the 466 day-one users) sorted by ID
    pub users: Vec<LegacyUserRow>,
    /// All products (e.g. 30 legacy products) sorted by ID
    pub products: Vec<LegacyProductRow>,
    /// The legacy transaction row count (e.g. 1,554) — for reporting only
    pub tx_count: i64,
}

/// Read the legacy snapshot from the SQLite file — read-only.
///
/// # Errors
/// The file was missing, the tables were missing, or SQLite failed to open.
pub async fn read_legacy(path: &str) -> DbResult<LegacySnapshot> {
    let file = path.strip_prefix("file:").unwrap_or(path);
    if !std::path::Path::new(file).is_file() {
        return Err(DbError::Config(format!("legacy db not found: {file}")));
    }
    let opts = SqliteConnectOptions::new().filename(file).read_only(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .map_err(|e| DbError::Config(format!("cannot open legacy db {file}: {e}")))?;

    let mut users = Vec::new();
    for r in sqlx::query(
        "SELECT CAST(user_id AS TEXT) AS user_id, username, first_name, credits, is_banned \
         FROM users ORDER BY user_id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| DbError::Config(format!("legacy users query failed: {e}")))?
    {
        users.push(LegacyUserRow {
            user_id: r.get("user_id"),
            username: r.get("username"),
            first_name: r.get("first_name"),
            credits: r.get("credits"),
            is_banned: r.get("is_banned"),
        });
    }

    let mut products = Vec::new();
    for r in sqlx::query(
        "SELECT CAST(id AS TEXT) AS id, CAST(creator_id AS TEXT) AS creator_id, title, \
         description, price_credits, category, link, is_active \
         FROM products ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| DbError::Config(format!("legacy products query failed: {e}")))?
    {
        products.push(LegacyProductRow {
            id: r.get("id"),
            creator_id: r.get("creator_id"),
            title: r.get("title"),
            description: r.get("description"),
            price_credits: r.get("price_credits"),
            category: r.get("category"),
            link: r.get("link"),
            is_active: r.get("is_active"),
        });
    }

    let tx_count: i64 = sqlx::query("SELECT COUNT(*) AS c FROM transactions")
        .fetch_one(&pool)
        .await
        .map_err(|e| DbError::Config(format!("legacy transactions query failed: {e}")))?
        .get("c");

    pool.close().await;
    Ok(LegacySnapshot { users, products, tx_count })
}
