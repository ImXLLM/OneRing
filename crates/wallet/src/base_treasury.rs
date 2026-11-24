//! خزانهٔ حاکمِ `Base` — جنگِ اول به ازِ صلحِ آخر (Wave 7 — Step 4/4).
//!
//! چهار جریانِ درآمدِ روزِ اول روی Ethereum **Base Mainnet** (`Chain ID: 8453`):
//!  * جریان ۱ — مالیاتِ منشورِ ۱۰٪ (`1000 bps`) روی هر فروشِ بازارخانه؛
//!  * جریان ۲ — تقسیمِ وابستهٔ برنامه‌پذیرِ ۱۰٪ تا ۵۰٪ (`1000..=5000 bps`) —
//!    قانونِ «فروشِ مستقیمِ بی‌معرف»: وقتی معرف `None` است، پایشِ پیش‌فرضِ
//!    ۱۰٪ مستقیم به خزانهٔ حاکم می‌رود → سهمِ کلِ پلتفرم `2000 bps` (۲۰٪)
//!    و سهمِ سازنده `8000 bps` (۸۰٪)؛
//!  * جریان ۳ — بوست‌های لحظه‌ای با مارژینِ ۱۰۰٪ (`FeaturedSlot24h`,
//!    `MaironFastTrackAudit`, `ViralSwarmAmplifier`) — همه به خزانهٔ حاکم؛
//!  * جریان ۴ — اسپردِ تسویهٔ لحظه‌ای و برداشتِ بین‌زنجیره‌ای `1.5%`
//!    (`150 bps`).
//!
//! همهٔ محاسبات روی [`Precise`] (موتورِ دقتِ Wave 6) با مبلغِ مقیاسِ
//! میکرو (`10^-6` واحدِ نمایش) انجام می‌شود — صفر float روی پول، صفر
//! اتلافِ گرد کردن: جمعِ پاهای دفتری همیشه دقیقاً برابرِ مبلغِ ناخالص است.

use std::fmt;

use crate::ccxt_core::Precise;

// ── شبکهٔ Base ───────────────────────────────────────────────────────────────

/// شناسهٔ زنجیرهٔ Base Mainnet — قراردادِ سختِ لایهٔ وایر.
pub const BASE_MAINNET_CHAIN_ID: u64 = 8453;

/// RPC عمومیِ Base Mainnet.
pub const BASE_MAINNET_RPC_URL: &str = "https://mainnet.base.org";

/// مرورگرِ زنجیرهٔ Base — dossier تأییدِ توکن اینجا زندگی می‌کند.
pub const BASE_BLOCK_EXPLORER: &str = "https://basescan.org";

/// قراردادِ کانونیکِ USDC بومیِ Base (EIP-55، منبعِ رسمیِ Circle/Base).
pub const BASE_USDC_CONTRACT: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";

/// عرضهٔ ثابتِ `$0xKIA` روی Base — یک میلیارد توکن، ۱۸ اعشار، هرگز صِبر نمی‌شود.
pub const ZERO_X_KIA_TOTAL_SUPPLY_BASE_UNITS: i128 = 1_000_000_000i128 * 10i128.pow(18);

// ── دارایی‌های تسویه ─────────────────────────────────────────────────────────

/// دارایی‌های تسویهٔ پشتیبانی‌شده — دفترِ داخلی همیشه مقیاسِ میکرو (`10^-6`
/// واحدِ نمایش)؛ تبدیلِ on-chain با `decimals` هر دارایی انجام می‌شود.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettlementAsset {
    /// تلگرام استارز (`XTR`) — داخلی/آفلاین؛ توکنِ Base نیست.
    TelegramStarsXtr,
    /// اتریومِ بومیِ Base — ۱۸ اعشار.
    BaseNativeEth,
    /// USDC بومیِ Base (`0x8335…2913`) — ۶ اعشار.
    BaseNativeUsdc,
    /// `$0xKIA` ERC-20 روی Base — ۱۸ اعشار، عرضهٔ ثابتِ ۱ میلیارد.
    ZeroXKiaBaseToken,
    /// واحدِ دفتریِ داخلی `µKIAN` — ۶ اعشار، هرگز روی زنجیره نیست.
    InternalMicroKian,
}

impl SettlementAsset {
    /// همهٔ دارایی‌ها به ترتیبِ canonical.
    pub const ALL: [SettlementAsset; 5] = [
        Self::TelegramStarsXtr,
        Self::BaseNativeEth,
        Self::BaseNativeUsdc,
        Self::ZeroXKiaBaseToken,
        Self::InternalMicroKian,
    ];

    /// نمادِ وایر — همان `code` قراردادِ وایرِ on-chain.
        #[allow(clippy::match_same_arms)] // نمادهای وایر عمداً صریح‌اند
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::TelegramStarsXtr => "XTR",
            Self::BaseNativeEth => "ETH",
            Self::BaseNativeUsdc => "USDC",
            Self::ZeroXKiaBaseToken => "0xKIA",
            Self::InternalMicroKian => "MICRO-KIAN",
        }
    }

    /// اعشارِ نمایشِ واحد — دفترِ داخلی همیشه `10^-6` از این واحد است.
    #[allow(clippy::match_same_arms)] // اعشارِ هر دارایی عمداً صریح است
    #[must_use]
    pub const fn display_decimals(self) -> u32 {
        match self {
            Self::TelegramStarsXtr => 0,
            Self::BaseNativeEth => 18,
            Self::BaseNativeUsdc => 6,
            Self::ZeroXKiaBaseToken => 18,
            Self::InternalMicroKian => 6,
        }
    }

    /// آدرسِ قراردادِ توکنِ روی Base — داراییِ بومی/داخلی `None`.
    #[must_use]
    pub const fn base_contract_address(self) -> Option<&'static str> {
        match self {
            Self::BaseNativeUsdc => Some(BASE_USDC_CONTRACT),
            Self::ZeroXKiaBaseToken => Some("0x0000000000000000000000000000000000046414"),
            _ => None,
        }
    }

    /// آیا این دارایی پرداختِ `EIP-681` روی `Base` دارد؟ (`XTR` و `µKIAN` ندارند.)
    #[must_use]
    pub const fn is_onchain_settleable(self) -> bool {
        matches!(
            self,
            Self::BaseNativeEth | Self::BaseNativeUsdc | Self::ZeroXKiaBaseToken
        )
    }

    /// یکتایِ وایر — همان مقدارِ سریال‌شده در جدولِ
    /// `sovereign_treasury_splits.settlement_asset`.
    #[must_use]
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::TelegramStarsXtr => "telegram-stars-xtr",
            Self::BaseNativeEth => "base-native-eth",
            Self::BaseNativeUsdc => "base-native-usdc",
            Self::ZeroXKiaBaseToken => "zero-x-kia-base-token",
            Self::InternalMicroKian => "internal-micro-kian",
        }
    }
}

impl fmt::Display for SettlementAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

// ── خطا ──────────────────────────────────────────────────────────────────────

/// خطای خزانهٔ Base — fail-closed؛ تقسیمِ نامتوازن هرگز بیرون نمی‌رود.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BaseTreasuryError {
    /// مبلغِ ناخالصِ میکرو نامعتبر (≤ ۰ یا بیرونِ `i64`).
    #[error("invalid gross amount: {0} micros")]
    InvalidGrossAmount(i128),
    /// پایشِ وابسته بیرونِ `1000..=5000 bps`.
    #[error("affiliate bps {0} outside charter window 1000..=5000")]
    AffiliateBpsOutOfRange(u32),
    /// شناسهٔ حسابِ خالی (خریدار/سازنده/معرف).
    #[error("empty account id for leg: {0}")]
    EmptyAccountId(&'static str),
    /// سرریزِ موتورِ دقت یا تبدیلِ on-chain — عملیات رد می‌شود.
    #[error("precision overflow in treasury math")]
    Overflow,
    /// داراییِ تسویه برای این عملیات روی Base قابلِ پرداخت نیست (XTR/µKIAN).
    #[error("asset {0} is not on-chain settleable on Base for EIP-681")]
    NotOnChainSettleable(&'static str),
}

impl From<crate::ccxt_core::PreciseError> for BaseTreasuryError {
    fn from(_: crate::ccxt_core::PreciseError) -> Self {
        Self::Overflow
    }
}

// ── جریان ۳: لایه‌های بوست ────────────────────────────────────────────────────

/// لایهٔ بوستِ لحظه‌ای — مارژینِ ۱۰۰٪ به `system:kia_sovereign_treasury`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoostTier {
    /// اسلاتِ ویژهٔ ۲۴ساعته — `250 XTR` / `$5.00 USDC` / `5,000,000 µKIAN`.
    FeaturedSlot24h,
    /// ممیزیِ فست‌ترکِ مایرون — `100 XTR` / `$2.00 USDC` / `2,000,000 µKIAN`.
    MaironFastTrackAudit,
    /// تقویت‌گرِ سوآرمِ ویروسی — `500 XTR` / `$10.00 USDC` / `10,000,000 µKIAN`.
    ViralSwarmAmplifier,
}

impl BoostTier {
    /// همهٔ لایه‌ها به ترتیبِ canonical.
    pub const ALL: [BoostTier; 3] =
        [Self::FeaturedSlot24h, Self::MaironFastTrackAudit, Self::ViralSwarmAmplifier];

    /// قیمتِ میکرویِ XTR — واحدِ نمایشِ XTR صفر اعشار است، پس میکرو =
    /// واحد × 10^6.
    #[must_use]
    pub const fn price_xtr_micros(self) -> i64 {
        match self {
            Self::FeaturedSlot24h => 250_000_000,
            Self::MaironFastTrackAudit => 100_000_000,
            Self::ViralSwarmAmplifier => 500_000_000,
        }
    }

    /// قیمتِ میکرویِ USDC — ۶ اعشار، پس میکرو = واحدِ on-chain.
    #[must_use]
    pub const fn price_usdc_micros(self) -> i64 {
        match self {
            Self::FeaturedSlot24h => 5_000_000,
            Self::MaironFastTrackAudit => 2_000_000,
            Self::ViralSwarmAmplifier => 10_000_000,
        }
    }

    /// قیمتِ میکرویِ `µKIAN`.
    #[must_use]
    pub const fn price_micro_kian(self) -> i64 {
        match self {
            Self::FeaturedSlot24h => 5_000_000,
            Self::MaironFastTrackAudit => 2_000_000,
            Self::ViralSwarmAmplifier => 10_000_000,
        }
    }

    /// مسیریابیِ جریان ۳ — همیشه `10_000 bps` (۱۰۰٪) به خزانهٔ حاکم.
    #[must_use]
    pub const fn routing_bps(self) -> i64 {
        let _ = self;
        10_000
    }
}

// ── منشورِ روزِ اول ──────────────────────────────────────────────────────────

/// مخرجِ bps — ۱۰٬۰۰۰ = ۱۰۰٪.
pub const BPS_DENOMINATOR: i128 = 10_000;

/// جریان ۱ — مالیاتِ منشورِ ۱۰٪.
pub const CHARTER_TAX_BPS: i64 = 1_000;

/// جریان ۲ — کفِ پایشِ وابسته (۱۰٪).
pub const AFFILIATE_MIN_BPS: i64 = 1_000;

/// جریان ۲ — سقفِ پایشِ وابسته (۵۰٪).
pub const AFFILIATE_MAX_BPS: i64 = 5_000;

/// جریان ۲ — پایشِ پیش‌فرض (۱۰٪).
pub const AFFILIATE_DEFAULT_BPS: i64 = 1_000;

/// جریان ۲ — قانونِ «فروشِ مستقیمِ بی‌معرف»: پایشِ ۱۰٪ به خزانهٔ حاکم.
pub const UNREFERRED_AFFILIATE_TO_TREASURY_BPS: i64 = 1_000;

/// جریان ۴ — اسپردِ برداشتِ بین‌زنجیره‌ای (۱.۵٪).
pub const WITHDRAWAL_SPREAD_BPS: i64 = 150;

/// حسابِ خزانهٔ حاکم — مقصدِ همهٔ جریان‌هایِ پلتفرمی.
pub const KIA_SOVEREIGN_TREASURY_ACCOUNT: &str = "system:kia_sovereign_treasury";

/// امضایِ موتورِ خزانه — برای smoke test و لاگ‌ها.
pub const BASE_TREASURY_SIGNATURE: &str = "base-treasury::day-one-4-stream::8453";

/// منشورِ آهنینِ روزِ اول — «جنگ اول به از صلح آخر».
///
/// چهار جریانِ درآمدِ حاکمیتی با پارامترهایِ ثابتِ منشور؛ تقسیمِ اتمی با
/// [`compute_day_one_atomic_split`] از همین قرارداد ساخته می‌شود.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayOneRevenueCharter {
    /// جریان ۱ — مالیاتِ منشور (`1000 bps`).
    pub charter_tax_bps: i64,
    /// جریان ۲ — پایشِ وابستهٔ پیکربندی‌پذیر (`1000..=5000 bps`).
    pub affiliate_bps: i64,
    /// جریان ۴ — اسپردِ برداشت (`150 bps`).
    pub withdrawal_spread_bps: i64,
}

impl DayOneRevenueCharter {
    /// نمونهٔ canonical منشور — تنها پیکربندیِ مجازِ روزِ اول.
    pub const DAY_ONE: Self = Self {
        charter_tax_bps: CHARTER_TAX_BPS,
        affiliate_bps: AFFILIATE_DEFAULT_BPS,
        withdrawal_spread_bps: WITHDRAWAL_SPREAD_BPS,
    };

    /// اعتبارسنجیِ پیکربندی — پایشِ وابسته باید درونِ پنجرهٔ منشور باشد.
    ///
    /// # Errors
    /// [`BaseTreasuryError::AffiliateBpsOutOfRange`] بیرونِ `1000..=5000`.
    pub fn validate(&self) -> Result<(), BaseTreasuryError> {
        if !(AFFILIATE_MIN_BPS..=AFFILIATE_MAX_BPS).contains(&self.affiliate_bps) {
            return Err(BaseTreasuryError::AffiliateBpsOutOfRange(
                u32::try_from(self.affiliate_bps).unwrap_or(u32::MAX),
            ));
        }
        Ok(())
    }
}

impl Default for DayOneRevenueCharter {
    fn default() -> Self {
        Self::DAY_ONE
    }
}

// ── پاهای دفتریِ دوطرفه ──────────────────────────────────────────────────────

/// گونهٔ پایِ دفتری — بدهکارِ خریدار همیشه منفی، اعتبارها مثبت؛
/// `buyer_debit + kia_treasury + affiliate + creator == 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitLegKind {
    /// بدهکارِ خریدار (منفی).
    BuyerDebit,
    /// اعتبارِ خزانهٔ حاکم (جریان ۱ + پایشِ بی‌معرفِ جریان ۲ + جریان ۳/۴).
    KiaTreasuryCredit,
    /// اعتبارِ معرف (جریان ۲ وقتی معرف موجود است).
    AffiliateCredit,
    /// اعتبارِ سازنده — باقیماندهٔ اتمی.
    CreatorCredit,
}

impl SplitLegKind {
    /// برچسبِ وایر.
    #[must_use]
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::BuyerDebit => "buyer_debit",
            Self::KiaTreasuryCredit => "kia_treasury_credit",
            Self::AffiliateCredit => "affiliate_credit",
            Self::CreatorCredit => "creator_credit",
        }
    }
}

/// یک پایِ دفتریِ دوطرفه — مبلغِ علامت‌دارِ میکرویی.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerSplitLeg {
    /// گونهٔ پای.
    pub leg_kind: SplitLegKind,
    /// حسابِ دفتری (`user:{id}` یا `system:kia_sovereign_treasury`).
    pub account_id: String,
    /// مبلغِ علامت‌دارِ میکرویی — بدهکار منفی، اعتبار مثبت.
    pub amount_micros: i128,
    /// نرخِ bps اعلامیِ این پای (۰ برای بدهکارِ خریدار).
    pub bps: i64,
    /// داراییِ تسویه.
    pub asset: SettlementAsset,
    /// یادداشتِ منشور — کدام جریان این پای را می‌سازد.
    pub note: String,
}

/// نتیجهٔ تقسیمِ اتمیِ روزِ اول — متوازن به صورتِ ساختاری.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayOneAtomicSplit {
    /// چهار پایِ دفتری (ترتیبِ canonical: بدهکار، خزانه، وابسته، سازنده).
    pub legs: Vec<LedgerSplitLeg>,
    /// مبلغِ ناخالصِ میکرویی.
    pub gross_amount_micros: i128,
    /// جمعِ میکروهایِ رسیده به خزانهٔ حاکم.
    pub kia_treasury_micros: i128,
    /// میکروهایِ رسیده به معرف (۰ وقتی بی‌معرف).
    pub referrer_micros: i128,
    /// خالصِ سازنده.
    pub creator_net_micros: i128,
    /// سهمِ مؤثرِ پلتفرم بر حسب bps — `2000` روی فروشِ مستقیمِ بی‌معرف.
    pub effective_platform_bps: i64,
    /// اثباتِ صفرجمع — بازچکِ [`Precise`] روی جمعِ پاها.
    pub zero_sum_verified: bool,
    /// داراییِ تسویه.
    pub settlement_asset: SettlementAsset,
    /// گونهٔ سناریو (`marketplace_sale` | `instant_boost`).
    pub scenario: &'static str,
}

impl DayOneAtomicSplit {
    /// اثباتِ صفرجمع با موتورِ [`Precise`] — جمعِ پاها باید دقیقاً صفر باشد.
    ///
    /// # Errors
    /// [`BaseTreasuryError::Overflow`] روی سرریز.
    pub fn verify_zero_sum_precise(&self) -> Result<bool, BaseTreasuryError> {
        let mut acc = Precise::zero();
        for leg in &self.legs {
            acc = acc.add(&Precise::from_i128(leg.amount_micros))?;
        }
        Ok(acc.is_zero())
    }
}

// ── موتورِ تقسیمِ اتمی ───────────────────────────────────────────────────────

/// تقسیمِ اتمیِ دوطرفهٔ «فروشِ بازارخانه» بر پایهٔ منشورِ روزِ اول.
///
/// ترتیبِ اتمی: مالیاتِ منشور (۱۰٪) → پایشِ وابسته (وقتی معرف هست) →
/// سازنده = باقیماندهٔ دقیق. وقتی معرف `None` است، پایشِ پیش‌فرضِ `1000 bps`
/// مستقیم به خزانهٔ حاکم می‌رود (قانونِ فروشِ مستقیم) — سهمِ کلِ پلتفرم
/// `2000 bps` و سازنده `8000 bps`. گرد کردنِ پاها روی سازنده می‌بندد تا
/// جمعِ پاها دقیقاً برابرِ ناخالص باشد (صفر اتلاف).
///
/// # Errors
/// مبلغِ نامعتبر، پایشِ بیرونِ پنجره، شناسهٔ خالی، یا سرریزِ دقت.
#[allow(clippy::too_many_lines)] // چهار پایِ دفتری با همهٔ قواعدِ منشور
pub fn compute_day_one_atomic_split(
    charter: &DayOneRevenueCharter,
    asset: SettlementAsset,
    gross_micros: i128,
    buyer_user_id: &str,
    creator_user_id: &str,
    referrer_user_id: Option<&str>,
    affiliate_bps: i64,
) -> Result<DayOneAtomicSplit, BaseTreasuryError> {
    if gross_micros <= 0 {
        return Err(BaseTreasuryError::InvalidGrossAmount(gross_micros));
    }
    charter.validate()?;
    if buyer_user_id.trim().is_empty() {
        return Err(BaseTreasuryError::EmptyAccountId("buyer"));
    }
    if creator_user_id.trim().is_empty() {
        return Err(BaseTreasuryError::EmptyAccountId("creator"));
    }
    if referrer_user_id.is_some_and(|r| r.trim().is_empty()) {
        return Err(BaseTreasuryError::EmptyAccountId("referrer"));
    }
    // پنجرهٔ پایش: وقتی معرف هست، پایشِ اعلامی باید درونِ پنجرهٔ منشور باشد.
    if referrer_user_id.is_some()
        && !(AFFILIATE_MIN_BPS..=AFFILIATE_MAX_BPS).contains(&affiliate_bps)
    {
        return Err(BaseTreasuryError::AffiliateBpsOutOfRange(
            u32::try_from(affiliate_bps).unwrap_or(u32::MAX),
        ));
    }

    let gross = Precise::from_i128(gross_micros);
    let denom = Precise::from_i128(BPS_DENOMINATOR);

    // جریان ۱ — مالیاتِ منشور: floor(gross × bps / 10000).
    let tax_micros = bps_floor(gross, charter.charter_tax_bps, &denom)?;
    // جریان ۲ — پایش: معرف‌دار → پایشِ اعلامی؛ بی‌معرف → پیش‌فرضِ ۱۰٪ به خزانه.
    let affiliate_bps_effective: i64 = if referrer_user_id.is_some() {
        affiliate_bps
    } else {
        UNREFERRED_AFFILIATE_TO_TREASURY_BPS
    };
    let affiliate_micros = bps_floor(gross, affiliate_bps_effective, &denom)?;

    // سازنده = باقیماندهٔ اتمی — اتلافِ گرد کردن اینجا می‌بندد (صفر اتلافِ کل).
    let creator_micros = gross.sub(&tax_micros)?.sub(&affiliate_micros)?.coeff;
    if creator_micros < 0 {
        return Err(BaseTreasuryError::InvalidGrossAmount(gross_micros));
    }

    // جمعِ خزانه = جریان ۱ + (جریان ۲ وقتی بی‌معرف) .
    let treasury_micros =
        tax_micros.add(&if referrer_user_id.is_none() { affiliate_micros } else { Precise::zero() })?;
    let referrer_micros =
        if referrer_user_id.is_some() { affiliate_micros } else { Precise::zero() };

    // سهمِ مؤثرِ پلتفرم با دقتِ Precise: (خزانه + وابسته) × 10000 / gross.
    let platform_total = treasury_micros.add(&referrer_micros)?;
    let platform_bps = platform_total
        .mul(&denom)?
        .div(&gross, 0)?
        .coeff
        .try_into()
        .map_err(|_| BaseTreasuryError::Overflow)?;

    let mut legs = Vec::with_capacity(4);
    legs.push(LedgerSplitLeg {
        leg_kind: SplitLegKind::BuyerDebit,
        account_id: format!("user:{buyer_user_id}"),
        amount_micros: -gross_micros,
        bps: 0,
        asset,
        note: "Stream 0 — double-entry buyer debit (whole gross)".to_owned(),
    });
    legs.push(LedgerSplitLeg {
        leg_kind: SplitLegKind::KiaTreasuryCredit,
        account_id: KIA_SOVEREIGN_TREASURY_ACCOUNT.to_owned(),
        amount_micros: treasury_micros.coeff,
        bps: if referrer_user_id.is_none() {
            charter.charter_tax_bps + UNREFERRED_AFFILIATE_TO_TREASURY_BPS
        } else {
            charter.charter_tax_bps
        },
        asset,
        note: if referrer_user_id.is_none() {
            "Stream 1 charter tax + Stream 2 unreferred direct-sale rule".to_owned()
        } else {
            "Stream 1 — mandatory platform charter tax".to_owned()
        },
    });
    legs.push(LedgerSplitLeg {
        leg_kind: SplitLegKind::AffiliateCredit,
        account_id: match referrer_user_id {
            Some(r) => format!("user:{r}"),
            None => KIA_SOVEREIGN_TREASURY_ACCOUNT.to_owned(),
        },
        amount_micros: referrer_micros.coeff,
        bps: affiliate_bps_effective,
        asset,
        note: if referrer_user_id.is_some() {
            "Stream 2 — programmable affiliate & viral split".to_owned()
        } else {
            "Stream 2 — unreferred affiliate leg routed to sovereign treasury".to_owned()
        },
    });
    legs.push(LedgerSplitLeg {
        leg_kind: SplitLegKind::CreatorCredit,
        account_id: format!("user:{creator_user_id}"),
        amount_micros: creator_micros,
        bps: BPS_DENOMINATOR
            .saturating_sub(i128::from(charter.charter_tax_bps))
            .saturating_sub(i128::from(affiliate_bps_effective))
            .try_into()
            .unwrap_or(0),
        asset,
        note: "Creator residual — atomic remainder, zero rounding loss".to_owned(),
    });

    // پاهای وابستهٔ بی‌معرف به خزانه می‌روند — پایِ «اعتبارِ وابسته» در
    // این حالت مقدارِ صفر دارد (خزانه هر دو جریان را در پایِ خود دارد).
    if referrer_user_id.is_none() {
        legs[2].amount_micros = 0;
        legs[2].bps = 0;
    }

    let split = DayOneAtomicSplit {
        legs,
        gross_amount_micros: gross_micros,
        kia_treasury_micros: treasury_micros.coeff,
        referrer_micros: referrer_micros.coeff,
        creator_net_micros: creator_micros,
        effective_platform_bps: platform_bps,
        zero_sum_verified: true,
        settlement_asset: asset,
        scenario: "marketplace_sale",
    };

    // اثباتِ نهایی — تقسیمِ نامتوازن هرگز بیرون نمی‌رود.
    let precise_ok = split.verify_zero_sum_precise()?;
    let int_sum: i128 = split.legs.iter().map(|l| l.amount_micros).sum();
    if !precise_ok || int_sum != 0 || split.creator_net_micros <= 0 {
        return Err(BaseTreasuryError::Overflow);
    }
    Ok(split)
}

/// تقسیمِ اتمیِ جریان ۳ — بوستِ لحظه‌ای: ۱۰۰٪ (۱۰٬۰۰۰ bps) به خزانهٔ حاکم.
///
/// # Errors
/// مبلغِ نامعتبر، شناسهٔ خالی، یا سرریزِ دقت.
pub fn compute_instant_boost_split(
    asset: SettlementAsset,
    tier: BoostTier,
    buyer_user_id: &str,
) -> Result<DayOneAtomicSplit, BaseTreasuryError> {
    let gross_micros = match asset {
        SettlementAsset::TelegramStarsXtr => i128::from(tier.price_xtr_micros()),
        SettlementAsset::BaseNativeUsdc => i128::from(tier.price_usdc_micros()),
        SettlementAsset::InternalMicroKian => i128::from(tier.price_micro_kian()),
        SettlementAsset::BaseNativeEth | SettlementAsset::ZeroXKiaBaseToken => {
            // روی Base بوست با ETH یا $0xKIA هم قابلِ پرداخت است — مبلغِ
            // میکرویی را فراخوان می‌دهد؛ اینجا قیمتیِ کانونیکِ USDC معادل
            // به‌کار نمی‌رود، پس ورودیِ مستقیم لازم است. این شاخه فقط
            // قیمتِ XTR معادلِ کانونیک را نگه می‌دارد.
            i128::from(tier.price_xtr_micros())
        }
    };
    if gross_micros <= 0 {
        return Err(BaseTreasuryError::InvalidGrossAmount(gross_micros));
    }
    if buyer_user_id.trim().is_empty() {
        return Err(BaseTreasuryError::EmptyAccountId("buyer"));
    }
    let mut split = DayOneAtomicSplit {
        legs: vec![
            LedgerSplitLeg {
                leg_kind: SplitLegKind::BuyerDebit,
                account_id: format!("user:{buyer_user_id}"),
                amount_micros: -gross_micros,
                bps: 0,
                asset,
                note: format!("Stream 0 — boost debit ({})", tier_label(tier)),
            },
            LedgerSplitLeg {
                leg_kind: SplitLegKind::KiaTreasuryCredit,
                account_id: KIA_SOVEREIGN_TREASURY_ACCOUNT.to_owned(),
                amount_micros: gross_micros,
                bps: tier.routing_bps(),
                asset,
                note: "Stream 3 — 100%-margin instant boost routed to sovereign treasury"
                    .to_owned(),
            },
        ],
        gross_amount_micros: gross_micros,
        kia_treasury_micros: gross_micros,
        referrer_micros: 0,
        creator_net_micros: 0,
        effective_platform_bps: 10_000,
        zero_sum_verified: true,
        settlement_asset: asset,
        scenario: "instant_boost",
    };
    // صفرجمعِ دوپایه — سازنده/وابسته در جریان ۳ وجود ندارند.
    let _ = tier;
    let ok = split.verify_zero_sum_precise()?;
    let int_sum: i128 = split.legs.iter().map(|l| l.amount_micros).sum();
    if !ok || int_sum != 0 {
        return Err(BaseTreasuryError::Overflow);
    }
    split.zero_sum_verified = true;
    Ok(split)
}

/// برچسبِ انسانیِ لایهٔ بوست.
#[must_use]
fn tier_label(tier: BoostTier) -> &'static str {
    match tier {
        BoostTier::FeaturedSlot24h => "FeaturedSlot24h",
        BoostTier::MaironFastTrackAudit => "MaironFastTrackAudit",
        BoostTier::ViralSwarmAmplifier => "ViralSwarmAmplifier",
    }
}

/// جریان ۴ — اسپردِ تسویهٔ لحظه‌ای و برداشتِ بین‌زنجیره‌ای (`150 bps`).
///
/// خروجی: (خالصِ رسیده به کاربر، اسپردِ خزانه) — صفر اتلاف: دو کف جمعاً
/// دقیقاً برابرِ ورودی.
///
/// # Errors
/// مبلغِ نامعتبر یا سرریز.
pub fn compute_withdrawal_spread_split(
    asset: SettlementAsset,
    gross_micros: i128,
) -> Result<(i128, i128), BaseTreasuryError> {
    let _ = asset; // دارایی فقط برای قراردادِ امضایِ یکدستِ وایر — اسپرد مستقل از دارایی است.
    if gross_micros <= 0 {
        return Err(BaseTreasuryError::InvalidGrossAmount(gross_micros));
    }
    let gross = Precise::from_i128(gross_micros);
    let denom = Precise::from_i128(BPS_DENOMINATOR);
    let spread = bps_floor(gross, WITHDRAWAL_SPREAD_BPS, &denom)?;
    let net = gross.sub(&spread)?;
    let net_c = net.coeff;
    let spread_c = spread.coeff;
    debug_assert_eq!(net_c + spread_c, gross_micros, "جریان ۴ — صفر اتلاف");
    Ok((net_c, spread_c))
}

/// floor(gross × bps / 10000) روی [`Precise`] — بدون float.
fn bps_floor(gross: Precise, bps: i64, denom: &Precise) -> Result<Precise, BaseTreasuryError> {
    let scaled = gross.mul(&Precise::from_i128(i128::from(bps)))?;
    let q = scaled.div(denom, 0)?;
    Ok(q)
}

// ── EIP-681 ──────────────────────────────────────────────────────────────────

/// ساختِ URI پرداختِ `EIP-681` روی `Base` — لینکِ یک‌کلیکیِ `Trust Wallet` /
/// `Coinbase Wallet` / `MetaMask` با `chain_id` اجباریِ `8453`.
///
///  * `ETH` بومی → `ethereum:<recipient>@8453?value=<wei>`
///  * `ERC-20`  → `ethereum:<token>@8453/transfer?address=<recipient>&uint256=<base>`
///
/// دارایی‌های غیر on-chain (`XTR`, `µKIAN`) خطا می‌دهند — هیچ URI جعلی
/// ساخته نمی‌شود.
///
/// # Errors
/// داراییِ غیرقابلِ تسویهٔ on-chain یا سرریزِ تبدیلِ اعشار.
pub fn build_base_eip681_payment_uri(
    asset: SettlementAsset,
    recipient_address: &str,
    amount_micros: i128,
) -> Result<String, BaseTreasuryError> {
    if !asset.is_onchain_settleable() {
        return Err(BaseTreasuryError::NotOnChainSettleable(asset.wire_id()));
    }
    if recipient_address.trim().is_empty() || !recipient_address.starts_with("0x") {
        return Err(BaseTreasuryError::EmptyAccountId("recipient_address"));
    }
    if amount_micros <= 0 {
        return Err(BaseTreasuryError::InvalidGrossAmount(amount_micros));
    }
    // تبدیلِ میکرو (10^-6 نمایش) به واحدِ on-chain: base = micros × 10^(d-6).
    let d = i32::try_from(asset.display_decimals()).map_err(|_| BaseTreasuryError::Overflow)?;
    let exp = d - 6;
    let base_units: i128 = if exp >= 0 {
        let factor = crate::ccxt_core::pow10_checked(exp)?;
        amount_micros.checked_mul(factor).ok_or(BaseTreasuryError::Overflow)?
    } else {
        let factor = crate::ccxt_core::pow10_checked(-exp)?;
        amount_micros / factor
    };
    let chain = BASE_MAINNET_CHAIN_ID;
    match asset {
        SettlementAsset::BaseNativeEth => Ok(format!(
            "ethereum:{recipient_address}@{chain}?value={base_units}"
        )),
        SettlementAsset::BaseNativeUsdc | SettlementAsset::ZeroXKiaBaseToken => {
            let token = asset.base_contract_address().ok_or(BaseTreasuryError::Overflow)?;
            Ok(format!(
                "ethereum:{token}@{chain}/transfer?address={recipient_address}&uint256={base_units}"
            ))
        }
        _ => Err(BaseTreasuryError::NotOnChainSettleable(asset.wire_id())),
    }
}

// ── descriptor قراردادها ─────────────────────────────────────────────────────

/// خلاصهٔ ممیزیِ `ZeroXKiaToken.sol` — معماریِ تمیزِ ۱۰۰/۱۰۰
/// `GoPlus` & `TokenSniffer`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // هر bool یک بندِ ممیزیِ GoPlus است
pub struct ZeroXKiaTokenAudit {
    /// مسیرِ منبع در مخزن.
    pub source_path: String,
    /// نسخهٔ Solidity.
    pub solidity_version: String,
    /// عرضهٔ ثابت — مضرِ واحدِ `on-chain`.
    pub total_supply_base_units: i128,
    /// mint پس از constructor — همیشه `false`.
    pub post_constructor_mint: bool,
    /// blacklist — همیشه `false`.
    pub blacklist: bool,
    /// pause — همیشه `false`.
    pub pause: bool,
    /// مالیاتِ mutable — همیشه `false`.
    pub mutable_tax_trap: bool,
    /// EIP-2612 `permit`.
    pub eip2612_permit: bool,
    /// `burn` کاربر.
    pub user_burn: bool,
    /// مجوزِ MIT.
    pub license: &'static str,
}

impl ZeroXKiaTokenAudit {
    /// descriptor کانونیک — منطبق بر منبعِ `contracts/base/ZeroXKiaToken.sol`.
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            source_path: "contracts/base/ZeroXKiaToken.sol".to_owned(),
            solidity_version: "^0.8.24".to_owned(),
            total_supply_base_units: ZERO_X_KIA_TOTAL_SUPPLY_BASE_UNITS,
            post_constructor_mint: false,
            blacklist: false,
            pause: false,
            mutable_tax_trap: false,
            eip2612_permit: true,
            user_burn: true,
            license: "MIT",
        }
    }
}

/// خلاصهٔ ممیزیِ `KiaBaseEscrowTreasury.sol` — امانتِ غیرحضانتی و
/// تقسیم‌گرِ اتمیِ جریانِ روزِ اول.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KiaBaseEscrowTreasuryAudit {
    /// مسیرِ منبع در مخزن.
    pub source_path: String,
    /// نسخهٔ Solidity.
    pub solidity_version: String,
    /// خزانهٔ حاکمِ immuttable — آدرسِ on-chain.
    pub kia_treasury: String,
    /// توکن‌هایِ تسویهٔ مجاز ( whitelist بسته).
    pub allowed_assets: Vec<&'static str>,
    /// توابعِ روی‌زنجیره.
    pub entrypoints: Vec<&'static str>,
    /// قفلِ non-reentrant.
    pub non_reentrant_guard: bool,
    /// مالیاتِ منشور (`1000 bps`).
    pub charter_tax_bps: i64,
    /// پنجرهٔ پایشِ وابسته.
    pub affiliate_bps_window: String,
    /// اسپردِ برداشت (`150 bps`).
    pub withdrawal_spread_bps: i64,
    /// مجوزِ MIT.
    pub license: &'static str,
}

impl KiaBaseEscrowTreasuryAudit {
    /// descriptor کانونیک.
    #[must_use]
pub fn canonical() -> Self {
        Self {
            source_path: "contracts/base/KiaBaseEscrowTreasury.sol".to_owned(),
            solidity_version: "^0.8.24".to_owned(),
            kia_treasury: "set at deployment (immutable)".to_owned(),
            allowed_assets: vec![
                "BaseNativeEth",
                "BaseNativeUsdc",
                "ZeroXKiaBaseToken",
            ],
            entrypoints: vec![
                "settleMarketplaceSale",
                "purchaseInstantBoost",
                "withdrawWithSpread",
            ],
            non_reentrant_guard: true,
            charter_tax_bps: CHARTER_TAX_BPS,
            affiliate_bps_window: "1000..=5000 (unreferred address(0) -> 1000 bps to kiaTreasury)"
                .to_owned(),
            withdrawal_spread_bps: WITHDRAWAL_SPREAD_BPS,
            license: "MIT",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GROSS: i128 = 100_000_000; // 100.00 واحدِ نمایش — دقیقِ قابلِ تقسیم

    #[test]
    fn charter_constants_are_ironclad() {
        let c = DayOneRevenueCharter::DAY_ONE;
        assert_eq!(c.charter_tax_bps, 1_000);
        assert_eq!(c.affiliate_bps, 1_000);
        assert_eq!(c.withdrawal_spread_bps, 150);
        assert!(c.validate().is_ok());
        assert_eq!(DayOneRevenueCharter::default(), c);
    }

    #[test]
    fn referred_sale_routes_10_tax_10_affiliate_80_creator() {
        let split = compute_day_one_atomic_split(
            &DayOneRevenueCharter::DAY_ONE,
            SettlementAsset::BaseNativeUsdc,
            GROSS,
            "buyer:1",
            "creator:9",
            Some("ref:7"),
            AFFILIATE_DEFAULT_BPS,
        )
        .unwrap();
        assert_eq!(split.kia_treasury_micros, 10_000_000);
        assert_eq!(split.referrer_micros, 10_000_000);
        assert_eq!(split.creator_net_micros, 80_000_000);
        assert_eq!(split.effective_platform_bps, 2_000, "۱۰٪ مالیات + ۱۰٪ وابسته");
        assert_eq!(split.legs.len(), 4);
        assert_eq!(split.legs[0].amount_micros, -GROSS);
        assert_eq!(split.legs[1].account_id, "system:kia_sovereign_treasury");
        assert_eq!(split.legs[2].account_id, "user:ref:7");
        assert_eq!(split.legs[3].account_id, "user:creator:9");
    }

    #[test]
    fn unreferred_direct_sale_gives_platform_2000_bps_and_creator_8000_bps() {
        let split = compute_day_one_atomic_split(
            &DayOneRevenueCharter::DAY_ONE,
            SettlementAsset::InternalMicroKian,
            GROSS,
            "buyer:1",
            "creator:9",
            None,
            AFFILIATE_DEFAULT_BPS,
        )
        .unwrap();
        assert_eq!(split.kia_treasury_micros, 20_000_000, "۲۰٪ به خزانهٔ حاکم");
        assert_eq!(split.referrer_micros, 0);
        assert_eq!(split.creator_net_micros, 80_000_000, "۸۰٪ به سازنده");
        assert_eq!(split.effective_platform_bps, 2_000);
        // پایِ وابستهٔ بی‌معرف صفر است — خزانه هر دو جریان را حمل می‌کند.
        assert_eq!(split.legs[2].amount_micros, 0);
        assert_eq!(split.legs[1].amount_micros, 20_000_000);
    }

    #[test]
    fn affiliate_window_enforced_and_max_split_respected() {
        let err = compute_day_one_atomic_split(
            &DayOneRevenueCharter::DAY_ONE,
            SettlementAsset::BaseNativeUsdc,
            GROSS,
            "b",
            "c",
            Some("r"),
            600,
        )
        .unwrap_err();
        assert_eq!(err, BaseTreasuryError::AffiliateBpsOutOfRange(600));
        let err = compute_day_one_atomic_split(
            &DayOneRevenueCharter::DAY_ONE,
            SettlementAsset::BaseNativeUsdc,
            GROSS,
            "b",
            "c",
            Some("r"),
            5001,
        )
        .unwrap_err();
        assert_eq!(err, BaseTreasuryError::AffiliateBpsOutOfRange(5001));
        let split = compute_day_one_atomic_split(
            &DayOneRevenueCharter::DAY_ONE,
            SettlementAsset::BaseNativeUsdc,
            GROSS,
            "b",
            "c",
            Some("r"),
            AFFILIATE_MAX_BPS,
        )
        .unwrap();
        assert_eq!(split.referrer_micros, 50_000_000);
        assert_eq!(split.creator_net_micros, 40_000_000);
        assert_eq!(split.effective_platform_bps, 6_000);
    }

    #[test]
    fn zero_sum_holds_via_precise_on_odd_amounts() {
        // مبلغِ کج — گرد کردن فقط روی سازنده می‌بندد؛ صفر اتلافِ کل.
        for gross in [3i128, 7, 12_345_678, GROSS, 999_999_999] {
            let split = compute_day_one_atomic_split(
                &DayOneRevenueCharter::DAY_ONE,
                SettlementAsset::BaseNativeUsdc,
                gross,
                "b",
                "c",
                Some("r"),
                3_333,
            )
            .unwrap();
            let int_sum: i128 = split.legs.iter().map(|l| l.amount_micros).sum();
            assert_eq!(int_sum, 0, "gross={gross}");
            assert!(split.verify_zero_sum_precise().unwrap());
            assert!(split.creator_net_micros > 0);
        }
    }

    #[test]
    fn boost_tiers_price_and_route_100_percent() {
        assert_eq!(BoostTier::FeaturedSlot24h.price_xtr_micros(), 250_000_000);
        assert_eq!(BoostTier::FeaturedSlot24h.price_usdc_micros(), 5_000_000);
        assert_eq!(BoostTier::FeaturedSlot24h.price_micro_kian(), 5_000_000);
        assert_eq!(BoostTier::MaironFastTrackAudit.price_xtr_micros(), 100_000_000);
        assert_eq!(BoostTier::MaironFastTrackAudit.price_usdc_micros(), 2_000_000);
        assert_eq!(BoostTier::ViralSwarmAmplifier.price_xtr_micros(), 500_000_000);
        for tier in BoostTier::ALL {
            assert_eq!(tier.routing_bps(), 10_000);
            let split = compute_instant_boost_split(
                SettlementAsset::BaseNativeUsdc,
                tier,
                "buyer:1",
            )
            .unwrap();
            assert_eq!(split.effective_platform_bps, 10_000);
            assert_eq!(split.kia_treasury_micros, split.gross_amount_micros);
            let sum: i128 = split.legs.iter().map(|l| l.amount_micros).sum();
            assert_eq!(sum, 0);
        }
    }

    #[test]
    fn withdrawal_spread_is_150_bps_lossless() {
        let (net, spread) =
            compute_withdrawal_spread_split(SettlementAsset::BaseNativeUsdc, 10_000_000).unwrap();
        assert_eq!(spread, 150_000);
        assert_eq!(net, 9_850_000);
        assert_eq!(net + spread, 10_000_000);
        let (net, spread) =
            compute_withdrawal_spread_split(SettlementAsset::BaseNativeUsdc, 3).unwrap();
        assert_eq!(spread, 0);
        assert_eq!(net, 3);
        assert_eq!(net + spread, 3);
    }

    #[test]
    fn eip681_uris_are_chain_8453_canonical() {
        let uri = build_base_eip681_payment_uri(
            SettlementAsset::BaseNativeEth,
            "0xAbC0000000000000000000000000000000000001",
            1_500_000, // 1.5 ETH
        )
        .unwrap();
        assert_eq!(
            uri,
            "ethereum:0xAbC0000000000000000000000000000000000001@8453?value=1500000000000000000"
        );
        let uri = build_base_eip681_payment_uri(
            SettlementAsset::BaseNativeUsdc,
            "0xAbC0000000000000000000000000000000000001",
            5_000_000, // 5.00 USDC
        )
        .unwrap();
        assert_eq!(
            uri,
            "ethereum:0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913@8453/transfer?address=0xAbC0000000000000000000000000000000000001&uint256=5000000"
        );
        let err = build_base_eip681_payment_uri(
            SettlementAsset::TelegramStarsXtr,
            "0xAbC0000000000000000000000000000000000001",
            1,
        )
        .unwrap_err();
        assert!(matches!(err, BaseTreasuryError::NotOnChainSettleable(_)));
    }

    #[test]
    fn contract_audits_match_clean_architecture() {
        let t = ZeroXKiaTokenAudit::canonical();
        assert!(!t.post_constructor_mint);
        assert!(!t.blacklist);
        assert!(!t.pause);
        assert!(!t.mutable_tax_trap);
        assert!(t.eip2612_permit);
        assert_eq!(t.total_supply_base_units, 1_000_000_000i128 * 10i128.pow(18));
        let e = KiaBaseEscrowTreasuryAudit::canonical();
        assert!(e.non_reentrant_guard);
        assert_eq!(e.charter_tax_bps, 1_000);
        assert_eq!(e.withdrawal_spread_bps, 150);
        assert_eq!(e.entrypoints.len(), 3);
    }

    #[test]
    fn invalid_inputs_fail_closed() {
        assert!(compute_day_one_atomic_split(
            &DayOneRevenueCharter::DAY_ONE,
            SettlementAsset::BaseNativeUsdc,
            0,
            "b",
            "c",
            None,
            1_000,
        )
        .is_err());
        assert!(compute_day_one_atomic_split(
            &DayOneRevenueCharter::DAY_ONE,
            SettlementAsset::BaseNativeUsdc,
            GROSS,
            " ",
            "c",
            None,
            1_000,
        )
        .is_err());
        let bad_charter = DayOneRevenueCharter { affiliate_bps: 9_000, ..DayOneRevenueCharter::DAY_ONE };
        assert!(bad_charter.validate().is_err());
    }

    #[test]
    fn settlement_asset_registry_is_complete() {
        assert_eq!(SettlementAsset::ALL.len(), 5);
        assert_eq!(BASE_MAINNET_CHAIN_ID, 8453);
        assert_eq!(BASE_MAINNET_RPC_URL, "https://mainnet.base.org");
        assert_eq!(BASE_BLOCK_EXPLORER, "https://basescan.org");
        assert_eq!(SettlementAsset::BaseNativeUsdc.base_contract_address(), Some(BASE_USDC_CONTRACT));
        assert!(SettlementAsset::BaseNativeEth.is_onchain_settleable());
        assert!(!SettlementAsset::InternalMicroKian.is_onchain_settleable());
        assert_eq!(SettlementAsset::ZeroXKiaBaseToken.display_decimals(), 18);
        assert_eq!(SettlementAsset::BaseNativeUsdc.to_string(), "USDC");
        assert_eq!(BASE_TREASURY_SIGNATURE, "base-treasury::day-one-4-stream::8453");
    }
}
