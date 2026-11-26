//! Phase 4 read/write queries — port of the `back/server/src/*.ts` queries that
//! are not domain commands: the HITL queue, dashboard products, the audit trail, AI usage, and
//! the `VACUUM INTO` backup.
//!
//! The Phase 2 ground rules hold:
//! * Balances are never stored — always derived from `Posting` (ADR-003).
//! * Every money-moving command, inside the same `apply`/HITL allocation transaction,
//!   writes both audit and outbox.
//! * The HITL "approve ⇒ settle" decision is atomic: one transaction, both or neither.
//!
//! HITL queue errors deliberately surface as [`kian_domain::DomainError`] with domain codes
//! (`HITL_NOT_FOUND` and so on) so the 404/422 mapping in `contracts::error`
//! keeps working without duplication.

// Legacy style: nested helper functions are sometimes defined after statements;
// restructuring the whole file is out of scope for the M2 migration.
#![allow(clippy::items_after_statements)]
#![allow(clippy::needless_pass_by_value)]

use crate::sqlsafe;
use kian_domain::{
    DeliverEscrowCommand, DomainError, DomainErrorCode, DomainEvent, DomainResult, Escrow,
    LedgerState, PostedTransaction, ReleaseEscrowCommand,
};

use crate::error::{DbError, DbResult};
use crate::store::insert_audit;
use crate::store::{
    load_escrow_registry, load_ledger_state, now_iso, persist_outcome, ApplyOptions,
    CommandOutcome, V3Store,
};
use crate::wire::event_payload;

// ── HITL queue ─────────────────────────────────────────────────────────────────

/// Raw HITL request row from the `HitlRequest` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitlRow {
    /// Identifier
    pub id: String,
    /// Kind (`ESCROW_RELEASE`)
    pub kind: String,
    /// Related escrow
    pub escrow_id: String,
    /// Amount
    pub amount: i64,
    /// Requester
    pub requester_id: String,
    /// Optional note
    pub note: Option<String>,
    /// Status (`PENDING`/`APPROVED`/`REJECTED`/`EXPIRED`)
    pub status: String,
    /// ISO creation time
    pub created_at: String,
    /// ISO expiry time
    pub expires_at: String,
    /// Decided by
    pub decided_by: Option<String>,
    /// Decision time
    pub decided_at: Option<String>,
    /// Decision note
    pub decision_note: Option<String>,
}

/// HITL decision output — updated request + settlement result (APPROVE only).
/// AI usage row for the `AiUsage` table (Phase 6).
#[derive(Debug, Clone)]
pub struct AiUsageRow {
    /// User (`user:tg:<id>`)
    pub user_id: String,
    /// Agent (`orchestrator`)
    pub agent: String,
    /// Provider
    pub provider: String,
    /// Model
    pub model: String,
    /// Cycle kind (`chat`)
    pub kind: String,
    /// Prompt tokens
    pub prompt_tokens: i64,
    /// Completion tokens
    pub completion_tokens: i64,
    /// Total
    pub total_tokens: i64,
    /// Audit outcome (`ai.ok|refused|failed|pending`)
    pub outcome: String,
}

/// HITL decision output — updated request + settlement result (APPROVE only).
#[derive(Debug, Clone)]
pub struct HitlDecision {
    /// Request after the decision
    pub request: HitlRow,
    /// Escrow after settlement (approve only)
    pub escrow: Option<Escrow>,
    /// Settlement transaction (approve only; the original tx on replay)
    pub tx: Option<PostedTransaction>,
    /// Was the settlement a replay?
    pub replayed: bool,
}

/// Settlement command with a deterministic key — `hitl:<id>:approve` (documented in Phase 3).
fn settlement_key(hitl_id: &str) -> String {
    format!("hitl:{hitl_id}:approve")
}

/// Builds a HITL row from a raw SQL row.
fn hitl_row_from(row: sqlx::sqlite::SqliteRow) -> HitlRow {
    use sqlx::Row;
    HitlRow {
        id: row.get("id"),
        kind: row.get("kind"),
        escrow_id: row.get("escrowId"),
        amount: row.get::<i64, _>("amount"),
        requester_id: row.get("requesterId"),
        note: row.get("note"),
        status: row.get("status"),
        created_at: row.get("createdAt"),
        expires_at: row.get("expiresAt"),
        decided_by: row.get("decidedBy"),
        decided_at: row.get("decidedAt"),
        decision_note: row.get("decisionNote"),
    }
}

const HITL_SELECT: &str = r#"SELECT "id","kind","escrowId","amount","requesterId","note","status","createdAt","expiresAt","decidedBy","decidedAt","decisionNote" FROM "HitlRequest""#;

impl V3Store {
    /// Read one HITL request — `None` means it does not exist.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_hitl(&self, id: &str) -> DbResult<Option<HitlRow>> {
        let sql = format!("{HITL_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlsafe::audited(sql.clone())).bind(id).fetch_optional(self.pool_ref()).await?;
        Ok(row.map(hitl_row_from))
    }

    /// List HITL requests with an optional status filter — newest first.
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_hitl(&self, status: Option<&str>) -> DbResult<Vec<HitlRow>> {
        let (sql, has_filter) = match status {
            Some(_) => (
                format!(
                    "{HITL_SELECT} WHERE \"status\" = ? ORDER BY \"createdAt\" DESC, \"id\" DESC"
                ),
                true,
            ),
            None => (format!("{HITL_SELECT} ORDER BY \"createdAt\" DESC, \"id\" DESC"), false),
        };
        let q = sqlx::query(sqlsafe::audited(sql.clone()));
        let rows = if has_filter {
            q.bind(status.unwrap_or_default()).fetch_all(self.pool_ref()).await?
        } else {
            q.fetch_all(self.pool_ref()).await?
        };
        Ok(rows.into_iter().map(hitl_row_from).collect())
    }

    /// Enqueue a HITL request — a duplicate `id` is not an error; the existing request is returned
    /// (replay-safe like `hitl.ts`). The amount is copied from the escrow; a concurrent insert race
    /// is resolved with `ON CONFLICT DO NOTHING` + re-read.
    ///
    /// # Errors
    /// Missing escrow → `ESCROW_NOT_FOUND` (domain); SQL error.
    pub async fn enqueue_hitl(&self, row: HitlRow, escrow_id: &str) -> DbResult<HitlRow> {
        // Escrow existence check — same TS contract (amount is copied from the escrow)
        let escrow = self.get_escrow(escrow_id).await?;
        if escrow.is_none() {
            return Err(DomainError::new(
                DomainErrorCode::EscrowNotFound,
                format!("Escrow {escrow_id} does not exist"),
            )
            .into());
        }
        let mut tx = self.pool_ref().begin().await?;
        sqlx::query(
            r#"INSERT INTO "HitlRequest"
               ("id","kind","escrowId","amount","requesterId","note","status","createdAt","expiresAt","decidedBy","decidedAt","decisionNote")
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?)
               ON CONFLICT("id") DO NOTHING"#,
        )
        .bind(&row.id)
        .bind(&row.kind)
        .bind(&row.escrow_id)
        .bind(row.amount)
        .bind(&row.requester_id)
        .bind(&row.note)
        .bind(&row.status)
        .bind(&row.created_at)
        .bind(&row.expires_at)
        .bind(&row.decided_by)
        .bind(&row.decided_at)
        .bind(&row.decision_note)
        .execute(&mut *tx)
        .await?;
        let sql = format!("{HITL_SELECT} WHERE \"id\" = ?");
        let existing =
            sqlx::query(sqlsafe::audited(sql.clone())).bind(&row.id).fetch_one(&mut *tx).await.map(hitl_row_from)?;
        if existing.created_at == row.created_at && existing.status == "PENDING" {
            // This insert was fresh — audit once
            insert_audit(&mut tx, "server", "hitl.create", Some(&row.id), Some(&row.kind)).await?;
        }
        tx.commit().await?;
        Ok(existing)
    }

    /// Reject a HITL request — no money moves.
    ///
    /// # Errors
    /// `HITL_NOT_FOUND` / `HITL_ALREADY_DECIDED` / `HITL_EXPIRED` (domain code →
    /// 404/422 in the contract layer) or a SQL error.
    pub async fn reject_hitl(
        &self,
        id: &str,
        decided_by: &str,
        note: Option<&str>,
    ) -> DbResult<HitlRow> {
        let mut tx = self.pool_ref().begin().await?;
        let row = self.hitl_pending_guard_tx(&mut tx, id).await?;
        let now = now_iso();
        sqlx::query(
            r#"UPDATE "HitlRequest" SET "status"='REJECTED', "decidedBy"=?, "decidedAt"=?, "decisionNote"=?
               WHERE "id"=?"#,
        )
        .bind(decided_by)
        .bind(&now)
        .bind(note)
        .bind(id)
        .execute(&mut *tx)
        .await?;
        insert_audit(&mut tx, decided_by, "hitl.reject", Some(id), note).await?;
        tx.commit().await?;
        let mut updated = row;
        updated.status = "REJECTED".into();
        updated.decided_by = Some(decided_by.into());
        updated.decided_at = Some(now);
        updated.decision_note = note.map(str::to_owned);
        Ok(updated)
    }

    /// Approve a HITL request and settle "in the same transaction" — the heart of Phase 4.
    ///
    /// Step order on one `sqlx::Transaction`:
    /// 1) Status guard (must be PENDING) via `UPDATE … WHERE status='PENDING'` —
    ///    a two-admin race lets exactly one through; the second gets `HITL_ALREADY_DECIDED`.
    /// 2) Settlement through the pure domain path (`release_escrow`) with the deterministic
    ///    `hitl:<id>:approve` key — the 90/10 money moves exactly once.
    /// 3) `persist_outcome` — Tx + Posting + Escrow + outbox + audit all in
    ///    the same transaction.
    ///
    /// # Errors
    /// Domain codes (`HITL_*`, `REQUIRES_HUMAN_APPROVAL`, and so on) or a SQL error —
    /// any failure rolls back the whole transaction (no money is left behind).
    pub async fn approve_hitl_and_settle(
        &self,
        id: &str,
        decided_by: &str,
        note: Option<&str>,
    ) -> DbResult<HitlDecision> {
        let mut tx = self.pool_ref().begin().await?;

        // 1) Atomic guard: only PENDING → APPROVED passes.
        let row = self.hitl_pending_guard_tx(&mut tx, id).await?;
        let now = now_iso();
        let updated = sqlx::query(
            r#"UPDATE "HitlRequest" SET "status"='APPROVED', "decidedBy"=?, "decidedAt"=?, "decisionNote"=?
               WHERE "id"=? AND "status"='PENDING'"#,
        )
        .bind(decided_by)
        .bind(&now)
        .bind(note)
        .bind(id)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() != 1 {
            // Race: another admin decided first → typed error, zero refund.
            tx.rollback().await?;
            return Err(DomainError::new(
                DomainErrorCode::HitlAlreadyDecided,
                format!("HITL request {id} has already been decided"),
            )
            .into());
        }
        insert_audit(&mut tx, decided_by, "hitl.approve", Some(id), note).await?;

        // 2+3) Settlement through the pure domain path — same transaction. The PENDING guard makes
        // the replay path practically impossible; the deterministic key is only a concurrency safety belt.
        let ledger = load_ledger_state(&mut tx).await?;
        let registry = load_escrow_registry(&mut tx, Some(&row.escrow_id)).await?;
        let settle_opts = ApplyOptions {
            actor: decided_by.into(),
            action: "hitl.approve".into(),
            target: Some(row.escrow_id.clone()),
            escrow_id: Some(row.escrow_id.clone()),
            idempotency_key: Some(settlement_key(id)),
        };
        let outcome = match (|| -> DbResult<CommandOutcome> {
            let released = kian_domain::release_escrow(
                &ledger,
                &registry,
                &ReleaseEscrowCommand {
                    escrow_id: row.escrow_id.clone(),
                    idempotency_key: Some(settlement_key(id)),
                    at: Some(now.clone()),
                    approved_by_admin: true,
                },
            )?;
            Ok(CommandOutcome {
                ledger: released.ledger,
                tx: released.tx,
                escrow: Some(released.escrow),
                events: released.events.into_iter().map(DomainEvent::Escrow).collect(),
            })
        })() {
            Ok(o) => o,
            Err(e) => {
                tx.rollback().await?;
                return Err(e);
            }
        };
        persist_outcome(&mut tx, &settle_opts, &outcome).await?;
        tx.commit().await?;

        let mut request = row;
        request.status = "APPROVED".into();
        request.decided_by = Some(decided_by.into());
        request.decided_at = Some(now);
        request.decision_note = note.map(str::to_owned);
        Ok(HitlDecision { request, escrow: outcome.escrow, tx: outcome.tx, replayed: false })
    }

    /// "PENDING" guard inside a transaction — returns typed domain errors.
    async fn hitl_pending_guard_tx(
        &self,
        exec: &mut sqlx::SqliteConnection,
        id: &str,
    ) -> DbResult<HitlRow> {
        let sql = format!("{HITL_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlsafe::audited(sql.clone()))
            .bind(id)
            .fetch_optional(&mut *exec)
            .await?
            .map(hitl_row_from)
            .ok_or_else(|| {
            DomainError::new(
                DomainErrorCode::HitlNotFound,
                format!("HITL request {id} does not exist"),
            )
        })?;
        if row.status != "PENDING" {
            return Err(DomainError::new(
                DomainErrorCode::HitlAlreadyDecided,
                format!("HITL request {id} has already been decided ({})", row.status),
            )
            .into());
        }
        if row.expires_at.as_str() < now_iso().as_str() {
            // Expired but not yet swept — same expiry semantics.
            return Err(DomainError::new(
                DomainErrorCode::HitlExpired,
                format!("HITL request {id} expired at {}", row.expires_at),
            )
            .into());
        }
        Ok(row)
    }

    /// Expiry sweep — all overdue PENDING requests → `EXPIRED`. Returns: the count.
    ///
    /// # Errors
    /// SQL error.
    pub async fn sweep_expired_hitl(&self) -> DbResult<i64> {
        let now = now_iso();
        let mut tx = self.pool_ref().begin().await?;
        let res = sqlx::query(
            r#"UPDATE "HitlRequest" SET "status"='EXPIRED'
               WHERE "status"='PENDING' AND "expiresAt" < ?"#,
        )
        .bind(&now)
        .execute(&mut *tx)
        .await?;
        let n = i64::try_from(res.rows_affected()).unwrap_or(i64::MAX);
        if n > 0 {
            insert_audit(&mut tx, "server", "hitl.sweep", None, Some(&format!("expired:{n}")))
                .await?;
        }
        tx.commit().await?;
        Ok(n)
    }
}

// ── Dashboard products ─────────────────────────────────────────────────────────

/// Product row from the `Product` table — columns one-to-one with `schema.prisma`:
/// dashboard-only fields (Stars/stock/featured/salesCount) have no column and are
/// derived in the dashboard view (Stars is not real currency — zero; stock is unlimited;
/// sales from settled escrows with a `productRef`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductRow {
    /// Identifier (`prod:<uuid>`)
    pub id: String,
    /// Seller
    pub seller_id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Category
    pub category: Option<String>,
    /// Price in din — the real `priceDin` column (real credits)
    pub price_din: i64,
    /// Safe pre-purchase HTML preview (public)
    pub preview_html: String,
    /// Affiliate commission rate in basis points (`0..=5000`; default 1000 = 10%)
    pub affiliate_bps: u16,
    /// Single-page artifact kind (default `"html"`)
    pub artifact_kind: String,
    /// Delivery payload (read only by the dedicated buyer-delivery query; `None` in public listings)
    pub delivery_payload: Option<String>,
    /// Active?
    pub active: bool,
    /// ISO creation time
    pub created_at: String,
    /// Stamped Mairon REA score (`0` = not yet stamped)
    pub mairon_score: u8,
    /// Visible Mairon badge (empty = not yet stamped)
    pub mairon_badge: String,
    /// ARDA origin locale — default `'en'` (migration 0008)
    pub origin_locale: String,
    /// Supported locales JSON — default `["en","fa","ar"]` (migration 0008)
    pub supported_locales_json: String,
    /// Mairon-forged `SKILL.md` document (`None` = not yet forged)
    pub skill_md: Option<String>,
    /// Mairon-forged `skill.json` manifest (`None` = not yet forged)
    pub skill_json: Option<String>,
    /// Timestamp of the last ARDA 24-hour cron tick
    pub arda_last_localized_at: Option<String>,
}

/// A citizen's recorded 5-star vote on a product (`"ProductReview"`) — Mairon stamp
/// plus community vote (Wave 3 — Trinity marketplace gate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductReviewRow {
    /// Unique vote identifier (`rev:<uuid>`)
    pub id: String,
    /// Product identifier
    pub product_id: String,
    /// Reviewing citizen account
    pub reviewer_id: String,
    /// Stars (`1..=5`)
    pub stars: u8,
    /// Validated comment
    pub comment: String,
    /// Verified purchase — linked to a settled escrow (anti-Sybil)
    pub verified_purchase: bool,
    /// Snapshot of the Mairon REA score at vote time
    pub mairon_score_at_review: u8,
    /// ISO recorded time
    pub created_at: String,
}

const PRODUCT_REVIEW_SELECT: &str = r#"SELECT "id","productId","reviewerId","stars","comment","verifiedPurchase","maironScoreAtReview","createdAt" FROM "ProductReview""#;

fn product_review_row_from(row: sqlx::sqlite::SqliteRow) -> ProductReviewRow {
    use sqlx::Row;
    ProductReviewRow {
        id: row.get("id"),
        product_id: row.get("productId"),
        reviewer_id: row.get("reviewerId"),
        stars: u8::try_from(row.get::<i64, _>("stars")).unwrap_or(1),
        comment: row.get("comment"),
        verified_purchase: row.get::<i64, _>("verifiedPurchase") != 0,
        mairon_score_at_review: u8::try_from(row.get::<i64, _>("maironScoreAtReview"))
            .unwrap_or(0),
        created_at: row.get("createdAt"),
    }
}

/// Product listing filters — the same TS queries (`q`/`limit`/`offset`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProductFilter {
    /// Substring search over title/description
    pub q: Option<String>,
    /// Page size
    pub limit: i64,
    /// From here
    pub offset: i64,
    /// Active only?
    pub active_only: bool,
}

/// Public product query — the `deliveryPayload` column is deliberately not read so it can never leak.
const PRODUCT_PUBLIC_SELECT: &str = r#"SELECT "id","sellerId","title","description","category","priceDin","previewHtml","affiliateBps","artifactKind","active","createdAt","maironScore","maironBadge","originLocale","supportedLocalesJson","skillMd","skillJson","ardaLastLocalizedAt" FROM "Product""#;

/// Safe ARDA default columns — for legacy INSERTs and the legacy path.
const ARDA_DEFAULT_LOCALES_JSON: &str = "[\"en\",\"fa\",\"ar\"]";

fn product_row_public_from(row: sqlx::sqlite::SqliteRow) -> ProductRow {
    use sqlx::Row;
    let bps_raw: i64 = row.get("affiliateBps");
    ProductRow {
        id: row.get("id"),
        seller_id: row.get("sellerId"),
        title: row.get("title"),
        description: row.get("description"),
        category: row.get("category"),
        price_din: row.get("priceDin"),
        preview_html: row.get("previewHtml"),
        affiliate_bps: u16::try_from(bps_raw).unwrap_or(kian_domain::DEFAULT_AFFILIATE_BPS),
        artifact_kind: row.get("artifactKind"),
        delivery_payload: None,
        active: row.get::<i64, _>("active") != 0,
        created_at: row.get("createdAt"),
        mairon_score: u8::try_from(row.get::<i64, _>("maironScore")).unwrap_or(0),
        mairon_badge: row.get("maironBadge"),
        origin_locale: row.get("originLocale"),
        supported_locales_json: row.get("supportedLocalesJson"),
        skill_md: row.get("skillMd"),
        skill_json: row.get("skillJson"),
        arda_last_localized_at: row.get("ardaLastLocalizedAt"),
    }
}

fn product_row_with_delivery_from(row: sqlx::sqlite::SqliteRow) -> ProductRow {
    use sqlx::Row;
    let bps_raw: i64 = row.get("affiliateBps");
    ProductRow {
        id: row.get("id"),
        seller_id: row.get("sellerId"),
        title: row.get("title"),
        description: row.get("description"),
        category: row.get("category"),
        price_din: row.get("priceDin"),
        preview_html: row.get("previewHtml"),
        affiliate_bps: u16::try_from(bps_raw).unwrap_or(kian_domain::DEFAULT_AFFILIATE_BPS),
        artifact_kind: row.get("artifactKind"),
        delivery_payload: row.get("deliveryPayload"),
        active: row.get::<i64, _>("active") != 0,
        created_at: row.get("createdAt"),
        mairon_score: u8::try_from(row.get::<i64, _>("maironScore")).unwrap_or(0),
        mairon_badge: row.get("maironBadge"),
        origin_locale: row.get("originLocale"),
        supported_locales_json: row.get("supportedLocalesJson"),
        skill_md: row.get("skillMd"),
        skill_json: row.get("skillJson"),
        arda_last_localized_at: row.get("ardaLastLocalizedAt"),
    }
}

impl V3Store {
    /// List products with a filter — newest first (without `deliveryPayload`).
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_products(&self, f: &ProductFilter) -> DbResult<Vec<ProductRow>> {
        let mut where_clauses: Vec<&str> = Vec::new();
        if f.active_only {
            where_clauses.push(r#""active" = 1"#);
        }
        if f.q.is_some() {
            where_clauses.push(r#"("title" LIKE ? ESCAPE '\' OR "description" LIKE ? ESCAPE '\')"#);
        }
        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_clauses.join(" AND "))
        };
        let sql = format!(
            "{PRODUCT_PUBLIC_SELECT}{where_sql} ORDER BY \"createdAt\" DESC, \"id\" DESC LIMIT ? OFFSET ?"
        );
        // like is kept outside the if — bind holds its borrow until fetch_all
        let like = f.q.as_ref().map(|qs| {
            let escaped = qs.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            format!("%{escaped}%")
        });
        let mut q = sqlx::query(sqlsafe::audited(sql.clone()));
        if let Some(pattern) = &like {
            q = q.bind(pattern).bind(pattern);
        }
        let rows = q.bind(f.limit).bind(f.offset).fetch_all(self.pool_ref()).await?;
        Ok(rows.into_iter().map(product_row_public_from).collect())
    }

    /// Count products with the same filter — for `total`.
    ///
    /// # Errors
    /// SQL error.
    pub async fn count_products(&self, f: &ProductFilter) -> DbResult<i64> {
        let mut where_clauses: Vec<&str> = Vec::new();
        if f.active_only {
            where_clauses.push(r#""active" = 1"#);
        }
        if f.q.is_some() {
            where_clauses.push(r#"("title" LIKE ? ESCAPE '\' OR "description" LIKE ? ESCAPE '\')"#);
        }
        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_clauses.join(" AND "))
        };
        let sql = format!(r#"SELECT COUNT(*) AS c FROM "Product"{where_sql}"#);
        let like = f.q.as_ref().map(|qs| {
            let escaped = qs.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            format!("%{escaped}%")
        });
        let mut q = sqlx::query(sqlsafe::audited(sql.clone()));
        if let Some(pattern) = &like {
            q = q.bind(pattern).bind(pattern);
        }
        let row = q.fetch_one(self.pool_ref()).await?;
        use sqlx::Row;
        Ok(row.get("c"))
    }

    /// Read a product's public view — without `deliveryPayload` (safe for the public path).
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_product(&self, id: &str) -> DbResult<Option<ProductRow>> {
        let sql = format!("{PRODUCT_PUBLIC_SELECT} WHERE \"id\" = ?");
        let row = sqlx::query(sqlsafe::audited(sql.clone())).bind(id).fetch_optional(self.pool_ref()).await?;
        Ok(row.map(product_row_public_from))
    }

    /// Full product read including `deliveryPayload` — call **only** after verifying the
    /// buyer/seller identity and the escrow's paid status.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_product_with_delivery(&self, id: &str) -> DbResult<Option<ProductRow>> {
        let row = sqlx::query(
            r#"SELECT "id","sellerId","title","description","category","priceDin",
               "previewHtml","affiliateBps","artifactKind","deliveryPayload","active","createdAt",
               "maironScore","maironBadge","originLocale","supportedLocalesJson",
               "skillMd","skillJson","ardaLastLocalizedAt"
               FROM "Product" WHERE "id" = ?"#,
        )
        .bind(id)
        .fetch_optional(self.pool_ref())
        .await?;
        Ok(row.map(product_row_with_delivery_from))
    }

    /// Insert a product — the id is pre-generated (`prod:<uuid>`).
    ///
    /// # Errors
    /// SQL error.
    pub async fn insert_product(&self, p: &ProductRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "Product"
               ("id","sellerId","title","description","category","priceDin",
                "previewHtml","affiliateBps","artifactKind","deliveryPayload","active","createdAt",
                "maironScore","maironBadge","originLocale","supportedLocalesJson",
                "skillMd","skillJson","ardaLastLocalizedAt")
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,
                       COALESCE(?, 0), COALESCE(?, ''),
                       COALESCE(?, 'en'), COALESCE(?, ?), ?, ?, ?)"#,
        )
        .bind(&p.id)
        .bind(&p.seller_id)
        .bind(&p.title)
        .bind(&p.description)
        .bind(&p.category)
        .bind(p.price_din)
        .bind(&p.preview_html)
        .bind(i64::from(p.affiliate_bps))
        .bind(&p.artifact_kind)
        .bind(&p.delivery_payload)
        .bind(i64::from(p.active))
        .bind(&p.created_at)
        .bind(i64::from(p.mairon_score))
        .bind(&p.mairon_badge)
        .bind(&p.origin_locale)
        .bind(&p.supported_locales_json)
        .bind(ARDA_DEFAULT_LOCALES_JSON)
        .bind(&p.skill_md)
        .bind(&p.skill_json)
        .bind(&p.arda_last_localized_at)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext("server", "product.create", Some(&p.id), None).await?;
        Ok(())
    }

    /// Mairon two-key stamp on the listing — score and badge land here only after the arbiter's
    /// verdict passes **and** the maker explicitly confirms (`maironScore`/`maironBadge`).
    ///
    /// # Errors
    /// Missing product → `PRODUCT_NOT_FOUND` (domain); SQL error.
    pub async fn stamp_product_mairon(
        &self,
        id: &str,
        score: u8,
        badge: &str,
    ) -> DbResult<()> {
        let updated = sqlx::query(
            r#"UPDATE "Product" SET "maironScore"=?, "maironBadge"=? WHERE "id"=?"#,
        )
        .bind(i64::from(score))
        .bind(badge)
        .bind(id)
        .execute(self.pool_ref())
        .await?
        .rows_affected();
        if updated == 0 {
            return Err(DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {id} does not exist"),
            )
            .into());
        }
        self.audit_ext("server", "product.mairon_stamp", Some(id), None).await?;
        Ok(())
    }

    /// ARDA localization stamp — writes the origin locale, the supported-locale band, the
    /// Mairon-forged skill bundle, and the 24-hour cron timestamp onto the product
    /// (migration 0008 — Wave 4). All 20 existing product columns are preserved.
    ///
    /// # Errors
    /// Missing product → `PRODUCT_NOT_FOUND` (domain); SQL error.
    pub async fn stamp_product_arda(
        &self,
        id: &str,
        origin_locale: &str,
        supported_locales_json: &str,
        skill_md: Option<&str>,
        skill_json: Option<&str>,
        localized_at: &str,
    ) -> DbResult<()> {
        let updated = sqlx::query(
            r#"UPDATE "Product" SET "originLocale"=?, "supportedLocalesJson"=?,
               "skillMd"=COALESCE(?, "skillMd"), "skillJson"=COALESCE(?, "skillJson"),
               "ardaLastLocalizedAt"=? WHERE "id"=?"#,
        )
        .bind(origin_locale)
        .bind(supported_locales_json)
        .bind(skill_md)
        .bind(skill_json)
        .bind(localized_at)
        .bind(id)
        .execute(self.pool_ref())
        .await?
        .rows_affected();
        if updated == 0 {
            return Err(DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {id} does not exist"),
            )
            .into());
        }
        self.audit_ext("server", "product.arda_stamp", Some(id), None).await?;
        Ok(())
    }

    /// Record a citizen vote — a one-vote-per-(product, reviewer) lock at the UNIQUE level;
    /// a duplicate vote → `EscrowAlreadyExists` (422, same domain rule).
    ///
    /// # Errors
    /// Missing product → `PRODUCT_NOT_FOUND`; duplicate vote → 422; SQL error.
    pub async fn insert_product_review(&self, r: &ProductReviewRow) -> DbResult<()> {
        let product = self.get_product(&r.product_id).await?;
        if product.is_none() {
            return Err(DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {} does not exist", r.product_id),
            )
            .into());
        }
        let duplicate: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM "ProductReview" WHERE "productId"=? AND "reviewerId"=?"#,
        )
        .bind(&r.product_id)
        .bind(&r.reviewer_id)
        .fetch_one(self.pool_ref())
        .await?;
        if duplicate > 0 {
            return Err(DomainError::new(
                DomainErrorCode::EscrowAlreadyExists,
                format!(
                    "Citizen {} has already reviewed product {}",
                    r.reviewer_id, r.product_id
                ),
            )
            .into());
        }
        sqlx::query(
            r#"INSERT INTO "ProductReview"
               ("id","productId","reviewerId","stars","comment","verifiedPurchase","maironScoreAtReview","createdAt")
               VALUES (?,?,?,?,?,?,?,?)"#,
        )
        .bind(&r.id)
        .bind(&r.product_id)
        .bind(&r.reviewer_id)
        .bind(i64::from(r.stars))
        .bind(&r.comment)
        .bind(i64::from(r.verified_purchase))
        .bind(i64::from(r.mairon_score_at_review))
        .bind(&r.created_at)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext("server", "product.review", Some(&r.id), None).await?;
        Ok(())
    }

    /// All votes for a product — newest first.
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_product_reviews(&self, product_id: &str) -> DbResult<Vec<ProductReviewRow>> {
        let sql = format!(
            "{PRODUCT_REVIEW_SELECT} WHERE \"productId\" = ? ORDER BY \"createdAt\" DESC, \"id\" DESC"
        );
        let rows = sqlx::query(sqlsafe::audited(sql))
            .bind(product_id)
            .fetch_all(self.pool_ref())
            .await?;
        Ok(rows.into_iter().map(product_review_row_from).collect())
    }

    /// Does this citizen have a verified purchase (`RELEASED` escrow) on this product?
    /// — real vote weight `1.0` versus `0.25` (anti-Sybil, Bayesian layer).
    ///
    /// # Errors
    /// SQL error.
    pub async fn has_verified_purchase(&self, product_id: &str, buyer_id: &str) -> DbResult<bool> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM "Escrow" WHERE "productRef"=? AND "buyerId"=? AND "status"='RELEASED'"#,
        )
        .bind(product_id)
        .bind(buyer_id)
        .fetch_one(self.pool_ref())
        .await?;
        Ok(count > 0)
    }

    /// Partial product update — only `Some` fields are applied. Editable
    /// columns: title/category/priceDin/description/active (registry-path
    /// summary — stock and stars are dashboard-view only).
    ///
    /// # Errors
    /// Missing product → `PRODUCT_NOT_FOUND` (domain); SQL error.
    pub async fn update_product(
        &self,
        id: &str,
        title: Option<&str>,
        category: Option<&str>,
        price_din: Option<i64>,
        description: Option<&str>,
        active: Option<bool>,
    ) -> DbResult<ProductRow> {
        let existing = self.get_product(id).await?.ok_or_else(|| {
            DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {id} does not exist"),
            )
        })?;
        let merged = ProductRow {
            title: title.map_or_else(|| existing.title.clone(), str::to_owned),
            category: category.map(str::to_owned).or_else(|| existing.category.clone()),
            price_din: price_din.unwrap_or(existing.price_din),
            description: description.map_or_else(|| existing.description.clone(), str::to_owned),
            active: active.unwrap_or(existing.active),
            ..existing.clone()
        };
        sqlx::query(
            r#"UPDATE "Product" SET "title"=?, "category"=?, "priceDin"=?,
               "description"=?, "active"=? WHERE "id"=?"#,
        )
        .bind(&merged.title)
        .bind(&merged.category)
        .bind(merged.price_din)
        .bind(&merged.description)
        .bind(i64::from(merged.active))
        .bind(id)
        .execute(self.pool_ref())
        .await?;
        self.audit_ext("server", "product.update", Some(id), None).await?;
        Ok(merged)
    }

    /// Delete a product.
    ///
    /// # Errors
    /// Missing product → `PRODUCT_NOT_FOUND` (domain); SQL error.
    pub async fn delete_product(&self, id: &str) -> DbResult<()> {
        let res = sqlx::query(r#"DELETE FROM "Product" WHERE "id"=?"#)
            .bind(id)
            .execute(self.pool_ref())
            .await?;
        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {id} does not exist"),
            )
            .into());
        }
        self.audit_ext("server", "product.delete", Some(id), None).await?;
        Ok(())
    }
}

// ── Audit + AI usage + dashboard aggregates ─────────────────────────────────────

/// Audit row for dashboard views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRowRead {
    /// Identifier
    pub id: String,
    /// Actor
    pub actor: String,
    /// Action
    pub action: String,
    /// Target
    pub target: Option<String>,
    /// Metadata (string JSON)
    pub meta: Option<String>,
    /// Time
    pub at: String,
}

impl V3Store {
    /// List audit — newest first (sample capped at `limit`, max 200).
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_audit(&self, limit: i64) -> DbResult<Vec<AuditRowRead>> {
        let rows = sqlx::query(
            r#"SELECT "id","actor","action","target","meta","at" FROM "AuditLog"
               ORDER BY "at" DESC, "id" DESC LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(self.pool_ref())
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            use sqlx::Row;
            out.push(AuditRowRead {
                id: r.get("id"),
                actor: r.get("actor"),
                action: r.get("action"),
                target: r.get("target"),
                meta: r.get("meta"),
                at: r.get("at"),
            });
        }
        Ok(out)
    }

    /// Write audit outside a domain command (products, sweep, and so on) — same TS columns.
    ///
    /// # Errors
    /// SQL error.
    pub async fn audit_ext(
        &self,
        actor: &str,
        action: &str,
        target: Option<&str>,
        meta: Option<&str>,
    ) -> DbResult<()> {
        let mut tx = self.pool_ref().begin().await?;
        insert_audit(&mut tx, actor, action, target, meta).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Insert an AI usage row — after every call (basis of the Phase 6 daily cap gate).
    ///
    /// # Errors
    /// SQL error.
    pub async fn insert_ai_usage(&self, r: &AiUsageRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "AiUsage" ("id","userId","agent","provider","model","kind","promptTokens","completionTokens","totalTokens","outcome","at")
               VALUES (?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(format!("aiuse:{}", uuid::Uuid::now_v7()))
        .bind(&r.user_id)
        .bind(&r.agent)
        .bind(&r.provider)
        .bind(&r.model)
        .bind(&r.kind)
        .bind(r.prompt_tokens)
        .bind(r.completion_tokens)
        .bind(r.total_tokens)
        .bind(&r.outcome)
        .bind(now_iso())
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// Knowledge base search — LIKE over title/body (Phase 6 `kb.search` skill).
    ///
    /// # Errors
    /// SQL error.
    pub async fn search_kb(&self, q: &str, limit: i64) -> DbResult<Vec<(String, String, String)>> {
        let pattern = format!("%{q}%");
        let rows = sqlx::query(
            r#"SELECT "id","title","body" FROM "KbEntry"
               WHERE "title" LIKE ? OR "body" LIKE ?
               ORDER BY "updatedAt" DESC LIMIT ?"#,
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(self.pool_ref())
        .await?;
        use sqlx::Row;
        Ok(rows.into_iter().map(|r| (r.get("id"), r.get("title"), r.get("body"))).collect())
    }

    /// Insert/update a knowledge base entry — `kb.upsert` skill (staff only).
    ///
    /// # Errors
    /// SQL error.
    pub async fn upsert_kb(&self, id: &str, title: &str, body: &str) -> DbResult<()> {
        let now = now_iso();
        sqlx::query(
            r#"INSERT INTO "KbEntry" ("id","title","body","createdAt","updatedAt")
               VALUES (?,?,?,?,?)
               ON CONFLICT("id") DO UPDATE SET "title"=excluded."title", "body"=excluded."body", "updatedAt"=excluded."updatedAt""#,
        )
        .bind(id)
        .bind(title)
        .bind(body)
        .bind(&now)
        .bind(&now)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// A user's total token consumption today (UTC) — for `/ai/usage`.
    ///
    /// # Errors
    /// SQL error.
    pub async fn ai_tokens_today(&self, user_id: &str) -> DbResult<(i64, i64)> {
        let today = utc_today();
        let row = sqlx::query(
            r#"SELECT COALESCE(SUM("totalTokens"),0) AS total, COUNT(*) AS calls
               FROM "AiUsage" WHERE "userId" = ? AND substr("at",1,10) = ?"#,
        )
        .bind(user_id)
        .bind(&today)
        .fetch_one(self.pool_ref())
        .await?;
        use sqlx::Row;
        Ok((row.get("total"), row.get("calls")))
    }

    /// Count AI calls (all time) — for the overview.
    ///
    /// # Errors
    /// SQL error.
    pub async fn ai_calls_total(&self) -> DbResult<i64> {
        let row = sqlx::query(r#"SELECT COUNT(*) AS c FROM "AiUsage""#)
            .fetch_one(self.pool_ref())
            .await?;
        use sqlx::Row;
        Ok(row.get("c"))
    }

    /// Account count — for `/economy` (accountCount).
    ///
    /// # Errors
    /// SQL error.
    pub async fn account_count(&self) -> DbResult<i64> {
        let row = sqlx::query(r#"SELECT COUNT(*) AS c FROM "Account""#)
            .fetch_one(self.pool_ref())
            .await?;
        use sqlx::Row;
        Ok(row.get("c"))
    }

    /// User account count (`kind='user'`) — dashboard users KPI.
    ///
    /// # Errors
    /// SQL error.
    pub async fn count_users(&self) -> DbResult<i64> {
        let row = sqlx::query(r#"SELECT COUNT(*) AS c FROM "Account" WHERE "kind" = 'user'"#)
            .fetch_one(self.pool_ref())
            .await?;
        use sqlx::Row;
        Ok(row.get("c"))
    }

    /// Count of all escrows (every status) — dashboard orders KPI.
    ///
    /// # Errors
    /// SQL error.
    pub async fn count_escrows(&self) -> DbResult<i64> {
        let row =
            sqlx::query(r#"SELECT COUNT(*) AS c FROM "Escrow""#).fetch_one(self.pool_ref()).await?;
        use sqlx::Row;
        Ok(row.get("c"))
    }

    /// HITL requests in the given status — pending approvals KPI.
    ///
    /// # Errors
    /// SQL error.
    pub async fn count_hitl(&self, status: &str) -> DbResult<i64> {
        let row = sqlx::query(r#"SELECT COUNT(*) AS c FROM "HitlRequest" WHERE "status" = ?"#)
            .bind(status)
            .fetch_one(self.pool_ref())
            .await?;
        use sqlx::Row;
        Ok(row.get("c"))
    }

    /// Total minted all time — mirror of the mint account balance.
    ///
    /// # Errors
    /// SQL error.
    pub async fn minted_all_time(&self) -> DbResult<i64> {
        let row = sqlx::query(
            r#"SELECT COALESCE(-SUM("amount"),0) AS c FROM "Posting" WHERE "accountId" = (SELECT "id" FROM "Account" WHERE "kind"='mint' LIMIT 1)"#,
        )
        .fetch_one(self.pool_ref())
        .await?;
        use sqlx::Row;
        Ok(row.get("c"))
    }

    /// Total burned all time — the burn account balance.
    ///
    /// # Errors
    /// SQL error.
    pub async fn burned_all_time(&self) -> DbResult<i64> {
        let row = sqlx::query(
            r#"SELECT COALESCE(SUM("amount"),0) AS c FROM "Posting" WHERE "accountId" = (SELECT "id" FROM "Account" WHERE "kind"='burn' LIMIT 1)"#,
        )
        .fetch_one(self.pool_ref())
        .await?;
        use sqlx::Row;
        Ok(row.get("c"))
    }

    /// A buyer's locked money — sum of open escrow amounts (FUNDED/DELIVERED/DISPUTED).
    ///
    /// # Errors
    /// SQL error.
    pub async fn locked_for_buyer(&self, buyer_id: &str) -> DbResult<i64> {
        let row = sqlx::query(
            r#"SELECT COALESCE(SUM("amount"),0) AS c FROM "Escrow"
               WHERE "buyerId" = ? AND "status" IN ('FUNDED','DELIVERED','DISPUTED')"#,
        )
        .bind(buyer_id)
        .fetch_one(self.pool_ref())
        .await?;
        use sqlx::Row;
        Ok(row.get("c"))
    }

    /// Real sales per seller — count of RELEASED escrows (sales leaderboard).
    ///
    /// # Errors
    /// SQL error.
    pub async fn sales_by_seller(&self) -> DbResult<Vec<(String, i64)>> {
        let rows = sqlx::query(
            r#"SELECT "sellerId" AS s, COUNT(*) AS c FROM "Escrow"
               WHERE "status"='RELEASED' GROUP BY "sellerId" ORDER BY c DESC, s ASC"#,
        )
        .fetch_all(self.pool_ref())
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            use sqlx::Row;
            out.push((r.get("s"), r.get("c")));
        }
        Ok(out)
    }

    /// Real sales per product — count of RELEASED escrows with a `productRef`
    /// (dashboard salesCount — derived from the live escrow table, not a column).
    ///
    /// # Errors
    /// SQL error.
    pub async fn sales_by_product(&self) -> DbResult<Vec<(String, i64)>> {
        let rows = sqlx::query(
            r#"SELECT "productRef" AS p, COUNT(*) AS c FROM "Escrow"
               WHERE "status"='RELEASED' AND "productRef" IS NOT NULL
               GROUP BY "productRef" ORDER BY c DESC"#,
        )
        .fetch_all(self.pool_ref())
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            use sqlx::Row;
            out.push((r.get("p"), r.get("c")));
        }
        Ok(out)
    }

    /// Product title by id — order views (`None` if the reference was free).
    ///
    /// # Errors
    /// SQL error.
    pub async fn product_title(&self, product_ref: Option<&str>) -> DbResult<Option<String>> {
        let Some(pref) = product_ref else { return Ok(None) };
        let row = sqlx::query(r#"SELECT "title" FROM "Product" WHERE "id" = ?"#)
            .bind(pref)
            .fetch_optional(self.pool_ref())
            .await?;
        use sqlx::Row;
        Ok(row.map(|r| r.get::<String, _>("title")))
    }

    /// 14-day revenue series (treasury balance per day) — overview chart.
    ///
    /// # Errors
    /// SQL error.
    pub async fn revenue_series_14d(&self) -> DbResult<Vec<(String, i64)>> {
        let rows = sqlx::query(
            r#"SELECT substr("Tx"."at",1,10) AS day, COALESCE(SUM("Posting"."amount"),0) AS revenue
               FROM "Posting" JOIN "Tx" ON "Tx"."id" = "Posting"."txId"
               WHERE "Posting"."accountId" = (SELECT "id" FROM "Account" WHERE "kind"='treasury' LIMIT 1)
                 AND "Posting"."amount" > 0
                 AND "Tx"."at" >= ?
               GROUP BY day ORDER BY day ASC"#,
        )
        .bind(utc_today_minus(13))
        .fetch_all(self.pool_ref())
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            use sqlx::Row;
            out.push((r.get("day"), r.get("revenue")));
        }
        Ok(out)
    }

    /// SQLite-consistent backup via `VACUUM INTO` + pruning down to the `keep` newest files.
    ///
    /// # Errors
    /// In-memory store → [`crate::error::DbError::Config`] (the server layer
    /// maps it to `BACKUP_UNSUPPORTED` 503); SQL/file error.
    pub async fn backup_to_dir(&self, dir: &std::path::Path, keep: usize) -> DbResult<BackupInfo> {
        let Some(src) = self.file_path() else {
            return Err(DbError::Config("in-memory store has no file to snapshot".into()));
        };
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(|e| DbError::Config(format!("cannot create {}: {e}", dir.display())))?;
        let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let file_name = format!("backup-{stamp}.sqlite");
        let dest = dir.join(&file_name);
        if dest.exists() {
            // Two backups within one second — numbered suffix
            let mut i = 1u32;
            loop {
                let alt = dir.join(format!("backup-{stamp}-{i}.sqlite"));
                if !alt.exists() {
                    return self.vacuum_into(&alt, src, dir, keep).await;
                }
                i += 1;
            }
        }
        self.vacuum_into(&dest, src, dir, keep).await
    }

    async fn vacuum_into(
        &self,
        dest: &std::path::Path,
        _src: &str,
        dir: &std::path::Path,
        keep: usize,
    ) -> DbResult<BackupInfo> {
        let dest_str = dest.display().to_string().replace('\'', "''");
        let sql = format!("VACUUM INTO '{dest_str}'");
        let res = sqlx::query(sqlsafe::audited(sql.clone())).execute(self.pool_ref()).await;
        if let Err(e) = res {
            return Err(DbError::Sql(e));
        }
        let size = tokio::fs::metadata(dest)
            .await
            .map_err(|e| DbError::Config(format!("backup file missing: {e}")))?
            .len();
        // Pruning — the newest ones are kept
        let mut entries: Vec<(std::path::PathBuf, std::time::SystemTime)> = Vec::new();
        let mut rd = tokio::fs::read_dir(dir)
            .await
            .map_err(|e| DbError::Config(format!("cannot read {}: {e}", dir.display())))?;
        while let Ok(Some(entry)) = rd.next_entry().await {
            let p = entry.path();
            if p.extension().is_some_and(|x| x == "sqlite")
                && let Ok(meta) = entry.metadata().await
                && let Ok(modified) = meta.modified()
            {
                entries.push((p, modified));
            }
        }
        entries.sort_by_key(|(_, m)| *m);
        entries.reverse();
        let mut removed = 0u64;
        for (path, _) in entries.iter().skip(keep) {
            if tokio::fs::remove_file(path).await.is_ok() {
                removed += 1;
            }
        }
        #[allow(clippy::cast_possible_truncation)] // removed ∈ [0, entries.len]
        let kept = i64::try_from(entries.len().saturating_sub(removed as usize)).unwrap_or(0);
        let created_at = now_iso();
        self.audit("server", "system.backup", Some(&dest.display().to_string()), None).await?;
        Ok(BackupInfo { file: dest.display().to_string(), size_bytes: size, kept, created_at })
    }
}

/// Backup result — port of `BackupCreated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupInfo {
    /// Absolute snapshot path
    pub file: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// Count kept after pruning
    pub kept: i64,
    /// ISO creation time
    pub created_at: String,
}

/// Today's UTC date as `YYYY-MM-DD`.
fn utc_today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// UTC "today minus n days" date — for the 14-day series window.
fn utc_today_minus(n: u32) -> String {
    (chrono::Utc::now() - chrono::Duration::days(i64::from(n))).format("%Y-%m-%d").to_string()
}

// silence unused-import warnings for re-exported helpers used only in docs
#[allow(unused_imports)]
use crate::store::RecentPosting;

/// Domain events as wire JSON — for manual outbox insertion (Phase 5).
#[must_use]
pub fn wire_event(event: &DomainEvent) -> (String, String) {
    (crate::wire::event_kind(event).to_owned(), event_payload(event))
}

/// Reload the full ledger state + one escrow — for future composite paths.
/// # Errors
/// SQL error or ledger/escrow state inconsistency during transactional load.
pub async fn load_world(
    store: &V3Store,
    escrow_id: Option<&str>,
) -> DbResult<(LedgerState, Option<Escrow>)> {
    let mut conn = store.acquire_conn().await?;
    let ledger = load_ledger_state(&mut conn).await?;
    let escrow = load_escrow_opt_pub(&mut conn, escrow_id).await?;
    Ok((ledger, escrow))
}

async fn load_escrow_opt_pub(
    exec: &mut sqlx::SqliteConnection,
    escrow_id: Option<&str>,
) -> DbResult<Option<Escrow>> {
    crate::store::load_escrow_opt(exec, escrow_id).await
}

#[allow(clippy::needless_pass_by_value)]
fn _event_payload_witness(ev: &DomainEvent) -> String {
    event_payload(ev)
}

/// Order delivery settlement through the domain path — for `POST /orders/{id}/status`.
///
/// # Errors
/// Domain or SQL error.
pub async fn deliver_order(
    store: &V3Store,
    escrow_id: &str,
    delivery_ref: Option<&str>,
) -> DbResult<Escrow> {
    store
        .apply(
            ApplyOptions {
                actor: "server".into(),
                action: "escrow.deliver".into(),
                target: Some(escrow_id.to_owned()),
                escrow_id: Some(escrow_id.to_owned()),
                idempotency_key: None,
            },
            |ledger, reg| {
                let o = kian_domain::deliver_escrow(
                    ledger,
                    reg,
                    &DeliverEscrowCommand {
                        escrow_id: escrow_id.to_owned(),
                        idempotency_key: None,
                        at: Some(now_iso()),
                        delivery_ref: delivery_ref.map(str::to_owned),
                    },
                )?;
                Ok(CommandOutcome {
                    ledger: o.ledger,
                    tx: o.tx,
                    escrow: Some(o.escrow),
                    events: o.events.into_iter().map(DomainEvent::Escrow).collect(),
                })
            },
        )
        .await?
        .escrow
        .ok_or_else(|| DbError::Corrupt("deliver completed without escrow".into()))
}

/// `DomainResult` for internal re-export — domain codes appear in the signatures above.
pub type QDomainResult<T> = DomainResult<T>;
