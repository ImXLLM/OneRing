//! Charter engine — the monetary charter of the digital nation (pure policy, no IO).
//!
//! A machine-readable policy module mirroring the `docs/economy-charter.md` v2 empire
//! charter (10 articles). Sanctity principle: money is conserved — outside the
//! mint/burn accounts it is never created or destroyed.

use crate::errors::{DomainError, DomainErrorCode, DomainResult};
use crate::money::Din;

/// Default affiliate commission rate (10% = 1000 bps).
pub const DEFAULT_AFFILIATE_BPS: u16 = 1_000;

/// Max affiliate commission a seller may set on a product (50% = 5000 bps).
pub const MAX_AFFILIATE_BPS: u16 = 5_000;

/// Charter constants — equivalent to `CHARTER` in TS.
pub struct Charter;

impl Charter {
    /// Article 1 — currency identity
    pub const CURRENCY: &'static str = "DNX";
    /// Currency symbol
    pub const SYMBOL: &'static str = "\u{110}";
    /// Article 2 — platform tax on marketplace settlement (10% = 1000 bps)
    pub const TAX_BPS: i64 = 1_000;
    /// Article 7 — human-approval threshold: operations ≥ 50,000 Đ require owner approval
    pub const HITL_THRESHOLD_DIN: Din = Din(50_000);
    /// Article 3 — annual mint cap: 500,000 KIAN (500 billion micro) — after
    /// the dropagent legacy community migration (the sanitized 58,700.2 KIAN genesis in
    /// 2026) the previous 60,000 KIAN cap was nearly full and the dev faucet and real
    /// deposits had no room; the new cap gives ample headroom for growth, and minting
    /// is controlled by verified deposits and this same cap.
    pub const YEARLY_MINT_CAP_DIN: Din = Din(500_000_000_000);
    /// Genesis supply of the legacy migration — dropagent 30-day base audit:
    /// 466 citizens, 435 accounts with positive balance, Σ = 58,700,200,000 micro
    /// (58,700.2 KIAN) — sanitized: ordinary users ≤ 1,000 KIAN,
    /// `@ImXsi` 1,000 KIAN (fix of double deposit #3), `@ImXforevr` 5,000 KIAN.
    pub const GENESIS_SUPPLY_DIN: Din = Din(58_700_200_000);
}

/// One charter article — as data (for UI and dashboard).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharterArticle {
    /// Article number (1 to 10)
    pub article: u8,
    /// Article title
    pub title: &'static str,
    /// Article body
    pub value: &'static str,
}

/// The 10 charter articles — one-to-one with the TS version (Persian preserved).
pub const CHARTER_ARTICLES: [CharterArticle; 10] = [
    CharterArticle {
        article: 1,
        title: "\u{648}\u{627}\u{62d}\u{62f} \u{67e}\u{648}\u{644}",
        value: "\u{62f}\u{6cc}\u{646}\u{627}\u{631} \u{62f}\u{6cc}\u{62c}\u{6cc}\u{62a}\u{627}\u{644} (Đ / DNX) — \u{639}\u{62f}\u{62f} \u{635}\u{62d}\u{6cc}\u{62d}\u{60c} \u{628}\u{62f}\u{648}\u{646} \u{627}\u{639}\u{634}\u{627}\u{631}",
    },
    CharterArticle {
        article: 2,
        title: "\u{645}\u{627}\u{644}\u{6cc}\u{627}\u{62a} \u{628}\u{627}\u{632}\u{627}\u{631}",
        value: "\u{6f1}\u{6f0}\u{66a} \u{647}\u{631} \u{62a}\u{633}\u{648}\u{6cc}\u{647} \u{627}\u{633}\u{6a9}\u{631}\u{648} \u{628}\u{647} \u{62e}\u{632}\u{627}\u{646}\u{647} \u{645}\u{6cc}‌\u{631}\u{648}\u{62f}",
    },
    CharterArticle {
        article: 3, title: "\u{633}\u{642}\u{641} \u{636}\u{631}\u{628}", value: "\u{62d}\u{62f}\u{627}\u{6a9}\u{62b}\u{631} \u{6f5}\u{6f0}\u{6f0}\u{66c}\u{6f0}\u{6f0}\u{6f0} KIAN \u{636}\u{631}\u{628} \u{62f}\u{631} \u{633}\u{627}\u{644}"
    },
    CharterArticle {
        article: 4,
        title: "\u{62d}\u{641}\u{627}\u{638}\u{62a} \u{639}\u{631}\u{636}\u{647}",
        value: "\u{67e}\u{648}\u{644} \u{641}\u{642}\u{637} \u{627}\u{632} \u{62d}\u{633}\u{627}\u{628} mint \u{635}\u{627}\u{62f}\u{631} \u{648} \u{641}\u{642}\u{637} \u{628}\u{647} \u{62d}\u{633}\u{627}\u{628} burn \u{645}\u{6cc}‌\u{631}\u{648}\u{62f}",
    },
    CharterArticle {
        article: 5,
        title: "\u{67e}\u{627}\u{6cc}\u{627}\u{6cc}\u{6cc} \u{62f}\u{641}\u{62a}\u{631}",
        value: "\u{647}\u{631} \u{62a}\u{631}\u{627}\u{6a9}\u{646}\u{634} \u{62f}\u{648}\u{637}\u{631}\u{641}\u{647} \u{627}\u{633}\u{62a} \u{648} \u{645}\u{62c}\u{645}\u{648}\u{639} \u{62a}\u{631}\u{627}\u{632} \u{647}\u{645}\u{647} \u{62d}\u{633}\u{627}\u{628}‌\u{647}\u{627} \u{647}\u{645}\u{6cc}\u{634}\u{647} \u{635}\u{641}\u{631} \u{627}\u{633}\u{62a}",
    },
    CharterArticle {
        article: 6, title: "\u{634}\u{641}\u{627}\u{641}\u{6cc}\u{62a}", value: "\u{647}\u{631} \u{62a}\u{631}\u{627}\u{6a9}\u{646}\u{634} \u{634}\u{646}\u{627}\u{633}\u{647} \u{6cc}\u{6a9}\u{62a}\u{627} \u{648} memo \u{62f}\u{627}\u{631}\u{62f}"
    },
    CharterArticle {
        article: 7,
        title: "\u{622}\u{633}\u{62a}\u{627}\u{646}\u{647} HITL",
        value: "\u{639}\u{645}\u{644}\u{6cc}\u{627}\u{62a} ≥ \u{6f5}\u{6f0}\u{66c}\u{6f0}\u{6f0}\u{6f0} Đ \u{646}\u{6cc}\u{627}\u{632}\u{645}\u{646}\u{62f} \u{62a}\u{627}\u{6cc}\u{6cc}\u{62f} \u{627}\u{646}\u{633}\u{627}\u{646}\u{6cc} \u{627}\u{633}\u{62a}",
    },
    CharterArticle {
        article: 8,
        title: "\u{627}\u{633}\u{6a9}\u{631}\u{648} \u{627}\u{62c}\u{628}\u{627}\u{631}\u{6cc}",
        value: "\u{62e}\u{631}\u{6cc}\u{62f} \u{627}\u{632} \u{645}\u{627}\u{631}\u{6a9}\u{62a} \u{641}\u{642}\u{637} \u{627}\u{632} \u{645}\u{633}\u{6cc}\u{631} \u{627}\u{633}\u{6a9}\u{631}\u{648} \u{627}\u{646}\u{62c}\u{627}\u{645} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f}",
    },
    CharterArticle {
        article: 9,
        title: "\u{628}\u{627}\u{632}\u{6af}\u{634}\u{62a} \u{648}\u{62c}\u{647}",
        value: "\u{627}\u{62e}\u{62a}\u{644}\u{627}\u{641} \u{627}\u{633}\u{6a9}\u{631}\u{648} \u{628}\u{647} \u{646}\u{641}\u{639} \u{62e}\u{631}\u{6cc}\u{62f}\u{627}\u{631} \u{628}\u{627}\u{632}\u{6af}\u{634}\u{62a} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f} \u{645}\u{6af}\u{631} \u{62f}\u{627}\u{648}\u{631}\u{6cc} \u{62e}\u{644}\u{627}\u{641} \u{622}\u{646} \u{631}\u{627} \u{62a}\u{639}\u{6cc}\u{6cc}\u{646} \u{6a9}\u{646}\u{62f}",
    },
    CharterArticle {
        article: 10,
        title: "\u{62a}\u{639}\u{62f}\u{627}\u{62f}\u{6cc}\u{644} \u{645}\u{6cc}\u{631}\u{627}\u{62b}",
        value: "\u{6af}\u{67e} \u{62a}\u{627}\u{631}\u{6cc}\u{62e}\u{6cc} −\u{6f6}\u{6f3} Đ \u{628}\u{627} \u{6cc}\u{6a9} \u{631}\u{62f}\u{6cc}\u{641} \u{62a}\u{639}\u{62f}\u{6cc}\u{644} \u{634}\u{641}\u{627}\u{641} \u{62b}\u{628}\u{62a} \u{634}\u{62f}\u{647} \u{627}\u{633}\u{62a}",
    },
];

/// Article 2 tax — `taxOf`.
///
/// # Errors
/// Passes through money-layer errors.
pub fn tax_of(amount: Din) -> DomainResult<Din> {
    amount.mul_bps(Charter::TAX_BPS)
}

/// Settlement split: seller share + treasury tax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellerSplit {
    /// Seller net share
    pub seller: Din,
    /// Treasury tax
    pub tax: Din,
}

/// Three-way split of the settlement amount: `(seller, treasury tax, affiliate commission)`.
///
/// * Platform tax is always computed per charter Article 2 (`Charter::TAX_BPS = 1000`, 10%).
/// * The affiliate share is `affiliate_bps` of the whole `amount` (overflow-safe `u128` integer math).
/// * Any floor-rounding remainder is kept in the seller's favor so that
///   `seller + tax + affiliate == amount` always holds.
///
/// # Errors
/// `BPS_OUT_OF_RANGE` if `affiliate_bps > MAX_AFFILIATE_BPS`.
pub fn split_seller_tax_affiliate(
    amount: Din,
    affiliate_bps: u16,
) -> DomainResult<(Din, Din, Din)> {
    if affiliate_bps > MAX_AFFILIATE_BPS {
        return Err(DomainError::new(
            DomainErrorCode::BpsOutOfRange,
            format!("affiliate_bps must be in [0, {MAX_AFFILIATE_BPS}], got {affiliate_bps}"),
        ));
    }
    let amt = u128::try_from(amount.as_i64()).map_err(|_| {
        DomainError::new(
            DomainErrorCode::NegativeAmount,
            format!("Money cannot be negative, got {}", amount.as_i64()),
        )
    })?;
    let tax_bps = u128::try_from(Charter::TAX_BPS).unwrap_or(0);
    let tax_raw = i64::try_from((amt * tax_bps) / 10_000).map_err(|_| {
        DomainError::new(DomainErrorCode::AmountOutOfRange, "Tax exceeds safe integer range")
    })?;
    let tax = Din::new(tax_raw)?;

    let affiliate = if affiliate_bps == 0 {
        Din::ZERO
    } else {
        let aff_raw = i64::try_from((amt * u128::from(affiliate_bps)) / 10_000).map_err(|_| {
            DomainError::new(
                DomainErrorCode::AmountOutOfRange,
                "Affiliate share exceeds safe integer range",
            )
        })?;
        Din::new(aff_raw)?
    };

    let seller = amount.checked_sub(tax)?.checked_sub(affiliate)?;
    Ok((seller, tax, affiliate))
}

/// Split the settlement amount into seller share + treasury tax
/// (seller + tax = amount, always).
///
/// # Errors
/// Passes through money-layer errors.
pub fn split_seller_tax(amount: Din) -> DomainResult<SellerSplit> {
    let (seller, tax, _) = split_seller_tax_affiliate(amount, 0)?;
    Ok(SellerSplit { seller, tax })
}

/// Article 7 — the HITL gate: does this amount require human approval?
#[must_use]
pub fn requires_hitl(amount: Din) -> bool {
    amount >= Charter::HITL_THRESHOLD_DIN
}

/// Article 3 — check the annual mint cap against the current total.
///
/// # Errors
/// `MINT_CAP_EXCEEDED` if the total exceeds the cap; passes through sum errors.
pub fn assert_mint_within_cap(minted_so_far: Din, amount: Din) -> DomainResult<()> {
    let next = minted_so_far.checked_add(amount)?;
    if next > Charter::YEARLY_MINT_CAP_DIN {
        return Err(DomainError::new(
            DomainErrorCode::MintCapExceeded,
            format!(
                "Yearly mint cap exceeded: {} + {} > {}",
                minted_so_far.as_i64(),
                amount.as_i64(),
                Charter::YEARLY_MINT_CAP_DIN.as_i64()
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn d(n: i64) -> Din {
        Din(n)
    }

    #[test]
    fn fixes_the_sanitized_genesis_supply_of_the_legacy_migration() {
        // dropagent migration audit: the sanitized mint Σ of 466 citizens
        assert_eq!(Charter::GENESIS_SUPPLY_DIN.as_i64(), 58_700_200_000);
        // genesis must keep ample distance below the guaranteed Article 3 cap
        assert!(Charter::GENESIS_SUPPLY_DIN.as_i64() * 4 < Charter::YEARLY_MINT_CAP_DIN.as_i64());
    }

    #[test]
    fn charges_exactly_10_percent_tax_article_2() {
        assert_eq!(tax_of(d(100)).unwrap().as_i64(), 10);
        assert_eq!(tax_of(d(1_000)).unwrap().as_i64(), 100);
        assert_eq!(tax_of(d(187_672)).unwrap().as_i64(), 18_767); // floor
    }

    #[test]
    fn floors_tax_to_keep_money_integer() {
        // 10% of 5 Đ = 0
        let split = split_seller_tax(d(5)).unwrap();
        assert_eq!(split.tax.as_i64(), 0);
        assert_eq!(split.seller.as_i64(), 5);
    }

    #[test]
    fn splits_settlements_so_seller_plus_tax_equals_amount_always() {
        for n in 0..=250 {
            let split = split_seller_tax(d(n)).unwrap();
            assert_eq!(split.seller.as_i64() + split.tax.as_i64(), n, "split broke at {n}");
            assert!(split.seller >= Din::ZERO);
            assert!(split.tax >= Din::ZERO);
        }
    }

    #[test]
    fn opens_the_hitl_gate_at_exactly_50_000_article_7() {
        assert!(!requires_hitl(d(49_999)));
        assert!(requires_hitl(d(50_000)));
        assert!(requires_hitl(d(187_672)));
    }

    #[test]
    fn enforces_the_yearly_mint_cap_article_3() {
        assert!(assert_mint_within_cap(d(500_000), d(500_000)).is_ok());
        let r = assert_mint_within_cap(d(499_999_000_000), d(2_000_000));
        assert_eq!(r.unwrap_err().code, DomainErrorCode::MintCapExceeded);
        // migrated genesis + one full dev faucet must stay under the cap
        assert!(assert_mint_within_cap(d(58_700_200_000), d(5_000_000_000)).is_ok());
    }

    #[test]
    fn small_mints_pass_and_articles_ship_as_data() {
        assert!(assert_mint_within_cap(d(0), d(1_000)).is_ok());
        assert_eq!(CHARTER_ARTICLES.len(), 10);
        assert_eq!(CHARTER_ARTICLES[0].article, 1);
        assert_eq!(CHARTER_ARTICLES[9].article, 10);
        assert_eq!(Charter::CURRENCY, "DNX");
        assert_eq!(Charter::SYMBOL, "\u{110}");
        assert_eq!(Charter::TAX_BPS, 1_000);
        assert_eq!(DEFAULT_AFFILIATE_BPS, 1_000);
        assert_eq!(MAX_AFFILIATE_BPS, 5_000);
    }

    #[test]
    fn splits_three_way_affiliate_with_exact_conservation() {
        // Round number with default 10% affiliate + 10% tax → 80% seller
        let (seller, tax, aff) = split_seller_tax_affiliate(d(1_000), DEFAULT_AFFILIATE_BPS).unwrap();
        assert_eq!((seller.as_i64(), tax.as_i64(), aff.as_i64()), (800, 100, 100));

        // Prime 101 at 10% → tax 10, affiliate 10, seller 81 (remainder favors the seller)
        let (seller, tax, aff) = split_seller_tax_affiliate(d(101), DEFAULT_AFFILIATE_BPS).unwrap();
        assert_eq!((seller.as_i64(), tax.as_i64(), aff.as_i64()), (81, 10, 10));
        assert_eq!(seller.as_i64() + tax.as_i64() + aff.as_i64(), 101);

        // Odd 999 at the 50% cap → tax 99, affiliate 499, seller 401
        let (seller, tax, aff) = split_seller_tax_affiliate(d(999), MAX_AFFILIATE_BPS).unwrap();
        assert_eq!((seller.as_i64(), tax.as_i64(), aff.as_i64()), (401, 99, 499));
        assert_eq!(seller.as_i64() + tax.as_i64() + aff.as_i64(), 999);

        // Full 0..=250 range across different rates
        for bps in [0u16, 1, 333, 1_000, 2_500, MAX_AFFILIATE_BPS] {
            for n in 0..=250 {
                let (s, t, a) = split_seller_tax_affiliate(d(n), bps).unwrap();
                assert_eq!(
                    s.as_i64() + t.as_i64() + a.as_i64(),
                    n,
                    "3-way conservation broke at n={n}, bps={bps}"
                );
            }
        }

        // u128 overflow safety on MAX_DIN with the 5000 bps cap (i64 would overflow)
        let max = Din::new(crate::money::MAX_DIN).unwrap();
        let (s, t, a) = split_seller_tax_affiliate(max, MAX_AFFILIATE_BPS).unwrap();
        assert_eq!(s.as_i64() + t.as_i64() + a.as_i64(), crate::money::MAX_DIN);
    }

    #[test]
    fn rejects_affiliate_bps_above_max() {
        let err = split_seller_tax_affiliate(d(1_000), MAX_AFFILIATE_BPS + 1).unwrap_err();
        assert_eq!(err.code, DomainErrorCode::BpsOutOfRange);
        let err2 = split_seller_tax_affiliate(d(1_000), 10_000).unwrap_err();
        assert_eq!(err2.code, DomainErrorCode::BpsOutOfRange);
    }
}
