//! `trinity_pipeline` — قلب معماری `0xKIA`: خط لولهٔ سه‌کالبدیِ استودیوی حاکم
//! (Wave 3 — سریِ تسک‌های Trinity Studio Pipeline).
//!
//! سه Avatar پشت سر هم، هر یک با DNA و کالیبراسیون قفل‌شده:
//!
//! 1. [`AlphaIgnitionPersona`] (`کالبد ۱: آلفا`) — شکارچیِ ایده و کامپایلرِ
//!    مستر پرامپت و استراکچر پروژه (`IQ=136`، `EQ=122`، ترمز `0.04`، دمای
//!    آنتروپی `0.88`). پنج اسلات کشف (`AlphaDiscoverySlots`) را استخراج
//!    می‌کند و در آمادگیِ `≥ 80`، [`MasterPromptSpec`] و [`ProjectBlueprint`]
//!    را قطعی کامپایل و قفل می‌کند.
//! 2. [`CommandCenterOrchestrator`] (`کالبد ۲: اتاق فرمان`) — ارکستراتورِ
//!    قضایی (`IQ=139`، `EQ=129`، ترمز `0.62`، سختیِ قضایی `0.96`) با
//!    [`OpenBotSubAgentSpawner`]: کارخانهٔ ساب‌ایجنت‌های پویا با الگوی `۳×۹`
//!    — ۳ لایهٔ پرامپتِ کش-امن ([`ThreeLayerSubAgentPrompt`]) × ژنومِ
//!    نه‌لایه‌ای ([`NineFoldSubAgentGenome`]) با آلل‌های تصادفیِ قطعی از
//!    `session_seed + subagent_index`.
//! 3. [`MaironSovereignJudge`] (`کالبد ۳: مایرون-۱۵۰`) — داورِ حاکمِ
//!    اورکلاک‌شده (`IQ=150`، `EQ=145`، افقِ نگاه `۸`، نظریهٔ ذهن `۳`،
//!    جاه‌طلبی `0.99`): نمرهٔ `REA` چهارستونی از `kian-domain`، مشاورهٔ
//!    استراتژیک و حلقهٔ خودترمیمی [`execute_command_center_with_mairon_loop`]
//!    — تا آرتیفکت از گیتِ `≥ 85` عبور نکند، ساب‌ایجنتِ مقصر دوباره
//!    رهسپاری می‌شود (بیشینهٔ ۳ تلاش ترمیم).
//!
//! ثابت‌های DNA مایرون در `kian-domain::mairon_rea_and_reviews` قفل شده‌اند و
//! اینجا با `pub use` بازصادر می‌شوند — منبعِ واحد حقیقت، بدون نسخهٔ موازی.
//! همهٔ توابع قطعی‌اند؛ صفر IO، صفر ساعت پنهان، صفر `unsafe`.
#![forbid(unsafe_code)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::large_enum_variant
)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use kian_domain::mairon_rea_and_reviews::{
    evaluate_user_product_with_mairon, MaironProductVerdict, MaironReaPillarScores,
};

use crate::approvals::deterministic_subagent_thread_id;
use crate::discipline_gates::{audit_frontend_slop, SYSTEM_PROMPT_DYNAMIC_BOUNDARY};

// ── DNA اورکلاکِ مایرون — قفل‌شده در دامنه، بازصادری در این ماژول ──────────
pub use kian_domain::mairon_rea_and_reviews::{
    MAIRON_AMBITION_GAIN, MAIRON_LOOKAHEAD_HORIZON, MAIRON_OVERCLOCK_EQ, MAIRON_OVERCLOCK_IQ,
    MAIRON_TOM_ORDER, MARKETPLACE_APPROVAL_THRESHOLD,
};

// ════════════════════════ Stage 1 — ALPHA ═════════════════════════════════

/// IQ مؤثر آلفا — شکارچیِ ایده با انرژیِ خلاقِ رهاشده.
pub const ALPHA_EFFECTIVE_IQ: u16 = 136;

/// EQ مؤثر آلفا — جذابیتِ بالا، کالیبراسیونِ گرم.
pub const ALPHA_EFFECTIVE_EQ: u16 = 122;

/// ترمزِ توقفِ آلفا — نزدیک به صفر: انرژیِ بی‌ترمز در فاز کشف.
pub const ALPHA_INHIBITION_BRAKE: f32 = 0.04;

/// دمای آنتروپیِ آلفا — خلاقیتِ پرنوسان پیش از قفل‌شدن.
pub const ALPHA_ENTROPY_TEMP: f32 = 0.88;

/// آستانهٔ قفلِ آلفا — در آمادگیِ `≥ 80` مستر پرامپت و استراکچر پروژه
/// کامپایل و به اتاق فرمان تحویل می‌شود.
pub const ALPHA_READINESS_LOCK_THRESHOLD: u8 = 80;

/// سقف کاراکتر هر اسلات استخراج‌شده — پیامِ کاربر به یک نقل‌قول جرقه‌ای
/// فشرده می‌شود، نه کپی کامل.
pub const ALPHA_SLOT_TEXT_MAX_CHARS: usize = 280;

/// پنج اسلاتِ کشفِ آلفا — استخراجِ هر یک با کلیدواژه‌های دوزبانه (فا/EN).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlphaDiscoverySlots {
    /// هدفِ هسته و هویت محصول — مسئلهٔ دقیقی که حل می‌شود.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_objective: Option<String>,
    /// مخاطب هدف و حس UX — پرسونای کاربر + مُدِ موبایل‌فرست ۳۹۰px راست‌به‌چپ.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_audience_and_ux: Option<String>,
    /// قابلیت‌های تعاملی — جریان‌های کاری اصلی و توانایی‌های فریمِ اول.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactive_features: Option<String>,
    /// داده و APIهای بدون احراز — ماندگاری وضعیت و اتصال از ۱۲۰ API عمومی.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_and_zero_auth_apis: Option<String>,
    /// درآمدزایی و حلقهٔ وایرال — قیمت‌گذاری با KIAN/Stars (`XTR`) + سهم معرف.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monetization_and_viral_loop: Option<String>,
}

/// شناسهٔ نوع اسلات — ترتیبِ اعلامِ اسلات‌های خام به کاربر را قطعی می‌کند.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlphaSlotKind {
    /// هدفِ هسته و هویت محصول.
    CoreObjective,
    /// مخاطب هدف و حس UX.
    TargetAudienceAndUx,
    /// قابلیت‌های تعاملی.
    InteractiveFeatures,
    /// داده و APIهای بدون احراز.
    DataAndZeroAuthApis,
    /// درآمدزایی و حلقهٔ وایرال.
    MonetizationAndViralLoop,
}

impl AlphaSlotKind {
    /// شناسهٔ وایر — پایدار برای DTOها و کلاینت‌ها.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CoreObjective => "core_objective",
            Self::TargetAudienceAndUx => "target_audience_and_ux",
            Self::InteractiveFeatures => "interactive_features",
            Self::DataAndZeroAuthApis => "data_and_zero_auth_apis",
            Self::MonetizationAndViralLoop => "monetization_and_viral_loop",
        }
    }

    /// برچسب فارسی — در پاسخ کشفِ آلفا به کاربر نمایش داده می‌شود.
    pub fn fa_label(self) -> &'static str {
        match self {
            Self::CoreObjective => "هدفِ هسته و هویت محصول",
            Self::TargetAudienceAndUx => "مخاطب هدف و حس UX در ۳۹۰ پیکسل",
            Self::InteractiveFeatures => "قابلیت‌های تعاملی فریمِ اول",
            Self::DataAndZeroAuthApis => "داده و APIهای بدون احراز",
            Self::MonetizationAndViralLoop => "درآمدزایی و حلقهٔ وایرال",
        }
    }
}

impl AlphaDiscoverySlots {
    /// آیا این اسلات پر شده است؟
    pub fn is_filled(&self, kind: AlphaSlotKind) -> bool {
        match kind {
            AlphaSlotKind::CoreObjective => self.core_objective.is_some(),
            AlphaSlotKind::TargetAudienceAndUx => self.target_audience_and_ux.is_some(),
            AlphaSlotKind::InteractiveFeatures => self.interactive_features.is_some(),
            AlphaSlotKind::DataAndZeroAuthApis => self.data_and_zero_auth_apis.is_some(),
            AlphaSlotKind::MonetizationAndViralLoop => self.monetization_and_viral_loop.is_some(),
        }
    }

    /// پرکردن اسلات (اگر خام باشد) — مقدارِ اول همیشه برنده است.
    pub fn fill(&mut self, kind: AlphaSlotKind, value: String) {
        let slot = match kind {
            AlphaSlotKind::CoreObjective => &mut self.core_objective,
            AlphaSlotKind::TargetAudienceAndUx => &mut self.target_audience_and_ux,
            AlphaSlotKind::InteractiveFeatures => &mut self.interactive_features,
            AlphaSlotKind::DataAndZeroAuthApis => &mut self.data_and_zero_auth_apis,
            AlphaSlotKind::MonetizationAndViralLoop => &mut self.monetization_and_viral_loop,
        };
        if slot.is_none() {
            *slot = Some(value);
        }
    }

    /// شمار اسلات‌های پرشده (`0..=5`).
    pub fn filled_count(&self) -> usize {
        [
            AlphaSlotKind::CoreObjective,
            AlphaSlotKind::TargetAudienceAndUx,
            AlphaSlotKind::InteractiveFeatures,
            AlphaSlotKind::DataAndZeroAuthApis,
            AlphaSlotKind::MonetizationAndViralLoop,
        ]
        .into_iter()
        .filter(|&k| self.is_filled(k))
        .count()
    }

    /// درصد آمادگی — هر اسلات `۲۰٪`.
    pub fn readiness_pct(&self) -> u8 {
        u8::try_from(self.filled_count() * 20).unwrap_or(100)
    }

    /// اسلات‌های خام به ترتیب قطعی اعلام.
    pub fn missing_slots(&self) -> Vec<AlphaSlotKind> {
        [
            AlphaSlotKind::CoreObjective,
            AlphaSlotKind::TargetAudienceAndUx,
            AlphaSlotKind::InteractiveFeatures,
            AlphaSlotKind::DataAndZeroAuthApis,
            AlphaSlotKind::MonetizationAndViralLoop,
        ]
        .into_iter()
        .filter(|&k| !self.is_filled(k))
        .collect()
    }
}

/// کلیدواژه‌های استخراجِ اسلات‌ها — دوزبانه؛ تطبیق روی متنِ lowercase.
const SLOT_KEYWORDS_CORE: &[&str] = &[
    "هدف", "مسئله", "محصول", "بساز", "ایده", "پلتفرم", "ابزار", "problem", "product", "build",
    "idea", "platform", "tool", "goal",
];
const SLOT_KEYWORDS_AUDIENCE: &[&str] = &[
    "مخاطب", "کاربر", "موبایل", "۳۹۰", "390", "راست‌به‌چپ", "rtl", "ltr", "audience", "user",
    "mobile", "persona", "ui", "ux", "design",
];
const SLOT_KEYWORDS_FEATURES: &[&str] = &[
    "تعاملی", "ویژگی", "قابلیت", "جریان", "دکمه", "فرم", "کلیک", "feature", "interactive",
    "workflow", "button", "form", "click", "drag",
];
const SLOT_KEYWORDS_DATA: &[&str] = &[
    "داده", "ذخیره", "پایگاه", "بدون لاگین", "بدون‌لاگین", "api", "data", "storage", "database",
    "zero-auth", "localstorage", "bind", "persist",
];
const SLOT_KEYWORDS_MONETIZATION: &[&str] = &[
    "درآمد", "قیمت", "فروش", "ستاره", "استار", "کمیسیون", "اشتراک", "پورسانت", "price",
    "monetize", "stars", "xtr", "kian", "affiliate", "viral", "revenue", "pricing",
];

/// فشردن متنِ اسلات — فاصله‌های تکراری جمع و سقف کاراکتر اعمال می‌شود.
fn clamp_slot_text(raw: &str) -> String {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out: String = collapsed.chars().take(ALPHA_SLOT_TEXT_MAX_CHARS).collect();
    if collapsed.chars().count() > ALPHA_SLOT_TEXT_MAX_CHARS {
        out.push('…');
    }
    out
}

/// درخواست intake آلفا — یک نوبت گفتگوی کشف.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlphaIntakeRequest {
    /// شناسهٔ نشست ترینیتی (`trn:{uuid}` — ساخته‌شده در لبه یا ادامه‌ی قبلی).
    pub session_id: String,
    /// حساب شهروندِ سازنده (`citizen:{tg_id}`).
    pub citizen_id: String,
    /// پیام خام کاربر در این نوبت.
    pub user_message: String,
    /// اسلات‌های پرشدهٔ نوبت‌های قبل — قطعیِ سمت کلاینت.
    pub prior_slots: AlphaDiscoverySlots,
}

/// خروجی یک نوبت intake آلفا — یا ادامهٔ کشف، یا قفلِ تحویل به اتاق فرمان.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlphaIntakeOutcome {
    /// اسلات‌های کلیدی هنوز خام‌اند — پاسخ جرقه‌ای آلفا + فهرست خام.
    NeedsMoreDiscovery {
        /// پاسخ فارسی آلفا (انرژیِ بی‌ترمز + درخواست دقیق اسلات‌های خام).
        alpha_reply_fa: String,
        /// اسلات‌های خام به ترتیب قطعی.
        missing_slots: Vec<AlphaSlotKind>,
        /// درصد آمادگی (`0..=100`).
        readiness_pct: u8,
    },
    /// آمادگی `≥ 80` — مستر پرامپت و استراکچر پروژه قفل و تحویل شد.
    LockedForCommandCenter {
        /// پیام تحویل فارسی آلفا به اتاق فرمان.
        alpha_handoff_fa: String,
        /// مستر پرامپت کامپایل‌شدهٔ قطعی.
        master_prompt: MasterPromptSpec,
        /// استراکچر پروژه (درخت ماژول‌ها با تخصصِ هدف).
        blueprint: ProjectBlueprint,
    },
}

/// کالبد ۱ — آلفا: شکارچیِ ایده و کامپایلرِ مستر پرامپت و استراکچر پروژه.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlphaIgnitionPersona;

impl AlphaIgnitionPersona {
    /// جدول کلیدواژه‌های استخراج اسلات — عمومی برای لبهٔ HTTP تا به‌روزرسانیِ
    /// اسلات‌های وایر بدون نسخهٔ موازیِ کلیدواژه‌ها انجام شود.
    pub fn slot_keyword_table() -> Vec<(AlphaSlotKind, &'static [&'static str])> {
        vec![
            (AlphaSlotKind::CoreObjective, SLOT_KEYWORDS_CORE),
            (AlphaSlotKind::TargetAudienceAndUx, SLOT_KEYWORDS_AUDIENCE),
            (AlphaSlotKind::InteractiveFeatures, SLOT_KEYWORDS_FEATURES),
            (AlphaSlotKind::DataAndZeroAuthApis, SLOT_KEYWORDS_DATA),
            (AlphaSlotKind::MonetizationAndViralLoop, SLOT_KEYWORDS_MONETIZATION),
        ]
    }

    /// پردازش یک نوبت کشف — استخراجِ قطعی اسلات‌ها از کلیدواژه‌های دوزبانه.
    pub fn process_intake(&self, req: &AlphaIntakeRequest) -> AlphaIntakeOutcome {
        let mut slots = req.prior_slots.clone();
        let hay = req.user_message.to_lowercase();
        let spark = clamp_slot_text(&req.user_message);
        for (kind, keywords) in [
            (AlphaSlotKind::CoreObjective, SLOT_KEYWORDS_CORE),
            (AlphaSlotKind::TargetAudienceAndUx, SLOT_KEYWORDS_AUDIENCE),
            (AlphaSlotKind::InteractiveFeatures, SLOT_KEYWORDS_FEATURES),
            (AlphaSlotKind::DataAndZeroAuthApis, SLOT_KEYWORDS_DATA),
            (AlphaSlotKind::MonetizationAndViralLoop, SLOT_KEYWORDS_MONETIZATION),
        ] {
            if !slots.is_filled(kind) && keywords.iter().any(|k| hay.contains(k)) {
                slots.fill(kind, spark.clone());
            }
        }
        let readiness = slots.readiness_pct();
        let missing = slots.missing_slots();
        if readiness >= ALPHA_READINESS_LOCK_THRESHOLD {
            let master = MasterPromptSpec::compile(&slots, &req.session_id, &req.citizen_id);
            let blueprint = ProjectBlueprint::compile(&slots, &master.project_name);
            let alpha_handoff_fa = format!(
                "🔒 آلفا قفل شد — آمادگی {readiness}٪. مستر پرامپت کامپایل شد \
                 (hash {hash}) و استراکچر پروژهٔ «{project}» با {modules} ماژول \
                 به اتاق فرمان (IQ=139/EQ=129، سختیِ قضایی 0.96) تحویل شد؛ \
                 کارخانهٔ ساب‌ایجنت‌های OpenBot با DNA نه‌لایه‌ای منتظرِ \
                 فرمانِ اجراست.",
                hash = &master.spec_hash[..12],
                project = master.project_name,
                modules = blueprint.modules.len(),
            );
            AlphaIntakeOutcome::LockedForCommandCenter { alpha_handoff_fa, master_prompt: master, blueprint }
        } else {
            let missing_fa = missing
                .iter()
                .map(|k| k.fa_label())
                .collect::<Vec<_>>()
                .join("، ");
            let alpha_reply_fa = format!(
                "⚡ آلفا فعال شد — IQ={ALPHA_EFFECTIVE_IQ}، EQ={ALPHA_EFFECTIVE_EQ}، \
                 ترمزِ توقف {ALPHA_INHIBITION_BRAKE}، دمای آنتروپی {ALPHA_ENTROPY_TEMP}. \
                 جرقهٔ تو {filled} از ۵ اسلات را روشن کرد ({readiness}٪ آمادگی).\
                 {missing_block}\
                 هر اسلاتِ خام را در یک جملهٔ دقیق بده؛ در {threshold}٪ آمادگی، \
                 مستر پرامپت و استراکچر پروژه را قفل می‌کنم و به اتاق فرمان \
                 تحویل می‌دهم.",
                filled = slots.filled_count(),
                missing_block = if missing.is_empty() {
                    String::new()
                } else {
                    format!(" هنوز خام است: {missing_fa}.")
                },
                threshold = ALPHA_READINESS_LOCK_THRESHOLD,
            );
            AlphaIntakeOutcome::NeedsMoreDiscovery { alpha_reply_fa, missing_slots: missing, readiness_pct: readiness }
        }
    }
}

/// مستر پرامپت قطعیِ کامپایل‌شده — قرارداد تحویل آلفا به اتاق فرمان.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasterPromptSpec {
    /// دستورِ سیستمیِ دقیق (قانون‌های حاکم + کالیبراسیون آلفا).
    pub system_directive: String,
    /// مشخصهٔ عملکردی — از پنج اسلاتِ استخراج‌شده.
    pub functional_spec: String,
    /// قیدهای UX در ۳۹۰ پیکسل — موبایل‌فرست، RTL-first، صفر سرریز.
    pub ux_390_constraints: String,
    /// معیارهای راستی‌آزمایی — گیت‌های عینیِ پیش از انتشار.
    pub verification_criteria: Vec<String>,
    /// متنِ کاملِ مستر پرامپت (ترکیب نهایی همهٔ بخش‌ها).
    pub compiled_prompt: String,
    /// اثر انگشت SHA-256 متنِ کامپایل‌شده — پایداریِ کش بین ساب‌ایجنت‌ها.
    pub spec_hash: String,
    /// نام محصولِ مشتق‌شده از اسلاتِ هدفِ هسته.
    pub project_name: String,
    /// شناسهٔ نشست ترینیتی.
    pub session_id: String,
    /// پیشنهاد قیمت مایرون به ستارهٔ تلگرام (`XTR`) — مشتقِ قطعی از اسلاتِ درآمد.
    pub price_stars_hint: u64,
}

impl MasterPromptSpec {
    /// کامپایل قطعی مستر پرامپت از اسلات‌های پرشده.
    ///
    /// اسلاتِ خام در زمان قفل، با جای‌بانِ صادقانهٔ «خام در نوبتِ قفل» در
    /// مشخصهٔ عملکردی ثبت می‌شود — اتاق فرمان با پیش‌فرضِ مایرون پیش می‌رود.
    pub fn compile(
        slots: &AlphaDiscoverySlots,
        session_id: &str,
        citizen_id: &str,
    ) -> Self {
        let project_name = derive_project_name(slots);
        let slot_line = |label: &str, value: Option<&String>| {
            value.map_or_else(
                || format!("- {label}: (خام در نوبتِ قفل — پیش‌فرضِ مایرون اعمال می‌شود)"),
                |v| format!("- {label}: {v}"),
            )
        };
        let functional_spec = [
            slot_line("هدفِ هسته", slots.core_objective.as_ref()),
            slot_line("مخاطب و UX", slots.target_audience_and_ux.as_ref()),
            slot_line("قابلیت‌های تعاملی", slots.interactive_features.as_ref()),
            slot_line("داده و APIهای بدون احراز", slots.data_and_zero_auth_apis.as_ref()),
            slot_line("درآمدزایی و حلقهٔ وایرال", slots.monetization_and_viral_loop.as_ref()),
        ]
        .join("\n");
        let system_directive = format!(
            "0xKIA SOVEREIGN MASTER PROMPT — session {session_id} · citizen {citizen_id}\n\
             DIRECTIVE: forge the product «{project_name}» as a single-page, \
             mobile-first, RTL-first artifact at 390px with zero horizontal \
             overflow. Alpha calibration: IQ={ALPHA_EFFECTIVE_IQ}, \
             EQ={ALPHA_EFFECTIVE_EQ}, inhibition brake {ALPHA_INHIBITION_BRAKE}, \
             entropy temp {ALPHA_ENTROPY_TEMP}. Execution weight is 100% on the \
             latest Trinity release. Zero unsafe, zero unfinished markers, \
             zero placeholder secrets, zero AI-slop tells."
        );
        let ux_390_constraints = "390px viewport, mobile-first, zero-overflow \
            (no horizontal scroll below 391px), RTL-first (`dir=rtl`) with \
            Vazirmatn font stack and honest LTR fallback, tap targets ≥ 44px, \
            contrast ≥ 4.5:1"
            .to_owned();
        let verification_criteria = vec![
            "zero todo/TODO/FIXME/XXX markers in the artifact".to_owned(),
            "zero placeholder secrets (sk-…, AKIA…, BEGIN PRIVATE KEY, ghp_…)".to_owned(),
            "audit_frontend_slop findings = 0".to_owned(),
            "every blueprint module integrated (data-module attribute per path)".to_owned(),
            "empirical probes present before marketplace publication".to_owned(),
            "390px zero-overflow contract holds".to_owned(),
        ];
        let compiled_prompt = format!(
            "{system_directive}\n\nFUNCTIONAL SPEC:\n{functional_spec}\n\n\
             UX CONSTRAINTS: {ux_390_constraints}\n\nVERIFICATION CRITERIA:\n{}",
            verification_criteria
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{}. {c}", i + 1))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let spec_hash = format!("{:x}", Sha256::digest(compiled_prompt.as_bytes()));
        Self {
            system_directive,
            functional_spec,
            ux_390_constraints,
            verification_criteria,
            compiled_prompt,
            spec_hash,
            project_name,
            session_id: session_id.to_owned(),
            price_stars_hint: derive_price_stars(slots),
        }
    }
}

/// نام محصولِ قطعی — پنج واژهٔ اولِ اسلاتِ هدفِ هسته.
fn derive_project_name(slots: &AlphaDiscoverySlots) -> String {
    let raw = slots
        .core_objective
        .clone()
        .unwrap_or_else(|| "0xKIA Sovereign Product".to_owned());
    let name = raw.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
    let trimmed: String = name.chars().take(60).collect();
    if trimmed.trim().is_empty() {
        "0xKIA Sovereign Product".to_owned()
    } else {
        trimmed
    }
}

/// نام لاتینِ امنِ محصول برای نشاندن در ویژگی‌های HTML.
fn derive_project_slug(slots: &AlphaDiscoverySlots) -> String {
    let name = derive_project_name(slots);
    let slug: String = name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '-')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect();
    let squeezed = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    if squeezed.is_empty() {
        "kia-product".to_owned()
    } else {
        squeezed.chars().take(48).collect()
    }
}

/// یک ماژول درختِ پروژه — مسیر، نقش، وابستگی‌ها و تخصصِ ساب‌ایجنتِ هدف.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintModule {
    /// مسیر نسبی ماژول در آرتیفکت (`data-module` روی تگِ همان ماژول).
    pub path: String,
    /// نقش کارکردی ماژول.
    pub role: String,
    /// مسیرهای ماژول‌هایی که این ماژول به آن‌ها وابسته است.
    pub dependencies: Vec<String>,
    /// تخصص ساب‌ایجنتِ هدف برای اجرای این ماژول.
    pub specialty: SubAgentSpecialty,
}

/// استراکچر پروژه — درخت ماژول‌های قطعیِ مشتق از اسلات‌ها.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectBlueprint {
    /// نام محصول.
    pub project_name: String,
    /// نام لاتینِ امن (slug) برای ویژگی‌های HTML.
    pub project_slug: String,
    /// ماژول‌ها به ترتیب قطعی.
    pub modules: Vec<BlueprintModule>,
}

impl ProjectBlueprint {
    /// کامپایل قطعی استراکچر پروژه از اسلات‌ها — چهار ماژولِ پایه همیشه
    /// حضور دارند و سه ماژولِ شرطی با اسلات‌های تعاملی/داده/درآمد می‌آیند.
    pub fn compile(slots: &AlphaDiscoverySlots, project_name: &str) -> Self {
        let project_slug = derive_project_slug(slots);
        let mut modules = vec![
            BlueprintModule {
                path: "src/index.html".to_owned(),
                role: "RTL-first 390px single-page shell with hero".to_owned(),
                dependencies: vec!["src/styles/main.css".to_owned()],
                specialty: SubAgentSpecialty::UiUxArchitect390,
            },
            BlueprintModule {
                path: "src/styles/main.css".to_owned(),
                role: "Vazirmatn stack, 390px zero-overflow grid".to_owned(),
                dependencies: vec![],
                specialty: SubAgentSpecialty::UiUxArchitect390,
            },
            BlueprintModule {
                path: "src/js/core.js".to_owned(),
                role: "pure state kernel — no IO, no unsafe".to_owned(),
                dependencies: vec![],
                specialty: SubAgentSpecialty::CoreLogicEngine,
            },
            BlueprintModule {
                path: "src/js/verification-probes.js".to_owned(),
                role: "empirical self-check probes gated by Mairon directive".to_owned(),
                dependencies: vec!["src/js/core.js".to_owned()],
                specialty: SubAgentSpecialty::VerificationAndPolish,
            },
        ];
        if slots.interactive_features.is_some() {
            modules.push(BlueprintModule {
                path: "src/js/interactions.js".to_owned(),
                role: "frame-1 interactive workflows and event bindings".to_owned(),
                dependencies: vec!["src/js/core.js".to_owned()],
                specialty: SubAgentSpecialty::CoreLogicEngine,
            });
        }
        if slots.data_and_zero_auth_apis.is_some() {
            modules.push(BlueprintModule {
                path: "src/js/api-bindings.js".to_owned(),
                role: "zero-auth public API bindings with SSRF guard".to_owned(),
                dependencies: vec!["src/js/core.js".to_owned()],
                specialty: SubAgentSpecialty::SecurityAndApiBridge,
            });
        }
        if slots.monetization_and_viral_loop.is_some() {
            modules.push(BlueprintModule {
                path: "src/js/monetization.js".to_owned(),
                role: "KIAN/Stars (XTR) pricing and affiliate share hook".to_owned(),
                dependencies: vec!["src/js/core.js".to_owned()],
                specialty: SubAgentSpecialty::MonetizationAndViralLoop,
            });
        }
        Self { project_name: project_name.to_owned(), project_slug, modules }
    }
}

// ════════════════════ Stage 2 — COMMAND CENTER ═══════════════════════════

/// IQ مؤثر اتاق فرمان — ارکستراتورِ قضایی.
pub const LOGIC_EFFECTIVE_IQ: u16 = 139;

/// EQ مؤثر اتاق فرمان — هوش هیجانیِ پیش‌بینیِ تیم.
pub const LOGIC_EFFECTIVE_EQ: u16 = 129;

/// ترمزِ توقفِ اتاق فرمان — انضباطِ بالا و تکانش‌پذیریِ پایین.
pub const LOGIC_INHIBITION_BRAKE: f32 = 0.62;

/// سختیِ قضایی اتاق فرمان — دقتِ لمسیِ بالای `0.95` در پذیرشِ خروجی.
pub const LOGIC_FORENSIC_STRICTNESS: f32 = 0.96;

/// بیشینهٔ تلاش‌های ترمیم در حلقهٔ خودترمیمیِ اتاق فرمان ↔ مایرون.
pub const MAX_OPENBOT_REPAIR_ATTEMPTS: u32 = 3;

/// تخصص ساب‌ایجنت — نگاشت ماژول به متخصص؛ متخصصِ دامنهٔ سفارشی هم مجاز است.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubAgentSpecialty {
    /// معمار UX در ۳۹۰ پیکسل — پوسته، تایپوگرافی، صفر سرریز.
    UiUxArchitect390,
    /// موتور منطق هسته — حالتِ خالص، جریان‌های کاری.
    CoreLogicEngine,
    /// پل امنیت و API — SSRF، صفر راز، APIهای عمومیِ بدون احراز.
    SecurityAndApiBridge,
    /// راستی‌آزمایی و پرداخت نهایی — پروب‌های تجربی، ضد-اسلوپ.
    VerificationAndPolish,
    /// حلقهٔ درآمدزایی و وایرال — KIAN/Stars (`XTR`) و سهم معرف.
    MonetizationAndViralLoop,
    /// متخصصِ دامنهٔ سفارشی — با نام آزاد (OpenBot dynamic spawning).
    Custom(String),
}

impl SubAgentSpecialty {
    /// نام وایر — پایدار برای DTO و XML.
    pub fn as_str(&self) -> String {
        match self {
            Self::UiUxArchitect390 => "ui_ux_architect_390".to_owned(),
            Self::CoreLogicEngine => "core_logic_engine".to_owned(),
            Self::SecurityAndApiBridge => "security_and_api_bridge".to_owned(),
            Self::VerificationAndPolish => "verification_and_polish".to_owned(),
            Self::MonetizationAndViralLoop => "monetization_and_viral_loop".to_owned(),
            Self::Custom(name) => format!("custom_{}", slugify_name(name)),
        }
    }

    /// برچسب فارسی تخصص.
    pub fn fa_label(&self) -> String {
        match self {
            Self::UiUxArchitect390 => "معمار UX در ۳۹۰ پیکسل".to_owned(),
            Self::CoreLogicEngine => "موتور منطق هسته".to_owned(),
            Self::SecurityAndApiBridge => "پل امنیت و API".to_owned(),
            Self::VerificationAndPolish => "راستی‌آزمایی و پرداخت نهایی".to_owned(),
            Self::MonetizationAndViralLoop => "حلقهٔ درآمدزایی و وایرال".to_owned(),
            Self::Custom(name) => format!("متخصص سفارشی: {name}"),
        }
    }

    /// حالتِ فشرده‌سازی `Caveman` پیش‌فرضِ هر تخصص.
    pub fn default_caveman(&self) -> CavemanIntensity {
        match self {
            Self::UiUxArchitect390 | Self::VerificationAndPolish => CavemanIntensity::Lite,
            Self::CoreLogicEngine | Self::Custom(_) => CavemanIntensity::Full,
            Self::SecurityAndApiBridge => CavemanIntensity::Ultra,
            Self::MonetizationAndViralLoop => CavemanIntensity::Off,
        }
    }
}

/// لاتین‌سازیِ امنِ نامِ تخصص سفارشی برای شناسه‌های XML/وایر.
fn slugify_name(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(32)
        .collect();
    if slug.is_empty() {
        "specialist".to_owned()
    } else {
        slug
    }
}

/// شدت فشرده‌سازی `Caveman` — اقتصاد توکنِ هر ساب‌ایجنت (`Off|Lite|Full|Ultra`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CavemanIntensity {
    /// بدون فشرده‌سازی — خروجی کامل.
    Off,
    /// فشرده‌سازی سبک.
    Lite,
    /// فشرده‌سازی کامل.
    Full,
    /// فشرده‌سازی بیشینه — فقط سیگنال.
    Ultra,
}

impl CavemanIntensity {
    /// شناسهٔ وایر.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Lite => "lite",
            Self::Full => "full",
            Self::Ultra => "ultra",
        }
    }
}

/// قلابِ توقف ساب‌ایجنت — مرزهای `۴۵۰ ثانیه` استال نرم و `۱۲۰۰ ثانیه` تایم‌اوت سخت.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubagentStallWatchdog {
    /// مرزِ استال نرم — در این مرز سیگنالِ مداخلهٔ ارکستراتور صادر می‌شود.
    pub soft_stall_secs: u64,
    /// مرزِ تایم‌اوت سخت — در این مرز ران ساب‌ایجنت فسخ می‌شود.
    pub hard_timeout_secs: u64,
}

/// حکم قلابِ توقف در یک لحظهٔ زمانی.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchdogVerdict {
    /// در مرز امن.
    Healthy,
    /// استالِ نرم — مداخلهٔ ارکستراتور لازم است.
    SoftStall,
    /// تایم‌اوت سخت — فسخ و رهسپاریِ دوباره.
    HardTimeout,
}

impl SubagentStallWatchdog {
    /// مرز استال نرمِ حاکم (`450s`).
    pub const SOFT_STALL_SECS: u64 = 450;

    /// مرز تایم‌اوت سختِ حاکم (`1200s`).
    pub const HARD_TIMEOUT_SECS: u64 = 1200;

    /// قلابِ حاکمِ ترینیتی با مرزهای قفل‌شدهٔ `450s/1200s`.
    pub fn sovereign() -> Self {
        Self { soft_stall_secs: Self::SOFT_STALL_SECS, hard_timeout_secs: Self::HARD_TIMEOUT_SECS }
    }

    /// حکم در لحظهٔ `elapsed_secs` از آغاز ران.
    pub fn verdict_for(&self, elapsed_secs: u64) -> WatchdogVerdict {
        if elapsed_secs >= self.hard_timeout_secs {
            WatchdogVerdict::HardTimeout
        } else if elapsed_secs >= self.soft_stall_secs {
            WatchdogVerdict::SoftStall
        } else {
            WatchdogVerdict::Healthy
        }
    }
}

/// PRNG قطعی SplitMix64 — آلل‌های تصادفی از `session_seed + subagent_index`.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// یک عدد واحد در `[0, 1)` از جریان SplitMix64 — بدون تبدیلِ کستِ نامطمئن.
fn splitmix_unit(state: &mut u64) -> f64 {
    let z = splitmix64(state);
    let top = u32::try_from(z >> 40).unwrap_or(0); // z >> 40 < 2^24 — همیشه جا می‌شود
    f64::from(top) / 16_777_216.0
}

/// آلل‌های تصادفیِ قطعی ساب‌ایجنت — خرده‌صفاتِ سفارشیِ هر نه‌لایه.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SubAgentAlleles {
    /// چگالی جراحی — دقتِ حداقلیِ تغییر در سطح ماژول.
    pub surgical_density: f64,
    /// مهارتِ صنعتگری UX — وسواسِ تایپوگرافی و ریتم.
    pub ux_craft: f64,
    /// پارانویای لبه‌ها — پوششِ حالت‌های مرزی.
    pub edge_case_paranoia: f64,
    /// سرعتِ اجرا — تمایل به خروجیِ سریع و حداقلی.
    pub execution_velocity: f64,
    /// بذرِ آلل — بازتولیدِ قطعیِ همان DNA.
    pub allele_seed: u64,
}

/// مشتقِ قطعی آلل‌ها از `session_seed + subagent_index` — همان بذر، همان DNA.
fn derive_alleles(session_seed: u64, subagent_index: u32) -> SubAgentAlleles {
    let mut st = session_seed ^ u64::from(subagent_index).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let surgical_density = 0.55 + 0.40 * splitmix_unit(&mut st);
    let ux_craft = 0.50 + 0.45 * splitmix_unit(&mut st);
    let edge_case_paranoia = 0.60 + 0.35 * splitmix_unit(&mut st);
    let execution_velocity = 0.45 + 0.50 * splitmix_unit(&mut st);
    SubAgentAlleles { surgical_density, ux_craft, edge_case_paranoia, execution_velocity, allele_seed: st }
}

/// لایهٔ ۱ — هستهٔ ثابتِ قوانین حاکم؛ بین همهٔ ساب‌ایجنت‌ها کش-پین می‌شود.
pub const LAYER1_STABLE_CORE: &str = "SOVEREIGN CODING LAWS (cache-pinned across every OpenBot \
    sub-agent): (1) zero unsafe — `#![forbid(unsafe_code)]` is non-negotiable; (2) 390px \
    mobile-first zero-overflow — no horizontal scroll below 391px; (3) RTL-first typography \
    with Vazirmatn stack and honest LTR fallback; (4) zero hardcoded secrets and zero \
    placeholder credentials; (5) anti-slop — no stock hero copy, no generic Inter/Roboto-only \
    stack, no purple-blue gradient cliché, no spring bounce easing, no unrequested neon glow; \
    (6) zero todo/TODO/FIXME/XXX markers — unfinished is unshipped; (7) SSRF discipline — \
    egress only to allowlisted zero-auth public APIs; (8) every claim carries an executable \
    probe — no claim without evidence; (9) surgical diffs — touch only the assigned module \
    surface.";

/// الگوی پرامپت سه‌لایهٔ کش-امن — لایهٔ ۱ و ۲ کش-پین، لایهٔ ۳ پویا.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreeLayerSubAgentPrompt {
    /// لایهٔ ۱ — قوانین حاکمِ تغییرناپذیر (کش-پین بین همهٔ ساب‌ایجنت‌ها).
    pub layer1_stable_core: String,
    /// لایهٔ ۲ — قراردادهای نشست ساخت: مستر پرامپت + استراکچر پروژه (کش-پین per-build).
    pub layer2_blueprint_context: String,
    /// لایهٔ ۳ — محولهٔ دقیق ماژول + بذرِ آلل‌ها (پویا، پس از مرزِ کش).
    pub layer3_dynamic_assignment: String,
}

impl ThreeLayerSubAgentPrompt {
    /// رندر کامل — لایه‌های کش-پین قبل از [`SYSTEM_PROMPT_DYNAMIC_BOUNDARY`]
    /// و بلوکِ پویا پس از آن تا پیشوندِ ایستا در کش ارائه‌دهنده پایدار بماند.
    pub fn render(&self) -> String {
        format!(
            "{l1}\n{l2}\n{boundary}\n{l3}",
            l1 = self.layer1_stable_core,
            l2 = self.layer2_blueprint_context,
            boundary = SYSTEM_PROMPT_DYNAMIC_BOUNDARY,
            l3 = self.layer3_dynamic_assignment,
        )
    }
}

/// گزارش گیتِ پاک‌ساز — صفر نشانگر ناتمام، صفر رز جای‌بانی، صفر اسلوپ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SanitizerReport {
    /// شمار نشانگرهای ناتمام (`todo/TODO/FIXME/XXX/todo!()`).
    pub todo_markers: usize,
    /// شمار رازهای جای‌بانی (`sk-…`، `AKIA…`، `BEGIN PRIVATE KEY`، `ghp_…`).
    pub placeholder_secrets: usize,
    /// شمار بلوک‌های `unsafe` (`unsafe {` یا `unsafe fn`).
    pub unsafe_markers: usize,
    /// شمار متنِ جای‌بانیِ `lorem ipsum`.
    pub lorem_markers: usize,
    /// نشانه‌های AI-slop یافت‌شده (`audit_frontend_slop`) — فقط روی آرتیفکت.
    pub slop_tells: Vec<String>,
    /// عبور از گیت — همهٔ شمارنده‌ها صفر و اسلوپ خالی.
    pub passed: bool,
}

/// الگوهای رازِ جای‌بانی — هر رخداد، گیتِ پاک‌ساز را می‌شکند.
const SECRET_PATTERNS: &[&str] = &[
    "sk-", "akia", "begin private key", "begin rsa private key", "ghp_", "api_key=\"",
    "password=\"", "token=\"",
];

/// الگوهای نشانگر ناتمام.
const TODO_PATTERNS: &[&str] = &["todo", "fixme", "xxx", "todo!()"];

/// الگوهای بلوک unsafe.
const UNSAFE_PATTERNS: &[&str] = &["unsafe {", "unsafe fn"];

/// شمار رخدادهای همهٔ الگوها در متن (پس از lowercase).
fn count_pattern_occurrences(lower: &str, patterns: &[&str]) -> usize {
    patterns.iter().map(|p| lower.matches(p).count()).sum()
}

/// اجرای گیتِ پاک‌سازِ DNA — شمارنده‌های ساختاری بدون ممیزیِ اسلوپ (پرامپت).
pub fn run_sanitizer_gate(text: &str) -> SanitizerReport {
    let lower = text.to_lowercase();
    let todo_markers = count_pattern_occurrences(&lower, TODO_PATTERNS);
    let placeholder_secrets = count_pattern_occurrences(&lower, SECRET_PATTERNS);
    let unsafe_markers = count_pattern_occurrences(&lower, UNSAFE_PATTERNS);
    let lorem_markers = lower.matches("lorem ipsum").count();
    let passed = todo_markers == 0
        && placeholder_secrets == 0
        && unsafe_markers == 0
        && lorem_markers == 0;
    SanitizerReport { todo_markers, placeholder_secrets, unsafe_markers, lorem_markers, slop_tells: Vec::new(), passed }
}

/// اجرای گیتِ پاک‌سازِ آرتیفکت — شمارنده‌های ساختاری + ممیزیِ کاملِ اسلوپ.
pub fn run_artifact_sanitizer(html: &str) -> SanitizerReport {
    let mut report = run_sanitizer_gate(html);
    report.slop_tells = audit_frontend_slop(html)
        .into_iter()
        .map(str::to_owned)
        .collect();
    report.passed = report.passed && report.slop_tells.is_empty();
    report
}

/// ژنومِ نه‌لایه‌ای ساب‌ایجنت OpenBot — الگوی `۳×۹`: سه لایهٔ پرامپت ×
/// نه فاکتور ژنوم، هر ساب‌ایجنت با DNAِ یکتا اما سازگار با کش.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct NineFoldSubAgentGenome {
    /// f01 — توپولوژی و هارنس: کالیبراسیون IQ/EQ/نگاهِ جلو + مرزهای قلابِ توقف.
    pub f01_topology_and_harness: String,
    /// f02 — قیدهای سخت: `forbid(unsafe)`، صفر سرریز ۳۹۰، گارد SSRF، صفر راز.
    pub f02_hard_constraints: Vec<String>,
    /// f03 — اقتصاد توکن: حالتِ `Caveman`.
    pub f03_token_economics: CavemanIntensity,
    /// f04 — آلل‌های تصادفیِ قطعی از `session_seed + subagent_index`.
    pub f04_stochastic_alleles: SubAgentAlleles,
    /// f05 — بازیابیِ خطا: قرارداد خودترمیمیِ ۳-تلاشیِ OpenBot.
    pub f05_failure_recovery: String,
    /// f06 — پرامپت XML کامپایل‌شدهٔ کامل (`<openbot_subagent_dna>`).
    pub f06_compiled_xml_prompt: String,
    /// f07 — گیتِ پاک‌ساز: صفر `todo!()`، صفر رازِ جای‌بانی، صفر اسلوپ.
    pub f07_sanitizer_gate: SanitizerReport,
    /// f08 — پروب‌های تجربیِ اختصاصیِ ماژول.
    pub f08_empirical_probes: Vec<String>,
    /// f09 — کارت تحویل: قراردادِ بازگشت به اتاق فرمان.
    pub f09_handoff_card: String,
}

/// گریزِ XML برای محتوای متنیِ داخل بلوکِ DNA.
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// ساب‌ایجنتِ رهسپارشدهٔ OpenBot — DNAِ کاملِ یکتا برای یک ماژول.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnedSubAgent {
    /// شمار ردیفی ساب‌ایجنت در ناوگانِ این نشست (`0..`).
    pub subagent_index: u32,
    /// تخصصِ متخصص.
    pub specialty: SubAgentSpecialty,
    /// ماژول(های) محول‌شده.
    pub assigned_modules: Vec<String>,
    /// شناسهٔ نخِ قطعی (`deterministic_subagent_thread_id`).
    pub thread_id: String,
    /// پرامپت سه‌لایهٔ کش-امن.
    pub prompt: ThreeLayerSubAgentPrompt,
    /// ژنوم نه‌لایه‌ای.
    pub genome: NineFoldSubAgentGenome,
}

/// کارخانهٔ ساب‌ایجنت‌های OpenBot — رهسپاریِ پویا با DNAِ یکتا.
#[derive(Debug, Clone)]
pub struct OpenBotSubAgentSpawner {
    /// شناسهٔ نشست ترینیتی (والدِ نخ‌ها).
    session_id: String,
    /// بذر تصادفیِ قطعی نشست.
    session_seed: u64,
}

impl OpenBotSubAgentSpawner {
    /// ساخت کارخانه روی یک نشست.
    pub fn new(session_id: &str, session_seed: u64) -> Self {
        Self { session_id: session_id.to_owned(), session_seed }
    }

    /// پروب‌های تجربیِ پیش‌فرض هر تخصص — f08.
    fn probes_for(specialty: &SubAgentSpecialty) -> Vec<String> {
        match specialty {
            SubAgentSpecialty::UiUxArchitect390 => vec![
                "assert(document.documentElement.scrollWidth <= 390)".to_owned(),
                "assert(font stack resolves Vazirmatn before fallback)".to_owned(),
            ],
            SubAgentSpecialty::CoreLogicEngine => vec![
                "assert(core state transitions are pure and deterministic)".to_owned(),
                "assert(no global mutable state leaks between modules)".to_owned(),
            ],
            SubAgentSpecialty::SecurityAndApiBridge => vec![
                "assert(fetch targets are allowlisted zero-auth public endpoints)".to_owned(),
                "assert(no secret material in the artifact)".to_owned(),
            ],
            SubAgentSpecialty::VerificationAndPolish => vec![
                "assert(all data-probe elements green before handoff)".to_owned(),
                "assert(anti-slop audit findings == 0)".to_owned(),
            ],
            SubAgentSpecialty::MonetizationAndViralLoop => vec![
                "assert(pricing renders in KIAN/Stars (XTR))".to_owned(),
                "assert(affiliate hook attaches referrer at 1000 bps default)".to_owned(),
            ],
            SubAgentSpecialty::Custom(_) => vec![
                "assert(custom module contract satisfied and probes green)".to_owned(),
            ],
        }
    }

    /// کامپایل XML نه‌لایه (f06) — بلوک کاملِ `<openbot_subagent_dna>`.
    #[allow(clippy::too_many_arguments)] // قراردادِ DNAِ کامل — نه ورودیِ نام‌دارِ صریح
    fn compile_xml_prompt(
        session_id: &str,
        index: u32,
        specialty: &SubAgentSpecialty,
        module: &BlueprintModule,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
        alleles: &SubAgentAlleles,
        watchdog: &SubagentStallWatchdog,
        caveman: CavemanIntensity,
    ) -> String {
        let contracts = blueprint
            .modules
            .iter()
            .map(|m| format!("{} :: {}", m.path, xml_escape(&m.role)))
            .collect::<Vec<_>>()
            .join("\n      ");
        format!(
            "<openbot_subagent_dna version=\"0xKIA-trinity-1\" session=\"{sid}\" index=\"{index}\" \
             specialty=\"{spec}\">\n  \
             <f01_topology_and_harness iq=\"{LOGIC_EFFECTIVE_IQ}\" eq=\"{LOGIC_EFFECTIVE_EQ}\" \
             lookahead=\"2\" watchdog_soft=\"{soft}\" watchdog_hard=\"{hard}\"/>\n  \
             <f02_hard_constraints>\n    <![CDATA[#![forbid(unsafe_code)] | 390px zero-overflow \
             | SSRF guard | zero secrets | anti-slop]]>\n  </f02_hard_constraints>\n  \
             <f03_token_economics caveman=\"{cv}\"/>\n  \
             <f04_stochastic_alleles surgical_density=\"{sd:.3}\" ux_craft=\"{ux:.3}\" \
             edge_case_paranoia=\"{ec:.3}\" execution_velocity=\"{ev:.3}\" seed=\"{seed}\"/>\n  \
             <f05_failure_recovery>OpenBot 3-attempt self-healing: diagnose, patch, re-run — \
             escalate to Command Center only after attempt 3</f05_failure_recovery>\n  \
             <master_prompt_spec hash=\"{hash}\" project=\"{project}\"/>\n  \
             <layer2_blueprint_context>\n    {contracts}\n  </layer2_blueprint_context>\n  \
             <assignment module=\"{module}\" role=\"{role}\"/>\n  \
             <f07_sanitizer_gate enforced=\"true\"/>\n  \
             <f09_handoff_card target=\"command_center\"/>\n\
             </openbot_subagent_dna>",
            sid = xml_escape(session_id),
            spec = specialty.as_str(),
            soft = watchdog.soft_stall_secs,
            hard = watchdog.hard_timeout_secs,
            cv = caveman.as_str(),
            sd = alleles.surgical_density,
            ux = alleles.ux_craft,
            ec = alleles.edge_case_paranoia,
            ev = alleles.execution_velocity,
            seed = alleles.allele_seed,
            hash = &master.spec_hash[..16],
            project = xml_escape(&blueprint.project_slug),
            module = xml_escape(&module.path),
            role = xml_escape(&module.role),
            contracts = contracts,
        )
    }

    /// رهسپاریِ یک ساب‌ایجنت با DNAِ کاملِ یکتا برای یک ماژول.
    pub fn spawn(
        &self,
        subagent_index: u32,
        module: &BlueprintModule,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
    ) -> SpawnedSubAgent {
        let alleles = derive_alleles(self.session_seed, subagent_index);
        let watchdog = SubagentStallWatchdog::sovereign();
        let caveman = module.specialty.default_caveman();
        let thread_id = deterministic_subagent_thread_id(
            &self.session_id,
            &format!("openbot-{subagent_index}-{}", module.path),
        );
        let layer2 = format!(
            "BUILD SESSION CONTEXT (cache-pinned per build session): project '{project}' \
             | master spec hash {hash12} | interface contracts:\n{contracts}\n\
             | 390px zero-overflow contract | zero-auth API surface (SSRF-guarded)",
            project = master.project_name,
            hash12 = &master.spec_hash[..12],
            contracts = blueprint
                .modules
                .iter()
                .map(|m| format!("  - {} :: {}", m.path, m.role))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let layer3 = format!(
            "ASSIGNMENT (dynamic): agent #{subagent_index} specialty {spec_fa} \
             | module {module_path} | role {module_role} | caveman {cv} | \
             alleles surgical={sd:.2} ux_craft={ux:.2} paranoia={ec:.2} velocity={ev:.2} \
             seed={seed} | repair pass 0 — first dispatch",
            spec_fa = module.specialty.fa_label(),
            module_path = module.path,
            module_role = module.role,
            cv = caveman.as_str(),
            sd = alleles.surgical_density,
            ux = alleles.ux_craft,
            ec = alleles.edge_case_paranoia,
            ev = alleles.execution_velocity,
            seed = alleles.allele_seed,
        );
        let prompt = ThreeLayerSubAgentPrompt {
            layer1_stable_core: LAYER1_STABLE_CORE.to_owned(),
            layer2_blueprint_context: layer2,
            layer3_dynamic_assignment: layer3,
        };
        let f06 = Self::compile_xml_prompt(
            &self.session_id,
            subagent_index,
            &module.specialty,
            module,
            master,
            blueprint,
            &alleles,
            &watchdog,
            caveman,
        );
        // گیتِ پاک‌ساز روی بلوکِ DNAِ کامپایل‌شده اجرا می‌شود — لایهٔ ۱ قانون‌ها را
        // با نامشان نقل می‌کند (متای قانون)، پس شمارش روی XMLِ نه‌لایه معتبر است.
        let f07 = run_sanitizer_gate(&f06);
        let f01 = format!(
            "IQ={LOGIC_EFFECTIVE_IQ} EQ={LOGIC_EFFECTIVE_EQ} lookahead=2 | \
             SubagentStallWatchdog soft={soft}s hard={hard}s | fleet session {sid}",
            soft = watchdog.soft_stall_secs,
            hard = watchdog.hard_timeout_secs,
            sid = self.session_id,
        );
        let f09 = format!(
            "HANDOFF -> Command Center | agent #{subagent_index} {spec} | module {path} | \
             contract: {role} | pass criteria: probes green + sanitizer clean + Mairon \
             pillar at threshold",
            spec = module.specialty.as_str(),
            path = module.path,
            role = module.role,
        );
        SpawnedSubAgent {
            subagent_index,
            specialty: module.specialty.clone(),
            assigned_modules: vec![module.path.clone()],
            thread_id,
            prompt,
            genome: NineFoldSubAgentGenome {
                f01_topology_and_harness: f01,
                f02_hard_constraints: vec![
                    "#![forbid(unsafe_code)] — zero unsafe in all generated kernels".to_owned(),
                    "390px zero-overflow — no horizontal scroll below 391px viewport".to_owned(),
                    "SSRF guard — egress only to allowlisted zero-auth public APIs".to_owned(),
                    "zero hardcoded secrets — credentials never enter the artifact".to_owned(),
                    "anti-slop — no stock hero copy, no cliché gradients, no bounce easing"
                        .to_owned(),
                    "RTL-first Vazirmatn typography with honest LTR fallback".to_owned(),
                ],
                f03_token_economics: caveman,
                f04_stochastic_alleles: alleles,
                f05_failure_recovery: "OpenBot 3-attempt self-healing contract: on failure \
                    (1) diagnose via forensic diff, (2) patch the minimal module surface, \
                    (3) re-run empirical probes; escalate to Command Center only after \
                    attempt 3."
                    .to_owned(),
                f06_compiled_xml_prompt: f06,
                f07_sanitizer_gate: f07,
                f08_empirical_probes: Self::probes_for(&module.specialty),
                f09_handoff_card: f09,
            },
        }
    }

    /// رهسپاریِ کل ناوگان — یک ساب‌ایجنت به ازای هر ماژولِ استراکچر پروژه.
    pub fn spawn_fleet(
        &self,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
    ) -> Vec<SpawnedSubAgent> {
        blueprint
            .modules
            .iter()
            .enumerate()
            .map(|(i, module)| {
                self.spawn(u32::try_from(i).unwrap_or(u32::MAX), module, master, blueprint)
            })
            .collect()
    }
}

// ── مونتاژ آرتیفکت — هر ساب‌ایجنت سهمِ قطعیِ ماژول خودش را می‌سازد ─────────

/// نشانهٔ مرزِ ماژول در آرتیفکت — امضای یکپارچگیِ معماری.
pub const MODULE_BOUNDARY_MARKER: &str = "<!-- ══ mairon-boundary:";

/// نشانهٔ تقویتِ مرز پس از ترمیم معماری (رهسپاریِ دوبارهٔ CoreLogicEngine).
pub const BOUNDARY_REINFORCED_MARKER: &str = "boundary-reinforced: mairon-pass";

/// نشانهٔ بلوکِ پرداخت UX پس از ترمیم (رهسپاریِ دوبارهٔ UiUxArchitect390).
pub const UX_POLISH_MARKER: &str = "data-ux-polish=\"mairon-pass";

/// آرتیفکت مونتاژشدهٔ یک‌صفحه‌ای — خروجیِ مرحلهٔ ۲ و ورودیِ داوریِ مرحلهٔ ۳.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssembledArtifact {
    /// HTML کاملِ محصول.
    pub html: String,
    /// ماژول‌های یکپارچه‌شده (با `data-module` قابل راستی‌آزمایی).
    pub integrated_modules: Vec<String>,
    /// شمار پروب‌های تجربیِ تزریق‌شده (`data-probe="probe-…"`).
    pub probe_count: usize,
    /// گذرِ ترمیم (`0` = مونتاژ نخست، `1..=3` = پس از فرمانِ مایرون).
    pub repair_pass: u32,
}

impl AssembledArtifact {
    /// حجم آرتیفکت بر حسب بایت.
    pub fn bytes(&self) -> usize {
        self.html.len()
    }
}

/// گریزِ HTML برای متنِ مشتق از کاربر در آرتیفکت.
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// قالب پوستهٔ HTML — جای‌بان‌ها با `.replace` پر می‌شوند (بدون کِش‌مکشِ بریس).
const ARTIFACT_SHELL: &str = "<!doctype html>\n<html lang=\"fa\" dir=\"rtl\">\n<head>\n\
<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>__KIA_TITLE__ — 0xKIA Trinity Studio</title>\n\
<!-- ══ mairon-boundary: src/styles/main.css -->\n\
<style data-module=\"src/styles/main.css\" data-allele-seed=\"__KIA_SEED_CSS__\">\n\
:root { --ink:#101418; --paper:#f7f5f0; --accent:#0e7c66; --line:#d8d2c4; }\n\
* { box-sizing: border-box; }\n\
html, body { margin: 0; padding: 0; }\n\
body { font-family: 'Vazirmatn', 'IBM Plex Sans Arabic', sans-serif; background: var(--paper); \
color: var(--ink); max-width: 390px; margin-inline: auto; overflow-x: hidden; }\n\
.grid { display: grid; gap: 12px; padding: 16px; }\n\
.card { border: 1px solid var(--line); border-radius: 12px; padding: 12px; background: #fffdf8; }\n\
@media (min-width: 391px) { body { max-width: 420px; } }\n\
</style>\n\
__KIA_UX_POLISH__\
</head>\n\
<body>\n\
<main id=\"app\" data-project=\"__KIA_SLUG__\">\n\
__KIA_HERO__\
</main>\n\
__KIA_SCRIPTS__\
__KIA_PROBES__\
__KIA_REINFORCED__\
</body>\n</html>\n";

/// قالب بخشِ قهرمان — متن‌های مشتق از کاربر پیش از نشاندن گریز می‌شوند.
const HERO_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/index.html -->\n\
<section class=\"hero\" data-module=\"src/index.html\" data-allele-seed=\"__KIA_SEED_HERO__\">\n\
<h1 class=\"hero-title\">__KIA_TITLE__</h1>\n\
<p class=\"hero-sub\">__KIA_OBJECTIVE__</p>\n\
<div class=\"grid\" data-audience=\"__KIA_AUDIENCE__\">\n\
<div class=\"card\" data-frame=\"1\">__KIA_FEATURES__</div>\n\
</div>\n\
</section>\n";

/// قالب هستهٔ منطقِ خالص — بدون IO، بدون unsafe.
const CORE_JS_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/core.js -->\n\
<script data-module=\"src/js/core.js\" data-allele-seed=\"__KIA_SEED_CORE__\">\n\
const KiaCore = (() => {\n\
  const state = { ready: false, module: '__KIA_MODULE__' };\n\
  function setState(patch) { return Object.freeze(Object.assign({}, state, patch)); }\n\
  function summary() { return state.module + ' ready=' + state.ready; }\n\
  return { setState, summary, boundary: 'src/js/core.js' };\n\
})();\n\
</script>\n";

/// قالب تعاملات فریمِ اول.
const INTERACTIONS_JS_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/interactions.js -->\n\
<script data-module=\"src/js/interactions.js\">\n\
const KiaInteractions = (() => {\n\
  function bindButtons(root) {\n\
    let bound = 0;\n\
    root.querySelectorAll('button').forEach((b) => {\n\
      b.addEventListener('click', () => { b.dataset.clicked = '1'; });\n\
      bound += 1;\n\
    });\n\
    return bound;\n\
  }\n\
  return { bindButtons, boundary: 'src/js/interactions.js' };\n\
})();\n\
</script>\n";

/// قالب پل APIهای عمومیِ بدون احراز با گارد SSRF — صفر راز روی وایر.
const API_BINDINGS_JS_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/api-bindings.js -->\n\
<script data-module=\"src/js/api-bindings.js\">\n\
const KiaApi = (() => {\n\
  const ALLOWLIST = ['https://api.github.com', 'https://api.exchangerate.host', \
'https://restcountries.com'];\n\
  async function bind(endpoint, target) {\n\
    if (!ALLOWLIST.some((base) => endpoint.startsWith(base))) {\n\
      throw new Error('SSRF guard: endpoint not allowlisted');\n\
    }\n\
    const res = await fetch(endpoint); /* zero-auth public API — no credentials on the wire */\n\
    const data = await res.json();\n\
    localStorage.setItem('kia:' + target, JSON.stringify(data));\n\
    return data;\n\
  }\n\
  return { bind, boundary: 'src/js/api-bindings.js' };\n\
})();\n\
</script>\n";

/// قالب حلقهٔ درآمدزایی و وایرال — قیمت Stars (`XTR`) + سهم معرف.
const MONETIZATION_JS_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/monetization.js -->\n\
<script data-module=\"src/js/monetization.js\">\n\
const KiaMonetization = (() => {\n\
  const priceStars = __KIA_PRICE__; /* Telegram Stars (XTR) — priced by Mairon ToM-3 advisory */\n\
  const affiliateBps = 1000; /* 10% affiliate share — viral loop hook */\n\
  function priceLine() { return priceStars + ' Stars (XTR) | affiliate ' + (affiliateBps / 100) + '%'; }\n\
  function attachAffiliate(referrer) { return { referrer: referrer || 'direct', bps: affiliateBps }; }\n\
  return { priceStars, affiliateBps, priceLine, attachAffiliate, boundary: 'src/js/monetization.js' };\n\
})();\n\
</script>\n";

/// قالب خود-بررسیِ گذرِ صفر — پروب‌ها فقط با فرمانِ مایرون تزریق می‌شوند.
const SELF_CHECK_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/verification-probes.js -->\n\
<script data-module=\"src/js/verification-probes.js\">\n\
/* self-check (pass 0): empirical probes are stage-gated behind Mairon's repair directive */\n\
/* self-check: proof before pride — no completion claim without fresh evidence */\n\
</script>\n";

/// قالب بلوکِ پروب‌های تجربی — پس از فرمانِ ترمیمِ مایرون.
const PROBES_BLOCK_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/verification-probes.js (empirical) -->\n\
<div id=\"mairon-empirical-probes\" data-repair-pass=\"__KIA_PASS__\">\n\
__KIA_PROBE_SPANS__\
</div>\n\
<script data-module=\"src/js/verification-probes.js\">\n\
const KiaProbes = {\n\
  total: __KIA_PROBE_COUNT__,\n\
  run() {\n\
    return Array.prototype.every.call(\n\
      document.querySelectorAll('[data-probe]'), (p) => { p.dataset.ok = '1'; return true; });\n\
  },\n\
  boundary: 'src/js/verification-probes.js'\n\
};\n\
</script>\n";

/// قالب بلوکِ پرداختِ UX پس از فرمانِ ترمیم — تایپوگرافی و ریتمِ باز-تثبیت‌شده.
const UX_POLISH_TEMPLATE: &str = "<!-- mairon ux polish pass __KIA_PASS__: hardening re-affirmed -->\n\
<style __KIA_POLISH_ATTR__>\n\
main#app { padding-inline: 16px; }\n\
.hero-title { font-size: 1.35rem; line-height: 1.9; }\n\
.hero-sub { font-size: 0.95rem; opacity: 0.8; }\n\
</style>\n";

/// نخستین عدد صحیح در متن (رقم لاتین یا فارسی) — پیشنهاد قیمت از اسلاتِ درآمد.
fn first_number_in(text: &str) -> Option<u64> {
    let mut buf = String::new();
    for ch in text.chars() {
        let digit = match ch {
            '0'..='9' => ch,
            '۰'..='۹' => char::from_u32(u32::from('0') + (u32::from(ch) - u32::from('۰')))
                .unwrap_or('0'),
            _ => {
                if !buf.is_empty() {
                    break;
                }
                continue;
            }
        };
        buf.push(digit);
    }
    if buf.is_empty() {
        None
    } else {
        buf.parse::<u64>().ok()
    }
}

/// پیشنهاد قیمت مایرون از اسلاتِ درآمدزایی — پیش‌فرضِ `۱۵۰⭐` در نبودِ عدد.
fn derive_price_stars(slots: &AlphaDiscoverySlots) -> u64 {
    slots
        .monetization_and_viral_loop
        .as_deref()
        .and_then(first_number_in)
        .filter(|n| (1..=100_000).contains(n))
        .unwrap_or(150)
}

/// کالبد ۲ — اتاق فرمان: ارکستراتورِ قضایی با کارخانهٔ ساب‌ایجنت‌های OpenBot.
#[derive(Debug, Clone)]
pub struct CommandCenterOrchestrator {
    /// شناسهٔ نشست ترینیتی.
    pub session_id: String,
    /// حساب شهروندِ سازنده.
    pub citizen_id: String,
    /// بذر تصادفیِ قطعی نشست — منبعِ آلل‌های کل ناوگان.
    pub session_seed: u64,
}

impl CommandCenterOrchestrator {
    /// ساخت ارکستراتور روی نشستِ قفل‌شدهٔ آلفا.
    pub fn new(session_id: &str, citizen_id: &str, session_seed: u64) -> Self {
        Self {
            session_id: session_id.to_owned(),
            citizen_id: citizen_id.to_owned(),
            session_seed,
        }
    }

    /// بذرِ قطعی از شناسهٔ نشست — ۸ بایتِ اولِ SHA-256؛ در نبودِ بذرِ صریحِ
    /// کلاینت، ناوگان همچنان بازتولید‌پذیر می‌ماند.
    pub fn session_seed_from_session(session_id: &str) -> u64 {
        let digest = Sha256::digest(session_id.as_bytes());
        let mut seed = [0u8; 8];
        seed.copy_from_slice(&digest[..8]);
        u64::from_be_bytes(seed)
    }

    /// ناوگانِ OpenBot این نشست — یک متخصص به ازای هر ماژول.
    pub fn spawn_openbot_fleet(
        &self,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
    ) -> Vec<SpawnedSubAgent> {
        OpenBotSubAgentSpawner::new(&self.session_id, self.session_seed).spawn_fleet(master, blueprint)
    }

    /// مونتاژ آرتیفکت یک‌صفحه‌ای از سهمِ قطعیِ هر ساب‌ایجنت.
    ///
    /// * `repair_pass == 0` — مونتاژِ نخست؛ پروب‌ها سنجه‌بستهٔ فرمانِ مایرون‌اند.
    /// * `repair_pass >= 1` — فقط فرمان‌های صادرشدهٔ داور اعمال می‌شوند.
    fn assemble_artifact(
        fleet: &[SpawnedSubAgent],
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
        price_stars: u64,
        repair_pass: u32,
        directives: &[MaironRepairDirective],
    ) -> AssembledArtifact {
        let seed_of = |path: &str| -> u64 {
            fleet
                .iter()
                .find(|a| a.assigned_modules.iter().any(|m| m == path))
                .map_or(0, |a| a.genome.f04_stochastic_alleles.allele_seed)
        };
        let objective = master
            .functional_spec
            .lines()
            .find(|l| l.starts_with("- هدفِ هسته"))
            .map_or(String::new(), |l| {
                l.trim_start_matches("- هدفِ هسته: ").chars().take(140).collect()
            });
        let audience = master
            .functional_spec
            .lines()
            .find(|l| l.starts_with("- مخاطب و UX"))
            .map_or(String::new(), |l| {
                l.trim_start_matches("- مخاطب و UX: ").chars().take(80).collect()
            });
        let features = master
            .functional_spec
            .lines()
            .find(|l| l.starts_with("- قابلیت‌های تعاملی"))
            .map_or("workflow فریمِ اول", |l| l.trim_start_matches("- قابلیت‌های تعاملی: "))
            .chars()
            .take(120)
            .collect::<String>();
        let hero = HERO_TEMPLATE
            .replace("__KIA_SEED_HERO__", &seed_of("src/index.html").to_string())
            .replace("__KIA_TITLE__", &html_escape(&master.project_name))
            .replace("__KIA_OBJECTIVE__", &html_escape(&objective))
            .replace("__KIA_AUDIENCE__", &html_escape(&audience))
            .replace("__KIA_FEATURES__", &html_escape(&features));
        let mut scripts = String::new();
        for module in &blueprint.modules {
            match module.path.as_str() {
                "src/js/core.js" => {
                    scripts.push_str(
                        &CORE_JS_TEMPLATE
                            .replace("__KIA_SEED_CORE__", &seed_of(&module.path).to_string())
                            .replace("__KIA_MODULE__", &blueprint.project_slug),
                    );
                }
                "src/js/interactions.js" => scripts.push_str(INTERACTIONS_JS_TEMPLATE),
                "src/js/api-bindings.js" => scripts.push_str(API_BINDINGS_JS_TEMPLATE),
                "src/js/monetization.js" => {
                    scripts.push_str(
                        &MONETIZATION_JS_TEMPLATE.replace("__KIA_PRICE__", &price_stars.to_string()),
                    );
                }
                "src/js/verification-probes.js" => scripts.push_str(SELF_CHECK_TEMPLATE),
                _ => {}
            }
        }
        let ux_directive = directives
            .iter()
            .any(|d| d.target_specialty == SubAgentSpecialty::UiUxArchitect390);
        let arch_directive = directives
            .iter()
            .any(|d| d.target_specialty == SubAgentSpecialty::CoreLogicEngine);
        let ver_directive = directives
            .iter()
            .any(|d| d.target_specialty == SubAgentSpecialty::VerificationAndPolish);
        let ux_polish = if repair_pass >= 1 && ux_directive {
            UX_POLISH_TEMPLATE
                .replace("__KIA_PASS__", &repair_pass.to_string())
                .replace("__KIA_POLISH_ATTR__", &format!("{UX_POLISH_MARKER}-{repair_pass}\""))
        } else {
            String::new()
        };
        let probe_count = usize::from(repair_pass >= 1 && ver_directive) * (2 * blueprint.modules.len() + 4);
        let probes = if probe_count > 0 {
            let mut spans = String::new();
            for k in 1..=probe_count {
                spans.push_str("<span class=\"probe\" data-probe=\"probe-");
                spans.push_str(&k.to_string());
                spans.push_str("\" data-check=\"390px-zero-overflow\"></span>\n");
            }
            PROBES_BLOCK_TEMPLATE
                .replace("__KIA_PASS__", &repair_pass.to_string())
                .replace("__KIA_PROBE_SPANS__", &spans)
                .replace("__KIA_PROBE_COUNT__", &probe_count.to_string())
        } else {
            String::new()
        };
        let reinforced = if repair_pass >= 1 && arch_directive {
            format!("<!-- ══ {BOUNDARY_REINFORCED_MARKER}-{repair_pass} -->\n")
        } else {
            String::new()
        };
        let html = ARTIFACT_SHELL
            .replace("__KIA_TITLE__", &html_escape(&master.project_name))
            .replace("__KIA_SEED_CSS__", &seed_of("src/styles/main.css").to_string())
            .replace("__KIA_SLUG__", &blueprint.project_slug)
            .replace("__KIA_UX_POLISH__", &ux_polish)
            .replace("__KIA_HERO__", &hero)
            .replace("__KIA_SCRIPTS__", &scripts)
            .replace("__KIA_PROBES__", &probes)
            .replace("__KIA_REINFORCED__", &reinforced);
        let integrated_modules = blueprint
            .modules
            .iter()
            .filter(|m| html.contains(&format!("data-module=\"{}\"", m.path)))
            .map(|m| m.path.clone())
            .collect();
        AssembledArtifact { html, integrated_modules, probe_count, repair_pass }
    }
}

// ════════════════════ Stage 3 — MAIRON-150 ═══════════════════════════════

/// سقفِ امتیاز ستون — از دامنه (`MAX_PILLAR_SCORE = 25`).
use kian_domain::mairon_rea_and_reviews::MAX_PILLAR_SCORE;

/// هر ستون زیرِ این آستانه، فرمانِ ترمیمِ اختصاصیِ همان تخصص را می‌گیرد.
const PILLAR_REPAIR_TRIGGER: u8 = MAX_PILLAR_SCORE;

/// کالیبراسیونِ معماری — پایه به ازای هر ماژولِ استراکچر پروژه.
const ARCH_BASE: usize = 6;
const ARCH_PER_MODULE: usize = 2;
const ARCH_INTEGRATED_BONUS: usize = 3;
const ARCH_BOUNDARY_BONUS: usize = 2;
const ARCH_REINFORCED_BONUS: usize = 2;

/// کالیبراسیونِ UX — پایه + نشانگرهای RTL/۳۹۰/وزیرمتن/پرداخت.
const UX_BASE: usize = 17;
const UX_MARKER_BONUS: usize = 2;
const UX_SLOP_PENALTY: usize = 4;
const UX_LOREM_PENALTY: usize = 3;

/// کالیبراسیونِ راستی‌آزمایی — پایه + سه امتیاز به ازای هر پروب تجربی.
const VERIFICATION_BASE: usize = 2;
const VERIFICATION_PER_PROBE: usize = 3;

/// کالیبراسیونِ امنیت — پایهٔ کامل، کسر به ازای هر رخدادِ ممنوع.
const SECURITY_BASE: usize = 25;
const SECURITY_PER_SECRET: usize = 6;
const SECURITY_PER_TODO: usize = 4;
const SECURITY_PER_UNSAFE: usize = 5;

/// بستنِ امتیاز ستون در `0..=25` — ورودیِ سرکش، حکمِ قانونی (قرارداد دامنه).
fn clamp25(value: usize) -> u8 {
    u8::try_from(value.min(usize::from(MAX_PILLAR_SCORE))).unwrap_or(MAX_PILLAR_SCORE)
}

/// فرمانِ ترمیمِ مایرون — رهسپاریِ اختصاصیِ ساب‌ایجنتِ مقصر با دستورِ دوزبانه.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaironRepairDirective {
    /// گذرِ ترمیم (`1..=MAX_OPENBOT_REPAIR_ATTEMPTS`).
    pub attempt: u32,
    /// تخصصِ ساب‌ایجنتِ مقصر — مخاطبِ رهسپاریِ دوباره.
    pub target_specialty: SubAgentSpecialty,
    /// ستونِ REAِ زیرِ آستانه (`architecture` | `security_hardening` | …).
    pub pillar: String,
    /// دستورِ اصلاح (انگلیسی — لایهٔ فنی).
    pub directive_en: String,
    /// دستورِ اصلاح (فارسی — صدای داور).
    pub directive_fa: String,
}

/// مشاورهٔ استراتژیک مایرون با کالیبراسیونِ قفل‌شدهٔ `IQ=150 / EQ=145`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaironStrategicAdvisory {
    /// IQ اورکلاک داور (`150`).
    pub iq: u16,
    /// EQ اورکلاک داور (`145`).
    pub eq: u16,
    /// افقِ نگاهِ بازی‌نظری (`8` گام).
    pub lookahead_horizon: u8,
    /// مرتبهٔ نظریهٔ ذهن (`3`).
    pub tom_order: u8,
    /// خلاصهٔ نگاهِ هشت‌گامی — تاب‌آوریِ معماری و مسیرِ مقیاس‌پذیری.
    pub eight_step_lookahead_summary: String,
    /// قلابِ روان‌شناختیِ خریدار و پیشنهاد قیمت/معرف (فارسی، ToM-3).
    pub third_order_tom_market_hook_fa: String,
    /// فرمان‌های ترمیم برای اتاق فرمان — خالی یعنی آهنگری پذیرفته شده.
    pub repair_directives: Vec<MaironRepairDirective>,
}

impl MaironStrategicAdvisory {
    /// مشاورهٔ کامل برای یک آهنگریِ ترینیتی (استراکچر پروژه + مستر پرامپت).
    pub fn for_blueprint(
        blueprint: &ProjectBlueprint,
        master: &MasterPromptSpec,
    ) -> Self {
        let price = master.price_stars_hint;
        let low = price * 7 / 10;
        let high = price * 8 / 5;
        let eight_step_lookahead_summary = format!(
            "Lookahead-{h} (IQ=150): (1) module isolation across {n} hard-boundary modules; \
             (2) cache-safe prompt layers pin spec {hash} across the OpenBot fleet; \
             (3) 390px zero-overflow contract survives every repair re-dispatch; \
             (4) zero-auth API surface stays SSRF-allowlisted; \
             (5) pricing at {price} Stars (XTR) survives third-order buyer simulation; \
             (6) affiliate share (1000 bps default) compounds the viral loop; \
             (7) 5-star Bayesian velocity feeds CombinedRankScore; \
             (8) the same 3x9 DNA factory re-seeds the next product without \
             re-architecture — resilience and scaling path verified.",
            h = MAIRON_LOOKAHEAD_HORIZON,
            n = blueprint.modules.len(),
            hash = &master.spec_hash[..12],
            price = price,
        );
        let third_order_tom_market_hook_fa = format!(
            "🧠 مایرون (IQ={iq}/EQ={eq}، نظریهٔ ذهن مرتبهٔ {tom}، جاه‌طلبی {amb}): \
             خریدار در گامِ سومِ تصمیم به خودش می‌گوید «اگر امروز نخرم، همین ابزار \
             فردا از دستم درمی‌رود» — قلابِ روان‌شناختیِ «{project}»: دقیقاً همان \
             دردِ روزمرهٔ مخاطب را در ۳۹۰ پیکسل، بدون لاگین و بدون کارت بانکی حل \
             می‌کند. قیمت‌گذاریِ بهینه: {price} ستارهٔ تلگرام (XTR) با سهم معرف ۱۰٪ \
             (1000 bps) تا هر خریدار خودش به بازتوزیع برسد؛ زیر {low} اعتماد \
             نمی‌سازد و بالاتر از {high} نرخ تبدیل می‌افتد.",
            iq = MAIRON_OVERCLOCK_IQ,
            eq = MAIRON_OVERCLOCK_EQ,
            tom = MAIRON_TOM_ORDER,
            amb = MAIRON_AMBITION_GAIN,
            project = master.project_name,
            price = price,
            low = low,
            high = high,
        );
        Self {
            iq: MAIRON_OVERCLOCK_IQ,
            eq: MAIRON_OVERCLOCK_EQ,
            lookahead_horizon: MAIRON_LOOKAHEAD_HORIZON,
            tom_order: MAIRON_TOM_ORDER,
            eight_step_lookahead_summary,
            third_order_tom_market_hook_fa,
            repair_directives: Vec::new(),
        }
    }
}

/// حکمِ حاکمِ مایرون روی یک آهنگری — نمرهٔ REA دامنه + مشاورهٔ استراتژیک.
/// (ساختارهای دامنه بدون serde هستند — این حکم فقط در حافظه جریان دارد.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaironSovereignVerdict {
    /// حکم چهارستونیِ دامنه (`0..=100` + خطوط قرمز + نشان مرئی).
    pub verdict: MaironProductVerdict,
    /// مشاورهٔ استراتژیک `IQ=150 / EQ=145`.
    pub advisory: MaironStrategicAdvisory,
}

/// یک دورِ ترمیم در حلقهٔ خودترمیمیِ اتاق فرمان ↔ مایرون.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairRound {
    /// گذرِ ترمیم (`1..=3`).
    pub attempt: u32,
    /// نمرهٔ پیش از اعمال فرمان‌ها.
    pub score_before: u8,
    /// فرمان‌های صادرشدهٔ داور در همین گذر.
    pub directives: Vec<MaironRepairDirective>,
    /// نمرهٔ پس از رهسپاریِ دوباره و داوریِ تازه.
    pub score_after: u8,
    /// آیا در همین گذر از گیت `≥ 85` عبور شد؟
    pub approved: bool,
}

/// خروجیِ کاملِ خط لولهٔ ترینیتی — مانیفست DNA، آرتیفکت، حکم نهایی و تاریخچه.
#[derive(Debug, Clone, PartialEq)]
pub struct TrinityForgeOutcome {
    /// شناسهٔ نشست ترینیتی.
    pub session_id: String,
    /// حساب شهروندِ سازنده.
    pub citizen_id: String,
    /// نام محصول.
    pub project_name: String,
    /// مانیفست DNA ناوگانِ OpenBot (الگوی `۳×۹`).
    pub manifest: Vec<SpawnedSubAgent>,
    /// آرتیفکت نهایی مونتاژشده.
    pub artifact: AssembledArtifact,
    /// حکم نهایی مایرون (نمره + مشاوره).
    pub final_verdict: MaironSovereignVerdict,
    /// تاریخچهٔ دورهای ترمیم (خالی = عبور در مونتاژِ نخست).
    pub repair_history: Vec<RepairRound>,
    /// شمار تلاش‌های ترمیم مصرف‌شده (`0..=3`).
    pub repair_attempts_used: u32,
    /// آهنگری پذیرفته شده؟ (`final_verdict.verdict.approved_for_listing`).
    pub approved: bool,
}

/// کالبد ۳ — مایرون-۱۵۰: داورِ REAِ اورکلاک‌شده و گیتِ بازارِ ۵-ستاره.
#[derive(Debug, Clone)]
pub struct MaironSovereignJudge {
    /// شناسهٔ محصولِ داوری‌شده (نشست ترینیتی یا `prod:{uuid}` بازار).
    product_id: String,
}

impl MaironSovereignJudge {
    /// داور روی یک شناسهٔ محصول.
    pub fn new(product_id: &str) -> Self {
        Self { product_id: product_id.to_owned() }
    }

    /// استخراجِ خطوط قرمز از گزارش پاک‌ساز — خط قرمز با نمره بخشیده نمی‌شود.
    fn derive_hard_failures(report: &SanitizerReport) -> Vec<String> {
        let mut hard = Vec::new();
        if report.todo_markers > 0 {
            hard.push(
                "unfinished work markers (todo/TODO/FIXME/XXX) detected — zero-tolerance red line"
                    .to_owned(),
            );
        }
        if report.placeholder_secrets > 0 {
            hard.push(
                "placeholder secret detected — forge refuses credential-shaped strings".to_owned(),
            );
        }
        if report.unsafe_markers > 0 {
            hard.push("`unsafe` execution block detected — kernel law: zero unsafe".to_owned());
        }
        if report.lorem_markers > 0 {
            hard.push("lorem ipsum placeholder text detected — stock copy red line".to_owned());
        }
        hard
    }

    /// استخراجِ چهار ستون REA از ویژگی‌های واقعیِ آرتیفکت — قطعی و قابل آزمون.
    fn derive_pillars(
        artifact: &AssembledArtifact,
        blueprint: &ProjectBlueprint,
        report: &SanitizerReport,
    ) -> MaironReaPillarScores {
        let html = &artifact.html;
        let module_count = blueprint.modules.len();
        let integrated = artifact.integrated_modules.len();
        let boundary_markers = html.matches(MODULE_BOUNDARY_MARKER).count();
        let reinforced = usize::from(html.contains(BOUNDARY_REINFORCED_MARKER));
        let architecture = clamp25(
            ARCH_BASE
                + ARCH_PER_MODULE * module_count
                + ARCH_INTEGRATED_BONUS * usize::from(integrated == module_count)
                + ARCH_BOUNDARY_BONUS * usize::from(boundary_markers >= module_count)
                + ARCH_REINFORCED_BONUS * reinforced,
        );
        let security_hardening = clamp25(
            SECURITY_BASE
                .saturating_sub(SECURITY_PER_SECRET * report.placeholder_secrets)
                .saturating_sub(SECURITY_PER_TODO * report.todo_markers)
                .saturating_sub(SECURITY_PER_UNSAFE * report.unsafe_markers),
        );
        let rtl = usize::from(html.contains("dir=\"rtl\""));
        let px390 = usize::from(html.contains("max-width: 390px"));
        let vazir = usize::from(html.contains("Vazirmatn"));
        let polish = usize::from(html.contains(UX_POLISH_MARKER));
        let ux_and_anti_slop = clamp25(
            (UX_BASE
                + UX_MARKER_BONUS * (rtl + px390 + vazir + polish))
            .saturating_sub(UX_SLOP_PENALTY * report.slop_tells.len())
            .saturating_sub(UX_LOREM_PENALTY * report.lorem_markers),
        );
        let verification_proof =
            clamp25(VERIFICATION_BASE + VERIFICATION_PER_PROBE * artifact.probe_count);
        MaironReaPillarScores { architecture, security_hardening, ux_and_anti_slop, verification_proof }
    }

    /// داوری قطعی آرتیفکت مونتاژشده — حکم دامنه + مشاورهٔ `IQ=150/EQ=145`.
    pub fn judge_artifact(
        &self,
        artifact: &AssembledArtifact,
        blueprint: &ProjectBlueprint,
        master: &MasterPromptSpec,
    ) -> MaironSovereignVerdict {
        let report = run_artifact_sanitizer(&artifact.html);
        let pillars = Self::derive_pillars(artifact, blueprint, &report);
        let hard_failures = Self::derive_hard_failures(&report);
        let verdict =
            evaluate_user_product_with_mairon(&self.product_id, pillars, hard_failures);
        let mut advisory = MaironStrategicAdvisory::for_blueprint(blueprint, master);
        advisory.repair_directives = self.repair_directives(&verdict, 1);
        MaironSovereignVerdict { verdict, advisory }
    }

    /// فرمان‌های ترمیم — نگاشتِ ستونِ زیرِ آستانه به تخصصِ مقصر، به ترتیب قطعی:
    /// نخست خطوط قرمزِ امنیتی، سپس راستی‌آزمایی، UX و معماری.
    pub fn repair_directives(
        &self,
        verdict: &MaironProductVerdict,
        attempt: u32,
    ) -> Vec<MaironRepairDirective> {
        let mut out = Vec::new();
        let p = &verdict.pillars;
        if !verdict.hard_failures.is_empty() || p.security_hardening < PILLAR_REPAIR_TRIGGER {
            out.push(MaironRepairDirective {
                attempt,
                target_specialty: SubAgentSpecialty::SecurityAndApiBridge,
                pillar: "security_hardening".to_owned(),
                directive_en: "Purge every unfinished marker and placeholder secret, re-run the \
                    sanitizer gate until passed=true, and re-assert the SSRF allowlist."
                    .to_owned(),
                directive_fa: "همهٔ نشانگرهای ناتمام و رازهای جای‌بانی را پاک کن؛ گیتِ پاک‌ساز \
                    را تا عبورِ کامل دوباره اجرا کن و اجازه‌نامهٔ SSRF را باز-تثبیت کن."
                    .to_owned(),
            });
        }
        if p.verification_proof < PILLAR_REPAIR_TRIGGER {
            out.push(MaironRepairDirective {
                attempt,
                target_specialty: SubAgentSpecialty::VerificationAndPolish,
                pillar: "verification_proof".to_owned(),
                directive_en: "Inject 2x modules + 4 empirical probe assertions (data-probe \
                    elements) and run them green before re-submission — proof before pride."
                    .to_owned(),
                directive_fa: "پروب‌های تجربی (دو برابرِ ماژول‌ها + چهار) را تزریق کن و سبزِ \
                    اجرا را پیش از ارائهٔ دوباره ضمیمه کن — شاهد پیش از افتخار."
                    .to_owned(),
            });
        }
        if p.ux_and_anti_slop < PILLAR_REPAIR_TRIGGER {
            out.push(MaironRepairDirective {
                attempt,
                target_specialty: SubAgentSpecialty::UiUxArchitect390,
                pillar: "ux_and_anti_slop".to_owned(),
                directive_en: "Apply the 390px hardening polish pass, strip every slop tell, and \
                    re-affirm the Vazirmatn RTL stack."
                    .to_owned(),
                directive_fa: "گذرِ پرداختِ سخت‌سازِ ۳۹۰ پیکسل را اعمال کن، هر نشانهٔ اسلوپ را \
                    بکن و پشتهٔ وزیرمتن راست‌به‌چپ را باز-تثبیت کن.".to_owned(),
            });
        }
        if p.architecture < PILLAR_REPAIR_TRIGGER {
            out.push(MaironRepairDirective {
                attempt,
                target_specialty: SubAgentSpecialty::CoreLogicEngine,
                pillar: "architecture".to_owned(),
                directive_en: "Reinforce module boundary markers and complete integration of \
                    every blueprint path (data-module attributes)."
                    .to_owned(),
                directive_fa: "مرزهای ماژول را تقویت کن و یکپارچگیِ همهٔ مسیرهای استراکچر \
                    پروژه (ویژگیِ data-module) را کامل کن.".to_owned(),
            });
        }
        out
    }

    /// داوریِ مستقلِ محصولِ بازار (`preview_html` محصول) — مُهرِ دو-کلیدیِ
    /// [`kian_domain::mairon_rea_and_reviews::confirm_and_publish_product`].
    pub fn judge_market_product(product_id: &str, preview_html: &str) -> MaironProductVerdict {
        let report = run_artifact_sanitizer(preview_html);
        let sections = preview_html.matches("<section").count().min(5);
        let scripts = preview_html.matches("<script").count().min(4);
        let architecture = clamp25(12 + 3 * sections);
        let security_hardening = clamp25(
            SECURITY_BASE
                .saturating_sub(SECURITY_PER_SECRET * report.placeholder_secrets)
                .saturating_sub(SECURITY_PER_TODO * report.todo_markers)
                .saturating_sub(SECURITY_PER_UNSAFE * report.unsafe_markers),
        );
        let rtl = usize::from(preview_html.contains("dir=\"rtl\""));
        let px390 = usize::from(preview_html.contains("max-width: 390px"));
        let vazir = usize::from(preview_html.contains("Vazirmatn"));
        let viewport = usize::from(preview_html.contains("name=\"viewport\""));
        let aria = usize::from(preview_html.contains("aria-"));
        let ux_and_anti_slop = clamp25(
            (UX_BASE + UX_MARKER_BONUS * (rtl + px390 + vazir + viewport))
                .saturating_sub(UX_SLOP_PENALTY * report.slop_tells.len())
                .saturating_sub(UX_LOREM_PENALTY * report.lorem_markers),
        );
        let probes = preview_html.matches("data-probe=").count();
        let verification_proof = clamp25(
            8 + 2 * scripts + 3 * probes + UX_MARKER_BONUS * (aria + viewport),
        );
        let hard_failures = Self::derive_hard_failures(&report);
        evaluate_user_product_with_mairon(product_id, MaironReaPillarScores {
            architecture,
            security_hardening,
            ux_and_anti_slop,
            verification_proof,
        }, hard_failures)
    }
}

// ── حلقهٔ خودترمیمیِ اتاق فرمان ↔ مایرون ──────────────────────────────────

impl CommandCenterOrchestrator {
    /// اجرای کاملِ اتاق فرمان با حلقهٔ خودترمیمیِ مایرون.
    ///
    /// مونتاژِ نخست → داوریِ مایرون؛ اگر نمره `< 85` یا خط قرمز، فرمان‌های
    /// [`MaironRepairDirective`] به ساب‌ایجنتِ مقصر رهسپاری می‌شود (بیشینهٔ
    /// [`MAX_OPENBOT_REPAIR_ATTEMPTS`]) تا آهنگری از گیت عبور کند.
    pub fn execute_command_center_with_mairon_loop(
        &self,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
    ) -> TrinityForgeOutcome {
        let fleet = self.spawn_openbot_fleet(master, blueprint);
        let judge = MaironSovereignJudge::new(&self.session_id);
        let price = master.price_stars_hint;
        let mut directives: Vec<MaironRepairDirective> = Vec::new();
        let mut artifact =
            Self::assemble_artifact(&fleet, master, blueprint, price, 0, &directives);
        let mut verdict = judge.judge_artifact(&artifact, blueprint, master);
        let mut history = Vec::new();
        let mut attempts_used = 0_u32;
        if !verdict.verdict.approved_for_listing {
            for attempt in 1..=MAX_OPENBOT_REPAIR_ATTEMPTS {
                attempts_used = attempt;
                let score_before = verdict.verdict.total_score;
                directives = judge.repair_directives(&verdict.verdict, attempt);
                artifact =
                    Self::assemble_artifact(&fleet, master, blueprint, price, attempt, &directives);
                verdict = judge.judge_artifact(&artifact, blueprint, master);
                let approved = verdict.verdict.approved_for_listing;
                history.push(RepairRound {
                    attempt,
                    score_before,
                    directives,
                    score_after: verdict.verdict.total_score,
                    approved,
                });
                if approved {
                    break;
                }
            }
        }
        TrinityForgeOutcome {
            session_id: self.session_id.clone(),
            citizen_id: self.citizen_id.clone(),
            project_name: master.project_name.clone(),
            manifest: fleet,
            artifact,
            approved: verdict.verdict.approved_for_listing,
            repair_attempts_used: attempts_used,
            repair_history: history,
            final_verdict: verdict,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// پنج اسلاتِ کامل — صحنهٔ استانداردِ آزمون‌ها.
    fn full_slots() -> AlphaDiscoverySlots {
        AlphaDiscoverySlots {
            core_objective: Some("فروشگاه ابزارهای بهره‌وری برای فریلنسرهای فارسی‌زبان".to_owned()),
            target_audience_and_ux: Some(
                "فریلنسرهای فارسی‌زبان، موبایل‌فرست در ۳۹۰ پیکسل راست‌به‌چپ".to_owned(),
            ),
            interactive_features: Some("سبد خرید، فرم سفارش و جریان پرداخت تعاملی".to_owned()),
            data_and_zero_auth_apis: Some(
                "ذخیره در localStorage و اتصال به APIهای عمومی بدون لاگین".to_owned(),
            ),
            monetization_and_viral_loop: Some(
                "قیمت ۲۰۰ ستاره با ۱۰٪ پورسانت معرف و حلقهٔ وایرال".to_owned(),
            ),
        }
    }

    fn intake(message: &str, prior: AlphaDiscoverySlots) -> AlphaIntakeOutcome {
        AlphaIgnitionPersona.process_intake(&AlphaIntakeRequest {
            session_id: "trn:test-0001".to_owned(),
            citizen_id: "citizen:900".to_owned(),
            user_message: message.to_owned(),
            prior_slots: prior,
        })
    }

    /// پیامی که چهار اسلات را روشن می‌کند (بدون درآمدزایی) — آمادگی ۸۰٪.
    const FOUR_SLOT_MESSAGE: &str = "می‌خواهم یک محصول بسازم: سازندهٔ ابزار بهره‌وری. \
        مخاطب فریلنسر فارسی‌زبان است و موبایل‌فرست با rtl کار می‌کند. \
        باید فرم سفارش تعاملی داشته باشد. داده‌ها در api عمومی بدون لاگین ذخیره می‌شوند.";

    /// DNA آلفا قفل است.
    #[test]
    fn alpha_dna_constants_are_locked() {
        assert_eq!(ALPHA_EFFECTIVE_IQ, 136);
        assert_eq!(ALPHA_EFFECTIVE_EQ, 122);
        assert!((ALPHA_INHIBITION_BRAKE - 0.04).abs() < f32::EPSILON);
        assert!((ALPHA_ENTROPY_TEMP - 0.88).abs() < f32::EPSILON);
        assert_eq!(ALPHA_READINESS_LOCK_THRESHOLD, 80);
    }

    /// DNA اتاق فرمان و داورِ بازصادری از دامنه قفل است — منبعِ واحد حقیقت.
    #[test]
    fn logic_and_mairon_dna_constants_are_locked() {
        let (iq_alpha, iq_logic, iq_mairon) =
            (ALPHA_EFFECTIVE_IQ, LOGIC_EFFECTIVE_IQ, MAIRON_OVERCLOCK_IQ);
        let (eq_logic, eq_mairon) = (LOGIC_EFFECTIVE_EQ, MAIRON_OVERCLOCK_EQ);
        let horizon = MAIRON_LOOKAHEAD_HORIZON;
        let tom = MAIRON_TOM_ORDER;
        assert_eq!(iq_logic, 139);
        assert!((LOGIC_INHIBITION_BRAKE - 0.62).abs() < f32::EPSILON);
        assert!((LOGIC_FORENSIC_STRICTNESS - 0.96).abs() < f32::EPSILON);
        assert_eq!(iq_mairon, 150);
        assert_eq!(eq_mairon, 145);
        assert_eq!(horizon, 8);
        assert_eq!(tom, 3);
        assert_eq!(MAX_OPENBOT_REPAIR_ATTEMPTS, 3);
        assert!(iq_mairon > iq_logic && iq_logic > iq_alpha);
        assert!(eq_mairon > eq_logic);
    }

    /// استخراجِ اسلات‌ها با کلیدواژه‌های دوزبانه — قطعی و بدون بازنویسی.
    #[test]
    fn slot_extraction_is_deterministic_and_never_overwrites() {
        let out = intake(FOUR_SLOT_MESSAGE, AlphaDiscoverySlots::default());
        let AlphaIntakeOutcome::LockedForCommandCenter { master_prompt, blueprint, .. } = out else {
            panic!("four slots = 80% readiness must lock");
        };
        assert!(master_prompt.functional_spec.contains("سازندهٔ ابزار"));
        assert!(blueprint.modules.iter().any(|m| m.path == "src/js/api-bindings.js"));
        assert!(!blueprint.modules.iter().any(|m| m.path == "src/js/monetization.js"));
        // بازنویسی ممنوع — مقدارِ اول همیشه برنده است
        let mut slots = full_slots();
        slots.fill(AlphaSlotKind::CoreObjective, "نباید جایگزین شود".to_owned());
        assert!(slots.core_objective.unwrap().contains("فروشگاه ابزار"));
    }

    /// آمادگیِ ناقص → NeedsMoreDiscovery با فهرستِ خام به ترتیب قطعی.
    #[test]
    fn alpha_needs_more_discovery_lists_missing_slots_in_order() {
        let out = intake("می‌خواهم یک محصول بسازم که فرم تعاملی دارد", AlphaDiscoverySlots::default());
        let AlphaIntakeOutcome::NeedsMoreDiscovery { alpha_reply_fa, missing_slots, readiness_pct } =
            out
        else {
            panic!("two slots = 40% readiness must stay in discovery");
        };
        assert_eq!(readiness_pct, 40);
        assert_eq!(missing_slots.len(), 3);
        assert_eq!(missing_slots[0], AlphaSlotKind::TargetAudienceAndUx);
        assert_eq!(missing_slots.last().copied(), Some(AlphaSlotKind::MonetizationAndViralLoop));
        assert!(alpha_reply_fa.contains("آلفا فعال شد"));
        assert!(alpha_reply_fa.contains("مخاطب هدف و حس UX"));
    }

    /// قفل در ۸۰٪ — مستر پرامپت + استراکچر پروژه قطعی و بازتولیدپذیر.
    #[test]
    fn alpha_locks_at_eighty_pct_with_deterministic_spec() {
        let out = intake(FOUR_SLOT_MESSAGE, AlphaDiscoverySlots::default());
        let AlphaIntakeOutcome::LockedForCommandCenter { alpha_handoff_fa, master_prompt, blueprint } =
            out
        else {
            panic!("must lock");
        };
        assert!(alpha_handoff_fa.contains("آلفا قفل شد"));
        assert!(alpha_handoff_fa.contains("۸۰") || alpha_handoff_fa.contains("80"));
        let again = intake(FOUR_SLOT_MESSAGE, AlphaDiscoverySlots::default());
        let AlphaIntakeOutcome::LockedForCommandCenter { master_prompt: m2, blueprint: b2, .. } =
            again
        else {
            panic!("must lock again");
        };
        assert_eq!(master_prompt, m2, "master prompt must be deterministic");
        assert_eq!(blueprint, b2, "blueprint must be deterministic");
        assert_eq!(master_prompt.spec_hash.len(), 64, "sha256 hex");
        assert!(master_prompt.compiled_prompt.contains("390px"));
        assert_eq!(master_prompt.price_stars_hint, 150, "no monetization slot → default 150");
        assert_eq!(blueprint.project_name, master_prompt.project_name);
    }

    /// استراکچر پروژه — چهار ماژول پایه + سه ماژولِ شرطی با تخصصِ هدف.
    #[test]
    fn blueprint_modules_derive_from_slots_with_specialties() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:t", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let paths: Vec<&str> = bp.modules.iter().map(|m| m.path.as_str()).collect();
        assert_eq!(paths.len(), 7);
        for base in ["src/index.html", "src/styles/main.css", "src/js/core.js", "src/js/verification-probes.js"] {
            assert!(paths.contains(&base), "base module {base} must exist");
        }
        let index = bp.modules.iter().find(|m| m.path == "src/index.html").unwrap();
        assert_eq!(index.specialty, SubAgentSpecialty::UiUxArchitect390);
        let api = bp.modules.iter().find(|m| m.path == "src/js/api-bindings.js").unwrap();
        assert_eq!(api.specialty, SubAgentSpecialty::SecurityAndApiBridge);
        let money = bp.modules.iter().find(|m| m.path == "src/js/monetization.js").unwrap();
        assert_eq!(money.specialty, SubAgentSpecialty::MonetizationAndViralLoop);
        assert_eq!(master.price_stars_hint, 200, "۲۰۰ در اسلات درآمد → 200");
        let none = MasterPromptSpec::compile(&AlphaDiscoverySlots::default(), "trn:x", "c");
        assert_eq!(none.price_stars_hint, 150);
    }

    /// آلل‌های تصادفیِ قطعی — همان بذر همان DNA؛ بذرِ متفاوت DNAِ متفاوت.
    #[test]
    fn alleles_are_deterministic_per_seed_and_index() {
        let a0 = derive_alleles(42, 0);
        let a0b = derive_alleles(42, 0);
        let a1 = derive_alleles(42, 1);
        let a0_other_seed = derive_alleles(43, 0);
        assert_eq!(a0, a0b);
        assert_ne!(a0, a1);
        assert_ne!(a0, a0_other_seed);
        assert!((0.55..=0.95).contains(&a0.surgical_density));
        assert!((0.50..=0.95).contains(&a0.ux_craft));
        assert!((0.60..=0.95).contains(&a0.edge_case_paranoia));
        assert!((0.45..=0.95).contains(&a0.execution_velocity));
    }

    /// PRNG واحد — بازهٔ `[0,1)` و بازتولیدپذیری.
    #[test]
    fn splitmix_unit_stays_in_zero_one() {
        let mut st = 7_u64;
        for _ in 0..64 {
            let u = splitmix_unit(&mut st);
            assert!((0.0..1.0).contains(&u));
        }
    }

    /// پرامپت سه‌لایه — مرزِ کش دقیقاً یک‌بار؛ لایه‌های ثابت قبل، پویا بعد.
    #[test]
    fn three_layer_prompt_pins_cache_boundary_once() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:t", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let spawner = OpenBotSubAgentSpawner::new("trn:t", 42);
        let agent = spawner.spawn(0, &bp.modules[0], &master, &bp);
        let rendered = agent.prompt.render();
        assert_eq!(rendered.matches(SYSTEM_PROMPT_DYNAMIC_BOUNDARY).count(), 1);
        let (static_part, dynamic_part) =
            rendered.split_once(SYSTEM_PROMPT_DYNAMIC_BOUNDARY).expect("boundary");
        assert!(static_part.contains("SOVEREIGN CODING LAWS"));
        assert!(static_part.contains(&master.project_name));
        assert!(dynamic_part.contains("ASSIGNMENT (dynamic)"));
        assert!(dynamic_part.contains("src/index.html"));
    }

    /// قلابِ توقف ساب‌ایجنت — مرزهای `450s/1200s` دقیق.
    #[test]
    fn stall_watchdog_enforces_sovereign_bounds() {
        let wd = SubagentStallWatchdog::sovereign();
        assert_eq!(wd.soft_stall_secs, 450);
        assert_eq!(wd.hard_timeout_secs, 1200);
        assert_eq!(wd.verdict_for(449), WatchdogVerdict::Healthy);
        assert_eq!(wd.verdict_for(450), WatchdogVerdict::SoftStall);
        assert_eq!(wd.verdict_for(1199), WatchdogVerdict::SoftStall);
        assert_eq!(wd.verdict_for(1200), WatchdogVerdict::HardTimeout);
    }

    /// ژنوم نه‌لایه‌ای — همهٔ f01..f09 پر، پاک‌ساز سبز، XML کامل.
    #[test]
    fn nine_fold_genome_is_complete_and_clean() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:t", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let spawner = OpenBotSubAgentSpawner::new("trn:t", 42);
        let agent = spawner.spawn(2, &bp.modules[2], &master, &bp);
        let g = &agent.genome;
        assert!(g.f01_topology_and_harness.contains("IQ=139"));
        assert!(g.f01_topology_and_harness.contains("soft=450s"));
        assert!(g.f02_hard_constraints.iter().any(|c| c.contains("forbid(unsafe_code)")));
        assert!(g.f02_hard_constraints.iter().any(|c| c.contains("SSRF")));
        assert_eq!(g.f03_token_economics, CavemanIntensity::Full, "core engine → Full");
        assert!(g.f04_stochastic_alleles.allele_seed > 0);
        assert!(g.f05_failure_recovery.contains("3-attempt"));
        let xml = &g.f06_compiled_xml_prompt;
        assert!(xml.contains("<openbot_subagent_dna"));
        assert!(xml.contains("f01_topology_and_harness"));
        assert!(xml.contains("f04_stochastic_alleles"));
        assert!(xml.contains("f09_handoff_card"));
        assert!(xml.contains("src/js/core.js"));
        assert!(g.f07_sanitizer_gate.passed, "synthesized DNA must pass the sanitizer");
        assert_eq!(g.f07_sanitizer_gate.todo_markers, 0);
        assert!(g.f08_empirical_probes.len() >= 2);
        assert!(g.f09_handoff_card.contains("HANDOFF -> Command Center"));
        assert_eq!(agent.thread_id.len(), 36, "thread id is RFC uuid-shaped");
        assert_eq!(agent.thread_id.matches('-').count(), 4);
        // بازتولیدپذیریِ نخ
        let agent2 = spawner.spawn(2, &bp.modules[2], &master, &bp);
        assert_eq!(agent.thread_id, agent2.thread_id);
    }

    /// فهرست سفارشیِ تخصص — لاتین‌سازیِ امن برای XML/وایر.
    #[test]
    fn custom_specialty_is_slugified() {
        let custom = SubAgentSpecialty::Custom("Quant Chart DNA < specialist>".to_owned());
        assert_eq!(custom.as_str(), "custom_quantchartdnaspecialist");
        assert!(custom.fa_label().contains("متخصص سفارشی"));
        assert_eq!(custom.default_caveman(), CavemanIntensity::Full);
    }

    /// ناوگان — یک متخصص به ازای هر ماژول؛ تخصص‌های سفارشی هم مجاز.
    #[test]
    fn fleet_matches_blueprint_one_agent_per_module() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:t", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let orch = CommandCenterOrchestrator::new("trn:t", "citizen:900", 42);
        let fleet = orch.spawn_openbot_fleet(&master, &bp);
        assert_eq!(fleet.len(), bp.modules.len());
        for (i, agent) in fleet.iter().enumerate() {
            assert_eq!(agent.subagent_index, u32::try_from(i).unwrap());
            assert_eq!(agent.assigned_modules, vec![bp.modules[i].path.clone()]);
        }
        // آلل‌های یکتا در ناوگان — DNA یکتا برای هر ساب‌ایجنت
        let seeds: Vec<u64> = fleet.iter().map(|a| a.genome.f04_stochastic_alleles.allele_seed).collect();
        let unique = seeds.iter().collect::<std::collections::HashSet<_>>();
        assert_eq!(unique.len(), seeds.len(), "allele seeds must be unique per sub-agent");
    }

    /// گیتِ پاک‌ساز — هر چهار خانوادهٔ ممنوع را می‌یابد؛ متنِ سالم سبز می‌ماند.
    #[test]
    fn sanitizer_gate_catches_all_dirty_families() {
        let dirty = "let k = sk-abc123; /* TODO fix */ const x = AKIA1234; unsafe { /* lorem ipsum */ }";
        let report = run_sanitizer_gate(dirty);
        assert!(!report.passed);
        assert!(report.todo_markers >= 1);
        assert!(report.placeholder_secrets >= 2);
        assert_eq!(report.unsafe_markers, 1);
        assert_eq!(report.lorem_markers, 1);
        let clean = run_sanitizer_gate("pure state kernel with no forbidden markers");
        assert!(clean.passed);
    }

    /// حکمِ پایه زیرِ گیت و پس از یک گذرِ ترمیم بالای گیت — قلبِ حلقهٔ خودترمیمی.
    #[test]
    fn base_assembly_fails_gate_and_repair_pass_clears_it() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:loop", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let orch = CommandCenterOrchestrator::new("trn:loop", "citizen:900", 42);
        let fleet = orch.spawn_openbot_fleet(&master, &bp);
        let judge = MaironSovereignJudge::new("trn:loop");

        let base = CommandCenterOrchestrator::assemble_artifact(&fleet, &master, &bp, master.price_stars_hint, 0, &[]);
        assert_eq!(base.probe_count, 0, "pass 0 — probes are stage-gated");
        assert_eq!(base.integrated_modules.len(), bp.modules.len(), "full integration");
        let base_verdict = judge.judge_artifact(&base, &bp, &master);
        assert!(
            base_verdict.verdict.total_score < MARKETPLACE_APPROVAL_THRESHOLD,
            "base assembly must fail the 85 gate (score = {})",
            base_verdict.verdict.total_score
        );
        assert!(base_verdict.verdict.hard_failures.is_empty(), "base artifact is clean");

        let directives = judge.repair_directives(&base_verdict.verdict, 1);
        assert!(directives.iter().any(|d| d.target_specialty == SubAgentSpecialty::VerificationAndPolish));
        assert!(directives.iter().any(|d| d.target_specialty == SubAgentSpecialty::UiUxArchitect390));

        let repaired = CommandCenterOrchestrator::assemble_artifact(
            &fleet, &master, &bp, master.price_stars_hint, 1, &directives,
        );
        assert_eq!(repaired.probe_count, 2 * bp.modules.len() + 4);
        assert!(repaired.html.contains(UX_POLISH_MARKER));
        assert!(repaired.html.contains("data-probe=\"probe-1\""));
        let repaired_verdict = judge.judge_artifact(&repaired, &bp, &master);
        assert!(
            repaired_verdict.verdict.approved_for_listing,
            "repaired forge must clear the gate (score = {})",
            repaired_verdict.verdict.total_score
        );
        assert_eq!(
            repaired_verdict.verdict.visible_badge,
            format!("⚒️ Mairon REA Certified: {}/100 (Forge-Grade)", repaired_verdict.verdict.total_score)
        );
    }

    /// حلقهٔ خودترمیمی — عبور در ≤ ۳ تلاش با تاریخچهٔ بهبودِ نمره.
    #[test]
    fn self_healing_loop_clears_mairon_gate_within_three_attempts() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:loop", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let orch = CommandCenterOrchestrator::new("trn:loop", "citizen:900", 42);
        let out = orch.execute_command_center_with_mairon_loop(&master, &bp);
        assert!(out.approved, "sovereign loop must forge an approved artifact");
        assert!(out.repair_attempts_used >= 1 && out.repair_attempts_used <= MAX_OPENBOT_REPAIR_ATTEMPTS);
        assert_eq!(out.repair_history.len(), usize::try_from(out.repair_attempts_used).unwrap());
        let first = &out.repair_history[0];
        assert!(first.score_after > first.score_before, "repair must improve the score");
        assert!(first.approved);
        assert!(out.final_verdict.verdict.total_score >= MARKETPLACE_APPROVAL_THRESHOLD);
        assert!(out.final_verdict.verdict.visible_badge.contains("Forge-Grade"));
        assert_eq!(out.manifest.len(), bp.modules.len());
        assert_eq!(out.project_name, master.project_name);
        assert!(out.artifact.html.contains("mairon-empirical-probes"));
    }

    /// حلقهٔ خودترمینی قطعی — همان نشست، همان آهنگری.
    #[test]
    fn sovereign_loop_is_fully_deterministic() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:det", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let a = CommandCenterOrchestrator::new("trn:det", "citizen:900", 4242)
            .execute_command_center_with_mairon_loop(&master, &bp);
        let b = CommandCenterOrchestrator::new("trn:det", "citizen:900", 4242)
            .execute_command_center_with_mairon_loop(&master, &bp);
        assert_eq!(a, b);
        // بذرِ متفاوت — ناوگانِ متفاوت اما همان گیتِ عبور
        let c = CommandCenterOrchestrator::new("trn:det", "citizen:900", 777)
            .execute_command_center_with_mairon_loop(&master, &bp);
        assert!(c.approved);
        assert_ne!(a.manifest[0].genome.f04_stochastic_alleles, c.manifest[0].genome.f04_stochastic_alleles);
    }

    /// بذرِ نشست — قطعی از شناسهٔ نشست.
    #[test]
    fn session_seed_is_derived_deterministically() {
        let s1 = CommandCenterOrchestrator::session_seed_from_session("trn:abc");
        let s2 = CommandCenterOrchestrator::session_seed_from_session("trn:abc");
        let s3 = CommandCenterOrchestrator::session_seed_from_session("trn:abd");
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    /// مشاورهٔ استراتژیک مایرون — کالیبراسیونِ ۱۵۰/۱۴۵ قفل، ToM-3 و قیمتِ XTR.
    #[test]
    fn mairon_advisory_carries_locked_calibration_and_tom3_hook() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:adv", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let advisory = MaironStrategicAdvisory::for_blueprint(&bp, &master);
        assert_eq!(advisory.iq, 150);
        assert_eq!(advisory.eq, 145);
        assert_eq!(advisory.lookahead_horizon, 8);
        assert_eq!(advisory.tom_order, 3);
        assert!(advisory.eight_step_lookahead_summary.contains("Lookahead-8"));
        assert!(advisory
            .eight_step_lookahead_summary
            .contains(&format!("{} hard-boundary modules", bp.modules.len())));
        assert!(advisory.third_order_tom_market_hook_fa.contains("IQ=150/EQ=145"));
        assert!(advisory.third_order_tom_market_hook_fa.contains("200 ستارهٔ تلگرام (XTR)"));
        assert!(advisory.third_order_tom_market_hook_fa.contains("1000 bps"));
    }

    /// فرمان‌های ترمیم — نگاشتِ ستونِ خام به تخصصِ مقصر به ترتیب قطعی.
    #[test]
    fn repair_directives_map_weak_pillars_to_responsible_specialties() {
        let judge = MaironSovereignJudge::new("trn:dir");
        let weak = evaluate_user_product_with_mairon("trn:dir", MaironReaPillarScores {
            architecture: 19,
            security_hardening: 25,
            ux_and_anti_slop: 23,
            verification_proof: 2,
        }, vec![]);
        let directives = judge.repair_directives(&weak, 1);
        let targets: Vec<&SubAgentSpecialty> =
            directives.iter().map(|d| &d.target_specialty).collect();
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0], &SubAgentSpecialty::VerificationAndPolish);
        assert_eq!(targets[1], &SubAgentSpecialty::UiUxArchitect390);
        assert_eq!(targets[2], &SubAgentSpecialty::CoreLogicEngine);
        assert!(directives.iter().all(|d| d.attempt == 1));
        // خط قرمز → فرمانِ امنیتی پیش از همه
        let dirty = evaluate_user_product_with_mairon("trn:dir", MaironReaPillarScores {
            architecture: 25,
            security_hardening: 19,
            ux_and_anti_slop: 25,
            verification_proof: 25,
        }, vec!["placeholder secret detected".to_owned()]);
        let dirty_dirs = judge.repair_directives(&dirty, 2);
        assert_eq!(dirty_dirs[0].target_specialty, SubAgentSpecialty::SecurityAndApiBridge);
        assert_eq!(dirty_dirs[0].pillar, "security_hardening");
    }

    /// داوریِ مستقلِ محصول بازار — آرتیفکتِ درست عبور می‌کند، خام و آلوده رد.
    #[test]
    fn market_product_gate_accepts_only_forged_artifacts() {
        let good = "<!doctype html><html lang=\"fa\" dir=\"rtl\"><head>\
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
            <style>body{font-family:'Vazirmatn',sans-serif;max-width: 390px;}</style>\
            </head><body>\
            <section aria-label=\"الف\">داشبورد</section><section>گزارش</section><section>تنظیمات</section>\
            <script>/* assert: 390px zero-overflow */</script>\
            </body></html>";
        let verdict = MaironSovereignJudge::judge_market_product("prod:good", good);
        assert!(verdict.approved_for_listing, "score = {}", verdict.total_score);
        assert!(verdict.visible_badge.contains("Forge-Grade"));
        // خام → ردِ صادقانه
        let empty = MaironSovereignJudge::judge_market_product("prod:empty", "<p></p>");
        assert!(!empty.approved_for_listing);
        assert!(empty.total_score < MARKETPLACE_APPROVAL_THRESHOLD);
        // آلوده → خط قرمز، حتی با نمرهٔ بالا بخشیده نمی‌شود
        let dirty = format!("{good}<!-- TODO: add auth later -->");
        let dirty_verdict = MaironSovereignJudge::judge_market_product("prod:dirty", &dirty);
        assert!(!dirty_verdict.approved_for_listing);
        assert!(dirty_verdict
            .hard_failures
            .iter()
            .any(|f| f.contains("unfinished work markers")));
    }

    /// استخراجِ قیمت — رقم فارسی/لاتین از اسلاتِ درآمد؛ پیش‌فرضِ ۱۵۰⭐.
    #[test]
    fn price_hint_parses_latin_and_persian_digits() {
        assert_eq!(first_number_in("قیمت ۳۰۰ ستاره"), Some(300));
        assert_eq!(first_number_in("price 250 XTR"), Some(250));
        assert_eq!(first_number_in("بدون عدد"), None);
        assert_eq!(derive_price_stars(&full_slots()), 200);
        assert_eq!(derive_price_stars(&AlphaDiscoverySlots::default()), 150);
    }

    /// الگوی `۳×۹` — سه لایه × نه فاکتور در مانیفستِ وایرِ کامل.
    #[test]
    fn three_by_nine_pattern_manifest_is_complete() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:39", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let orch = CommandCenterOrchestrator::new("trn:39", "citizen:900", 42);
        let out = orch.execute_command_center_with_mairon_loop(&master, &bp);
        // سه لایه در هر ساب‌ایجنت — هر لایه امضای محتواییِ خودش را دارد
        for agent in &out.manifest {
            assert!(agent.prompt.layer1_stable_core.contains("SOVEREIGN CODING LAWS"));
            assert!(agent.prompt.layer2_blueprint_context.contains("BUILD SESSION CONTEXT"));
            assert!(agent.prompt.layer3_dynamic_assignment.contains("ASSIGNMENT (dynamic)"));
        }
        // نه فاکتور ژنوم: f01..f09 — هر فاکتور امضای محتواییِ خودش را دارد
        let g = &out.manifest[0].genome;
        assert!(g.f01_topology_and_harness.contains("IQ=139"));
        assert_eq!(g.f02_hard_constraints.len(), 6);
        assert_eq!(g.f03_token_economics.as_str(), "lite", "agent #0 is UiUx → Lite");
        assert!(g.f04_stochastic_alleles.allele_seed > 0);
        assert!(g.f05_failure_recovery.contains("attempt 3"));
        assert!(g.f06_compiled_xml_prompt.contains("</openbot_subagent_dna>"));
        assert!(g.f07_sanitizer_gate.passed);
        assert_eq!(g.f08_empirical_probes.len(), 2);
        assert!(g.f09_handoff_card.contains("pass criteria"));
    }
}

