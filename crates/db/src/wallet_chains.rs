//! M5 four-chain wallet layer — user deposit addresses and USDT deposits.
//!
//! The same M3 contracts as for TON deposits, now across TRX/SOL/TON/BSC:
//!  * Addresses are validated against the official chain checksums (kian-wallet)
//!    before reaching this layer — it never accepts a bad address.
//!  * Deposit hash UNIQUE on (chain, txHash) — "the same hash is never minted twice".
//!  * Actual minting happens in the server handler via the Article 3 path; only proof lives here.

use crate::error::{DbError, DbResult};
use crate::store::{V3Store, now_iso};
use kian_domain::{DomainError, DomainErrorCode};

/// User deposit address row — `WalletAddress` table (PK: userId+chain).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletAddressRow {
    /// Owning user (`citizen:…`)
    pub user_id: String,
    /// Chain: TRX / SOL / TON / BSC
    pub chain: String,
    /// Validated address
    pub address: String,
    /// Registration time (ISO)
    pub created_at: String,
}

/// USDT deposit row — `UsdtDeposit` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsdtDepositRow {
    /// ID (`udep:{uuid}`)
    pub id: String,
    /// Depositing user
    pub user_id: String,
    /// Chain: TRX / SOL / TON / BSC
    pub chain: String,
    /// On-chain transaction hash — UNIQUE(chain, txHash)
    pub tx_hash: String,
    /// Sender address
    pub from_address: String,
    /// Amount in micro-USDT scale (integer, no float)
    pub amount_micro: i64,
    /// User-declared decimal string (≤6 decimals)
    pub amount_usdt: String,
    /// `PENDING` / `CONFIRMED` / `REJECTED`
    pub status: String,
    /// Registration time (ISO)
    pub created_at: String,
    /// Decision time (ISO) — `None` until resolved
    pub confirmed_at: Option<String>,
    /// Deciding admin — `None` until resolved
    pub confirmed_by: Option<String>,
}

const ADDR_SELECT: &str = r#"SELECT "userId","chain","address","createdAt" FROM "WalletAddress""#;

#[allow(clippy::needless_pass_by_value)] // sqlx pattern
fn addr_row_from(row: sqlx::sqlite::SqliteRow) -> WalletAddressRow {
    use sqlx::Row as _;
    WalletAddressRow {
        user_id: row.get("userId"),
        chain: row.get("chain"),
        address: row.get("address"),
        created_at: row.get("createdAt"),
    }
}

const USDT_SELECT: &str = r#"SELECT "id","userId","chain","txHash","fromAddress","amountMicro","amountUsdt","status","createdAt","confirmedAt","confirmedBy" FROM "UsdtDeposit""#;

#[allow(clippy::needless_pass_by_value)] // sqlx pattern
fn usdt_row_from(row: sqlx::sqlite::SqliteRow) -> UsdtDepositRow {
    use sqlx::Row as _;
    UsdtDepositRow {
        id: row.get("id"),
        user_id: row.get("userId"),
        chain: row.get("chain"),
        tx_hash: row.get("txHash"),
        from_address: row.get("fromAddress"),
        amount_micro: row.get("amountMicro"),
        amount_usdt: row.get("amountUsdt"),
        status: row.get("status"),
        created_at: row.get("createdAt"),
        confirmed_at: row.get("confirmedAt"),
        confirmed_by: row.get("confirmedBy"),
    }
}

impl V3Store {
    /// Upsert a user's deposit address for one chain — PK (userId, chain)
    /// means one address per chain; a new address replaces the old one (wallet change).
    ///
    /// # Errors
    /// SQL error.
    pub async fn upsert_wallet_address(&self, row: &WalletAddressRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "WalletAddress" ("userId","chain","address","createdAt")
               VALUES (?,?,?,?)
               ON CONFLICT("userId","chain") DO UPDATE SET "address" = excluded."address""#,
        )
        .bind(&row.user_id)
        .bind(&row.chain)
        .bind(&row.address)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext(
            "server",
            format!("wallet.address.{}", row.chain).as_str(),
            Some(&row.user_id),
            None,
        )
        .await?;
        Ok(())
    }

    /// A user's addresses — up to four chains.
    ///
    /// # Errors
    /// SQL error.
    pub async fn wallet_addresses_for(&self, user_id: &str) -> DbResult<Vec<WalletAddressRow>> {
        // audited: const prefix + bind
        let sql = format!("{ADDR_SELECT} WHERE \"userId\" = ? ORDER BY \"chain\" ASC");
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user_id)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.into_iter().map(addr_row_from).collect())
    }

    /// Insert a USDT deposit — (chain, txHash) pre-checked by the caller; the UNIQUE constraint is the backstop.
    ///
    /// # Errors
    /// Concurrent duplicate hash → sqlx UNIQUE error; SQL error.
    pub async fn insert_usdt_deposit(&self, d: &UsdtDepositRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "UsdtDeposit"
               ("id","userId","chain","txHash","fromAddress","amountMicro","amountUsdt","status","createdAt")
               VALUES (?,?,?,?,?,?,?,'PENDING',?)"#,
        )
        .bind(&d.id)
        .bind(&d.user_id)
        .bind(&d.chain)
        .bind(&d.tx_hash)
        .bind(&d.from_address)
        .bind(d.amount_micro)
        .bind(&d.amount_usdt)
        .bind(&d.created_at)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext(
            "server",
            format!("usdt_deposit.create.{}", d.chain).as_str(),
            Some(&d.id),
            None,
        )
        .await?;
        Ok(())
    }

    /// Find a deposit by (chain, hash) — replay pre-check for re-registration.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_usdt_deposit_by_hash(
        &self,
        chain: &str,
        tx_hash: &str,
    ) -> DbResult<Option<UsdtDepositRow>> {
        let sql = format!("{USDT_SELECT} WHERE \"chain\" = ? AND \"txHash\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(chain)
            .bind(tx_hash)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(usdt_row_from))
    }

    /// A user's deposits — newest first (last 50).
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_usdt_deposits_for(&self, user_id: &str) -> DbResult<Vec<UsdtDepositRow>> {
        let sql = format!(r#"{USDT_SELECT} WHERE "userId" = ? ORDER BY "createdAt" DESC LIMIT 50"#);
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user_id)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.into_iter().map(usdt_row_from).collect())
    }

    /// Read one deposit by ID.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_usdt_deposit(&self, id: &str) -> DbResult<Option<UsdtDepositRow>> {
        let sql = format!("{USDT_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(usdt_row_from))
    }

    /// Resolve a deposit — only from `PENDING` (one-way transition).
    ///
    /// # Errors
    /// Unknown → `EscrowNotFound`; non-PENDING → `InvalidTransition`.
    pub async fn set_usdt_deposit_status(
        &self,
        id: &str,
        status: &str,
        by: &str,
    ) -> DbResult<UsdtDepositRow> {
        let res = sqlx::query(
            r#"UPDATE "UsdtDeposit" SET "status" = ?, "confirmedAt" = ?, "confirmedBy" = ?
               WHERE "id" = ? AND "status" = 'PENDING'"#,
        )
        .bind(status)
        .bind(now_iso())
        .bind(by)
        .bind(id)
        .execute(self.pool_ref())
        .await?;
        if res.rows_affected() == 0 {
            return Err(match self.get_usdt_deposit(id).await? {
                None => DbError::Domain(DomainError::new(
                    DomainErrorCode::EscrowNotFound,
                    format!("Usdt deposit {id} does not exist"),
                )),
                Some(_) => DbError::Domain(DomainError::new(
                    DomainErrorCode::InvalidTransition,
                    format!("Usdt deposit {id} is not PENDING anymore"),
                )),
            });
        }
        self.audit_ext(
            "server",
            format!("usdt_deposit.{status}").as_str(),
            Some(id),
            None,
        )
        .await?;
        self.get_usdt_deposit(id)
            .await?
            .ok_or_else(|| DbError::Config(format!("usdt deposit {id} vanished after update")))
    }

    /// Register a new withdrawal request (`PENDING`) — after the atomic Din lock/burn in the ledger.
    ///
    /// # Errors
    /// SQL error.
    pub async fn insert_withdrawal_request(&self, row: &WithdrawalRequestRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "WithdrawalRequest"
               ("id","accountId","chain","asset","destinationAddress","amountDin",
                "txHash","status","createdAt","resolvedAt")
               VALUES (?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.chain)
        .bind(&row.asset)
        .bind(&row.destination_address)
        .bind(row.amount_din)
        .bind(&row.tx_hash)
        .bind(&row.status)
        .bind(row.created_at)
        .bind(row.resolved_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// Read one withdrawal request by ID.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_withdrawal_request(&self, id: &str) -> DbResult<Option<WithdrawalRequestRow>> {
        let row = sqlx::query(
            r#"SELECT "id","accountId","chain","asset","destinationAddress","amountDin",
               "txHash","status","createdAt","resolvedAt"
               FROM "WithdrawalRequest" WHERE "id" = ?"#,
        )
        .bind(id)
        .fetch_optional(self.pool_ref())
        .await?;
        Ok(row.map(withdrawal_row_from))
    }

    /// List an account's withdrawal requests — newest first (last 50).
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_withdrawals_for_account(
        &self,
        account_id: &str,
    ) -> DbResult<Vec<WithdrawalRequestRow>> {
        let rows = sqlx::query(
            r#"SELECT "id","accountId","chain","asset","destinationAddress","amountDin",
               "txHash","status","createdAt","resolvedAt"
               FROM "WithdrawalRequest"
               WHERE "accountId" = ?
               ORDER BY "createdAt" DESC, "id" DESC
               LIMIT 50"#,
        )
        .bind(account_id)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows.into_iter().map(withdrawal_row_from).collect())
    }

    /// Admin list of withdrawal requests — optional status filter.
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_withdrawals_all(
        &self,
        status: Option<&str>,
        limit: i64,
    ) -> DbResult<Vec<WithdrawalRequestRow>> {
        let cap = limit.clamp(1, 200);
        let rows = match status {
            Some(st) => {
                sqlx::query(
                    r#"SELECT "id","accountId","chain","asset","destinationAddress","amountDin",
                       "txHash","status","createdAt","resolvedAt"
                       FROM "WithdrawalRequest"
                       WHERE "status" = ?
                       ORDER BY "createdAt" DESC, "id" DESC
                       LIMIT ?"#,
                )
                .bind(st)
                .bind(cap)
                .fetch_all(self.pool_ref())
                .await?
            }
            None => {
                sqlx::query(
                    r#"SELECT "id","accountId","chain","asset","destinationAddress","amountDin",
                       "txHash","status","createdAt","resolvedAt"
                       FROM "WithdrawalRequest"
                       ORDER BY "createdAt" DESC, "id" DESC
                       LIMIT ?"#,
                )
                .bind(cap)
                .fetch_all(self.pool_ref())
                .await?
            }
        };
        Ok(rows.into_iter().map(withdrawal_row_from).collect())
    }

    /// One-way resolution of a withdrawal request (`PENDING` → `CONFIRMED` or `REJECTED`).
    ///
    /// # Errors
    /// Unknown → `EscrowNotFound`; already decided → `InvalidTransition`.
    pub async fn resolve_withdrawal_request(
        &self,
        id: &str,
        status: &str,
        tx_hash: Option<&str>,
        resolved_at: i64,
    ) -> DbResult<WithdrawalRequestRow> {
        let current = self.get_withdrawal_request(id).await?.ok_or_else(|| {
            DomainError::new(
                DomainErrorCode::EscrowNotFound,
                format!("Withdrawal request {id} does not exist"),
            )
        })?;
        if current.status != "PENDING" {
            return Err(DomainError::new(
                DomainErrorCode::InvalidTransition,
                format!("Withdrawal request {id} is already {}", current.status),
            )
            .into());
        }
        let res = sqlx::query(
            r#"UPDATE "WithdrawalRequest"
               SET "status" = ?, "txHash" = COALESCE(?, "txHash"), "resolvedAt" = ?
               WHERE "id" = ? AND "status" = 'PENDING'"#,
        )
        .bind(status)
        .bind(tx_hash)
        .bind(resolved_at)
        .bind(id)
        .execute(self.pool_ref())
        .await?;
        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                DomainErrorCode::InvalidTransition,
                format!("Withdrawal request {id} was already resolved concurrently"),
            )
            .into());
        }
        self.audit_ext(
            "server",
            format!("withdrawal.{status}").as_str(),
            Some(id),
            None,
        )
        .await?;
        self.get_withdrawal_request(id)
            .await?
            .ok_or_else(|| DbError::Config(format!("withdrawal {id} vanished after update")))
    }
}

/// Crypto withdrawal request row — `"WithdrawalRequest"` table (Phase 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithdrawalRequestRow {
    /// Unique ID (`wdr:{uuid}`)
    pub id: String,
    /// Citizen account (`citizen:{tg_id}`)
    pub account_id: String,
    /// Destination chain (`TON`/`TRX`/`SOL`/`BSC`)
    pub chain: String,
    /// Asset (`TON`/`USDT`)
    pub asset: String,
    /// Validated destination address
    pub destination_address: String,
    /// Amount in Din (deducted from the account at registration time)
    pub amount_din: i64,
    /// On-chain payment transaction hash (after admin confirmation)
    pub tx_hash: Option<String>,
    /// Status (`PENDING`/`CONFIRMED`/`REJECTED`)
    pub status: String,
    /// Registration time (Unix epoch seconds)
    pub created_at: i64,
    /// Admin decision time (Unix epoch seconds)
    pub resolved_at: Option<i64>,
}

#[allow(clippy::needless_pass_by_value)]
fn withdrawal_row_from(r: sqlx::sqlite::SqliteRow) -> WithdrawalRequestRow {
    use sqlx::Row as _;
    WithdrawalRequestRow {
        id: r.get("id"),
        account_id: r.get("accountId"),
        chain: r.get("chain"),
        asset: r.get("asset"),
        destination_address: r.get("destinationAddress"),
        amount_din: r.get("amountDin"),
        tx_hash: r.get("txHash"),
        status: r.get("status"),
        created_at: r.get("createdAt"),
        resolved_at: r.get("resolvedAt"),
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

    fn deposit(id: &str, chain: &str, hash: &str) -> UsdtDepositRow {
        UsdtDepositRow {
            id: id.into(),
            user_id: "citizen:5".into(),
            chain: chain.into(),
            tx_hash: hash.into(),
            from_address: "addr".into(),
            amount_micro: 1_500_000,
            amount_usdt: "1.5".into(),
            status: "PENDING".into(),
            created_at: now_iso(),
            confirmed_at: None,
            confirmed_by: None,
        }
    }

    #[tokio::test]
    async fn wallet_address_upsert_replaces_per_chain() {
        let store = fresh_store().await;
        store
            .upsert_wallet_address(&WalletAddressRow {
                user_id: "citizen:5".into(),
                chain: "BSC".into(),
                address: "0xaaa".into(),
                created_at: now_iso(),
            })
            .await
            .unwrap();
        // same chain — the new address replaces; no duplicate row
        store
            .upsert_wallet_address(&WalletAddressRow {
                user_id: "citizen:5".into(),
                chain: "BSC".into(),
                address: "0xbbb".into(),
                created_at: now_iso(),
            })
            .await
            .unwrap();
        store
            .upsert_wallet_address(&WalletAddressRow {
                user_id: "citizen:5".into(),
                chain: "TRX".into(),
                address: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".into(),
                created_at: now_iso(),
            })
            .await
            .unwrap();
        let all = store.wallet_addresses_for("citizen:5").await.unwrap();
        assert_eq!(all.len(), 2);
        let bsc = all.iter().find(|r| r.chain == "BSC").unwrap();
        assert_eq!(bsc.address, "0xbbb");
    }

    #[tokio::test]
    async fn usdt_deposit_unique_per_chain_and_single_decision() {
        let store = fresh_store().await;
        store
            .insert_usdt_deposit(&deposit("udep:1", "TON", "hash111"))
            .await
            .unwrap();

        // same hash on the same chain — replay rejected
        assert!(
            store
                .insert_usdt_deposit(&deposit("udep:2", "TON", "hash111"))
                .await
                .is_err()
        );
        // same hash on a different chain — allowed (chain hash namespaces are separate)
        store
            .insert_usdt_deposit(&deposit("udep:3", "BSC", "hash111"))
            .await
            .unwrap();

        let mine = store.list_usdt_deposits_for("citizen:5").await.unwrap();
        assert_eq!(mine.len(), 2);

        let confirmed = store
            .set_usdt_deposit_status("udep:1", "CONFIRMED", "admin")
            .await
            .unwrap();
        assert_eq!(confirmed.status, "CONFIRMED");
        assert_eq!(confirmed.confirmed_by.as_deref(), Some("admin"));

        // second decision — 422
        assert!(
            store
                .set_usdt_deposit_status("udep:1", "REJECTED", "admin")
                .await
                .is_err()
        );
        // unknown — 404-style domain error
        let missing = store
            .set_usdt_deposit_status("udep:x", "CONFIRMED", "admin")
            .await
            .unwrap_err();
        assert!(matches!(missing, DbError::Domain(_)));
    }
}
