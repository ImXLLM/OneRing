//! Wave 9 wire contracts (Commit #29) — the neural HUD studio, the
//! 10-DNA lab, the Grok UX engine, and the day-one i18n matrix.
//!
//! On-wire DTOs for the eight new endpoints:
//! - `GET  /api/v1/hud/presets` / `POST /api/v1/hud/synthesize` — the catalog of
//!   8 presets + dynamic synthesis ([`HudPresetsResponse`], [`HudSynthesizeResponse`]);
//! - `GET  /api/v1/grok/ui-bundle` / `POST /api/v1/grok/chat-turn` — the
//!   "cosmic dark matter" UI + the signature chat turn ([`GrokUiBundleResponse`],
//!   [`GrokChatTurnResponse`]);
//! - `GET  /api/v1/i18n/ecosystem-matrix` / `POST /api/v1/i18n/apply-patch` —
//!   the 20-locale matrix + hot patch ([`I18nMatrixResponse`],
//!   [`I18nApplyPatchResponse`]);
//! - `GET  /api/v1/admin/lab-telemetry` / `POST /api/v1/admin/export-training-dataset`
//!   — the admin log observatory + RLHF/SFT export ([`AdminLabTelemetryResponse`],
//!   [`AdminExportTrainingDatasetResponse`]).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── HUD — catalog and synthesis ────────────────────────────────────────────────────

/// A HUD preset view on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HudPresetView {
    /// Unique ID — `mwie`, `kia-monopoly-core`, …
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Subtitle tagline.
    pub tagline: String,
    /// Accent color in hex.
    pub accent_hex: String,
    /// HUD corner badge.
    pub badge: String,
    /// Origin — `upstream` | `kia_sovereign` | `synthesized`.
    pub source: String,
    /// Node labels.
    pub node_labels: Vec<String>,
}

/// `GET /api/v1/hud/presets` response — the 8-preset catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HudPresetsResponse {
    /// Presets in fixed order.
    pub presets: Vec<HudPresetView>,
    /// HUD studio signature.
    pub signature: String,
}

/// `POST /api/v1/hud/synthesize` request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HudSynthesizeRequest {
    /// Repository name — the deterministic basis of preset synthesis.
    pub repo_name: String,
    /// Optional tagline.
    #[serde(default)]
    pub tagline: Option<String>,
    /// Optional accent color — `#RRGGBB` or `#RGB`.
    #[serde(default)]
    pub accent_hex: Option<String>,
    /// Aspect ratio — `9:16` | `16:9`; default `9:16`.
    #[serde(default)]
    pub aspect: Option<String>,
    /// Resolution — `720p` | `1080p`; default `1080p`.
    #[serde(default)]
    pub resolution: Option<String>,
    /// Frame rate — `24` | `60`; default `24`.
    #[serde(default)]
    pub fps: Option<String>,
    /// Duration in seconds — 1..=180; default 60.
    #[serde(default)]
    pub duration_sec: Option<u32>,
}

/// `POST /api/v1/hud/synthesize` response — the HTML package + `FFmpeg` commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HudSynthesizeResponse {
    /// The synthesized preset.
    pub preset: HudPresetView,
    /// The deterministic HTML5 package (`window.seekTo` + `WebAudio`).
    pub html: String,
    /// SHA-256 of the HTML package.
    pub html_sha256: String,
    /// Frame width.
    pub frame_width: u32,
    /// Frame height.
    pub frame_height: u32,
    /// Frame rate.
    pub fps: u32,
    /// Duration in seconds.
    pub duration_sec: u32,
    /// Node count — 180.
    pub node_count: usize,
    /// Edge count.
    pub edge_count: usize,
    /// `FFmpeg` audio command — deterministic argv with the master limiter.
    pub audio_command_argv: Vec<String>,
    /// `FFmpeg` video command — deterministic argv with `+faststart`.
    pub video_command_argv: Vec<String>,
    /// Whether the render run persisted in `NeuralHudRenderJob`.
    pub persisted: bool,
    /// Signature.
    pub signature: String,
}

// ── Grok — UI bundle and chat turn ──────────────────────────────────────────────

/// `GET /api/v1/grok/ui-bundle` parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokUiBundleQuery {
    /// Locale — `fa` | `en` | a patch slot; default `fa`.
    #[serde(default)]
    pub locale: Option<String>,
    /// UX layer — `toyspark` | `simple` | `creator_studio` | `pro_terminal`.
    #[serde(default)]
    pub ux_tier: Option<String>,
    /// Persona — `fun_witty` | `unhinged_roast` | `regular_balanced` |
    /// `mairon_sovereign`.
    #[serde(default)]
    pub persona: Option<String>,
}

/// `GET /api/v1/grok/ui-bundle` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokUiBundleResponse {
    /// Self-contained HTML document.
    pub html: String,
    /// Active locale.
    pub locale: String,
    /// Direction — `rtl` | `ltr`.
    pub direction: String,
    /// UX layer.
    pub ux_tier: String,
    /// Persona.
    pub persona: String,
    /// Quick-action chips — 6.
    pub chips: Vec<String>,
    /// Three next-vibe suggestions.
    pub ghost_follow_ups: Vec<String>,
    /// The "cosmic dark matter" CSS signature.
    pub css_signature: String,
    /// Locale slot count — 20.
    pub locale_slots: usize,
}

/// `POST /api/v1/grok/chat-turn` request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokChatTurnRequest {
    /// User text.
    pub prompt: String,
    /// Locale; default `fa`.
    #[serde(default)]
    pub locale: Option<String>,
    /// UX layer; default `pro_terminal`.
    #[serde(default)]
    pub ux_tier: Option<String>,
    /// Persona; default `regular_balanced`.
    #[serde(default)]
    pub persona: Option<String>,
    /// `DeepSearch` bar enabled? Default off.
    #[serde(default)]
    pub deep_search: Option<bool>,
    /// Thinking accordion enabled? Default on.
    #[serde(default)]
    pub think: Option<bool>,
}

/// One visible reasoning step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokThinkStepView {
    /// Step text.
    pub step: String,
}

/// One verified source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokCitationView {
    /// Source title.
    pub title: String,
    /// Domain.
    pub domain: String,
    /// Verified?
    pub verified: bool,
}

/// `POST /api/v1/grok/chat-turn` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokChatTurnResponse {
    /// Thinking-accordion steps.
    pub thinking_steps: Vec<GrokThinkStepView>,
    /// Thinking duration in milliseconds.
    pub thinking_duration_ms: u32,
    /// `DeepSearch` sources.
    pub citations: Vec<GrokCitationView>,
    /// Reply text.
    pub response_text: String,
    /// Three next-vibe chips.
    pub ghost_follow_ups: Vec<String>,
    /// Persona.
    pub persona: String,
    /// Routed model.
    pub model_slug: String,
    /// Locale.
    pub locale: String,
    /// UX layer.
    pub ux_tier: String,
}

// ── i18n — matrix and patch ──────────────────────────────────────────────────────

/// A locale view on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nLocaleView {
    /// Locale code.
    pub code: String,
    /// English name.
    pub name: String,
    /// Native name.
    pub native_name: String,
    /// Direction — `rtl` | `ltr`.
    pub direction: String,
    /// Authentic font.
    pub font_family: String,
    /// SVG flag (`viewBox="0 0 20 14"`).
    pub flag_svg: String,
    /// Native sample character.
    pub glyph_sample: String,
    /// Status — `ACTIVE_DAY1` | `PATCH_READY_SLOT`.
    pub state: String,
    /// Completeness in bps.
    pub completion_bps: u32,
    /// Fallback locale.
    pub fallback: String,
}

/// `GET /api/v1/i18n/ecosystem-matrix` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nMatrixResponse {
    /// 20 locales in fixed order.
    pub locales: Vec<I18nLocaleView>,
    /// Day-one active count — 2 (`fa`, `en`).
    pub active_day1: usize,
    /// Patch-ready slot count — 18.
    pub patch_ready: usize,
    /// Signature.
    pub signature: String,
}

/// One translation-patch entry on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nPatchEntryDto {
    /// Panel module — one of the 9 modules.
    pub module: String,
    /// Key.
    pub key: String,
    /// Localized value.
    pub value: String,
}

/// `POST /api/v1/i18n/apply-patch` request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nApplyPatchRequest {
    /// Target locale code.
    pub locale_code: String,
    /// Patch entries.
    pub patch_entries: Vec<I18nPatchEntryDto>,
}

/// `POST /api/v1/i18n/apply-patch` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nApplyPatchResponse {
    /// Locale code.
    pub locale_code: String,
    /// Post-patch status.
    pub activation_state: String,
    /// New completeness in bps.
    pub completion_bps: u32,
    /// Number of applied entries.
    pub applied_count: usize,
    /// Patch version.
    pub patch_version: String,
    /// Applied entries (`module.key` → value).
    pub applied_entries: BTreeMap<String, String>,
    /// Whether it persisted in `LocalePatchRegistry`.
    pub persisted: bool,
}

// ── Admin — lab observatory and RLHF export ───────────────────────────────

/// A lab telemetry entry view on the wire — 1:1 with `UserLabTelemetryLog`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LabTelemetryEntryView {
    /// Deterministic ID — `lab:{dna}:{hour}:{turn}`.
    pub id: String,
    /// Cohort code.
    pub dna_code: String,
    /// User handle.
    pub user_handle: String,
    /// Hour 0..24.
    pub hour_offset: u32,
    /// UX layer.
    pub ux_tier: String,
    /// Locale.
    pub locale: String,
    /// Persona.
    pub grok_persona: String,
    /// Routed model.
    pub model_slug: String,
    /// Routing mode.
    pub routing_mode: String,
    /// User prompt.
    pub prompt_text: String,
    /// Thinking trace as JSON.
    pub thinking_trace_json: String,
    /// Reply.
    pub response_text: String,
    /// REA score.
    pub rea_score: u8,
    /// Latency in ms.
    pub latency_ms: u32,
    /// Trainable?
    pub usable_for_training: bool,
    /// ISO timestamp.
    pub created_at: String,
}

/// A cohort's health on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CohortHealthView {
    /// Cohort code.
    pub dna_code: String,
    /// User handle.
    pub user_handle: String,
    /// Turns.
    pub turns: usize,
    /// Mean REA.
    pub avg_rea_score: u8,
    /// Untrainable.
    pub unusable: usize,
    /// Friction.
    pub friction_events: usize,
    /// Recovered friction.
    pub friction_recovered: usize,
    /// Dominant model.
    pub dominant_model: String,
    /// Locales seen.
    pub locales: Vec<String>,
}

/// The admin dashboard on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminLabDashboardView {
    /// Total entries.
    pub total_entries: usize,
    /// Cohort healths.
    pub cohort_health: Vec<CohortHealthView>,
    /// Model distribution.
    pub by_model: BTreeMap<String, usize>,
    /// Locale distribution.
    pub by_locale: BTreeMap<String, usize>,
    /// UX-layer distribution.
    pub by_ux_tier: BTreeMap<String, usize>,
    /// Persona distribution.
    pub by_persona: BTreeMap<String, usize>,
    /// Mean REA.
    pub avg_rea_score: u8,
    /// Min REA.
    pub min_rea_score: u8,
    /// Max REA.
    pub max_rea_score: u8,
    /// Untrainable.
    pub unusable_entries: usize,
    /// Total friction.
    pub friction_events: usize,
    /// Recovered friction.
    pub friction_recovered: usize,
}

/// `GET /api/v1/admin/lab-telemetry` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminLabTelemetryResponse {
    /// Persisted telemetry entries (up to the limit cap).
    pub entries: Vec<LabTelemetryEntryView>,
    /// The aggregate dashboard.
    pub dashboard: AdminLabDashboardView,
    /// Signature.
    pub signature: String,
}

/// `POST /api/v1/admin/export-training-dataset` request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminExportTrainingDatasetRequest {
    /// REA threshold — only entries with `rea_score >=` this value.
    pub min_rea_score: u8,
}

/// `POST /api/v1/admin/export-training-dataset` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminExportTrainingDatasetResponse {
    /// SFT JSONL (ChatML/ShareGPT).
    pub sft_jsonl: String,
    /// DPO JSONL (`prompt/chosen/rejected`).
    pub dpo_jsonl: String,
    /// SFT records.
    pub sft_records: usize,
    /// DPO records.
    pub dpo_records: usize,
    /// The applied threshold.
    pub min_rea_score: u8,
    /// Signature.
    pub signature: String,
}
