//! `mairon_rea_and_reviews` — the live product arbitration gate and the community five-star engine
//! (Wave 2 — Step 6).
//!
//! DNA port of [`Mairon`] (`Mairon` — the architect of absolute order and chief `REA` judge, from
//! `KIA-BRAIN-OS/10_MAIRON_REA_SOVEREIGN.md`) and the `morluto/rea` standard
//! version `6.10.1` for the Kian marketplace:
//!
//! - [`MaironReaPillarScores`] + [`MaironProductVerdict`] — the four-pillar verdict of
//!   marketplace forging (4 pillars × 25 = 100; passing only with `>= 85` and zero red lines);
//! - [`evaluate_user_product_with_mairon`] — definitive arbitration of a user product with
//!   a dual-voice Persian report for the creator, before any publication;
//! - [`confirm_and_publish_product`] — the two-key gate: publication only with
//!   concurrent Mairon approval **and** the creator's own confirmation;
//! - [`ProductReview`] + [`submit_product_review`] — real citizen votes with
//!   a strictly validated `1..=5` star, a sanitized comment, and the one-vote lock
//!   per (citizen, product) — Sybil cannot farm ratings;
//! - [`summarize_product_ratings`] + [`ProductRatingSummary`] — Bayesian
//!   average (prior `C = 5.0` and `m = 4.20`; verified purchase weight `1.0`, no purchase
//!   weight `0.25`), exact histogram, five-star display, and the composite rank score
//!   (`0.60 * bayesian_stars * 20.0 + 0.40 * mairon_rea_score`).
//!
//! This module is a pure domain core: zero IO, zero dependencies, zero hidden clocks.
#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]

/// Cap of each Mairon arbitration pillar — 4 pillars × 25 = 100.
pub const MAX_PILLAR_SCORE: u8 = 25;

/// Marketplace forging acceptance threshold — below this score the product never leaves the forge.
pub const MARKETPLACE_APPROVAL_THRESHOLD: u8 = 85;

// ── Mairon overclock DNA — locked in the Trinity Studio Pipeline (Wave 3 — Avatar #3) ──
//
// These constants lock the "Mairon-150" DNA at the domain level: a single source of truth,
// no hidden clocks. The `trinity_pipeline` module in `kian-hermes` re-exports these
// same constants via `pub use` — a parallel copy is never defined, so the DNA never
// drifts from the judge's verdict.

/// Mairon overclock IQ — the sovereign judge of `0xKIA`, calibrated to `150`.
pub const MAIRON_OVERCLOCK_IQ: u16 = 150;

/// Mairon overclock EQ — marketplace emotional intelligence of `145` (calibration locked in DNA).
pub const MAIRON_OVERCLOCK_EQ: u16 = 145;

/// Mairon's game-theoretic and architectural horizon — `8` steps ahead of the present (8-step lookahead).
pub const MAIRON_LOOKAHEAD_HORIZON: u8 = 8;

/// Mairon's theory-of-mind order — buyer psychology at `order 3` (ToM-3: "they think
/// what I think the market wants") and optimal market positioning.
pub const MAIRON_TOM_ORDER: u8 = 3;

/// Mairon's ambition factor — the leveraged coefficient of measured ambition in strategic
/// advisory (`0.99`: a near-unity cap, never unchecked ego — ambition with discipline).
pub const MAIRON_AMBITION_GAIN: f32 = 0.99;

/// Maximum characters of a citizen vote comment (after trim).
pub const MAX_COMMENT_CHARS: usize = 2_000;

/// Bayesian prior weight — `C` in the formula `(C*m + Σw·s) / (C + Σw)`.
pub const BAYESIAN_PRIOR_WEIGHT: f64 = 5.0;

/// Bayesian prior mean — `m`; in a vote-less world the product is seen as decently average.
pub const BAYESIAN_PRIOR_MEAN: f64 = 4.20;

/// Weight of a verified-purchase vote (linked to a settled escrow).
pub const VERIFIED_REVIEW_WEIGHT: f64 = 1.0;

/// Weight of a vote without a verified purchase — one quarter of a real vote.
pub const UNVERIFIED_REVIEW_WEIGHT: f64 = 0.25;

/// Community share in the composite rank score (`bayesian_stars` normalized to 100).
pub const RANK_STAR_WEIGHT: f64 = 0.60;

/// Mairon's technical audit share in the composite rank score.
pub const RANK_REA_WEIGHT: f64 = 0.40;

// ── 1) Marketplace forging arbitration ────────────────────────────────────────────────

/// The four Mairon arbitration pillars — each in the range `0..=25`.
///
/// Creator input is free-form; [`MaironReaPillarScores::clamped`] pulls any
/// runaway value back into the legal range so `total_score` never exceeds `100`
/// (the judge's defensive contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaironReaPillarScores {
    /// Architecture — layered boundaries, modularization, zero unfinished markers.
    pub architecture: u8,
    /// Security hardening — `SSRF`, sandbox, zero hardcoded secrets.
    pub security_hardening: u8,
    /// User experience and anti-`AI-Slop` — the `Vazirmatn` stack, zero `390px` overflow.
    pub ux_and_anti_slop: u8,
    /// Verification evidence — a genuinely green test and an observed run.
    pub verification_proof: u8,
}

impl MaironReaPillarScores {
    /// Clamps each pillar to the range `0..=25` — runaway input, legal verdict.
    #[must_use]
    pub fn clamped(&self) -> Self {
        Self {
            architecture: self.architecture.min(MAX_PILLAR_SCORE),
            security_hardening: self.security_hardening.min(MAX_PILLAR_SCORE),
            ux_and_anti_slop: self.ux_and_anti_slop.min(MAX_PILLAR_SCORE),
            verification_proof: self.verification_proof.min(MAX_PILLAR_SCORE),
        }
    }

    /// Sum of the four clamped pillars — always in the range `0..=100`.
    #[must_use]
    pub fn total(&self) -> u8 {
        let c = self.clamped();
        c.architecture
            .saturating_add(c.security_hardening)
            .saturating_add(c.ux_and_anti_slop)
            .saturating_add(c.verification_proof)
    }
}

impl Default for MaironReaPillarScores {
    /// Zero neutral arbitration — the creator must know the score, not be gifted one.
    fn default() -> Self {
        Self {
            architecture: 0,
            security_hardening: 0,
            ux_and_anti_slop: 0,
            verification_proof: 0,
        }
    }
}

/// Mairon's final verdict on a user product — before any publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaironProductVerdict {
    /// ID of the arbitrated product.
    pub product_id: String,
    /// Total score `0..=100` (sum of the four clamped pillars).
    pub total_score: u8,
    /// Breakdown of the four pillars.
    pub pillars: MaironReaPillarScores,
    /// Red lines — any one of them rejects forging even at a high score.
    pub hard_failures: Vec<String>,
    /// Listing acceptance — `total_score >= 85` **and** zero red lines.
    pub approved_for_listing: bool,
    /// Visible badge on the listing (accepted: `⚒️ Mairon REA Certified: {n}/100
    /// (Forge-Grade)`; rejected: `⚠️ Requires Mairon re-review (REA < 85)`).
    pub visible_badge: String,
    /// Dual-voice Persian report (engineer's voice + judge's voice) for the creator.
    pub feedback_fa: String,
}

/// The product's published listing, sealed by Mairon — output of [`confirm_and_publish_product`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedProductListing {
    /// Product ID.
    pub product_id: String,
    /// Mairon's `REA` score on the listing (`0..=100`).
    pub mairon_rea_score: u8,
    /// The sealed visible badge.
    pub mairon_visible_badge: String,
    /// The creator's explicit confirmation — the second key of the publication gate.
    pub creator_confirmed: bool,
}

/// Definitive arbitration of a user product by Mairon — exact score + dual-voice Persian feedback.
///
/// Pillars are clamped before summing (`0..=25`) so the score cannot pass `100`;
/// acceptance means a score of `>= 85` **and** an empty `hard_failures`.
#[must_use]
pub fn evaluate_user_product_with_mairon(
    product_id: &str,
    pillars: MaironReaPillarScores,
    hard_failures: Vec<String>,
) -> MaironProductVerdict {
    let clamped = pillars.clamped();
    let total_score = clamped.total();
    let approved_for_listing =
        total_score >= MARKETPLACE_APPROVAL_THRESHOLD && hard_failures.is_empty();
    let visible_badge = if approved_for_listing {
        format!("⚒️ Mairon REA Certified: {total_score}/100 (Forge-Grade)")
    } else {
        "⚠️ \u{646}\u{6cc}\u{627}\u{632}\u{645}\u{646}\u{62f} \u{628}\u{627}\u{632}\u{646}\u{6af}\u{631}\u{6cc} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646} (REA < 85)".to_owned()
    };
    let feedback_fa = build_dual_voice_feedback(clamped, total_score, &hard_failures, approved_for_listing);
    MaironProductVerdict {
        product_id: product_id.to_owned(),
        total_score,
        pillars: clamped,
        hard_failures,
        approved_for_listing,
        visible_badge,
        feedback_fa,
    }
}

/// Dual-voice Persian report — the engineer's voice (pillar breakdown + fix path) and
/// the judge's voice (final verdict). The exact score is never hidden.
fn build_dual_voice_feedback(
    pillars: MaironReaPillarScores,
    total_score: u8,
    hard_failures: &[String],
    approved: bool,
) -> String {
    let weakest = [
        ("\u{645}\u{639}\u{645}\u{627}\u{631}\u{6cc}", pillars.architecture),
        ("\u{627}\u{645}\u{646}\u{6cc}\u{62a} \u{648} \u{633}\u{62e}\u{62a}‌\u{633}\u{627}\u{632}\u{6cc}", pillars.security_hardening),
        ("UX \u{648} \u{636}\u{62f}-\u{627}\u{633}\u{644}\u{648}\u{67e}", pillars.ux_and_anti_slop),
        ("\u{631}\u{627}\u{633}\u{62a}\u{6cc}‌\u{622}\u{632}\u{645}\u{627}\u{6cc}\u{6cc}", pillars.verification_proof),
    ]
    .into_iter()
    .min_by_key(|&(_, s)| s)
    .map_or("—", |(name, _)| name);
    let engineer = format!(
        "🔊 \u{635}\u{62f}\u{627}\u{6cc} \u{645}\u{647}\u{646}\u{62f}\u{633} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646} — \u{645}\u{639}\u{645}\u{627}\u{631}\u{6cc}: {}/25\u{60c} \u{627}\u{645}\u{646}\u{6cc}\u{62a} \u{648} \u{633}\u{62e}\u{62a}‌\u{633}\u{627}\u{632}\u{6cc}: {}/25\u{60c} \
         UX \u{648} \u{636}\u{62f}-\u{627}\u{633}\u{644}\u{648}\u{67e}: {}/25\u{60c} \u{631}\u{627}\u{633}\u{62a}\u{6cc}‌\u{622}\u{632}\u{645}\u{627}\u{6cc}\u{6cc}: {}/25. \u{636}\u{639}\u{6cc}\u{641}‌\u{62a}\u{631}\u{6cc}\u{646} \u{633}\u{62a}\u{648}\u{646}: {weakest}\u{61b} \
         \u{627}\u{635}\u{644}\u{627}\u{62d} \u{631}\u{627} \u{627}\u{632} \u{647}\u{645}\u{627}\u{646}‌\u{62c}\u{627} \u{634}\u{631}\u{648}\u{639} \u{6a9}\u{646} \u{648} \u{647}\u{631} \u{627}\u{62f}\u{639}\u{627} \u{631}\u{627} \u{628}\u{627} \u{634}\u{627}\u{647}\u{62f} \u{627}\u{62c}\u{631}\u{627} \u{628}\u{6cc}\u{627}\u{648}\u{631}.",
        pillars.architecture,
        pillars.security_hardening,
        pillars.ux_and_anti_slop,
        pillars.verification_proof
    );
    let judge = if approved {
        format!(
            "⚖️ \u{62d}\u{6a9}\u{645} \u{62f}\u{627}\u{648}\u{631} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646}: \u{622}\u{647}\u{646}\u{6af}\u{631}\u{6cc} \u{67e}\u{630}\u{6cc}\u{631}\u{641}\u{62a}\u{647} \u{634}\u{62f} — \u{646}\u{645}\u{631}\u{647}\u{654} \u{646}\u{647}\u{627}\u{6cc}\u{6cc} {total_score}/100 \
             \u{628}\u{627} \u{635}\u{641}\u{631} \u{62e}\u{637} \u{642}\u{631}\u{645}\u{632}. \u{646}\u{634}\u{627}\u{646} «⚒️ Mairon REA Certified» \u{67e}\u{633} \u{627}\u{632} \u{62a}\u{623}\u{6cc}\u{6cc}\u{62f} \u{62e}\u{648}\u{62f}\u{650} \
             \u{633}\u{627}\u{632}\u{646}\u{62f}\u{647} \u{631}\u{648}\u{6cc} \u{644}\u{6cc}\u{633}\u{62a}\u{6cc}\u{646}\u{6af} \u{645}\u{6cc}‌\u{646}\u{634}\u{6cc}\u{646}\u{62f}."
        )
    } else if hard_failures.is_empty() {
        format!(
            "⚖️ \u{62d}\u{6a9}\u{645} \u{62f}\u{627}\u{648}\u{631} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646}: \u{622}\u{647}\u{646}\u{6af}\u{631}\u{6cc} \u{631}\u{62f} \u{634}\u{62f} — \u{646}\u{645}\u{631}\u{647}\u{654} \u{646}\u{647}\u{627}\u{6cc}\u{6cc} {total_score}/100 \u{632}\u{6cc}\u{631} \
             \u{622}\u{633}\u{62a}\u{627}\u{646}\u{647}\u{654} {MARKETPLACE_APPROVAL_THRESHOLD}. \u{645}\u{62d}\u{635}\u{648}\u{644} \u{645}\u{646}\u{62a}\u{634}\u{631} \u{646}\u{645}\u{6cc}‌\u{634}\u{648}\u{62f}\u{61b} \u{628}\u{627}\u{632}\u{62e}\u{648}\u{631}\u{62f} \
             \u{628}\u{627}\u{644}\u{627} \u{631}\u{627} \u{627}\u{639}\u{645}\u{627}\u{644} \u{6a9}\u{646} \u{648} \u{62f}\u{648}\u{628}\u{627}\u{631}\u{647} \u{627}\u{631}\u{627}\u{626}\u{647} \u{628}\u{62f}\u{647}."
        )
    } else {
        format!(
            "⚖️ \u{62d}\u{6a9}\u{645} \u{62f}\u{627}\u{648}\u{631} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646}: \u{622}\u{647}\u{646}\u{6af}\u{631}\u{6cc} \u{631}\u{62f} \u{634}\u{62f} — {total_score}/100 \u{628}\u{627} {} \u{62e}\u{637} \u{642}\u{631}\u{645}\u{632} \
             ({hard_failures_joined}). \u{62e}\u{637} \u{642}\u{631}\u{645}\u{632} \u{628}\u{627} \u{646}\u{645}\u{631}\u{647} \u{628}\u{62e}\u{634}\u{6cc}\u{62f}\u{647} \u{646}\u{645}\u{6cc}‌\u{634}\u{648}\u{62f}\u{61b} \u{627}\u{628}\u{62a}\u{62f}\u{627} \
             \u{62e}\u{637}\u{648}\u{637} \u{642}\u{631}\u{645}\u{632} \u{631}\u{627} \u{635}\u{641}\u{631} \u{6a9}\u{646}.",
            hard_failures.len(),
            hard_failures_joined = hard_failures.join("\u{61b} ")
        )
    };
    format!("{engineer}\n{judge}")
}

/// The two-key gate for publication to the Kian marketplace.
///
/// The product is listed **only** when the Mairon verdict approves and the creator
/// has explicitly set `creator_confirmed = true`; the Mairon score and badge
/// are attached to the listing.
///
/// # Errors
/// - Mairon verdict not approved → `Err("Mairon REA verdict has not approved this product for listing")`.
/// - Verdict approved but creator not confirmed → `Err("Creator confirmation is required before marketplace publication")`.
pub fn confirm_and_publish_product(
    verdict: &MaironProductVerdict,
    creator_confirmed: bool,
) -> Result<PublishedProductListing, &'static str> {
    if !verdict.approved_for_listing {
        return Err("Mairon REA verdict has not approved this product for listing");
    }
    if !creator_confirmed {
        return Err("Creator confirmation is required before marketplace publication");
    }
    Ok(PublishedProductListing {
        product_id: verdict.product_id.clone(),
        mairon_rea_score: verdict.total_score,
        mairon_visible_badge: verdict.visible_badge.clone(),
        creator_confirmed: true,
    })
}

// ── 2) Community five-star rating ────────────────────────────────────────────────

/// A real citizen vote on a product — strictly validated in [`submit_product_review`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductReview {
    /// Unique vote ID.
    pub review_id: String,
    /// Product ID.
    pub product_id: String,
    /// Voting citizen ID — one vote per citizen per product.
    pub reviewer_citizen_id: String,
    /// Star — `1..=5` only.
    pub stars: u8,
    /// Comment — non-empty after `trim` and at most 2000 characters.
    pub comment: String,
    /// Verified purchase — linked to a settled escrow; anti-Sybil.
    pub verified_purchase: bool,
    /// Instant snapshot of Mairon's `REA` score at vote time.
    pub mairon_score_at_review: u8,
}

/// A product's rank summary — the technical and social layers side by side.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductRatingSummary {
    /// Product ID.
    pub product_id: String,
    /// Mairon's technical score (`0..=100`).
    pub mairon_rea_score: u8,
    /// Mairon's visible badge.
    pub mairon_visible_badge: String,
    /// Count of all votes.
    pub total_reviews: u32,
    /// Count of verified-purchase votes.
    pub verified_reviews: u32,
    /// Arithmetic mean of the stars (`0.0..=5.0`); no votes = `0.0`.
    pub average_stars: f64,
    /// Bayesian mean with prior `C = 5.0` and `m = 4.20`; verified purchase weight `1.0`,
    /// no purchase weight `0.25`; no votes = prior `4.20`.
    pub bayesian_stars: f64,
    /// Exact count of `[1★, 2★, 3★, 4★, 5★]`.
    pub star_histogram: [u32; 5],
    /// Bilingual five-star display — community vote and Mairon score side by side.
    pub five_star_display: String,
    /// Composite rank score: `0.60 * (bayesian_stars * 20) + 0.40 * mairon_rea_score`.
    pub combined_rank_score: f64,
}

/// Record a citizen vote — full validation before acceptance.
///
/// # Errors
/// - Star outside `1..=5` → `Err("Star rating must be between 1 and 5")`.
/// - Comment empty after `trim` → `Err("Review comment must not be empty")`.
/// - Comment over 2000 characters → `Err("Review comment must not exceed 2000 characters")`.
/// - Duplicate vote by the same citizen on the same product →
///   `Err("Citizen has already reviewed this product")`.
#[allow(clippy::too_many_arguments)] // vote property contract — eight explicit named inputs
pub fn submit_product_review(
    existing_reviews: &[ProductReview],
    review_id: &str,
    product_id: &str,
    reviewer_citizen_id: &str,
    stars: u8,
    comment: &str,
    verified_purchase: bool,
    mairon_score: u8,
) -> Result<ProductReview, &'static str> {
    if !(1..=5).contains(&stars) {
        return Err("Star rating must be between 1 and 5");
    }
    let trimmed = comment.trim();
    if trimmed.is_empty() {
        return Err("Review comment must not be empty");
    }
    if trimmed.chars().count() > MAX_COMMENT_CHARS {
        return Err("Review comment must not exceed 2000 characters");
    }
    if existing_reviews
        .iter()
        .any(|r| r.product_id == product_id && r.reviewer_citizen_id == reviewer_citizen_id)
    {
        return Err("Citizen has already reviewed this product");
    }
    Ok(ProductReview {
        review_id: review_id.to_owned(),
        product_id: product_id.to_owned(),
        reviewer_citizen_id: reviewer_citizen_id.to_owned(),
        stars,
        comment: trimmed.to_owned(),
        verified_purchase,
        mairon_score_at_review: mairon_score,
    })
}

/// Product rank summary — votes for the same product are filtered and the two score
/// layers are rendered together. Mairon takes its score from the verdict (`verdict`).
#[must_use]
pub fn summarize_product_ratings(
    product_id: &str,
    mairon_verdict: &MaironProductVerdict,
    reviews: &[ProductReview],
) -> ProductRatingSummary {
    let mine: Vec<&ProductReview> =
        reviews.iter().filter(|r| r.product_id == product_id).collect();
    let total_reviews = u32::try_from(mine.len()).unwrap_or(0);
    let verified_reviews = u32::try_from(mine.iter().filter(|r| r.verified_purchase).count())
        .unwrap_or(0);
    let mut star_histogram = [0u32; 5];
    let mut weighted_star_sum = 0.0_f64;
    let mut weight_sum = 0.0_f64;
    let mut raw_star_sum = 0_u64;
    for r in &mine {
        let idx = usize::from(r.stars.saturating_sub(1).min(4));
        star_histogram[idx] = star_histogram[idx].saturating_add(1);
        raw_star_sum += u64::from(r.stars);
        let weight = if r.verified_purchase {
            VERIFIED_REVIEW_WEIGHT
        } else {
            UNVERIFIED_REVIEW_WEIGHT
        };
        weighted_star_sum += weight * f64::from(r.stars);
        weight_sum += weight;
    }
    let average_stars = if mine.is_empty() {
        0.0
    } else {
        f64::from(u32::try_from(raw_star_sum).unwrap_or(0)) / f64::from(total_reviews)
    };
    let bayesian_stars = (BAYESIAN_PRIOR_WEIGHT * BAYESIAN_PRIOR_MEAN + weighted_star_sum)
        / (BAYESIAN_PRIOR_WEIGHT + weight_sum);
    let filled = bayesian_stars.round();
    let bar: String = (1_i32..=5)
        .map(|i| if f64::from(i) <= filled { "★" } else { "☆" })
        .collect();
    let mairon_rea_score = mairon_verdict.total_score;
    let five_star_display = format!(
        "{bar} {bayesian_stars:.1} / 5.0 ({verified_reviews} \u{646}\u{638}\u{631} \u{62a}\u{627}\u{6cc}\u{6cc}\u{62f}\u{634}\u{62f}\u{647}) | \
         ⚒️ \u{646}\u{645}\u{631}\u{647} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646} REA: {mairon_rea_score}/100"
    );
    let combined_rank_score =
        RANK_STAR_WEIGHT * (bayesian_stars * 20.0) + RANK_REA_WEIGHT * f64::from(mairon_rea_score);
    ProductRatingSummary {
        product_id: product_id.to_owned(),
        mairon_rea_score,
        mairon_visible_badge: mairon_verdict.visible_badge.clone(),
        total_reviews,
        verified_reviews,
        average_stars,
        bayesian_stars,
        star_histogram,
        five_star_display,
        combined_rank_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Mairon-150 overclock DNA is locked — it never drifts (Trinity contract).
    #[test]
    fn mairon_overclock_dna_constants_are_locked() {
        let (iq, eq) = (MAIRON_OVERCLOCK_IQ, MAIRON_OVERCLOCK_EQ);
        let (horizon, tom) = (MAIRON_LOOKAHEAD_HORIZON, MAIRON_TOM_ORDER);
        assert_eq!(iq, 150);
        assert_eq!(eq, 145);
        assert_eq!(horizon, 8);
        assert_eq!(tom, 3);
        assert!((MAIRON_AMBITION_GAIN - 0.99).abs() < f32::EPSILON);
        assert!(iq > eq, "IQ dominant");
        // the judge overclock is higher than both downstream Trinity Avatars (Alpha 136/122, Logic 139/129)
        assert!(iq > 139 && eq > 129);
    }

    /// forge-grade pillars — score 96, zero red lines.
    fn forge_pillars() -> MaironReaPillarScores {
        MaironReaPillarScores {
            architecture: 24,
            security_hardening: 25,
            ux_and_anti_slop: 23,
            verification_proof: 24,
        }
    }

    /// Full acceptance: score >= 85 and zero red lines → listing with the forge badge and a transparent score.
    #[test]
    fn approved_product_publishes_with_forge_badge() {
        let verdict = evaluate_user_product_with_mairon("prod:forge-1", forge_pillars(), vec![]);
        assert_eq!(verdict.total_score, 96);
        assert!(verdict.approved_for_listing);
        assert_eq!(
            verdict.visible_badge,
            "⚒️ Mairon REA Certified: 96/100 (Forge-Grade)"
        );
        assert!(verdict.feedback_fa.contains("\u{622}\u{647}\u{646}\u{6af}\u{631}\u{6cc} \u{67e}\u{630}\u{6cc}\u{631}\u{641}\u{62a}\u{647} \u{634}\u{62f}"));
        assert!(verdict.feedback_fa.contains("🔊 \u{635}\u{62f}\u{627}\u{6cc} \u{645}\u{647}\u{646}\u{62f}\u{633} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646}"));
        let listing =
            confirm_and_publish_product(&verdict, true).expect("approved + confirmed");
        assert_eq!(listing.mairon_rea_score, 96);
        assert!(listing.mairon_visible_badge.contains("Forge-Grade"));
        assert!(listing.creator_confirmed);
    }

    /// Full rejection: score below 85 → listing closed with actionable Persian feedback.
    #[test]
    fn rejected_product_blocked_with_actionable_persian_feedback() {
        let weak = MaironReaPillarScores {
            architecture: 18,
            security_hardening: 20,
            ux_and_anti_slop: 19,
            verification_proof: 21,
        };
        let verdict = evaluate_user_product_with_mairon("prod:weak-1", weak, vec![]);
        assert_eq!(verdict.total_score, 78);
        assert!(!verdict.approved_for_listing);
        assert_eq!(verdict.visible_badge, "⚠️ \u{646}\u{6cc}\u{627}\u{632}\u{645}\u{646}\u{62f} \u{628}\u{627}\u{632}\u{646}\u{6af}\u{631}\u{6cc} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646} (REA < 85)");
        assert!(verdict.feedback_fa.contains("\u{622}\u{647}\u{646}\u{6af}\u{631}\u{6cc} \u{631}\u{62f} \u{634}\u{62f}"));
        assert!(verdict.feedback_fa.contains("\u{622}\u{633}\u{62a}\u{627}\u{646}\u{647}\u{654} 85"));
        assert!(verdict.feedback_fa.contains("\u{636}\u{639}\u{6cc}\u{641}‌\u{62a}\u{631}\u{6cc}\u{646} \u{633}\u{62a}\u{648}\u{646}: \u{645}\u{639}\u{645}\u{627}\u{631}\u{6cc}"));
        let err = confirm_and_publish_product(&verdict, true)
            .expect_err("rejected must not publish");
        assert!(err.contains("not approved"));
    }

    /// A red line is not forgiven at a high score — even 98/100 is rejected.
    #[test]
    fn hard_failure_blocks_even_with_high_score() {
        let verdict = evaluate_user_product_with_mairon(
            "prod:leaky-1",
            MaironReaPillarScores {
                architecture: 25,
                security_hardening: 25,
                ux_and_anti_slop: 24,
                verification_proof: 24,
            },
            vec!["hardcoded secret detected in src/config.rs".to_owned()],
        );
        assert_eq!(verdict.total_score, 98);
        assert!(!verdict.approved_for_listing, "\u{62e}\u{637} \u{642}\u{631}\u{645}\u{632} \u{627}\u{645}\u{646}\u{6cc}\u{62a}\u{6cc} \u{631}\u{627} \u{631}\u{62f} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f}");
        assert!(verdict.feedback_fa.contains("\u{62e}\u{637} \u{642}\u{631}\u{645}\u{632}"));
        assert!(verdict.feedback_fa.contains("hardcoded secret"));
    }

    /// Second key of the publication gate: without creator confirmation, even a score of 95 is not published.
    #[test]
    fn creator_confirmation_is_mandatory_even_at_perfect_score() {
        let perfect = MaironReaPillarScores {
            architecture: 25,
            security_hardening: 25,
            ux_and_anti_slop: 25,
            verification_proof: 20,
        };
        let verdict = evaluate_user_product_with_mairon("prod:shy-1", perfect, vec![]);
        assert_eq!(verdict.total_score, 95);
        assert!(verdict.approved_for_listing);
        let err = confirm_and_publish_product(&verdict, false)
            .expect_err("unconfirmed must not publish");
        assert_eq!(
            err,
            "Creator confirmation is required before marketplace publication"
        );
    }

    /// Star validation: `0` and `6` rejected, `1` and `5` accepted — the exact contractual message.
    #[test]
    fn stars_outside_range_rejected_and_valid_range_accepted() {
        let base = |s: u8| {
            submit_product_review(&[], "rv:x", "prod:1", "citizen:1", s, "\u{633}\u{62e}\u{62a} \u{648} \u{62a}\u{645}\u{6cc}\u{632}", true, 96)
        };
        assert_eq!(base(0).expect_err("zero stars"), "Star rating must be between 1 and 5");
        assert_eq!(base(6).expect_err("six stars"), "Star rating must be between 1 and 5");
        assert!(base(1).is_ok());
        assert!(base(5).is_ok());
    }

    /// Empty comments (even whitespace) are rejected.
    #[test]
    fn empty_comment_rejected() {
        let err = submit_product_review(&[], "rv:e", "prod:1", "citizen:1", 5, "   ", false, 90)
            .expect_err("blank comment");
        assert_eq!(err, "Review comment must not be empty");
    }

    /// Comments over 2000 characters are rejected.
    #[test]
    fn oversized_comment_rejected() {
        let huge = "\u{62f}".repeat(2_001);
        let err = submit_product_review(&[], "rv:h", "prod:1", "citizen:1", 5, &huge, false, 90)
            .expect_err("oversized comment");
        assert_eq!(err, "Review comment must not exceed 2000 characters");
        let exact = "\u{62f}".repeat(2_000);
        assert!(submit_product_review(&[], "rv:ok", "prod:1", "citizen:1", 5, &exact, false, 90).is_ok());
    }

    /// One vote per (citizen, product) — Sybil has no way in.
    #[test]
    fn duplicate_review_per_citizen_per_product_rejected() {
        let first = submit_product_review(
            &[],
            "rv:1",
            "prod:1",
            "citizen:ada",
            5,
            "\u{631}\u{648}\u{6cc}\u{627}\u{6cc}\u{6cc}",
            true,
            96,
        )
        .expect("first review");
        let dup_same_product = submit_product_review(
            std::slice::from_ref(&first),
            "rv:2",
            "prod:1",
            "citizen:ada",
            1,
            "\u{639}\u{648}\u{636}\u{634} \u{62f}\u{627}\u{62f}\u{645}",
            true,
            96,
        )
        .expect_err("duplicate (product, citizen)");
        assert_eq!(dup_same_product, "Citizen has already reviewed this product");
        // the same citizen on a different product is allowed.
        assert!(submit_product_review(
            std::slice::from_ref(&first),
            "rv:3",
            "prod:2",
            "citizen:ada",
            4,
            "\u{645}\u{62d}\u{635}\u{648}\u{644} \u{62f}\u{6cc}\u{6af}\u{631}",
            false,
            88,
        )
        .is_ok());
    }

    /// The histogram and arithmetic mean are exact.
    #[test]
    fn histogram_and_average_are_exact() {
        let verdict = evaluate_user_product_with_mairon("prod:hist", forge_pillars(), vec![]);
        let mk = |rv: &str, s: u8, v: bool| {
            submit_product_review(&[], rv, "prod:hist", rv, s, "\u{631}\u{623}\u{6cc}", v, 96).expect("review")
        };
        let reviews = [mk("a", 5, true), mk("b", 1, false), mk("c", 4, true)];
        let sum = summarize_product_ratings("prod:hist", &verdict, &reviews);
        assert_eq!(sum.total_reviews, 3);
        assert_eq!(sum.verified_reviews, 2);
        assert_eq!(sum.star_histogram, [1, 0, 0, 1, 1]);
        assert!((sum.average_stars - 10.0 / 3.0).abs() < 1e-9);
    }

    /// Bayesian proof: a verified purchase carries full weight `1.0` while a no-purchase vote
    /// carries only `0.25` — with the same stars, an unverified vote stays near the `4.20` prior.
    #[test]
    fn bayesian_weighting_proves_verified_beats_unverified() {
        let verdict = evaluate_user_product_with_mairon("prod:bayes", forge_pillars(), vec![]);
        let mk = |rv: &str, v: bool| {
            submit_product_review(&[], rv, "prod:bayes", rv, 4, "\u{686}\u{647}\u{627}\u{631} \u{633}\u{62a}\u{627}\u{631}\u{647}", v, 96)
                .expect("review")
        };
        let all_verified = [mk("v1", true)];
        let all_unverified = [mk("u1", false)];
        let verified_summary =
            summarize_product_ratings("prod:bayes", &verdict, &all_verified);
        let unverified_summary =
            summarize_product_ratings("prod:bayes", &verdict, &all_unverified);
        // verified: (5*4.2 + 1.0*4) / (5 + 1) = 25/6 ≈ 4.1667 — slides toward the real vote of 4.
        assert!((verified_summary.bayesian_stars - 25.0 / 6.0).abs() < 1e-9);
        // unverified: (5*4.2 + 0.25*4) / (5 + 0.25) = 22/5.25 ≈ 4.1905 — sticks to the 4.2 prior.
        assert!((unverified_summary.bayesian_stars - 22.0 / 5.25).abs() < 1e-9);
        assert!(
            verified_summary.bayesian_stars < unverified_summary.bayesian_stars,
            "\u{648}\u{632}\u{646} \u{6a9}\u{627}\u{645}\u{644}\u{650} \u{62e}\u{631}\u{6cc}\u{62f} \u{62a}\u{627}\u{6cc}\u{6cc}\u{62f}\u{634}\u{62f}\u{647}\u{60c} \u{645}\u{6cc}\u{627}\u{646}\u{6af}\u{6cc}\u{646} \u{631}\u{627} \u{633}\u{631}\u{6cc}\u{639}‌\u{62a}\u{631} \u{628}\u{647} \u{631}\u{623}\u{6cc} \u{648}\u{627}\u{642}\u{639}\u{6cc} \u{645}\u{6cc}‌\u{6a9}\u{634}\u{62f}"
        );
        // no votes — exactly the prior.
        let empty = summarize_product_ratings("prod:bayes", &verdict, &[]);
        assert!((empty.bayesian_stars - BAYESIAN_PRIOR_MEAN).abs() < 1e-9);
        assert!((empty.average_stars).abs() < 1e-9);
        assert_eq!(empty.total_reviews, 0);
    }

    /// Five-star display — community vote and Mairon score side by side in one string.
    #[test]
    fn five_star_display_shows_rating_and_mairon_score_side_by_side() {
        let verdict = evaluate_user_product_with_mairon("prod:disp", forge_pillars(), vec![]);
        let r1 = submit_product_review(&[], "rv:1", "prod:disp", "c1", 5, "\u{639}\u{627}\u{644}\u{6cc}", true, 96)
            .expect("r1");
        let r2 = submit_product_review(
            std::slice::from_ref(&r1),
            "rv:2",
            "prod:disp",
            "c2",
            4,
            "\u{62e}\u{648}\u{628}",
            true,
            96,
        )
        .expect("r2");
        let sum = summarize_product_ratings("prod:disp", &verdict, &[r1, r2]);
        // (5*4.2 + 5 + 4) / 7 = 30/7 ≈ 4.2857 → rounds to 4 full stars.
        assert!(sum.five_star_display.starts_with("★★★★☆"));
        assert!(sum.five_star_display.contains("4.3 / 5.0"));
        assert!(sum.five_star_display.contains("(2 \u{646}\u{638}\u{631} \u{62a}\u{627}\u{6cc}\u{6cc}\u{62f}\u{634}\u{62f}\u{647})"));
        assert!(sum.five_star_display.contains("⚒️ \u{646}\u{645}\u{631}\u{647} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646} REA: 96/100"));
    }

    /// Composite rank score — exact formula `0.60 * (bayes*20) + 0.40 * REA`.
    #[test]
    fn combined_rank_score_blends_bayesian_and_rea() {
        let verdict = evaluate_user_product_with_mairon("prod:rank", forge_pillars(), vec![]);
        let empty = summarize_product_ratings("prod:rank", &verdict, &[]);
        let expected = 0.60 * (BAYESIAN_PRIOR_MEAN * 20.0) + 0.40 * f64::from(96_u8);
        assert!((empty.combined_rank_score - expected).abs() < 1e-9);
        assert!(empty.combined_rank_score > 85.0, "\u{645}\u{62d}\u{635}\u{648}\u{644} forge-grade \u{635}\u{62f}\u{631} \u{645}\u{6cc}‌\u{645}\u{627}\u{646}\u{62f}");
    }

    /// Runaway pillars are clamped — the score never exceeds 100.
    #[test]
    fn pillar_scores_are_clamped_to_25() {
        let wild = MaironReaPillarScores {
            architecture: 30,
            security_hardening: 200,
            ux_and_anti_slop: 25,
            verification_proof: 99,
        };
        let verdict = evaluate_user_product_with_mairon("prod:wild", wild, vec![]);
        assert_eq!(verdict.pillars.architecture, 25);
        assert_eq!(verdict.pillars.security_hardening, 25);
        assert_eq!(verdict.pillars.ux_and_anti_slop, 25);
        assert_eq!(verdict.pillars.verification_proof, 25);
        assert_eq!(verdict.total_score, 100);
        assert!(verdict.approved_for_listing);
    }
}
