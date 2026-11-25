//! قراردادهای وایرِ Wave 9 (Commit #29) — استودیوی HUD عصبی، آزمایشگاهِ
//! ۱۰-DNA، موتورِ UXِ Grok و ماتریسِ i18nِ روزِ اول.
//!
//! DTOهای روی وایر برای هشت اندپوینتِ تازه:
//! - `GET  /api/v1/hud/presets` / `POST /api/v1/hud/synthesize` — کاتالوگِ
//!   ۸ پیش‌تن + سنتزِ پویا ([`HudPresetsResponse`], [`HudSynthesizeResponse`])؛
//! - `GET  /api/v1/grok/ui-bundle` / `POST /api/v1/grok/chat-turn` — بستهٔ
//!   UIِ «مادهٔ تاریکِ کیهانی» + نوبتِ چتِ امضایی ([`GrokUiBundleResponse`],
//!   [`GrokChatTurnResponse`])؛
//! - `GET  /api/v1/i18n/ecosystem-matrix` / `POST /api/v1/i18n/apply-patch` —
//!   ماتریسِ ۲۰ لوکال + پچِ داغ ([`I18nMatrixResponse`],
//!   [`I18nApplyPatchResponse`])؛
//! - `GET  /api/v1/admin/lab-telemetry` / `POST /api/v1/admin/export-training-dataset`
//!   — رصدخانهٔ لاگِ ادمین + صادراتِ RLHF/SFT ([`AdminLabTelemetryResponse`],
//!   [`AdminExportTrainingDatasetResponse`]).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── HUD — کاتالوگ و سنتز ────────────────────────────────────────────────────

/// نمایِ یک پیش‌تنِ HUD روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HudPresetView {
    /// شناسهٔ یکتا — `mwie`، `kia-monopoly-core`، …
    pub slug: String,
    /// عنوانِ نمایشی.
    pub title: String,
    /// شعارِ زیرِ عنوان.
    pub tagline: String,
    /// رنگِ accent به هگز.
    pub accent_hex: String,
    /// نشانِ گوشهٔ HUD.
    pub badge: String,
    /// خاستگاه — `upstream` | `kia_sovereign` | `synthesized`.
    pub source: String,
    /// برچسب‌هایِ گره‌ها.
    pub node_labels: Vec<String>,
}

/// پاسخِ `GET /api/v1/hud/presets` — کاتالوگِ ۸ پیش‌تن.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HudPresetsResponse {
    /// پیش‌تن‌ها به ترتیبِ ثابت.
    pub presets: Vec<HudPresetView>,
    /// امضایِ استودیوی HUD.
    pub signature: String,
}

/// درخواستِ `POST /api/v1/hud/synthesize`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HudSynthesizeRequest {
    /// نامِ مخزن — مبنایِ قطعیِ سنتزِ پیش‌تن.
    pub repo_name: String,
    /// شعارِ اختیاری.
    #[serde(default)]
    pub tagline: Option<String>,
    /// رنگِ accent اختیاری — `#RRGGBB` یا `#RGB`.
    #[serde(default)]
    pub accent_hex: Option<String>,
    /// نسبت — `9:16` | `16:9`؛ پیش‌فرض `9:16`.
    #[serde(default)]
    pub aspect: Option<String>,
    /// رزولوشن — `720p` | `1080p`؛ پیش‌فرض `1080p`.
    #[serde(default)]
    pub resolution: Option<String>,
    /// فریم‌ریت — `24` | `60`؛ پیش‌فرض `24`.
    #[serde(default)]
    pub fps: Option<String>,
    /// مدت بر حسبِ ثانیه — ۱..=۱۸۰؛ پیش‌فرض ۶۰.
    #[serde(default)]
    pub duration_sec: Option<u32>,
}

/// پاسخِ `POST /api/v1/hud/synthesize` — بستهٔ HTML + فرمان‌هایِ `FFmpeg`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HudSynthesizeResponse {
    /// پیش‌تنِ سنتزشده.
    pub preset: HudPresetView,
    /// بستهٔ HTML5 قطعی (`window.seekTo` + `WebAudio`).
    pub html: String,
    /// SHA-256 بستهٔ HTML.
    pub html_sha256: String,
    /// عرضِ فریم.
    pub frame_width: u32,
    /// ارتفاعِ فریم.
    pub frame_height: u32,
    /// فریم‌ریت.
    pub fps: u32,
    /// مدت بر حسبِ ثانیه.
    pub duration_sec: u32,
    /// تعدادِ گره‌ها — ۱۸۰.
    pub node_count: usize,
    /// تعدادِ یال‌ها.
    pub edge_count: usize,
    /// فرمانِ `FFmpeg` صدا — argv قطعی با محدودگرِ مستر.
    pub audio_command_argv: Vec<String>,
    /// فرمانِ `FFmpeg` ویدئو — argv قطعی با `+faststart`.
    pub video_command_argv: Vec<String>,
    /// آیا رانِ رندر در `NeuralHudRenderJob` ماندگار شد.
    pub persisted: bool,
    /// امضا.
    pub signature: String,
}

// ── Grok — بستهٔ UI و نوبتِ چت ──────────────────────────────────────────────

/// پارامترهایِ `GET /api/v1/grok/ui-bundle`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokUiBundleQuery {
    /// لوکال — `fa` | `en` | اسلاتِ پچ؛ پیش‌فرض `fa`.
    #[serde(default)]
    pub locale: Option<String>,
    /// لایهٔ UX — `toyspark` | `simple` | `creator_studio` | `pro_terminal`.
    #[serde(default)]
    pub ux_tier: Option<String>,
    /// پرسونا — `fun_witty` | `unhinged_roast` | `regular_balanced` |
    /// `mairon_sovereign`.
    #[serde(default)]
    pub persona: Option<String>,
}

/// پاسخِ `GET /api/v1/grok/ui-bundle`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokUiBundleResponse {
    /// سندِ HTML مستقل.
    pub html: String,
    /// لوکالِ فعال.
    pub locale: String,
    /// جهت — `rtl` | `ltr`.
    pub direction: String,
    /// لایهٔ UX.
    pub ux_tier: String,
    /// پرسونا.
    pub persona: String,
    /// چیپ‌هایِ اکشنِ سریع — ۶.
    pub chips: Vec<String>,
    /// سه پیشنهادِ روحِ بعدی.
    pub ghost_follow_ups: Vec<String>,
    /// امضایِ CSSِ «مادهٔ تاریکِ کیهانی».
    pub css_signature: String,
    /// تعدادِ اسلات‌هایِ لوکال — ۲۰.
    pub locale_slots: usize,
}

/// درخواستِ `POST /api/v1/grok/chat-turn`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokChatTurnRequest {
    /// متنِ کاربر.
    pub prompt: String,
    /// لوکال؛ پیش‌فرض `fa`.
    #[serde(default)]
    pub locale: Option<String>,
    /// لایهٔ UX؛ پیش‌فرض `pro_terminal`.
    #[serde(default)]
    pub ux_tier: Option<String>,
    /// پرسونا؛ پیش‌فرض `regular_balanced`.
    #[serde(default)]
    pub persona: Option<String>,
    /// نوارِ `DeepSearch` فعال؟ پیش‌فرض خاموش.
    #[serde(default)]
    pub deep_search: Option<bool>,
    /// آکاردئونِ تفکر فعال؟ پیش‌فرض روشن.
    #[serde(default)]
    pub think: Option<bool>,
}

/// یک گامِ استدلالِ مرئی.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokThinkStepView {
    /// متنِ گام.
    pub step: String,
}

/// یک منبعِ راستی‌آزمایی‌شده.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokCitationView {
    /// عنوانِ منبع.
    pub title: String,
    /// دامنه.
    pub domain: String,
    /// راستی‌آزمایی‌شده؟
    pub verified: bool,
}

/// پاسخِ `POST /api/v1/grok/chat-turn`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrokChatTurnResponse {
    /// گام‌هایِ آکاردئونِ تفکر.
    pub thinking_steps: Vec<GrokThinkStepView>,
    /// مدتِ تفکر بر حسبِ میلی‌ثانیه.
    pub thinking_duration_ms: u32,
    /// منابعِ `DeepSearch`.
    pub citations: Vec<GrokCitationView>,
    /// متنِ پاسخ.
    pub response_text: String,
    /// سه چیپِ روحِ بعدی.
    pub ghost_follow_ups: Vec<String>,
    /// پرسونا.
    pub persona: String,
    /// مدلِ روت‌شده.
    pub model_slug: String,
    /// لوکال.
    pub locale: String,
    /// لایهٔ UX.
    pub ux_tier: String,
}

// ── i18n — ماتریس و پچ ──────────────────────────────────────────────────────

/// نمایِ یک لوکال روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nLocaleView {
    /// کدِ لوکال.
    pub code: String,
    /// نامِ انگلیسی.
    pub name: String,
    /// نامِ بومی.
    pub native_name: String,
    /// جهت — `rtl` | `ltr`.
    pub direction: String,
    /// فونتِ اصیل.
    pub font_family: String,
    /// پرچمِ SVG (`viewBox="0 0 20 14"`).
    pub flag_svg: String,
    /// نمونهٔ نویسهٔ بومی.
    pub glyph_sample: String,
    /// وضعیت — `ACTIVE_DAY1` | `PATCH_READY_SLOT`.
    pub state: String,
    /// تکمیل بر حسبِ bps.
    pub completion_bps: u32,
    /// لوکالِ fallback.
    pub fallback: String,
}

/// پاسخِ `GET /api/v1/i18n/ecosystem-matrix`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nMatrixResponse {
    /// ۲۰ لوکال به ترتیبِ ثابت.
    pub locales: Vec<I18nLocaleView>,
    /// تعدادِ فعال‌هایِ روزِ اول — ۲ (`fa`، `en`).
    pub active_day1: usize,
    /// تعدادِ اسلات‌هایِ آمادهٔ پچ — ۱۸.
    pub patch_ready: usize,
    /// امضا.
    pub signature: String,
}

/// یک مدخلِ پچِ ترجمه روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nPatchEntryDto {
    /// ماژولِ پنل — یکی از ۹ ماژول.
    pub module: String,
    /// کلید.
    pub key: String,
    /// مقدارِ بومی‌شده.
    pub value: String,
}

/// درخواستِ `POST /api/v1/i18n/apply-patch`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nApplyPatchRequest {
    /// کدِ لوکالِ هدف.
    pub locale_code: String,
    /// مدخل‌هایِ پچ.
    pub patch_entries: Vec<I18nPatchEntryDto>,
}

/// پاسخِ `POST /api/v1/i18n/apply-patch`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct I18nApplyPatchResponse {
    /// کدِ لوکال.
    pub locale_code: String,
    /// وضعیتِ پس از پچ.
    pub activation_state: String,
    /// تکمیلِ جدید بر حسبِ bps.
    pub completion_bps: u32,
    /// تعدادِ مدخل‌هایِ اعمال‌شده.
    pub applied_count: usize,
    /// نسخهٔ پچ.
    pub patch_version: String,
    /// مدخل‌هایِ اعمال‌شده (`module.key` → مقدار).
    pub applied_entries: BTreeMap<String, String>,
    /// آیا در `LocalePatchRegistry` ماندگار شد.
    pub persisted: bool,
}

// ── ادمین — رصدخانهٔ آزمایشگاه و صادراتِ RLHF ───────────────────────────────

/// نمایِ یک مدخلِ تلمتریِ آزمایشگاه روی وایر — ۱:۱ با `UserLabTelemetryLog`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LabTelemetryEntryView {
    /// شناسهٔ قطعی — `lab:{dna}:{hour}:{turn}`.
    pub id: String,
    /// کدِ کوهورت.
    pub dna_code: String,
    /// هندل کاربر.
    pub user_handle: String,
    /// ساعتِ ۰..۲۴.
    pub hour_offset: u32,
    /// لایهٔ UX.
    pub ux_tier: String,
    /// لوکال.
    pub locale: String,
    /// پرسونا.
    pub grok_persona: String,
    /// مدلِ روت‌شده.
    pub model_slug: String,
    /// حالتِ روتینگ.
    pub routing_mode: String,
    /// پرامپتِ کاربر.
    pub prompt_text: String,
    /// ردِ تفکر به JSON.
    pub thinking_trace_json: String,
    /// پاسخ.
    pub response_text: String,
    /// امتیازِ REA.
    pub rea_score: u8,
    /// تأخیر بر حسبِ ms.
    pub latency_ms: u32,
    /// قابلِ آموزش؟
    pub usable_for_training: bool,
    /// مُهرِ زمانِ ISO.
    pub created_at: String,
}

/// سلامتِ یک کوهورت روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CohortHealthView {
    /// کدِ کوهورت.
    pub dna_code: String,
    /// هندل کاربر.
    pub user_handle: String,
    /// نوبت‌ها.
    pub turns: usize,
    /// میانگینِ REA.
    pub avg_rea_score: u8,
    /// غیرقابلِ آموزش.
    pub unusable: usize,
    /// اصطکاک.
    pub friction_events: usize,
    /// اصطکاکِ بهبودیافته.
    pub friction_recovered: usize,
    /// مدلِ غالب.
    pub dominant_model: String,
    /// لوکال‌هایِ دیده‌شده.
    pub locales: Vec<String>,
}

/// داشبوردِ ادمین روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminLabDashboardView {
    /// کلِ مدخل‌ها.
    pub total_entries: usize,
    /// سلامتِ کوهورت‌ها.
    pub cohort_health: Vec<CohortHealthView>,
    /// توزیعِ مدل.
    pub by_model: BTreeMap<String, usize>,
    /// توزیعِ لوکال.
    pub by_locale: BTreeMap<String, usize>,
    /// توزیعِ لایهٔ UX.
    pub by_ux_tier: BTreeMap<String, usize>,
    /// توزیعِ پرسونا.
    pub by_persona: BTreeMap<String, usize>,
    /// میانگینِ REA.
    pub avg_rea_score: u8,
    /// کمینهٔ REA.
    pub min_rea_score: u8,
    /// بیشینهٔ REA.
    pub max_rea_score: u8,
    /// غیرقابلِ آموزش.
    pub unusable_entries: usize,
    /// کلِ اصطکاک.
    pub friction_events: usize,
    /// اصطکاکِ بهبودیافته.
    pub friction_recovered: usize,
}

/// پاسخِ `GET /api/v1/admin/lab-telemetry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminLabTelemetryResponse {
    /// مدخل‌هایِ تلمتریِ ماندگار (تا سقفِ limit).
    pub entries: Vec<LabTelemetryEntryView>,
    /// داشبوردِ تجمیعی.
    pub dashboard: AdminLabDashboardView,
    /// امضا.
    pub signature: String,
}

/// درخواستِ `POST /api/v1/admin/export-training-dataset`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminExportTrainingDatasetRequest {
    /// آستانهٔ REA — فقط مدخل‌هایِ با `rea_score >=` این مقدار.
    pub min_rea_score: u8,
}

/// پاسخِ `POST /api/v1/admin/export-training-dataset`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminExportTrainingDatasetResponse {
    /// JSONLِ SFT (ChatML/ShareGPT).
    pub sft_jsonl: String,
    /// JSONLِ DPO (`prompt/chosen/rejected`).
    pub dpo_jsonl: String,
    /// رکوردهایِ SFT.
    pub sft_records: usize,
    /// رکوردهایِ DPO.
    pub dpo_records: usize,
    /// آستانهٔ اعمال‌شده.
    pub min_rea_score: u8,
    /// امضا.
    pub signature: String,
}
