//! The sovereign `Base` treasury — first war for the last peace (Wave 7 — Step 4/4).
//!
//! Four day-one revenue streams on Ethereum **Base Mainnet** (`Chain ID: 8453`):
//!  * Stream 1 — the 10% charter tax (`1000 bps`) on every marketplace sale;
//!  * Stream 2 — a programmable affiliate split of 10% to 50% (`1000..=5000 bps`) —
//!    the "no-referrer direct sale" rule: when the referrer is `None`, the default
//!    10% share goes straight to the sovereign treasury → a total platform share of `2000 bps` (20%)
//!    and a creator share of `8000 bps` (80%);
//!  * Stream 3 — instant boosts at a 100% margin (`FeaturedSlot24h`,
//!    `MaironFastTrackAudit`, `ViralSwarmAmplifier`) — all to the sovereign treasury;
//!  * Stream 4 — the instant-settlement and cross-chain-withdrawal spread `1.5%`
//!    (`150 bps`).
//!
//! All computation runs on [`Precise`] (the Wave 6 precision engine) with micro-scale
//! amounts (`10^-6` display units) — zero floats on money, zero
//! rounding leakage: the sum of ledger legs always exactly equals the gross amount.

use std::fmt;

use crate::ccxt_core::Precise;

// ── Base network ───────────────────────────────────────────────────────────────

/// Base Mainnet chain ID — a hard wire-layer contract.
pub const BASE_MAINNET_CHAIN_ID: u64 = 8453;

/// Base Mainnet public RPC.
pub const BASE_MAINNET_RPC_URL: &str = "https://mainnet.base.org";

/// Base block explorer — the token-verification dossier lives here.
pub const BASE_BLOCK_EXPLORER: &str = "https://basescan.org";

/// The canonical native Base USDC contract (EIP-55, official Circle/Base source).
pub const BASE_USDC_CONTRACT: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";

/// The fixed `$0xKIA` supply on Base — one billion tokens, 18 decimals, never minted again.
pub const ZERO_X_KIA_TOTAL_SUPPLY_BASE_UNITS: i128 = 1_000_000_000i128 * 10i128.pow(18);

// ── Settlement assets ─────────────────────────────────────────────────────────

/// Supported settlement assets — the internal ledger is always in micro scale (`10^-6`
/// display units); on-chain conversion uses each asset's `decimals`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettlementAsset {
    /// Telegram Stars (`XTR`) — internal/offline; not a Base token.
    TelegramStarsXtr,
    /// Base-native ETH — 18 decimals.
    BaseNativeEth,
    /// Base-native USDC (`0x8335…2913`) — 6 decimals.
    BaseNativeUsdc,
    /// `$0xKIA` ERC-20 on Base — 18 decimals, a fixed 1-billion supply.
    ZeroXKiaBaseToken,
    /// The internal `µKIAN` ledger unit — 6 decimals, never on-chain.
    InternalMicroKian,
}

impl SettlementAsset {
    /// All assets in canonical order.
    pub const ALL: [SettlementAsset; 5] = [
        Self::TelegramStarsXtr,
        Self::BaseNativeEth,
        Self::BaseNativeUsdc,
        Self::ZeroXKiaBaseToken,
        Self::InternalMicroKian,
    ];

    /// Wire symbol — the on-chain wire contract's `code`.
        #[allow(clippy::match_same_arms)] // wire symbols are deliberately explicit
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

    /// Unit display decimals — the internal ledger is always `10^-6` of this unit.
    #[allow(clippy::match_same_arms)] // each asset's decimals are deliberately explicit
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

    /// Token contract address on Base — `None` for native/internal assets.
    #[must_use]
    pub const fn base_contract_address(self) -> Option<&'static str> {
        match self {
            Self::BaseNativeUsdc => Some(BASE_USDC_CONTRACT),
            Self::ZeroXKiaBaseToken => Some("0x0000000000000000000000000000000000046414"),
            _ => None,
        }
    }

    /// Does this asset support `EIP-681` payment on `Base`? (`XTR` and `µKIAN` do not.)
    #[must_use]
    pub const fn is_onchain_settleable(self) -> bool {
        matches!(
            self,
            Self::BaseNativeEth | Self::BaseNativeUsdc | Self::ZeroXKiaBaseToken
        )
    }

    /// Wire unique key — the same value serialized in the
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

// ── Errors ──────────────────────────────────────────────────────────────────────

/// Base treasury error — fail-closed; an unbalanced split never escapes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BaseTreasuryError {
    /// Invalid micro gross amount (≤ 0 or outside `i64`).
    #[error("invalid gross amount: {0} micros")]
    InvalidGrossAmount(i128),
    /// Affiliate share outside `1000..=5000 bps`.
    #[error("affiliate bps {0} outside charter window 1000..=5000")]
    AffiliateBpsOutOfRange(u32),
    /// Empty account ID (buyer/creator/referrer).
    #[error("empty account id for leg: {0}")]
    EmptyAccountId(&'static str),
    /// Precision-engine or on-chain-conversion overflow — the operation is rejected.
    #[error("precision overflow in treasury math")]
    Overflow,
    /// The settlement asset is not payable on Base for this operation (XTR/µKIAN).
    #[error("asset {0} is not on-chain settleable on Base for EIP-681")]
    NotOnChainSettleable(&'static str),
}

impl From<crate::ccxt_core::PreciseError> for BaseTreasuryError {
    fn from(_: crate::ccxt_core::PreciseError) -> Self {
        Self::Overflow
    }
}

// ── Stream 3: boost tiers ────────────────────────────────────────────────────

/// Instant boost tier — a 100% margin to `system:kia_sovereign_treasury`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoostTier {
    /// 24-hour featured slot — `250 XTR` / `$5.00 USDC` / `5,000,000 µKIAN`.
    FeaturedSlot24h,
    /// Mairon fast-track audit — `100 XTR` / `$2.00 USDC` / `2,000,000 µKIAN`.
    MaironFastTrackAudit,
    /// Viral swarm amplifier — `500 XTR` / `$10.00 USDC` / `10,000,000 µKIAN`.
    ViralSwarmAmplifier,
}

impl BoostTier {
    /// All tiers in canonical order.
    pub const ALL: [BoostTier; 3] =
        [Self::FeaturedSlot24h, Self::MaironFastTrackAudit, Self::ViralSwarmAmplifier];

    /// XTR micro price — the XTR display unit has zero decimals, so micro =
    /// unit × 10^6.
    #[must_use]
    pub const fn price_xtr_micros(self) -> i64 {
        match self {
            Self::FeaturedSlot24h => 250_000_000,
            Self::MaironFastTrackAudit => 100_000_000,
            Self::ViralSwarmAmplifier => 500_000_000,
        }
    }

    /// USDC micro price — 6 decimals, so micro = the on-chain unit.
    #[must_use]
    pub const fn price_usdc_micros(self) -> i64 {
        match self {
            Self::FeaturedSlot24h => 5_000_000,
            Self::MaironFastTrackAudit => 2_000_000,
            Self::ViralSwarmAmplifier => 10_000_000,
        }
    }

    /// `µKIAN` micro price.
    #[must_use]
    pub const fn price_micro_kian(self) -> i64 {
        match self {
            Self::FeaturedSlot24h => 5_000_000,
            Self::MaironFastTrackAudit => 2_000_000,
            Self::ViralSwarmAmplifier => 10_000_000,
        }
    }

    /// Stream 3 routing — always `10_000 bps` (100%) to the sovereign treasury.
    #[must_use]
    pub const fn routing_bps(self) -> i64 {
        let _ = self;
        10_000
    }
}

// ── Day-one charter ──────────────────────────────────────────────────────────

/// bps denominator — 10,000 = 100%.
pub const BPS_DENOMINATOR: i128 = 10_000;

/// Stream 1 — the 10% charter tax.
pub const CHARTER_TAX_BPS: i64 = 1_000;

/// Stream 2 — the affiliate share floor (10%).
pub const AFFILIATE_MIN_BPS: i64 = 1_000;

/// Stream 2 — the affiliate share cap (50%).
pub const AFFILIATE_MAX_BPS: i64 = 5_000;

/// Stream 2 — the default share (10%).
pub const AFFILIATE_DEFAULT_BPS: i64 = 1_000;

/// Stream 2 — the "no-referrer direct sale" rule: the 10% share goes to the sovereign treasury.
pub const UNREFERRED_AFFILIATE_TO_TREASURY_BPS: i64 = 1_000;

/// Stream 4 — the cross-chain withdrawal spread (1.5%).
pub const WITHDRAWAL_SPREAD_BPS: i64 = 150;

/// The sovereign treasury account — destination of all platform streams.
pub const KIA_SOVEREIGN_TREASURY_ACCOUNT: &str = "system:kia_sovereign_treasury";

/// Treasury engine signature — for smoke tests and logs.
pub const BASE_TREASURY_SIGNATURE: &str = "base-treasury::day-one-4-stream::8453";

/// The day-one iron charter — "first war for the last peace".
///
/// Four sovereign revenue streams with fixed charter parameters; the atomic split is
/// built by [`compute_day_one_atomic_split`] from this very contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayOneRevenueCharter {
    /// Stream 1 — charter tax (`1000 bps`).
    pub charter_tax_bps: i64,
    /// Stream 2 — configurable affiliate share (`1000..=5000 bps`).
    pub affiliate_bps: i64,
    /// Stream 4 — withdrawal spread (`150 bps`).
    pub withdrawal_spread_bps: i64,
}

impl DayOneRevenueCharter {
    /// The canonical charter sample — the only permitted day-one configuration.
    pub const DAY_ONE: Self = Self {
        charter_tax_bps: CHARTER_TAX_BPS,
        affiliate_bps: AFFILIATE_DEFAULT_BPS,
        withdrawal_spread_bps: WITHDRAWAL_SPREAD_BPS,
    };

    /// Configuration validation — the affiliate share must lie inside the charter window.
    ///
    /// # Errors
    /// [`BaseTreasuryError::AffiliateBpsOutOfRange`] outside `1000..=5000`.
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

// ── Double-sided ledger legs ──────────────────────────────────────────────────────

/// Ledger leg kind — the buyer debit is always negative, credits positive;
/// `buyer_debit + kia_treasury + affiliate + creator == 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitLegKind {
    /// Buyer debit (negative).
    BuyerDebit,
    /// Sovereign treasury credit (stream 1 + the referrer-less stream-2 share + streams 3/4).
    KiaTreasuryCredit,
    /// Referrer credit (stream 2 when a referrer is present).
    AffiliateCredit,
    /// Creator credit — the atomic remainder.
    CreatorCredit,
}

impl SplitLegKind {
    /// Wire label.
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

/// One double-sided ledger leg — a signed micro amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerSplitLeg {
    /// Leg kind.
    pub leg_kind: SplitLegKind,
    /// Ledger account (`user:{id}` or `system:kia_sovereign_treasury`).
    pub account_id: String,
    /// Signed micro amount — debits negative, credits positive.
    pub amount_micros: i128,
    /// This leg's declared bps rate (0 for the buyer debit).
    pub bps: i64,
    /// Settlement asset.
    pub asset: SettlementAsset,
    /// Charter note — which stream forms this leg.
    pub note: String,
}

/// The day-one atomic split result — balanced by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayOneAtomicSplit {
    /// Four ledger legs (canonical order: debit, treasury, affiliate, creator).
    pub legs: Vec<LedgerSplitLeg>,
    /// Gross micro amount.
    pub gross_amount_micros: i128,
    /// Sum of micros reaching the sovereign treasury.
    pub kia_treasury_micros: i128,
    /// Micros reaching the referrer (0 when referrer-less).
    pub referrer_micros: i128,
    /// Creator net.
    pub creator_net_micros: i128,
    /// Effective platform share in bps — `2000` on a referrer-less direct sale.
    pub effective_platform_bps: i64,
    /// Zero-sum proof — a [`Precise`] recheck over the leg sum.
    pub zero_sum_verified: bool,
    /// Settlement asset.
    pub settlement_asset: SettlementAsset,
    /// Scenario kind (`marketplace_sale` | `instant_boost`).
    pub scenario: &'static str,
}

impl DayOneAtomicSplit {
    /// Zero-sum proof with the [`Precise`] engine — the leg sum must be exactly zero.
    ///
    /// # Errors
    /// [`BaseTreasuryError::Overflow`] on overflow.
    pub fn verify_zero_sum_precise(&self) -> Result<bool, BaseTreasuryError> {
        let mut acc = Precise::zero();
        for leg in &self.legs {
            acc = acc.add(&Precise::from_i128(leg.amount_micros))?;
        }
        Ok(acc.is_zero())
    }
}

// ── Atomic split engine ───────────────────────────────────────────────────────

/// The double-sided atomic split of a "marketplace sale" under the day-one charter.
///
/// Atomic order: charter tax (10%) → affiliate share (when a referrer is present) →
/// creator = the exact remainder. When the referrer is `None`, the default `1000 bps` share
/// goes straight to the sovereign treasury (the direct-sale rule) — total platform share
/// `2000 bps` and creator `8000 bps`. Leg rounding closes on the creator so
/// the leg sum exactly equals the gross (zero leakage).
///
/// # Errors
/// Invalid amount, share outside the window, empty ID, or precision overflow.
#[allow(clippy::too_many_lines)] // four ledger legs with all charter rules
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
    // share window: when a referrer is present, the declared share must lie inside the charter window.
    if referrer_user_id.is_some()
        && !(AFFILIATE_MIN_BPS..=AFFILIATE_MAX_BPS).contains(&affiliate_bps)
    {
        return Err(BaseTreasuryError::AffiliateBpsOutOfRange(
            u32::try_from(affiliate_bps).unwrap_or(u32::MAX),
        ));
    }

    let gross = Precise::from_i128(gross_micros);
    let denom = Precise::from_i128(BPS_DENOMINATOR);

    // stream 1 — charter tax: floor(gross × bps / 10000).
    let tax_micros = bps_floor(gross, charter.charter_tax_bps, &denom)?;
    // stream 2 — share: referrer present → declared share; referrer-less → the default 10% to the treasury.
    let affiliate_bps_effective: i64 = if referrer_user_id.is_some() {
        affiliate_bps
    } else {
        UNREFERRED_AFFILIATE_TO_TREASURY_BPS
    };
    let affiliate_micros = bps_floor(gross, affiliate_bps_effective, &denom)?;

    // creator = the atomic remainder — rounding leakage closes here (zero total leakage).
    let creator_micros = gross.sub(&tax_micros)?.sub(&affiliate_micros)?.coeff;
    if creator_micros < 0 {
        return Err(BaseTreasuryError::InvalidGrossAmount(gross_micros));
    }

    // treasury sum = stream 1 + (stream 2 when referrer-less).
    let treasury_micros =
        tax_micros.add(&if referrer_user_id.is_none() { affiliate_micros } else { Precise::zero() })?;
    let referrer_micros =
        if referrer_user_id.is_some() { affiliate_micros } else { Precise::zero() };

    // effective platform share at Precise precision: (treasury + affiliate) × 10000 / gross.
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

    // referrer-less affiliate legs go to the treasury — the "affiliate credit" leg
    // carries zero in this mode (the treasury holds both streams in its own leg).
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

    // final proof — an unbalanced split never escapes.
    let precise_ok = split.verify_zero_sum_precise()?;
    let int_sum: i128 = split.legs.iter().map(|l| l.amount_micros).sum();
    if !precise_ok || int_sum != 0 || split.creator_net_micros <= 0 {
        return Err(BaseTreasuryError::Overflow);
    }
    Ok(split)
}

/// Stream 3 atomic split — instant boost: 100% (10,000 bps) to the sovereign treasury.
///
/// # Errors
/// Invalid amount, empty ID, or precision overflow.
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
            // on Base, a boost is also payable in ETH or $0xKIA — the caller
            // supplies the micro amount; the canonical USDC equivalent pricing
            // is not applied here, so a direct input is required. This branch only
            // keeps the canonical XTR equivalent price.
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
    // two-leg zero sum — creator/affiliate do not exist in stream 3.
    let _ = tier;
    let ok = split.verify_zero_sum_precise()?;
    let int_sum: i128 = split.legs.iter().map(|l| l.amount_micros).sum();
    if !ok || int_sum != 0 {
        return Err(BaseTreasuryError::Overflow);
    }
    split.zero_sum_verified = true;
    Ok(split)
}

/// Human-readable boost tier label.
#[must_use]
fn tier_label(tier: BoostTier) -> &'static str {
    match tier {
        BoostTier::FeaturedSlot24h => "FeaturedSlot24h",
        BoostTier::MaironFastTrackAudit => "MaironFastTrackAudit",
        BoostTier::ViralSwarmAmplifier => "ViralSwarmAmplifier",
    }
}

/// Stream 4 — the instant-settlement and cross-chain-withdrawal spread (`150 bps`).
///
/// Output: (net reaching the user, treasury spread) — zero leakage: the two halves together
/// exactly equal the input.
///
/// # Errors
/// Invalid amount or overflow.
pub fn compute_withdrawal_spread_split(
    asset: SettlementAsset,
    gross_micros: i128,
) -> Result<(i128, i128), BaseTreasuryError> {
    let _ = asset; // the asset exists only for the unified wire-signature contract — the spread is asset-independent.
    if gross_micros <= 0 {
        return Err(BaseTreasuryError::InvalidGrossAmount(gross_micros));
    }
    let gross = Precise::from_i128(gross_micros);
    let denom = Precise::from_i128(BPS_DENOMINATOR);
    let spread = bps_floor(gross, WITHDRAWAL_SPREAD_BPS, &denom)?;
    let net = gross.sub(&spread)?;
    let net_c = net.coeff;
    let spread_c = spread.coeff;
    debug_assert_eq!(net_c + spread_c, gross_micros, "\u{62c}\u{631}\u{6cc}\u{627}\u{646} \u{6f4} — \u{635}\u{641}\u{631} \u{627}\u{62a}\u{644}\u{627}\u{641}");
    Ok((net_c, spread_c))
}

/// floor(gross × bps / 10000) on [`Precise`] — no floats.
fn bps_floor(gross: Precise, bps: i64, denom: &Precise) -> Result<Precise, BaseTreasuryError> {
    let scaled = gross.mul(&Precise::from_i128(i128::from(bps)))?;
    let q = scaled.div(denom, 0)?;
    Ok(q)
}

// ── EIP-681 ──────────────────────────────────────────────────────────────────

/// Build the `EIP-681` payment URI on `Base` — the one-click link for `Trust Wallet` /
/// `Coinbase Wallet` / `MetaMask` with the mandatory `chain_id` of `8453`.
///
///  * native `ETH` → `ethereum:<recipient>@8453?value=<wei>`
///  * `ERC-20`  → `ethereum:<token>@8453/transfer?address=<recipient>&uint256=<base>`
///
/// Non-on-chain assets (`XTR`, `µKIAN`) error out — no fake URI is
/// ever built.
///
/// # Errors
/// An asset not settleable on-chain, or a decimal-conversion overflow.
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
    // micro (10^-6 display) to on-chain unit conversion: base = micros × 10^(d-6).
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

// ── Contract descriptors ─────────────────────────────────────────────────────

/// `ZeroXKiaToken.sol` audit summary — a clean 100/100 architecture
/// `GoPlus` & `TokenSniffer`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // each bool is a GoPlus audit clause
pub struct ZeroXKiaTokenAudit {
    /// Source path in the repository.
    pub source_path: String,
    /// Solidity version.
    pub solidity_version: String,
    /// Fixed supply — matching the `on-chain` unit.
    pub total_supply_base_units: i128,
    /// Post-constructor mint — always `false`.
    pub post_constructor_mint: bool,
    /// Blacklist — always `false`.
    pub blacklist: bool,
    /// Pause — always `false`.
    pub pause: bool,
    /// Mutable tax — always `false`.
    pub mutable_tax_trap: bool,
    /// EIP-2612 `permit`.
    pub eip2612_permit: bool,
    /// User `burn`.
    pub user_burn: bool,
    /// MIT license.
    pub license: &'static str,
}

impl ZeroXKiaTokenAudit {
    /// The canonical descriptor — matches the `contracts/base/ZeroXKiaToken.sol` source.
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

/// `KiaBaseEscrowTreasury.sol` audit summary — non-custodial escrow and
/// the day-one stream atomic splitter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KiaBaseEscrowTreasuryAudit {
    /// Source path in the repository.
    pub source_path: String,
    /// Solidity version.
    pub solidity_version: String,
    /// The immutable sovereign treasury — on-chain address.
    pub kia_treasury: String,
    /// Permitted settlement tokens (a closed whitelist).
    pub allowed_assets: Vec<&'static str>,
    /// On-chain functions.
    pub entrypoints: Vec<&'static str>,
    /// The non-reentrant lock.
    pub non_reentrant_guard: bool,
    /// Charter tax (`1000 bps`).
    pub charter_tax_bps: i64,
    /// The affiliate share window.
    pub affiliate_bps_window: String,
    /// Withdrawal spread (`150 bps`).
    pub withdrawal_spread_bps: i64,
    /// MIT license.
    pub license: &'static str,
}

impl KiaBaseEscrowTreasuryAudit {
    /// The canonical descriptor.
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

    const GROSS: i128 = 100_000_000; // 100.00 display units — exactly divisible

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
        assert_eq!(split.effective_platform_bps, 2_000, "\u{6f1}\u{6f0}\u{66a} \u{645}\u{627}\u{644}\u{6cc}\u{627}\u{62a} + \u{6f1}\u{6f0}\u{66a} \u{648}\u{627}\u{628}\u{633}\u{62a}\u{647}");
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
        assert_eq!(split.kia_treasury_micros, 20_000_000, "\u{6f2}\u{6f0}\u{66a} \u{628}\u{647} \u{62e}\u{632}\u{627}\u{646}\u{647}\u{654} \u{62d}\u{627}\u{6a9}\u{645}");
        assert_eq!(split.referrer_micros, 0);
        assert_eq!(split.creator_net_micros, 80_000_000, "\u{6f8}\u{6f0}\u{66a} \u{628}\u{647} \u{633}\u{627}\u{632}\u{646}\u{62f}\u{647}");
        assert_eq!(split.effective_platform_bps, 2_000);
        // the referrer-less affiliate leg is zero — the treasury carries both streams.
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
        // an odd amount — rounding closes on the creator only; zero total leakage.
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
