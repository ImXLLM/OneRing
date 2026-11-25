//! `war_room_simulation_engine` — اتاق جنگِ بنیان‌گذار (Wave 10 — Commit #30).
//!
//! قلبِ «به‌روزرسانیِ عظیمِ UI»: یک شبیه‌سازِ واقعیِ زمانیِ «روز ۰ → روز ۱» برای
//! سه هم‌بنیان‌گذارِ افسانه‌ای که ۲۴/۷ دورکار زیرِ ریاستِ `0xKIA` مخزنِ
//! `KiaAgentX/xA` را تجاری می‌کنند:
//!
//! - **ایلان ماسک** ([`FounderPersona::ElonMusk`]) — سرعتِ محصول، مهندسیِ
//!   ابتدا-به-اصول و ویروسی‌شدن روی X؛
//! - **چانگپنگ ژائو** ([`FounderPersona::CzBinance`]) — نقدینگی، اسکروی
//!   Base 8453، کوانتِ CCXT و رشدِ Web3؛
//! - **جف بزوس** ([`FounderPersona::JeffBezos`]) — چرخ‌دندهٔ مشتری،
//!   اقتصادِ واحد و استراتژیِ «همیشه روزِ اول».
//!
//! قاعدهٔ سرِ سفتِ زمان: **هیچ جهشِ یک‌ساله‌ای وجود ندارد** — شبیه‌سازی گام‌به‌گام
//! در واحدِ واقعیِ ثانیه پیش می‌رود ([`advance_single_tick`]، سقفِ ۳۶۰۰ ثانیه
//! در هر تیک) و هر دلارِ درآمدی که در دفتر می‌نشیند **علّی** است: فقط از
//! عملِ موردِ توافقِ سه شریک در چتِ خصوصی و اجرای ترمینال/شبکهٔ اجتماعی
//! تولید می‌شود؛ از `$0.00` درآمدِ خالص و `$33.00` بذرِ کریپتو + `$15.00`
//! نقدِ واقعی شروع می‌شود.
//!
//! منشورِ تقسیمِ Base 8453 رویِ هر فروش اعمال می‌شود:
//! فروشِ بدونِ رفرال → خزانهٔ حاکم `2000 bps` (۲۰٪) + سازنده `8000 bps` (۸۰٪)؛
//! فروشِ با رفرال → مالیاتِ بازارِ `1000 bps` (۱۰٪) + پورسانتِ آفیلیتِ
//! `1000 bps` + سازنده `8000 bps`. چهار جریانِ درآمدِ روزِ اول ([`RevenueStream`])
//! و چهار گامِ «۰ تا ۱۰۰ برای سوادِ صفر» ([`ZeroLiteracyPipelineStep`])
//! همه در همین موتورِ قطعی، بی‌شبکه و با تستِ واحد پوشش‌داده شده‌اند.

use serde::{Deserialize, Serialize};

/// امضایِ اتاقِ جنگِ بنیان‌گذار — روی هر بستهٔ وایر حاضر است.
pub const WAR_ROOM_SIGNATURE: &str = "KIA-WAR-ROOM-W10";

/// مخزنِ هدفِ تجاری‌سازی — همان مخزنِ ۱۱-کرِیتیِ ۷۵۰تست.
pub const GITHUB_REPO: &str = "KiaAgentX/xA";

/// هشِ کوتاهِ کمیتِ شاخص روی سترِ UI — نوارِ وضعیتِ ربعِ سوم.
pub const GITHUB_COMMIT_SHORT: &str = "97b18bd";

/// شناسهٔ زنجیرهٔ Base Mainnet — منشورِ روزِ اول.
pub const BASE_CHAIN_ID: u64 = 8453;

/// آدرسِ قراردادِ `USDC` بومیِ Base — اسکروی روزِ اول.
pub const BASE_USDC_CONTRACT: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";

/// نمادِ ستارهٔ تلگرام — ریلِ پرداختِ دومِ روزِ اول.
pub const TELEGRAM_STARS_CURRENCY: &str = "XTR";

/// بذرِ کریپتوی واقعیِ سازمان — `$33.00` در Trust Wallet
/// (`Base` / `Solana` / `BNB` / `USDC` / `USDD`).
pub const CRYPTO_SEED_MICROS: i64 = 33_000_000;

/// بذرِ نقدیِ واقعی — `$15.00` پولِ نقد.
pub const CASH_SEED_CENTS: i64 = 1_500;

/// ذخیرهٔ کاغذیِ قفل‌شده — `$350.00` دست‌نخورده (حسابِ احتیاطی).
pub const PAPER_RESERVE_LOCKED_CENTS: i64 = 35_000;

/// سهمِ خزانهٔ حاکم روی فروشِ بدونِ رفرال — `2000 bps` = ۲۰٪
/// (۱۰٪ مالیاتِ بازار + ۱۰٪ روتینگِ آفیلیتِ ناموجه).
pub const PLATFORM_TAKE_BPS_UNREFERRED: i64 = 2_000;

/// سهمِ همیشگیِ سازنده — `8000 bps` = ۸۰٪.
pub const CREATOR_PAYOUT_BPS: i64 = 8_000;

/// مالیاتِ بازار روی هر فروش — `1000 bps` = ۱۰٪.
pub const MARKETPLACE_TAX_BPS: i64 = 1_000;

/// جریانِ اسلاتِ ویژه/بوستِ دیده‌شدن — `10000 bps` = ۱۰۰٪ به خزانه.
pub const FEATURED_BOOST_BPS: i64 = 10_000;

/// اختلافِ تسویهٔ آنی (`Base USDC` + `Telegram Stars XTR`) — `150 bps` = ۱٫۵٪.
pub const SETTLEMENT_SPREAD_BPS: i64 = 150;

/// سقفِ سختِ پیشرویِ زمان در هر تیک — **هیچ جهشی حتی یک ساعت مجاز نیست**
/// (قاعدهٔ «پیشرفتِ واقعیِ واحد»؛ جهشِ یک‌سالهٔ جعلی ممنوع).
pub const MAX_TICK_DELTA_SECONDS: i64 = 3_600;

/// کلِ درآمدِ صادقانهٔ روزِ اولِ سناریوی کانونی — `$9.55`
/// (`۹_۵۵۰_۰۰۰` `µUSDC`) از `$0.00` آغاز.
pub const DAY1_TOTAL_EARNED_MICROS: i64 = 9_550_000;

/// تعدادِ تیک‌هایِ کانونیِ «روز ۰ → روز ۱» — ۲۴ گامِ عملیاتیِ ثروتمند.
pub const CANONICAL_TICK_COUNT: usize = 24;

/// بیشینهٔ نویسهٔ مجازِ دستورِ رئیس (سقفِ وایر).
pub const MAX_DIRECTIVE_CHARS: usize = 2_000;

/// شبکهٔ Base با شناسهٔ ۸۴۵۳ — برای نمایشِ وایر.
pub const BASE_CHAIN_LABEL: &str = "Base Mainnet (Chain ID: 8453)";

// ── پرسوناهایِ هم‌بنیان‌گذار ─────────────────────────────────────────────────

/// پرسوناهایِ اتاقِ جنگ — سه هم‌بنیان‌گذار + رئیس.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FounderPersona {
    /// ایلان ماسک — سرعتِ محصول و ویروسی‌شدن روی X.
    ElonMusk,
    /// CZ — نقدینگی، Base 8453 و رشدِ Web3.
    CzBinance,
    /// جف بزوس — چرخ‌دندهٔ مشتری و «همیشه روزِ اول».
    JeffBezos,
    /// `0xKIA` — معمارِ ارشد و رئیسِ هیئت‌مدیره.
    Chairman,
}

impl FounderPersona {
    /// هندلِ عمومیِ پرسونا روی X.
    #[must_use]
    pub const fn handle(self) -> &'static str {
        match self {
            Self::ElonMusk => "@elonmusk",
            Self::CzBinance => "@cz_binance",
            Self::JeffBezos => "@JeffBezos",
            Self::Chairman => "@0xKIA",
        }
    }

    /// نامِ نمایشیِ فارسی.
    #[must_use]
    pub const fn display_name_fa(self) -> &'static str {
        match self {
            Self::ElonMusk => "ایلان ماسک",
            Self::CzBinance => "چانگپنگ ژائو (CZ)",
            Self::JeffBezos => "جف بزوس",
            Self::Chairman => "۰xKIA (رئیس)",
        }
    }

    /// نامِ نمایشیِ انگلیسی.
    #[must_use]
    pub const fn display_name_en(self) -> &'static str {
        match self {
            Self::ElonMusk => "Elon Musk",
            Self::CzBinance => "Changpeng Zhao (CZ)",
            Self::JeffBezos => "Jeff Bezos",
            Self::Chairman => "0xKIA (Chairman)",
        }
    }

    /// مأموریتِ رسمیِ پرسونا — فارسی.
    #[must_use]
    pub const fn mandate_fa(self) -> &'static str {
        match self {
            Self::ElonMusk => "سرعتِ محصول، مهندسیِ ابتدا-به-اصول، ویدئوی HUD عصبی و ویروسی‌شدن روی X",
            Self::CzBinance => "نقدینگی، اسکروی Base 8453، کوانتِ CCXT و رشدِ جهانیِ بدونِ KYC",
            Self::JeffBezos => "وسواسِ مشتری، اقتصادِ واحد، سادگیِ ۵ثانیه‌ای و چرخ‌دندهٔ بازارگاه",
            Self::Chairman => "معمارِ ارشد و رئیسِ هیئت‌مدیره — فرمانِ تاکتیکیِ نهایی",
        }
    }

    /// مأموریتِ رسمیِ پرسونا — انگلیسی.
    #[must_use]
    pub const fn mandate_en(self) -> &'static str {
        match self {
            Self::ElonMusk => "Chief Product Velocity — first-principles engineering & X viral distribution",
            Self::CzBinance => "Chief Liquidity — Base 8453 escrow, CCXT quant & zero-KYC Web3 growth",
            Self::JeffBezos => "Chief Flywheel — customer obsession, unit economics & 'Always Day 1'",
            Self::Chairman => "Sovereign Architect & Chairman — final tactical override",
        }
    }

    /// رنگِ accent پرسونا به هگز — آواتارِ SVG در HTML.
    #[must_use]
    pub const fn accent_hex(self) -> &'static str {
        match self {
            Self::ElonMusk => "#7EF7D2",
            Self::CzBinance => "#59B7FF",
            Self::JeffBezos => "#FFB454",
            Self::Chairman => "#F768BA",
        }
    }

    /// فقط سه هم‌بنیان‌گذارِ «شبکهٔ ۲۴/۷» — رئیس در شبکه نیست.
    #[must_use]
    pub const fn is_co_founder(self) -> bool {
        matches!(self, Self::ElonMusk | Self::CzBinance | Self::JeffBezos)
    }

    /// شناسهٔ وایرِ پرسونا — `snake_case` برایِ DTO و ستونِ دیتابیس.
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

// ── چهار جریانِ درآمدِ روزِ اول ──────────────────────────────────────────────

/// چهار جریانِ درآمدِ روزِ اول — زبانِ ساده، بدونِ اصطلاحِ مالی.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevenueStream {
    /// بدونِ جریان — تیکِ عملیاتیِ بدونِ پول.
    None,
    /// مالیاتِ ۱۰٪ بازارگاه روی هر فروشِ اپ/مهارت.
    MarketplaceTax,
    /// روتینگِ آفیلیت: فروشِ بدونِ رفرال، ۱۰٪ اضافه به خزانهٔ حاکم می‌رود
    /// (مجموعِ سهمِ پلتفرم = ۲۰٪؛ سازنده همچنان ۸۰٪).
    AffiliateRouting,
    /// اسلاتِ ویژه و بوستِ دیده‌شدن — ۱۰۰٪ به خزانه.
    FeaturedBoost,
    /// اختلافِ تسویهٔ آنی روی `Base USDC` و `Telegram Stars XTR` — ۱٫۵٪.
    SettlementSpread,
}

impl RevenueStream {
    /// شناسهٔ وایرِ جریان.
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

    /// برچسبِ فارسیِ به-زبانِ-ساده.
    #[must_use]
    pub const fn label_fa(self) -> &'static str {
        match self {
            Self::None => "بدون درآمد در این گام",
            Self::MarketplaceTax => "مالیات ۱۰٪ بازارگاه روی هر فروش",
            Self::AffiliateRouting => "روتینگ آفیلیت — ۱۰٪ اضافهٔ فروش بدون رفرال (مجموع ۲۰٪ پلتفرم)",
            Self::FeaturedBoost => "اسلات ویژه و بوست دیده‌شدن — ۱۰۰٪ به خزانه",
            Self::SettlementSpread => "اختلاف تسویهٔ آنی (Base USDC + استارز XTR) — ۱٫۵٪",
        }
    }

    /// برچسبِ انگلیسیِ به-زبانِ-ساده.
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

    /// نرخِ take بر حسبِ bps — صفر برای `None`.
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

// ── مراحلِ عملیاتیِ روزِ ۰ ───────────────────────────────────────────────────

/// مراحلِ عملیاتیِ خطِ زمانِ روزِ ۰ → روزِ ۱ — یکتا و صعودی.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    /// گرم‌کردنِ مخزن: تست‌ها، قراردادها، سلامتِ کرنل.
    Ignition,
    /// سادگیِ محصول برای سوادِ صفر — آزمونِ بچهٔ ۹ساله.
    ProductClarity,
    /// پرتابِ ویروسیِ X با ویدئوی HUD عصبی.
    ViralLaunch,
    /// بازکردنِ ریلِ پرداخت: استارز XTR + Base USDC.
    PaymentRails,
    /// پذیرشِ نخستین سازنده‌ها روی بازارگاه.
    CreatorOnboarding,
    /// جریانِ درآمدِ زنده با منشورِ ۲۰/۸۰.
    RevenueFlow,
    /// عملیاتِ کوانتومِ Vibe-Trading روی CCXT.
    QuantOps,
    /// مُهرِ روزِ اول — خلاصهٔ صادقانه و «همیشه روزِ اول».
    DayOneSealed,
}

impl PipelineStage {
    /// شناسهٔ وایرِ مرحله.
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

    /// برچسبِ فارسیِ مرحله.
    #[must_use]
    pub const fn label_fa(self) -> &'static str {
        match self {
            Self::Ignition => "احتراق — سلامتِ مخزن",
            Self::ProductClarity => "شفافیتِ محصول — آزمونِ بچهٔ ۹ساله",
            Self::ViralLaunch => "پرتابِ ویروسی روی X",
            Self::PaymentRails => "ریل‌هایِ پرداخت — XTR + USDC",
            Self::CreatorOnboarding => "پذیرشِ سازنده‌ها",
            Self::RevenueFlow => "جریانِ درآمد — منشورِ ۲۰/۸۰",
            Self::QuantOps => "عملیاتِ کوانتومِ Vibe-Trading",
            Self::DayOneSealed => "روزِ اول مُهر شد",
        }
    }

    /// برچسبِ انگلیسیِ مرحله.
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

    /// رتبهٔ صعودیِ مرحله — برای گاردِ یکنواختیِ پیشروی.
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

// ── گام‌هایِ «۰ تا ۱۰۰» برای سوادِ صفر ───────────────────────────────────────

/// چهار گامِ خطِ لولهٔ «۰ تا ۱۰۰» — طوری که بچهٔ ۹ساله هم بفهمد.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZeroLiteracyPipelineStep {
    /// گامِ الف — ورودِ کاربرِ عادی با یک جمله یا ویس.
    StepAUserIntent,
    /// گامِ ب — کارخانهٔ ۳ آواتار و ۵ مدلِ هوش مصنوعی.
    StepBAvatarFactory,
    /// گامِ ج — فروشگاه و سوشال‌مدیا با ویدئوی سه‌بعدی.
    StepCStoreAndSocial,
    /// گامِ د — تقسیمِ خودکارِ پول در ولت (۸۰٪ سازنده / ۲۰٪ سازمان).
    StepDAutoSplit,
}

impl ZeroLiteracyPipelineStep {
    /// شناسهٔ وایرِ گام.
    #[must_use]
    pub const fn step_id(self) -> &'static str {
        match self {
            Self::StepAUserIntent => "step_a_user_intent",
            Self::StepBAvatarFactory => "step_b_avatar_factory",
            Self::StepCStoreAndSocial => "step_c_store_and_social",
            Self::StepDAutoSplit => "step_d_auto_split",
        }
    }

    /// عنوانِ فارسیِ گام.
    #[must_use]
    pub const fn title_fa(self) -> &'static str {
        match self {
            Self::StepAUserIntent => "الف — ورود کاربر عادی",
            Self::StepBAvatarFactory => "ب — کارخانهٔ ۳ آواتار و ۵ مدل",
            Self::StepCStoreAndSocial => "ج — فروشگاه و سوشال‌مدیا",
            Self::StepDAutoSplit => "د — تقسیم خودکار پول در ولت",
        }
    }

    /// عنوانِ انگلیسیِ گام.
    #[must_use]
    pub const fn title_en(self) -> &'static str {
        match self {
            Self::StepAUserIntent => "Step A — a normal user walks in",
            Self::StepBAvatarFactory => "Step B — the 3-avatar & 5-model factory",
            Self::StepCStoreAndSocial => "Step C — store & social media",
            Self::StepDAutoSplit => "Step D — automatic wallet split",
        }
    }

    /// بدنهٔ فارسیِ گام — یک جملهٔ بلندِ شفاف.
    #[must_use]
    pub const fn body_fa(self) -> &'static str {
        match self {
            Self::StepAUserIntent => "کاربر با یک جملهٔ ساده — حتی با ویس یا دکمه‌هایِ بزرگِ ToySpark — می‌گوید چه می‌خواهد؛ نه کد، نه فرم پیچیده.",
            Self::StepBAvatarFactory => "آلفا (طراح)، اتاق فرمان (مهندس) و مایرون (قاضیِ سخت‌گیر با استاندارد ۱۰۰/۱۰۰) با ۵ مدلِ هوش مصنوعی در چند ثانیه محصول یا رباتِ ترید را می‌سازند.",
            Self::StepCStoreAndSocial => "محصول با ویدئوی سه‌بعدیِ Neural HUD در X و تلگرام منتشر می‌شود؛ فروشگاهِ بازارگاه بلافاصله باز است.",
            Self::StepDAutoSplit => "مشتری با استارز تلگرام یا کریپتو (Base USDC) خرید می‌کند؛ قراردادِ هوشمند در همان ثانیه ۸۰٪ را به سازنده و ۲۰٪ را به ولتِ سازمان واریز می‌کند!",
        }
    }

    /// بدنهٔ انگلیسیِ گام.
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

// ── لوکال ────────────────────────────────────────────────────────────────────

/// لوکالِ وایر — فارسی (RTL) یا انگلیسی (LTR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// فارسی — راست‌به‌چپ.
    Fa,
    /// انگلیسی — چپ‌به‌راست.
    En,
}

impl Locale {
    /// رشتهٔ وایر.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fa => "fa",
            Self::En => "en",
        }
    }

    /// جهتِ متن — `rtl` یا `ltr`.
    #[must_use]
    pub const fn direction(self) -> &'static str {
        match self {
            Self::Fa => "rtl",
            Self::En => "ltr",
        }
    }

    /// تجزیهٔ تحمل‌پذیر — هر نویسهٔ دیگر به `En` برمی‌گردد (fallback قانونی).
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        if raw.eq_ignore_ascii_case("fa") || raw.eq_ignore_ascii_case("fa-ir") {
            Self::Fa
        } else {
            Self::En
        }
    }
}

// ── منشورِ تقسیمِ Base 8453 ─────────────────────────────────────────────────

/// نتیجهٔ دقیقِ منشورِ ۲۰/۸۰ روی یک حجمِ فروش — همهٔ اجزا به `µUSDC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharterSplit {
    /// سهمِ خزانهٔ حاکم — همان دلتایِ ولتِ سازمان.
    pub treasury_delta_micros: i64,
    /// پرداختِ سازنده — همیشه `8000 bps` حجم.
    pub creator_payout_micros: i64,
    /// پرداختِ آفیلیت — فقط روی فروشِ با رفرال (`1000 bps`).
    pub affiliate_payout_micros: i64,
    /// سهمِ مالیاتِ بازار (`1000 bps`) — بخشی از خزانه.
    pub marketplace_tax_micros: i64,
    /// سهمِ روتینگِ آفیلیتِ ناموجه (`1000 bps`) — فقط فروشِ بدونِ رفرال.
    pub affiliate_routing_micros: i64,
}

/// اعمالِ منشورِ Base 8453 روی یک حجمِ فروش — توابعِ تقسیمِ قطعیِ
/// «صخره‌سخت»: سازنده همیشه `8000 bps`؛ بدونِ رفرال خزانه `2000 bps`؛
/// با رفرال خزانه `1000 bps` و آفیلیت `1000 bps`. جمعِ اجزا == حجم.
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

/// اعمالِ جریانِ بوستِ ویژه — `10000 bps` یعنی ۱۰۰٪ حجم به خزانه.
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

/// اعمالِ اختلافِ تسویهٔ آنی — `150 bps` از حجمِ تسویه به خزانه.
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

/// قالب‌بندیِ انسانیِ `µUSDC` به رشتهٔ دلار — `$9.55` / `$0.00`.
#[must_use]
pub fn format_usd_micros(micros: i64) -> String {
    format!("${}.{:02}", micros / 1_000_000, (micros % 1_000_000) / 10_000)
}

/// قالب‌بندیِ انسانیِ سنت به رشتهٔ دلار — `$15.00`.
#[must_use]
pub fn format_usd_cents(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

// ── دفترِ ولتِ سازمان ────────────────────────────────────────────────────────

/// دفترِ ولتِ سازمان — وضعیتِ صادقانهٔ پول در هر لحظه.
///
/// شروعِ واقعی: `$33.00` بذرِ کریپتو ([`CRYPTO_SEED_MICROS`]) + `$15.00` نقد
/// ([`CASH_SEED_CENTS`]) + `$350.00` ذخیرهٔ کاغذیِ قفل‌شده
/// ([`PAPER_RESERVE_LOCKED_CENTS`]) و `$0.00` درآمدِ خالصِ پلتفرم.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationWalletLedger {
    /// بذرِ کریپتو — `۳۳_۰۰۰_۰۰۰` `µUSDC` = `$33.00`.
    pub crypto_seed_micros: i64,
    /// بذرِ نقد — `۱_۵۰۰` سنت = `$15.00`.
    pub cash_seed_cents: i64,
    /// ذخیرهٔ کاغذیِ قفل‌شده — `۳۵_۰۰۰` سنت = `$350.00` (دست‌نخورده).
    pub paper_reserve_locked_cents: i64,
    /// درآمدِ خالصِ پلتفرم — از `$0.00` شروع می‌شود.
    pub earned_revenue_micros: i64,
    /// مجموعِ پرداخت‌هایِ ۸۰٪ به سازنده‌ها.
    pub creator_payouts_micros: i64,
    /// مجموعِ پرداخت‌هایِ آفیلیت روی فروش‌هایِ با رفرال.
    pub affiliate_payouts_micros: i64,
    /// سطلِ جریانِ ۱ — مالیاتِ بازار.
    pub stream_marketplace_tax_micros: i64,
    /// سطلِ جریانِ ۲ — روتینگِ آفیلیتِ ناموجه.
    pub stream_affiliate_routing_micros: i64,
    /// سطلِ جریانِ ۳ — بوستِ ویژه.
    pub stream_featured_boost_micros: i64,
    /// سطلِ جریانِ ۴ — اختلافِ تسویه.
    pub stream_settlement_spread_micros: i64,
    /// کاربرانِ فعالِ روز.
    pub active_users: u64,
    /// قیفِ بازدید.
    pub funnel_visits: u64,
    /// قیفِ ثبت‌نام.
    pub funnel_signups: u64,
    /// قیفِ خرید.
    pub funnel_purchases: u64,
}

impl OrganizationWalletLedger {
    /// دفترِ آغازینِ روزِ ۰، ثانیهٔ ۰ — صادقانه و بدونِ هیچ پولِ جعلی.
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

    /// موجودیِ عملیاتیِ کل — بذرِ کریپتو + درآمدِ کسب‌شده (`µUSDC`).
    #[must_use]
    pub fn total_operational_usdc_micros(&self) -> i64 {
        self.crypto_seed_micros + self.earned_revenue_micros
    }

    /// جمعِ چهار سطلِ جریانِ درآمد — باید همیشه == `earned_revenue_micros`.
    #[must_use]
    pub fn streams_sum_micros(&self) -> i64 {
        self.stream_marketplace_tax_micros
            + self.stream_affiliate_routing_micros
            + self.stream_featured_boost_micros
            + self.stream_settlement_spread_micros
    }

    /// درآمدِ خالص به رشتهٔ دلار — `$9.55`.
    #[must_use]
    pub fn earned_revenue_usd_string(&self) -> String {
        format_usd_micros(self.earned_revenue_micros)
    }

    /// بذرِ کریپتو به رشتهٔ دلار — `$33.00`.
    #[must_use]
    pub fn crypto_seed_usd_string(&self) -> String {
        format_usd_micros(self.crypto_seed_micros)
    }

    /// بذرِ نقد به رشتهٔ دلار — `$15.00`.
    #[must_use]
    pub fn cash_seed_usd_string(&self) -> String {
        format_usd_cents(self.cash_seed_cents)
    }

    /// ذخیرهٔ کاغذیِ قفل‌شده به رشتهٔ دلار — `$350.00`.
    #[must_use]
    pub fn paper_reserve_usd_string(&self) -> String {
        format_usd_cents(self.paper_reserve_locked_cents)
    }

    /// اعمالِ مستقیمِ یک برشِ منشور روی دفتر — تنها مسیرِ تغییرِ پول.
    ///
    /// سطل‌هایِ جریانِ ۱ و ۲ از اجزایِ واقعیِ برش (`marketplace_tax_micros` +
    /// `affiliate_routing_micros`) پر می‌شوند تا فروشِ بدونِ رفرال دقیقاً
    /// «۱۰٪ + ۱۰٪» را در دو سطلِ درست بنشاند؛ سطل‌هایِ ۳ و ۴ کلِ
    /// `treasury_delta_micros` را می‌گیرند.
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

// ── رخدادِ تیک ───────────────────────────────────────────────────────────────

/// یک گامِ عملیاتیِ واقعیِ «روز ۰ → روز ۱» — همان ردیفِ `"FounderWarRoomTick"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationTickEvent {
    /// شمارهٔ تیک — `0..24`.
    pub tick_index: u32,
    /// ثانیهٔ شبیه‌سازی از `Day 0, 00:00:00` — هیچ جهشی از سقفِ ۳۶۰۰.
    pub simulation_second: i64,
    /// سخنرانِ تیک.
    pub speaker: FounderPersona,
    /// پیامِ فارسی — طبیعی و در-شخصیت.
    pub message_fa: String,
    /// پیامِ انگلیسی — پاریتهٔ کامل با فارسی.
    pub message_en: String,
    /// فرمانِ دقیقِ ترمینال روی `KiaAgentX/xA` — در صورتِ اجرا.
    pub terminal_cmd: Option<String>,
    /// پستِ زندهٔ اجتماعی (X/تلگرام) به JSON — در صورتِ انتشار.
    pub social_post_json: Option<String>,
    /// دلتایِ خزانه در این تیک (`µUSDC`) — صفر برای تیک‌هایِ بدونِ پول.
    pub wallet_delta_usdc_micros: i64,
    /// موجودیِ کلِ عملیاتی پس از این تیک (بذرِ ۳۳ دلار + درآمدِ تجمعی).
    pub wallet_total_usdc_micros: i64,
    /// مرحلهٔ فعالِ خطِ عملیات.
    pub active_pipeline_stage: PipelineStage,
    /// جریانِ درآمدِ فعالِ این تیک.
    pub revenue_stream: RevenueStream,
    /// حجمِ ناخالصِ فروش/بوست/تسویه در این تیک (`µUSDC`) — مبنایِ برشِ منشور.
    pub sale_volume_micros: i64,
    /// پرداختِ ۸۰٪ سازنده در این تیک.
    pub creator_payout_micros: i64,
    /// فروش با رفرال بود؟
    pub referred: bool,
}

/// مشخصهٔ داخلیِ تیک — دادهٔ ثابتِ سناریو (بدونِ ارقامِ مشتق‌شده).
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

/// سناریویِ کانونیِ ۲۴ تیکِ «روز ۰ → روز ۱» — منبعِ واحدِ حقیقتِ
/// سرور (seed)، تست‌ها و اپِ HTML (`tools/war-room/render_war_room.py`).
///
/// هر دلارِ درآمد **علّی** است: ابتدا گپِ سه شریک، سپس اجرا (ترمینال/شبکهٔ
/// اجتماعی)، سپس پول. جمعِ نهاییِ درآمد: `$9.55` — صادقانه و بدونِ جهش.
const DAY0_TICKS: [TickSpec; CANONICAL_TICK_COUNT] = [
    // ── تیک ۰ — روزِ ۰، ثانیهٔ ۰ ──
    TickSpec {
        second: 0,
        speaker: FounderPersona::JeffBezos,
        fa: "روزِ اول! سه نفر از ما زیرِ ریاستِ 0xKIA دورکار ۲۴/۷ شریک شدیم. از مخزن KiaAgentX/xA شروع می‌کنیم: ۱۱ کرِیتِ Rust، ۷۵۰ تستِ پاس‌شده، ۹ موجِ تحویل‌شده. سرمایه واقعی؟ ۳۳ دلار کریپتو، ۱۵ دلار نقد، ۳۵۰ دلار کاغذیِ قفل. درآمدِ پلتفرم دقیقاً صفر است — و باید از دلِ کارِ واقعی بیرون بیاید. همیشه روزِ اول.",
        en: "Day 1! The three of us just partnered 24/7 remotely under 0xKIA. We start from the KiaAgentX/xA repo: 11 Rust crates, 750 passing tests, 9 delivered waves. Real capital? $33 crypto, $15 cash, $350 locked paper. Platform revenue is exactly $0.00 — and it must come from real work. It is always Day 1.",
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
    // ── تیک ۱ ──
    TickSpec {
        second: 3600,
        speaker: FounderPersona::ElonMusk,
        fa: "اول سلامتِ مخزن را ثابت می‌کنیم. تستِ کاریِ کاملِ workspace را اجرا می‌کنم — ۷۵۰ پاس، صفر شکست، صفر clippy. این کورهٔ اعتبارِ ماست؛ بدونِ آن هیچ ویروسی‌شدنی معنا ندارد.",
        en: "First we prove the repo is healthy. Running the full workspace test suite — 750 passed, zero failures, zero clippy. That is our credibility furnace; nothing viral means anything without it.",
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
    // ── تیک ۲ ──
    TickSpec {
        second: 7200,
        speaker: FounderPersona::CzBinance,
        fa: "ریل‌هایِ پول را قبل از هر تبلیغی وریفای می‌کنم: قراردادهایِ Base با Chain ID 8453 و USDC بومیِ 0x8335...2913 روی شبکهٔ اصلی تأیید شدند؛ تلگرام استارز (XTR) هم آماده است. صندوق‌ها SAFU — هر فروش ۸۰٪ سازنده، بقیه طبق منشور.",
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
    // ── تیک ۳ ──
    TickSpec {
        second: 10800,
        speaker: FounderPersona::JeffBezos,
        fa: "قبل از ویروسی‌شدن، آزمونِ بچهٔ ۹ساله را می‌گذارم: یک کاربرِ عادی در ۵ ثانیه باید بفهمد سیستم چه می‌کند. خطِ لولهٔ «۰ تا ۱۰۰» را می‌نویسم — الف: کاربر می‌گوید چه می‌خواهد؛ ب: کارخانهٔ ۳ آواتار و ۵ مدل می‌سازد؛ ج: محصول به فروشگاه و سوشال می‌رود؛ د: پول به‌طور خودکار ۸۰/۲۰ تقسیم می‌شود. اگر این چهار گام واضح نباشند، هیچ دکمه‌ای نجاتمان نمی‌دهد.",
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
    // ── تیک ۴ ──
    TickSpec {
        second: 14400,
        speaker: FounderPersona::ElonMusk,
        fa: "ویدئویِ HUD عصبی را می‌سازم — مشِ کرویِ فیبوناچیِ ۱۸۰گره، seekTo قطعی، سینتِ WebAudio. این آرتِ ویروسیِ ماست: ۶۰ ثانیه که در یک نگاه ۱۱ کرِیت و خزانهٔ Base را نشان می‌دهد. ابزارِ خودِ مخزن، بدونِ هیچ سرویسِ بیرونی.",
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
    // ── تیک ۵ ──
    TickSpec {
        second: 18000,
        speaker: FounderPersona::ElonMusk,
        fa: "نخستین رشتهٔ X منتشر شد — ویدئوی HUD + «چطور در ۵ ثانیه بفهمید این سیستم چه می‌کند». منحنیِ بازدید از صفر شروع می‌شود؛ بگذاریم داده حرف بزند.",
        en: "First X thread is live — the HUD video plus 'how to understand this system in 5 seconds'. View curve starts at zero; let the data speak.",
        cmd: None,
        social: Some(r#"{"platform":"x","handle":"@elonmusk","kind":"launch_thread","text_en":"We built xA: 11 Rust crates, 750 tests, a 3D Neural HUD, and a wallet that auto-splits 80/20 on every sale. A 9-year-old can understand it in 5 seconds. Day 0 starts now.","text_fa":"سیستم xA را ساختیم: ۱۱ کرِیتِ Rust، ۷۵۰ تست، HUD سه‌بعدی عصبی، و ولتی که هر فروش را ۸۰/۲۰ خودکار تقسیم می‌کند. یک بچهٔ ۹ساله در ۵ ثانیه می‌فهمد. روزِ صفر از همین لحظه شروع است.","views":0,"likes":0}"#),
        stage: PipelineStage::ViralLaunch,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 0,
        visits_delta: 12_400,
        signups_delta: 0,
        purchases_delta: 0,
    },
    // ── تیک ۶ ──
    TickSpec {
        second: 21600,
        speaker: FounderPersona::CzBinance,
        fa: "دروازهٔ پرداختِ روزِ اول باز شد: مینی‌اپِ تلگرام با استارز (XTR) برای خریدِ آنی، و Base USDC برای تسویهٔ کریپتو. اختلافِ تسویهٔ ۱٫۵٪ شفاف است و بوستِ دیده‌شدن ۱۰۰٪ به خزانه می‌رود. پل‌ها سبز.",
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
    // ── تیک ۷ ──
    TickSpec {
        second: 25200,
        speaker: FounderPersona::JeffBezos,
        fa: "اولین سازنده onboard شد: یک معلمِ داستان‌نویس که با حالتِ ToySpark کتابِ صوتیِ کودکان می‌سازد. محصولش «سفرِ ماه» نام دارد و قیمتش ۳ دلار است. منشور را برایش ترسیم کردم: از هر فروش، ۸۰٪ خودش، ۲۰٪ پلتفرم. چرخ‌دنده از همین‌جا می‌چرخد.",
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
    // ── تیک ۸ — اولین فروش واقعی ──
    TickSpec {
        second: 28800,
        speaker: FounderPersona::JeffBezos,
        fa: "اولین فروش واقعیِ تاریخِ پلتفرم: «سفرِ ماه» — ۳٫۰۰ دلار با استارزِ تلگرام، بدونِ رفرال. قرارداد در همان ثانیه اجرا شد: ۲٫۴۰ دلار (۸۰٪) به سازنده، ۰٫۶۰ دلار (۲۰٪) به خزانهٔ حاکم. درآمدِ پلتفرم از صفر به ۰٫۶۰ دلار رسید — کاوشِ فضا آغاز شد!",
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
    // ── تیک ۹ ──
    TickSpec {
        second: 32400,
        speaker: FounderPersona::CzBinance,
        fa: "اولین تسویهٔ Base USDC هم انجام شد: فروشِ ۵٫۰۰ دلاری با رفرالِ آفیلیت. طبق منشور: ۴٫۰۰ دلار سازنده، ۰٫۵۰ دلار مالیاتِ بازار به خزانه، ۰٫۵۰ دلار پورسانتِ آفیلیت. هشِ تراکنش روی Base ثبت شد — همه‌چیز قابلِ حسابرسی.",
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
    // ── تیک ۱۰ ──
    TickSpec {
        second: 36000,
        speaker: FounderPersona::ElonMusk,
        fa: "رشتهٔ X داغ شد: ۴۸٬۹۰۰ بازدید، ۳٫۱هزار لایک، صدها ریپلای. کامنتِ برتر: «واقعاً ۸۰/۲۰ تقسیم می‌کند؟» — بله، با هشِ روی زنجیره. ترافیک به لندینگ سرازیر شد؛ نرخِ تبدیل را رصد می‌کنیم، نه فقط بازدید.",
        en: "The X thread caught fire: 48,900 views, 3.1k likes, hundreds of replies. Top comment: 'does it really split 80/20?' — yes, with an on-chain hash. Traffic is pouring into the landing; we watch conversion rate, not just views.",
        cmd: None,
        social: Some(r#"{"platform":"x","handle":"@elonmusk","kind":"viral_milestone","text_en":"48,900 views in 2 hours. The 9-year-old test passed in public: a kid built a storybook, a bot sold it, the wallet split 80/20 instantly. Funds are auditable on Base 8453.","text_fa":"۴۸٬۹۰۰ بازدید در ۲ ساعت. آزمونِ بچهٔ ۹ساله در ملع.public پاس شد: یک کودک کتاب ساخت، بات آن را فروخت، ولت همان لحظه ۸۰/۲۰ تقسیم کرد. همه‌چیز روی Base 8453 قابلِ حسابرسی است.","views":48900,"likes":3140}"#),
        stage: PipelineStage::ViralLaunch,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 200,
        visits_delta: 18_600,
        signups_delta: 62,
        purchases_delta: 0,
    },
    // ── تیک ۱۱ ──
    TickSpec {
        second: 39600,
        speaker: FounderPersona::JeffBezos,
        fa: "اولین بوستِ دیده‌شدن فروخته شد: سازندهٔ «سفرِ ماه» ۲٫۰۰ دلار برای اسلاتِ ویژهٔ بازارگاه پرداخت کرد — ۱۰۰٪ آن به خزانه می‌رود، طبق جریانِ درآمدِ سوم. قرار شد تا ۲۴ ساعت بالای کاتالوگ بماند؛ داده‌هایِ تبدیل را مقایسه می‌کنیم.",
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
    // ── تیک ۱۲ ──
    TickSpec {
        second: 43200,
        speaker: FounderPersona::CzBinance,
        fa: "اولین مسابقهٔ کوانتومِ Vibe-Trading را باز کردم: استراتژی‌هایِ CCXT روی دفترِ علّی، شارپِ ≥ ۱٫۲۰ برای صندلیِ قهرمانی. سازنده‌ها بات‌هایِ ترید می‌فروشند و ما فقط هشتاد-بیست می‌گیریم. لایو برای همه، بدونِ KYC.",
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
    // ── تیک ۱۳ ──
    TickSpec {
        second: 46800,
        speaker: FounderPersona::ElonMusk,
        fa: "دموی UXِ امضاییِ Grok را ضبط کردم: حالتِ Fun، آکاردئونِ تفکرِ قابلِ دیدن، و چیپ‌هایِ دنباله‌رو. یک ریپلایِ ویدیویی به پرتکرارترین سوال: «این چطور با یک جمله کار می‌کند؟» — جواب در ۹ ثانیه.",
        en: "I recorded the Grok signature UX demo: Fun Mode, the visible think accordion, and ghost follow-up chips. A video reply to the most repeated question: 'how does this work with one sentence?' — answer in 9 seconds.",
        cmd: None,
        social: Some(r#"{"platform":"x","handle":"@ImXforevr","kind":"ux_demo","text_en":"xA answers with a visible thinking accordion, live citations, and a split artifact canvas. Fun Mode makes a 9-year-old laugh; Mairon mode ships 100/100. No API key needed.","text_fa":"xA با آکاردئونِ تفکرِ مرئی، استنادِ زنده و بومِ آرتیفکتِ دوپاره پاسخ می‌دهد. حالتِ Fun بچهٔ ۹ساله را می‌خنداند؛ حالتِ مایرون ۱۰۰/۱۰۰ محوصل می‌کند. بدونِ نیاز به کلید.","views":21500,"likes":1870}"#),
        stage: PipelineStage::ViralLaunch,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 184,
        visits_delta: 7_400,
        signups_delta: 51,
        purchases_delta: 0,
    },
    // ── تیک ۱۴ ──
    TickSpec {
        second: 50400,
        speaker: FounderPersona::CzBinance,
        fa: "تسویهٔ آنیِ صبح را بستیم: ۱۰٫۰۰ دلار حجمِ تسویه روی Base USDC و استارز — اختلافِ ۱٫۵٪ یعنی ۰٫۱۵ دلار به خزانه، شفاف روی داشبورد. جریانِ درآمدِ چهارم فعال شد؛ کوچک اما واقعی و قابلِ تکرار.",
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
    // ── تیک ۱۵ ──
    TickSpec {
        second: 54000,
        speaker: FounderPersona::JeffBezos,
        fa: "سازندهٔ دوم onboard شد: یک توسعه‌دهندهٔ اتوماسیون که قالب‌هایِ DAG می‌فروشد. صفحهٔ فروشنده را با خودش بازسازی کردیم — دکمهٔ «بخر» در نیمهٔ بالاییِ صفحه، بدونِ هیچ اصطلاحِ فنی. قیفِ تبدیل: بازدید → ثبت‌نام → خرید، سه قلابِ واقعی.",
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
    // ── تیک ۱۶ ──
    TickSpec {
        second: 57600,
        speaker: FounderPersona::JeffBezos,
        fa: "فروشِ سوم — بدونِ رفرال، ۴٫۵۰ دلار: قالبِ اتوماسیونِ DAG. تقسیمِ همان ثانیه: ۳٫۶۰ دلار سازنده، ۰٫۹۰ دلار خزانه (۱۰٪ مالیاتِ بازار + ۱۰٪ روتینگِ آفیلیتِ ناموجه). مجموعِ خزانه تا این لحظه: ۳٫۱۵ دلار.",
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
    // ── تیک ۱۷ ──
    TickSpec {
        second: 61200,
        speaker: FounderPersona::ElonMusk,
        fa: "HUDِ دوم را رندر کردم — پیش‌تنِ quant-vibe-terminal برای مسابقهٔ کوانتوم. همان مشِ فیبوناچی، این بار با تیکِرهایِ زنده. هر ویدئو یک دعوت‌نامهٔ ۶۰ثانیه‌ای به مخزن است؛ این بازدیدها به ثبت‌نام تبدیل می‌شوند.",
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
    // ── تیک ۱۸ ──
    TickSpec {
        second: 64800,
        speaker: FounderPersona::CzBinance,
        fa: "بوستِ دوم — این بار با استارزِ تلگرام: سازندهٔ باتِ ترید ۳٫۰۰ دلار XTR برای اسلاتِ ویژه پرداخت کرد؛ ۱۰۰٪ به خزانه. استارز یعنی اصطکاکِ صفر برای کاربرِ بدونِ کریپتو — دقیقاً ریلِ رویِ نقشه.",
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
    // ── تیک ۱۹ ──
    TickSpec {
        second: 68400,
        speaker: FounderPersona::JeffBezos,
        fa: "لحظهٔ قهرمانیِ چرخ‌دنده: یک کاربرِ ۹ساله با دکمه‌هایِ ToySpark کتابش را ساخت و در همان پنل منتشر کرد — بدونِ حتی یک خط کد. پدرش در X نوشت: «این اولین محصولیِ است که بچه‌ام از صفر فروخت.» این پیامِ برندِ ماست، نه تبلیغِ ما.",
        en: "The flywheel's hero moment: a 9-year-old built a storybook with the ToySpark buttons and published it in the same panel — without a single line of code. Her dad posted on X: 'this is the first product my kid sold from zero.' That is our brand message, not our ad.",
        cmd: None,
        social: Some(r#"{"platform":"telegram","handle":"@KiaAgentX","kind":"toy_spark_hero","text_en":"A 9-year-old published her first storybook today — built with ToySpark buttons, sold in the marketplace, 80% went to her wallet instantly. Always Day 1.","text_fa":"یک بچهٔ ۹ساله امروز اولین کتابش را منتشر کرد — با دکمه‌هایِ ToySpark ساخت، در بازارگاه فروخت، و ۸۰٪ همان لحظه به ولتش رفت. همیشه روزِ اول.","views":12700,"likes":2140}"#),
        stage: PipelineStage::CreatorOnboarding,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 457,
        visits_delta: 9_900,
        signups_delta: 128,
        purchases_delta: 0,
    },
    // ── تیک ۲۰ ──
    TickSpec {
        second: 72000,
        speaker: FounderPersona::CzBinance,
        fa: "فروشِ چهارم — با رفرال، ۶٫۰۰ دلار: باتِ تریدِ کوانتوم. تقسیم: ۴٫۸۰ سازنده، ۰٫۶۰ مالیاتِ بازار، ۰٫۶۰ آفیلیت. حلقهٔ رفرال دارد کار می‌کند؛ مدلِ ۱۰٪–۵۰٪ برای طبقاتِ بالاترِ آفیلیت روی نقشه است.",
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
    // ── تیک ۲۱ ──
    TickSpec {
        second: 75600,
        speaker: FounderPersona::JeffBezos,
        fa: "فروشِ پنجم — بزرگ‌ترینِ روز: ۹٫۰۰ دلار بدونِ رفرال برای پکیجِ «ربات + قالب + بوست». تقسیمِ همان ثانیه: ۷٫۲۰ سازنده، ۱٫۸۰ خزانه. اقتصادِ واحد قشنگ است: هزینهٔ نهاییِ ما تقریباً صفر، چون مخزنِ خودمان اپ‌ها را می‌سازد.",
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
    // ── تیک ۲۲ ──
    TickSpec {
        second: 79200,
        speaker: FounderPersona::ElonMusk,
        fa: "رشتهٔ جمع‌بندیِ روزِ اول با @ImXforevr منتشر شد: یک دقیقه HUD، پنج عددِ واقعی — ۷۵۰ تست، ۵ فروش، ۹٫۵۵ دلار درآمدِ خزانه، ۱٬۵۲۷ کاربر، صفر KYC. بدونِ هیچ عددِ جعلی؛ همه‌چیز قابلِ بازبینی روی مخزن و زنجیره.",
        en: "The Day-1 recap thread with @ImXforevr is live: one minute of HUD, five real numbers — 750 tests, 5 sales, $9.55 treasury revenue, 1,527 users, zero KYC. No made-up numbers; everything re-derivable from the repo and the chain.",
        cmd: None,
        social: Some(r#"{"platform":"x","handle":"@ImXforevr","kind":"day1_recap","text_en":"Day 0 → Day 1 recap with @elonmusk @cz_binance @JeffBezos: 750 tests green, 5 real sales, $9.55 sovereign treasury revenue from $0.00, 1,527 users. Every number auditable. Keep building.","text_fa":"خلاصهٔ روزِ ۰ → روزِ ۱ با تیمِ کامل: ۷۵۰ تستِ سبز، ۵ فروشِ واقعی، ۹٫۵۵ دلار درآمدِ خزانه از صفر، ۱٬۵۲۷ کاربر. هر عدد قابلِ حسابرسی. به ساخت ادامه بدهید.","views":96400,"likes":8210}"#),
        stage: PipelineStage::ViralLaunch,
        stream: RevenueStream::None,
        volume_micros: 0,
        referred: false,
        users_delta: 127,
        visits_delta: 11_800,
        signups_delta: 91,
        purchases_delta: 0,
    },
    // ── تیک ۲۳ — مُهرِ روزِ اول ──
    TickSpec {
        second: 82800,
        speaker: FounderPersona::ElonMusk,
        fa: "روزِ اول مُهر شد — و هیچ عددِ جعلی‌ای در آن نیست: از $0.00 شروع کردیم و با ۸ تراکنشِ علّی به $9.55 درآمدِ خزانه رسیدیم؛ سازنده‌ها $22.00 گرفتند و آفیلیت‌ها $1.10. ولتِ عملیاتی: $42.55 روی زنجیره + $15 نقد + $350 قفل. فردا دوباره از روزِ اول شروع می‌کنیم — همیشه روزِ اول.",
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

/// سناریویِ کانونیِ ۲۴ تیک با همهٔ ارقامِ مشتق‌شده (دلتاها، جمع‌هایِ
/// تجمعی، برش‌هایِ منشور) — منبعِ واحدِ حقیقتِ سرور و اپِ HTML.
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

// ── خطاهایِ تایپ‌شده ─────────────────────────────────────────────────────────

/// خطاهایِ اتاقِ جنگ — همهٔ مسیرهایِ ناموفقِ تایپ‌شده.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WarRoomError {
    /// دستورِ خالی — رئیس چیزی نفرستاده است.
    #[error("chairman directive is empty")]
    EmptyDirective,
    /// دستورِ بیش از حدِ بلند — سقفِ وایر.
    #[error("chairman directive exceeds the {0}-charter wire cap")]
    DirectiveTooLong(usize),
    /// ضریبِ سرعتِ نامعتبر — فقط ۱، ۲ و ۵.
    #[error("unknown speed multiplier {0}: allowed 1, 2, 5")]
    UnknownSpeed(u32),
    /// سناریو تمام شده — هر ۲۴ تیک اجرا شده است.
    #[error("scenario exhausted: all 24 Day-0 ticks already executed")]
    ScenarioExhausted,
    /// جهشِ زمان — سقفِ یک‌ساعته در هر تیک نقض شد (قاعدهٔ ضدِ-جهش).
    #[error("time jump detected: delta {0}s exceeds the 3600s hourly ceiling")]
    TimeJumpDetected(i64),
}

// ── وضعیتِ اتاقِ جنگ ─────────────────────────────────────────────────────────

/// وضعیتِ زندهٔ اتاقِ جنگ — نشانگرِ تیک، ساعتِ شبیه‌سازی، دفترِ ولت،
/// ترنسکریپتِ اجراشده و کنترل‌هایِ پخش (`Play 1x`، `2x`، `5x`، `Reset`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomState {
    /// نشانگرِ تیکِ بعدی برای اجرا — `0` یعنی روزِ ۰، ثانیهٔ ۰، بکر.
    pub tick_cursor: usize,
    /// ثانیهٔ سپری‌شدهٔ شبیه‌سازی — همیشه == ثانیهٔ آخرین تیکِ اجراشده.
    pub elapsed_seconds: i64,
    /// دفترِ ولتِ سازمان — فقط با برشِ منشور تغییر می‌کند.
    pub wallet: OrganizationWalletLedger,
    /// ترنسکریپتِ تیک‌هایِ اجراشده — به ترتیبِ علّی.
    pub transcript: Vec<SimulationTickEvent>,
    /// پخشِ خودکار روشن است؟ (`Play 1x` / `Pause`)
    pub playing: bool,
    /// ضریبِ سرعت — فقط ۱، ۲ یا ۵.
    pub speed_multiplier: u32,
    /// تعدادِ دستورهایِ رئیسِ اجراشده در این نشست.
    pub directives_executed: u32,
}

impl WarRoomState {
    /// وضعیتِ تازهٔ روزِ ۰، ثانیهٔ ۰ — `$0.00` درآمد، `$33.00` بذرِ کریپتو،
    /// `$15.00` نقد و `$350.00` ذخیرهٔ قفل؛ هیچ تیکی اجرا نشده.
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

    /// پخشِ خودکار را روشن می‌کند (`Play`).
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// پخشِ خودکار را متوقف می‌کند (`Pause`).
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// ضریبِ سرعت را می‌گذارد — فقط ۱، ۲ و ۵ مجازند (`Play 1x` / `2x` / `5x`).
    ///
    /// # Errors
    /// [`WarRoomError::UnknownSpeed`] برای هر ضریبی جز ۱، ۲، ۵.
    pub fn set_speed(&mut self, multiplier: u32) -> Result<(), WarRoomError> {
        if multiplier != 1 && multiplier != 2 && multiplier != 5 {
            return Err(WarRoomError::UnknownSpeed(multiplier));
        }
        self.speed_multiplier = multiplier;
        Ok(())
    }

    /// بازنشانی به «روز ۰، ثانیهٔ ۰» — وضعیتِ بکرِ صادقانه (`Reset`).
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

    /// درصدِ پیشرفتِ گام‌ها — `0..=100`.
    #[must_use]
    pub fn progress_percent(&self) -> u8 {
        u8::try_from(self.tick_cursor * 100 / CANONICAL_TICK_COUNT).unwrap_or(100)
    }

    /// **پیشرویِ علّیِ تک‌گام** — دقیقاً یک تیکِ بعدی را اجرا می‌کند.
    ///
    /// گاردِ ضدِ-جهش: فاصلهٔ ثانیه‌ایِ هر تیک با قبلی هرگز از ۳۶۰۰ بیشتر
    /// نیست؛ دفترِ ولت فقط با برشِ منشور تغییر می‌کند.
    ///
    /// # Errors
    /// [`WarRoomError::ScenarioExhausted`] وقتی ۲۴ تیک تمام شده؛
    /// [`WarRoomError::TimeJumpDetected`] اگر فاصلهٔ تیکِ بعدی سقفِ ساعتی را نقض کند.
    pub fn advance_single_tick(&mut self) -> Result<SimulationTickEvent, WarRoomError> {
        if self.tick_cursor >= CANONICAL_TICK_COUNT {
            return Err(WarRoomError::ScenarioExhausted);
        }
        let spec = &DAY0_TICKS[self.tick_cursor];
        let delta = spec.second - self.elapsed_seconds;
        if delta > MAX_TICK_DELTA_SECONDS {
            return Err(WarRoomError::TimeJumpDetected(delta));
        }
        // برشِ منشور از حجمِ خامِ تیک «بازمحاسبه» می‌شود — هیچ عددی
        // از پیشِ ذخیره‌شده بدونِ اثبات قابلِ اعتماد نیست.
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

    /// تزریقِ **دستورِ تاکتیکیِ رئیس** — پیام به ترنسکریپتِ پخش می‌رود و
    /// هر سه هم‌بنیان‌گذار در-شخصیت (FA/EN) پاسخ می‌دهند.
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
        headline_fa: "سیستم از ۰ تا ۱۰۰ — به ساده‌ترین زبانِ دنیا".to_owned(),
        headline_en: "The system from 0 to 100 — in the simplest possible language".to_owned(),
        charter_fa: "قراردادِ پول ساده است: هر فروش، ۸۰٪ سازنده و ۲۰٪ ولتِ سازمان — همان لحظه، روی Base (8453) یا استارزِ تلگرام.".to_owned(),
        charter_en: "The money charter is simple: every sale pays 80% to the creator and 20% to the organization wallet — instantly, on Base (8453) or Telegram Stars.".to_owned(),
        steps: [
            ZeroLiteracyPipelineStep::StepAUserIntent,
            ZeroLiteracyPipelineStep::StepBAvatarFactory,
            ZeroLiteracyPipelineStep::StepCStoreAndSocial,
            ZeroLiteracyPipelineStep::StepDAutoSplit,
        ],
    }
}

/// بستهٔ توضیحِ «۰ تا ۱۰۰» — چهار گامِ سوادِ صفر + سرخط + منشورِ پول.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZeroToHundredExplainer {
    /// لوکالِ بسته.
    pub locale: Locale,
    /// سرخطِ فارسی.
    pub headline_fa: String,
    /// سرخطِ انگلیسی.
    pub headline_en: String,
    /// جملهٔ منشورِ پول — فارسی.
    pub charter_fa: String,
    /// جملهٔ منشورِ پول — انگلیسی.
    pub charter_en: String,
    /// چهار گامِ خطِ لوله، به ترتیبِ الف → د.
    pub steps: [ZeroLiteracyPipelineStep; 4],
}

// ── موتورِ چتِ رئیس ──────────────────────────────────────────────────────────

/// قصدِ تشخیص‌داده‌شدهٔ دستورِ رئیس — کلاسه‌بندیِ قطعیِ آفلاین.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChairmanIntent {
    /// ویدئوی HUD را روی X وایرال کن.
    ViralHudLaunch,
    /// درگاهِ Base USDC و استارزِ تلگرام را فعال کن.
    PaymentRails,
    /// سیستم را ساده کن تا بچهٔ ۹ساله هم بفهمد.
    SimplifyForBeginners,
    /// گزارشِ لحظه‌ایِ ولت و گامِ بعدی.
    StatusReport,
    /// اجرای استراتژیِ تریدِ کوانتومِ Vibe-Trading.
    QuantTrading,
    /// توضیحِ ۰ تا ۱۰۰ سیستم به ساده‌ترین زبان.
    ExplainSimple,
    /// فرمانِ عمومیِ اجرا — گامِ بعدیِ عملیاتی.
    ShipNextStep,
}

impl ChairmanIntent {
    /// کلیدواژه‌هایِ فارسیِ هر قصد — به ترتیبِ اعلان.
    pub const KEYWORDS_FA: [&'static str; 7] = [
        "وایرال",
        "درگاه",
        "ساده",
        "گزارش",
        "ترید",
        "توضیح",
        "اجرا",
    ];

    /// کلیدواژه‌هایِ انگلیسیِ هر قصد — به ترتیبِ اعلان.
    pub const KEYWORDS_EN: [&'static str; 7] = [
        "viral",
        "gateway",
        "simple",
        "report",
        "quant",
        "explain",
        "ship",
    ];

    /// همهٔ قصل‌ها به ترتیبِ اعلان — برایِ تای-بریکِ پایدار.
    pub const ALL: [ChairmanIntent; 7] = [
        Self::ViralHudLaunch,
        Self::PaymentRails,
        Self::SimplifyForBeginners,
        Self::StatusReport,
        Self::QuantTrading,
        Self::ExplainSimple,
        Self::ShipNextStep,
    ];

    /// کلاسه‌بندیِ قطعی — بیشترینِ برخوردِ کلیدواژه (فارسی + انگلیسی)،
    /// تای-بریک به ترتیبِ اعلان؛ بدونِ برخورد → [`ChairmanIntent::ShipNextStep`].
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
            // کلیدواژه‌هایِ اضافیِ پرتکرارِ هر قصد — وزنِ دوم.
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

    /// کلیدواژه‌هایِ کمکیِ هر قصد — «hud»، «usdc»، «بچه» و … .
    fn extra_keywords(intent: Self) -> &'static [&'static str] {
        match intent {
            Self::ViralHudLaunch => &["hud", "ویدیو", "video", "x", "neural"],
            Self::PaymentRails => &["usdc", "استارز", "stars", "پرداخت", "payment", "8453"],
            Self::SimplifyForBeginners => &["بچه", "کودک", "child", "toyspark", "eli5", "۹ساله"],
            Self::StatusReport => &["وضعیت", "ولت", "wallet", "status", "گام", "step"],
            Self::QuantTrading => &["کوانتوم", "vibe", "trade", "trading", "ccxt", "backtest"],
            Self::ExplainSimple => &["صفر", "zero", "بفهم", "understand", "۰", "100"],
            Self::ShipNextStep => &["next", "بعدی", "launch", "deploy", "فردا"],
        }
    }

    /// برچسبِ فارسیِ قصد.
    #[must_use]
    pub const fn label_fa(self) -> &'static str {
        match self {
            Self::ViralHudLaunch => "پرتابِ ویروسیِ HUD عصبی",
            Self::PaymentRails => "فعال‌سازیِ ریل‌هایِ پرداخت",
            Self::SimplifyForBeginners => "سادگی برای تازه‌کارها",
            Self::StatusReport => "گزارشِ لحظه‌ای",
            Self::QuantTrading => "اجرای کوانتومِ Vibe-Trading",
            Self::ExplainSimple => "توضیحِ ۰ تا ۱۰۰",
            Self::ShipNextStep => "گامِ بعدیِ اجرا",
        }
    }

    /// برچسبِ انگلیسیِ قصد.
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

/// پاسخِ در-شخصیتِ یک هم‌بنیان‌گذار به دستورِ رئیس — دوزبانه.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FounderReply {
    /// پاسخ‌دهنده.
    pub speaker: FounderPersona,
    /// متنِ فارسی.
    pub message_fa: String,
    /// متنِ انگلیسی — پاریتهٔ کامل.
    pub message_en: String,
    /// فرمانِ ترمینالِ پیشنهادی — در صورتِ اجرای فنی.
    pub terminal_cmd: Option<String>,
    /// پستِ اجتماعیِ پیشنهادی — در صورتِ اکشنِ ویروسی.
    pub social_post_json: Option<String>,
}

/// گزارشِ کاملِ یک دستورِ رئیس — همان ردیفِ `"ChairmanDirectiveLog"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChairmanDirectiveLog {
    /// متنِ خامِ دستور.
    pub directive_text: String,
    /// قصدِ تشخیص‌داده‌شده.
    pub intent: ChairmanIntent,
    /// سه پاسخِ در-شخصیت (ایلان، CZ، جف).
    pub replies: Vec<FounderReply>,
    /// تیکِ فعالِ زمانِ تزریق.
    pub triggered_tick: usize,
    /// ثانیهٔ شبیه‌سازیِ زمانِ تزریق.
    pub simulation_second: i64,
    /// لوکالِ پاسخ‌ها.
    pub locale: Locale,
}

/// موتورِ چتِ رئیس — پاسخِ در-شخصیتِ هر سه هم‌بنیان‌گذار، ۱۰۰٪ آفلاین و
/// بدونِ کلید؛ برایِ تولیدِ بی‌نهایت، مودالِ LLMِ زندهٔ اپِ HTML روی
/// Pollinations/OpenRouter/لوکال سوار است (اختیاری، خارج از این کرنل).
pub struct ChairmanChatEngine;

impl ChairmanChatEngine {
    /// پاسخِ در-شخصیتِ ایلان ماسک — سرعت، HUD و ویروسی‌شدن.
    fn respond_elon(intent: ChairmanIntent) -> FounderReply {
        let (fa, en, cmd, social): (&'static str, &'static str, Option<&'static str>, Option<&'static str>) =
            match intent {
                ChairmanIntent::ViralHudLaunch => (
                    "دستورِ واضح است: HUDِ عصبی را با پیش‌تنِ monopoly-core دوباره رندر می‌کنم و رشتهٔ X را با تایم‌لاینِ چهارفازی می‌بندم. ویدئو زیرِ ۹ ثانیه باید ۱۸۰ گره را نشان بدهد وگرنه بازنویسی می‌کنم. سرعت، اولویت است.",
                    "Clear directive: I re-render the Neural HUD with the monopoly-core preset and close the X thread with the 4-phase timeline. If the video doesn't show all 180 nodes under 9 seconds, I rewrite it. Velocity first.",
                    Some("python3 tools/neural-hud/render_neural_hud.py html --preset kia-monopoly-core --aspect 9:16 --out hud_viral.html"),
                    Some(r#"{"platform":"x","handle":"@elonmusk","kind":"chairman_reply","text_en":"CHAIRMAN DIRECTIVE RECEIVED: Neural HUD viral launch in progress. 180 nodes, 4 phases, 60 seconds. Watch this space.","text_fa":"دستورِ رئیس دریافت شد: پرتابِ ویروسیِ HUD عصبی در جریان است. ۱۸۰ گره، ۴ فاز، ۶۰ ثانیه.","views":0,"likes":0}"#),
                ),
                ChairmanIntent::PaymentRails => (
                    "ریل‌هایِ پرداخت باید زیرِ یک ثانیه پاس بدهند؛ اگر تأخیرِ استارز بالای ۸۰۰ms رفت، مینی‌اپ را با پیش‌بارگذاریِ نشست اصلاح می‌کنم. سرعتِ پرداخت = نرخِ تبدیل.",
                    "Payment rails must answer under one second; if Stars latency goes above 800ms I fix the mini-app with session prewarming. Payment speed = conversion rate.",
                    None,
                    None,
                ),
                ChairmanIntent::SimplifyForBeginners => (
                    "بیرونِ راندنِ هر کلیکِ اضافه: دکمه‌هایِ ToySpark به ۶۴ پیکسل می‌رسند و جریانِ یک‌ضربه‌ایِ کتابِ فارسی می‌ماند. هر مرحله‌ای که کلمهٔ فنی دارد، حذف می‌شود.",
                    "Kill every extra click: ToySpark buttons go to 64px and the one-tap Persian storybook flow stays. Any step with a technical word gets deleted.",
                    None,
                    None,
                ),
                ChairmanIntent::StatusReport => (
                    "از سمتِ من: ۷۵۰ تست سبز، HUDِ رندرشده، و صفِ رندرِ بعدی. گلوگاهِ سرعت، ترمینال است؛ یک دستورِ CI دیگر اضافه می‌کنم تا هر تیک خودش را اثبات کند.",
                    "From my side: 750 tests green, HUD rendered, next render queued. The velocity bottleneck is the terminal; I add one more CI command so every tick proves itself.",
                    Some("cargo test --workspace --quiet 2>&1 | tail -4"),
                    None,
                ),
                ChairmanIntent::QuantTrading => (
                    "کوانت باید سریع و دیدنی باشد: ترمینالِ quant-vibe را با HUDِ زنده می‌بندم تا هر بک‌تستِ علّی مثلِ یک پخشِ زنده به نظر برسد. سرعتِ دیتافید را می‌سنجم.",
                    "Quant must be fast and watchable: I wire the quant-vibe terminal to a live HUD so every causal backtest looks like a live broadcast. I measure feed latency.",
                    Some("python3 tools/neural-hud/render_neural_hud.py html --preset quant-vibe-terminal --aspect 16:9 --out hud_quant.html"),
                    None,
                ),
                ChairmanIntent::ExplainSimple => (
                    "توضیحِ من در یک خط: «یک جمله می‌گویی، سه ربات در چند ثانیه می‌سازند، ویدئو می‌فروشد، و پول همان لحظه ۸۰/۲۰ می‌شکند.» هر جمله‌ای که بلندتر از این شود، UI خراب است.",
                    "My one-line explanation: 'you say one sentence, three robots build it in seconds, a video sells it, and the money splits 80/20 instantly.' Any sentence longer than that is a broken UI.",
                    None,
                    None,
                ),
                ChairmanIntent::ShipNextStep => (
                    "گامِ بعدیِ من: HUDِ سوم با پیش‌تنِ genome و رشتهٔ فنیِ «۷۵۰ تست، صفر unsafe» — مهندسیِ خودش بازاریابی است. اجرا می‌کنم و نتیجه را می‌سنجم.",
                    "My next step: HUD #3 with the genome preset and the technical thread '750 tests, zero unsafe' — engineering is its own marketing. I ship it and measure.",
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

    /// پاسخِ در-شخصیتِ CZ — نقدینگی، Base 8453 و ریل‌هایِ پرداخت.
    fn respond_cz(intent: ChairmanIntent) -> FounderReply {
        let (fa, en, cmd): (&'static str, &'static str, Option<&'static str>) = match intent {
            ChairmanIntent::ViralHudLaunch => (
                "از سمتِ من: ویدئوی HUD را کنارِ هشِ وریفایِ Base 8453 سنجاق می‌کنم؛ هر کسی که بپرسد «پول کجاست؟»، لینکِ تراکنشِ روی-زنجیره را می‌فرستم. اعتماد، سوختِ ویروسی‌شدن است.",
                "From my side: I pin the HUD video next to the Base 8453 verification hash; anyone asking 'where is the money?' gets the on-chain transaction link. Trust is the fuel of virality.",
                None,
            ),
            ChairmanIntent::PaymentRails => (
                "دروازه را همین حالا باز می‌کنم: استارزِ تلگرام (XTR) برایِ خریدِ آنی، Base USDC (0x8335...2913، زنجیرهٔ 8453) برایِ تسویه. اختلافِ ۱٫۵٪ شفاف، بوست ۱۰۰٪ خزانه، فروش ۸۰/۲۰. صندوق‌ها SAFU.",
                "Opening the gateway right now: Telegram Stars (XTR) for instant purchases, Base USDC (0x8335...2913, chain 8453) for settlement. The 1.5% spread is transparent, boosts go 100% to treasury, sales split 80/20. Funds are SAFU.",
                Some("curl -s https://kia-api.up.railway.app/api/v1/monopoly/charter-and-base | head -c 400"),
            ),
            ChairmanIntent::SimplifyForBeginners => (
                "سادگیِ پول مهم‌تر است: کاربرِ تازه فقط باید ببیند «می‌خرم → می‌سازم → ۸۰٪ مالِ من». ریلِ استارز را پیش‌فرض می‌کنم؛ کریپتو برایِ حرفه‌ای‌ها ثانویه می‌ماند.",
                "Money simplicity wins: a new user should only see 'I buy → it builds → 80% is mine'. I make Stars the default rail; crypto stays secondary for pros.",
                None,
            ),
            ChairmanIntent::StatusReport => (
                "گزارشِ نقدینگی: بذرِ $33.00 روی زنجیره، درآمدِ خالص از $0.00 تا لحظهٔ گزارش، چهار جریانِ درآمد آماده. هر افزایشی با هشِ 8453 قابلِ ردیابی است — صفرِ دست‌کاری.",
                "Liquidity report: $33.00 seed on-chain, net revenue from $0.00 as of this report, four revenue streams ready. Every increase is traceable with a 8453 hash — zero tampering.",
                Some("curl -s https://kia-api.up.railway.app/api/v1/war-room/state | head -c 400"),
            ),
            ChairmanIntent::QuantTrading => (
                "استراتژیِ Vibe-Trading را روی فیدِ CCXT اجرا می‌کنم: سفارش‌نامهٔ زنده، بک‌تستِ علّی با گیتِ شارپِ ≥ ۱٫۲۰، و ثبتِ کامل روی دفتر. بردِ مسابقه = فروشِ بات با تقسیمِ ۸۰/۲۰.",
                "I run the Vibe-Trading strategy on the CCXT feed: live orderbook, causal backtest with the Sharpe >= 1.20 gate, full ledger recording. Competition win = bot sales with the 80/20 split.",
                Some("node ccxt-feed.js binance BTC/USDC --depth 5 --json"),
            ),
            ChairmanIntent::ExplainSimple => (
                "توضیحِ پول در یک خط: «۸۰٪ مالِ سازنده، ۲۰٪ مالِ پلتفرم، تسویه با استارز یا USDC روی Base؛ همه‌چیز با هش قابلِ اثبات.» بدونِ KYC، بدونِ اصطلاحِ بانکی.",
                "The money in one line: '80% to the creator, 20% to the platform, settled with Stars or USDC on Base; everything provable with a hash.' No KYC, no banking jargon.",
                None,
            ),
            ChairmanIntent::ShipNextStep => (
                "گامِ بعدیِ من: دومین مسابقهٔ کوانتوم با جایزهٔ اسلاتِ ویژه؛ وریفایِ دومین قرارداد روی Base و بستنِ گزارشِ هفتگیِ نقدینگی. ساختن ادامه دارد.",
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

    /// پاسخِ در-شخصیتِ جف بزوس — چرخ‌دنده، مشتری و سادگی.
    fn respond_jeff(intent: ChairmanIntent) -> FounderReply {
        let (fa, en): (&'static str, &'static str) = match intent {
            ChairmanIntent::ViralHudLaunch => (
                "ویدئو را به آزمونِ ۵ثانیه‌ای وصل می‌کنم: زیرِ ویدئو یک جملهٔ ثابت — «کاربر می‌گوید، کارخانه می‌سازد، پول ۸۰/۲۰ تقسیم می‌شود.» اگر بیننده در ۵ ثانیه این سه را بفهمد، بازدید به خرید تبدیل می‌شود.",
                "I wire the video into the 5-second test: one fixed caption under it — 'the user asks, the factory builds, the money splits 80/20.' If a viewer gets those three in 5 seconds, views convert into purchases.",
            ),
            ChairmanIntent::PaymentRails => (
                "هر ریلِ پرداخت باید با یک جمله توضیح‌پذیر باشد: «استارز = خریدِ یک‌ضربه‌ای؛ کریپتو = تسویهٔ جهانی.» صفحهٔ پرداخت را با همین دو جمله بازنویسی می‌کنم و نرخِ رهاسازیِ سبد را می‌سنجم.",
                "Every payment rail must be explainable in one sentence: 'Stars = one-tap buy; crypto = global settlement.' I rewrite the checkout page around those two lines and measure cart drop-off.",
            ),
            ChairmanIntent::SimplifyForBeginners => (
                "آزمونِ بچهٔ ۹ساله را خودم اجرا می‌کنم: صفحهٔ اصلی باید در ۵ ثانیه سه جواب بدهد — چه‌کاری می‌توانم بکنم؟ چقدر می‌گیرم؟ پولِ من کجاست؟ هر جزئیاتی جز این سه، به پایینِ صفحه می‌رود.",
                "I personally run the 9-year-old test: the landing must answer three things in 5 seconds — what can I do? how much do I earn? where is my money? Everything else goes below the fold.",
            ),
            ChairmanIntent::StatusReport => (
                "گزارشِ چرخ‌دنده: قیفِ بازدید→ثبت‌نام→خرید زنده است؛ سازنده‌هایِ فعال و نرخِ تبدیل را با هر تیک می‌سنجم. گامِ بعدی: اسلاتِ ویژه را برایِ بهترین محصولِ هفته آزاد می‌کنم.",
                "Flywheel report: the visits→signups→purchases funnel is live; I measure active creators and conversion on every tick. Next step: free the featured slot for the week's best product.",
            ),
            ChairmanIntent::QuantTrading => (
                "برایِ کاربرِ عادی، «کوانت» را به «رباتِ پس‌انداز» ترجمه می‌کنم: یک دکمه، یک عددِ ساده — «این هفته چقدر رشد کرد؟» — و یک توضیحِ یک‌خطیِ ریسک. حرفه‌ای‌ها جزئیاتِ CCXT را می‌بینند؛ بقیه فقط نتیجه.",
                "For normal users I translate 'quant' into 'a savings robot': one button, one plain number — 'how much did it grow this week?' — and a one-line risk note. Pros see the CCXT details; everyone else sees the outcome.",
            ),
            ChairmanIntent::ExplainSimple => (
                "توضیحِ چرخ‌دنده در یک خط: «بچه‌ها با دکمه‌هایِ بزرگ می‌سازند، فروشگاه می‌فروشد، ولت خودش تقسیم می‌کند؛ هر فروش، موتور را برایِ همه سریع‌تر می‌کند.» این کلِ داستان است.",
                "The flywheel in one line: 'kids build with big buttons, the store sells, the wallet splits by itself; every sale makes the engine faster for everyone.' That is the whole story.",
            ),
            ChairmanIntent::ShipNextStep => (
                "گامِ بعدیِ من: بازکردنِ اسلاتِ ویژهٔ هفته و تستِ A/B رویِ دکمهٔ «انتشار یک‌کلیکی»؛ تبدیلِ هر بازدیدِ HUD به یک سازندهٔ جدید. همیشه روزِ اول.",
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

    /// پاسخِ پرسونایِ ثبت‌نشده — صادقانه، بدونِ توهم.
    fn respond_unmapped() -> FounderReply {
        FounderReply {
            speaker: FounderPersona::Chairman,
            message_fa: "پاسخِ در-شخصیتِ ثبت‌نشده — به‌جایِ توهم، صادقانه: این پرسونا در جدولِ ثابت نیست؛ لطفاً یکی از سه هم‌بنیان‌گذار را انتخاب کنید.".to_owned(),
            message_en: "Unmapped in-character reply — honestly, instead of hallucinating: this persona is not in the fixed table; please pick one of the three co-founders.".to_owned(),
            terminal_cmd: None,
            social_post_json: None,
        }
    }

    /// پاسخِ در-شخصیتِ یک پرسونا به یک قصد — جدولِ ثابتِ ۳×۷.
    ///
    /// # Errors
    /// هرگز — برایِ هم‌سانیِ امضا با بقیهٔ موتور، `Result` نگه داشته شده و
    /// همیشه `Ok` برمی‌گرداند.
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

    /// پاسخِ هر سه هم‌بنیان‌گذار به یک قصد — ترتیبِ ثابت: ایلان، CZ، جف.
    ///
    /// هر دو زبان همیشه حمل می‌شوند تا پاریتهٔ دوزبانه در ممیزی اثبات بماند؛
    /// انتخابِ زبانِ نمایش کارِ لایهٔ UI است.
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

// ── تست‌ها — ۲۶ گاردِ قطعیِ اتاقِ جنگ ────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// کلِ سناریو با ترتیبِ ثابت — برایِ همهٔ تست‌ها.
    fn scenario() -> Vec<SimulationTickEvent> {
        canonical_day0_scenario()
    }

    /// اجرای کاملِ ۲۴ تیک روی وضعیتِ تازه.
    fn run_all(state: &mut WarRoomState) {
        for _ in 0..CANONICAL_TICK_COUNT {
            state
                .advance_single_tick()
                .expect("tick must advance within scenario");
        }
    }

    // ── ۱. شروعِ صادقانه ──
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

    // ── ۲. ساختارِ سناریو ──
    #[test]
    fn canonical_scenario_has_24_sequential_ticks() {
        let ticks = scenario();
        assert_eq!(ticks.len(), CANONICAL_TICK_COUNT);
        assert_eq!(CANONICAL_TICK_COUNT, 24, "spec: 24 rich Day-0 operational steps");
        for (i, t) in ticks.iter().enumerate() {
            assert_eq!(t.tick_index as usize, i, "dense tick indices");
        }
    }

    // ── ۳. گاردِ ضدِ-جهشِ زمان ──
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
        // هیچ جهشِ یک‌ساله‌ای وجود ندارد: کلِ سناریو < یک سال.
        assert!(
            ticks[ticks.len() - 1].simulation_second < 31_536_000,
            "the whole scenario must stay inside one year"
        );
    }

    // ── ۴. منشورِ ۲۰/۸۰ روی هر افزایشِ ولت ──
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

    // ── ۵. یکنواییِ تجمعی ──
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

    // ── ۶. مُهرِ روزِ اول — $9.55 ──
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

    // ── ۷. سطل‌هایِ جریان ──
    #[test]
    fn stream_buckets_sum_to_total_earned() {
        let mut state = WarRoomState::new();
        run_all(&mut state);
        assert_eq!(state.wallet.streams_sum_micros(), state.wallet.earned_revenue_micros);
        assert_eq!(state.wallet.earned_revenue_micros, 9_550_000);
        // توزیعِ دقیقِ چهار جریانِ روزِ اول:
        // مالیاتِ بازار: 0.30+0.50+0.45+0.60+0.90 = $2.75
        assert_eq!(state.wallet.stream_marketplace_tax_micros, 2_750_000);
        // روتینگِ آفیلیتِ ناموجه: 0.30+0.45+0.90 = $1.65
        assert_eq!(state.wallet.stream_affiliate_routing_micros, 1_650_000);
        // بوست‌ها: 2.00+3.00 = $5.00
        assert_eq!(state.wallet.stream_featured_boost_micros, 5_000_000);
        // اختلافِ تسویه: $0.15
        assert_eq!(state.wallet.stream_settlement_spread_micros, 150_000);
        // پرداختِ ۸۰٪ سازنده‌ها: 2.40+4.00+3.60+4.80+7.20 = $22.00
        assert_eq!(state.wallet.creator_payouts_micros, 22_000_000);
        // پورسانتِ آفیلیت‌ها: 0.50+0.60 = $1.10
        assert_eq!(state.wallet.affiliate_payouts_micros, 1_100_000);
        // قیفِ نهایی: بازدید ≥ ثبت‌نام ≥ خرید
        assert!(state.wallet.funnel_visits >= state.wallet.funnel_signups);
        assert!(state.wallet.funnel_signups >= state.wallet.funnel_purchases);
        assert_eq!(state.wallet.funnel_purchases, 5, "five real Day-1 sales");
    }

    // ── ۸. پاریتهٔ دوزبانه ──
    #[test]
    fn all_ticks_have_bilingual_parity() {
        for t in scenario() {
            assert_ne!(t.message_fa.trim(), "", "tick {} fa empty", t.tick_index);
            assert_ne!(t.message_en.trim(), "", "tick {} en empty", t.tick_index);
            assert_ne!(t.message_fa, t.message_en, "fa and en must be distinct languages");
            assert!(
                t.message_fa.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c)),
                "tick {} fa must contain Persian script",
                t.tick_index
            );
            assert!(
                !t.message_en
                    .chars()
                    .any(|c| ('\u{0600}'..='\u{06FF}').contains(&c) || ('\u{FB50}'..='\u{FEFF}').contains(&c)),
                "tick {} en must never contain Arabic-script glyphs",
                t.tick_index
            );
        }
    }

    // ── ۹. چرخشِ سه شریک ──
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

    // ── ۱۰. پیشرویِ یکنواختِ مراحل ──
    #[test]
    fn all_eight_campaign_lanes_visited_and_day1_seals() {
        let ticks = scenario();
        // روز با احتراق شروع می‌شود و با مُهرِ روزِ اول تمام — لِین‌هایِ عملیاتی
        // ممکن‌اند در طولِ روز دوباره فعال شوند (چرخهٔ واقعیِ لانچ).
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
        // مُهرِ نهایی فقط یک بار و در تیکِ آخر.
        assert_eq!(
            ticks.iter().filter(|t| t.active_pipeline_stage == PipelineStage::DayOneSealed).count(),
            1,
            "DayOneSealed fires exactly once"
        );
    }

    // ── ۱۱. پیشرویِ علّیِ تک‌گام ──
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

    // ── ۱۲. پایانِ سناریو ──
    #[test]
    fn advance_beyond_scenario_is_exhausted() {
        let mut state = WarRoomState::new();
        run_all(&mut state);
        let err = state.advance_single_tick().expect_err("25th tick must fail");
        assert_eq!(err, WarRoomError::ScenarioExhausted);
    }

    // ── ۱۳. بازنشانیِ صادقانه ──
    #[test]
    fn reset_returns_to_day_zero_honest_state() {
        let mut state = WarRoomState::new();
        state.play();
        state.set_speed(5).expect("5x legal");
        run_all(&mut state);
        state.reset_to_day_zero();
        assert_eq!(state, WarRoomState::new(), "reset is a full honest return");
    }

    // ── ۱۴. سرعت‌هایِ مجاز ──
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

    // ── ۱۵. دستورِ خالی ──
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

    // ── ۱۶. دستورِ غول‌پیکر ──
    #[test]
    fn oversized_directive_is_rejected() {
        let mut state = WarRoomState::new();
        let long = "دستورِ طولانیِ آزمایشی ".repeat(150);
        assert!(long.chars().count() > MAX_DIRECTIVE_CHARS);
        assert_eq!(
            state.inject_chairman_directive(&long, Locale::Fa).unwrap_err(),
            WarRoomError::DirectiveTooLong(MAX_DIRECTIVE_CHARS)
        );
    }

    // ── ۱۷. سه پاسخِ در-شخصیت ──
    #[test]
    fn directive_gets_three_in_character_replies() {
        let mut state = WarRoomState::new();
        state.play();
        state
            .advance_single_tick()
            .expect("warm-up tick so the directive fires mid-scenario");
        let log = state
            .inject_chairman_directive("ویدئوی HUD را روی X وایرال کن", Locale::Fa)
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
        // وایرال → فقط ایلان ترمینالِ HUD دارد.
        assert!(log.replies[0].terminal_cmd.is_some());
        assert!(log.replies[0].social_post_json.is_some());
    }

    // ── ۱۸. کلاسه‌بندیِ فارسیِ شش چیپ ──
    #[test]
    fn fa_directives_classify_all_six_chips() {
        let cases = [
            ("🚀 ایلان: ویدیوی Neural HUD رو توی X وایرال کن", ChairmanIntent::ViralHudLaunch),
            ("💰 CZ: درگاه Base USDC و استارز تلگرام رو فعال کن", ChairmanIntent::PaymentRails),
            ("🛒 جف: سیستم رو طوری ساده کن که یه بچه ۹ ساله هم بفهمه", ChairmanIntent::SimplifyForBeginners),
            ("📊 گزارش لحظه‌ای ولت و قدم بعدی رو بدید", ChairmanIntent::StatusReport),
            ("⚡ اجرای استراتژی ترید کوانتومی Vibe-Trading", ChairmanIntent::QuantTrading),
            ("🧠 توضیح ۰ تا ۱۰۰ سیستم به ساده‌ترین زبان", ChairmanIntent::ExplainSimple),
        ];
        for (text, want) in cases {
            assert_eq!(ChairmanIntent::classify(text), want, "fa classify: {text}");
        }
    }

    // ── ۱۹. کلاسه‌بندیِ انگلیسی ──
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

    // ── ۲۰. وایرِ آمادهٔ JSON ──
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
            .inject_chairman_directive("گزارش ولت رو بده", Locale::En)
            .expect("directive");
        let lv = serde_json::to_value(&log).expect("log json");
        assert!(lv.get("directiveText").is_some());
        assert!(lv.get("triggeredTick").is_some());
    }

    // ── ۲۱. پست‌هایِ اجتماعیِ JSON معتبر ──
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
        // حداقل سه پستِ اجتماعی در روزِ اول — X و تلگرام.
        assert!(scenario().iter().filter(|t| t.social_post_json.is_some()).count() >= 3);
    }

    // ── ۲۲. توضیح‌گرِ ۰ تا ۱۰۰ ──
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
        assert!(bundle.headline_fa.contains('۰'));
        assert!(bundle.headline_en.contains("0 to 100"));
    }

    // ── ۲۳. گامِ د = منشورِ ۸۰/۲۰ ──
    #[test]
    fn eli5_step_d_states_the_80_20_charter() {
        let fa = ZeroLiteracyPipelineStep::StepDAutoSplit.body_fa();
        let en = ZeroLiteracyPipelineStep::StepDAutoSplit.body_en();
        assert!(fa.contains("۸۰٪"), "fa must say 80%");
        assert!(fa.contains("۲۰٪"), "fa must say 20%");
        assert!(en.contains("80%"));
        assert!(en.contains("20%"));
    }

    // ── ۲۴. برشِ منشورِ دقیق ──
    #[test]
    fn charter_split_is_exact_for_unreferred_and_referred() {
        // فروشِ ۳ دلاریِ بدونِ رفرال: ۲۰٪ خزانه، ۸۰٪ سازنده.
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
        // فروشِ ۵ دلاریِ با رفرال: ۱۰٪ خزانه، ۱۰٪ آفیلیت، ۸۰٪ سازنده.
        let r = charter_split(5_000_000, true);
        assert_eq!(r.treasury_delta_micros, 500_000);
        assert_eq!(r.creator_payout_micros, 4_000_000);
        assert_eq!(r.affiliate_payout_micros, 500_000);
        assert_eq!(r.affiliate_routing_micros, 0);
        assert_eq!(
            r.treasury_delta_micros + r.creator_payout_micros + r.affiliate_payout_micros,
            5_000_000
        );
        // بوست: ۱۰۰٪ خزانه. اختلافِ تسویه: ۱۵۰ bps.
        assert_eq!(featured_boost_split(2_000_000).treasury_delta_micros, 2_000_000);
        assert_eq!(settlement_spread_split(10_000_000).treasury_delta_micros, 150_000);
    }

    // ── ۲۵. قالب‌بندیِ انسانیِ دلار ──
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

    // ── ۲۶. فرادادهٔ پرسوناها ──
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
        // منشورِ ثابت‌ها — مقادیرِ ویژِ اسپک.
        assert_eq!(GITHUB_REPO, "KiaAgentX/xA");
        assert_eq!(GITHUB_COMMIT_SHORT, "97b18bd");
        assert_eq!(BASE_CHAIN_ID, 8453);
        assert_eq!(TELEGRAM_STARS_CURRENCY, "XTR");
        assert_eq!(Locale::Fa.direction(), "rtl");
        assert_eq!(Locale::En.direction(), "ltr");
        assert_eq!(Locale::parse("fa"), Locale::Fa);
        assert_eq!(Locale::parse("en"), Locale::En);
        assert_eq!(Locale::parse("anything"), Locale::En);
    }
}
