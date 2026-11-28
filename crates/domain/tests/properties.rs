//! Pillar 1 — property-based tests for the pure domain core (proptest).
//!
//! Deterministic properties (every case checked over randomly generated,
//! reproducible inputs):
//!
//! 1. **Double-entry ledger balance invariant** — every posted transaction's
//!    `SUM(Posting.amount) == 0` and the global trial balance stays `0` across
//!    random valid mint/purchase/split flows.
//! 2. **Bayesian 5-star Sybil-resistance** — the weighted mean stays strictly
//!    inside `[1.0, 5.0]` for every vote matrix, and a verified vote (weight
//!    `1.0`) always moves the mean strictly more than an unverified vote
//!    (weight `0.25`).

use kian_domain::ledger::{
    open_ledger, post, trial_balance, Account, AccountKind, LedgerState, MINT_ACCOUNT_ID,
    PostCommand, Posting, TREASURY_ACCOUNT_ID,
};
use kian_domain::mairon_rea_and_reviews::{
    summarize_product_ratings, MaironProductVerdict, MaironReaPillarScores, ProductRatingSummary,
    ProductReview,
};
use proptest::prelude::*;

const PRODUCT_ID: &str = "prod-prop";

fn verdict() -> MaironProductVerdict {
    MaironProductVerdict {
        product_id: PRODUCT_ID.to_owned(),
        total_score: 92,
        pillars: MaironReaPillarScores::default(),
        hard_failures: Vec::new(),
        approved_for_listing: true,
        visible_badge: "Mairon REA Certified: 92/100".to_owned(),
        feedback_fa: "[FA-PATCH-SLOT] feedback".to_owned(),
    }
}

fn add_account(state: &mut LedgerState, id: &str, kind: AccountKind) {
    state.accounts.insert(id.to_owned(), Account { id: id.to_owned(), kind, label: None });
}

fn ledger_with_citizens() -> LedgerState {
    let mut state = open_ledger();
    add_account(&mut state, MINT_ACCOUNT_ID, AccountKind::Mint);
    add_account(&mut state, TREASURY_ACCOUNT_ID, AccountKind::Treasury);
    add_account(&mut state, "citizen:prop-buyer", AccountKind::User);
    add_account(&mut state, "citizen:prop-seller", AccountKind::User);
    state
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Property 1 — double-entry: every transaction's postings sum to exactly
    /// zero and the trial balance of the whole ledger stays balanced.
    #[test]
    fn double_entry_postings_sum_to_zero_across_random_flows(
        mint_amount in 1i64..=1_000_000_000_000,
        purchase_fee in 0i64..=10_000,
    ) {
        let mut state = ledger_with_citizens();

        // Flow 1 — mint: the mint account emits `mint_amount` to the buyer.
        let mint_tx = PostCommand {
            id: None,
            idempotency_key: None,
            postings: vec![
                Posting::new(MINT_ACCOUNT_ID, -mint_amount),
                Posting::new("citizen:prop-buyer", mint_amount),
            ],
            memo: None,
            at: None,
        };
        let out = post(&state, &mint_tx).expect("mint must post");
        prop_assert_eq!(
            out.tx.postings.iter().map(|p| p.amount).sum::<i64>(),
            0,
            "SUM(Posting.amount) of the mint transaction must be exactly 0"
        );
        state = out.state;

        // Flow 2 — purchase with fee: buyer pays, the treasury takes the fee,
        // the seller receives the remainder — one balanced transaction.
        let fee = purchase_fee.min(mint_amount.saturating_sub(1));
        let seller_leg = mint_amount - fee;
        let purchase_tx = PostCommand {
            id: None,
            idempotency_key: None,
            postings: vec![
                Posting::new("citizen:prop-buyer", -mint_amount),
                Posting::new(TREASURY_ACCOUNT_ID, fee),
                Posting::new("citizen:prop-seller", seller_leg),
            ],
            memo: None,
            at: None,
        };
        let out = post(&state, &purchase_tx).expect("purchase must post");
        prop_assert_eq!(
            out.tx.postings.iter().map(|p| p.amount).sum::<i64>(),
            0,
            "SUM(Posting.amount) of the purchase transaction must be exactly 0"
        );
        state = out.state;

        // The whole ledger stays balanced — trial balance sum == 0.
        let tb = trial_balance(&state);
        prop_assert_eq!(tb.sum, 0, "trial balance must stay 0");
        prop_assert!(tb.balanced);

        // Conservation across citizen + treasury accounts: mint emitted exactly mint_amount.
        prop_assert_eq!(
            state.balances.get("citizen:prop-seller").copied().unwrap_or(0)
                + state.balances.get(TREASURY_ACCOUNT_ID).copied().unwrap_or(0)
                + state.balances.get("citizen:prop-buyer").copied().unwrap_or(0),
            mint_amount
        );
    }

    /// Property 2 — Bayesian 5-star Sybil resistance: the weighted mean is
    /// strictly bounded inside `[1.0, 5.0]` for every vote matrix.
    #[test]
    fn bayesian_mean_stays_strictly_bounded(
        votes in prop::collection::vec(
            (1u8..=5, prop::bool::ANY),
            1usize..=48,
        ),
    ) {
        let reviews: Vec<ProductReview> = votes
            .iter()
            .enumerate()
            .map(|(i, (stars, verified))| ProductReview {
                review_id: format!("review-prop:{i}"),
                product_id: PRODUCT_ID.to_owned(),
                reviewer_citizen_id: format!("citizen:prop-{i}"),
                stars: *stars,
                comment: "property-generated vote".to_owned(),
                verified_purchase: *verified,
                mairon_score_at_review: 90,
            })
            .collect();
        let summary: ProductRatingSummary =
            summarize_product_ratings(PRODUCT_ID, &verdict(), &reviews);
        prop_assert!(
            summary.bayesian_stars >= 1.0 && summary.bayesian_stars <= 5.0,
            "bayesian mean {} must stay inside [1.0, 5.0]",
            summary.bayesian_stars
        );
        prop_assert_eq!(summary.total_reviews, u32::try_from(reviews.len()).unwrap_or(0));
        prop_assert_eq!(
            summary.verified_reviews,
            u32::try_from(reviews.iter().filter(|r| r.verified_purchase).count()).unwrap_or(0)
        );
    }

    /// Property 3 — Sybil resistance: `k` verified five-star votes move the
    /// Bayesian mean strictly closer to `5.0` than `k` unverified ones
    /// (verified weight `1.0` vs unverified weight `0.25`).
    #[test]
    fn verified_weight_outweighs_sybil_weight(
        base_votes in prop::collection::vec((1u8..=4, prop::bool::ANY), 1usize..=24),
        sybil_count in 1usize..=12,
    ) {
        let build = |extra: &[(u8, bool)]| -> ProductRatingSummary {
            let mut votes = base_votes.clone();
            votes.extend_from_slice(extra);
            let reviews: Vec<ProductReview> = votes
                .iter()
                .enumerate()
                .map(|(i, (stars, verified))| ProductReview {
                    review_id: format!("review-sybil:{i}"),
                    product_id: PRODUCT_ID.to_owned(),
                    reviewer_citizen_id: format!("citizen:sybil-{i}"),
                    stars: *stars,
                    comment: "property-generated vote".to_owned(),
                    verified_purchase: *verified,
                    mairon_score_at_review: 90,
                })
                .collect();
            summarize_product_ratings(PRODUCT_ID, &verdict(), &reviews)
        };
        // Base mean is strictly below 5.0 (the 4.20 prior anchors it), so extra
        // five-star votes always pull upward — and the verified batch pulls
        // strictly more than the unverified batch of the same size.
        let base = build(&[]);
        let with_verified = build(&vec![(5u8, true); sybil_count]);
        let with_sybil = build(&vec![(5u8, false); sybil_count]);
        let delta_verified = with_verified.bayesian_stars - base.bayesian_stars;
        let delta_sybil = with_sybil.bayesian_stars - base.bayesian_stars;
        prop_assert!(
            delta_verified > delta_sybil,
            "verified delta {delta_verified} must exceed sybil delta {delta_sybil}"
        );
        prop_assert!(delta_sybil > 0.0, "even a sybil swarm must pull toward 5.0, not away");
    }
}
