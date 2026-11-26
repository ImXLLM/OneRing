//! M5 governance layer — parliament bills and votes.
//!
//! All queries pass through [`V3Store`]. Transition logic (what becomes PASSED,
//! the Article 7 gate, one vote per voter) lives in `kian-nation` — here only
//! honest persistence: insert, read, close (only from OPEN), execute (only from PASSED).
//!
//! Table contracts: `crates/db/migrations/0004_m5_nation_wallet.sql`.

use crate::error::{DbError, DbResult};
use crate::store::{V3Store, now_iso};
use kian_domain::{DomainError, DomainErrorCode};

/// Bill row — one-to-one with the `Bill` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillRow {
    /// ID (`bill:…`)
    pub id: String,
    /// `escrowRelease` / `treasuryMove`
    pub kind: String,
    /// Escrow or destination account ID
    pub subject: String,
    /// Din amount (0 for unpaid)
    pub amount_din: i64,
    /// `release` / `refund` — escrowRelease only
    pub resolution: Option<String>,
    /// Title
    pub title: String,
    /// `OPEN` / `PASSED` / `REJECTED` / `EXECUTED`
    pub status: String,
    /// Deterministic reason after closing
    pub decision_reason: Option<String>,
    /// ISO opened-at time
    pub opened_at: String,
    /// ISO closed-at time
    pub closed_at: Option<String>,
    /// ISO executed-at time
    pub executed_at: Option<String>,
    /// Article 7 — amount ≥ 50,000 Đ?
    pub hitl_required: bool,
    /// Yes count after closing
    pub votes_yes: i64,
    /// No count after closing
    pub votes_no: i64,
    /// Human yes count after closing
    pub human_yes: i64,
}

/// Vote row — one-to-one with the `BillVote` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteRow {
    /// Target bill
    pub bill_id: String,
    /// Voter (`citizen:…` or `agent:…`)
    pub voter: String,
    /// Human? (Article 7 gate)
    pub is_human: bool,
    /// In favor?
    pub approve: bool,
    /// ISO voted-at time
    pub created_at: String,
}

const BILL_SELECT: &str = r#"SELECT "id","kind","subject","amountDin","resolution","title","status","decisionReason","openedAt","closedAt","executedAt","hitlRequired","votesYes","votesNo","humanYes" FROM "Bill""#;

#[allow(clippy::needless_pass_by_value)] // sqlx pattern: Row ownership comes from the fetch result
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

#[allow(clippy::needless_pass_by_value)] // sqlx pattern
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
    /// Insert open bill — fixed ID (`bill:{uuid}`); status always starts `OPEN`.
    ///
    /// # Errors
    /// SQL error.
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

    /// Read one bill.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_bill(&self, id: &str) -> DbResult<Option<BillRow>> {
        // Audited: const prefix + bind — no user input in the string
        let sql = format!("{BILL_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(bill_row_from))
    }

    /// List bills — optional status filter; newest first (max 100).
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_bills(&self, status: Option<&str>) -> DbResult<Vec<BillRow>> {
        // Audited: const prefix + bind — no user input in the string
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

    /// Cast vote — the PRIMARY KEY (billId, voter) constraint closes the race;
    /// collision → `InvalidTransition` (honest message: already voted).
    ///
    /// # Errors
    /// SQL error or a duplicate vote.
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
            // PK constraint (billId, voter) — the honest "already voted" message
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

    /// Votes of one bill in chronological order.
    ///
    /// # Errors
    /// SQL error.
    pub async fn votes_for(&self, bill_id: &str) -> DbResult<Vec<VoteRow>> {
        // Audited: const prefix + bind
        let sql =
            format!("{VOTE_SELECT} WHERE \"billId\" = ? ORDER BY \"createdAt\" ASC, \"voter\" ASC");
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(bill_id)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.into_iter().map(vote_row_from).collect())
    }

    /// Has this voter voted on this bill? — honest 409 precheck.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_vote(&self, bill_id: &str, voter: &str) -> DbResult<Option<VoteRow>> {
        let sql = format!("{VOTE_SELECT} WHERE \"billId\" = ? AND \"voter\" = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(bill_id)
            .bind(voter)
            .fetch_optional(self.pool_ref())
            .await?;
        Ok(row.map(vote_row_from))
    }

    /// Close a bill with a deterministic tally — only from `OPEN` (a one-way transition).
    ///
    /// # Errors
    /// Unknown bill → `EscrowNotFound`; non-OPEN → `InvalidTransition`.
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

    /// Execute flag — only from `PASSED`; once and for all.
    ///
    /// # Errors
    /// Unknown → `EscrowNotFound`; non-PASSED → `InvalidTransition`.
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
            title: "\u{62c}\u{627}\u{628}\u{647}‌\u{62c}\u{627}\u{6cc}\u{6cc} \u{62e}\u{632}\u{627}\u{646}\u{647}".into(),
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

        // Votes — duplicates rejected (PK constraint with an honest message)
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
        assert!(matches!(dup, DbError::Domain(_)), "\u{62a}\u{6a9}\u{631}\u{627}\u{631}\u{6cc} → \u{62e}\u{637}\u{627}\u{6cc} \u{62f}\u{627}\u{645}\u{646}\u{647}");

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

        // Double close — 422
        assert!(
            store
                .close_bill("bill:1", "REJECTED", "majority_no", 1, 2, 0)
                .await
                .is_err()
        );

        let executed = store.mark_bill_executed("bill:1").await.unwrap();
        assert_eq!(executed.status, "EXECUTED");
        assert!(executed.executed_at.is_some());
        // Double execute — 422
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
