//! لایهٔ حاکمیت M5 — لایحه‌ها و رأی‌های پارلمان.
//!
//! همهٔ کوئری‌ها از [`V3Store`] می‌گذرند. منطق گذارها (چه چیزی PASSED می‌شود،
//! گیت مادهٔ ۷، یک رأی به ازای هر رأی‌دهنده) در `kian-nation` است — اینجا فقط
//! ماندگاری صادقانه: درج، خواندن، بستن (فقط از OPEN)، اجرا (فقط از PASSED).
//!
//! قراردادهای جدول‌ها: `crates/db/migrations/0004_m5_nation_wallet.sql`.

use crate::error::{DbError, DbResult};
use crate::store::{V3Store, now_iso};
use kian_domain::{DomainError, DomainErrorCode};

/// ردیف لایحه — یک‌به‌یک با جدول `Bill`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillRow {
    /// شناسه (`bill:…`)
    pub id: String,
    /// `escrowRelease` / `treasuryMove`
    pub kind: String,
    /// شناسهٔ اسکرو یا حساب مقصد
    pub subject: String,
    /// مبلغ دیین (۰ برای بی‌مبلغ)
    pub amount_din: i64,
    /// `release` / `refund` — فقط escrowRelease
    pub resolution: Option<String>,
    /// عنوان
    pub title: String,
    /// `OPEN` / `PASSED` / `REJECTED` / `EXECUTED`
    pub status: String,
    /// دلیل قطعی پس از بستن
    pub decision_reason: Option<String>,
    /// زمان باز شدن ISO
    pub opened_at: String,
    /// زمان بستن ISO
    pub closed_at: Option<String>,
    /// زمان اجرا ISO
    pub executed_at: Option<String>,
    /// مادهٔ ۷ — مبلغ ≥ ۵۰,۰۰۰ Đ؟
    pub hitl_required: bool,
    /// شمارش موافق پس از بستن
    pub votes_yes: i64,
    /// شمارش مخالف پس از بستن
    pub votes_no: i64,
    /// شمارش موافق انسانی پس از بستن
    pub human_yes: i64,
}

/// ردیف رأی — یک‌به‌یک با جدول `BillVote`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteRow {
    /// لایحهٔ هدف
    pub bill_id: String,
    /// رأی‌دهنده (`citizen:…` یا `agent:…`)
    pub voter: String,
    /// انسان؟ (گیت مادهٔ ۷)
    pub is_human: bool,
    /// موافق؟
    pub approve: bool,
    /// زمان رأی ISO
    pub created_at: String,
}

const BILL_SELECT: &str = r#"SELECT "id","kind","subject","amountDin","resolution","title","status","decisionReason","openedAt","closedAt","executedAt","hitlRequired","votesYes","votesNo","humanYes" FROM "Bill""#;

#[allow(clippy::needless_pass_by_value)] // الگوی sqlx: مالکیت Row از نتیجهٔ fetch می‌آید
fn bill_row_from(row: sqlx::sqlite::SqliteRow) -> BillRow {
    use sqlx::Row as _;
    BillRow {
        id: row.get("id"),
        kind: row.get("kind"),
        subject: row.get("subject"),
        amount_din: row.get("amountDin"),
        resolution: row.get("resolution"),
        title: row.get("title"),
        status: row.get("status"),
        decision_reason: row.get("decisionReason"),
        opened_at: row.get("openedAt"),
        closed_at: row.get("closedAt"),
        executed_at: row.get("executedAt"),
        hitl_required: row.get::<i64, _>("hitlRequired") != 0,
        votes_yes: row.get("votesYes"),
        votes_no: row.get("votesNo"),
        human_yes: row.get("humanYes"),
    }
}

const VOTE_SELECT: &str =
    r#"SELECT "billId","voter","isHuman","approve","createdAt" FROM "BillVote""#;

#[allow(clippy::needless_pass_by_value)] // الگوی sqlx
fn vote_row_from(row: sqlx::sqlite::SqliteRow) -> VoteRow {
    use sqlx::Row as _;
    VoteRow {
        bill_id: row.get("billId"),
        voter: row.get("voter"),
        is_human: row.get::<i64, _>("isHuman") != 0,
        approve: row.get::<i64, _>("approve") != 0,
        created_at: row.get("createdAt"),
    }
}

impl V3Store {
    /// درج لایحهٔ باز — id ثابت (`bill:{uuid}`)؛ وضعیت همیشه `OPEN` شروع می‌شود.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn insert_bill(&self, b: &BillRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "Bill"
               ("id","kind","subject","amountDin","resolution","title","status","openedAt","hitlRequired","votesYes","votesNo","humanYes")
               VALUES (?,?,?,?,?,?,'OPEN',?,?,0,0,0)"#,
        )
        .bind(&b.id)
        .bind(&b.kind)
        .bind(&b.subject)
        .bind(b.amount_din)
        .bind(&b.resolution)
        .bind(&b.title)
        .bind(&b.opened_at)
        .bind(b.hitl_required)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext("server", "bill.open", Some(&b.id), None)
            .await?;
        Ok(())
    }

    /// خواندن یک لایحه.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn get_bill(&self, id: &str) -> DbResult<Option<BillRow>> {
        // حسابرسی‌شده: پیشوند const + bind — بدون ورودی کاربر در رشته
        let sql = format!("{BILL_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(bill_row_from))
    }

    /// فهرست لایحه‌ها — اختیاری فیلتر وضعیت؛ جدیدترین اول (حداکثر ۱۰۰).
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_bills(&self, status: Option<&str>) -> DbResult<Vec<BillRow>> {
        // حسابرسی‌شده: پیشوند const + bind — بدون ورودی کاربر در رشته
        let sql = match status {
            Some(_) => {
                format!(r#"{BILL_SELECT} WHERE "status" = ? ORDER BY "openedAt" DESC LIMIT 100"#)
            }
            None => format!(r#"{BILL_SELECT} ORDER BY "openedAt" DESC LIMIT 100"#),
        };
        let mut q = sqlx::query(sqlx::AssertSqlSafe(sql));
        if let Some(s) = status {
            q = q.bind(s);
        }
        let rows = q.fetch_all(self.pool_ref()).await?;
        Ok(rows.into_iter().map(bill_row_from).collect())
    }

    /// ثبت رأی — قید PRIMARY KEY (billId, voter) مسابقهٔ هم‌زمانی را می‌بندد؛
    /// برخورد → `InvalidTransition` (پیام صادقانه: قبلاً رأی داده).
    ///
    /// # Errors
    /// خطای SQL یا رأی تکراری.
    pub async fn insert_vote(&self, v: &VoteRow) -> DbResult<()> {
        let res = sqlx::query(
            r#"INSERT INTO "BillVote" ("billId","voter","isHuman","approve","createdAt")
               VALUES (?,?,?,?,?)"#,
        )
        .bind(&v.bill_id)
        .bind(&v.voter)
        .bind(v.is_human)
        .bind(v.approve)
        .bind(&v.created_at)
        .execute(self.pool_ref())
        .await;
        if let Err(sqlx::Error::Database(db)) = &res {
            // قید PK (billId, voter) — پیام صادقانهٔ «قبلاً رأی داده»
            let is_dup = db.constraint().is_some_and(|c| c.contains("BillVote"))
                || db.message().contains("UNIQUE constraint failed: BillVote");
            if is_dup {
                return Err(DbError::Domain(DomainError::new(
                    DomainErrorCode::InvalidTransition,
                    format!("Voter {} already voted on bill {}", v.voter, v.bill_id),
                )));
            }
        }
        res?;
        self.audit_ext("server", "bill.vote", Some(&v.bill_id), Some(&v.voter))
            .await?;
        Ok(())
    }

    /// رأی‌های یک لایحه به ترتیب زمانی.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn votes_for(&self, bill_id: &str) -> DbResult<Vec<VoteRow>> {
        // حسابرسی‌شده: پیشوند const + bind
        let sql =
            format!("{VOTE_SELECT} WHERE \"billId\" = ? ORDER BY \"createdAt\" ASC, \"voter\" ASC");
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(bill_id)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.into_iter().map(vote_row_from).collect())
    }

    /// آیا این رأی‌دهنده به این لایحه رأی داده؟ — پیش‌چک صادقانهٔ ۴۰۹.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn get_vote(&self, bill_id: &str, voter: &str) -> DbResult<Option<VoteRow>> {
        let sql = format!("{VOTE_SELECT} WHERE \"billId\" = ? AND \"voter\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(bill_id)
            .bind(voter)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(vote_row_from))
    }

    /// بستن لایحه با شمارش قطعی — فقط از `OPEN` (گذار یک‌طرفه).
    ///
    /// # Errors
    /// لایحهٔ ناشناخته → `EscrowNotFound`؛ غیر-OPEN → `InvalidTransition`.
    pub async fn close_bill(
        &self,
        id: &str,
        status: &str,
        decision_reason: &str,
        votes_yes: i64,
        votes_no: i64,
        human_yes: i64,
    ) -> DbResult<BillRow> {
        let res = sqlx::query(
            r#"UPDATE "Bill" SET "status" = ?, "decisionReason" = ?, "closedAt" = ?,
               "votesYes" = ?, "votesNo" = ?, "humanYes" = ?
               WHERE "id" = ? AND "status" = 'OPEN'"#,
        )
        .bind(status)
        .bind(decision_reason)
        .bind(now_iso())
        .bind(votes_yes)
        .bind(votes_no)
        .bind(human_yes)
        .bind(id)
        .execute(self.pool_ref())
        .await?;
        if res.rows_affected() == 0 {
            return Err(match self.get_bill(id).await? {
                None => DbError::Domain(DomainError::new(
                    DomainErrorCode::EscrowNotFound,
                    format!("Bill {id} does not exist"),
                )),
                Some(_) => DbError::Domain(DomainError::new(
                    DomainErrorCode::InvalidTransition,
                    format!("Bill {id} is not OPEN anymore"),
                )),
            });
        }
        self.audit_ext("server", format!("bill.{status}").as_str(), Some(id), None)
            .await?;
        self.get_bill(id)
            .await?
            .ok_or_else(|| DbError::Config(format!("bill {id} vanished after close")))
    }

    /// علامت اجرا — فقط از `PASSED`؛ یک‌بار برای همیشه.
    ///
    /// # Errors
    /// ناشناخته → `EscrowNotFound`؛ غیر-PASSED → `InvalidTransition`.
    pub async fn mark_bill_executed(&self, id: &str) -> DbResult<BillRow> {
        let res = sqlx::query(
            r#"UPDATE "Bill" SET "status" = 'EXECUTED', "executedAt" = ?
               WHERE "id" = ? AND "status" = 'PASSED'"#,
        )
        .bind(now_iso())
        .bind(id)
        .execute(self.pool_ref())
        .await?;
        if res.rows_affected() == 0 {
            return Err(match self.get_bill(id).await? {
                None => DbError::Domain(DomainError::new(
                    DomainErrorCode::EscrowNotFound,
                    format!("Bill {id} does not exist"),
                )),
                Some(_) => DbError::Domain(DomainError::new(
                    DomainErrorCode::InvalidTransition,
                    format!("Bill {id} is not PASSED — execution refused"),
                )),
            });
        }
        self.audit_ext("server", "bill.executed", Some(id), None)
            .await?;
        self.get_bill(id)
            .await?
            .ok_or_else(|| DbError::Config(format!("bill {id} vanished after execute")))
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

    fn open_bill(id: &str) -> BillRow {
        BillRow {
            id: id.into(),
            kind: "treasuryMove".into(),
            subject: "citizen:1".into(),
            amount_din: 10_000,
            resolution: None,
            title: "جابه‌جایی خزانه".into(),
            status: "OPEN".into(),
            decision_reason: None,
            opened_at: now_iso(),
            closed_at: None,
            executed_at: None,
            hitl_required: false,
            votes_yes: 0,
            votes_no: 0,
            human_yes: 0,
        }
    }

    #[tokio::test]
    async fn bill_lifecycle_open_close_execute() {
        let store = fresh_store().await;
        store.insert_bill(&open_bill("bill:1")).await.unwrap();

        // رأی‌ها — تکراری رد می‌شود (قید PK با پیام صادقانه)
        store
            .insert_vote(&VoteRow {
                bill_id: "bill:1".into(),
                voter: "citizen:1".into(),
                is_human: true,
                approve: true,
                created_at: now_iso(),
            })
            .await
            .unwrap();
        let dup = store
            .insert_vote(&VoteRow {
                bill_id: "bill:1".into(),
                voter: "citizen:1".into(),
                is_human: true,
                approve: false,
                created_at: now_iso(),
            })
            .await
            .unwrap_err();
        assert!(matches!(dup, DbError::Domain(_)), "تکراری → خطای دامنه");

        let votes = store.votes_for("bill:1").await.unwrap();
        assert_eq!(votes.len(), 1);
        assert!(
            store
                .get_vote("bill:1", "citizen:1")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            store
                .get_vote("bill:1", "citizen:2")
                .await
                .unwrap()
                .is_none()
        );

        let closed = store
            .close_bill("bill:1", "PASSED", "passed", 1, 0, 1)
            .await
            .unwrap();
        assert_eq!(closed.status, "PASSED");
        assert_eq!(closed.decision_reason.as_deref(), Some("passed"));

        // بستن دوباره — ۴۲۲
        assert!(
            store
                .close_bill("bill:1", "REJECTED", "majority_no", 1, 2, 0)
                .await
                .is_err()
        );

        let executed = store.mark_bill_executed("bill:1").await.unwrap();
        assert_eq!(executed.status, "EXECUTED");
        assert!(executed.executed_at.is_some());
        // اجرای دوباره — ۴۲۲
        assert!(store.mark_bill_executed("bill:1").await.is_err());
    }

    #[tokio::test]
    async fn list_bills_filters_by_status() {
        let store = fresh_store().await;
        store.insert_bill(&open_bill("bill:a")).await.unwrap();
        store.insert_bill(&open_bill("bill:b")).await.unwrap();
        store
            .close_bill("bill:b", "REJECTED", "quorum_not_met", 0, 0, 0)
            .await
            .unwrap();

        let all = store.list_bills(None).await.unwrap();
        assert_eq!(all.len(), 2);
        let open = store.list_bills(Some("OPEN")).await.unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, "bill:a");
        let rejected = store.list_bills(Some("REJECTED")).await.unwrap();
        assert_eq!(rejected[0].id, "bill:b");
    }
}
