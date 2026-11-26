//! The Kian market viral ranker — an exact port of `xai-org/x-algorithm` (commit 269340c).
//!
//! Port sources:
//! - `home-mixer/params/param.rs` — exact engagement-signal weights (whole file, 2026)
//! - `home-mixer/scorers/author_cold_start.rs` — bootstrap for novice creators
//! - `xai-value-model/scoring.rs` — the `fuse_heads` formula (sum of weight × probability)
//!
//! The X-algorithm golden rule: weights multiply each interaction's **predicted probability**,
//! not raw counts. `ShareViaCopyLink = +20.0` is the strongest positive signal
//! (about 28x favorite=0.708) and `Report = -234.0` is the strongest
//! negative signal — one report cancels 30 positive likes.
//!
//! Cold start (`ColdStartPolicy`): a product aged <= 7200 seconds (2 hours),
//! a creator with < 50,000 followers and < 200 impressions qualifies for injection into
//! discovery slots 15..=16 — Thompson sampling over `Beta(0.75, 49.25)`
//! (mean 0.015) + a deterministic exploration bonus (`SplitMix64` over the product id),
//! with no non-deterministic RNG.
#![forbid(unsafe_code)]

/// Exact engagement weights — a line-by-line port of `home-mixer/params/param.rs`.
///
/// Negative values are penalties and never sparkle as rewards; one large negative
/// signal (report/mute) pushes the product's rank below zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViralEngagementWeights {
    /// `share_via_copy_link = 20.0` — the strongest positive signal (copying the product link).
    pub share_via_copy_link: f64,
    /// `reply = 5.0`.
    pub reply: f64,
    /// `bidirectional_reply_boost = 15.0` — when both parties have replied to each other
    /// (total bidirectional reply = 20.0, on par with copy-link).
    pub bidirectional_reply_boost: f64,
    /// `quote = 5.0` — a product remix/quote.
    pub quote: f64,
    /// `share_via_dm = 5.0`.
    pub share_via_dm: f64,
    /// `follow_creator = 4.0`.
    pub follow_creator: f64,
    /// `share = 2.118`.
    pub share: f64,
    /// `repost = 1.0` (`retweet_weight`).
    pub repost: f64,
    /// `favorite = 0.708`.
    pub favorite: f64,
    /// `cont_click_dwell_per_sec = 0.385` — dwell seconds after a click.
    pub cont_click_dwell_per_sec: f64,
    /// `click = 0.274`.
    pub click: f64,
    /// `open_link = 0.2`.
    pub open_link: f64,
    /// `dwell = 0.05` — a plain scroll dwell.
    pub dwell: f64,
    /// `not_dwelled = -0.02` — seen but not read.
    pub not_dwelled: f64,
    /// `block_creator = -31.2`.
    pub block_creator: f64,
    /// `not_interested = -53.07`.
    pub not_interested: f64,
    /// `mute_creator = -58.8`.
    pub mute_creator: f64,
    /// `report = -234.0` — the strongest negative signal.
    pub report: f64,
}

impl Default for ViralEngagementWeights {
    fn default() -> Self {
        Self {
            share_via_copy_link: 20.0,
            reply: 5.0,
            bidirectional_reply_boost: 15.0,
            quote: 5.0,
            share_via_dm: 5.0,
            follow_creator: 4.0,
            share: 2.118,
            repost: 1.0,
            favorite: 0.708,
            cont_click_dwell_per_sec: 0.385,
            click: 0.274,
            open_link: 0.2,
            dwell: 0.05,
            not_dwelled: -0.02,
            block_creator: -31.2,
            not_interested: -53.07,
            mute_creator: -58.8,
            report: -234.0,
        }
    }
}

/// Predicted engagement probabilities for a market product (`CandidateSignals`).
///
/// Every field is in [0, 1] except `cont_click_dwell_secs` and `dwell_secs`, which
/// are «expected seconds» (mean probability × seconds).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ProductSignals {
    pub share_via_copy_link: f64,
    /// A plain reply; `bidirectional_reply` sums it with the +15 boost.
    pub reply: f64,
    /// Bidirectional reply (buyer and builder answered each other)?
    pub bidirectional_reply: bool,
    pub quote: f64,
    pub share_via_dm: f64,
    pub follow_creator: f64,
    pub share: f64,
    pub repost: f64,
    pub favorite: f64,
    pub cont_click_dwell_secs: f64,
    pub click: f64,
    pub open_link: f64,
    pub dwell_secs: f64,
    pub not_dwelled: f64,
    pub block_creator: f64,
    pub not_interested: f64,
    pub mute_creator: f64,
    pub report: f64,
}

/// The `xai-value-model/scoring.rs::fuse_heads` fusion formula — sum of weight × signal.
///
/// A bidirectional reply counts as `reply + bidirectional_reply_boost = 20.0`;
/// negatives are always subtracted. The output can go negative — a reported product
/// is effectively gone from discovery.
#[must_use]
pub fn fuse_score(w: &ViralEngagementWeights, s: &ProductSignals) -> f64 {
    let reply_w = if s.bidirectional_reply {
        w.reply + w.bidirectional_reply_boost
    } else {
        w.reply
    };
    s.share_via_copy_link * w.share_via_copy_link
        + s.reply * reply_w
        + s.quote * w.quote
        + s.share_via_dm * w.share_via_dm
        + s.follow_creator * w.follow_creator
        + s.share * w.share
        + s.repost * w.repost
        + s.favorite * w.favorite
        + s.cont_click_dwell_secs * w.cont_click_dwell_per_sec
        + s.click * w.click
        + s.open_link * w.open_link
        + s.dwell_secs * w.dwell
        + s.not_dwelled * w.not_dwelled
        + s.block_creator * w.block_creator
        + s.not_interested * w.not_interested
        + s.mute_creator * w.mute_creator
        + s.report * w.report
}

/// Fusion score with the 2026 X-algorithm default weights.
#[must_use]
pub fn score_with_defaults(s: &ProductSignals) -> f64 {
    fuse_score(&ViralEngagementWeights::default(), s)
}

// ── Cold-Start (port of `author_cold_start.rs`) ────────────────────────────────

/// Creator cold-start policy — exactly the upstream test parameters:
/// `max_post_age=7200`, `beta_alpha0=0.75`, `beta_beta0=49.25`.
///
/// A qualifying product is injected into discovery slots `slot_min..=slot_max` (default
/// 15..=16); displaced items are **not removed**, they only shift down one
/// step (ranking fairness).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColdStartPolicy {
    /// A creator with >= this many followers is no longer a novice.
    pub max_creator_followers: u64,
    /// Product age <= this value (7200 seconds = 2 hours).
    pub max_product_age_secs: i64,
    /// Impressions >= this value, no longer cold.
    pub max_impressions: u64,
    /// First discovery slot (zero-based) — 15.
    pub slot_min: usize,
    /// Last discovery slot (zero-based) — 16.
    pub slot_max: usize,
    /// `ColdStartBetaAlpha0 = 0.75`.
    pub beta_alpha0: f64,
    /// `ColdStartBetaBeta0 = 49.25`.
    pub beta_beta0: f64,
    /// Deterministic exploration bonus range in [0, this value).
    pub exploration_bonus: f64,
}

impl Default for ColdStartPolicy {
    fn default() -> Self {
        Self {
            max_creator_followers: 50_000,
            max_product_age_secs: 7_200,
            max_impressions: 200,
            slot_min: 15,
            slot_max: 16,
            beta_alpha0: 0.75,
            beta_beta0: 49.25,
            exploration_bonus: 0.01,
        }
    }
}

/// A novice product, candidate for a discovery slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColdStartCandidate {
    pub product_id: u64,
    pub creator_followers: u64,
    pub product_age_secs: i64,
    pub impressions: u64,
}

impl ColdStartPolicy {
    /// Cold-start eligible? (none of the three conditions may fail)
    #[must_use]
    pub fn is_eligible(&self, c: &ColdStartCandidate) -> bool {
        c.creator_followers < self.max_creator_followers
            && c.product_age_secs <= self.max_product_age_secs
            && c.impressions < self.max_impressions
    }

    /// Mean of the `Beta(alpha0, beta0)` prior — the deterministic 0.015.
    #[must_use]
    pub fn beta_prior_mean(&self) -> f64 {
        self.beta_alpha0 / (self.beta_alpha0 + self.beta_beta0)
    }

    /// Deterministic priority score: `Beta` mean + `SplitMix64` exploration bonus.
    ///
    /// Upstream uses Thompson sampling (`Beta.sample(rng)`);
    /// the Kian version is **deterministic** to keep ranking reproducible — the same
    /// `0.015` mean as the base plus an exploration jitter in [0, 0.01) derived
    /// from the product id so that equivalent candidates do not remain tied.
    #[must_use]
    pub fn priority_score(&self, c: &ColdStartCandidate) -> f64 {
        self.beta_prior_mean() + self.exploration_bonus * splitmix64_unit(c.product_id)
    }
}

/// `SplitMix64` — a deterministic, stateless pseudo-random generator (exploration bonus).
// the >> 11 output has exactly 53 significant bits; converting to f64 is lossless but
// clippy cannot prove it — a targeted allow per the ledger.rs contract.
#[allow(clippy::cast_precision_loss)]
fn splitmix64_unit(seed: u64) -> f64 {
    // >> 11 → a 53-bit integer; divided by 2^53 → in [0, 1)
    ((splitmix64(seed) >> 11) as f64) / ((1u64 << 53) as f64)
}

fn splitmix64(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// One ranked market row — input/output of the cold injection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RankedProduct {
    pub product_id: u64,
    /// The final fusion score (`fuse_score`).
    pub score: f64,
}

/// Deterministic injection of cold candidates into discovery slots `slot_min..=slot_max`.
///
/// * Only eligible candidates, sorted by descending `priority_score`, are
///   considered (up to `slot_max - slot_min + 1` items).
/// * The input list must be score-descending (the caller's contract after sorting).
/// * Output length = input + number of injections — no product is ever dropped.
#[must_use]
pub fn inject_cold_start(
    policy: &ColdStartPolicy,
    ranked: Vec<RankedProduct>,
    candidates: &[ColdStartCandidate],
) -> Vec<RankedProduct> {
    let mut eligible: Vec<(ColdStartCandidate, f64)> = candidates
        .iter()
        .filter(|c| policy.is_eligible(c))
        .map(|c| (*c, policy.priority_score(c)))
        .collect();
    // priority-descending; ties resolve deterministically to the smaller id so sorting stays reproducible.
    eligible.sort_by(|a, b| {
        b.1.total_cmp(&a.1).then_with(|| a.0.product_id.cmp(&b.0.product_id))
    });

    let slot_count = policy.slot_max.saturating_sub(policy.slot_min) + 1;
    let picked: Vec<RankedProduct> = eligible
        .into_iter()
        .take(slot_count)
        .map(|(c, priority)| RankedProduct { product_id: c.product_id, score: priority })
        .collect();

    let mut out = ranked;
    for (offset, injected) in picked.into_iter().enumerate() {
        let idx = policy.slot_min + offset;
        if idx <= out.len() {
            out.insert(idx, injected);
        } else {
            // the list is shorter than the discovery slots — injections go at the end, nothing dropped.
            out.push(injected);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weights() -> ViralEngagementWeights {
        ViralEngagementWeights::default()
    }

    /// The ported weights equal `param.rs` exactly.
    #[test]
    fn weights_match_upstream_param_rs() {
        let w = weights();
        assert_eq!(w.share_via_copy_link, 20.0);
        assert_eq!(w.reply, 5.0);
        assert_eq!(w.bidirectional_reply_boost, 15.0);
        assert_eq!(w.quote, 5.0);
        assert_eq!(w.share_via_dm, 5.0);
        assert_eq!(w.follow_creator, 4.0);
        assert_eq!(w.share, 2.118);
        assert_eq!(w.repost, 1.0);
        assert_eq!(w.favorite, 0.708);
        assert_eq!(w.cont_click_dwell_per_sec, 0.385);
        assert_eq!(w.click, 0.274);
        assert_eq!(w.open_link, 0.2);
        assert_eq!(w.dwell, 0.05);
        assert_eq!(w.not_dwelled, -0.02);
        assert_eq!(w.block_creator, -31.2);
        assert_eq!(w.not_interested, -53.07);
        assert_eq!(w.mute_creator, -58.8);
        assert_eq!(w.report, -234.0);
    }

    /// Copy-link dominance: +20.0, about 28x favorite=0.708.
    #[test]
    fn copy_link_dominance_is_28x_over_favorite() {
        let copy_sig = ProductSignals { share_via_copy_link: 1.0, ..ProductSignals::default() };
        let fav_sig = ProductSignals { favorite: 1.0, ..ProductSignals::default() };

        let copy = score_with_defaults(&copy_sig);
        let fav = score_with_defaults(&fav_sig);
        assert_eq!(copy, 20.0);
        assert_eq!(fav, 0.708);
        let ratio = copy / fav;
        assert!(
            (27.0..29.0).contains(&ratio),
            "copy-link must be ~28x favorite, got {ratio}"
        );
        // bidirectional reply (20.0) matches copy-link and beats every other single signal.
        let mutual = ProductSignals { reply: 1.0, bidirectional_reply: true, ..ProductSignals::default() };
        assert_eq!(score_with_defaults(&mutual), 20.0);
    }

    /// Negative-penalty suppression: one report pushes the product below zero, seating it under
    /// the clean signal-free product — even if it also has likes and copy-links.
    #[test]
    fn negative_penalty_suppresses_reported_product() {
        let reported = ProductSignals {
            share_via_copy_link: 0.4, // +8.0
            favorite: 0.5, // +0.354
            report: 0.05, // -11.7
            ..ProductSignals::default()
        };
        let s_reported = score_with_defaults(&reported);
        assert!(s_reported < 0.0, "report must drag below zero: {s_reported}");

        let clean = ProductSignals { favorite: 0.1, ..ProductSignals::default() }; // +0.0708
        let s_clean = score_with_defaults(&clean);
        assert!(s_clean > 0.0);
        assert!(
            s_clean > s_reported,
            "clean product must outrank reported one: {s_clean} vs {s_reported}"
        );

        // penalty severity order: report(-234) > mute(-58.8) > not_interested(-53.07)
        // > block(-31.2) > not_dwelled(-0.02)
        let r = score_with_defaults(&ProductSignals { report: 1.0, ..ProductSignals::default() });
        let m = score_with_defaults(&ProductSignals { mute_creator: 1.0, ..ProductSignals::default() });
        let ni = score_with_defaults(&ProductSignals { not_interested: 1.0, ..ProductSignals::default() });
        let b = score_with_defaults(&ProductSignals { block_creator: 1.0, ..ProductSignals::default() });
        let nd = score_with_defaults(&ProductSignals { not_dwelled: 1.0, ..ProductSignals::default() });
        assert!(r < m && m < ni && ni < b && b < nd);
    }

    /// Cold eligibility: exact thresholds <50k followers, <=7200s, <200 impressions.
    #[test]
    fn cold_start_eligibility_boundaries() {
        let policy = ColdStartPolicy::default();
        let ok = ColdStartCandidate {
            product_id: 1,
            creator_followers: 49_999,
            product_age_secs: 7_200,
            impressions: 199,
        };
        assert!(policy.is_eligible(&ok));

        let too_famous = ColdStartCandidate { creator_followers: 50_000, ..ok };
        assert!(!policy.is_eligible(&too_famous));
        let too_old = ColdStartCandidate { product_age_secs: 7_201, ..ok };
        assert!(!policy.is_eligible(&too_old));
        let too_seen = ColdStartCandidate { impressions: 200, ..ok };
        assert!(!policy.is_eligible(&too_seen));
    }

    /// The Beta(0.75, 49.25) prior mean is exactly 0.015 and the priority score is deterministic.
    #[test]
    fn cold_start_priority_is_deterministic_and_bounded() {
        let policy = ColdStartPolicy::default();
        assert!((policy.beta_prior_mean() - 0.015).abs() < 1e-12);

        let c = ColdStartCandidate {
            product_id: 42,
            creator_followers: 10,
            product_age_secs: 100,
            impressions: 5,
        };
        let s1 = policy.priority_score(&c);
        let s2 = policy.priority_score(&c);
        assert_eq!(s1, s2, "priority must be deterministic");
        assert!(s1 >= 0.015, "score ≥ Beta mean");
        assert!(
            s1 < 0.015 + 0.01,
            "score < mean + exploration bonus ceiling"
        );

        // two distinct products — the exploration jitter separates them (almost always).
        let mut distinct = 0;
        for id in 1..64u64 {
            let other = ColdStartCandidate { product_id: id, ..c };
            if (policy.priority_score(&other) - s1).abs() > f64::EPSILON {
                distinct += 1;
            }
        }
        assert!(distinct > 32, "exploration bonus must spread scores");
    }

    /// Cold injection: the two top winners (highest deterministic priority) land exactly in slots
    /// 15 and 16 and no ranked item is dropped.
    #[test]
    #[allow(clippy::cast_precision_loss)] // a 30-item score-descending list — f64 precision suffices
    fn cold_start_slot_injection_15_and_16() {
        let policy = ColdStartPolicy::default();
        let ranked: Vec<RankedProduct> = (0u64..30)
            .map(|i| RankedProduct {
                product_id: 1_000 + i,
                score: 100.0 - i as f64,
            })
            .collect();

        let base = ColdStartCandidate {
            product_id: 0,
            creator_followers: 0,
            product_age_secs: 60,
            impressions: 0,
        };
        let candidates = [
            ColdStartCandidate { product_id: 500, ..base },
            ColdStartCandidate { product_id: 700, ..base },
            ColdStartCandidate { product_id: 900, ..base }, // only 2 slots exist
            ColdStartCandidate { product_id: 999, creator_followers: 500_000, ..base }, // famous
        ];

        // expectation = the two eligible candidates with the highest deterministic priority
        // (the same tie-breaking as the injection: score-descending, then smaller id).
        let mut priorities: Vec<(f64, u64)> = [500u64, 700, 900]
            .map(|id| {
                (
                    policy.priority_score(&ColdStartCandidate { product_id: id, ..base }),
                    id,
                )
            })
            .to_vec();
        priorities
            .sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

        let out = inject_cold_start(&policy, ranked, &candidates);
        assert_eq!(out.len(), 32, "30 ranked + 2 injected (only 2 slots)");
        assert_eq!(out[15].product_id, priorities[0].1, "top cold-start at slot 15");
        assert_eq!(out[15].score, priorities[0].0);
        assert_eq!(out[16].product_id, priorities[1].1, "second cold-start at slot 16");
        assert_eq!(out[16].score, priorities[1].0);
        assert!(
            out.iter().all(|p| p.product_id != 999),
            "famous creator must never be injected"
        );
        // the original items' relative order is preserved — only a one-step shift.
        let originals: Vec<u64> = out
            .iter()
            .filter(|p| p.product_id >= 1_000)
            .map(|p| p.product_id)
            .collect();
        let expected: Vec<u64> = (0u64..30).map(|i| 1_000 + i).collect();
        assert_eq!(originals, expected);
    }

    /// list shorter than the discovery slots — injections at the end, nothing at risk.
    #[test]
    fn injection_handles_short_lists() {
        let policy = ColdStartPolicy::default();
        let ranked = vec![RankedProduct { product_id: 1, score: 5.0 }];
        let candidates = [ColdStartCandidate {
            product_id: 2,
            creator_followers: 0,
            product_age_secs: 60,
            impressions: 0,
        }];
        let out = inject_cold_start(&policy, ranked, &candidates);
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].product_id, 2, "falls to the end when list < slot");
    }

    /// The exploration jitter must be non-zero and stay within range — if `SplitMix64`
    /// broke, all candidates would tie and exploration would die.
    #[test]
    fn exploration_bonus_is_real_and_bounded() {
        let policy = ColdStartPolicy::default();
        let c = ColdStartCandidate {
            product_id: 7,
            creator_followers: 0,
            product_age_secs: 60,
            impressions: 0,
        };
        let score = policy.priority_score(&c);
        let bonus = score - policy.beta_prior_mean();
        assert!(bonus > 0.0, "bonus must be strictly positive for a nonzero id");
        assert!(bonus < policy.exploration_bonus);
    }
}
