//! M3 identity layer — Telegram users, bearer sessions and TON deposits.
//!
//! All queries go through the same [`V3Store`] (the single point where the domain touches
//! SQLite). Identity knows nothing about "money" — balance is always derived from `Posting`
//! (ADR-003); only identity/session/deposit-proof rows live here.
//!
//! Table contracts: `crates/db/migrations/0002_m3_sessions.sql`.
//!
//! sqlx 0.9 note — "dynamic SQL strings must be audited": in the 4 queries on
//! [`USER_SELECT`]/[`DEPOSIT_SELECT`] the dynamic part is only the compile-time constant
//! prefix (no user input is interpolated into the string; every value is bound
//! with `?`) — so per sqlx's own guidance they are marked as audited with
//! `AssertSqlSafe`.
use crate::error::{DbError, DbResult};
use crate::store::{now_iso, V3Store};

/// Citizen account prefix — the Telegram identity is tied to this ledger account.
pub const CITIZEN_PREFIX: &str = "citizen:";

/// Ledger account corresponding to a Telegram identity — `citizen:{telegram_id}`.
#[must_use]
pub fn citizen_account_id(telegram_id: &str) -> String {
    format!("{CITIZEN_PREFIX}{telegram_id}")
}

/// Mini-app user row — one-to-one with the `User` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRow {
    /// Stable ID = citizen account (`citizen:{telegramId}`)
    pub id: String,
    /// Telegram numeric ID as a string
    pub telegram_id: String,
    /// Username without `@` — optional
    pub username: Option<String>,
    /// Connected TON wallet address — optional
    pub ton_address: Option<String>,
    /// Connected double-entry ledger account
    pub account_id: String,
    /// Registration time (ISO)
    pub created_at: String,
}

/// TON deposit row — user proof until admin verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TonDepositRow {
    /// ID (`dep:{uuid}`)
    pub id: String,
    /// Depositing user
    pub user_id: String,
    /// On-chain transaction hash — UNIQUE (replay lesson)
    pub tx_hash: String,
    /// Sender address
    pub wallet_address: String,
    /// TON amount as a decimal string
    pub amount_ton: String,
    /// `PENDING` / `CONFIRMED` / `REJECTED`
    pub status: String,
    /// Registration time (ISO)
    pub created_at: String,
    /// Decision time (ISO) — `None` until resolved
    pub confirmed_at: Option<String>,
    /// Deciding admin — `None` until resolved
    pub confirmed_by: Option<String>,
}

const USER_SELECT: &str = r#"SELECT "id","telegramId","username","tonAddress","accountId","createdAt" FROM "User""#;

#[allow(clippy::needless_pass_by_value)] // sqlx pattern: Row ownership comes from the fetch result and is consumed
fn user_row_from(row: sqlx::sqlite::SqliteRow) -> UserRow {
    use sqlx::Row as _;
    UserRow {
        id: row.get("id"),
        telegram_id: row.get("telegramId"),
        username: row.get("username"),
        ton_address: row.get("tonAddress"),
        account_id: row.get("accountId"),
        created_at: row.get("createdAt"),
    }
}

const DEPOSIT_SELECT: &str = r#"SELECT "id","userId","txHash","walletAddress","amountTon","status","createdAt","confirmedAt","confirmedBy" FROM "TonDeposit""#;

#[allow(clippy::needless_pass_by_value)] // sqlx pattern: Row ownership comes from the fetch result and is consumed
fn deposit_row_from(row: sqlx::sqlite::SqliteRow) -> TonDepositRow {
    use sqlx::Row as _;
    TonDepositRow {
        id: row.get("id"),
        user_id: row.get("userId"),
        tx_hash: row.get("txHash"),
        wallet_address: row.get("walletAddress"),
        amount_ton: row.get("amountTon"),
        status: row.get("status"),
        created_at: row.get("createdAt"),
        confirmed_at: row.get("confirmedAt"),
        confirmed_by: row.get("confirmedBy"),
    }
}

impl V3Store {
    /// Register/update a Telegram citizen — idempotent on `telegramId`.
    ///
    /// The `citizen:{telegramId}` ledger account is created if missing (`kind=User`);
    /// a newer username replaces the old one, so Telegram renames propagate too.
    ///
    /// # Errors
    /// SQL error or ledger account creation error.
    pub async fn ensure_telegram_user(
        &self,
        telegram_id: &str,
        username: Option<&str>,
    ) -> DbResult<UserRow> {
        let id = citizen_account_id(telegram_id);
        self.create_account_if_missing(&id, kian_domain::AccountKind::User, username)
            .await?;
        sqlx::query(
            r#"INSERT INTO "User" ("id","telegramId","username","accountId","createdAt")
               VALUES (?,?,?,?,?)
               ON CONFLICT("telegramId") DO UPDATE SET "username" =
                   COALESCE(excluded."username", "User"."username")"#,
        )
        .bind(&id)
        .bind(telegram_id)
        .bind(username)
        .bind(&id)
        .bind(now_iso())
        .execute(self.pool_ref())
        .await?;
        self.audit_ext("server", "user.upsert", Some(&id), None).await?;
        self.get_user(&id).await?
            .ok_or_else(|| DbError::Config(format!("user {id} vanished after upsert")))
    }

    /// Read a user by ID.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_user(&self, id: &str) -> DbResult<Option<UserRow>> {
        // audited: const prefix + bind — no user input in the string
        let sql = format!("{USER_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(user_row_from))
    }

    /// Link a TON address to the identity — after TON Connect.
    ///
    /// # Errors
    /// Unknown user → `USER_NOT_FOUND`.
    pub async fn set_user_ton_address(&self, user_id: &str, address: &str) -> DbResult<()> {
        let res = sqlx::query(r#"UPDATE "User" SET "tonAddress" = ? WHERE "id" = ?"#)
            .bind(address)
            .bind(user_id)
            .execute(self.pool_ref())
            .await?;
        if res.rows_affected() == 0 {
            return Err(DbError::Domain(kian_domain::DomainError::new(
                kian_domain::DomainErrorCode::UnknownAccount,
                format!("User {user_id} does not exist"),
            )));
        }
        self.audit_ext("server", "user.ton_address", Some(user_id), None).await?;
        Ok(())
    }

    /// Open a session — opaque token with an explicit expiry.
    ///
    /// # Errors
    /// SQL error.
    pub async fn create_session(&self, token: &str, user_id: &str, expires_at: &str) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "Session" ("token","userId","createdAt","expiresAt") VALUES (?,?,?,?)"#,
        )
        .bind(token)
        .bind(user_id)
        .bind(now_iso())
        .bind(expires_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// User of a valid session — `None` means the token is unknown or expired (fail-closed).
    ///
    /// # Errors
    /// SQL error.
    pub async fn session_user(&self, token: &str) -> DbResult<Option<UserRow>> {
        let row = sqlx::query(
            r#"SELECT u."id",u."telegramId",u."username",u."tonAddress",u."accountId",u."createdAt"
               FROM "User" u INNER JOIN "Session" s ON s."userId" = u."id"
               WHERE s."token" = ? AND s."expiresAt" > ?"#,
        )
        .bind(token)
        .bind(now_iso())
        .fetch_optional(self.pool_ref())
        .await?;
        Ok(row.map(user_row_from))
    }

    /// Delete a session (logout) — calling again is a no-op with no error.
    ///
    /// # Errors
    /// SQL error.
    pub async fn delete_session(&self, token: &str) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "Session" WHERE "token" = ?"#)
            .bind(token)
            .execute(self.pool_ref())
            .await?;
        Ok(())
    }

    /// Prune expired sessions — keeps the table light (periodic sweep).
    ///
    /// # Errors
    /// SQL error.
    pub async fn sweep_expired_sessions(&self) -> DbResult<u64> {
        let res = sqlx::query(r#"DELETE FROM "Session" WHERE "expiresAt" <= ?"#)
            .bind(now_iso())
            .execute(self.pool_ref())
            .await?;
        Ok(res.rows_affected())
    }

    /// Register a TON deposit — the same `txHash` is never accepted twice (table UNIQUE constraint;
    /// the caller pre-checks `get_ton_deposit_by_hash` to return an honest message).
    ///
    /// # Errors
    /// Concurrent duplicate hash → sqlx UNIQUE error; SQL error.
    pub async fn insert_ton_deposit(&self, d: &TonDepositRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "TonDeposit"
               ("id","userId","txHash","walletAddress","amountTon","status","createdAt")
               VALUES (?,?,?,?,?,?,?)"#,
        )
        .bind(&d.id)
        .bind(&d.user_id)
        .bind(&d.tx_hash)
        .bind(&d.wallet_address)
        .bind(&d.amount_ton)
        .bind("PENDING")
        .bind(&d.created_at)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext("server", "ton_deposit.create", Some(&d.id), None).await?;
        Ok(())
    }

    /// Find a deposit by on-chain hash — replay pre-check for re-registration.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_ton_deposit_by_hash(&self, tx_hash: &str) -> DbResult<Option<TonDepositRow>> {
        // audited: const prefix + bind — no user input in the string
        let sql = format!("{DEPOSIT_SELECT} WHERE \"txHash\" = ?");
        let row =
            sqlx::query(sqlx::AssertSqlSafe(sql)).bind(tx_hash).fetch_optional(self.pool_ref()).await?;
        Ok(row.map(deposit_row_from))
    }

    /// A user's deposits — newest first.
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_ton_deposits_for(&self, user_id: &str) -> DbResult<Vec<TonDepositRow>> {
        // audited: const prefix + bind — no user input in the string
        let sql = format!(
            r#"{DEPOSIT_SELECT} WHERE "userId" = ? ORDER BY "createdAt" DESC LIMIT 50"#
        );
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user_id)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.into_iter().map(deposit_row_from).collect())
    }

    /// Read one deposit.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_ton_deposit(&self, id: &str) -> DbResult<Option<TonDepositRow>> {
        // audited: const prefix + bind — no user input in the string
        let sql = format!("{DEPOSIT_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(deposit_row_from))
    }

    /// Resolve a deposit — only from `PENDING` (one-way transition).
    ///
    /// # Errors
    /// Unknown deposit → `NOT_FOUND`; non-PENDING status → `INVALID_TRANSITION`.
    pub async fn set_ton_deposit_status(
        &self,
        id: &str,
        status: &str,
        by: &str,
    ) -> DbResult<TonDepositRow> {
        let res = sqlx::query(
            r#"UPDATE "TonDeposit" SET "status" = ?, "confirmedAt" = ?, "confirmedBy" = ?
               WHERE "id" = ? AND "status" = 'PENDING'"#,
        )
        .bind(status)
        .bind(now_iso())
        .bind(by)
        .bind(id)
        .execute(self.pool_ref())
        .await?;
        if res.rows_affected() == 0 {
            let existing = self.get_ton_deposit(id).await?;
            return Err(match existing {
                None => DbError::Domain(kian_domain::DomainError::new(
                    kian_domain::DomainErrorCode::EscrowNotFound,
                    format!("Ton deposit {id} does not exist"),
                )),
                Some(_) => DbError::Domain(kian_domain::DomainError::new(
                    kian_domain::DomainErrorCode::InvalidTransition,
                    format!("Ton deposit {id} is not PENDING anymore"),
                )),
            });
        }
        self.audit_ext("server", format!("ton_deposit.{status}").as_str(), Some(id), None).await?;
        self.get_ton_deposit(id).await?
            .ok_or_else(|| DbError::Config(format!("deposit {id} vanished after update")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn fresh_store() -> V3Store {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        store
    }

    #[tokio::test]
    async fn ensure_telegram_user_is_idempotent_and_renames() {
        let store = fresh_store().await;
        let first = store.ensure_telegram_user("123", Some("ali")).await.unwrap();
        assert_eq!(first.id, "citizen:123");
        assert_eq!(first.account_id, "citizen:123");
        let again = store.ensure_telegram_user("123", Some("ali_new")).await.unwrap();
        assert_eq!(again.created_at, first.created_at, "same row — not duplicated");
        assert_eq!(again.username.as_deref(), Some("ali_new"), "username refreshed");
        // a login without a username (raw dev-login) never wipes a migrated username
        let anonymous = store.ensure_telegram_user("123", None).await.unwrap();
        assert_eq!(anonymous.username.as_deref(), Some("ali_new"), "None preserves username");
        // the ledger account is actually created — zero balance
        assert_eq!(store.balance_of("citizen:123").await.unwrap(), Some(0));
    }

    #[tokio::test]
    async fn session_lifecycle_fails_closed_on_expiry() {
        let store = fresh_store().await;
        let user = store.ensure_telegram_user("77", None).await.unwrap();

        store
            .create_session(&"tok_valid".repeat(8), &user.id, "2099-01-01T00:00:00.000Z")
            .await
            .unwrap();
        store
            .create_session(&"tok_expired".repeat(8), &user.id, "2000-01-01T00:00:00.000Z")
            .await
            .unwrap();

        assert!(store.session_user(&"tok_valid".repeat(8)).await.unwrap().is_some());
        assert!(store.session_user(&"tok_expired".repeat(8)).await.unwrap().is_none());
        assert!(store.session_user("unknown-token").await.unwrap().is_none());

        store.delete_session(&"tok_valid".repeat(8)).await.unwrap();
        assert!(store.session_user(&"tok_valid".repeat(8)).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn ton_deposit_unique_hash_and_single_decision() {
        let store = fresh_store().await;
        let user = store.ensure_telegram_user("9", Some("sara")).await.unwrap();

        let row = TonDepositRow {
            id: "dep:1".into(),
            user_id: user.id.clone(),
            tx_hash: "abc123def".into(),
            wallet_address: "UQ_wallet".into(),
            amount_ton: "0.5".into(),
            status: "PENDING".into(),
            created_at: now_iso(),
            confirmed_at: None,
            confirmed_by: None,
        };
        store.insert_ton_deposit(&row).await.unwrap();

        // same hash again — replay rejected
        assert!(store.insert_ton_deposit(&row).await.is_err());

        let mine = store.list_ton_deposits_for(&user.id).await.unwrap();
        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].status, "PENDING");

        let confirmed = store
            .set_ton_deposit_status("dep:1", "CONFIRMED", "admin")
            .await
            .unwrap();
        assert_eq!(confirmed.status, "CONFIRMED");
        assert_eq!(confirmed.confirmed_by.as_deref(), Some("admin"));

        // a second transition from PENDING is impossible
        assert!(store.set_ton_deposit_status("dep:1", "REJECTED", "admin").await.is_err());
    }
}
