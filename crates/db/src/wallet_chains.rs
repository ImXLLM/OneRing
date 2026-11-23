//! لایهٔ کیف پول چهارزنجیره M5 — آدرس‌های واریز کاربر و واریزهای USDT.
//!
//! همان قراردادهای M3 برای واریز TON، این‌بار روی TRX/SOL/TON/BSC:
//!  * آدرس‌ها قبل از رسیدن اینجا با چک‌سام‌های رسمی زنجیره (kian-wallet)
//!    اعتبارسنجی شده‌اند — این لایه هرگز آدرس بد را نمی‌پذیرد.
//!  * هش واریز UNIQUE روی (chain, txHash) — «همان هش دوبار ضرب نمی‌شود».
//!  * ضرب واقعی در هندلر سرور از مسیر مادهٔ ۳ می‌گذرد؛ اینجا فقط اثبات.

use crate::error::{DbError, DbResult};
use crate::store::{V3Store, now_iso};
use kian_domain::{DomainError, DomainErrorCode};

/// ردیف آدرس واریز کاربر — جدول `WalletAddress` (PK: userId+chain).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletAddressRow {
    /// کاربر مالک (`citizen:…`)
    pub user_id: String,
    /// زنجیره: TRX / SOL / TON / BSC
    pub chain: String,
    /// آدرس اعتبارسنجی‌شده
    pub address: String,
    /// زمان ثبت ISO
    pub created_at: String,
}

/// ردیف واریز USDT — جدول `UsdtDeposit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsdtDepositRow {
    /// شناسه (`udep:{uuid}`)
    pub id: String,
    /// کاربر ثبت‌کننده
    pub user_id: String,
    /// زنجیره: TRX / SOL / TON / BSC
    pub chain: String,
    /// هش تراکنش زنجیره — UNIQUE(chain, txHash)
    pub tx_hash: String,
    /// آدرس فرستنده
    pub from_address: String,
    /// مبلغ در مقیاس micro-USDT (صحیح، بدون float)
    pub amount_micro: i64,
    /// رشتهٔ دهدهی اعلامی کاربر (≤۶ اعشار)
    pub amount_usdt: String,
    /// `PENDING` / `CONFIRMED` / `REJECTED`
    pub status: String,
    /// زمان ثبت ISO
    pub created_at: String,
    /// زمان تصمیم ISO — تا قبل از تکلیف `None`
    pub confirmed_at: Option<String>,
    /// تصمیم‌گیرنده (ادمین) — تا قبل از تکلیف `None`
    pub confirmed_by: Option<String>,
}

const ADDR_SELECT: &str = r#"SELECT "userId","chain","address","createdAt" FROM "WalletAddress""#;

#[allow(clippy::needless_pass_by_value)] // الگوی sqlx
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

#[allow(clippy::needless_pass_by_value)] // الگوی sqlx
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
    /// ثبت/به‌روزرسانی آدرس واریز کاربر برای یک زنجیره — PK (userId, chain)
    /// یعنی هر زنجیره فقط یک آدرس؛ آدرس جدید جایگزین می‌شود (تغییر ولت).
    ///
    /// # Errors
    /// خطای SQL.
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

    /// آدرس‌های یک کاربر — تا چهار زنجیره.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn wallet_addresses_for(&self, user_id: &str) -> DbResult<Vec<WalletAddressRow>> {
        // حسابرسی‌شده: پیشوند const + bind
        let sql = format!("{ADDR_SELECT} WHERE \"userId\" = ? ORDER BY \"chain\" ASC");
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user_id)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.into_iter().map(addr_row_from).collect())
    }

    /// درج واریز USDT — پیش‌چک (chain, txHash) در فراخواننده؛ قید UNIQUE پشتوانه.
    ///
    /// # Errors
    /// هش تکراری هم‌زمان → خطای UNIQUE sqlx؛ خطای SQL.
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

    /// جست‌وجوی واریز با (زنجیره، هش) — پیش‌چک replay ثبت دوباره.
    ///
    /// # Errors
    /// خطای SQL.
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

    /// واریزهای یک کاربر — جدیدترین اول (۵۰ آخر).
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_usdt_deposits_for(&self, user_id: &str) -> DbResult<Vec<UsdtDepositRow>> {
        let sql = format!(r#"{USDT_SELECT} WHERE "userId" = ? ORDER BY "createdAt" DESC LIMIT 50"#);
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user_id)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.into_iter().map(usdt_row_from).collect())
    }

    /// خواندن یک واریز با شناسه.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn get_usdt_deposit(&self, id: &str) -> DbResult<Option<UsdtDepositRow>> {
        let sql = format!("{USDT_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(usdt_row_from))
    }

    /// تعیین تکلیف واریز — فقط از `PENDING` (گذار یک‌طرفه).
    ///
    /// # Errors
    /// ناشناخته → `EscrowNotFound`؛ غیر-PENDING → `InvalidTransition`.
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

    /// ثبت درخواست برداشت جدید (`PENDING`) — پس از قفل/سوزاندن اتمی Din در دفتر.
    ///
    /// # Errors
    /// خطای SQL.
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

    /// خواندن یک درخواست برداشت با شناسه.
    ///
    /// # Errors
    /// خطای SQL.
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

    /// فهرست درخواست‌های برداشت یک حساب — جدیدترین اول (۵۰ آخر).
    ///
    /// # Errors
    /// خطای SQL.
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

    /// فهرست ادمین درخواست‌های برداشت — فیلتر اختیاری روی وضعیت.
    ///
    /// # Errors
    /// خطای SQL.
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

    /// تعیین تکلیف یک‌طرفهٔ درخواست برداشت (`PENDING` → `CONFIRMED` یا `REJECTED`).
    ///
    /// # Errors
    /// ناشناخته → `EscrowNotFound`؛ قبلاً تصمیم‌گرفته‌شده → `InvalidTransition`.
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

/// ردیف درخواست برداشت کریپتو — جدول `"WithdrawalRequest"` (فاز ۶).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithdrawalRequestRow {
    /// شناسهٔ یکتا (`wdr:{uuid}`)
    pub id: String,
    /// حساب شهروند (`citizen:{tg_id}`)
    pub account_id: String,
    /// زنجیره مقصد (`TON`/`TRX`/`SOL`/`BSC`)
    pub chain: String,
    /// دارایی (`TON`/`USDT`)
    pub asset: String,
    /// آدرس مقصد اعتبارسنجی‌شده
    pub destination_address: String,
    /// مبلغ به دیین (در لحظهٔ ثبت از حساب کسر می‌شود)
    pub amount_din: i64,
    /// هش تراکنش پرداخت روی زنجیره (پس از تأیید ادمین)
    pub tx_hash: Option<String>,
    /// وضعیت (`PENDING`/`CONFIRMED`/`REJECTED`)
    pub status: String,
    /// زمان ثبت (Unix epoch ثانیه)
    pub created_at: i64,
    /// زمان تصمیم ادمین (Unix epoch ثانیه)
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
        // همان زنجیره — آدرس جدید جایگزین، ردیف تکراری نمی‌شود
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

        // همان هش روی همان زنجیره — replay رد می‌شود
        assert!(
            store
                .insert_usdt_deposit(&deposit("udep:2", "TON", "hash111"))
                .await
                .is_err()
        );
        // همان هش روی زنجیرهٔ دیگر — مجاز (فضای هش زنجیره‌ها جدا است)
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

        // تصمیم دوباره — ۴۲۲
        assert!(
            store
                .set_usdt_deposit_status("udep:1", "REJECTED", "admin")
                .await
                .is_err()
        );
        // ناشناخته — 404-style دامنه
        let missing = store
            .set_usdt_deposit_status("udep:x", "CONFIRMED", "admin")
            .await
            .unwrap_err();
        assert!(matches!(missing, DbError::Domain(_)));
    }
}
