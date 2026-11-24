//! `kian-domain` — هسته خالص دامنه KIAN.
//!
//! لایه‌ای که همه‌چیز به آن بسته می‌شود و خودش به هیچ‌چیز بسته نمی‌شود:
//! صفر وابستگی خارجی، صفر IO، صفر ساعت پنهان. یک‌به‌یک مهاجرت‌یافته از
//! کرت `dropagentx-domain` (پورت TypeScript طبق ADR-009) — طبق
//! docs/migration-map.md با «همان کرت، بازتأیید تست‌ها».
//!
//! ماژول‌ها:
//! - [`money`] — ADR-002: پول صحیح برندشده (Din — مرز 2^53−1 حفظ شده)
//! - [`ledger`] — ADR-003: دفتر دوطرفه تغییرناپذیر — Posting تنها منبع حقیقت
//! - [`escrow`] — ADR-004: ماشین وضعیت ۸ حالته امانت روی دفتر
//! - [`charter`] — منشور ۱۰ ماده‌ای کشور دیجیتال (مالیات ۱۰٪ + آستانه HITL)
//! - [`errors`] — ۲۰ کد خطا + Result
//! - [`events`] — رویدادهای دامنه با شناسه دترمینیستیک
//! - [`viral_ranker`] — رتبه‌سنج وایرال بازار (پورت `xai-org/x-algorithm` ۲۰۲۶)
//! - [`cas`] — قفل خوش‌بینانهٔ CAS + کلید idempotency امانت (پورت `JavaGuide`)
//! - [`mairon_rea_and_reviews`] — دروازهٔ داوری زندهٔ محصول `Mairon` + موتور
//!   ستارهٔ ۵گانهٔ جامعه (`morluto/rea` `v6.10.1` — Wave 2 Step 6)
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

/// شناسه کرِیت — برای smoke test و لاگ‌ها.
pub const NAME: &str = "kian-domain";

/// نسخه کرِیت — از `Cargo.toml` خوانده می‌شود.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// سلام‌سنج — نشانه زنده‌بودن سیم‌کشی.
#[must_use]
pub fn hello() -> &'static str {
    "kian_domain · KIAN is alive"
}

// ---------- re-exports اصلی (API آرگونومیک برای آداپتورها) ----------
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

    /// خوشه‌آزمون یکپارچگی مهاجرت — سفر کامل پول در یک تست:
    /// ضرب → اسکرو → تسویه با مالیات ۱۰٪ → تراز آزمایشی تقدس.
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
            Account { id: "user:kia".into(), kind: AccountKind::User, label: Some("کیا".into()) },
        )
        .unwrap();
        let l = add_account(
            &l,
            Account { id: "user:ada".into(), kind: AccountKind::User, label: None },
        )
        .unwrap();
        // ضرب ۱۰۰۰ Đ برای کیا
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
        // خرید ۴۰۰ Đ با اسکرو
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
        assert_eq!(balance_of(&o4.ledger, "user:ada").unwrap(), 360); // ۴۰۰ − ۱۰٪
        assert_eq!(balance_of(&o4.ledger, TREASURY_ACCOUNT_ID).unwrap(), 40);
        assert_eq!(balance_of(&o4.ledger, "user:kia").unwrap(), 600);
        let tb = trial_balance(&o4.ledger);
        assert!(tb.balanced);
        assert_eq!(circulating_supply(&o4.ledger), 1_000);
    }
}
