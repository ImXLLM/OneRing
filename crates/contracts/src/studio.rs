//! قراردادهای استودیوی ادراک‌سیار — ابراستودیوی ۹-مدالیتیِ صفر-هزینه و
//! ویترین پورتفولیو (Wave 5 — Step 2/4).
//!
//! چهار DTO روی وایر:
//! - [`StudioUxProfilesResponse`] — سه لایهٔ شناختی + ۹ مدالیتی + گزارشِ
//!   ۶ درمانِ قضاییِ پورتفولیو؛
//! - [`StudioGenerateRequest`] / [`StudioGenerateResponse`] — تولیدِ زندهٔ
//!   آرتیفکت با باندلِ دوگانهٔ `Fishkal+Cosmos`، بسته‌بندیِ ARDA و فورجِ
//!   مایرون؛
//! - [`PortfolioCatalogResponse`] — ویترینِ غنی‌شده با ارتقای دو
//!   ویترینیِ برتر، توکن‌های گلسِ تاریک و پالتِ فرمانِ کیبورد-اول.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// یک لایهٔ شناختی روی وایر — پروفایلِ مورفِ UX.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CognitiveTierView {
    /// شناسهٔ لایه (`ToySpark` | `CreatorStudio` | `ProTerminal`).
    pub tier: String,
    /// برچسب بومی.
    pub fa_label: String,
    /// باند سنی/هوشی.
    pub age_band: String,
    /// سقفِ اقدامِ هر صفحه.
    pub max_actions_per_screen: u32,
    /// حداقلِ هدفِ لمسی (پیکسل).
    pub min_tap_target_px: u32,
    /// سقفِ واژهٔ هر برچسب.
    pub max_words_per_label: u32,
    /// جادوگرِ صوتی؟
    pub voice_guided: bool,
    /// سندباکسِ صفر-زیان؟
    pub zero_loss_sandbox: bool,
    /// کیبورد-اول؟
    pub keyboard_first: bool,
    /// خلاصهٔ چیدمان.
    pub layout_summary: String,
}

/// یک مدالیتی روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StudioModalityView {
    /// شناسهٔ مدالیتی (`search` | `deep` | … | `sfx`).
    pub modality: String,
    /// برچسب بومی.
    pub fa_label: String,
    /// احرازِ لازم — همیشه `none`.
    pub auth: String,
    /// موتورِ اجرایی.
    pub engine: String,
    /// همیشه `true` — گیتِ صفر-هزینه.
    pub zero_cost: bool,
}

/// یک درمانِ قضایی روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSafetyFixView {
    /// شناسهٔ درمان (`1..=6`).
    pub fix_id: u32,
    /// عنوانِ درمان.
    pub title: String,
    /// وضعیت — همیشه `fixed`.
    pub status: String,
    /// شاهدِ قطعی.
    pub evidence: String,
}

/// گزارشِ ۶ درمانِ قضاییِ `KiaAgentX/portfolio`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSafetyReportView {
    /// شمارِ درمان‌های اعمال‌شده — همیشه `6`.
    pub fixed_count: u32,
    /// همیشه `true`.
    pub all_fixed: bool,
    /// ردیفِ هر درمان به ترتیب.
    pub fixes: Vec<PortfolioSafetyFixView>,
}

/// پاسخ `GET /api/v1/studio/ux-profiles` — مورفِ شناختی + مدالیتی‌ها + ایمنی.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StudioUxProfilesResponse {
    /// سه لایهٔ شناختی به ترتیبِ نردبان.
    pub tiers: Vec<CognitiveTierView>,
    /// ۹ مدالیتیِ صفر-هزینه.
    pub modalities: Vec<StudioModalityView>,
    /// گزارشِ ۶ درمانِ قضایی.
    pub portfolio_safety: PortfolioSafetyReportView,
    /// امضای باندل (`studio-ux-profiles-v1`).
    pub signature: String,
}

/// درخواست `POST /api/v1/studio/generate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StudioGenerateRequest {
    /// پرامپتِ سازنده — از یک جملهٔ کودک تا مشخصهٔ مهندسی.
    pub prompt: String,
    /// مدالیتیِ هدف (`search` | `deep` | `agent` | `workspace` | `voice` | `image` | `video` | `copy` | `sfx`).
    pub modality: String,
    /// لایهٔ صریح — اختیاری؛ پیش‌فرض: استنتاجِ تلمتری.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognitive_tier: Option<String>,
    /// زبانِ مبدأ — اختیاری؛ پیش‌فرض: تشخیصِ قطعیِ ARDA.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_locale: Option<String>,
    /// زبان‌های هدف — اختیاری؛ پیش‌فرض: هر ۲۰ زبانِ سیاره‌ای.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_locales: Option<Vec<String>>,
}

/// کارتِ پروژهٔ ویترین روی وایر — ۱۲ کلیدِ کانونی + پسوندهای تجاری.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorefrontProjectCardView {
    /// `slug`.
    pub slug: String,
    /// `name`.
    pub name: String,
    /// `tagline`.
    pub tagline: String,
    /// `category`.
    pub category: String,
    /// `stack`.
    pub stack: Vec<String>,
    /// `highlights`.
    pub highlights: Vec<String>,
    /// `Preview` — HTML زندهٔ پیش‌نمایش.
    pub preview: String,
    /// `sourceDir`.
    pub source_dir: String,
    /// `readme`.
    pub readme: String,
    /// `loc`.
    pub loc: u32,
    /// `fileCount`.
    pub file_count: u32,
    /// `featured`.
    pub featured: bool,
    /// قیمت به میکرو-کیان.
    pub price_ukian: u64,
    /// قیمت به ستارهٔ تلگرام.
    pub price_xtr_stars: u32,
    /// قیمت به میکرو-`USDC`.
    pub price_usdc_micros: u64,
    /// نمرهٔ REA مایرون.
    pub rea_score: u8,
    /// نشانیِ `SKILL.md`.
    pub skill_md_url: String,
    /// شمارِ زبان‌های سیاره‌ای.
    pub supported_locales_count: u8,
    /// نشانِ لایهٔ شناختی.
    pub cognitive_tier_badge: String,
}

/// باندلِ دوگانهٔ رویه‌ای روی وایر.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DualBundleView {
    /// موتور A — `Fishkal` اقیانوس + بلوم.
    pub fishkal_js: String,
    /// موتور B — `Cosmos` ستاره‌باران + سینتِ دیلی-بازخورد.
    pub cosmos_js: String,
    /// چرخهٔ عمرِ مشترکِ توقفِ RAF (Fix #6).
    pub pause_lifecycle_js: String,
    /// امضای باندل.
    pub signature: String,
    /// مُهر `sha256`.
    pub content_sha256: String,
}

/// پاسخ `POST /api/v1/studio/generate` — آرتیفکتِ کاملِ قابل بازی.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StudioGenerateResponse {
    /// شناسهٔ آفرینش (`studio:<uuid>`) — ماندگار در `studio_creations`.
    pub creation_id: String,
    /// لایهٔ شناختیِ مورف‌شده.
    pub cognitive_tier: String,
    /// مدالیتیِ مولد.
    pub modality: String,
    /// عنوانِ قطعی.
    pub title: String,
    /// آرتیفکتِ کامل با باندلِ تزریقیِ ARDA.
    pub artifact_html: String,
    /// باندلِ دوگانهٔ `Fishkal+Cosmos`.
    pub dual_bundle: DualBundleView,
    /// کارتِ ویترینِ آمادهٔ فروش.
    pub storefront_card: StorefrontProjectCardView,
    /// نوارِ پرچمِ دایره‌ایِ ARDA برای کارت بازار.
    pub circular_flag_strip_html: String,
    /// زبانِ مبدأ.
    pub origin_locale: String,
    /// زبان‌های فعالِ باندل.
    pub supported_locales: Vec<String>,
    /// `SKILL.md` فورج‌شدهٔ مایرون (همیشه حاضر — REA بالای گیت).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_md: Option<String>,
    /// `skill.json` فورج‌شده.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_json: Option<String>,
    /// نمرهٔ REA قطعی (`88..=97`).
    pub rea_score: u8,
    /// قیمت به میکرو-کیان.
    pub price_ukian: u64,
    /// قیمت به ستارهٔ تلگرام.
    pub price_xtr_stars: u32,
    /// مُهرِ زمانِ تولید (ISO-8601).
    pub created_at: String,
}

/// پالتِ فرمان روی وایر — کیبورد-اول.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandPaletteView {
    /// میان‌برهای بازکردن (`Ctrl+K` / `Meta+K` / `/`).
    pub open_shortcuts: Vec<String>,
    /// میان‌برِ تمرکز.
    pub focus_shortcut: String,
    /// کیبورد-اول؟
    pub keyboard_first: bool,
    /// کلیدهای ناوبری.
    pub navigation_keys: Vec<String>,
    /// JS آمادهٔ تزریقِ پالت.
    pub js: String,
}

/// پاسخ `GET /api/v1/studio/portfolio-catalog` — ویترینِ غنی‌شده.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioCatalogResponse {
    /// کاتالوگِ غنی‌شده — دو `featured` برتر در صدر.
    pub projects: Vec<StorefrontProjectCardView>,
    /// `slug`های دو ویترینیِ برتر.
    pub featured_top: Vec<String>,
    /// توکن‌های گلسِ تاریک (CSS خام).
    pub css_tokens: String,
    /// پیکربندیِ پالتِ فرمان.
    pub command_palette: CommandPaletteView,
    /// ریشهٔ قابل‌حملِ کاتالوگ (Fix #1) — بدونِ گریزِ والد.
    pub portfolio_root: String,
    /// گزارشِ ۶ درمانِ قضایی.
    pub portfolio_safety: PortfolioSafetyReportView,
    /// امضای باندل (`portfolio-storefront-v1`).
    pub signature: String,
}
