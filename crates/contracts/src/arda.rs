//! قراردادهای ARDA — بومی‌سازیِ سیاره‌ایِ ۲۴ساعته روی معماری اصیل `Fishkal`
//! و فورجِ مهارتِ مایرون (Wave 4 — Step 1/4).
//!
//! سه DTO روی وایر:
//! - [`ArdaLocalizeRequest`] / [`ArdaLocalizeResponse`] — اجرای زندهٔ ARDA روی
//!   HTML خام یا شناسهٔ محصول، با باندلِ تزریقی، نوارِ پرچمِ دایره‌ای،
//!   گزارشِ توازنِ کاتالوگ و مهرِ `sha256` محتوا؛
//! - [`ArdaFlagsCatalogResponse`] — کاتالوگِ کاملِ ۲۰ زبانِ سیاره‌ای با SVG
//!   وکتورِ حفظ‌شدهٔ حرف‌به‌حرف و نشانِ دایره‌ای؛
//! - [`ProductSkillPackageResponse`] — بستهٔ `SKILL.md` + `skill.json` فورج‌شدهٔ
//!   مایرون همراهِ نوارِ پرچمِ محصول.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// گزارشِ توازنِ یک زبان در کاتالوگِ یکپارچهٔ ۶-دامنه‌ای.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaLocaleParityView {
    /// کد زبان (`fa`، `en`، …).
    pub locale: String,
    /// کلیدهای غایب نسبت به زبانِ پایه (باید همیشه خالی باشد).
    pub missing_keys: Vec<String>,
    /// نمرهٔ پوششِ دقیق `0.0..=1.0`.
    pub coverage_score: f64,
}

/// گزارشِ کاملِ توازنِ مکانیکیِ کاتالوگ — درمانِ نقصِ `unitFt` و `OPTSTR`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaParityReportView {
    /// زبانِ پایهٔ توازن.
    pub base_locale: String,
    /// تعدادِ کلیدهای مرجع.
    pub total_keys: u32,
    /// بدترینِ پوشش بین ۲۰ زبان.
    pub coverage_score: f64,
    /// توازنِ کامل؟
    pub fully_parity: bool,
    /// توازنِ هر زبان به ترتیبِ دیکشنری.
    pub locales: Vec<ArdaLocaleParityView>,
}

/// یک زبان در کاتالوگِ پرچمِ وکتورِ فیشکال.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaFlagView {
    /// کد ISO کوتاه (`fa`، `en`، …).
    pub code: String,
    /// نام بومیِ کنونیکال (`'فارسی'`، `'English'`).
    pub native_name: String,
    /// گلیفِ سوییچرِ اسکریپت‌اصیل (`STR.code`: `'فا'`، `'ع'`، `'اردو'`، `'עب'`).
    pub str_code: String,
    /// جهت متن (`rtl` | `ltr`).
    pub dir: String,
    /// زیرتگ منطقه‌ایِ `Intl` (`fa-IR`، `ar-AE`، …).
    pub intl_locale: String,
    /// SVG وکتورِ حفظ‌شدهٔ حرف‌به‌حرف با `viewBox="0 0 20 14"`.
    pub svg: String,
    /// نشانِ دایره‌ایِ مینی-پرچمِ آمادهٔ رندر.
    pub circular_badge_html: String,
}

/// پاسخ `GET /api/v1/arda/flags` — کاتالوگِ کاملِ ۲۰ زبانِ سیاره‌ای.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaFlagsCatalogResponse {
    /// ۲۰ زبان به ترتیبِ دیکشنریِ فیشکال.
    pub locales: Vec<ArdaFlagView>,
    /// تعدادِ کل — همیشه `20`.
    pub total: u32,
    /// امضای باندل (`arda-fishkal-20`).
    pub signature: String,
}

/// درخواست `POST /api/v1/arda/localize-now` — HTML خام یا شناسهٔ محصول.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaLocalizeRequest {
    /// شناسهٔ محصول (`prod:<uuid>`) — اگر داده شود، زبان‌ها روی محصول مُهر می‌شوند.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    /// HTML خامِ ورودی (حداکثر ۵۰ کیلوبایت — همان سقفِ پیش‌نمایش بازار).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// زبانِ مبدأ — پیش‌فرض: تشخیصِ قطعی از اسکریپتِ محتوا.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_locale: Option<String>,
    /// زبان‌های هدف — پیش‌فرض: هر ۲۰ زبانِ سیاره‌ای.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_locales: Option<Vec<String>>,
}

/// پاسخ localize — باندلِ تزریقی + نوارِ پرچم + توازن + مهرِ محتوا.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaLocalizeResponse {
    /// شناسهٔ محصول — فقط وقتی `product_id` داده شده بود.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    /// زبانِ مبدأیِ تشخیص‌شده/داده‌شده.
    pub origin_locale: String,
    /// زبان‌های فعالِ باندل (مبدأ + هدف‌های معتبر).
    pub supported_locales: Vec<String>,
    /// HTML نهایی با باندلِ تزریقی (ایدمپوتنت).
    pub localized_html: String,
    /// نوارِ نشانِ دایره‌ایِ مبدأ + ترجمه‌ها برای کارتِ بازار.
    pub circular_flag_strip_html: String,
    /// گزارشِ توازنِ مکانیکیِ کاتالوگِ ۶-دامنه‌ای.
    pub parity_report: ArdaParityReportView,
    /// آیا باندل تزریق شد؟ (`false` = قبلاً تزریق شده بود)
    pub injected: bool,
    /// مهرِ `sha256` محتوای بومی‌سازی‌شده.
    pub content_sha256: String,
    /// مهرِ زمانِ بومی‌سازی (ISO-8601) — فقط برای محصول.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_localized_at: Option<String>,
}

/// پاسخ `GET /api/v1/market/products/{id}/skill` — بستهٔ مهارتِ فورج‌شده.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductSkillPackageResponse {
    /// شناسهٔ محصول.
    pub product_id: String,
    /// نمرهٔ REA مایرون (`0..=100`).
    pub rea_score: u8,
    /// نشان مرئی مایرون مُهرشده.
    pub mairon_visible_badge: String,
    /// زبانِ مبدأ.
    pub origin_locale: String,
    /// زبان‌های پشتیبانی‌شده (مبدأ + ترجمه‌ها).
    pub supported_locales: Vec<String>,
    /// نوارِ پرچمِ دایره‌ایِ محصول.
    pub circular_flag_strip_html: String,
    /// سند `SKILL.md` با پیش‌مادهٔ YAML.
    pub skill_md: String,
    /// مانیفست `skill.json` سازگار با Hermes / MCP / `OpenCode`.
    pub skill_json: String,
    /// آیا بسته در همین فراخوانی به‌صورت خودکار فورج شد؟
    pub forged_on_the_fly: bool,
}
