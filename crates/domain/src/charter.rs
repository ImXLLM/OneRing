//! موتور منشور — منشور پولی کشور دیجیتال (سیاست خالص، بدون IO).
//!
//! آینه `docs/economy-charter.md` امپراتوری v2 (۱۰ ماده) به‌صورت ماژول
//! سیاست ماشین‌پذیر. اصل تقدس: پول حفظ می‌شود — خارج از حساب‌های
//! mint/burn هرگز ساخته یا نابود نمی‌شود.

use crate::errors::{DomainError, DomainErrorCode, DomainResult};
use crate::money::Din;

/// نرخ پیش‌فرض پورسانت معرف (۱۰٪ = 1000 bps).
pub const DEFAULT_AFFILIATE_BPS: u16 = 1_000;

/// سقف مجاز پورسانت معرف که فروشنده می‌تواند روی محصول بگذارد (۵۰٪ = 5000 bps).
pub const MAX_AFFILIATE_BPS: u16 = 5_000;

/// ثابت‌های منشور — معادل `CHARTER` در TS.
pub struct Charter;

impl Charter {
    /// ماده ۱ — هویت ارز
    pub const CURRENCY: &'static str = "DNX";
    /// نماد پول
    pub const SYMBOL: &'static str = "\u{110}";
    /// ماده ۲ — مالیات پلتفرم روی تسویه بازار (۱۰٪ = 1000 bps)
    pub const TAX_BPS: i64 = 1_000;
    /// ماده ۷ — آستانه تأیید انسانی: عملیات ≥ ۵۰,۰۰۰ Đ نیاز به تأیید مالک دارد
    pub const HITL_THRESHOLD_DIN: Din = Din(50_000);
    /// ماده ۳ — سقف ضرب سالانه
    pub const YEARLY_MINT_CAP_DIN: Din = Din(1_000_000);
    /// عرضه اندازه‌گیری‌شده‌ی genesis (ممیزی مهر ۱۴۰۵: Σ = 187,672 Đ، گپ −۶۳ Đ توضیح‌داده‌شده)
    pub const GENESIS_SUPPLY_DIN: Din = Din(187_672);
}

/// یک ماده از منشور — به‌صورت داده (برای UI و داشبورد).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharterArticle {
    /// شماره ماده (۱ تا ۱۰)
    pub article: u8,
    /// عنوان ماده
    pub title: &'static str,
    /// متن ماده
    pub value: &'static str,
}

/// ۱۰ ماده منشور — یک‌به‌یک با نسخه TS (فارسی حفظ شده).
pub const CHARTER_ARTICLES: [CharterArticle; 10] = [
    CharterArticle {
        article: 1,
        title: "واحد پول",
        value: "دینار دیجیتال (Đ / DNX) — عدد صحیح، بدون اعشار",
    },
    CharterArticle {
        article: 2,
        title: "مالیات بازار",
        value: "۱۰٪ هر تسویه اسکرو به خزانه می‌رود",
    },
    CharterArticle {
        article: 3, title: "سقف ضرب", value: "حداکثر ۱٬۰۰۰٬۰۰۰ Đ ضرب در سال"
    },
    CharterArticle {
        article: 4,
        title: "حفاظت عرضه",
        value: "پول فقط از حساب mint صادر و فقط به حساب burn می‌رود",
    },
    CharterArticle {
        article: 5,
        title: "پایایی دفتر",
        value: "هر تراکنش دوطرفه است و مجموع تراز همه حساب‌ها همیشه صفر است",
    },
    CharterArticle {
        article: 6, title: "شفافیت", value: "هر تراکنش شناسه یکتا و memo دارد"
    },
    CharterArticle {
        article: 7,
        title: "آستانه HITL",
        value: "عملیات ≥ ۵۰٬۰۰۰ Đ نیازمند تایید انسانی است",
    },
    CharterArticle {
        article: 8,
        title: "اسکرو اجباری",
        value: "خرید از مارکت فقط از مسیر اسکرو انجام می‌شود",
    },
    CharterArticle {
        article: 9,
        title: "بازگشت وجه",
        value: "اختلاف اسکرو به نفع خریدار بازگشت می‌شود مگر داوری خلاف آن را تعیین کند",
    },
    CharterArticle {
        article: 10,
        title: "تعدادیل میراث",
        value: "گپ تاریخی −۶۳ Đ با یک ردیف تعدیل شفاف ثبت شده است",
    },
];

/// مالیات ماده ۲ — `taxOf`.
///
/// # Errors
/// خطاهای لایه پول را عبور می‌دهد.
pub fn tax_of(amount: Din) -> DomainResult<Din> {
    amount.mul_bps(Charter::TAX_BPS)
}

/// تفکیک تسویه: سهم فروشنده + مالیات خزانه.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellerSplit {
    /// سهم خالص فروشنده
    pub seller: Din,
    /// مالیات خزانه
    pub tax: Din,
}

/// تفکیک سه‌گانهٔ مبلغ تسویه: `(فروشنده، مالیات خزانه، پورسانت معرف)`.
///
/// * مالیات پلتفرم همیشه طبق مادهٔ ۲ منشور (`Charter::TAX_BPS = 1000`، ۱۰٪) محاسبه می‌شود.
/// * سهم معرف بر اساس `affiliate_bps` از کل `amount` (با ریاضی صحیح `u128` ضدسرریز) است.
/// * هر باقیماندهٔ گردکردن به پایین به نفع فروشنده حفظ می‌شود تا همواره
///   `seller + tax + affiliate == amount` برقرار باشد.
///
/// # Errors
/// `BPS_OUT_OF_RANGE` اگر `affiliate_bps > MAX_AFFILIATE_BPS` باشد.
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

/// تفکیک مبلغ تسویه به سهم فروشنده + مالیات خزانه
/// (seller + tax = amount، همیشه).
///
/// # Errors
/// خطاهای لایه پول را عبور می‌دهد.
pub fn split_seller_tax(amount: Din) -> DomainResult<SellerSplit> {
    let (seller, tax, _) = split_seller_tax_affiliate(amount, 0)?;
    Ok(SellerSplit { seller, tax })
}

/// ماده ۷ — دروازه HITL: آیا این مبلغ تأیید انسانی می‌خواهد؟
#[must_use]
pub fn requires_hitl(amount: Din) -> bool {
    amount >= Charter::HITL_THRESHOLD_DIN
}

/// ماده ۳ — بررسی سقف ضرب سالانه نسبت به مجموع جاری.
///
/// # Errors
/// `MINT_CAP_EXCEEDED` اگر مجموع از سقف بگذرد؛ خطاهای جمع را عبور می‌دهد.
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
    fn fixes_the_genesis_supply_measured_in_the_mehr_1405_audit() {
        assert_eq!(Charter::GENESIS_SUPPLY_DIN.as_i64(), 187_672);
    }

    #[test]
    fn charges_exactly_10_percent_tax_article_2() {
        assert_eq!(tax_of(d(100)).unwrap().as_i64(), 10);
        assert_eq!(tax_of(d(1_000)).unwrap().as_i64(), 100);
        assert_eq!(tax_of(d(187_672)).unwrap().as_i64(), 18_767); // floor
    }

    #[test]
    fn floors_tax_to_keep_money_integer() {
        // ۱۰٪ از ۵ Đ = ۰
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
        let r = assert_mint_within_cap(d(999_000), d(2_000));
        assert_eq!(r.unwrap_err().code, DomainErrorCode::MintCapExceeded);
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
        // عدد رند با پیش‌فرض ۱۰٪ معرف + ۱۰٪ مالیات → ۸۰٪ فروشنده
        let (seller, tax, aff) = split_seller_tax_affiliate(d(1_000), DEFAULT_AFFILIATE_BPS).unwrap();
        assert_eq!((seller.as_i64(), tax.as_i64(), aff.as_i64()), (800, 100, 100));

        // عدد اول ۱۰۱ با ۱۰٪ → مالیات ۱۰، معرف ۱۰، فروشنده ۸۱ (باقیمانده به نفع فروشنده)
        let (seller, tax, aff) = split_seller_tax_affiliate(d(101), DEFAULT_AFFILIATE_BPS).unwrap();
        assert_eq!((seller.as_i64(), tax.as_i64(), aff.as_i64()), (81, 10, 10));
        assert_eq!(seller.as_i64() + tax.as_i64() + aff.as_i64(), 101);

        // عدد فرد ۹۹۹ با سقف ۵۰٪ → مالیات ۹۹، معرف ۴۹۹، فروشنده ۴۰۱
        let (seller, tax, aff) = split_seller_tax_affiliate(d(999), MAX_AFFILIATE_BPS).unwrap();
        assert_eq!((seller.as_i64(), tax.as_i64(), aff.as_i64()), (401, 99, 499));
        assert_eq!(seller.as_i64() + tax.as_i64() + aff.as_i64(), 999);

        // رنج کامل ۰..=۲۵۰ روی نرخ‌های مختلف
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

        // ضدسرریز u128 روی MAX_DIN با سقف ۵۰۰۰ bps (در i64 سرریز می‌کرد)
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
