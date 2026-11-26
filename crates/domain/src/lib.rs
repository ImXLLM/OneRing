//! `kian-domain` — the pure core of the KIAN domain.
//!
//! The layer everything depends on and that itself depends on nothing:
//! zero external dependencies, zero IO, zero hidden clocks. Migrated one-to-one from
//! the `dropagentx-domain` crate (TypeScript port per ADR-009) — per
//! docs/migration-map.md with "same crate, tests re-verified".
//!
//! Modules:
//! - [`money`] — ADR-002: correct branded money (Din — 2^53-1 boundary preserved)
//! - [`ledger`] — ADR-003: immutable double-sided ledger — Posting as the single source of truth
//! - [`escrow`] — ADR-004: 8-state escrow state machine on the ledger
//! - [`charter`] — the 10-article charter of the digital nation (10% tax + HITL threshold)
//! - [`errors`] — 20 error codes + Result
//! - [`events`] — domain events with deterministic IDs
//! - [`viral_ranker`] — the market's viral ranker (port of `xai-org/x-algorithm` 2026)
//! - [`cas`] — CAS optimistic lock + escrow idempotency key (port of `JavaGuide`)
//! - [`mairon_rea_and_reviews`] — the `Mairon` live product REA arbitration gateway + the
//!   community five-star engine (`morluto/rea` `v6.10.1` — Wave 2 Step 6)
#![forbid(unsafe_code)]

pub mod cas;
pub mod charter;
pub mod errors;
pub mod escrow;
pub mod events;
pub mod ledger;
pub mod mairon_rea_and_reviews;
pub mod money;
pub mod viral_ranker;

/// Crate ID — for the smoke test and logs.
pub const NAME: &str = "kian-domain";

/// Crate version — read from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Health probe — a sign that the wiring is alive.
#[must_use]
pub fn hello() -> &'static str {
    "kian_domain · KIAN is alive"
}

// ---------- main re-exports (ergonomic API for adapters) ----------
pub use charter::{
    assert_mint_within_cap, requires_hitl, split_seller_tax, split_seller_tax_affiliate, tax_of,
    Charter, CharterArticle, SellerSplit, CHARTER_ARTICLES, DEFAULT_AFFILIATE_BPS,
    MAX_AFFILIATE_BPS,
};
pub use errors::{DomainError, DomainErrorCode, DomainResult, Meta, MetaValue};
pub use cas::{
    build_cas_update_sql, CasEscrowRecord, CasEscrowState, CasEscrowTransition, CasTransitionOutcome,
};
pub use mairon_rea_and_reviews::{
    confirm_and_publish_product, evaluate_user_product_with_mairon, submit_product_review,
    summarize_product_ratings, MaironProductVerdict, MaironReaPillarScores, ProductRatingSummary,
    ProductReview, PublishedProductListing,
};
pub use escrow::{
    cancel_escrow, create_escrow, create_escrow_with_affiliate, deliver_escrow, dispute_escrow,
    escrow_account_id, escrow_transitions, expire_escrow, find_expired, fund_escrow, open_registry,
    refund_escrow, release_escrow, CreateEscrowCommand, DeliverEscrowCommand, DisputeEscrowCommand,
    Escrow, EscrowOutcome, EscrowRefCommand, EscrowRegistry, EscrowStatus, ReleaseEscrowCommand,
    ESCROW_TERMINAL_STATUSES,
};
pub use events::{DomainEvent, EscrowEvent, EscrowEventKind, TransactionPostedEvent, EPOCH_ISO};
pub use ledger::{
    add_account, balance_of, circulating_supply, is_overdraft_violation, open_ledger, post,
    trial_balance, Account, AccountId, AccountKind, LedgerState, PostCommand, PostOutcome,
    PostedTransaction, Posting, TrialBalance, TrialBalanceRow, BURN_ACCOUNT_ID, MINT_ACCOUNT_ID,
    TREASURY_ACCOUNT_ID,
};
pub use money::{format_din, Din, MAX_DIN};
pub use viral_ranker::{
    fuse_score, inject_cold_start, score_with_defaults, ColdStartCandidate, ColdStartPolicy,
    ProductSignals, RankedProduct, ViralEngagementWeights,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(NAME, "kian-domain");
        assert_ne!(VERSION, "", "version string present");
        assert!(hello().ends_with("alive"));
    }

    /// Migration integration test cluster — the full money journey in one test:
    /// mint → escrow → settlement with 10% tax → the sacred trial balance.
    #[test]
    fn end_to_end_money_journey_conserved() {
        let l0 = open_ledger();
        let l = add_account(
            &l0,
            Account { id: MINT_ACCOUNT_ID.into(), kind: AccountKind::Mint, label: None },
        )
        .unwrap();
        let l = add_account(
            &l,
            Account { id: TREASURY_ACCOUNT_ID.into(), kind: AccountKind::Treasury, label: None },
        )
        .unwrap();
        let l = add_account(
            &l,
            Account { id: "user:kia".into(), kind: AccountKind::User, label: Some("\u{6a9}\u{6cc}\u{627}".into()) },
        )
        .unwrap();
        let l = add_account(
            &l,
            Account { id: "user:ada".into(), kind: AccountKind::User, label: None },
        )
        .unwrap();
        // Mint 1000 Đ for Kian
        let l = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: Some("kian:genesis:v1".into()),
                postings: vec![
                    Posting::new(MINT_ACCOUNT_ID, -1_000),
                    Posting::new("user:kia", 1_000),
                ],
                memo: Some("genesis".into()),
                at: None,
            },
        )
        .unwrap()
        .state;
        // Buy 400 Đ with escrow
        let o1 = create_escrow(
            &l,
            &open_registry(),
            &CreateEscrowCommand {
                id: "e9".into(),
                buyer_id: "user:kia".into(),
                seller_id: "user:ada".into(),
                amount: Din::new(400).unwrap(),
                product_ref: None,
                expires_at: None,
                at: None,
            },
        )
        .unwrap();
        let o2 = fund_escrow(
            &o1.ledger,
            &o1.registry,
            &EscrowRefCommand {
                escrow_id: "e9".into(),
                idempotency_key: Some("fund:e9".into()),
                at: None,
            },
        )
        .unwrap();
        let o3 = deliver_escrow(
            &o2.ledger,
            &o2.registry,
            &DeliverEscrowCommand {
                escrow_id: "e9".into(),
                idempotency_key: None,
                at: None,
                delivery_ref: None,
            },
        )
        .unwrap();
        let o4 = release_escrow(
            &o3.ledger,
            &o3.registry,
            &ReleaseEscrowCommand {
                escrow_id: "e9".into(),
                idempotency_key: Some("settle:e9".into()),
                at: None,
                approved_by_admin: false,
            },
        )
        .unwrap();
        assert_eq!(balance_of(&o4.ledger, "user:ada").unwrap(), 360); // 400 − 10%
        assert_eq!(balance_of(&o4.ledger, TREASURY_ACCOUNT_ID).unwrap(), 40);
        assert_eq!(balance_of(&o4.ledger, "user:kia").unwrap(), 600);
        let tb = trial_balance(&o4.ledger);
        assert!(tb.balanced);
        assert_eq!(circulating_supply(&o4.ledger), 1_000);
    }
}
