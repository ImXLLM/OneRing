//! Contracts for the mobile-perception studio — the zero-cost 9-modality superstudio and
//! portfolio showcase (Wave 5 — Step 2/4).
//!
//! Four DTOs on the wire:
//! - [`StudioUxProfilesResponse`] — three cognitive tiers + 9 modalities + the report of
//!   the portfolio's 6 forensic remedies;
//! - [`StudioGenerateRequest`] / [`StudioGenerateResponse`] — live artifact generation with the
//!   dual `Fishkal+Cosmos` bundle, ARDA packaging, and the
//!   Mairon forge;
//! - [`PortfolioCatalogResponse`] — the enriched showcase with the two top
//!   showcase upgrades, dark glass tokens, and the keyboard-first command palette.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A cognitive tier on the wire — the UX morph profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CognitiveTierView {
    /// Tier id (`ToySpark` | `CreatorStudio` | `ProTerminal`).
    pub tier: String,
    /// Native label.
    pub fa_label: String,
    /// Age/mental band.
    pub age_band: String,
    /// Cap on actions per screen.
    pub max_actions_per_screen: u32,
    /// Minimum touch target (pixels).
    pub min_tap_target_px: u32,
    /// Word cap per label.
    pub max_words_per_label: u32,
    /// Voice wizard?
    pub voice_guided: bool,
    /// Zero-loss sandbox?
    pub zero_loss_sandbox: bool,
    /// Keyboard-first?
    pub keyboard_first: bool,
    /// Layout summary.
    pub layout_summary: String,
}

/// A modality on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StudioModalityView {
    /// Modality id (`search` | `deep` | … | `sfx`).
    pub modality: String,
    /// Native label.
    pub fa_label: String,
    /// Required auth — always `none`.
    pub auth: String,
    /// Execution engine.
    pub engine: String,
    /// Always `true` — the zero-cost gate.
    pub zero_cost: bool,
}

/// A forensic remedy on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSafetyFixView {
    /// Remedy id (`1..=6`).
    pub fix_id: u32,
    /// Remedy title.
    pub title: String,
    /// Status — always `fixed`.
    pub status: String,
    /// Deterministic evidence.
    pub evidence: String,
}

/// The `KiaAgentX/portfolio` 6-forensic-remedy report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSafetyReportView {
    /// Count of applied remedies — always `6`.
    pub fixed_count: u32,
    /// Always `true`.
    pub all_fixed: bool,
    /// Each remedy's row, in order.
    pub fixes: Vec<PortfolioSafetyFixView>,
}

/// Response of `GET /api/v1/studio/ux-profiles` — cognitive morph + modalities + safety.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StudioUxProfilesResponse {
    /// The three cognitive tiers in ladder order.
    pub tiers: Vec<CognitiveTierView>,
    /// The 9 zero-cost modalities.
    pub modalities: Vec<StudioModalityView>,
    /// The 6-forensic-remedy report.
    pub portfolio_safety: PortfolioSafetyReportView,
    /// Bundle signature (`studio-ux-profiles-v1`).
    pub signature: String,
}

/// Request of `POST /api/v1/studio/generate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StudioGenerateRequest {
    /// Builder prompt — from a child's sentence to an engineering spec.
    pub prompt: String,
    /// Target modality (`search` | `deep` | `agent` | `workspace` | `voice` | `image` | `video` | `copy` | `sfx`).
    pub modality: String,
    /// Explicit tier — optional; default: telemetry inference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognitive_tier: Option<String>,
    /// Source language — optional; default: ARDA's deterministic detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_locale: Option<String>,
    /// Target languages — optional; default: all 20 planetary languages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_locales: Option<Vec<String>>,
}

/// Showcase project card on the wire — 12 canonical keys + commercial extensions.
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
    /// `Preview` — live preview HTML.
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
    /// Price in micro-Kian.
    pub price_ukian: u64,
    /// Price in Telegram Stars.
    pub price_xtr_stars: u32,
    /// Price in micro-`USDC`.
    pub price_usdc_micros: u64,
    /// Mairon's REA score.
    pub rea_score: u8,
    /// Address of the `SKILL.md`.
    pub skill_md_url: String,
    /// Count of planetary languages.
    pub supported_locales_count: u8,
    /// Cognitive-tier badge.
    pub cognitive_tier_badge: String,
}

/// The procedural dual bundle on the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DualBundleView {
    /// Engine A — the `Fishkal` ocean + bloom.
    pub fishkal_js: String,
    /// Engine B — the `Cosmos` starfall + a daily-feedback synth.
    pub cosmos_js: String,
    /// Shared lifecycle for RAF stop (Fix #6).
    pub pause_lifecycle_js: String,
    /// Bundle signature.
    pub signature: String,
    /// `sha256` stamp.
    pub content_sha256: String,
}

/// Response of `POST /api/v1/studio/generate` — the complete playable artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StudioGenerateResponse {
    /// Creation id (`studio:<uuid>`) — persisted in `studio_creations`.
    pub creation_id: String,
    /// The morphed cognitive tier.
    pub cognitive_tier: String,
    /// The generating modality.
    pub modality: String,
    /// The deterministic title.
    pub title: String,
    /// The complete artifact with the ARDA-injected bundle.
    pub artifact_html: String,
    /// The dual `Fishkal+Cosmos` bundle.
    pub dual_bundle: DualBundleView,
    /// The storefront-ready showcase card.
    pub storefront_card: StorefrontProjectCardView,
    /// ARDA's circular flag ribbon for the market card.
    pub circular_flag_strip_html: String,
    /// Source language.
    pub origin_locale: String,
    /// The bundle's active languages.
    pub supported_locales: Vec<String>,
    /// The Mairon-forged `SKILL.md` (always present — REA above the gate).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_md: Option<String>,
    /// The forged `skill.json`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_json: Option<String>,
    /// Deterministic REA score (`88..=97`).
    pub rea_score: u8,
    /// Price in micro-Kian.
    pub price_ukian: u64,
    /// Price in Telegram Stars.
    pub price_xtr_stars: u32,
    /// Generation timestamp (ISO-8601).
    pub created_at: String,
}

/// Command palette on the wire — keyboard-first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandPaletteView {
    /// Open shortcuts (`Ctrl+K` / `Meta+K` / `/`).
    pub open_shortcuts: Vec<String>,
    /// Focus shortcut.
    pub focus_shortcut: String,
    /// Keyboard-first?
    pub keyboard_first: bool,
    /// Navigation keys.
    pub navigation_keys: Vec<String>,
    /// Ready-to-inject palette JS.
    pub js: String,
}

/// Response of `GET /api/v1/studio/portfolio-catalog` — the enriched showcase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioCatalogResponse {
    /// The enriched catalog — the two top `featured` entries up front.
    pub projects: Vec<StorefrontProjectCardView>,
    /// The slugs of the two top showcase items.
    pub featured_top: Vec<String>,
    /// Dark glass tokens (raw CSS).
    pub css_tokens: String,
    /// Command palette configuration.
    pub command_palette: CommandPaletteView,
    /// The catalog's portable root (Fix #1) — no parent escape.
    pub portfolio_root: String,
    /// The 6-forensic-remedy report.
    pub portfolio_safety: PortfolioSafetyReportView,
    /// Bundle signature (`portfolio-storefront-v1`).
    pub signature: String,
}
