//! لایهٔ هویت M3 — کاربران تلگرام، نشست‌های bearer و واریزهای TON.
//!
//! همهٔ کوئری‌ها از همان [`V3Store`] می‌گذرند (نقطهٔ تنها اتصال دامنه به
//! SQLite). هویت «پول» نمی‌داند — تراز همیشه از `Posting` مشتق می‌شود
//! (ADR-003)؛ اینجا فقط ردیف‌های هویت/نشست/اثبات واریز زندگی می‌کنند.
//!
//! قراردادهای جدول‌ها: `crates/db/migrations/0002_m3_sessions.sql`.
//!
//! نکتهٔ sqlx 0.9 — «رشتهٔ SQL داینامیک باید حسابرسی شود»: در ۴ کوئریِ
//! [`USER_SELECT`]/[`DEPOSIT_SELECT`] بخش داینامیک فقط پیشوند ثابت کامپایل‌تایم
//! است (هیچ ورودی کاربری در رشته interpolate نمی‌شود؛ همهٔ مقادیر با `?`
//! bind می‌شوند) — پس طبق راهنمای خود sqlx با `AssertSqlSafe` حسابرسی‌شده
//! علامت خورده‌اند.
use crate::error::{DbError, DbResult};
use crate::store::{now_iso, V3Store};

/// پیشوند حساب شهروند — هویت تلگرام به این حساب دفتر گره می‌خورد.
pub const CITIZEN_PREFIX: &str = "citizen:";

/// حساب دفتر متناظر با هویت تلگرام — `citizen:{telegram_id}`.
#[must_use]
pub fn citizen_account_id(telegram_id: &str) -> String {
    format!("{CITIZEN_PREFIX}{telegram_id}")
}

/// ردیف کاربر مینی‌اپ — یک‌به‌یک با جدول `User`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRow {
    /// شناسه پایدار = حساب شهروند (`citizen:{telegramId}`)
    pub id: String,
    /// شناسهٔ عددی تلگرام به‌صورت رشته
    pub telegram_id: String,
    /// یوزرنیم بدون `@` — اختیاری
    pub username: Option<String>,
    /// آدرس ولت TON متصل — اختیاری
    pub ton_address: Option<String>,
    /// حساب دفتر دوطرفهٔ متصل
    pub account_id: String,
    /// زمان ثبت ISO
    pub created_at: String,
}

/// ردیف واریز TON — اثبات کاربر تا راستی‌آزمایی ادمین.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TonDepositRow {
    /// شناسه (`dep:{uuid}`)
    pub id: String,
    /// کاربر ثبت‌کننده
    pub user_id: String,
    /// هش تراکنش زنجیره — UNIQUE (درس replay)
    pub tx_hash: String,
    /// آدرس فرستنده
    pub wallet_address: String,
    /// مبلغ TON رشتهٔ دهدهی
    pub amount_ton: String,
    /// `PENDING` / `CONFIRMED` / `REJECTED`
    pub status: String,
    /// زمان ثبت ISO
    pub created_at: String,
    /// زمان تصمیم ISO — تا قبل از تکلیف `None`
    pub confirmed_at: Option<String>,
    /// تصمیم‌گیرنده (ادمین) — تا قبل از تکلیف `None`
    pub confirmed_by: Option<String>,
}

const USER_SELECT: &str = r#"SELECT "id","telegramId","username","tonAddress","accountId","createdAt" FROM "User""#;

#[allow(clippy::needless_pass_by_value)] // الگوی sqlx: مالکیت Row از نتیجهٔ fetch می‌آید و مصرف می‌شود
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

#[allow(clippy::needless_pass_by_value)] // الگوی sqlx: مالکیت Row از نتیجهٔ fetch می‌آید و مصرف می‌شود
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
    /// ثبت‌نام/به‌روزرسانی شهروند تلگرام — idempotent روی `telegramId`.
    ///
    /// حساب دفتر `citizen:{telegramId}` اگر نبود ساخته می‌شود (`kind=User`)؛
    /// یوزرنیم جدیدتر جایگزین می‌شود تا تغییر نام تلگرام هم می‌آید.
    ///
    /// # Errors
    /// خطای SQL یا خطای ساخت حساب دفتر.
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
               ON CONFLICT("telegramId") DO UPDATE SET "username" = excluded."username""#,
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

    /// خواندن کاربر با شناسه.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn get_user(&self, id: &str) -> DbResult<Option<UserRow>> {
        // حسابرسی‌شده: پیشوند const + bind — بدون ورودی کاربر در رشته
        let sql = format!("{USER_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(user_row_from))
    }

    /// اتصال آدرس TON به هویت — پس از اتصال TON Connect.
    ///
    /// # Errors
    /// کاربر ناشناخته → `USER_NOT_FOUND`.
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

    /// باز کردن نشست — توکن اپاک با انقضای صریح.
    ///
    /// # Errors
    /// خطای SQL.
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

    /// کاربرِ نشست معتبر — `None` یعنی توکن ناشناخته یا منقضی (fail-closed).
    ///
    /// # Errors
    /// خطای SQL.
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

    /// حذف نشست (خروج) — صدا زدن دوباره بی‌اثر و بی‌خطا.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn delete_session(&self, token: &str) -> DbResult<()> {
        sqlx::query(r#"DELETE FROM "Session" WHERE "token" = ?"#)
            .bind(token)
            .execute(self.pool_ref())
            .await?;
        Ok(())
    }

    /// هرس نشست‌های منقضی — نگهداری جدول سبک (sweep دوره‌ای).
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn sweep_expired_sessions(&self) -> DbResult<u64> {
        let res = sqlx::query(r#"DELETE FROM "Session" WHERE "expiresAt" <= ?"#)
            .bind(now_iso())
            .execute(self.pool_ref())
            .await?;
        Ok(res.rows_affected())
    }

    /// ثبت واریز TON — همان `txHash` دوبار پذیرفته نمی‌شود (قید UNIQUE جدول؛
    /// فراخواننده پیش‌چک `get_ton_deposit_by_hash` می‌کند تا پیام صادقانه بدهد).
    ///
    /// # Errors
    /// هش تکراری هم‌زمان → خطای UNIQUE sqlx؛ خطای SQL.
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

    /// جست‌وجوی واریز با هش زنجیره — پیش‌چک replay ثبت دوباره.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn get_ton_deposit_by_hash(&self, tx_hash: &str) -> DbResult<Option<TonDepositRow>> {
        // حسابرسی‌شده: پیشوند const + bind — بدون ورودی کاربر در رشته
        let sql = format!("{DEPOSIT_SELECT} WHERE \"txHash\" = ?");
        let row =
            sqlx::query(sqlx::AssertSqlSafe(sql)).bind(tx_hash).fetch_optional(self.pool_ref()).await?;
        Ok(row.map(deposit_row_from))
    }

    /// واریزهای یک کاربر — جدیدترین اول.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_ton_deposits_for(&self, user_id: &str) -> DbResult<Vec<TonDepositRow>> {
        // حسابرسی‌شده: پیشوند const + bind — بدون ورودی کاربر در رشته
        let sql = format!(
            r#"{DEPOSIT_SELECT} WHERE "userId" = ? ORDER BY "createdAt" DESC LIMIT 50"#
        );
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user_id)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.into_iter().map(deposit_row_from).collect())
    }

    /// خواندن یک واریز.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn get_ton_deposit(&self, id: &str) -> DbResult<Option<TonDepositRow>> {
        // حسابرسی‌شده: پیشوند const + bind — بدون ورودی کاربر در رشته
        let sql = format!("{DEPOSIT_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(deposit_row_from))
    }

    /// تعیین تکلیف واریز — فقط از `PENDING` (گذار یک‌طرفه).
    ///
    /// # Errors
    /// واریز ناشناخته → `NOT_FOUND`؛ وضعیت غیر-PENDING → `INVALID_TRANSITION`.
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
        // حساب دفتر واقعاً ساخته شده — تراز صفر
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

        // همان هش دوباره — replay رد می‌شود
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

        // گذار دوباره از PENDING ناممکن است
        assert!(store.set_ton_deposit_status("dep:1", "REJECTED", "admin").await.is_err());
    }
}
