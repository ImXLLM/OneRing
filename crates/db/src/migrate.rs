//! Port of `back/server/scripts/migrate-legacy.ts` — the day-one bridge from the
//! recovered Railway database to v3.
//!
//! One-to-one behavior with the TS version:
//!  1. the genesis plan from the raw legacy rows (floor of every citizen — ADR-002)
//!  2. if genesis already exists (`legacy:genesis:v1`) → a safe replay exit with
//!     a reconciliation report — a second run never mints twice.
//!  3. dry-run → plan only, no writes.
//!  4. inserts: 466 `user:tg:<id>` accounts + the genesis transaction + an audit row
//!     `legacy.adjustment` with the −63.0 gap detail + 30 `legacy-<id>` products.
//!
//! One deliberate, stricter difference from TS: genesis financial rows are not inserted
//! directly; they pass through the Phase 1 domain’s pure `post()` function (the
//! load → apply → persist pattern). So if any citizen has a negative floor or the
//! rows become unbalanced, the "domain" rejects the migration — fail-closed.

use crate::error::{DbError, DbResult};
use crate::genesis::{plan_genesis, GENESIS_KEY, GENESIS_TX_ID};
use crate::store::{
    insert_audit, insert_tx_with_postings, load_ledger_state, now_iso, tx_by_key, V3Store,
};
use kian_domain::{AccountKind, PostCommand, Posting, MINT_ACCOUNT_ID};
use sqlx::Row;

/// Migration report — print-ready lines + status flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrateReport {
    /// Report lines (Persian/English mix, like the TS console)
    pub lines: Vec<String>,
    /// Was genesis already present and replayed?
    pub replayed: bool,
    /// Was this a dry-run?
    pub dry_run: bool,
}

/// Reconciliation report — the same `reconcile` as in TS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileReport {
    /// Σ user wallets — acceptance figure: 187,672 Đ
    pub users_sum: i64,
    /// user count
    pub user_count: usize,
    /// total account count (user + system)
    pub account_count: usize,
    /// Does the trial balance hold?
    pub balanced: bool,
    /// Σ of all rows — must be 0
    pub trial_sum: i64,
    /// available product count
    pub product_count: i64,
    /// legacy products that had to be imported
    pub expected_products: i64,
    /// legacy adjustment row count
    pub adjustment_rows: i64,
}

impl ReconcileReport {
    /// Report print lines — the same format as TS.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        vec![
            format!(
                "  Σ user wallets        = {} Đ  (acceptance: {})",
                self.users_sum,
                crate::genesis::EXPECTED_SUPPLY
            ),
            format!("  trialBalance.balanced = {} (Σ postings {})", self.balanced, self.trial_sum),
            format!(
                "  accounts              = {} ({} users + system)",
                self.account_count, self.user_count
            ),
            format!("  products              = {}/{}", self.product_count, self.expected_products),
            format!("  adjustment rows       = {}", self.adjustment_rows),
        ]
    }
}

/// Reconcile the target store — for the replay exit and migration end (like `reconcile` in TS).
///
/// # Errors
/// SQL error.
pub async fn reconcile(store: &V3Store, expected_products: i64) -> DbResult<ReconcileReport> {
    let accounts = store.list_accounts_with_balances().await?;
    let users: Vec<_> = accounts.iter().filter(|a| a.id.starts_with("user:")).collect();
    let users_sum: i64 = users.iter().map(|a| a.balance).sum();
    let pool = store.pool_ref();
    let products: i64 =
        sqlx::query(r#"SELECT COUNT(*) AS "c" FROM "Product""#).fetch_one(pool).await?.get("c");
    let adjustments: i64 = sqlx::query(
        r#"SELECT COUNT(*) AS "c" FROM "AuditLog" WHERE "action" = 'legacy.adjustment'"#,
    )
    .fetch_one(pool)
    .await?
    .get("c");
    let summary = store.economy_summary().await?;
    Ok(ReconcileReport {
        users_sum,
        user_count: users.len(),
        account_count: accounts.len(),
        balanced: summary.balanced,
        trial_sum: summary.trial_sum,
        product_count: products,
        expected_products,
        adjustment_rows: adjustments,
    })
}

/// Run the legacy migration against the target store.
///
/// # Errors
/// SQL error, an invariant violation in the domain (fail-closed), or unreadable legacy data.
#[allow(clippy::cast_possible_truncation)] // floor() before cast — the ADR-002 explicit rounding
pub async fn migrate_legacy(
    store: &V3Store,
    users: &[crate::genesis::LegacyUserRow],
    products: &[crate::genesis::LegacyProductRow],
    tx_count: i64,
    source: &str,
    dry_run: bool,
) -> DbResult<MigrateReport> {
    let plan = plan_genesis(users);
    let mut lines = vec![
        format!(
            "[migrate] legacy users: {} (banned {}) · Σ credits {:.1} Đ · tx rows {tx_count}",
            users.len(),
            plan.banned,
            plan.sum_credits
        ),
        format!(
            "[migrate] per-citizen floor: Σ {} Đ across {} holder(s)",
            plan.sum_din, plan.holders
        ),
        format!("[migrate] legacy products: {}", products.len()),
    ];

    store.ensure_system_accounts().await?;

    // second run → safe replay exit with the reconciliation report (no double mint).
    let mut conn = store.acquire_conn().await?;
    if tx_by_key(&mut conn, GENESIS_KEY).await?.is_some() {
        lines.push("[migrate] genesis already present — replay-safe exit.".into());
        let rec = reconcile(store, i64::try_from(products.len()).unwrap_or(i64::MAX)).await?;
        lines.extend(rec.lines());
        return Ok(MigrateReport { lines, replayed: true, dry_run });
    }

    if dry_run {
        lines.push("[migrate] dry-run — no writes. Plan:".into());
        lines.push(format!("  accounts: {} user account(s) + 4 system accounts", users.len()));
        lines.push(format!(
            "  genesis tx: mint:genesis −{} → {} user posting(s)",
            plan.sum_din, plan.holders
        ));
        lines.push(format!("  products: {} row(s)", products.len()));
        return Ok(MigrateReport { lines, replayed: false, dry_run });
    }

    // main transaction — all or nothing.
    let mut tx = store.begin_tx().await?;
    for c in &plan.citizens {
        crate::store::insert_account(&mut tx, &c.account_id, AccountKind::User, Some(&c.label))
            .await?;
    }

    // genesis through the pure domain path — fail-closed (stricter than TS).
    let ledger = load_ledger_state(&mut tx).await?;
    let mut postings = vec![Posting::new(MINT_ACCOUNT_ID, -plan.sum_din)];
    postings.extend(
        plan.citizens
            .iter()
            .filter(|c| c.din > 0)
            .map(|c| Posting::new(c.account_id.clone(), c.din)),
    );
    let cmd = PostCommand {
        id: Some(GENESIS_TX_ID.into()),
        idempotency_key: Some(GENESIS_KEY.into()),
        postings,
        memo: Some(plan.memo.clone()),
        at: Some(now_iso()),
    };
    let posted = kian_domain::post(&ledger, &cmd).map_err(DbError::from)?;
    insert_tx_with_postings(&mut tx, &posted.tx, Some(GENESIS_KEY)).await?;

    // audit row — the −63.0 gap detail stays provable forever.
    let meta = serde_json::json!({
        "legacyUsers": users.len(),
        "legacyTransactions": tx_count,
        "storedBalanceSum": plan.sum_credits,
        "perCitizenGenesis": plan.sum_din,
        "impliedByLegacyLedger": 187_735.8,
        "gap": -63.0,
        "gapItemization": "9×50 purchases without tx rows (−450) + manual owner credit (+387)",
        "source": source,
    })
    .to_string();
    insert_audit(&mut tx, "legacy-migration", "legacy.adjustment", Some("ledger"), Some(&meta))
        .await?;

    // 30 legacy products — delivery via `link` only (the Railway disk files are gone for good).
    for p in products {
        let title: String = p.title.chars().take(120).collect();
        let description: String =
            p.description.clone().unwrap_or_default().chars().take(800).collect();
        let category = p.category.as_ref().map(|c| c.chars().take(60).collect::<String>());
        let delivery = p.link.as_ref().filter(|l| !l.is_empty()).cloned();
        sqlx::query(
            r#"INSERT INTO "Product" ("id", "sellerId", "title", "description", "priceDin",
                 "category", "deliveryPayload", "active", "createdAt")
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT("id") DO NOTHING"#,
        )
        .bind(format!("legacy-{}", p.id))
        .bind(format!("user:tg:{}", p.creator_id))
        .bind(title)
        .bind(description)
        .bind(p.price_credits.floor() as i64)
        .bind(category)
        .bind(delivery)
        .bind(p.is_active == 1)
        .bind(now_iso())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    lines.push("[migrate] done.".into());
    let rec = reconcile(store, i64::try_from(products.len()).unwrap_or(i64::MAX)).await?;
    lines.extend(rec.lines());
    Ok(MigrateReport { lines, replayed: false, dry_run })
}
