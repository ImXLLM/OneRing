//! قراردادهای وایر خط لولهٔ ترینیتی (Wave 3 — استودیوی حاکم `0xKIA`).
//!
//! مسیرهای این ماژول با نشست Bearer شهروند کار می‌کنند:
//! `POST /api/v1/builder/trinity/alpha` — کالبد ۱ (آلفا؛ کشف و قفل مستر پرامپت)
//! و `POST /api/v1/builder/trinity/orchestrate` — کالبد ۲ (اتاق فرمان با
//! ساب‌ایجنت‌های `OpenBot` الگوی `۳×۹`) + کالبد ۳ (داوری `مایرون-۱۵۰` و حلقهٔ
//! خودترمیمی). شکل‌های وایر mirrorِ قطعیِ موتور `kian-hermes::trinity_pipeline`
//! هستند — هیچ منطقی اینجا نوشته نمی‌شود (ADR-005).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// حالت پنج اسلات کشف آلفا — بازگشتِ قطعی به کلاینت برای نوبت بعدی.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinitySlotsState {
    /// هدفِ هسته و هویت محصول.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_objective: Option<String>,
    /// مخاطب هدف و حس UX در ۳۹۰ پیکسل.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_audience_and_ux: Option<String>,
    /// قابلیت‌های تعاملی فریمِ اول.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactive_features: Option<String>,
    /// داده و APIهای بدون احراز.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_and_zero_auth_apis: Option<String>,
    /// درآمدزایی و حلقهٔ وایرال.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monetization_and_viral_loop: Option<String>,
}

/// درخواست نوبت کشف آلفا.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityAlphaRequest {
    /// شناسهٔ نشست ترینیتی — اختیاری؛ در نبودش سرور `trn:{uuid}` می‌سازد.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// پیام خام کاربر در این نوبت (`1..=4000` کاراکتر).
    pub user_message: String,
    /// اسلات‌های پرشدهٔ نوبت‌های قبل — اختیاری.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_slots: Option<TrinitySlotsState>,
}

/// نمای مستر پرامپت کامپایل‌شده روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityMasterPromptView {
    /// اثر انگشت SHA-256 متن کامپایل‌شده.
    pub spec_hash: String,
    /// دستور سیستمی دقیق.
    pub system_directive: String,
    /// مشخصهٔ عملکردی از پنج اسلات.
    pub functional_spec: String,
    /// قیدهای UX در ۳۹۰ پیکسل.
    pub ux_390_constraints: String,
    /// معیارهای راستی‌آزمایی.
    pub verification_criteria: Vec<String>,
    /// متن کامل مستر پرامپت.
    pub compiled_prompt: String,
    /// نام محصول مشتق‌شده.
    pub project_name: String,
    /// پیشنهاد قیمت مایرون به ستارهٔ تلگرام (`XTR`).
    pub price_stars_hint: u64,
}

/// نمای یک ماژول استراکچر پروژه روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityBlueprintModuleView {
    /// مسیر نسبی ماژول (`data-module` در آرتیفکت).
    pub path: String,
    /// نقش کارکردی.
    pub role: String,
    /// وابستگی‌ها.
    pub dependencies: Vec<String>,
    /// تخصص ساب‌ایجنت هدف (شناسهٔ وایر).
    pub specialty: String,
    /// برچسب فارسی تخصص.
    pub specialty_fa: String,
}

/// نمای استراکچر پروژه روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityBlueprintView {
    /// نام محصول.
    pub project_name: String,
    /// نام لاتین امن (`slug`).
    pub project_slug: String,
    /// ماژول‌ها به ترتیب قطعی.
    pub modules: Vec<TrinityBlueprintModuleView>,
}

/// پاسخ یک نوبت کشف آلفا — یا ادامهٔ کشف یا قفلِ تحویل به اتاق فرمان.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityAlphaResponse {
    /// وضعیت (`needs_more_discovery` | `locked_for_command_center`).
    pub status: String,
    /// شناسهٔ نشست ترینیتی.
    pub session_id: String,
    /// حساب شهروند سازنده.
    pub citizen_id: String,
    /// پاسخ فارسی آلفا (جرقه یا تحویل).
    pub alpha_reply_fa: String,
    /// اسلات‌های خام به ترتیب قطعی (شناسه‌های وایر).
    pub missing_slots: Vec<String>,
    /// درصد آمادگی (`0..=100`).
    pub readiness_pct: u8,
    /// وضعیت به‌روز اسلات‌ها — برای نوبت بعدی بازگردانده می‌شود.
    pub updated_slots: TrinitySlotsState,
    /// مستر پرامپت — فقط در وضعیت قفل.
    pub master_prompt: Option<TrinityMasterPromptView>,
    /// استراکچر پروژه — فقط در وضعیت قفل.
    pub blueprint: Option<TrinityBlueprintView>,
}

/// آلل‌های تصادفی قطعی ساب‌ایجنت روی وایر.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityAlleleView {
    /// چگالی جراحی.
    pub surgical_density: f64,
    /// مهارت صنعتگری UX.
    pub ux_craft: f64,
    /// پارانویای لبه‌ها.
    pub edge_case_paranoia: f64,
    /// سرعت اجرا.
    pub execution_velocity: f64,
    /// بذر آلل (بازتولید قطعی).
    pub allele_seed: u64,
}

/// مانیفست DNA یک ساب‌ایجنت `OpenBot` (الگوی `۳×۹`) روی وایر.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinitySubAgentDnaView {
    /// شمار ردیفی ساب‌ایجنت.
    pub subagent_index: u32,
    /// تخصص (شناسهٔ وایر).
    pub specialty: String,
    /// برچسب فارسی تخصص.
    pub specialty_fa: String,
    /// ماژول(های) محول‌شده.
    pub assigned_modules: Vec<String>,
    /// شناسهٔ نخ قطعی.
    pub thread_id: String,
    /// IQ مؤثر اتاق فرمان (`139`).
    pub logic_iq: u16,
    /// EQ مؤثر اتاق فرمان (`129`).
    pub logic_eq: u16,
    /// مرز استال نرم قلاب توقف (`450s`).
    pub watchdog_soft_secs: u64,
    /// مرز تایم‌اوت سخت قلاب توقف (`1200s`).
    pub watchdog_hard_secs: u64,
    /// حالت فشرده‌سازی `Caveman` (`off|lite|full|ultra`).
    pub caveman: String,
    /// آلل‌های تصادفی.
    pub alleles: TrinityAlleleView,
    /// طول لایهٔ ۱ (هستهٔ ثابت کش-پین).
    pub layer1_chars: usize,
    /// طول لایهٔ ۲ (زمینهٔ نشست کش-پین).
    pub layer2_chars: usize,
    /// طول لایهٔ ۳ (محولهٔ پویا).
    pub layer3_chars: usize,
    /// پیش‌نمایش پرامپت رندرشده (حدود ۲۲۰ کاراکتر اول).
    pub prompt_preview: String,
    /// طول XML ژنوم کامپایل‌شده (`<openbot_subagent_dna>`).
    pub xml_prompt_chars: usize,
    /// پروب‌های تجربیِ فاکتور f08.
    pub empirical_probes: Vec<String>,
    /// کارت تحویل فاکتور f09.
    pub handoff_card: String,
    /// گیت پاک‌ساز f07 — صفر نشانگر ناتمام/راز/اسلوپ.
    pub sanitizer_passed: bool,
}

/// چهار ستون REA روی وایر (`0..=25` هر ستون).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityPillarsView {
    /// معماری.
    pub architecture: u8,
    /// سخت‌سازی امنیت.
    pub security_hardening: u8,
    /// UX و ضد-اسلوپ.
    pub ux_and_anti_slop: u8,
    /// شواهد راستی‌آزمایی.
    pub verification_proof: u8,
}

/// فرمان ترمیم مایرون روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityRepairDirectiveView {
    /// گذر ترمیم (`1..=3`).
    pub attempt: u32,
    /// تخصص مقصر (شناسهٔ وایر).
    pub target_specialty: String,
    /// ستون REA زیر آستانه.
    pub pillar: String,
    /// دستور اصلاح (انگلیسی).
    pub directive_en: String,
    /// دستور اصلاح (فارسی).
    pub directive_fa: String,
}

/// یک دور ترمیم روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityRepairRoundView {
    /// گذر ترمیم.
    pub attempt: u32,
    /// نمرهٔ پیش از ترمیم.
    pub score_before: u8,
    /// نمرهٔ پس از ترمیم.
    pub score_after: u8,
    /// عبور از گیت `>= 85`.
    pub approved: bool,
    /// فرمان‌های همین گذر.
    pub directives: Vec<TrinityRepairDirectiveView>,
}

/// مشاورهٔ استراتژیک مایرون `IQ=150 / EQ=145` روی وایر.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityAdvisoryView {
    /// IQ اورکلاک داور (`150`).
    pub iq: u16,
    /// EQ اورکلاک داور (`145`).
    pub eq: u16,
    /// افق نگاه بازی‌نظری (`8`).
    pub lookahead_horizon: u8,
    /// مرتبهٔ نظریهٔ ذهن (`3`).
    pub tom_order: u8,
    /// بهرهٔ جاه‌طلبی (`0.99`).
    pub ambition_gain: f32,
    /// خلاصهٔ نگاه هشت‌گامی.
    pub eight_step_lookahead_summary: String,
    /// قلاب روان‌شناختی خریدار و قیمت/معرف (فارسی، ToM-3).
    pub third_order_tom_market_hook_fa: String,
}

/// درخواست اجرای اتاق فرمان + داوری مایرون — اسلات‌های قفل‌شدهٔ آلفا.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityOrchestrateRequest {
    /// شناسهٔ نشست ترینیتیِ قفل‌شده.
    pub session_id: String,
    /// بذر قطعی نشست — اختیاری؛ پیش‌فرض از `SHA-256(session_id)` مشتق می‌شود.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_seed: Option<u64>,
    /// پنج اسلات — آمادگی باید `>= 80` باشد.
    pub slots: TrinitySlotsState,
}

/// پاسخ کامل آهنگری ترینیتی — مانیفست DNA، آرتیفکت، حکم مایرون و تاریخچهٔ ترمیم.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityOrchestrateResponse {
    /// شناسهٔ نشست ترینیتی.
    pub session_id: String,
    /// نام محصول.
    pub project_name: String,
    /// آرتیفکت HTML کاملِ مونتاژشده.
    pub artifact_html: String,
    /// حجم آرتیفکت بر حسب بایت.
    pub artifact_bytes: usize,
    /// شمار پروب‌های تجربی تزریق‌شده.
    pub probe_count: usize,
    /// ماژول‌های یکپارچه‌شده (`data-module`).
    pub integrated_modules: Vec<String>,
    /// مانیفست DNA ناوگان `OpenBot`.
    pub manifest: Vec<TrinitySubAgentDnaView>,
    /// نمرهٔ نهایی مایرون (`0..=100`).
    pub final_score: u8,
    /// عبور از گیت آهنگری (`>= 85` و صفر خط قرمز).
    pub approved_for_listing: bool,
    /// نشان مرئی مایرون.
    pub mairon_badge: String,
    /// تفکیک چهار ستون.
    pub pillars: TrinityPillarsView,
    /// مشاورهٔ استراتژیک `IQ=150/EQ=145`.
    pub advisory: TrinityAdvisoryView,
    /// شمار تلاش‌های ترمیم مصرف‌شده (`0..=3`).
    pub repair_attempts_used: u32,
    /// تاریخچهٔ دورهای ترمیم.
    pub repair_history: Vec<TrinityRepairRoundView>,
}
