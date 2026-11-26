//! Persistence adapter — the "only" place the pure domain touches storage (ADR-001/003).
//!
//! Design (exactly the `back/server/src/persistence.ts` pattern, with sqlx):
//!  - the `Posting` table is the single source of truth; balances are always derived
//!    via `SUM(amount)` and never stored anywhere.
//!  - every command runs inside one interactive transaction:
//!    1. a full ledger-state load — accounts, derived balances, transactions, idempotency keys
//!    2. run the pure domain closure (no IO — the same Phase 1 functions)
//!    3. insert the transaction + rows, upsert the escrow, outbox and audit events
//!  - idempotency: an in-transaction replay precheck short-circuits the normal path;
//!    the concurrency race is closed by the UNIQUE constraint on `Tx.idempotencyKey` — i.e.
//!    "a retry race never moves money twice". On collision, the request becomes
//!    an honest replay read (the TS P2002 behavior).
//!
//! Scale note: v3 loads the entire state per command (correctness first, speed later).
//! At the current scale (~1.5k transactions) this stays in the microsecond range; a separate
//! read model is warranted only once the ledger grows by orders of magnitude.

use crate::error::{DbError, DbResult};
use crate::wire;
use kian_domain::{
    escrow_account_id, Account, AccountKind, Din, DomainEvent, DomainResult, Escrow,
    EscrowRegistry, EscrowStatus, LedgerState, PostedTransaction, Posting, BURN_ACCOUNT_ID,
    MINT_ACCOUNT_ID, TREASURY_ACCOUNT_ID,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Row, Sqlite, SqliteConnection, SqlitePool,
};
use std::{path::Path, time::Duration};

/// Safe grace period before rejection due to the SQLite writer lock.
const BUSY_TIMEOUT: Duration = Duration::from_secs(10);

/// The v3 store adapter — the single point where the domain touches SQLite.
#[derive(Debug, Clone)]
pub struct V3Store {
    pool: SqlitePool,
    /// Database file path — `None` means in-memory (the VACUUM INTO backup does not apply).
    file_path: Option<String>,
}

/// Domain closure output — the same `CommandOutcome` as in TS.
#[derive(Debug, Clone)]
pub struct CommandOutcome {
    /// New ledger state (domain output)
    pub ledger: LedgerState,
    /// New transaction (if the command moved money)
    pub tx: Option<PostedTransaction>,
    /// Touched escrow (if the command was an escrow one)
    pub escrow: Option<Escrow>,
    /// Domain events for the outbox
    pub events: Vec<DomainEvent>,
}

/// Command execution options — the same `ApplyOptions` as in TS.
#[derive(Debug, Clone)]
pub struct ApplyOptions {
    /// Executor (for audit)
    pub actor: String,
    /// Command name (for audit)
    pub action: String,
    /// Optional target (for audit)
    pub target: Option<String>,
    /// Target escrow — documented in the reports
    pub escrow_id: Option<String>,
    /// Idempotency key — becomes UNIQUE on `Tx`
    pub idempotency_key: Option<String>,
}

/// Command execution result — the same `ApplyResult` as in TS.
#[derive(Debug, Clone)]
pub struct ApplyResult {
    /// Was this a replay? (money did not move again)
    pub replayed: bool,
    /// The transaction (new, or the original in a replay)
    pub tx: Option<PostedTransaction>,
    /// The escrow (if related to the command)
    pub escrow: Option<Escrow>,
    /// Ledger state after execution
    pub ledger: LedgerState,
}

/// Account balance + metadata — for reports and the dashboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountBalance {
    /// Account ID
    pub id: String,
    /// Canonical type string (`user`, `escrow`, …)
    pub kind: String,
    /// Optional label
    pub label: Option<String>,
    /// Balance derived from `SUM(Posting.amount)`
    pub balance: i64,
    /// ISO creation time
    pub created_at: String,
}

/// Economy summary — the same `economySummary` as in TS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EconomySummary {
    /// Money held by citizens + escrow + treasury
    pub circulating_supply: i64,
    /// Treasury balance
    pub treasury_balance: i64,
    /// Money locked in escrows
    pub escrow_locked: i64,
    /// Σ of user wallets — the 187,672 Đ acceptance figure
    pub users_sum: i64,
    /// Mint account balance (never positive)
    pub mint_balance: i64,
    /// Burn account balance (never negative)
    pub burn_balance: i64,
    /// Current-year mint under UTC
    pub minted_this_year: i64,
    /// Σ of all balances — invariant: always 0
    pub trial_sum: i64,
    /// Is `trial_sum == 0`?
    pub balanced: bool,
    /// Number of open escrows (CREATED/FUNDED/DELIVERED/DISPUTED)
    pub open_escrows: i64,
}

/// Wallet-history row — fuel for the `wallet_history` skill (Phase 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentPosting {
    /// Transaction ID
    pub tx_id: String,
    /// Signed amount
    pub amount: i64,
    /// Note
    pub memo: Option<String>,
    /// ISO time
    pub at: String,
}

/// Outbox row for dispatch (Phase 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxRow {
    /// Outbox row ID
    pub id: String,
    /// Event `kind` string
    pub kind: String,
    /// Event JSON (the TS wire shape)
    pub payload: String,
    /// ISO creation time
    pub created_at: String,
    /// Dispatched?
    pub published: bool,
}

/// ISO-8601 UTC time with milliseconds — the domain `at` contract (`EPOCH_ISO` format).
#[must_use]
pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// UTC year start — for the annual mint cap.
fn year_start_iso() -> DbResult<String> {
    use chrono::{Datelike, TimeZone};
    let y = chrono::Utc::now().year();
    let start = chrono::Utc
        .with_ymd_and_hms(y, 1, 1, 0, 0, 0)
        .single()
        .ok_or_else(|| DbError::Config(format!("invalid year start for {y}")))?;
    Ok(start.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

fn kind_from_str(s: &str) -> DbResult<AccountKind> {
    match s {
        "user" => Ok(AccountKind::User),
        "escrow" => Ok(AccountKind::Escrow),
        "treasury" => Ok(AccountKind::Treasury),
        "mint" => Ok(AccountKind::Mint),
        "burn" => Ok(AccountKind::Burn),
        other => Err(DbError::Corrupt(format!("unknown account kind {other:?}"))),
    }
}

fn status_from_str(s: &str) -> DbResult<EscrowStatus> {
    match s {
        "CREATED" => Ok(EscrowStatus::Created),
        "FUNDED" => Ok(EscrowStatus::Funded),
        "DELIVERED" => Ok(EscrowStatus::Delivered),
        "RELEASED" => Ok(EscrowStatus::Released),
        "DISPUTED" => Ok(EscrowStatus::Disputed),
        "REFUNDED" => Ok(EscrowStatus::Refunded),
        "CANCELLED" => Ok(EscrowStatus::Cancelled),
        "EXPIRED" => Ok(EscrowStatus::Expired),
        other => Err(DbError::Corrupt(format!("unknown escrow status {other:?}"))),
    }
}

/// Is this error a UNIQUE violation on `Tx.idempotencyKey`?
fn is_idempotency_conflict(err: &DbError) -> bool {
    match err {
        DbError::Sql(sqlx::Error::Database(db)) => {
            db.constraint().is_some_and(|c| c == "Tx.idempotencyKey")
                || db.message().contains("UNIQUE constraint failed: Tx.idempotencyKey")
        }
        _ => false,
    }
}

// ── Shared queries — on a specific connection (inside a transaction or raw) ─────────────────

pub(crate) async fn load_ledger_state(exec: &mut SqliteConnection) -> DbResult<LedgerState> {
    let mut accounts = std::collections::BTreeMap::new();
    let mut balances = std::collections::BTreeMap::new();
    let rows = sqlx::query(r#"SELECT "id", "kind", "label" FROM "Account" ORDER BY "id""#)
        .fetch_all(&mut *exec)
        .await?;
    for r in rows {
        let id: String = r.get("id");
        let kind = kind_from_str(&r.get::<String, _>("kind"))?;
        let label: Option<String> = r.get("label");
        balances.insert(id.clone(), 0);
        accounts.insert(id.clone(), Account { id, kind, label });
    }
    // balances = derived from rows (ADR-003) — an account with no rows stays zero.
    let sums = sqlx::query(
        r#"SELECT "accountId", SUM("amount") AS "s" FROM "Posting" GROUP BY "accountId""#,
    )
    .fetch_all(&mut *exec)
    .await?;
    for r in sums {
        balances.insert(r.get::<String, _>("accountId"), r.get::<i64, _>("s"));
    }

    let mut txs = std::collections::BTreeMap::new();
    let mut idempotency = std::collections::BTreeMap::new();
    let tx_rows = sqlx::query(r#"SELECT "id", "idempotencyKey", "memo", "at" FROM "Tx""#)
        .fetch_all(&mut *exec)
        .await?;
    let mut by_tx: std::collections::BTreeMap<String, Vec<Posting>> =
        std::collections::BTreeMap::new();
    for r in sqlx::query(r#"SELECT "txId", "accountId", "amount" FROM "Posting" ORDER BY "id" ASC"#)
        .fetch_all(&mut *exec)
        .await?
    {
        by_tx
            .entry(r.get::<String, _>("txId"))
            .or_default()
            .push(Posting { account_id: r.get("accountId"), amount: r.get("amount") });
    }
    for r in tx_rows {
        let id: String = r.get("id");
        let key: Option<String> = r.get("idempotencyKey");
        if let Some(k) = &key {
            idempotency.insert(k.clone(), id.clone());
        }
        let postings = by_tx.remove(&id).unwrap_or_default();
        txs.insert(
            id.clone(),
            PostedTransaction {
                id,
                idempotency_key: key,
                postings,
                memo: r.get("memo"),
                at: r.get("at"),
            },
        );
    }
    Ok(LedgerState { accounts, balances, txs, idempotency })
}

pub(crate) async fn load_escrow_registry(
    exec: &mut SqliteConnection,
    escrow_id: Option<&str>,
) -> DbResult<EscrowRegistry> {
    // deliberate deviation from TS: the full registration is loaded (in TS only the
    // target escrow), so `EscrowAlreadyExists` is actually detectable in the domain. At the
    // current scale the escrow table is small.
    let rows = match escrow_id {
        Some(id) => {
            sqlx::query(r#"SELECT * FROM "Escrow" WHERE "id" = ?"#)
                .bind(id)
                .fetch_all(&mut *exec)
                .await?
        }
        None => sqlx::query(r#"SELECT * FROM "Escrow""#).fetch_all(&mut *exec).await?,
    };
    let mut escrows = std::collections::BTreeMap::new();
    for r in rows {
        let escrow = row_to_escrow(&r)?;
        escrows.insert(escrow.id.clone(), escrow);
    }
    Ok(EscrowRegistry { escrows })
}

pub(crate) fn row_to_escrow(r: &sqlx::sqlite::SqliteRow) -> DbResult<Escrow> {
    let affiliate_bps_raw: i64 = r.get("affiliateBps");
    let affiliate_bps = u16::try_from(affiliate_bps_raw).map_err(|_| {
        DbError::Corrupt(format!("invalid affiliateBps {affiliate_bps_raw} in Escrow"))
    })?;
    Ok(Escrow {
        id: r.get("id"),
        buyer_id: r.get("buyerId"),
        seller_id: r.get("sellerId"),
        amount: Din::new(r.get::<i64, _>("amount"))?,
        product_ref: r.get("productRef"),
        status: status_from_str(&r.get::<String, _>("status"))?,
        created_at: r.get("createdAt"),
        funded_tx_id: r.get("fundedTxId"),
        settle_tx_id: r.get("settleTxId"),
        dispute_reason: r.get("disputeReason"),
        delivery_ref: r.get("deliveryRef"),
        expires_at: r.get("expiresAt"),
        referrer: r.get("referrer"),
        affiliate_bps,
    })
}

pub(crate) async fn tx_by_key(
    exec: &mut SqliteConnection,
    key: &str,
) -> DbResult<Option<PostedTransaction>> {
    let Some(r) = sqlx::query(
        r#"SELECT "id", "idempotencyKey", "memo", "at" FROM "Tx" WHERE "idempotencyKey" = ?"#,
    )
    .bind(key)
    .fetch_optional(&mut *exec)
    .await?
    else {
        return Ok(None);
    };
    let id: String = r.get("id");
    let postings: Vec<Posting> = sqlx::query(
        r#"SELECT "accountId", "amount" FROM "Posting" WHERE "txId" = ? ORDER BY "id" ASC"#,
    )
    .bind(&id)
    .fetch_all(&mut *exec)
    .await?
    .into_iter()
    .map(|p| Posting { account_id: p.get("accountId"), amount: p.get("amount") })
    .collect();
    Ok(Some(PostedTransaction {
        idempotency_key: r.get("idempotencyKey"),
        postings,
        memo: r.get("memo"),
        at: r.get("at"),
        id,
    }))
}

pub(crate) async fn load_escrow_opt(
    exec: &mut SqliteConnection,
    escrow_id: Option<&str>,
) -> DbResult<Option<Escrow>> {
    let Some(id) = escrow_id else { return Ok(None) };
    sqlx::query(r#"SELECT * FROM "Escrow" WHERE "id" = ?"#)
        .bind(id)
        .fetch_optional(&mut *exec)
        .await?
        .as_ref()
        .map(row_to_escrow)
        .transpose()
}

pub(crate) async fn insert_account(
    exec: &mut SqliteConnection,
    id: &str,
    kind: AccountKind,
    label: Option<&str>,
) -> DbResult<()> {
    sqlx::query(
        r#"INSERT INTO "Account" ("id", "kind", "label", "createdAt") VALUES (?, ?, ?, ?)
           ON CONFLICT("id") DO NOTHING"#,
    )
    .bind(id)
    .bind(kind.as_str())
    .bind(label)
    .bind(now_iso())
    .execute(&mut *exec)
    .await?;
    Ok(())
}

pub(crate) async fn insert_tx_with_postings(
    exec: &mut SqliteConnection,
    txn: &PostedTransaction,
    idempotency_override: Option<&str>,
) -> DbResult<()> {
    let key = idempotency_override.map(str::to_owned).or_else(|| txn.idempotency_key.clone());
    sqlx::query(r#"INSERT INTO "Tx" ("id", "idempotencyKey", "memo", "at") VALUES (?, ?, ?, ?)"#)
        .bind(&txn.id)
        .bind(key)
        .bind(&txn.memo)
        .bind(&txn.at)
        .execute(&mut *exec)
        .await?;
    for p in &txn.postings {
        sqlx::query(r#"INSERT INTO "Posting" ("txId", "accountId", "amount") VALUES (?, ?, ?)"#)
            .bind(&txn.id)
            .bind(&p.account_id)
            .bind(p.amount)
            .execute(&mut *exec)
            .await?;
    }
    Ok(())
}

pub(crate) async fn upsert_escrow(exec: &mut SqliteConnection, escrow: &Escrow) -> DbResult<()> {
    insert_account(
        exec,
        &escrow_account_id(&escrow.id),
        AccountKind::Escrow,
        Some(&format!("escrow {}", escrow.id)),
    )
    .await?;
    sqlx::query(
        r#"INSERT INTO "Escrow" ("id", "buyerId", "sellerId", "amount", "productRef",
             "status", "createdAt", "fundedTxId", "settleTxId", "disputeReason",
             "deliveryRef", "expiresAt", "referrer", "affiliateBps")
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT("id") DO UPDATE SET
             "buyerId" = excluded."buyerId", "sellerId" = excluded."sellerId",
             "amount" = excluded."amount", "productRef" = excluded."productRef",
             "status" = excluded."status", "createdAt" = excluded."createdAt",
             "fundedTxId" = excluded."fundedTxId", "settleTxId" = excluded."settleTxId",
             "disputeReason" = excluded."disputeReason", "deliveryRef" = excluded."deliveryRef",
             "expiresAt" = excluded."expiresAt", "referrer" = excluded."referrer",
             "affiliateBps" = excluded."affiliateBps""#,
    )
    .bind(&escrow.id)
    .bind(&escrow.buyer_id)
    .bind(&escrow.seller_id)
    .bind(escrow.amount.as_i64())
    .bind(&escrow.product_ref)
    .bind(escrow.status.as_str())
    .bind(&escrow.created_at)
    .bind(&escrow.funded_tx_id)
    .bind(&escrow.settle_tx_id)
    .bind(&escrow.dispute_reason)
    .bind(&escrow.delivery_ref)
    .bind(&escrow.expires_at)
    .bind(&escrow.referrer)
    .bind(i64::from(escrow.affiliate_bps))
    .execute(&mut *exec)
    .await?;
    Ok(())
}

pub(crate) async fn insert_outbox_events(
    exec: &mut SqliteConnection,
    events: &[DomainEvent],
) -> DbResult<()> {
    for e in events {
        sqlx::query(
            r#"INSERT INTO "OutboxEvent" ("id", "kind", "payload", "createdAt", "published")
               VALUES (?, ?, ?, ?, 0)"#,
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(wire::event_kind(e))
        .bind(wire::event_payload(e))
        .bind(now_iso())
        .execute(&mut *exec)
        .await?;
    }
    Ok(())
}

pub(crate) async fn insert_audit(
    exec: &mut SqliteConnection,
    actor: &str,
    action: &str,
    target: Option<&str>,
    meta: Option<&str>,
) -> DbResult<()> {
    sqlx::query(
        r#"INSERT INTO "AuditLog" ("id", "actor", "action", "target", "meta", "at")
           VALUES (?, ?, ?, ?, ?, ?)"#,
    )
    .bind(uuid::Uuid::now_v7().to_string())
    .bind(actor)
    .bind(action)
    .bind(target)
    .bind(meta)
    .bind(now_iso())
    .execute(&mut *exec)
    .await?;
    Ok(())
}

/// Insert a domain command's output inside an open transaction — in Phase 4 the HITL decision path
/// ("approve ⇒ settle in the same transaction") also goes through this single point.
pub(crate) async fn persist_outcome(
    exec: &mut SqliteConnection,
    opts: &ApplyOptions,
    outcome: &CommandOutcome,
) -> DbResult<()> {
    if let Some(escrow) = &outcome.escrow {
        upsert_escrow(exec, escrow).await?;
    }
    if let Some(txn) = &outcome.tx {
        // the command key takes precedence over the domain key — the same `options.idempotencyKey ?? ...` as in TS
        insert_tx_with_postings(exec, txn, opts.idempotency_key.as_deref()).await?;
    }
    crate::outbox::enqueue(exec, &outcome.events).await?;
    let meta = outcome.tx.as_ref().map(|t| serde_json::json!({ "txId": t.id }).to_string());
    crate::outbox::audit(exec, &opts.actor, &opts.action, opts.target.as_deref(), meta.as_deref())
        .await?;
    Ok(())
}

/// Safely roll back an open transaction after an error — the rollback error is only logged so the original error is never hidden.
async fn rollback_quietly(conn: &mut SqliteConnection) {
    if let Err(err) = sqlx::query("ROLLBACK").execute(&mut *conn).await {
        tracing::warn!(error = %err, "sqlite ROLLBACK failed after error");
    }
}

impl V3Store {
    /// Open by target URL — `file:./db/v3.db`, `sqlite::memory:`, or a raw path.
    ///
    /// # Errors
    /// Invalid path/URL or SQLite open failure.
    pub async fn connect(url: &str) -> DbResult<Self> {
        // normalization: the `file:` or `sqlite:` scheme is stripped; the `?query` part
        // (e.g. `?mode=rwc`) is ignored — sqlx creates the file itself.
        let without_file = url.strip_prefix("file:").unwrap_or(url);
        let without_scheme = without_file
            .strip_prefix("sqlite:")
            .filter(|&rest| rest != ":memory:")
            .unwrap_or(without_file);
        let normalized = without_scheme.split('?').next().unwrap_or(without_scheme);
        let in_memory = normalized == ":memory:" || normalized == "sqlite::memory:";
        let mut opts = if in_memory {
            // in-memory + shared cache — with the single connection below, the database
            // stays unified for the pool's lifetime.
            SqliteConnectOptions::new().in_memory(true).shared_cache(true)
        } else {
            let path = Path::new(normalized);
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DbError::Config(format!("cannot create dir {}: {e}", parent.display()))
                })?;
            }
            SqliteConnectOptions::new().filename(normalized)
        };
        opts = opts.create_if_missing(true).foreign_keys(true).busy_timeout(BUSY_TIMEOUT);
        // 2025-11 load-test fix: WAL — concurrent reads during writes (the previous
        // journal=delete blocked every write against parallel reads); synchronous=NORMAL —
        // stable durability at lower fsync cost. Meaningless on :memory: and not applied.
        if !in_memory {
            opts = opts
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal);
        }
        // :memory: with multiple connections = several separate databases; we use a single connection.
        let max = if in_memory { 1 } else { 4 };
        let pool = SqlitePoolOptions::new().max_connections(max).connect_with(opts).await?;
        let file_path = if in_memory { None } else { Some(normalized.to_owned()) };
        Ok(Self { pool, file_path })
    }

    /// In-memory store — for tests.
    ///
    /// # Errors
    /// SQLite open failure (does not occur in practice).
    pub async fn connect_in_memory() -> DbResult<Self> {
        Self::connect("sqlite::memory:").await
    }

    /// Read-only open — for external audit (genesis-proof).
    ///
    /// # Errors
    /// The file is missing or SQLite fails to open.
    pub async fn connect_read_only(path: &str) -> DbResult<Self> {
        let file = path.strip_prefix("file:").unwrap_or(path);
        let opts =
            SqliteConnectOptions::new().filename(file).read_only(true).busy_timeout(BUSY_TIMEOUT);
        let pool = SqlitePoolOptions::new().max_connections(1).connect_with(opts).await?;
        Ok(Self { pool, file_path: Some(file.to_owned()) })
    }

    /// Is this store file-backed? — the `VACUUM INTO` backup only makes sense
    /// for a file-backed store (`BACKUP_UNSUPPORTED` for in-memory).
    #[must_use]
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// Versioned schema migrations — `migrations/0001_init.sql` through
    /// `0005_builder_referral.sql` (automatic via `sqlx::migrate!()`).
    ///
    /// # Errors
    /// Migration execution failure.
    pub async fn migrate(&self) -> DbResult<()> {
        sqlx::migrate!().run(&self.pool).await?;
        Ok(())
    }

    /// Raw pool access — for external audit/parity (read-only SQL like
    /// `proof.rs`); normal paths must go through store methods.
    #[must_use]
    pub fn pool_ref(&self) -> &SqlitePool {
        &self.pool
    }

    /// One connection from the pool — for multi-step query sequences outside `apply`.
    ///
    /// # Errors
    /// Connection failure.
    pub(crate) async fn acquire_conn(&self) -> DbResult<sqlx::pool::PoolConnection<Sqlite>> {
        Ok(self.pool.acquire().await?)
    }

    /// Begin a transaction on the pool — for legacy migrations (all-or-nothing).
    ///
    /// # Errors
    /// Connection failure.
    pub(crate) async fn begin_tx(&self) -> DbResult<sqlx::Transaction<'static, Sqlite>> {
        Ok(self.pool.begin().await?)
    }

    /// Create an account if missing — new citizen registration (Phase 5 uses this).
    ///
    /// # Errors
    /// SQL error.
    pub async fn create_account_if_missing(
        &self,
        id: &str,
        kind: AccountKind,
        label: Option<&str>,
    ) -> DbResult<()> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
        match insert_account(&mut conn, id, kind, label).await {
            Ok(()) => {
                if let Err(e) = sqlx::query("COMMIT").execute(&mut *conn).await {
                    rollback_quietly(&mut conn).await;
                    return Err(e.into());
                }
                Ok(())
            }
            Err(e) => {
                rollback_quietly(&mut conn).await;
                Err(e)
            }
        }
    }

    /// System accounts must exist before any money movement (charter Article 4).
    ///
    /// # Errors
    /// SQL error.
    pub async fn ensure_system_accounts(&self) -> DbResult<()> {
        let defs = [
            (MINT_ACCOUNT_ID, AccountKind::Mint, "\u{645}\u{646}\u{628}\u{639} \u{639}\u{631}\u{636}\u{647} (\u{636}\u{631}\u{628})"),
            (TREASURY_ACCOUNT_ID, AccountKind::Treasury, "\u{62e}\u{632}\u{627}\u{646}\u{647} \u{645}\u{644}\u{6cc}"),
            (BURN_ACCOUNT_ID, AccountKind::Burn, "\u{645}\u{642}\u{635}\u{62f} \u{633}\u{648}\u{632}\u{627}\u{646}\u{62f}\u{646}"),
        ];
        let mut conn = self.pool.acquire().await?;
        sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
        let res: DbResult<()> = async {
            for (id, kind, label) in defs {
                insert_account(&mut conn, id, kind, Some(label)).await?;
            }
            Ok(())
        }
        .await;
        match res {
            Ok(()) => {
                if let Err(e) = sqlx::query("COMMIT").execute(&mut *conn).await {
                    rollback_quietly(&mut conn).await;
                    return Err(e.into());
                }
                Ok(())
            }
            Err(e) => {
                rollback_quietly(&mut conn).await;
                Err(e)
            }
        }
    }

    /// The adapter's core — load → pure domain → insert, all within one `BEGIN IMMEDIATE` transaction.
    ///
    /// Acquiring the write lock at transaction start (`BEGIN IMMEDIATE`), honoring
    /// `busy_timeout`, prevents `SQLITE_BUSY` at the `DEFERRED` lock-upgrade moment
    /// under parallel load.
    ///
    /// # Errors
    /// SQL error, or a domain error (which fully rolls back the transaction — an error never
    /// corrupts state).
    pub async fn apply<F>(&self, opts: ApplyOptions, run: F) -> DbResult<ApplyResult>
    where
        F: FnOnce(&LedgerState, &EscrowRegistry) -> DomainResult<CommandOutcome>,
    {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;

        let body_res: DbResult<ApplyResult> = async {
            // 1) replay precheck — the normal safe path for bot/API retries.
            if let Some(key) = opts.idempotency_key.as_deref()
                && let Some(existing) = tx_by_key(&mut conn, key).await?
            {
                let ledger = load_ledger_state(&mut conn).await?;
                let escrow = load_escrow_opt(&mut conn, opts.escrow_id.as_deref()).await?;
                tracing::info!(key, action = %opts.action, "apply replayed");
                Ok(ApplyResult { replayed: true, tx: Some(existing), escrow, ledger })
            } else {
                // 2) full state load under the IMMEDIATE write lock.
                let ledger = load_ledger_state(&mut conn).await?;
                let registry = load_escrow_registry(&mut conn, opts.escrow_id.as_deref()).await?;

                // 3) pure domain — no IO.
                let outcome = run(&ledger, &registry)?;

                // 4) insert all domain effects within the same transaction.
                persist_outcome(&mut conn, &opts, &outcome).await?;

                Ok(ApplyResult {
                    replayed: false,
                    tx: outcome.tx,
                    escrow: outcome.escrow,
                    ledger: outcome.ledger,
                })
            }
        }
        .await;

        match body_res {
            Ok(result) => {
                if let Err(e) = sqlx::query("COMMIT").execute(&mut *conn).await {
                    rollback_quietly(&mut conn).await;
                    return Err(e.into());
                }
                Ok(result)
            }
            Err(e) => {
                rollback_quietly(&mut conn).await;
                if is_idempotency_conflict(&e)
                    && let Some(key) = opts.idempotency_key.as_deref()
                    && let Some(existing) = tx_by_key(&mut conn, key).await?
                {
                    // race: a concurrent writer registered the same key first —
                    // the UNIQUE constraint saved us; convert to an honest replay read.
                    let ledger = load_ledger_state(&mut conn).await?;
                    let escrow = load_escrow_opt(&mut conn, opts.escrow_id.as_deref()).await?;
                    tracing::info!(key, "apply replayed after unique conflict");
                    return Ok(ApplyResult {
                        replayed: true,
                        tx: Some(existing),
                        escrow,
                        ledger,
                    });
                }
                Err(e)
            }
        }
    }

    // ── Read-only reads ─────────────────────────────────────────────

    /// One account's balance — `None` means the account does not exist.
    ///
    /// # Errors
    /// SQL error.
    pub async fn balance_of(&self, account_id: &str) -> DbResult<Option<i64>> {
        let Some(_) = sqlx::query(r#"SELECT "id" FROM "Account" WHERE "id" = ?"#)
            .bind(account_id)
            .fetch_optional(&self.pool)
            .await?
        else {
            return Ok(None);
        };
        let sum: Option<i64> =
            sqlx::query(r#"SELECT SUM("amount") AS "s" FROM "Posting" WHERE "accountId" = ?"#)
                .bind(account_id)
                .fetch_one(&self.pool)
                .await?
                .get("s");
        Ok(Some(sum.unwrap_or(0)))
    }

    /// All accounts with derived balances — the reconciliation report.
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_accounts_with_balances(&self) -> DbResult<Vec<AccountBalance>> {
        let mut out = Vec::new();
        let rows = sqlx::query(
            r#"SELECT "id", "kind", "label", "createdAt" FROM "Account" ORDER BY "id" ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        for r in rows {
            out.push(AccountBalance {
                id: r.get("id"),
                kind: r.get("kind"),
                label: r.get("label"),
                balance: 0,
                created_at: r.get("createdAt"),
            });
        }
        let sums = sqlx::query(
            r#"SELECT "accountId", SUM("amount") AS "s" FROM "Posting" GROUP BY "accountId""#,
        )
        .fetch_all(&self.pool)
        .await?;
        for r in sums {
            let id: String = r.get("accountId");
            if let Some(a) = out.iter_mut().find(|a| a.id == id) {
                a.balance = r.get("s");
            }
        }
        Ok(out)
    }

    /// Economy summary — the same `economySummary` as in TS (for the dashboard and reconciliation).
    ///
    /// 2025-11 load-test fix: each kind's sum now happens in SQL itself (one
    /// GROUP BY query) — previously all accounts + all Posting sums were shipped to Rust
    /// and merged with a linear O(N²) search; on 2,500 accounts ≈ 7 ms of CPU
    /// per call, which had made `/health` and `/economy` the system's slowest
    /// endpoints (86 RPS). The cost is now independent of N.
    ///
    /// # Errors
    /// SQL error, or an unknown kind (`Corrupt` — the same contract as before).
    pub async fn economy_summary(&self) -> DbResult<EconomySummary> {
        let rows = sqlx::query(
            r#"SELECT a."kind" AS "kind", COALESCE(SUM(p."amount"), 0) AS "s"
               FROM "Account" a LEFT JOIN "Posting" p ON p."accountId" = a."id"
               GROUP BY a."kind""#,
        )
        .fetch_all(&self.pool)
        .await?;
        let mut s = EconomySummary {
            circulating_supply: 0,
            treasury_balance: 0,
            escrow_locked: 0,
            users_sum: 0,
            mint_balance: 0,
            burn_balance: 0,
            minted_this_year: self.minted_this_year().await?,
            trial_sum: 0,
            balanced: false,
            open_escrows: self.open_escrow_count().await?,
        };
        for r in rows {
            let kind: String = r.get("kind");
            let v: i64 = r.get("s");
            s.trial_sum += v;
            match kind.as_str() {
                "user" => s.users_sum += v,
                "escrow" => s.escrow_locked += v,
                "treasury" => s.treasury_balance += v,
                "mint" => s.mint_balance += v,
                "burn" => s.burn_balance += v,
                _ => return Err(DbError::Corrupt(format!("unknown kind {kind:?}"))),
            }
        }
        s.circulating_supply = s.users_sum + s.escrow_locked + s.treasury_balance;
        s.balanced = s.trial_sum == 0;
        Ok(s)
    }

    /// Light DB ping for liveness — `SELECT 1`; never reads the ledger.
    ///
    /// 2025-11 load-test fix: for the health check, `/health` used to call `economy_summary`
    /// (a full ledger scan) and had become the system's most expensive endpoint —
    /// monitoring hit it every 30 s and it grew linearly heavier with user growth.
    /// health means "the process reaches the database"; the deep reconciliation read
    /// stayed in `/economy` (now light, with aggregate SQL).
    ///
    /// # Errors
    /// SQL error — the database is unavailable.
    pub async fn ping(&self) -> DbResult<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    /// Number of open escrows.
    async fn open_escrow_count(&self) -> DbResult<i64> {
        let n: i64 = sqlx::query(
            r#"SELECT COUNT(*) AS "c" FROM "Escrow"
               WHERE "status" IN ('CREATED', 'FUNDED', 'DELIVERED', 'DISPUTED')"#,
        )
        .fetch_one(&self.pool)
        .await?
        .get("c");
        Ok(n)
    }

    /// Current-year mint under UTC — the charter cap contract.
    ///
    /// # Errors
    /// SQL error.
    pub async fn minted_this_year(&self) -> DbResult<i64> {
        let sum: Option<i64> = sqlx::query(
            r#"SELECT SUM(p."amount") AS "s" FROM "Posting" p
               JOIN "Tx" t ON t."id" = p."txId"
               WHERE p."accountId" = ? AND t."at" >= ?"#,
        )
        .bind(MINT_ACCOUNT_ID)
        .bind(year_start_iso()?)
        .fetch_one(&self.pool)
        .await?
        .get("s");
        Ok(-sum.unwrap_or(0))
    }

    /// An account's latest rows — fuel for `wallet_history`.
    ///
    /// # Errors
    /// SQL error.
    pub async fn recent_postings_for(
        &self,
        account_id: &str,
        limit: i64,
    ) -> DbResult<Vec<RecentPosting>> {
        let mut out = Vec::new();
        for r in sqlx::query(
            r#"SELECT p."txId" AS "txId", p."amount" AS "amount", t."memo" AS "memo", t."at" AS "at"
               FROM "Posting" p JOIN "Tx" t ON t."id" = p."txId"
               WHERE p."accountId" = ? ORDER BY p."id" DESC LIMIT ?"#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?
        {
            out.push(RecentPosting {
                tx_id: r.get("txId"),
                amount: r.get("amount"),
                memo: r.get("memo"),
                at: r.get("at"),
            });
        }
        Ok(out)
    }

    /// One escrow by ID.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_escrow(&self, id: &str) -> DbResult<Option<Escrow>> {
        sqlx::query(r#"SELECT * FROM "Escrow" WHERE "id" = ?"#)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .as_ref()
            .map(row_to_escrow)
            .transpose()
    }

    /// List escrows — optional status filter, max 200 (like TS).
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_escrows(&self, status: Option<EscrowStatus>) -> DbResult<Vec<Escrow>> {
        let rows = match status {
            Some(st) => sqlx::query(
                r#"SELECT * FROM "Escrow" WHERE "status" = ? ORDER BY "createdAt" DESC LIMIT 200"#,
            )
            .bind(st.as_str())
            .fetch_all(&self.pool)
            .await?,
            None => {
                sqlx::query(r#"SELECT * FROM "Escrow" ORDER BY "createdAt" DESC LIMIT 200"#)
                    .fetch_all(&self.pool)
                    .await?
            }
        };
        rows.iter().map(row_to_escrow).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kian_domain::{
        create_escrow_with_affiliate, deliver_escrow, fund_escrow, post, release_escrow,
        CreateEscrowCommand, DeliverEscrowCommand, EscrowRefCommand, PostCommand,
        ReleaseEscrowCommand,
    };

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn escrow_referral_fields_persist_and_settle_four_legs_in_sqlite() {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();

        for acc in ["citizen:buyer", "citizen:seller", "citizen:ref"] {
            store.create_account_if_missing(acc, AccountKind::User, None).await.unwrap();
        }

        // charge the buyer with 1000 Đ
        store
            .apply(
                ApplyOptions {
                    actor: "admin".into(),
                    action: "mint".into(),
                    target: Some("citizen:buyer".into()),
                    escrow_id: None,
                    idempotency_key: Some("mint:buyer:1".into()),
                },
                |ledger, _reg| {
                    let posted = post(
                        ledger,
                        &PostCommand {
                            id: None,
                            idempotency_key: Some("mint:buyer:1".into()),
                            postings: vec![
                                Posting::new(MINT_ACCOUNT_ID, -1_000),
                                Posting::new("citizen:buyer", 1_000),
                            ],
                            memo: Some("seed".into()),
                            at: Some(now_iso()),
                        },
                    )?;
                    Ok(CommandOutcome {
                        ledger: posted.state,
                        tx: Some(posted.tx),
                        escrow: None,
                        events: vec![],
                    })
                },
            )
            .await
            .unwrap();

        // create and fund escrow with a referrer and a 15% commission
        let create_cmd = CreateEscrowCommand {
            id: "esc:ref:1".into(),
            buyer_id: "citizen:buyer".into(),
            seller_id: "citizen:seller".into(),
            amount: Din::new(400).unwrap(),
            product_ref: Some("prod:html-1".into()),
            expires_at: None,
            at: Some(now_iso()),
        };
        store
            .apply(
                ApplyOptions {
                    actor: "citizen:buyer".into(),
                    action: "escrow.create_fund".into(),
                    target: Some("esc:ref:1".into()),
                    escrow_id: Some("esc:ref:1".into()),
                    idempotency_key: Some("buy:ref:1".into()),
                },
                |ledger, reg| {
                    let created = create_escrow_with_affiliate(
                        ledger,
                        reg,
                        &create_cmd,
                        Some("citizen:ref".into()),
                        1_500,
                    )?;
                    let funded = fund_escrow(
                        &created.ledger,
                        &created.registry,
                        &EscrowRefCommand {
                            escrow_id: "esc:ref:1".into(),
                            idempotency_key: None,
                            at: Some(now_iso()),
                        },
                    )?;
                    Ok(CommandOutcome {
                        ledger: funded.ledger,
                        tx: funded.tx,
                        escrow: Some(funded.escrow),
                        events: vec![],
                    })
                },
            )
            .await
            .unwrap();

        // restoring from SQLite must return referrer and affiliate_bps exactly
        let loaded = store.get_escrow("esc:ref:1").await.unwrap().expect("escrow exists");
        assert_eq!(loaded.referrer.as_deref(), Some("citizen:ref"));
        assert_eq!(loaded.affiliate_bps, 1_500);

        // delivery and release → 4 postings (seller 300 + treasury 40 + referrer 60 = 400)
        store
            .apply(
                ApplyOptions {
                    actor: "citizen:buyer".into(),
                    action: "escrow.release".into(),
                    target: Some("esc:ref:1".into()),
                    escrow_id: Some("esc:ref:1".into()),
                    idempotency_key: Some("rel:ref:1".into()),
                },
                |ledger, reg| {
                    let del = deliver_escrow(
                        ledger,
                        reg,
                        &DeliverEscrowCommand {
                            escrow_id: "esc:ref:1".into(),
                            idempotency_key: None,
                            at: Some(now_iso()),
                            delivery_ref: Some("instant".into()),
                        },
                    )?;
                    let rel = release_escrow(
                        &del.ledger,
                        &del.registry,
                        &ReleaseEscrowCommand {
                            escrow_id: "esc:ref:1".into(),
                            idempotency_key: Some("rel:ref:1".into()),
                            at: Some(now_iso()),
                            approved_by_admin: false,
                        },
                    )?;
                    Ok(CommandOutcome {
                        ledger: rel.ledger,
                        tx: rel.tx,
                        escrow: Some(rel.escrow),
                        events: vec![],
                    })
                },
            )
            .await
            .unwrap();

        assert_eq!(store.balance_of("citizen:seller").await.unwrap(), Some(300));
        assert_eq!(store.balance_of(TREASURY_ACCOUNT_ID).await.unwrap(), Some(40));
        assert_eq!(store.balance_of("citizen:ref").await.unwrap(), Some(60));
        assert_eq!(store.balance_of("escrow:esc:ref:1").await.unwrap(), Some(0));
        assert!(store.economy_summary().await.unwrap().balanced);
    }
}
