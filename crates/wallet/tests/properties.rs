//! Pillar 1 — property-based tests for the sovereign wallet (proptest).
//!
//! Deterministic properties (every case checked over randomly generated,
//! reproducible inputs):
//!
//! 1. **Day-1 4-Stream revenue split invariant** — for every gross amount in
//!    `1..=1_000_000_000_000` micro-units, `creator + treasury + referrer ==
//!    gross` with **zero rounding dust** (both the referred `10%/80%+10%` and
//!    the unreferred `20%/80%` routes), and the leg sum is exactly zero.
//! 2. **Instant boosts route 100% to the MAIRON Sovereign Treasury** —
//!    `0 bps` to the creator, `10_000 bps` to the treasury, zero-sum legs.
//! 3. **CCXT `Precise` i128 fixed-point tick rounding** — truncating to a
//!    tick size yields exact multiples of the tick (zero remainder in the
//!    co-scaled coefficient), never exceeds the input, and is idempotent.

use kian_wallet::base_treasury::{
    compute_day_one_atomic_split, compute_instant_boost_split, BoostTier, DayOneRevenueCharter,
    SettlementAsset,
};
use kian_wallet::ccxt_core::{Precise, RoundingMode, to_precision_tick_size};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Property 1 — the day-one split conserves every micro-unit: creator +
    /// treasury + referrer == gross, the signed leg sum is exactly 0, and the
    /// `Precise` zero-sum recheck holds for both referred and unreferred sales.
    #[test]
    fn day_one_split_conserves_every_micro(
        gross in 1i128..=1_000_000_000_000,
        referred in proptest::bool::ANY,
        affiliate_bps in 1000i64..=5000,
    ) {
        let charter = DayOneRevenueCharter::DAY_ONE;
        let split = compute_day_one_atomic_split(
            &charter,
            SettlementAsset::BaseNativeUsdc,
            gross,
            "citizen:prop-buyer",
            "citizen:prop-creator",
            referred.then_some("citizen:prop-referrer"),
            if referred { affiliate_bps } else { charter.affiliate_bps },
        )
        .expect("a positive gross must always split");

        // Conservation with zero rounding dust.
        prop_assert_eq!(
            split.creator_net_micros + split.kia_treasury_micros + split.referrer_micros,
            split.gross_amount_micros,
            "creator + treasury + referrer must equal the gross exactly"
        );
        // Signed double-entry legs cancel to exactly zero.
        prop_assert_eq!(
            split.legs.iter().map(|l| l.amount_micros).sum::<i128>(),
            0,
            "SUM(leg.amount) must be exactly 0"
        );
        prop_assert!(split.verify_zero_sum_precise().expect("no overflow"));

        if referred {
            // The engine rounds each leg half-away-from-zero at 0 decimals
            // (`Precise::div(denom, 0)`) — mirror that exact rounding.
            let expected_referrer = (gross * i128::from(affiliate_bps) + 5_000) / 10_000;
            prop_assert_eq!(split.referrer_micros, expected_referrer, "referred affiliate leg");
            prop_assert_eq!(
                split.kia_treasury_micros,
                (gross * 1_000 + 5_000) / 10_000,
                "referred treasury leg = 10% charter tax"
            );
        } else {
            prop_assert_eq!(split.referrer_micros, 0, "unreferred sale has no referrer leg");
            let tax = (gross * 1_000 + 5_000) / 10_000;
            let default_affiliate = (gross * 1_000 + 5_000) / 10_000;
            prop_assert_eq!(
                split.kia_treasury_micros,
                tax + default_affiliate,
                "unreferred platform total = 20% (tax + default affiliate routing)"
            );
        }
        prop_assert!(split.creator_net_micros > 0, "the creator leg is the positive remainder");
    }

    /// Property 2 — instant boosts route 100% to the treasury and 0% to the
    /// creator across all three tiers and both settlement rails.
    #[test]
    fn instant_boosts_route_100_percent_to_treasury(
        tier in prop_oneof![
            Just(BoostTier::FeaturedSlot24h),
            Just(BoostTier::MaironFastTrackAudit),
            Just(BoostTier::ViralSwarmAmplifier),
        ],
        stars_rail in proptest::bool::ANY,
    ) {
        let asset = if stars_rail {
            SettlementAsset::TelegramStarsXtr
        } else {
            SettlementAsset::BaseNativeUsdc
        };
        let split = compute_instant_boost_split(asset, tier, "citizen:prop-buyer")
            .expect("a canonical boost price must always split");
        prop_assert_eq!(split.kia_treasury_micros, split.gross_amount_micros,
            "100% of an instant boost reaches the MAIRON Sovereign Treasury");
        prop_assert_eq!(split.creator_net_micros, 0, "0% of an instant boost reaches the creator");
        prop_assert_eq!(split.referrer_micros, 0);
        prop_assert_eq!(split.legs.iter().map(|l| l.amount_micros).sum::<i128>(), 0);
        prop_assert!(split.verify_zero_sum_precise().expect("no overflow"));
    }

    /// Property 3 — CCXT `Precise` i128 tick rounding: truncation to a tick
    /// size produces exact tick multiples (zero remainder), never exceeds the
    /// input, and is idempotent.
    #[test]
    fn precise_tick_rounding_is_exact_and_idempotent(
        units in 1i128..=1_000_000_000_000,
        frac_nanos in 0i128..=999_999_999,
        tick_choice in 0usize..=4,
    ) {
        let tick = ["0.01", "0.0001", "1", "0.5", "0.001"][tick_choice];
        let x = format!("{units}.{frac_nanos:09}");
        let rounded = to_precision_tick_size(&x, tick, RoundingMode::Truncate)
            .expect("valid decimal inputs must always round");

        // Exact multiple: aligned coefficients leave zero remainder.
        let r = Precise::parse(&rounded).expect("output must re-parse");
        let t = Precise::parse(tick).expect("tick must parse");
        let (va, vb, _) = r.align(&t).expect("alignment must not overflow");
        prop_assert_eq!(va % vb, 0, "rounded value must be an exact multiple of the tick");

        // Never exceeds the input (truncation), verified with the float-free engine.
        let original = Precise::parse(&x).expect("input must parse");
        prop_assert!(r.lt(&original).expect("no overflow") || r.eq_val(&original).expect("no overflow"),
            "truncation must never exceed the input");

        // Idempotence: rounding an already-rounded value changes nothing.
        let again = to_precision_tick_size(&rounded, tick, RoundingMode::Truncate)
            .expect("re-round must not overflow");
        prop_assert_eq!(again, rounded, "tick rounding must be idempotent");
    }
}
