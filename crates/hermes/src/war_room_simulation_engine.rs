//! `war_room_simulation_engine` — the founder war room (Wave 10 — Commit #30).
//!
//! The heart of the "massive UI update": a real-time "day 0 → day 1" simulator for
//! the three legendary co-founders who work remotely 24/7 under the leadership of `0xKIA` to commercialize the
//! `ImXLLM/OneRing` repository:
//!
//! - **Elon Musk** ([`FounderPersona::ElonMusk`]) — product velocity, first-principles
//!   engineering and virality on X;
//! - **Changpeng Zhao** ([`FounderPersona::CzBinance`]) — liquidity, the
//!   Base 8453 escrow, CCXT quant and Web3 growth;
//! - **Jeff Bezos** ([`FounderPersona::JeffBezos`]) — the customer flywheel,
//!   unit economics and the "always Day 1" strategy.
//!
//! The hard time rule: **there is no one-year leap** — the simulation advances tick by tick
//! in real seconds ([`advance_single_tick`], a cap of 3600 seconds
//! per tick) and every dollar of revenue booked in the ledger is **causal**: it comes only from
//! the agreed action of the three partners in the private chat and terminal/social network execution;
//! it starts from `$0.00` net revenue and `$33.00` crypto seed + `$15.00`
//! real cash.
//!
//! The Base 8453 split charter applies to every sale:
//! sale without referral → sovereign treasury `2000 bps` (20%) + builder `8000 bps` (80%);
//! sale with referral → marketplace tax of `1000 bps` (10%) + affiliate commission of
//! `1000 bps` + builder `8000 bps`. Four day-one revenue streams ([`RevenueStream`])
//! and four "0-to-100 for zero literacy" steps ([`ZeroLiteracyPipelineStep`])
//! are all covered in this deterministic, network-free, unit-tested engine.

use serde::{Deserialize, Serialize};

/// Founder war room signature — present on every wire payload.
pub const WAR_ROOM_SIGNATURE: &str = "MAIRON-WAR-ROOM";

/// Commercialization target repository — the same 11-crate repo with 826 tests.
pub const GITHUB_REPO: &str = "ImXLLM/OneRing";

/// Short hash of the headline metric on the UI setter — the third-quarter status bar.
pub const GITHUB_COMMIT_SHORT: &str = "282eb68";

/// Base Mainnet chain ID — the day-one charter.
pub const BASE_CHAIN_ID: u64 = 8453;

/// Native Base `USDC` contract address — the day-one escrow.
pub const BASE_USDC_CONTRACT: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";

/// Telegram star glyph — the second day-one payment rail.
pub const TELEGRAM_STARS_CURRENCY: &str = "XTR";

/// The organization's real crypto seed — `$33.00` in Trust Wallet
/// (`Base` / `Solana` / `BNB` / `USDC` / `USDD`).
pub const CRYPTO_SEED_MICROS: i64 = 33_000_000;

/// Real cash seed — `$15.00` in cash.
pub const CASH_SEED_CENTS: i64 = 1_500;

/// Locked paper reserve — `$350.00` untouched (the prudence account).
pub const PAPER_RESERVE_LOCKED_CENTS: i64 = 35_000;

/// Sovereign treasury share on no-referral sales — `2000 bps` = 20%
/// (10% marketplace tax + 10% unattributed affiliate routing).
pub const PLATFORM_TAKE_BPS_UNREFERRED: i64 = 2_000;

/// Perpetual builder share — `8000 bps` = 80%.
pub const CREATOR_PAYOUT_BPS: i64 = 8_000;

/// Marketplace tax on every sale — `1000 bps` = 10%.
pub const MARKETPLACE_TAX_BPS: i64 = 1_000;

/// Featured-slot/visibility-boost stream — `10000 bps` = 100% to the treasury.
pub const FEATURED_BOOST_BPS: i64 = 10_000;

/// Instant-settlement spread (`Base USDC` + `Telegram Stars XTR`) — `150 bps` = 1.5%.
pub const SETTLEMENT_SPREAD_BPS: i64 = 150;

/// Hard cap on time progression per tick — **no jump, not even one hour, is allowed**
/// (the "real unit progress" rule; fake one-year leaps are forbidden).
pub const MAX_TICK_DELTA_SECONDS: i64 = 3_600;

/// Total honest revenue of the canonical scenario's day one — `$9.55`
/// (`9_550_000` `µUSDC`) starting from `$0.00`.
pub const DAY1_TOTAL_EARNED_MICROS: i64 = 9_550_000;

/// Number of canonical "day 0 → day 1" ticks — 24 wealth-building operational steps.
pub const CANONICAL_TICK_COUNT: usize = 24;

/// Maximum allowed characters of the chairman's directive (the wire cap).
pub const MAX_DIRECTIVE_CHARS: usize = 2_000;

/// The Base network with ID 8453 — for wire display.
pub const BASE_CHAIN_LABEL: &str = "Base Mainnet (Chain ID: 8453)";

// ── Founder co-personas ─────────────────────────────────────────────────

/// War-room personas — the three co-founders + the chairman.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FounderPersona {
    /// Elon Musk — product velocity and virality on X.
    ElonMusk,
    /// CZ — liquidity, Base 8453 and Web3 growth.
    CzBinance,
    /// Jeff Bezos — the customer flywheel and "always Day 1".
    JeffBezos,
    /// `0xKIA` — chief architect and chairman of the board.
    Chairman,
}

impl FounderPersona {
    /// The persona's public handle on X.
    #[must_use]
    pub const fn handle(self) -> &'static str {
        match self {
            Self::ElonMusk => "@elonmusk",
            Self::CzBinance => "@cz_binance",
            Self::JeffBezos => "@JeffBezos",
            Self::Chairman => "@0xKIA",
        }
    }

    /// Persian display name.
    #[must_use]
    pub const fn display_name_fa(self) -> &'static str {
        match self {
            Self::ElonMusk => "[FA-PATCH-SLOT] Elon Musk",
            Self::CzBinance => "[FA-PATCH-SLOT] Changpeng Zhao (CZ)",
            Self::JeffBezos => "[FA-PATCH-SLOT] Jeff Bezos",
            Self::Chairman => "[FA-PATCH-SLOT] 0xKIA (Chairman)",
        }
    }

    /// English display name.
    #[must_use]
    pub const fn display_name_en(self) -> &'static str {
        match self {
            Self::ElonMusk => "Elon Musk",
            Self::CzBinance => "Changpeng Zhao (CZ)",
            Self::JeffBezos => "Jeff Bezos",
            Self::Chairman => "0xKIA (Chairman)",
        }
    }

    /// The persona's official mission — Persian.
    #[must_use]
    pub const fn mandate_fa(self) -> &'static str {
        match self {
            Self::ElonMusk => "[FA-PATCH-SLOT] Product velocity, first-principles engineering, the Neural HUD video and going viral on X",
            Self::CzBinance => "[FA-PATCH-SLOT] Liquidity, Base 8453 escrow, the CCXT quant core and global zero-KYC growth",
            Self::JeffBezos => "[FA-PATCH-SLOT] Customer obsession, unit economics, the 5-second simplicity test and the marketplace flywheel",
            Self::Chairman => "[FA-PATCH-SLOT] Chief architect and chairman of the board — the final tactical override",
        }
    }

    /// The persona's official mission — English.
    #[must_use]
    pub const fn mandate_en(self) -> &'static str {
        match self {
            Self::ElonMusk => "Chief Product Velocity — first-principles engineering & X viral distribution",
            Self::CzBinance => "Chief Liquidity — Base 8453 escrow, CCXT quant & zero-KYC Web3 growth",
            Self::JeffBezos => "Chief Flywheel — customer obsession, unit economics & 'Always Day 1'",
            Self::Chairman => "Sovereign Architect & Chairman — final tactical override",
        }
    }

    /// The persona's accent color as hex — the SVG avatar in HTML.
    #[must_use]
    pub const fn accent_hex(self) -> &'static str {
        match self {
            Self::ElonMusk => "#7EF7D2",
            Self::CzBinance => "#59B7FF",
            Self::JeffBezos => "#FFB454",
            Self::Chairman => "#F768BA",
        }
    }

    /// Only the three "24/7 network" co-founders — the chairman is not on the network.
    #[must_use]
    pub const fn is_co_founder(self) -> bool {
        matches!(self, Self::ElonMusk | Self::CzBinance | Self::JeffBezos)
    }

    /// The persona wire ID — `snake_case` for the DTO and the database column.
    #[must_use]
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::ElonMusk => "elon_musk",
            Self::CzBinance => "cz_binance",
            Self::JeffBezos => "jeff_bezos",
            Self::Chairman => "chairman",
        }
    }
}

// ── Four day-one revenue streams ──────────────────────────────────────────────

/// Four day-one revenue streams — plain language, no financial jargon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevenueStream {
    /// No stream — an operational tick with no money.
    None,
    /// The 10% marketplace tax on every app/skill sale.
    MarketplaceTax,
    /// Affiliate routing: on a no-referral sale, an extra 10% goes to the sovereign treasury
    /// (total platform share = 20%; the builder still gets 80%).
    AffiliateRouting,
    /// Featured slot and visibility boost — 100% to the treasury.
    FeaturedBoost,
    /// Instant-settlement spread on `Base USDC` and `Telegram Stars XTR` — 1.5%.
    SettlementSpread,
}

impl RevenueStream {
    /// Stream wire ID.
    #[must_use]
    pub const fn stream_id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::MarketplaceTax => "marketplace_tax",
            Self::AffiliateRouting => "affiliate_routing",
            Self::FeaturedBoost => "featured_boost",
            Self::SettlementSpread => "settlement_spread",
        }
    }

    /// The plain-language Persian label.
    #[must_use]
    pub const fn label_fa(self) -> &'static str {
        match self {
            Self::None => "[FA-PATCH-SLOT] no revenue in this step",
            Self::MarketplaceTax => "[FA-PATCH-SLOT] 10% marketplace tax on every sale",
            Self::AffiliateRouting => "[FA-PATCH-SLOT] affiliate routing — the extra 10% on unreferred sales (20% platform total)",
            Self::FeaturedBoost => "[FA-PATCH-SLOT] featured slot and visibility boost — 100% to the treasury",
            Self::SettlementSpread => "[FA-PATCH-SLOT] instant settlement spread (Base USDC + Stars XTR) — 1.5%",
        }
    }

    /// The plain-language English label.
    #[must_use]
    pub const fn label_en(self) -> &'static str {
        match self {
            Self::None => "no revenue in this step",
            Self::MarketplaceTax => "10% marketplace tax on every AI app/skill sold",
            Self::AffiliateRouting => "affiliate engine — unreferred extra 10% (20% total platform take)",
            Self::FeaturedBoost => "featured slot & instant visibility boosts — 100% to treasury",
            Self::SettlementSpread => "1.5% instant settlement spread (Base USDC & Telegram Stars XTR)",
        }
    }

    /// Take rate in bps — zero for `None`.
    #[must_use]
    pub const fn take_bps(self) -> i64 {
        match self {
            Self::None => 0,
            Self::MarketplaceTax | Self::AffiliateRouting => MARKETPLACE_TAX_BPS,
            Self::FeaturedBoost => FEATURED_BOOST_BPS,
            Self::SettlementSpread => SETTLEMENT_SPREAD_BPS,
        }
    }
}

// ── Day-0 operational stages ───────────────────────────────────────────────────

/// Operational stages of the day-0 → day-1 timeline — unique and ascending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    /// Repository warm-up: tests, contracts, kernel health.
    Ignition,
    /// Product simplicity for zero literacy — the 9-year-old test.
    ProductClarity,
    /// The viral X launch with the neural HUD video.
    ViralLaunch,
    /// Opening the payment rails: Stars XTR + Base USDC.
    PaymentRails,
    /// Onboarding the first builders on the marketplace.
    CreatorOnboarding,
    /// A live revenue stream under the 20/80 charter.
    RevenueFlow,
    /// Vibe-Trading quantum operations on CCXT.
    QuantOps,
    /// The day-one seal — an honest summary and "always Day 1".
    DayOneSealed,
}

impl PipelineStage {
    /// Stage wire ID.
    #[must_use]
    pub const fn stage_id(self) -> &'static str {
        match self {
            Self::Ignition => "ignition",
            Self::ProductClarity => "product_clarity",
            Self::ViralLaunch => "viral_launch",
            Self::PaymentRails => "payment_rails",
            Self::CreatorOnboarding => "creator_onboarding",
            Self::RevenueFlow => "revenue_flow",
            Self::QuantOps => "quant_ops",
            Self::DayOneSealed => "day_one_sealed",
        }
    }

    /// The stage's Persian label.
    #[must_use]
    pub const fn label_fa(self) -> &'static str {
        match self {
            Self::Ignition => "[FA-PATCH-SLOT] Ignition — treasury health",
            Self::ProductClarity => "[FA-PATCH-SLOT] Product clarity — the 9-year-old test",
            Self::ViralLaunch => "[FA-PATCH-SLOT] Viral launch on X",
            Self::PaymentRails => "[FA-PATCH-SLOT] Payment rails — XTR + USDC",
            Self::CreatorOnboarding => "[FA-PATCH-SLOT] Creator onboarding",
            Self::RevenueFlow => "[FA-PATCH-SLOT] Revenue flow — the 20/80 charter",
            Self::QuantOps => "[FA-PATCH-SLOT] Vibe-Trading quant operations",
            Self::DayOneSealed => "[FA-PATCH-SLOT] Day 1 sealed",
        }
    }

    /// The stage's English label.
    #[must_use]
    pub const fn label_en(self) -> &'static str {
        match self {
            Self::Ignition => "Ignition — repository health",
            Self::ProductClarity => "Product clarity — the 9-year-old test",
            Self::ViralLaunch => "Viral launch on X",
            Self::PaymentRails => "Payment rails — XTR + USDC",
            Self::CreatorOnboarding => "Creator onboarding",
            Self::RevenueFlow => "Revenue flow — the 20/80 charter",
            Self::QuantOps => "Vibe-Trading quant operations",
            Self::DayOneSealed => "Day 1 sealed",
        }
    }

    /// The stage's ascending rank — for the monotonic progression guard.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Ignition => 0,
            Self::ProductClarity => 1,
            Self::ViralLaunch => 2,
            Self::PaymentRails => 3,
            Self::CreatorOnboarding => 4,
            Self::RevenueFlow => 5,
            Self::QuantOps => 6,
            Self::DayOneSealed => 7,
        }
    }
}

// ── "0 to 100" steps for zero literacy ───────────────────────────────────────

/// The four steps of the "0-to-100" pipeline — clear enough for a 9-year-old.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZeroLiteracyPipelineStep {
    /// Step A — a regular user signs up with one sentence or a voice note.
    StepAUserIntent,
    /// Step B — the factory of 3 avatars and 5 AI models.
    StepBAvatarFactory,
    /// Step C — the storefront and social media with 3D video.
    StepCStoreAndSocial,
    /// Step D — automatic money split in the wallet (80% builder / 20% organization).
    StepDAutoSplit,
}

impl ZeroLiteracyPipelineStep {
    /// Step wire ID.
    #[must_use]
    pub const fn step_id(self) -> &'static str {
        match self {
            Self::StepAUserIntent => "step_a_user_intent",
            Self::StepBAvatarFactory => "step_b_avatar_factory",
            Self::StepCStoreAndSocial => "step_c_store_and_social",
            Self::StepDAutoSplit => "step_d_auto_split",
        }
    }

    /// The step's Persian title.
    #[must_use]
    pub const fn title_fa(self) -> &'static str {
        match self {
            Self::StepAUserIntent => "[FA-PATCH-SLOT] A — a regular user signs in",
            Self::StepBAvatarFactory => "[FA-PATCH-SLOT] B — the factory of 3 avatars and 5 models",
            Self::StepCStoreAndSocial => "[FA-PATCH-SLOT] C — storefront and social media",
            Self::StepDAutoSplit => "[FA-PATCH-SLOT] D — the wallet splits the money automatically",
        }
    }

    /// The step's English title.
    #[must_use]
    pub const fn title_en(self) -> &'static str {
        match self {
            Self::StepAUserIntent => "Step A — a normal user walks in",
            Self::StepBAvatarFactory => "Step B — the 3-avatar & 5-model factory",
            Self::StepCStoreAndSocial => "Step C — store & social media",
            Self::StepDAutoSplit => "Step D — automatic wallet split",
        }
    }

    /// The step's Persian body — one long, clear sentence.
    #[must_use]
    pub const fn body_fa(self) -> &'static str {
        match self {
            Self::StepAUserIntent => "[FA-PATCH-SLOT] The user states what they want in one plain sentence — even by voice or through the big ToySpark buttons; no code, no complex forms.",
            Self::StepBAvatarFactory => "[FA-PATCH-SLOT] Alpha (designer), the Command Center (engineer) and Mairon (the strict judge with a 100/100 standard) build the product or trading bot in seconds with 5 AI models.",
            Self::StepCStoreAndSocial => "[FA-PATCH-SLOT] The product ships with a 3D Neural HUD video on X and Telegram; the marketplace storefront opens instantly.",
            Self::StepDAutoSplit => "[FA-PATCH-SLOT] The customer pays with Telegram Stars or crypto (Base USDC); the smart contract splits 80% to the creator and 20% to the organization wallet in the same second.",
        }
    }

    /// The step's English body.
    #[must_use]
    pub const fn body_en(self) -> &'static str {
        match self {
            Self::StepAUserIntent => "The user says what they want with one plain sentence — even via voice or big ToySpark buttons; no code, no complex forms.",
            Self::StepBAvatarFactory => "Alpha (designer), the Command Room (engineer) and Mairon (the strict judge with the 100/100 standard) build the product or trading bot in seconds with 5 AI models.",
            Self::StepCStoreAndSocial => "The product is published on X and Telegram with a 3D Neural HUD video; the marketplace storefront opens instantly.",
            Self::StepDAutoSplit => "The customer pays with Telegram Stars or crypto (Base USDC); the smart contract deposits 80% to the creator and 20% to the organization wallet in the same second!",
        }
    }
}

// ── Locale ────────────────────────────────────────────────────────────────────

/// The wire locale — Persian (RTL) or English (LTR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// Persian — right-to-left.
    Fa,
    /// English — left-to-right.
    En,
}

impl Locale {
    /// The wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fa => "fa",
            Self::En => "en",
        }
    }

    /// Text direction — `rtl` or `ltr`.
    #[must_use]
    pub const fn direction(self) -> &'static str {
        match self {
            Self::Fa => "rtl",
            Self::En => "ltr",
        }
    }

    /// Tolerant parsing — any other character falls back to `En` (legal fallback).
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        if raw.eq_ignore_ascii_case("fa") || raw.eq_ignore_ascii_case("fa-ir") {
            Self::Fa
        } else {
            Self::En
        }
    }
}

// ── Base 8453 split charter ─────────────────────────────────────────────────

/// The exact 20/80 charter result on a sales volume — every component in `µUSDC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharterSplit {
    /// The sovereign treasury share — equal to the organization wallet delta.
    pub treasury_delta_micros: i64,
    /// The builder payout — always `8000 bps` of the volume.
    pub creator_payout_micros: i64,
    /// The affiliate payout — referred sales only (`1000 bps`).
    pub affiliate_payout_micros: i64,
    /// The marketplace tax share (`1000 bps`) — part of the treasury.
    pub marketplace_tax_micros: i64,
    /// The unattributed affiliate routing share (`1000 bps`) — no-referral sales only.
    pub affiliate_routing_micros: i64,
}

/// Applying the Base 8453 charter to a sales volume — deterministic
/// "rock-solid" split functions: the builder always gets `8000 bps`; without a referral the treasury gets `2000 bps`;
/// with a referral the treasury gets `1000 bps` and the affiliate `1000 bps`. The sum of components == volume.
#[must_use]
pub fn charter_split(volume_micros: i64, referred: bool) -> CharterSplit {
    let creator = volume_micros * CREATOR_PAYOUT_BPS / 10_000;
    if referred {
        let tax = volume_micros * MARKETPLACE_TAX_BPS / 10_000;
        CharterSplit {
            treasury_delta_micros: tax,
            creator_payout_micros: creator,
            affiliate_payout_micros: tax,
            marketplace_tax_micros: tax,
            affiliate_routing_micros: 0,
        }
    } else {
        let tax = volume_micros * MARKETPLACE_TAX_BPS / 10_000;
        let routing = volume_micros * MARKETPLACE_TAX_BPS / 10_000;
        CharterSplit {
            treasury_delta_micros: tax + routing,
            creator_payout_micros: creator,
            affiliate_payout_micros: 0,
            marketplace_tax_micros: tax,
            affiliate_routing_micros: routing,
        }
    }
}

/// Applying the featured-boost stream — `10000 bps` means 100% of the volume to the treasury.
#[must_use]
pub fn featured_boost_split(volume_micros: i64) -> CharterSplit {
    CharterSplit {
        treasury_delta_micros: volume_micros,
        creator_payout_micros: 0,
        affiliate_payout_micros: 0,
        marketplace_tax_micros: 0,
        affiliate_routing_micros: 0,
    }
}

/// Applying the instant-settlement spread — `150 bps` of the settlement volume to the treasury.
#[must_use]
pub fn settlement_spread_split(volume_micros: i64) -> CharterSplit {
    let spread = volume_micros * SETTLEMENT_SPREAD_BPS / 10_000;
    CharterSplit {
        treasury_delta_micros: spread,
        creator_payout_micros: 0,
        affiliate_payout_micros: 0,
        marketplace_tax_micros: 0,
        affiliate_routing_micros: 0,
    }
}

/// Human formatting of `µUSDC` into a dollar string — `$9.55` / `$0.00`.
#[must_use]
pub fn format_usd_micros(micros: i64) -> String {
    format!("${}.{:02}", micros / 1_000_000, (micros % 1_000_000) / 10_000)
}

/// Human formatting of cents into a dollar string — `$15.00`.
#[must_use]
pub fn format_usd_cents(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

// ── Organization wallet ledger ────────────────────────────────────────────────────────

/// The organization wallet ledger — the honest money state at every moment.
///
/// Real starting point: `$33.00` crypto seed ([`CRYPTO_SEED_MICROS`]) + `$15.00` cash
/// ([`CASH_SEED_CENTS`]) + `$350.00` locked paper reserve
/// ([`PAPER_RESERVE_LOCKED_CENTS`]) and `$0.00` net platform revenue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationWalletLedger {
    /// Crypto seed — `33_000_000` `µUSDC` = `$33.00`.
    pub crypto_seed_micros: i64,
    /// Cash seed — `1_500` cents = `$15.00`.
    pub cash_seed_cents: i64,
    /// Locked paper reserve — `35_000` cents = `$350.00` (untouched).
    pub paper_reserve_locked_cents: i64,
    /// Net platform revenue — starts at `$0.00`.
    pub earned_revenue_micros: i64,
    /// Total 80% payouts to builders.
    pub creator_payouts_micros: i64,
    /// Total affiliate payouts on referred sales.
    pub affiliate_payouts_micros: i64,
    /// Stream bucket 1 — marketplace tax.
    pub stream_marketplace_tax_micros: i64,
    /// Stream bucket 2 — unattributed affiliate routing.
    pub stream_affiliate_routing_micros: i64,
    /// Stream bucket 3 — featured boost.
    pub stream_featured_boost_micros: i64,
    /// Stream bucket 4 — settlement spread.
    pub stream_settlement_spread_micros: i64,
    /// Active users of the day.
    pub active_users: u64,
    /// Visit funnel.
    pub funnel_visits: u64,
    /// Signup funnel.
    pub funnel_signups: u64,
    /// Purchase funnel.
    pub funnel_purchases: u64,
}

impl OrganizationWalletLedger {
    /// The day-0, second-0 starting ledger — honest, with no fake money.
    #[must_use]
    pub fn seed() -> Self {
        Self {
            crypto_seed_micros: CRYPTO_SEED_MICROS,
            cash_seed_cents: CASH_SEED_CENTS,
            paper_reserve_locked_cents: PAPER_RESERVE_LOCKED_CENTS,
            earned_revenue_micros: 0,
            creator_payouts_micros: 0,
            affiliate_payouts_micros: 0,
            stream_marketplace_tax_micros: 0,
            stream_affiliate_routing_micros: 0,
            stream_featured_boost_micros: 0,
            stream_settlement_spread_micros: 0,
            active_users: 0,
            funnel_visits: 0,
            funnel_signups: 0,
            funnel_purchases: 0,
        }
    }

    /// Total operational balance — crypto seed + earned revenue (`µUSDC`).
    #[must_use]
    pub fn total_operational_usdc_micros(&self) -> i64 {
        self.crypto_seed_micros + self.earned_revenue_micros
    }

    /// Sum of the four revenue-stream buckets — must always == `earned_revenue_micros`.
    #[must_use]
    pub fn streams_sum_micros(&self) -> i64 {
        self.stream_marketplace_tax_micros
            + self.stream_affiliate_routing_micros
            + self.stream_featured_boost_micros
            + self.stream_settlement_spread_micros
    }

    /// Net revenue as a dollar string — `$9.55`.
    #[must_use]
    pub fn earned_revenue_usd_string(&self) -> String {
        format_usd_micros(self.earned_revenue_micros)
    }

    /// Crypto seed as a dollar string — `$33.00`.
    #[must_use]
    pub fn crypto_seed_usd_string(&self) -> String {
        format_usd_micros(self.crypto_seed_micros)
    }

    /// Cash seed as a dollar string — `$15.00`.
    #[must_use]
    pub fn cash_seed_usd_string(&self) -> String {
        format_usd_cents(self.cash_seed_cents)
    }

    /// Locked paper reserve as a dollar string — `$350.00`.
    #[must_use]
    pub fn paper_reserve_usd_string(&self) -> String {
        format_usd_cents(self.paper_reserve_locked_cents)
    }

    /// Apply a charter cut directly to the ledger — the only path that moves money.
    ///
    /// Stream buckets 1 and 2 are filled from the real cut components (`marketplace_tax_micros` +
    /// `affiliate_routing_micros`) so a no-referral sale lands exactly
    /// "10% + 10%" in the two correct buckets; buckets 3 and 4 take the entire
    /// `treasury_delta_micros`.
    pub fn apply_charter_split(&mut self, split: &CharterSplit, stream: RevenueStream) {
        self.earned_revenue_micros += split.treasury_delta_micros;
        self.creator_payouts_micros += split.creator_payout_micros;
        self.affiliate_payouts_micros += split.affiliate_payout_micros;
        self.stream_marketplace_tax_micros += split.marketplace_tax_micros;
        self.stream_affiliate_routing_micros += split.affiliate_routing_micros;
        match stream {
            RevenueStream::FeaturedBoost => {
                self.stream_featured_boost_micros += split.treasury_delta_micros;
            }
            RevenueStream::SettlementSpread => {
                self.stream_settlement_spread_micros += split.treasury_delta_micros;
            }
            RevenueStream::MarketplaceTax
            | RevenueStream::AffiliateRouting
            | RevenueStream::None => {}
        }
    }
}

// ── Tick event ───────────────────────────────────────────────────────────────

/// One real operational "day 0 → day 1" step — the same `"FounderWarRoomTick"` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationTickEvent {
    /// Tick number — `0..24`.
    pub tick_index: u32,
    /// Simulation seconds from `Day 0, 00:00:00` — no jump beyond the 3600 cap.
    pub simulation_second: i64,
    /// The tick's speaker.
    pub speaker: FounderPersona,
    /// Persian message — natural and in character.
    pub message_fa: String,
    /// English message — full parity with the Persian.
    pub message_en: String,
    /// The exact terminal command on `ImXLLM/OneRing` — when executed.
    pub terminal_cmd: Option<String>,
    /// The live social post (X/Telegram) as JSON — when published.
    pub social_post_json: Option<String>,
    /// The treasury delta in this tick (`µUSDC`) — zero for money-free ticks.
    pub wallet_delta_usdc_micros: i64,
    /// Total operational balance after this tick (the $33 seed + cumulative revenue).
    pub wallet_total_usdc_micros: i64,
    /// The active stage of the operations line.
    pub active_pipeline_stage: PipelineStage,
    /// This tick's active revenue stream.
    pub revenue_stream: RevenueStream,
    /// Gross sales/boost/settlement volume in this tick (`µUSDC`) — the charter-cut base.
    pub sale_volume_micros: i64,
    /// The 80% builder payout in this tick.
    pub creator_payout_micros: i64,
    /// Was the sale referred?
    pub referred: bool,
}

/// The tick's internal signature — fixed scenario data (no derived figures).
struct TickSpec {
    second: i64,
    speaker: FounderPersona,
    fa: &'static str,
    en: &'static str,
    cmd: Option<&'static str>,
    social: Option<&'static str>,
    stage: PipelineStage,
    stream: RevenueStream,
    volume_micros: i64,
    referred: bool,
    users_delta: u64,
    visits_delta: u64,
    signups_delta: u64,
    purchases_delta: u64,
}

/// The canonical 24-tick "day 0 → day 1" scenario — the single source of truth
/// for the server (seed), the tests and the HTML app (`tools/war-room/render_war_room.py`).
///
/// Every revenue dollar is **causal**: first the three-partner chat, then execution (terminal/social
/// network), then money. Final revenue total: `$9.55` — honest, with no leap.
const DAY0_TICKS: [TickSpec; CANONICAL_TICK_COUNT] = [
    // ── Tick 0 — day 0, second 0 ──
    TickSpec {
        second: 0,
        speaker: FounderPersona::JeffBezos,
        fa: "[FA-PATCH-SLOT] Day 1! The three of us partnered 24/7 remotely under 0xKIA's chairmanship. We start from the ImXLLM/OneRing repo: 11 Rust crates, 826 passing tests, 9 delivered waves. Real capital? $33 crypto, $15 cash, $350 locked paper. Platform revenue is exactly $0.00 — and it must come from real work. Always Day 1.",
        en: "Day 1! The three of us just partnered 24/7 remotely under 0xKIA. We start from the ImXLLM/OneRing repo: 11 Rust crates, 826 passing tests, 9 delivered waves. Real capital? $33 crypto, $15 cash, $350 locked paper. Platform revenue is exactly $0.00 — and it must come from real work. It is always Day 1.",
        cmd: None,
        social: None,
        stage: PipelineStage::Ignition,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 0,
        visits_delta: 0,
        signups_delta: 0,
        purchases_delta: 0,
    },
    // ── Tick 1 ──
    TickSpec {
        second: 3600,
        speaker: FounderPersona::ElonMusk,
        fa: "[FA-PATCH-SLOT] First we prove the repo is healthy. I run the full workspace test suite — 826 passed, zero failures, zero clippy. This is our credibility furnace; without it no virality means anything.",
        en: "First we prove the repo is healthy. Running the full workspace test suite — 826 passed, zero failures, zero clippy. That is our credibility furnace; nothing viral means anything without it.",
        cmd: Some("cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings"),
        social: None,
        stage: PipelineStage::Ignition,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 0,
        visits_delta: 0,
        signups_delta: 0,
        purchases_delta: 0,
    },
    // ── Tick 2 ──
    TickSpec {
        second: 7200,
        speaker: FounderPersona::CzBinance,
        fa: "[FA-PATCH-SLOT] I verify the money rails before any marketing: the Base contracts with Chain ID 8453 and native USDC 0x8335...2913 are confirmed on mainnet; Telegram Stars (XTR) is ready too. Funds SAFU — every sale pays 80% creator, the rest per charter.",
        en: "I verify the money rails before any marketing: Base contracts with Chain ID 8453 and native USDC 0x8335...2913 verified on mainnet; Telegram Stars (XTR) ready too. Funds are SAFU — every sale pays 80% to the creator, the rest per the charter.",
        cmd: Some("forge verify-contract 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 --chain-id 8453 --rpc-url https://mainnet.base.org"),
        social: None,
        stage: PipelineStage::Ignition,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 0,
        visits_delta: 0,
        signups_delta: 0,
        purchases_delta: 0,
    },
    // ── Tick 3 ──
    TickSpec {
        second: 10800,
        speaker: FounderPersona::JeffBezos,
        fa: "[FA-PATCH-SLOT] Before going viral I run the 9-year-old test: a regular user must understand what the system does in 5 seconds. I write the 0-to-100 pipeline — A: the user says what they want; B: the factory of 3 avatars and 5 models builds it; C: the product goes to storefront and social; D: money splits 80/20 automatically. If these four steps are not obvious, no button saves us.",
        en: "Before going viral I run the 9-year-old test: a normal user must understand the system in 5 seconds. I write the zero-to-100 pipeline — A: the user says what they want; B: the 3-avatar & 5-model factory builds it; C: the product hits the store and social; D: money auto-splits 80/20. If those four steps are not obvious, no button will save us.",
        cmd: None,
        social: None,
        stage: PipelineStage::ProductClarity,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 0,
        visits_delta: 0,
        signups_delta: 0,
        purchases_delta: 0,
    },
    // ── Tick 4 ──
    TickSpec {
        second: 14400,
        speaker: FounderPersona::ElonMusk,
        fa: "[FA-PATCH-SLOT] I build the Neural HUD video — the 180-node Fibonacci sphere mesh, deterministic seekTo, WebAudio synth. This is our viral art: 60 seconds that show 11 crates and the Base treasury at a glance. Our own repo tooling, no external service.",
        en: "Building the Neural HUD video — the 180-node Fibonacci sphere mesh, deterministic seekTo, WebAudio synth. This is our viral art: 60 seconds that show 11 crates and the Base treasury at a glance. Built with the repo's own tool, zero external services.",
        cmd: Some("python3 tools/neural-hud/render_neural_hud.py html --preset kia-monopoly-core --aspect 9:16 --out hud_monopoly.html"),
        social: None,
        stage: PipelineStage::Ignition,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 0,
        visits_delta: 0,
        signups_delta: 0,
        purchases_delta: 0,
    },
    // ── Tick 5 ──
    TickSpec {
        second: 18000,
        speaker: FounderPersona::ElonMusk,
        fa: "[FA-PATCH-SLOT] The first X thread is live — HUD video plus how to understand this system in 5 seconds. The view curve starts from zero; let the data speak.",
        en: "First X thread is live — the HUD video plus 'how to understand this system in 5 seconds'. View curve starts at zero; let the data speak.",
        cmd: None,
        social: Some(r#"{"platform":"x","handle":"@elonmusk","kind":"launch_thread","text_en":"We built OneRing: 11 Rust crates, 826 tests, a 3D Neural HUD, and a wallet that auto-splits 80/20 on every sale. A 9-year-old can understand it in 5 seconds. Day 0 starts now.","text_fa":"We built the OneRing system: 11 Rust crates, 826 tests, a 3D neural HUD, and a wallet that auto-splits every sale 80/20. A 9-year-old gets it in 5 seconds. Day zero starts this instant.","views":0,"likes":0}"#),
        stage: PipelineStage::ViralLaunch,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 0,
        visits_delta: 12_400,
        signups_delta: 0,
        purchases_delta: 0,
    },
    // ── Tick 6 ──
    TickSpec {
        second: 21600,
        speaker: FounderPersona::CzBinance,
        fa: "[FA-PATCH-SLOT] The Day-1 payment gateway is open: the Telegram mini-app with Stars (XTR) for instant purchases, and Base USDC for crypto settlement. The 1.5% settlement spread is transparent and the visibility boost goes 100% to the treasury. Rails green.",
        en: "Day-1 payment gateway is open: the Telegram mini-app with Stars (XTR) for instant purchases, and Base USDC for crypto settlement. The 1.5% settlement spread is transparent and featured boosts send 100% to the treasury. Bridges green.",
        cmd: Some("curl -s https://kia-api.up.railway.app/api/v1/monopoly/charter-and-base | head -c 400"),
        social: None,
        stage: PipelineStage::PaymentRails,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 0,
        visits_delta: 3_100,
        signups_delta: 0,
        purchases_delta: 0,
    },
    // ── Tick 7 ──
    TickSpec {
        second: 25200,
        speaker: FounderPersona::JeffBezos,
        fa: "[FA-PATCH-SLOT] First creator onboarded: a storytelling teacher building children's audiobooks with ToySpark mode. The product is Moon Journey, priced at $3. I drew the charter for her: 80% creator, 20% platform on every sale. The flywheel turns from here.",
        en: "First creator onboarded: a storybook teacher building kids' audiobooks in ToySpark mode. Her product is 'The Moon Trip', priced at $3.00. I drew the charter for her: 80% hers on every sale, 20% platform. The flywheel starts turning right here.",
        cmd: None,
        social: None,
        stage: PipelineStage::CreatorOnboarding,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 38,
        visits_delta: 2_200,
        signups_delta: 41,
        purchases_delta: 0,
    },
    // ── Tick 8 — first real sale ──
    TickSpec {
        second: 28800,
        speaker: FounderPersona::JeffBezos,
        fa: "[FA-PATCH-SLOT] The first real sale in platform history: Moon Journey — $3.00 via Telegram Stars, unreferred. The contract executed in the same second: $2.40 (80%) to the creator, $0.60 (20%) to the sovereign treasury. Platform revenue moved from zero to $0.60 — space exploration has begun!",
        en: "The first real sale in platform history: 'The Moon Trip' — $3.00 via Telegram Stars, unreferred. The contract executed in the same second: $2.40 (80%) to the creator, $0.60 (20%) to the sovereign treasury. Platform revenue moved from $0.00 to $0.60 — liftoff!",
        cmd: None,
        social: None,
        stage: PipelineStage::RevenueFlow,
        stream: RevenueStream::MarketplaceTax,
        volume_micros: 3_000_000,
        referred: false,
        users_delta: 53,
        visits_delta: 6_800,
        signups_delta: 96,
        purchases_delta: 1,
    },
    // ── Tick 9 ──
    TickSpec {
        second: 32400,
        speaker: FounderPersona::CzBinance,
        fa: "[FA-PATCH-SLOT] The first Base USDC settlement is done too: a $5.00 sale with an affiliate referral. Per charter: $4.00 creator, $0.50 marketplace tax to the treasury, $0.50 affiliate commission. The tx hash is recorded on Base — everything auditable.",
        en: "The first Base USDC settlement also landed: a $5.00 sale with an affiliate referral. Per the charter: $4.00 creator, $0.50 marketplace tax to the treasury, $0.50 affiliate commission. The tx hash is recorded on Base — everything auditable.",
        cmd: None,
        social: None,
        stage: PipelineStage::RevenueFlow,
        stream: RevenueStream::MarketplaceTax,
        volume_micros: 5_000_000,
        referred: true,
        users_delta: 61,
        visits_delta: 3_400,
        signups_delta: 77,
        purchases_delta: 1,
    },
    // ── Tick 10 ──
    TickSpec {
        second: 36000,
        speaker: FounderPersona::ElonMusk,
        fa: "[FA-PATCH-SLOT] The X thread went hot: 48,900 views, 3.1k likes, hundreds of replies. Top comment: does it really split 80/20? — yes, with an on-chain hash. Traffic floods the landing; we watch conversion, not just views.",
        en: "The X thread caught fire: 48,900 views, 3.1k likes, hundreds of replies. Top comment: 'does it really split 80/20?' — yes, with an on-chain hash. Traffic is pouring into the landing; we watch conversion rate, not just views.",
        cmd: None,
        social: Some(r#"{"platform":"x","handle":"@elonmusk","kind":"viral_milestone","text_en":"48,900 views in 2 hours. The 9-year-old test passed in public: a kid built a storybook, a bot sold it, the wallet split 80/20 instantly. Funds are auditable on Base 8453.","text_fa":"48,900 views in 2 hours. The 9-year-old test passed in public: a child built a storybook, a bot sold it, and the wallet split 80/20 on the spot. Everything is auditable on Base 8453.","views":48900,"likes":3140}"#),
        stage: PipelineStage::ViralLaunch,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 200,
        visits_delta: 18_600,
        signups_delta: 62,
        purchases_delta: 0,
    },
    // ── Tick 11 ──
    TickSpec {
        second: 39600,
        speaker: FounderPersona::JeffBezos,
        fa: "[FA-PATCH-SLOT] First visibility boost sold: the Moon Journey creator paid $2.00 for the featured marketplace slot — 100% goes to the treasury, per revenue stream three. It stays on top of the catalog for 24 hours; we compare conversion data.",
        en: "First visibility boost sold: the 'Moon Trip' creator paid $2.00 for a marketplace featured slot — 100% of it goes to the treasury, per revenue stream #3. She stays on top of the catalog for 24 hours; we compare conversion data.",
        cmd: None,
        social: None,
        stage: PipelineStage::RevenueFlow,
        stream: RevenueStream::FeaturedBoost,
        volume_micros: 2_000_000,
        referred: false,
        users_delta: 22,
        visits_delta: 2_900,
        signups_delta: 18,
        purchases_delta: 0,
    },
    // ── Tick 12 ──
    TickSpec {
        second: 43200,
        speaker: FounderPersona::CzBinance,
        fa: "[FA-PATCH-SLOT] I opened the first Vibe-Trading quant contest: CCXT strategies on the causal ledger, Sharpe >= 1.20 for the champion seat. Creators sell trading bots and we take only eighty-twenty. Live for everyone, no KYC.",
        en: "I opened the first Vibe-Trading quant competition: CCXT strategies on the causal ledger, Sharpe >= 1.20 for the champion seat. Builders sell trading bots and we only ever take eighty-twenty. Live for everyone, zero KYC.",
        cmd: Some("node ccxt-feed.js binance BTC/USDC --depth 5 --json"),
        social: None,
        stage: PipelineStage::QuantOps,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 47,
        visits_delta: 4_100,
        signups_delta: 33,
        purchases_delta: 0,
    },
    // ── Tick 13 ──
    TickSpec {
        second: 46800,
        speaker: FounderPersona::ElonMusk,
        fa: "[FA-PATCH-SLOT] I recorded the signature Grok UX demo: Fun mode, the visible-thinking accordion, and the follow-up chips. A video reply to the most frequent question: how does it work with one sentence? — the answer takes 9 seconds.",
        en: "I recorded the Grok signature UX demo: Fun Mode, the visible think accordion, and ghost follow-up chips. A video reply to the most repeated question: 'how does this work with one sentence?' — answer in 9 seconds.",
        cmd: None,
        social: Some(r#"{"platform":"x","handle":"@ImXforevr","kind":"ux_demo","text_en":"xA answers with a visible thinking accordion, live citations, and a split artifact canvas. Fun Mode makes a 9-year-old laugh; Mairon mode ships 100/100. No API key needed.","text_fa":"xA answers with a visible thinking accordion, live citations, and a split artifact canvas. Fun Mode makes a 9-year-old laugh; Mairon mode ships 100/100. No key required.","views":21500,"likes":1870}"#),
        stage: PipelineStage::ViralLaunch,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 184,
        visits_delta: 7_400,
        signups_delta: 51,
        purchases_delta: 0,
    },
    // ── Tick 14 ──
    TickSpec {
        second: 50400,
        speaker: FounderPersona::CzBinance,
        fa: "[FA-PATCH-SLOT] We closed the morning instant settlement: $10.00 of settlement volume on Base USDC and Stars — the 1.5% spread means $0.15 to the treasury, transparent on the dashboard. Revenue stream four is live; small but real and repeatable.",
        en: "Closed the morning instant settlement: $10.00 of settlement volume across Base USDC and Stars — the 1.5% spread means $0.15 to the treasury, transparent on the dashboard. Revenue stream #4 is live; small but real and repeatable.",
        cmd: None,
        social: None,
        stage: PipelineStage::RevenueFlow,
        stream: RevenueStream::SettlementSpread,
        volume_micros: 10_000_000,
        referred: false,
        users_delta: 0,
        visits_delta: 0,
        signups_delta: 0,
        purchases_delta: 0,
    },
    // ── Tick 15 ──
    TickSpec {
        second: 54000,
        speaker: FounderPersona::JeffBezos,
        fa: "[FA-PATCH-SLOT] Second creator onboarded: an automation developer selling DAG templates. We rebuilt the seller page together — the Buy button above the fold, zero technical jargon. Conversion funnel: views, sign-up, purchase — three real hooks.",
        en: "Second creator onboarded: an automation developer selling DAG templates. We rebuilt her storefront together — 'Buy' button in the top half of the page, zero technical jargon. The conversion funnel: visits → signups → purchases, three real hooks.",
        cmd: None,
        social: None,
        stage: PipelineStage::CreatorOnboarding,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 78,
        visits_delta: 3_800,
        signups_delta: 55,
        purchases_delta: 0,
    },
    // ── Tick 16 ──
    TickSpec {
        second: 57600,
        speaker: FounderPersona::JeffBezos,
        fa: "[FA-PATCH-SLOT] Third sale — unreferred, $4.50: the DAG automation template. Same-second split: $3.60 creator, $0.90 treasury (10% marketplace tax + 10% unreferred affiliate routing). Treasury total so far: $3.15.",
        en: "Third sale — unreferred, $4.50 for the DAG automation template. Same-second split: $3.60 creator, $0.90 treasury (10% marketplace tax + 10% unreferred affiliate routing). Treasury total so far: $3.15.",
        cmd: None,
        social: None,
        stage: PipelineStage::RevenueFlow,
        stream: RevenueStream::MarketplaceTax,
        volume_micros: 4_500_000,
        referred: false,
        users_delta: 66,
        visits_delta: 2_700,
        signups_delta: 44,
        purchases_delta: 1,
    },
    // ── Tick 17 ──
    TickSpec {
        second: 61200,
        speaker: FounderPersona::ElonMusk,
        fa: "[FA-PATCH-SLOT] Rendered the second HUD — the quant-vibe-terminal preset for the quant contest. Same Fibonacci mesh, this time with live tickers. Every video is a 60-second invitation to the repo; those views turn into sign-ups.",
        en: "Rendered HUD #2 — the quant-vibe-terminal preset for the quant competition. Same Fibonacci mesh, live tickers this time. Every video is a 60-second invitation to the repo; those views convert into signups.",
        cmd: Some("python3 tools/neural-hud/render_neural_hud.py html --preset quant-vibe-terminal --aspect 16:9 --out hud_quant.html"),
        social: None,
        stage: PipelineStage::QuantOps,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 0,
        visits_delta: 5_200,
        signups_delta: 38,
        purchases_delta: 0,
    },
    // ── Tick 18 ──
    TickSpec {
        second: 64800,
        speaker: FounderPersona::CzBinance,
        fa: "[FA-PATCH-SLOT] Second boost — this time via Telegram Stars: the trading-bot creator paid $3.00 XTR for the featured slot; 100% to the treasury. Stars means zero friction for users without crypto — exactly the rail on the map.",
        en: "Second boost — this time via Telegram Stars: the trading-bot creator paid $3.00 in XTR for a featured slot; 100% to the treasury. Stars means zero friction for non-crypto users — exactly the rail on the map.",
        cmd: None,
        social: None,
        stage: PipelineStage::RevenueFlow,
        stream: RevenueStream::FeaturedBoost,
        volume_micros: 3_000_000,
        referred: false,
        users_delta: 31,
        visits_delta: 3_300,
        signups_delta: 27,
        purchases_delta: 0,
    },
    // ── Tick 19 ──
    TickSpec {
        second: 68400,
        speaker: FounderPersona::JeffBezos,
        fa: "[FA-PATCH-SLOT] The flywheel's champion moment: a 9-year-old built her book with the ToySpark buttons and published it from the same panel — without a single line of code. Her father wrote on X: this is the first product my child sold from zero. That is our brand message, not our ad.",
        en: "The flywheel's hero moment: a 9-year-old built a storybook with the ToySpark buttons and published it in the same panel — without a single line of code. Her dad posted on X: 'this is the first product my kid sold from zero.' That is our brand message, not our ad.",
        cmd: None,
        social: Some(r#"{"platform":"telegram","handle":"@ImXLLM","kind":"toy_spark_hero","text_en":"A 9-year-old published her first storybook today — built with ToySpark buttons, sold in the marketplace, 80% went to her wallet instantly. Always Day 1.","text_fa":"A 9-year-old published her first book today — built it with ToySpark buttons, sold it in the marketplace, and 80% went straight to her wallet. Always Day 1.","views":12700,"likes":2140}"#),
        stage: PipelineStage::CreatorOnboarding,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 457,
        visits_delta: 9_900,
        signups_delta: 128,
        purchases_delta: 0,
    },
    // ── Tick 20 ──
    TickSpec {
        second: 72000,
        speaker: FounderPersona::CzBinance,
        fa: "[FA-PATCH-SLOT] Fourth sale — referred, $6.00: the quant trading bot. Split: $4.80 creator, $0.60 marketplace tax, $0.60 affiliate. The referral loop works; the 10%-50% tier model for higher affiliate tiers is on the map.",
        en: "Fourth sale — referred, $6.00 for the quant trading bot. Split: $4.80 creator, $0.60 marketplace tax, $0.60 affiliate. The referral loop is working; the 10%-50% tier model for top affiliates is on the map.",
        cmd: None,
        social: None,
        stage: PipelineStage::RevenueFlow,
        stream: RevenueStream::MarketplaceTax,
        volume_micros: 6_000_000,
        referred: true,
        users_delta: 73,
        visits_delta: 4_400,
        signups_delta: 58,
        purchases_delta: 1,
    },
    // ── Tick 21 ──
    TickSpec {
        second: 75600,
        speaker: FounderPersona::JeffBezos,
        fa: "[FA-PATCH-SLOT] Fifth sale — the biggest day: $9.00 unreferred for the bot + template + boost bundle. Same-second split: $7.20 creator, $1.80 treasury. Unit economics look beautiful: our marginal cost is nearly zero because our own repo builds the apps.",
        en: "Fifth sale — the biggest of the day: $9.00 unreferred for the 'bot + template + boost' bundle. Same-second split: $7.20 creator, $1.80 treasury. Unit economics look beautiful: our marginal cost is near zero because our own repo builds the apps.",
        cmd: None,
        social: None,
        stage: PipelineStage::RevenueFlow,
        stream: RevenueStream::MarketplaceTax,
        volume_micros: 9_000_000,
        referred: false,
        users_delta: 90,
        visits_delta: 5_600,
        signups_delta: 63,
        purchases_delta: 1,
    },
    // ── Tick 22 ──
    TickSpec {
        second: 79200,
        speaker: FounderPersona::ElonMusk,
        fa: "[FA-PATCH-SLOT] The Day-1 recap thread with @ImXforevr is live: one minute of HUD, five real numbers — 826 tests, 5 sales, $9.55 treasury revenue, 1,527 users, zero KYC. No made-up numbers; everything re-derivable from the repo and the chain.",
        en: "The Day-1 recap thread with @ImXforevr is live: one minute of HUD, five real numbers — 826 tests, 5 sales, $9.55 treasury revenue, 1,527 users, zero KYC. No made-up numbers; everything re-derivable from the repo and the chain.",
        cmd: None,
        social: Some(r#"{"platform":"x","handle":"@ImXforevr","kind":"day1_recap","text_en":"Day 0 → Day 1 recap with @elonmusk @cz_binance @JeffBezos: 826 tests green, 5 real sales, $9.55 sovereign treasury revenue from $0.00, 1,527 users. Every number auditable. Keep building.","text_fa":"Day 0 → Day 1 recap with the full team: 826 tests green, 5 real sales, $9.55 treasury revenue from zero, 1,527 users. Every number auditable. Keep building.","views":96400,"likes":8210}"#),
        stage: PipelineStage::ViralLaunch,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 127,
        visits_delta: 11_800,
        signups_delta: 91,
        purchases_delta: 0,
    },
    // ── Tick 23 — day-one seal ──
    TickSpec {
        second: 82800,
        speaker: FounderPersona::ElonMusk,
        fa: "[FA-PATCH-SLOT] Day 1 is sealed — and it contains no fake numbers: from $0.00 we reached $9.55 treasury revenue with 8 causal transactions; creators earned $22.00 and affiliates $1.10. The operational wallet: $42.55 on-chain + $15 cash + $350 locked. Tomorrow we start from Day 1 again — always Day 1.",
        en: "Day 1 is sealed — with zero made-up numbers: we started from $0.00 and reached $9.55 of treasury revenue through 8 causal transactions; creators earned $22.00 and affiliates $1.10. Operational wallet: $42.55 on-chain + $15 cash + $350 locked. Tomorrow we start again from Day 1 — it is always Day 1.",
        cmd: None,
        social: None,
        stage: PipelineStage::DayOneSealed,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 63,
        visits_delta: 2_000,
        signups_delta: 55,
        purchases_delta: 0,
    },
];

/// The canonical 24-tick scenario with all derived figures (deltas, cumulative
/// sums, charter cuts) — the single source of truth for the server and the HTML app.
#[must_use]
pub fn canonical_day0_scenario() -> Vec<SimulationTickEvent> {
    let mut wallet = OrganizationWalletLedger::seed();
    let mut out = Vec::with_capacity(CANONICAL_TICK_COUNT);
    for (index, spec) in DAY0_TICKS.iter().enumerate() {
        let split = match spec.stream {
            RevenueStream::MarketplaceTax | RevenueStream::AffiliateRouting => {
                charter_split(spec.volume_micros, spec.referred)
            }
            RevenueStream::FeaturedBoost => featured_boost_split(spec.volume_micros),
            RevenueStream::SettlementSpread => settlement_spread_split(spec.volume_micros),
            RevenueStream::None => CharterSplit {
                treasury_delta_micros: 0,
                creator_payout_micros: 0,
                affiliate_payout_micros: 0,
                marketplace_tax_micros: 0,
                affiliate_routing_micros: 0,
            },
        };
        wallet.apply_charter_split(&split, spec.stream);
        wallet.active_users += spec.users_delta;
        wallet.funnel_visits += spec.visits_delta;
        wallet.funnel_signups += spec.signups_delta;
        wallet.funnel_purchases += spec.purchases_delta;
        out.push(SimulationTickEvent {
            tick_index: u32::try_from(index).unwrap_or(0),
            simulation_second: spec.second,
            speaker: spec.speaker,
            message_fa: spec.fa.to_owned(),
            message_en: spec.en.to_owned(),
            terminal_cmd: spec.cmd.map(str::to_owned),
            social_post_json: spec.social.map(str::to_owned),
            wallet_delta_usdc_micros: split.treasury_delta_micros,
            wallet_total_usdc_micros: wallet.total_operational_usdc_micros(),
            active_pipeline_stage: spec.stage,
            revenue_stream: spec.stream,
            sale_volume_micros: spec.volume_micros,
            creator_payout_micros: split.creator_payout_micros,
            referred: spec.referred,
        });
    }
    out
}

// ── Typed errors ─────────────────────────────────────────────────────────

/// War-room errors — every typed failure path.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WarRoomError {
    /// Empty directive — the chairman sent nothing.
    #[error("chairman directive is empty")]
    EmptyDirective,
    /// Overlong directive — the wire cap.
    #[error("chairman directive exceeds the {0}-charter wire cap")]
    DirectiveTooLong(usize),
    /// Invalid speed factor — only 1, 2 and 5.
    #[error("unknown speed multiplier {0}: allowed 1, 2, 5")]
    UnknownSpeed(u32),
    /// Scenario exhausted — all 24 ticks have been played.
    #[error("scenario exhausted: all 24 Day-0 ticks already executed")]
    ScenarioExhausted,
    /// Time jump — the one-hour-per-tick cap was violated (the anti-jump rule).
    #[error("time jump detected: delta {0}s exceeds the 3600s hourly ceiling")]
    TimeJumpDetected(i64),
}

// ── War-room state ─────────────────────────────────────────────────────────

/// Live war-room state — the tick cursor, the simulation clock, the wallet ledger,
/// the played transcript and the playback controls (`Play 1x`, `2x`, `5x`, `Reset`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomState {
    /// Cursor of the next tick to play — `0` means day 0, second 0, pristine.
    pub tick_cursor: usize,
    /// Elapsed simulation seconds — always == the seconds of the last played tick.
    pub elapsed_seconds: i64,
    /// The organization wallet ledger — changes only via charter cuts.
    pub wallet: OrganizationWalletLedger,
    /// Transcript of played ticks — in causal order.
    pub transcript: Vec<SimulationTickEvent>,
    /// Is autoplay on? (`Play 1x` / `Pause`)
    pub playing: bool,
    /// Speed factor — only 1, 2 or 5.
    pub speed_multiplier: u32,
    /// Number of chairman directives played in this session.
    pub directives_executed: u32,
}

impl WarRoomState {
    /// Fresh day-0, second-0 state — `$0.00` revenue, `$33.00` crypto seed,
    /// `$15.00` cash and `$350.00` locked reserve; no tick played yet.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tick_cursor: 0,
            elapsed_seconds: 0,
            wallet: OrganizationWalletLedger::seed(),
            transcript: Vec::new(),
            playing: false,
            speed_multiplier: 1,
            directives_executed: 0,
        }
    }

    /// Turns autoplay on (`Play`).
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Stops autoplay (`Pause`).
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Sets the speed factor — only 1, 2 and 5 are allowed (`Play 1x` / `2x` / `5x`).
    ///
    /// # Errors
    /// [`WarRoomError::UnknownSpeed`] for any factor other than 1, 2, 5.
    pub fn set_speed(&mut self, multiplier: u32) -> Result<(), WarRoomError> {
        if multiplier != 1 && multiplier != 2 && multiplier != 5 {
            return Err(WarRoomError::UnknownSpeed(multiplier));
        }
        self.speed_multiplier = multiplier;
        Ok(())
    }

    /// Reset to "day 0, second 0" — the honest pristine state (`Reset`).
    pub fn reset_to_day_zero(&mut self) {
        *self = Self::new();
    }
}

impl Default for WarRoomState {
    fn default() -> Self {
        Self::new()
    }
}

impl WarRoomState {

    /// The steps' progress percentage — `0..=100`.
    #[must_use]
    pub fn progress_percent(&self) -> u8 {
        u8::try_from(self.tick_cursor * 100 / CANONICAL_TICK_COUNT).unwrap_or(100)
    }

    /// **Causal single-step progression** — plays exactly the next tick.
    ///
    /// Anti-jump guard: any tick's second gap from its predecessor never exceeds 3600;
    /// the wallet ledger changes only through charter cuts.
    ///
    /// # Errors
    /// [`WarRoomError::ScenarioExhausted`] once all 24 ticks are done;
    /// [`WarRoomError::TimeJumpDetected`] if the next tick's gap violates the hourly cap.
    pub fn advance_single_tick(&mut self) -> Result<SimulationTickEvent, WarRoomError> {
        if self.tick_cursor >= CANONICAL_TICK_COUNT {
            return Err(WarRoomError::ScenarioExhausted);
        }
        let spec = &DAY0_TICKS[self.tick_cursor];
        let delta = spec.second - self.elapsed_seconds;
        if delta > MAX_TICK_DELTA_SECONDS {
            return Err(WarRoomError::TimeJumpDetected(delta));
        }
        // The charter cut is "recomputed" from the tick's raw volume — no number
        // stored in advance is trusted without verifiable proof.
        let split = match spec.stream {
            RevenueStream::MarketplaceTax | RevenueStream::AffiliateRouting => {
                charter_split(spec.volume_micros, spec.referred)
            }
            RevenueStream::FeaturedBoost => featured_boost_split(spec.volume_micros),
            RevenueStream::SettlementSpread => settlement_spread_split(spec.volume_micros),
            RevenueStream::None => CharterSplit {
                treasury_delta_micros: 0,
                creator_payout_micros: 0,
                affiliate_payout_micros: 0,
                marketplace_tax_micros: 0,
                affiliate_routing_micros: 0,
            },
        };
        self.wallet.apply_charter_split(&split, spec.stream);
        self.wallet.active_users = self.wallet.active_users.saturating_add(spec.users_delta);
        self.wallet.funnel_visits = self.wallet.funnel_visits.saturating_add(spec.visits_delta);
        self.wallet.funnel_signups = self.wallet.funnel_signups.saturating_add(spec.signups_delta);
        self.wallet.funnel_purchases = self.wallet.funnel_purchases.saturating_add(spec.purchases_delta);
        let event = SimulationTickEvent {
            tick_index: u32::try_from(self.tick_cursor).unwrap_or(0),
            simulation_second: spec.second,
            speaker: spec.speaker,
            message_fa: spec.fa.to_owned(),
            message_en: spec.en.to_owned(),
            terminal_cmd: spec.cmd.map(str::to_owned),
            social_post_json: spec.social.map(str::to_owned),
            wallet_delta_usdc_micros: split.treasury_delta_micros,
            wallet_total_usdc_micros: self.wallet.total_operational_usdc_micros(),
            active_pipeline_stage: spec.stage,
            revenue_stream: spec.stream,
            sale_volume_micros: spec.volume_micros,
            creator_payout_micros: split.creator_payout_micros,
            referred: spec.referred,
        };
        self.elapsed_seconds = spec.second;
        self.tick_cursor += 1;
        self.transcript.push(event.clone());
        Ok(event)
    }

    /// Injecting the **chairman's tactical directive** — the message goes into the playback transcript and
    /// all three co-founders answer in character (FA/EN).
    ///
    /// # Errors
    /// [`WarRoomError::EmptyDirective`] / [`WarRoomError::DirectiveTooLong`].
    pub fn inject_chairman_directive(
        &mut self,
        directive_text: &str,
        locale: Locale,
    ) -> Result<ChairmanDirectiveLog, WarRoomError> {
        let text = directive_text.trim();
        if text.is_empty() {
            return Err(WarRoomError::EmptyDirective);
        }
        if text.chars().count() > MAX_DIRECTIVE_CHARS {
            return Err(WarRoomError::DirectiveTooLong(MAX_DIRECTIVE_CHARS));
        }
        let intent = ChairmanIntent::classify(text);
        let replies = ChairmanChatEngine::respond_all(intent, locale);
        let log = ChairmanDirectiveLog {
            directive_text: text.to_owned(),
            intent,
            replies,
            triggered_tick: self.tick_cursor,
            simulation_second: self.elapsed_seconds,
            locale,
        };
        self.directives_executed += 1;
        Ok(log)
    }
}

#[must_use]
pub fn explain_zero_to_hundred_elif5(locale: Locale) -> ZeroToHundredExplainer {
    ZeroToHundredExplainer {
        locale,
        headline_fa: "[FA-PATCH-SLOT] The system from 0 to 100 — in the simplest language on Earth".to_owned(),
        headline_en: "The system from 0 to 100 — in the simplest possible language".to_owned(),
        charter_fa: "[FA-PATCH-SLOT] The money contract is simple: every sale pays 80% creator and 20% organization wallet — the same second, on Base (8453) or Telegram Stars.".to_owned(),
        charter_en: "The money charter is simple: every sale pays 80% to the creator and 20% to the organization wallet — instantly, on Base (8453) or Telegram Stars.".to_owned(),
        steps: [
            ZeroLiteracyPipelineStep::StepAUserIntent,
            ZeroLiteracyPipelineStep::StepBAvatarFactory,
            ZeroLiteracyPipelineStep::StepCStoreAndSocial,
            ZeroLiteracyPipelineStep::StepDAutoSplit,
        ],
    }
}

/// The "0-to-100" explainer package — four zero-literacy steps + headline + money charter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZeroToHundredExplainer {
    /// Package locale.
    pub locale: Locale,
    /// Persian headline.
    pub headline_fa: String,
    /// English headline.
    pub headline_en: String,
    /// The money-charter sentence — Persian.
    pub charter_fa: String,
    /// The money-charter sentence — English.
    pub charter_en: String,
    /// The four pipeline steps, in order A → D.
    pub steps: [ZeroLiteracyPipelineStep; 4],
}

// ── Chairman chat engine ──────────────────────────────────────────────────────────

/// The detected chairman-directive intent — deterministic offline classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChairmanIntent {
    /// Make the HUD video go viral on X.
    ViralHudLaunch,
    /// Enable the Base USDC and Telegram Stars rails.
    PaymentRails,
    /// Simplify the system so even a 9-year-old understands it.
    SimplifyForBeginners,
    /// An instant wallet report and the next step.
    StatusReport,
    /// Run the Vibe-Trading quantum trading strategy.
    QuantTrading,
    /// Explain the system's 0-to-100 in the simplest language.
    ExplainSimple,
    /// A generic execution order — the next operational step.
    ShipNextStep,
}

impl ChairmanIntent {
    /// Persian keywords per intent — in declaration order.
    pub const KEYWORDS_FA: [&'static str; 7] = [
        "[FA-PATCH-SLOT] viral",
        "[FA-PATCH-SLOT] gateway",
        "[FA-PATCH-SLOT] simple",
        "[FA-PATCH-SLOT] report",
        "[FA-PATCH-SLOT] trade",
        "[FA-PATCH-SLOT] explain",
        "[FA-PATCH-SLOT] run",
    ];

    /// English keywords per intent — in declaration order.
    pub const KEYWORDS_EN: [&'static str; 7] = [
        "viral",
        "gateway",
        "simple",
        "report",
        "quant",
        "explain",
        "ship",
    ];

    /// All intents in declaration order — for a stable tie-break.
    pub const ALL: [ChairmanIntent; 7] = [
        Self::ViralHudLaunch,
        Self::PaymentRails,
        Self::SimplifyForBeginners,
        Self::StatusReport,
        Self::QuantTrading,
        Self::ExplainSimple,
        Self::ShipNextStep,
    ];

    /// Deterministic classification — the highest keyword-hit count (Persian + English),
    /// tie-broken by declaration order; no hits → [`ChairmanIntent::ShipNextStep`].
    #[must_use]
    pub fn classify(raw: &str) -> Self {
        let text = raw.to_lowercase();
        let mut best = Self::ShipNextStep;
        let mut best_hits = 0usize;
        for (idx, intent) in Self::ALL.iter().enumerate() {
            let mut hits = 0usize;
            if text.contains(Self::KEYWORDS_FA[idx]) {
                hits += 1;
            }
            if text.contains(Self::KEYWORDS_EN[idx]) {
                hits += 1;
            }
            // Extra high-frequency keywords per intent — the second weight.
            for extra in Self::extra_keywords(*intent) {
                if text.contains(extra) {
                    hits += 1;
                }
            }
            if hits > best_hits {
                best_hits = hits;
                best = *intent;
            }
        }
        best
    }

    /// Auxiliary keywords per intent — "hud", "usdc", "kid", etc.
    fn extra_keywords(intent: Self) -> &'static [&'static str] {
        match intent {
            Self::ViralHudLaunch => &["hud", "[FA-PATCH-SLOT] video", "video", "x", "neural"],
            Self::PaymentRails => &["usdc", "[FA-PATCH-SLOT] stars", "stars", "[FA-PATCH-SLOT] payment", "payment", "8453"],
            Self::SimplifyForBeginners => &["[FA-PATCH-SLOT] kid", "[FA-PATCH-SLOT] child", "child", "toyspark", "eli5", "[FA-PATCH-SLOT] 9-year-old"],
            Self::StatusReport => &["[FA-PATCH-SLOT] status", "[FA-PATCH-SLOT] wallet", "wallet", "status", "[FA-PATCH-SLOT] step", "step"],
            Self::QuantTrading => &["[FA-PATCH-SLOT] quant", "vibe", "trade", "trading", "ccxt", "backtest"],
            Self::ExplainSimple => &["[FA-PATCH-SLOT] zero", "zero", "[FA-PATCH-SLOT] understand", "understand", "[FA-PATCH-SLOT] 0", "100"],
            Self::ShipNextStep => &["next", "[FA-PATCH-SLOT] next", "launch", "deploy", "[FA-PATCH-SLOT] tomorrow"],
        }
    }

    /// The intent's Persian label.
    #[must_use]
    pub const fn label_fa(self) -> &'static str {
        match self {
            Self::ViralHudLaunch => "[FA-PATCH-SLOT] Neural HUD viral launch",
            Self::PaymentRails => "[FA-PATCH-SLOT] Activate the payment rails",
            Self::SimplifyForBeginners => "[FA-PATCH-SLOT] Simplicity for newcomers",
            Self::StatusReport => "[FA-PATCH-SLOT] Live status report",
            Self::QuantTrading => "[FA-PATCH-SLOT] Run the Vibe-Trading quant engine",
            Self::ExplainSimple => "[FA-PATCH-SLOT] The 0-to-100 explanation",
            Self::ShipNextStep => "[FA-PATCH-SLOT] The next execution step",
        }
    }

    /// The intent's English label.
    #[must_use]
    pub const fn label_en(self) -> &'static str {
        match self {
            Self::ViralHudLaunch => "Neural HUD viral launch",
            Self::PaymentRails => "Activate payment rails",
            Self::SimplifyForBeginners => "Simplify for beginners",
            Self::StatusReport => "Live status report",
            Self::QuantTrading => "Run Vibe-Trading quant",
            Self::ExplainSimple => "Explain 0 to 100",
            Self::ShipNextStep => "Ship the next step",
        }
    }
}

/// A co-founder's in-character reply to the chairman's directive — bilingual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FounderReply {
    /// The responder.
    pub speaker: FounderPersona,
    /// Persian text.
    pub message_fa: String,
    /// English text — full parity.
    pub message_en: String,
    /// The suggested terminal command — when there is a technical execution.
    pub terminal_cmd: Option<String>,
    /// The suggested social post — when there is a viral action.
    pub social_post_json: Option<String>,
}

/// Full report of one chairman directive — the same `"ChairmanDirectiveLog"` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChairmanDirectiveLog {
    /// Raw directive text.
    pub directive_text: String,
    /// The detected intent.
    pub intent: ChairmanIntent,
    /// Three in-character replies (Elon, CZ, Jeff).
    pub replies: Vec<FounderReply>,
    /// The active tick at injection time.
    pub triggered_tick: usize,
    /// The simulation second at injection time.
    pub simulation_second: i64,
    /// The replies' locale.
    pub locale: Locale,
}

/// Chairman chat engine — an in-character reply from all three co-founders, 100% offline and
/// key-free; for unlimited generation, the HTML app's live LLM modal rides on
/// Pollinations/OpenRouter/local (optional, outside this kernel).
pub struct ChairmanChatEngine;

impl ChairmanChatEngine {
    /// Elon Musk's in-character reply — speed, the HUD and virality.
    fn respond_elon(intent: ChairmanIntent) -> FounderReply {
        let (fa, en, cmd, social): (&'static str, &'static str, Option<&'static str>, Option<&'static str>) =
            match intent {
                ChairmanIntent::ViralHudLaunch => (
                    "[FA-PATCH-SLOT] The directive is clear: I re-render the Neural HUD with the monopoly-core preset and close the X thread with the four-phase timeline. The video must show 180 nodes in under 9 seconds, otherwise I rewrite it. Speed first.",
                    "Clear directive: I re-render the Neural HUD with the monopoly-core preset and close the X thread with the 4-phase timeline. If the video doesn't show all 180 nodes under 9 seconds, I rewrite it. Velocity first.",
                    Some("python3 tools/neural-hud/render_neural_hud.py html --preset kia-monopoly-core --aspect 9:16 --out hud_viral.html"),
                    Some(r#"{"platform":"x","handle":"@elonmusk","kind":"chairman_reply","text_en":"CHAIRMAN DIRECTIVE RECEIVED: Neural HUD viral launch in progress. 180 nodes, 4 phases, 60 seconds. Watch this space.","text_fa":"Chairman directive received: the neural HUD viral launch is in progress. 180 nodes, 4 phases, 60 seconds.","views":0,"likes":0}"#),
                ),
                ChairmanIntent::PaymentRails => (
                    "[FA-PATCH-SLOT] Payment rails must answer in under a second; if Stars latency goes above 800ms I fix the mini-app with session pre-warming. Payment speed = conversion rate.",
                    "Payment rails must answer under one second; if Stars latency goes above 800ms I fix the mini-app with session prewarming. Payment speed = conversion rate.",
                    None,
                    None,
                ),
                ChairmanIntent::SimplifyForBeginners => (
                    "[FA-PATCH-SLOT] Drive out every extra click: ToySpark buttons reach 64 pixels and the one-tap book flow stays. Any step carrying a technical word gets deleted.",
                    "Kill every extra click: ToySpark buttons go to 64px and the one-tap Persian storybook flow stays. Any step with a technical word gets deleted.",
                    None,
                    None,
                ),
                ChairmanIntent::StatusReport => (
                    "[FA-PATCH-SLOT] From my side: 826 tests green, the HUD rendered, and the next render queue. The speed bottleneck is the terminal; I add one more CI command so every tick proves itself.",
                    "From my side: 826 tests green, HUD rendered, next render queued. The velocity bottleneck is the terminal; I add one more CI command so every tick proves itself.",
                    Some("cargo test --workspace --quiet 2>&1 | tail -4"),
                    None,
                ),
                ChairmanIntent::QuantTrading => (
                    "[FA-PATCH-SLOT] Quant must be fast and visible: I close the quant-vibe terminal with a live HUD so every causal backtest looks like a live broadcast. I measure the datafeed speed.",
                    "Quant must be fast and watchable: I wire the quant-vibe terminal to a live HUD so every causal backtest looks like a live broadcast. I measure feed latency.",
                    Some("python3 tools/neural-hud/render_neural_hud.py html --preset quant-vibe-terminal --aspect 16:9 --out hud_quant.html"),
                    None,
                ),
                ChairmanIntent::ExplainSimple => (
                    "[FA-PATCH-SLOT] My explanation in one line: you say one sentence, three robots build in seconds, the video sells, and money splits 80/20 the same second. Any sentence longer than that means the UI is broken.",
                    "My one-line explanation: 'you say one sentence, three robots build it in seconds, a video sells it, and the money splits 80/20 instantly.' Any sentence longer than that is a broken UI.",
                    None,
                    None,
                ),
                ChairmanIntent::ShipNextStep => (
                    "[FA-PATCH-SLOT] My next step: the third HUD with the genome preset and the technical thread 826 tests, zero unsafe — engineering is its own marketing. I run it and measure the result.",
                    "My next step: HUD #3 with the genome preset and the technical thread '826 tests, zero unsafe' — engineering is its own marketing. I ship it and measure.",
                    Some("python3 tools/neural-hud/render_neural_hud.py html --preset trinity-5model-genome --aspect 9:16 --out hud_genome.html"),
                    None,
                ),
            };
        FounderReply {
            speaker: FounderPersona::ElonMusk,
            message_fa: fa.to_owned(),
            message_en: en.to_owned(),
            terminal_cmd: cmd.map(str::to_owned),
            social_post_json: social.map(str::to_owned),
        }
    }

    /// CZ's in-character reply — liquidity, Base 8453 and the payment rails.
    fn respond_cz(intent: ChairmanIntent) -> FounderReply {
        let (fa, en, cmd): (&'static str, &'static str, Option<&'static str>) = match intent {
            ChairmanIntent::ViralHudLaunch => (
                "[FA-PATCH-SLOT] From my side: I pin the HUD video next to the Base 8453 verify hash; whoever asks where the money is gets the on-chain transaction link. Trust is the fuel of virality.",
                "From my side: I pin the HUD video next to the Base 8453 verification hash; anyone asking 'where is the money?' gets the on-chain transaction link. Trust is the fuel of virality.",
                None,
            ),
            ChairmanIntent::PaymentRails => (
                "[FA-PATCH-SLOT] I open the gateway right now: Telegram Stars (XTR) for instant purchases, Base USDC (0x8335...2913, chain 8453) for settlement. The 1.5% spread is transparent, boosts go 100% treasury, sales split 80/20. Funds SAFU.",
                "Opening the gateway right now: Telegram Stars (XTR) for instant purchases, Base USDC (0x8335...2913, chain 8453) for settlement. The 1.5% spread is transparent, boosts go 100% to treasury, sales split 80/20. Funds are SAFU.",
                Some("curl -s https://kia-api.up.railway.app/api/v1/monopoly/charter-and-base | head -c 400"),
            ),
            ChairmanIntent::SimplifyForBeginners => (
                "[FA-PATCH-SLOT] Money simplicity matters more: a new user only needs to see I pay, it gets built, 80% is mine. I make Stars the default rail; crypto stays secondary for professionals.",
                "Money simplicity wins: a new user should only see 'I buy → it builds → 80% is mine'. I make Stars the default rail; crypto stays secondary for pros.",
                None,
            ),
            ChairmanIntent::StatusReport => (
                "[FA-PATCH-SLOT] Liquidity report: the $33.00 seed on-chain, net revenue from $0.00 at report time, four revenue streams ready. Every increase is traceable with an 8453 hash — zero tampering.",
                "Liquidity report: $33.00 seed on-chain, net revenue from $0.00 as of this report, four revenue streams ready. Every increase is traceable with a 8453 hash — zero tampering.",
                Some("curl -s https://kia-api.up.railway.app/api/v1/war-room/state | head -c 400"),
            ),
            ChairmanIntent::QuantTrading => (
                "[FA-PATCH-SLOT] I run the Vibe-Trading strategy on the CCXT feed: live order book, causal backtest with the Sharpe >= 1.20 gate, and full ledger recording. Contest win = selling the bot with the 80/20 split.",
                "I run the Vibe-Trading strategy on the CCXT feed: live orderbook, causal backtest with the Sharpe >= 1.20 gate, full ledger recording. Competition win = bot sales with the 80/20 split.",
                Some("node ccxt-feed.js binance BTC/USDC --depth 5 --json"),
            ),
            ChairmanIntent::ExplainSimple => (
                "[FA-PATCH-SLOT] Money in one line: 80% belongs to the creator, 20% to the platform, settled with Stars or USDC on Base; everything provable by hash. No KYC, no banking jargon.",
                "The money in one line: '80% to the creator, 20% to the platform, settled with Stars or USDC on Base; everything provable with a hash.' No KYC, no banking jargon.",
                None,
            ),
            ChairmanIntent::ShipNextStep => (
                "[FA-PATCH-SLOT] My next step: open the second quant contest with the featured-slot prize; verify the second contract on Base and close the weekly liquidity report. Building continues.",
                "My next step: the second quant competition with a featured-slot prize; verify the second contract on Base and close the weekly liquidity report. Keep building.",
                None,
            ),
        };
        FounderReply {
            speaker: FounderPersona::CzBinance,
            message_fa: fa.to_owned(),
            message_en: en.to_owned(),
            terminal_cmd: cmd.map(str::to_owned),
            social_post_json: None,
        }
    }

    /// Jeff Bezos's in-character reply — the flywheel, the customer and simplicity.
    fn respond_jeff(intent: ChairmanIntent) -> FounderReply {
        let (fa, en): (&'static str, &'static str) = match intent {
            ChairmanIntent::ViralHudLaunch => (
                "[FA-PATCH-SLOT] I wire the video into the 5-second test: under the video one fixed sentence — the user speaks, the factory builds, money splits 80/20. If a viewer gets those three in 5 seconds, views turn into purchases.",
                "I wire the video into the 5-second test: one fixed caption under it — 'the user asks, the factory builds, the money splits 80/20.' If a viewer gets those three in 5 seconds, views convert into purchases.",
            ),
            ChairmanIntent::PaymentRails => (
                "[FA-PATCH-SLOT] Every payment rail must be explainable in one sentence: Stars = one-tap purchase; crypto = global settlement. I rewrite the payment page with exactly these two sentences and measure cart abandonment.",
                "Every payment rail must be explainable in one sentence: 'Stars = one-tap buy; crypto = global settlement.' I rewrite the checkout page around those two lines and measure cart drop-off.",
            ),
            ChairmanIntent::SimplifyForBeginners => (
                "[FA-PATCH-SLOT] I run the 9-year-old test myself: the home page must answer three questions in 5 seconds — what can I do? how much do I earn? where is my money? Everything else goes below the fold.",
                "I personally run the 9-year-old test: the landing must answer three things in 5 seconds — what can I do? how much do I earn? where is my money? Everything else goes below the fold.",
            ),
            ChairmanIntent::StatusReport => (
                "[FA-PATCH-SLOT] Flywheel report: the views, sign-up, purchase funnel is live; I measure active creators and conversion every tick. Next step: open the featured slot for the best product of the week.",
                "Flywheel report: the visits→signups→purchases funnel is live; I measure active creators and conversion on every tick. Next step: free the featured slot for the week's best product.",
            ),
            ChairmanIntent::QuantTrading => (
                "[FA-PATCH-SLOT] For regular users I translate quant into savings robot: one button, one simple number — how much did it grow this week? — and a one-line risk note. Pros see the CCXT details; everyone else sees the result.",
                "For normal users I translate 'quant' into 'a savings robot': one button, one plain number — 'how much did it grow this week?' — and a one-line risk note. Pros see the CCXT details; everyone else sees the outcome.",
            ),
            ChairmanIntent::ExplainSimple => (
                "[FA-PATCH-SLOT] The flywheel in one line: kids build with big buttons, the storefront sells, the wallet splits by itself; every sale makes the engine faster for everyone. That is the whole story.",
                "The flywheel in one line: 'kids build with big buttons, the store sells, the wallet splits by itself; every sale makes the engine faster for everyone.' That is the whole story.",
            ),
            ChairmanIntent::ShipNextStep => (
                "[FA-PATCH-SLOT] My next step: open the featured slot of the week and A/B test the one-click publish button; turn every HUD view into a new creator. Always Day 1.",
                "My next step: open the weekly featured slot and A/B test the one-click publish button; convert every HUD view into a new creator. Always Day 1.",
            ),
        };
        FounderReply {
            speaker: FounderPersona::JeffBezos,
            message_fa: fa.to_owned(),
            message_en: en.to_owned(),
            terminal_cmd: None,
            social_post_json: None,
        }
    }

    /// The unregistered persona's reply — honest, no hallucination.
    fn respond_unmapped() -> FounderReply {
        FounderReply {
            speaker: FounderPersona::Chairman,
            message_fa: "[FA-PATCH-SLOT] Unregistered in-persona reply — honestly, instead of hallucinating: this persona is not in the fixed table; please pick one of the three co-founders.".to_owned(),
            message_en: "Unmapped in-character reply — honestly, instead of hallucinating: this persona is not in the fixed table; please pick one of the three co-founders.".to_owned(),
            terminal_cmd: None,
            social_post_json: None,
        }
    }

    /// A persona's in-character reply to one intent — a fixed 3×7 table.
    ///
    /// # Errors
    /// Never — kept as `Result` for signature consistency with the rest of the engine and
    /// always returns `Ok`.
    pub fn respond(
        persona: FounderPersona,
        intent: ChairmanIntent,
    ) -> Result<FounderReply, WarRoomError> {
        Ok(match persona {
            FounderPersona::ElonMusk => Self::respond_elon(intent),
            FounderPersona::CzBinance => Self::respond_cz(intent),
            FounderPersona::JeffBezos => Self::respond_jeff(intent),
            FounderPersona::Chairman => Self::respond_unmapped(),
        })
    }

    /// All three co-founders' replies to one intent — fixed order: Elon, CZ, Jeff.
    ///
    /// Both languages are always carried so bilingual parity stays provable in the audit;
    /// choosing the display language is the UI layer's job.
    #[must_use]
    pub fn respond_all(intent: ChairmanIntent, _locale: Locale) -> Vec<FounderReply> {
        let mut out = Vec::with_capacity(3);
        for persona in [
            FounderPersona::ElonMusk,
            FounderPersona::CzBinance,
            FounderPersona::JeffBezos,
        ] {
            if let Ok(reply) = Self::respond(persona, intent) {
                out.push(reply);
            }
        }
        out
    }
}

// ── Tests — 26 deterministic war-room guards ────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// The full scenario in its fixed order — for all tests.
    fn scenario() -> Vec<SimulationTickEvent> {
        canonical_day0_scenario()
    }

    /// Playing the full 24 ticks on a fresh state.
    fn run_all(state: &mut WarRoomState) {
        for _ in 0..CANONICAL_TICK_COUNT {
            state
                .advance_single_tick()
                .expect("tick must advance within scenario");
        }
    }

    // ── 1. Honest start ──
    #[test]
    fn tick0_starts_honest_at_zero_revenue() {
        let state = WarRoomState::new();
        assert_eq!(state.wallet.earned_revenue_micros, 0, "earned revenue starts at $0.00");
        assert_eq!(state.wallet.crypto_seed_micros, 33_000_000, "$33.00 crypto seed");
        assert_eq!(state.wallet.cash_seed_cents, 1_500, "$15.00 cash");
        assert_eq!(state.wallet.paper_reserve_locked_cents, 35_000, "$350.00 locked paper");
        assert_eq!(state.tick_cursor, 0);
        assert_eq!(state.elapsed_seconds, 0);
        assert_eq!(state.wallet.total_operational_usdc_micros(), 33_000_000);
        assert_eq!(state.wallet.earned_revenue_usd_string(), "$0.00");
        assert_eq!(state.wallet.crypto_seed_usd_string(), "$33.00");
        assert_eq!(state.wallet.cash_seed_usd_string(), "$15.00");
        assert_eq!(state.wallet.paper_reserve_usd_string(), "$350.00");
    }

    // ── 2. Scenario structure ──
    #[test]
    fn canonical_scenario_has_24_sequential_ticks() {
        let ticks = scenario();
        assert_eq!(ticks.len(), CANONICAL_TICK_COUNT);
        assert_eq!(CANONICAL_TICK_COUNT, 24, "spec: 24 rich Day-0 operational steps");
        for (i, t) in ticks.iter().enumerate() {
            assert_eq!(t.tick_index as usize, i, "dense tick indices");
        }
    }

    // ── 3. Time anti-jump guard ──
    #[test]
    fn no_tick_ever_jumps_more_than_one_hour() {
        let ticks = scenario();
        let mut prev = 0i64;
        for t in &ticks {
            let delta = t.simulation_second - prev;
            assert!(
                delta <= MAX_TICK_DELTA_SECONDS,
                "tick {} jumped {delta}s — real unit progression violated (max {}s)",
                t.tick_index,
                MAX_TICK_DELTA_SECONDS
            );
            prev = t.simulation_second;
        }
        // There is no one-year leap: the entire scenario is < one year.
        assert!(
            ticks[ticks.len() - 1].simulation_second < 31_536_000,
            "the whole scenario must stay inside one year"
        );
    }

    // ── 4. The 20/80 charter on every wallet increase ──
    #[test]
    fn every_wallet_increase_matches_charter_bps() {
        for t in scenario() {
            match t.revenue_stream {
                RevenueStream::None => {
                    assert_eq!(t.wallet_delta_usdc_micros, 0, "non-revenue tick has no delta");
                    assert_eq!(t.creator_payout_micros, 0);
                }
                RevenueStream::FeaturedBoost => {
                    assert_eq!(t.wallet_delta_usdc_micros, t.sale_volume_micros, "boost = 10000 bps");
                }
                RevenueStream::SettlementSpread => {
                    assert_eq!(
                        t.wallet_delta_usdc_micros,
                        t.sale_volume_micros * SETTLEMENT_SPREAD_BPS / 10_000,
                        "spread = 150 bps"
                    );
                }
                RevenueStream::MarketplaceTax | RevenueStream::AffiliateRouting => {
                    let split = charter_split(t.sale_volume_micros, t.referred);
                    assert_eq!(
                        t.wallet_delta_usdc_micros, split.treasury_delta_micros,
                        "treasury delta must equal the recomputed charter split"
                    );
                    assert_eq!(
                        t.creator_payout_micros,
                        t.sale_volume_micros * CREATOR_PAYOUT_BPS / 10_000,
                        "creator always receives 8000 bps"
                    );
                    if t.referred {
                        assert_eq!(
                            t.wallet_delta_usdc_micros,
                            t.sale_volume_micros * MARKETPLACE_TAX_BPS / 10_000,
                            "referred sale: treasury takes 1000 bps"
                        );
                        assert_eq!(
                            split.affiliate_payout_micros,
                            t.sale_volume_micros * MARKETPLACE_TAX_BPS / 10_000,
                            "referred sale: affiliate takes 1000 bps"
                        );
                    } else {
                        assert_eq!(
                            t.wallet_delta_usdc_micros,
                            t.sale_volume_micros * PLATFORM_TAKE_BPS_UNREFERRED / 10_000,
                            "unreferred sale: treasury takes the full 2000 bps (10% tax + 10% routing)"
                        );
                    }
                }
            }
        }
    }

    // ── 5. Cumulative monotonicity ──
    #[test]
    fn wallet_totals_are_monotonic_and_equal_seed_plus_earned() {
        let mut cumulative = 0i64;
        for t in scenario() {
            cumulative += t.wallet_delta_usdc_micros;
            assert_eq!(
                t.wallet_total_usdc_micros,
                CRYPTO_SEED_MICROS + cumulative,
                "total must be seed + earned-to-date, nothing else"
            );
        }
        let totals: Vec<i64> = scenario().into_iter().map(|t| t.wallet_total_usdc_micros).collect();
        for w in totals.windows(2) {
            assert!(w[1] >= w[0], "wallet totals never decrease");
        }
    }

    // ── 6. Day-one seal — $9.55 ──
    #[test]
    fn day1_seals_at_nine_fifty_five_earned() {
        let ticks = scenario();
        let total_earned: i64 = ticks.iter().map(|t| t.wallet_delta_usdc_micros).sum();
        assert_eq!(total_earned, DAY1_TOTAL_EARNED_MICROS);
        assert_eq!(total_earned, 9_550_000, "$9.55 honest Day-1 treasury revenue");
        let last = &ticks[ticks.len() - 1];
        assert_eq!(last.wallet_total_usdc_micros, 42_550_000, "$33 seed + $9.55 earned");
        assert_eq!(last.active_pipeline_stage, PipelineStage::DayOneSealed);
    }

    // ── 7. Revenue buckets ──
    #[test]
    fn stream_buckets_sum_to_total_earned() {
        let mut state = WarRoomState::new();
        run_all(&mut state);
        assert_eq!(state.wallet.streams_sum_micros(), state.wallet.earned_revenue_micros);
        assert_eq!(state.wallet.earned_revenue_micros, 9_550_000);
        // The exact distribution of the four day-one streams:
        // Marketplace tax: 0.30+0.50+0.45+0.60+0.90 = $2.75
        assert_eq!(state.wallet.stream_marketplace_tax_micros, 2_750_000);
        // Unattributed affiliate routing: 0.30+0.45+0.90 = $1.65
        assert_eq!(state.wallet.stream_affiliate_routing_micros, 1_650_000);
        // Boosts: 2.00+3.00 = $5.00
        assert_eq!(state.wallet.stream_featured_boost_micros, 5_000_000);
        // Settlement spread: $0.15
        assert_eq!(state.wallet.stream_settlement_spread_micros, 150_000);
        // The builders' 80% payouts: 2.40+4.00+3.60+4.80+7.20 = $22.00
        assert_eq!(state.wallet.creator_payouts_micros, 22_000_000);
        // Affiliate commissions: 0.50+0.60 = $1.10
        assert_eq!(state.wallet.affiliate_payouts_micros, 1_100_000);
        // Final funnel: visits ≥ signups ≥ purchases
        assert!(state.wallet.funnel_visits >= state.wallet.funnel_signups);
        assert!(state.wallet.funnel_signups >= state.wallet.funnel_purchases);
        assert_eq!(state.wallet.funnel_purchases, 5, "five real Day-1 sales");
    }

    // ── 8. Bilingual parity (PATCH_READY_SLOT doctrine) ──
    #[test]
    fn all_ticks_have_bilingual_parity() {
        for t in scenario() {
            assert_ne!(t.message_fa.trim(), "", "tick {} fa empty", t.tick_index);
            assert_ne!(t.message_en.trim(), "", "tick {} en empty", t.tick_index);
            assert_ne!(t.message_fa, t.message_en, "fa and en must be distinct languages");
            // `fa` is a patch-ready slot: International English carrying the patch
            // marker, pending the hot-patched Persian from `locales/fa.patch.json`.
            assert!(
                t.message_fa.starts_with("[FA-PATCH-SLOT]"),
                "tick {} fa must carry the patch-slot marker",
                t.tick_index
            );
            // Arabic-script glyph test computed from code points — the source tree
            // itself never embeds \u escapes, so the decoded source stays 100%
            // International English.
            let is_arabic_script =
                |c: char| (0x0600..=0x06FF).contains(&(c as u32)) || (0xFB50..=0xFEFF).contains(&(c as u32));
            assert!(
                !t.message_en.chars().any(is_arabic_script),
                "tick {} en must never contain Arabic-script glyphs",
                t.tick_index
            );
        }
    }

    // ── 9. Three-partner rotation ──
    #[test]
    fn speaker_rotation_covers_all_three_founders() {
        let ticks = scenario();
        for persona in [FounderPersona::ElonMusk, FounderPersona::CzBinance, FounderPersona::JeffBezos] {
            assert!(
                ticks.iter().any(|t| t.speaker == persona),
                "{persona:?} must speak at least once"
            );
        }
        assert!(
            !ticks.iter().any(|t| t.speaker == FounderPersona::Chairman),
            "the chairman observes Day-0; only co-founders narrate ticks"
        );
    }

    // ── 10. Monotonic stage progression ──
    #[test]
    fn all_eight_campaign_lanes_visited_and_day1_seals() {
        let ticks = scenario();
        // The day starts with ignition and ends with the day-one seal — operational
        // lanes may reactivate during the day (the real launch cycle).
        assert_eq!(ticks[0].active_pipeline_stage, PipelineStage::Ignition);
        assert_eq!(
            ticks[ticks.len() - 1].active_pipeline_stage,
            PipelineStage::DayOneSealed
        );
        for stage in [
            PipelineStage::Ignition,
            PipelineStage::ProductClarity,
            PipelineStage::ViralLaunch,
            PipelineStage::PaymentRails,
            PipelineStage::CreatorOnboarding,
            PipelineStage::RevenueFlow,
            PipelineStage::QuantOps,
            PipelineStage::DayOneSealed,
        ] {
            assert!(
                ticks.iter().any(|t| t.active_pipeline_stage == stage),
                "campaign lane {stage:?} must be visited during Day 0"
            );
        }
        // The final seal happens exactly once, on the last tick.
        assert_eq!(
            ticks.iter().filter(|t| t.active_pipeline_stage == PipelineStage::DayOneSealed).count(),
            1,
            "DayOneSealed fires exactly once"
        );
    }

    // ── 11. Causal single-step progression ──
    #[test]
    fn advance_single_tick_is_strictly_sequential() {
        let mut state = WarRoomState::new();
        for expected in 0..CANONICAL_TICK_COUNT {
            let event = state
                .advance_single_tick()
                .expect("tick must advance");
            assert_eq!(event.tick_index as usize, expected);
            assert_eq!(state.tick_cursor, expected + 1);
            assert_eq!(state.transcript.len(), expected + 1);
            assert_eq!(state.elapsed_seconds, event.simulation_second);
        }
    }

    // ── 12. Scenario end ──
    #[test]
    fn advance_beyond_scenario_is_exhausted() {
        let mut state = WarRoomState::new();
        run_all(&mut state);
        let err = state.advance_single_tick().expect_err("25th tick must fail");
        assert_eq!(err, WarRoomError::ScenarioExhausted);
    }

    // ── 13. Honest reset ──
    #[test]
    fn reset_returns_to_day_zero_honest_state() {
        let mut state = WarRoomState::new();
        state.play();
        state.set_speed(5).expect("5x legal");
        run_all(&mut state);
        state.reset_to_day_zero();
        assert_eq!(state, WarRoomState::new(), "reset is a full honest return");
    }

    // ── 14. Allowed speeds ──
    #[test]
    fn speed_multiplier_only_allows_1_2_5() {
        let mut state = WarRoomState::new();
        for ok in [1, 2, 5] {
            state.set_speed(ok).expect("legal speed");
            assert_eq!(state.speed_multiplier, ok);
        }
        for bad in [0, 3, 7, 100] {
            let err = state.set_speed(bad).expect_err("illegal speed");
            assert_eq!(err, WarRoomError::UnknownSpeed(bad));
        }
    }

    // ── 15. Empty directive ──
    #[test]
    fn empty_directive_is_rejected() {
        let mut state = WarRoomState::new();
        assert_eq!(
            state.inject_chairman_directive("", Locale::Fa).unwrap_err(),
            WarRoomError::EmptyDirective
        );
        assert_eq!(
            state.inject_chairman_directive("     ", Locale::En).unwrap_err(),
            WarRoomError::EmptyDirective
        );
    }

    // ── 16. Giant directive ──
    #[test]
    fn oversized_directive_is_rejected() {
        let mut state = WarRoomState::new();
        let long = "[FA-PATCH-SLOT] long experimental directive".repeat(150);
        assert!(long.chars().count() > MAX_DIRECTIVE_CHARS);
        assert_eq!(
            state.inject_chairman_directive(&long, Locale::Fa).unwrap_err(),
            WarRoomError::DirectiveTooLong(MAX_DIRECTIVE_CHARS)
        );
    }

    // ── 17. Three in-character replies ──
    #[test]
    fn directive_gets_three_in_character_replies() {
        let mut state = WarRoomState::new();
        state.play();
        state
            .advance_single_tick()
            .expect("warm-up tick so the directive fires mid-scenario");
        let log = state
            .inject_chairman_directive("[FA-PATCH-SLOT] Make the HUD video go viral on X", Locale::Fa)
            .expect("legal directive");
        assert_eq!(log.replies.len(), 3, "Elon, CZ and Jeff all respond");
        assert_eq!(log.replies[0].speaker, FounderPersona::ElonMusk);
        assert_eq!(log.replies[1].speaker, FounderPersona::CzBinance);
        assert_eq!(log.replies[2].speaker, FounderPersona::JeffBezos);
        assert_eq!(log.intent, ChairmanIntent::ViralHudLaunch);
        assert_eq!(log.triggered_tick, 1, "fires at the live cursor");
        assert_eq!(state.directives_executed, 1);
        for reply in &log.replies {
            assert_ne!(reply.message_fa.trim(), "");
            assert_ne!(reply.message_en.trim(), "");
            assert_ne!(reply.message_fa, reply.message_en);
        }
        // Viral → only Elon has the HUD terminal.
        assert!(log.replies[0].terminal_cmd.is_some());
        assert!(log.replies[0].social_post_json.is_some());
    }

    // ── 18. Persian classification of the six chips ──
    #[test]
    fn fa_directives_classify_all_six_chips() {
        let cases = [
            ("🚀 [FA-PATCH-SLOT] Elon: make the Neural HUD video go viral on X", ChairmanIntent::ViralHudLaunch),
            ("💰 [FA-PATCH-SLOT] CZ: activate the Base USDC and Telegram Stars rails", ChairmanIntent::PaymentRails),
            ("🛒 [FA-PATCH-SLOT] Jeff: make the system so simple a 9-year-old gets it", ChairmanIntent::SimplifyForBeginners),
            ("📊 [FA-PATCH-SLOT] Give the live wallet report and the next step", ChairmanIntent::StatusReport),
            ("⚡ [FA-PATCH-SLOT] Run the Vibe-Trading quant strategy", ChairmanIntent::QuantTrading),
            ("🧠 [FA-PATCH-SLOT] Explain the system from 0 to 100 in the simplest language", ChairmanIntent::ExplainSimple),
        ];
        for (text, want) in cases {
            assert_eq!(ChairmanIntent::classify(text), want, "fa classify: {text}");
        }
    }

    // ── 19. English classification ──
    #[test]
    fn en_directives_classify_too() {
        assert_eq!(
            ChairmanIntent::classify("make the Neural HUD video viral on X"),
            ChairmanIntent::ViralHudLaunch
        );
        assert_eq!(
            ChairmanIntent::classify("activate the Base USDC payment gateway now"),
            ChairmanIntent::PaymentRails
        );
        assert_eq!(
            ChairmanIntent::classify("keep it simple so a child understands"),
            ChairmanIntent::SimplifyForBeginners
        );
        assert_eq!(
            ChairmanIntent::classify("give me a wallet status report"),
            ChairmanIntent::StatusReport
        );
        assert_eq!(
            ChairmanIntent::classify("run the quant vibe-trading backtest"),
            ChairmanIntent::QuantTrading
        );
        assert_eq!(
            ChairmanIntent::classify("explain the system from zero to 100"),
            ChairmanIntent::ExplainSimple
        );
    }

    // ── 20. JSON-ready wire ──
    #[test]
    fn wire_dto_serializes_camel_case() {
        let mut state = WarRoomState::new();
        let event = state.advance_single_tick().expect("tick");
        let value = serde_json::to_value(&event).expect("event json");
        assert!(value.get("messageFa").is_some(), "camelCase wire keys");
        assert!(value.get("messageEn").is_some());
        assert!(value.get("walletTotalUsdcMicros").is_some());
        assert!(value.get("activePipelineStage").is_some());
        let log = state
            .inject_chairman_directive("[FA-PATCH-SLOT] Give the wallet report", Locale::En)
            .expect("directive");
        let lv = serde_json::to_value(&log).expect("log json");
        assert!(lv.get("directiveText").is_some());
        assert!(lv.get("triggeredTick").is_some());
    }

    // ── 21. Valid JSON social posts ──
    #[test]
    fn social_posts_are_valid_json_objects() {
        for t in scenario() {
            if let Some(post) = &t.social_post_json {
                let value: serde_json::Value = serde_json::from_str(post)
                    .unwrap_or_else(|e| panic!("tick {} social post invalid JSON: {e}", t.tick_index));
                assert!(value.is_object());
                assert!(value.get("platform").is_some());
                assert!(value.get("handle").is_some());
                assert!(value.get("text_fa").is_some() && value.get("text_en").is_some());
            }
        }
        // At least three social posts on day one — X and Telegram.
        assert!(scenario().iter().filter(|t| t.social_post_json.is_some()).count() >= 3);
    }

    // ── 22. The 0-to-100 explainer ──
    #[test]
    fn eli5_explainer_has_four_ordered_steps_bilingual() {
        let bundle = explain_zero_to_hundred_elif5(Locale::Fa);
        assert_eq!(bundle.steps.len(), 4);
        assert_eq!(bundle.steps[0], ZeroLiteracyPipelineStep::StepAUserIntent);
        assert_eq!(bundle.steps[1], ZeroLiteracyPipelineStep::StepBAvatarFactory);
        assert_eq!(bundle.steps[2], ZeroLiteracyPipelineStep::StepCStoreAndSocial);
        assert_eq!(bundle.steps[3], ZeroLiteracyPipelineStep::StepDAutoSplit);
        for step in bundle.steps {
            assert_ne!(step.title_fa().trim(), "");
            assert_ne!(step.title_en().trim(), "");
            assert_ne!(step.body_fa().trim(), "");
            assert_ne!(step.body_en().trim(), "");
            assert_ne!(step.body_fa(), step.body_en());
        }
        assert!(bundle.headline_fa.starts_with("[FA-PATCH-SLOT]"));
        assert!(bundle.headline_en.contains("0 to 100"));
    }

    // ── 23. Step D = the 80/20 charter ──
    #[test]
    fn eli5_step_d_states_the_80_20_charter() {
        let fa = ZeroLiteracyPipelineStep::StepDAutoSplit.body_fa();
        let en = ZeroLiteracyPipelineStep::StepDAutoSplit.body_en();
        assert!(fa.contains("[FA-PATCH-SLOT]"), "fa must carry the patch-slot marker");
        assert!(fa.contains("80%"), "fa must say 80%");
        assert!(fa.contains("20%"), "fa must say 20%");
        assert!(en.contains("80%"));
        assert!(en.contains("20%"));
    }

    // ── 24. The exact charter split ──
    #[test]
    fn charter_split_is_exact_for_unreferred_and_referred() {
        // A $3 sale without referral: 20% treasury, 80% builder.
        let s = charter_split(3_000_000, false);
        assert_eq!(s.treasury_delta_micros, 600_000);
        assert_eq!(s.creator_payout_micros, 2_400_000);
        assert_eq!(s.marketplace_tax_micros, 300_000);
        assert_eq!(s.affiliate_routing_micros, 300_000);
        assert_eq!(s.affiliate_payout_micros, 0);
        assert_eq!(
            s.treasury_delta_micros + s.creator_payout_micros,
            3_000_000,
            "split components sum to the gross volume"
        );
        // A $5 sale with referral: 10% treasury, 10% affiliate, 80% builder.
        let r = charter_split(5_000_000, true);
        assert_eq!(r.treasury_delta_micros, 500_000);
        assert_eq!(r.creator_payout_micros, 4_000_000);
        assert_eq!(r.affiliate_payout_micros, 500_000);
        assert_eq!(r.affiliate_routing_micros, 0);
        assert_eq!(
            r.treasury_delta_micros + r.creator_payout_micros + r.affiliate_payout_micros,
            5_000_000
        );
        // Boost: 100% treasury. Settlement spread: 150 bps.
        assert_eq!(featured_boost_split(2_000_000).treasury_delta_micros, 2_000_000);
        assert_eq!(settlement_spread_split(10_000_000).treasury_delta_micros, 150_000);
    }

    // ── 25. Human dollar formatting ──
    #[test]
    fn formatting_helpers_are_human() {
        assert_eq!(format_usd_micros(0), "$0.00");
        assert_eq!(format_usd_micros(600_000), "$0.60");
        assert_eq!(format_usd_micros(9_550_000), "$9.55");
        assert_eq!(format_usd_micros(42_550_000), "$42.55");
        assert_eq!(format_usd_cents(0), "$0.00");
        assert_eq!(format_usd_cents(1_500), "$15.00");
        assert_eq!(format_usd_cents(35_000), "$350.00");
    }

    // ── 26. Persona metadata ──
    #[test]
    fn persona_metadata_is_complete() {
        for persona in [
            FounderPersona::ElonMusk,
            FounderPersona::CzBinance,
            FounderPersona::JeffBezos,
            FounderPersona::Chairman,
        ] {
            assert!(persona.handle().starts_with('@'));
            assert_ne!(persona.display_name_fa(), "");
            assert_ne!(persona.display_name_en(), "");
            assert_ne!(persona.mandate_fa(), "");
            assert_ne!(persona.mandate_en(), "");
            assert!(persona.accent_hex().starts_with('#'));
            assert_eq!(persona.accent_hex().len(), 7);
        }
        assert!(FounderPersona::ElonMusk.is_co_founder());
        assert!(FounderPersona::CzBinance.is_co_founder());
        assert!(FounderPersona::JeffBezos.is_co_founder());
        assert!(!FounderPersona::Chairman.is_co_founder());
        // The constants' charter — the spec's exact values.
        assert_eq!(GITHUB_REPO, "ImXLLM/OneRing");
        assert_eq!(GITHUB_COMMIT_SHORT, "282eb68");
        assert_eq!(BASE_CHAIN_ID, 8453);
        assert_eq!(TELEGRAM_STARS_CURRENCY, "XTR");
        assert_eq!(Locale::Fa.direction(), "rtl");
        assert_eq!(Locale::En.direction(), "ltr");
        assert_eq!(Locale::parse("fa"), Locale::Fa);
        assert_eq!(Locale::parse("en"), Locale::En);
        assert_eq!(Locale::parse("anything"), Locale::En);
    }
}
