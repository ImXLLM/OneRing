//! ARDA contracts — round-the-clock planetary localization on the original `Fishkal` architecture
//! and Mairon skill forging (Wave 4 — Step 1/4).
//!
//! Three wire DTOs:
//! - [`ArdaLocalizeRequest`] / [`ArdaLocalizeResponse`] — live ARDA execution on
//!   raw HTML or a product ID, with the injected bundle, circular flag strip,
//!   catalog balance report and a `sha256` content stamp;
//! - [`ArdaFlagsCatalogResponse`] — the complete catalog of 20 planetary languages with
//!   verbatim-preserved vector SVG and circular badges;
//! - [`ProductSkillPackageResponse`] — Mairon's forged `SKILL.md` + `skill.json` package
//!   together with the product flag strip.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Balance report for one language in the unified 6-domain catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaLocaleParityView {
    /// Language code (`fa`, `en`, …).
    pub locale: String,
    /// Keys missing relative to the base language (must always be empty).
    pub missing_keys: Vec<String>,
    /// Exact coverage score `0.0..=1.0`.
    pub coverage_score: f64,
}

/// Full mechanical catalog balance report — the cure for the `unitFt` and `OPTSTR` gaps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaParityReportView {
    /// Balance base language.
    pub base_locale: String,
    /// Number of reference keys.
    pub total_keys: u32,
    /// Worst coverage among the 20 languages.
    pub coverage_score: f64,
    /// Fully balanced?
    pub fully_parity: bool,
    /// Balance of each language in lexicographic order.
    pub locales: Vec<ArdaLocaleParityView>,
}

/// One language in the Fishkal vector flag catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaFlagView {
    /// Short ISO code (`fa`, `en`, …).
    pub code: String,
    /// Canonical native name (`'\u{0641}\u{0627}\u{0631}\u{0633}\u{06CC}'`, `'English'`).
    pub native_name: String,
    /// Script-switcher glyph (`STR.code`: `'\u{0641}\u{0627}'`, `'\u{0639}'`, `'\u{0627}\u{0631}\u{062F}\u{0648}'`, `'\u{05E2}\u{0628}'`).
    pub str_code: String,
    /// Text direction (`rtl` | `ltr`).
    pub dir: String,
    /// `Intl` region subtag (`fa-IR`, `ar-AE`, …).
    pub intl_locale: String,
    /// Verbatim-preserved vector SVG with `viewBox="0 0 20 14"`.
    pub svg: String,
    /// Circular badge of a render-ready mini-flag.
    pub circular_badge_html: String,
}

/// `GET /api/v1/arda/flags` response — the complete catalog of 20 planetary languages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaFlagsCatalogResponse {
    /// 20 languages in Fishkal lexicographic order.
    pub locales: Vec<ArdaFlagView>,
    /// Total count — always `20`.
    pub total: u32,
    /// Bundle signature (`arda-fishkal-20`).
    pub signature: String,
}

/// `POST /api/v1/arda/localize-now` request — raw HTML or a product ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaLocalizeRequest {
    /// Product ID (`prod:<uuid>`) — when provided, the languages are stamped onto the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    /// Raw input HTML (max 50 KB — the same cap as the market preview).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Source language — default: deterministic detection from the content script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_locale: Option<String>,
    /// Target languages — default: all 20 planetary languages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_locales: Option<Vec<String>>,
}

/// Localize response — injected bundle + flag strip + balance + content stamp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArdaLocalizeResponse {
    /// Product ID — only when `product_id` was provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    /// Detected/provided source language.
    pub origin_locale: String,
    /// The bundle's active languages (source + valid targets).
    pub supported_locales: Vec<String>,
    /// Final HTML with the injected bundle (idempotent).
    pub localized_html: String,
    /// Circular badge strip of source + translations for the market card.
    pub circular_flag_strip_html: String,
    /// Mechanical balance report of the 6-domain catalog.
    pub parity_report: ArdaParityReportView,
    /// Was the bundle injected? (`false` = it was already injected)
    pub injected: bool,
    /// `sha256` stamp of the localized content.
    pub content_sha256: String,
    /// Localization timestamp (ISO-8601) — for products only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_localized_at: Option<String>,
}

/// `GET /api/v1/market/products/{id}/skill` response — the forged skill package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductSkillPackageResponse {
    /// Product ID.
    pub product_id: String,
    /// Mairon REA score (`0..=100`).
    pub rea_score: u8,
    /// The stamped Mairon visual badge.
    pub mairon_visible_badge: String,
    /// Source language.
    pub origin_locale: String,
    /// Supported languages (source + translations).
    pub supported_locales: Vec<String>,
    /// Product circular flag strip.
    pub circular_flag_strip_html: String,
    /// `SKILL.md` document with YAML front matter.
    pub skill_md: String,
    /// `skill.json` manifest compatible with Hermes / MCP / `OpenCode`.
    pub skill_json: String,
    /// Was the package forged automatically in this same call?
    pub forged_on_the_fly: bool,
}
