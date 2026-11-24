//! موتورِ انحصارِ حاکم — روترِ ژنومِ پنتامدلی + اورکستراتورِ انحصار
//! (Wave 7 — Step 4/4 + Commit #27 پنتا-مدل).
//!
//! دو هستهٔ روی هم:
//!  * [`QuadModelGenomeRouter`] — رجیستریِ ۵ ژنومِ فرانتیرِ حاکم
//!    (`GPT 6-Web`, `GEMINI 4 ARAGON HIGH`, `DeepSeek-3.2-WEB-Think-mode`,
//!    `Cluade-sonnet-5.5-extra`, `GLM 5.3 flash MAX`) با تأییدِ ۹فایلیِ هر مدل
//!    (۴۵ فایل روی دیسک، نامِ canonical جدید) + سنتزِ پرامپتِ کلونِ تقطیرشده
//!    به ازای مدل × دامنهٔ کار + انتخابِ دومُدِ (`ManualOverride` /
//!    `AgentAutoDecision`) + سیم‌کشی به ترامینیتیِ سه‌آواتار
//!    (`Alpha` → `Logic-Prime` → `Mairon`). نامِ تاریخیِ `QuadModelGenomeRouter`
//!    به‌عنوان alias همان روترِ ۵مدلی می‌ماند.
//!  * [`SovereignMonopolyOrchestrator`] — ترکیبِ روترِ ژنوم، منشورِ
//!    چهارجریانیِ روزِ اولِ [`kian_wallet::base_treasury`]، descriptor هایِ
//!    قراردادهایِ Base `8453` و کیتِ لانچِ Product Hunt/CMC در یک موتورِ
//!    وضعیت و اجرایِ واحد.
//!
//! قواعدِ اکتشافی: alias ها همیشه حل می‌شوند (`Cluade-sonnet-5.5-extra` و
//! `Claude-sonnet-5.5-extra` به مدل #04، `GLM 5.3 flash MAX` و
//! `glm-5.3-flash-max` و `GLM-5.3-Flash-Max` به مدل #05 می‌روند)؛ فایلِ ناموجود
//! هرگز «موجود» فرض نمی‌شود — تأییدِ دیسکی fail-closed است و مسیرِ builtin فقط
//! تعدادِ ۴۵ را حفظ می‌کند، نه محتوایِ جعلی.

use std::fmt;
use std::path::{Path, PathBuf};

use kian_wallet::base_treasury::{
    compute_day_one_atomic_split, BaseTreasuryError, BoostTier, DayOneAtomicSplit,
    DayOneRevenueCharter, SettlementAsset, BASE_BLOCK_EXPLORER, BASE_MAINNET_CHAIN_ID,
    BASE_MAINNET_RPC_URL, BASE_TREASURY_SIGNATURE,
};

// ── امضا ─────────────────────────────────────────────────────────────────────

/// امضایِ موتورِ انحصارِ حاکم — برای smoke test و لاگ‌ها.
pub const SOVEREIGN_MONOPOLY_SIGNATURE: &str = "sovereign-monopoly::quad-genome::8453::wave7";

/// نام‌هایِ canonical ۹فایلی — قراردادِ سختِ `omni-genome-v4`.
pub const CANONICAL_GENOME_FILES: [&str; 9] = [
    "01_IDENTITY_AND_PERSONA_GENOME.md",
    "02_SYSTEM_ARCHITECTURE_AND_CONSTRAINTS.md",
    "03_REASONING_AND_DECISION_TREE.md",
    "04_TOOL_ORCHESTRATION_AND_AGENTIC_LOOP.md",
    "05_CODING_AND_ENGINEERING_DOCTRINE.md",
    "06_UI_UX_AND_ARTIFACT_CRAFT.md",
    "07_SAFETY_GUARDRAILS_AND_EDGE_CASES.md",
    "08_FAILURE_MODES_AND_BLINDSPOTS.md",
    "09_DISTILLED_CLONE_SYSTEM_PROMPT.md",
];

/// تعدادِ مدل‌هایِ ژنوم — ۵ فرانتیرِ حاکم (پنتا-مدل، Commit #27).
pub const SOVEREIGN_MODEL_COUNT: usize = 5;

/// تعدادِ کلِ فایل‌هایِ ژنوم — ۵ × ۹.
pub const SOVEREIGN_GENOME_FILE_COUNT: usize = SOVEREIGN_MODEL_COUNT * CANONICAL_GENOME_FILES.len();

/// نامِ تاریخیِ ثابتِ شمارشِ مدل — حالا همان ۵مدلِ حاکم.
pub const QUAD_MODEL_COUNT: usize = SOVEREIGN_MODEL_COUNT;

/// نامِ تاریخیِ ثابتِ شمارشِ فایل — حالا همان ۴۵فایلِ حاکم.
pub const QUAD_GENOME_FILE_COUNT: usize = SOVEREIGN_GENOME_FILE_COUNT;

// ── مدل‌ها ───────────────────────────────────────────────────────────────────

/// پنج ژنومِ فرانتیرِ حاکم — ترتیبِ canonical ماتریس.
///
/// `SovereignModelId` نامِ هم‌ارزِ همین enum در وایر/مستنداتِ Commit #27 است.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadModelGenome {
    /// مدل #01 — سنتزِ وبِ لحظه‌ای و هنرِ آرتیفکتِ تعاملیِ فریم‌۱.
    Gpt6Web,
    /// مدل #02 — سنتزِ میان‌مخزنیِ ۱M-توکنی و اورکستراتورِ ۹-مدالیتی.
    Gemini4AragonHigh,
    /// مدل #03 — ریاضیاتِ کمّیِ `<think>` و اثباتِ آلفایِ علّی.
    DeepSeek32WebThinkMode,
    /// مدل #04 — مهندسیِ جراحیِ Rust و ممیزیِ قراردادِ هوشمند.
    ClaudeSonnet55Extra,
    /// مدل #05 — اجرایِ فلشِ چندابزاری با تأخیرِ حداقلی (Commit #27).
    Glm53FlashMax,
}

/// شناسهٔ حاکمِ مدل در وایر — alias صریحِ [`QuadModelGenome`] (Commit #27).
pub type SovereignModelId = QuadModelGenome;

impl QuadModelGenome {
    /// همهٔ مدل‌ها به ترتیبِ canonical ماتریس.
    pub const ALL: [QuadModelGenome; SOVEREIGN_MODEL_COUNT] = [
        Self::Gpt6Web,
        Self::Gemini4AragonHigh,
        Self::DeepSeek32WebThinkMode,
        Self::ClaudeSonnet55Extra,
        Self::Glm53FlashMax,
    ];

    /// شناسهٔ دقیقِ نمایش — همان شناسهٔ آپلودِ ماجیک.
    #[must_use]
    pub const fn exact_id(self) -> &'static str {
        match self {
            Self::Gpt6Web => "GPT 6-Web",
            Self::Gemini4AragonHigh => "GEMINI 4 ARAGON HIGH",
            Self::DeepSeek32WebThinkMode => "DeepSeek-3.2-WEB-Think-mode",
            Self::ClaudeSonnet55Extra => "Cluade-sonnet-5.5-extra",
            Self::Glm53FlashMax => "GLM 5.3 flash MAX",
        }
    }

    /// شناسه‌هایِ alias — همه به همین مدل حل می‌شوند.
    #[must_use]
    pub fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Gpt6Web => &["GPT-6", "gpt-6-web", "gpt-6-standard-free"],
            Self::Gemini4AragonHigh => &["gemini-4-aragon-high", "ARAGON HIGH"],
            Self::DeepSeek32WebThinkMode => &["deepseek-3.2-web-think-mode", "DS-3.2-Think"],
            Self::ClaudeSonnet55Extra => {
                &["Claude-sonnet-5.5-extra", "claude-sonnet-5-5", "Claude Sonnet 5.5"]
            }
            Self::Glm53FlashMax => &["glm-5.3-flash-max", "GLM-5.3-Flash-Max", "GLM 5.3 Flash Max"],
        }
    }

    /// پوشهٔ canonical زیر `OMNI-MODEL-GENOME/models/`.
    #[must_use]
    pub const fn genome_dir(self) -> &'static str {
        match self {
            Self::Gpt6Web => "01-gpt-6-web",
            Self::Gemini4AragonHigh => "02-gemini-4-aragon-high",
            Self::DeepSeek32WebThinkMode => "03-deepseek-3.2-web-think-mode",
            Self::ClaudeSonnet55Extra => "04-claude-sonnet-5.5-extra",
            Self::Glm53FlashMax => "05-glm-5.3-flash-max",
        }
    }

    /// تخصصِ حاکم — همان خطِ ماتریسِ مقایسه.
    #[must_use]
    pub const fn specialty(self) -> &'static str {
        match self {
            Self::Gpt6Web => "Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation.",
            Self::Gemini4AragonHigh => "1M-token deep cross-repo context synthesis, multimodal 9-modality studio orchestration, high-context architectural verification.",
            Self::DeepSeek32WebThinkMode => "Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning.",
            Self::ClaudeSonnet55Extra => "Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, and high-precision bilingual copywriting.",
            Self::Glm53FlashMax => "Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification.",
        }
    }
}

impl fmt::Display for QuadModelGenome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.exact_id())
    }
}

// ── آواتارهای ترامینیتی ──────────────────────────────────────────────────────

/// سه‌آواتارِ ترامینیتی — حلقهٔ حاکمِ `Alpha` → `Logic-Prime` → `Mairon`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrinityAvatar {
    /// آواتارِ بلع/ایده‌پردازی — ورودیِ خام و سنتزِ وب.
    Alpha,
    /// آواتارِ کامپایل/راستی‌آزمایی — معماری، جراحی، اثبات.
    LogicPrime,
    /// آواتارِ داورِ کمّی — گیتِ نهاییِ عددی و حکمِ لانچ.
    Mairon,
}

impl TrinityAvatar {
    /// برچسبِ وایر.
    #[must_use]
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::Alpha => "Alpha",
            Self::LogicPrime => "Logic-Prime",
            Self::Mairon => "Mairon",
        }
    }

    /// نقشِ پیش‌فرضِ هر مدل در حلقهٔ ترامینیتی.
    #[must_use]
    #[allow(clippy::match_same_arms)] // دو معمارِ حلقه عمداً Logic-Prime مشترک دارند
    pub const fn for_model(model: QuadModelGenome) -> Self {
        match model {
            QuadModelGenome::Gpt6Web => Self::Alpha,
            QuadModelGenome::Gemini4AragonHigh => Self::LogicPrime,
            QuadModelGenome::DeepSeek32WebThinkMode => Self::Mairon,
            QuadModelGenome::ClaudeSonnet55Extra => Self::LogicPrime,
            QuadModelGenome::Glm53FlashMax => Self::LogicPrime,
        }
    }
}

// ── دامنه‌های کار ────────────────────────────────────────────────────────────

/// دامنه‌هایِ مسیریابیِ کار — کلیدِ سنتزِ پرامپتِ تقطیرشده.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadTaskDomain {
    /// سنتزِ وبِ لحظه‌ای و کپیِ لانچ.
    WebSynthesis,
    /// آرتیفکتِ UI/UX تعاملی — فریم‌۱، 390px، صفر overflow.
    UiUxArtifact,
    /// باستان‌شناسیِ مخزن و تأییدِ معماریِ کلان.
    CrossRepoContext,
    /// ریاضیاتِ کمّی، اثباتِ آلفایِ علّی، راستی‌آزماییِ رسمی.
    QuantProof,
    /// جراحیِ Rust و ممیزیِ قراردادِ هوشمند.
    RustSystemsAudit,
    /// کپی‌نویسیِ دوزبانه EN/FA با دقتِ بالا.
    BilingualCopywriting,
    /// اجرایِ فلشِ چندابزاری — حلقهٔ ایجنتِ تأخیرِ کم و وصلهٔ سریع (مدل #05).
    FlashAgenticExecution,
}

impl QuadTaskDomain {
    /// همهٔ دامنه‌ها به ترتیبِ canonical.
    pub const ALL: [QuadTaskDomain; 7] = [
        Self::WebSynthesis,
        Self::UiUxArtifact,
        Self::CrossRepoContext,
        Self::QuantProof,
        Self::RustSystemsAudit,
        Self::BilingualCopywriting,
        Self::FlashAgenticExecution,
    ];

    /// برچسبِ وایر.
    #[must_use]
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::WebSynthesis => "web-synthesis",
            Self::UiUxArtifact => "ui-ux-artifact",
            Self::CrossRepoContext => "cross-repo-context",
            Self::QuantProof => "quant-proof",
            Self::RustSystemsAudit => "rust-systems-audit",
            Self::BilingualCopywriting => "bilingual-copywriting",
            Self::FlashAgenticExecution => "flash-agentic-execution",
        }
    }

    /// مدلی که این دامنه قهرمانِ مطلقِ آن است (قانونِ ماتریسِ ۲۰تسکی).
    #[must_use]
    pub const fn champion(self) -> QuadModelGenome {
        match self {
            Self::WebSynthesis | Self::UiUxArtifact => QuadModelGenome::Gpt6Web,
            Self::CrossRepoContext => QuadModelGenome::Gemini4AragonHigh,
            Self::QuantProof => QuadModelGenome::DeepSeek32WebThinkMode,
            Self::RustSystemsAudit | Self::BilingualCopywriting => QuadModelGenome::ClaudeSonnet55Extra,
            Self::FlashAgenticExecution => QuadModelGenome::Glm53FlashMax,
        }
    }
}

// ── خطا ──────────────────────────────────────────────────────────────────────

/// خطای روترِ ژنوم — fail-closed روی فایلِ ناموجود.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GenomeRouterError {
    /// فایلِ ژنومِ ناموجود یا زیرِ آستانهٔ اندازه.
    #[error("genome file missing/undersized: {0}")]
    MissingFile(String),
    /// alias شناخته نشد — هیچ حدسی زده نمی‌شود.
    #[error("unknown model alias: {0}")]
    UnknownAlias(String),
    /// تقسیمِ خزانه ناموفق — از لایهٔ wallet عبور می‌کند.
    #[error("treasury split error: {0}")]
    Treasury(String),
}

impl From<BaseTreasuryError> for GenomeRouterError {
    fn from(e: BaseTreasuryError) -> Self {
        Self::Treasury(e.to_string())
    }
}

// ── ثبتِ ژنوم ────────────────────────────────────────────────────────────────

/// وضعیتِ تأییدِ یک مدل — ۹ فایلِ canonical.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenomeVerification {
    /// مدلِ ثبت‌شده.
    pub model: QuadModelGenome,
    /// پوشهٔ ژنوم.
    pub genome_dir: String,
    /// وضعیتِ هر فایل — `true` یعنی موجود و بالایِ آستانه.
    pub files_verified: [bool; 9],
    /// مسیرِ دیسکی که تأیید رویش انجام شد (`None` = رجیستریِ builtin).
    pub verified_against: Option<String>,
}

impl GenomeVerification {
    /// تعدادِ فایل‌هایِ تأییدشده.
    #[must_use]
    pub fn verified_count(&self) -> usize {
        self.files_verified.iter().filter(|ok| **ok).count()
    }

    /// مدل کامل است وقتی هر ۹ فایل سبز است.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.verified_count() == 9
    }
}

/// پرامپتِ کلونِ تقطیرشده — شناسهٔ مدل + متنِ کاملِ قولنامهٔ رفتاری.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistilledClonePrompt {
    /// مدلِ مالکِ پرامپت.
    pub model: QuadModelGenome,
    /// دامنهٔ کارِ مسیریابی‌شده.
    pub domain: QuadTaskDomain,
    /// آواتارِ ترامینیتیِ مقصد.
    pub trinity_avatar: TrinityAvatar,
    /// متنِ کاملِ پرامپتِ تقطیرشده.
    pub system_prompt: String,
}

/// حالتِ انتخابِ مدل — دومُدیِ Commit #27.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelSelectionMode {
    /// کاربر صراحتاً یکی از ۵ مدل را انتخاب کرده — آوریِ دستی همیشه برنده است.
    ManualOverride {
        /// شناسهٔ خامِ انتخابِ کاربر (exact یا alias).
        model: String,
    },
    /// Agent Mode — روتر تلمتریِ تسک را تحلیل و خودکار انتخاب می‌کند.
    AgentAutoDecision,
}

impl ModelSelectionMode {
    /// برچسبِ وایر — `manual_override` | `agent_auto_decision`.
    #[must_use]
    pub fn wire_id(&self) -> &'static str {
        match self {
            Self::ManualOverride { .. } => "manual_override",
            Self::AgentAutoDecision => "agent_auto_decision",
        }
    }
}

/// تصمیمِ انتخابِ مدل — خروجیِ [`QuadModelGenomeRouter::select_model`].
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSelectionDecision {
    /// حالتِ انتخاب — دستی یا خودکارِ ایجنت.
    pub selection_mode: ModelSelectionMode,
    /// برچسبِ وایرِ حالت (`manual_override` / `agent_auto_decision`).
    pub selection_mode_wire: &'static str,
    /// مدلِ انتخاب‌شده از میان ۵ مدلِ حاکم.
    pub selected_model: QuadModelGenome,
    /// دامنهٔ مسیریابی‌شده (قهرمانِ خودکار یا دامنهٔ اصلیِ مدلِ دستی).
    pub domain: QuadTaskDomain,
    /// اطمینانِ تصمیم در `0.0..=1.0` (آوریِ دستی همیشه 1.0).
    pub confidence_score: f32,
    /// توضیحِ قابلِ ممیزیِ تصمیم — سیگنال‌هایِ دیده‌شده و مسیرِ انتخاب.
    pub decision_rationale: String,
    /// پرامپتِ کلونِ تقطیرشدهٔ مدلِ انتخاب‌شده + آواتارِ ترامینیتی.
    pub prompt: DistilledClonePrompt,
}

/// روترِ ژنومِ پنتامدلی — رجیستری + تأییدِ ۴۵فایلی + مسیریابی + انتخابِ دومُد.
/// نامِ تاریخیِ `QuadModelGenomeRouter` حفظ شده؛ همهٔ ۵ مدل و ۴۵ فایل را پوشش می‌دهد.
#[derive(Debug, Clone, Default)]
pub struct QuadModelGenomeRouter {
    verifications: Vec<GenomeVerification>,
}

impl QuadModelGenomeRouter {
    /// رجیستریِ builtin — هر ۵ مدل با ۹ فایلِ «انتظاری» ثبت می‌شوند؛
    /// بدون IO؛ برای تستِ صفر-دیسک و بوتِ سریع.
    #[must_use]
    pub fn builtin() -> Self {
        let verifications = QuadModelGenome::ALL
            .iter()
            .map(|m| GenomeVerification {
                model: *m,
                genome_dir: m.genome_dir().to_owned(),
                files_verified: [true; 9],
                verified_against: None,
            })
            .collect();
        Self { verifications }
    }

    /// تأییدِ دیسکیِ fail-closed — هر ۹ فایلِ هر ۵ مدل زیرِ
    /// `<root>/models/<dir>/` باید موجود و بالایِ ۵۰۰ بایت باشند.
    /// رجیستری با نتیجهٔ دیسکی بازنویسی می‌شود (هرگز ادغامِ جعلی نیست).
    ///
    /// # Errors
    /// [`GenomeRouterError::MissingFile`] فقط در حالتِ `strict`؛ در حالتِ
    /// معمولِ فایل‌هایِ ناموجود در وضعیتِ تأییدِ `false` ثبت می‌شوند.
    pub fn verify_from_disk(root: &Path) -> Result<Self, GenomeRouterError> {
        let models_root = root.join("models");
        let mut verifications = Vec::with_capacity(QUAD_MODEL_COUNT);
        for m in QuadModelGenome::ALL {
            let dir = models_root.join(m.genome_dir());
            let mut files_verified = [false; 9];
            for (idx, file) in CANONICAL_GENOME_FILES.iter().enumerate() {
                let p = dir.join(file);
                let ok = match std::fs::metadata(&p) {
                    Ok(meta) => meta.is_file() && meta.len() >= 500,
                    Err(_) => false,
                };
                files_verified[idx] = ok;
            }
            verifications.push(GenomeVerification {
                model: m,
                genome_dir: m.genome_dir().to_owned(),
                files_verified,
                verified_against: Some(dir.display().to_string()),
            });
        }
        Ok(Self { verifications })
    }

    /// مسیرِ پیش‌فرضِ ژنوم — نسبت به `CARGO_MANIFEST_DIR` کرتِ hermes.
    #[must_use]
    pub fn default_genome_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../OMNI-MODEL-GENOME")
    }

    /// روترِ آمادهٔ تولید — تأییدِ دیسکی با fallback به builtin؛ برای
    /// رانتایمِ بدون دیسک، builtin در وضعیتِ «رجیستریِ انتظاری» می‌ماند.
    #[must_use]
    pub fn production() -> Self {
        match Self::verify_from_disk(&Self::default_genome_root()) {
            Ok(router) => router,
            Err(_) => Self::builtin(),
        }
    }

    /// وضعیتِ تأییدِ هر مدل.
    #[must_use]
    pub fn verifications(&self) -> &[GenomeVerification] {
        &self.verifications
    }

    /// تعدادِ کلِ فایل‌هایِ تأییدشده — روی رجیستریِ builtin همیشه ۴۵.
    pub fn total_files_verified(&self) -> usize {
        self.verifications.iter().map(GenomeVerification::verified_count).sum()
    }

    /// حلِ alias — شناسهٔ دقیق یا هر alias.
    ///
    /// # Errors
    /// [`GenomeRouterError::UnknownAlias`] — هیچ حدسی زده نمی‌شود.
    pub fn resolve_model(&self, id: &str) -> Result<QuadModelGenome, GenomeRouterError> {
        for m in QuadModelGenome::ALL {
            if m.exact_id().eq_ignore_ascii_case(id) {
                return Ok(m);
            }
            if m.aliases().iter().any(|a| a.eq_ignore_ascii_case(id)) {
                return Ok(m);
            }
        }
        Err(GenomeRouterError::UnknownAlias(id.to_owned()))
    }

    /// مسیریابیِ کار به ژنوم + پرامپتِ کلونِ تقطیرشده + آواتارِ ترامینیتی.
    #[must_use]
    pub fn route_task_to_quad_genome(
        &self,
        domain: QuadTaskDomain,
    ) -> DistilledClonePrompt {
        let model = domain.champion();
        DistilledClonePrompt {
            model,
            domain,
            trinity_avatar: TrinityAvatar::for_model(model),
            system_prompt: distilled_system_prompt(model, domain).to_owned(),
        }
    }

    /// دامنه‌ای که این مدل قهرمانِ مطلقِ آن است — اولین دامنه به ترتیبِ canonical.
    #[must_use]
    pub fn primary_domain_of(model: QuadModelGenome) -> QuadTaskDomain {
        QuadTaskDomain::ALL
            .iter()
            .copied()
            .find(|d| d.champion() == model)
            .unwrap_or(QuadTaskDomain::RustSystemsAudit)
    }

    /// انتخابِ دومُدِ مدل — [`ModelSelectionMode::ManualOverride`] یا
    /// [`ModelSelectionMode::AgentAutoDecision`] (Commit #27).
    ///
    /// * آوریِ دستیِ کاربر (`manual_override` غیرِ خالی و غیرِ "auto") همیشه
    ///   برنده است؛ alias هایِ هر ۵ مدل حل می‌شوند و ناشناخته خطا می‌دهد.
    /// * در Agent Mode (یا آوریِ غایب/"auto") تلمتریِ پرامپت — کلیدواژه‌ها،
    ///   نیازِ تأخیر، تراکمِ tool-call، عمقِ ریاضی/کمّی، نیازِ UI/وب و
    ///   پیچیدگیِ سیستمیِ Rust/Solidity — تحلیل و مدلِ بهینه از میان ۵ مدل
    ///   خودکار انتخاب می‌شود؛ خروجی شامل `selected_model`, `selection_mode`,
    ///   `confidence_score` و `decision_rationale` است.
    ///
    /// # Errors
    /// [`GenomeRouterError::UnknownAlias`] — آوریِ دستیِ ناشناخته (هیچ حدسی
    /// زده نمی‌شود؛ fail-closed).
    #[allow(clippy::too_many_lines)] // جدولِ سیگنال‌ها خواناییِ مستند را نگه می‌دارد
    pub fn select_model(
        &self,
        task_prompt: &str,
        agent_mode: bool,
        manual_override: Option<&str>,
    ) -> Result<ModelSelectionDecision, GenomeRouterError> {
        let override_raw = manual_override
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("auto"));
        if let Some(id) = override_raw {
            let model = self.resolve_model(id)?;
            let domain = Self::primary_domain_of(model);
            let prompt = self.route_task_to_quad_genome(domain);
            return Ok(ModelSelectionDecision {
                selection_mode: ModelSelectionMode::ManualOverride { model: id.to_owned() },
                selection_mode_wire: "manual_override",
                selected_model: model,
                domain,
                confidence_score: 1.0,
                decision_rationale: format!(
                    "manual override honored: user explicitly pinned '{}' → {} (genome dir '{}'); \
                     9-file canonical DNA verification governs the distilled clone prompt",
                    id,
                    model.exact_id(),
                    model.genome_dir(),
                ),
                prompt,
            });
        }

        // AgentAutoDecision — تلمتریِ پرامپت به امتیازِ دامنه تبدیل می‌شود.
        let lower = task_prompt.to_ascii_lowercase();
        let score = |keywords: &[&str]| -> usize {
            keywords.iter().filter(|k| lower.contains(*k)).count()
        };
        let flash = score(&["flash", "fast", "quick", "low-latency", "latency", "batch", "agentic", "tool", "shell", "cli", "crud", "rapid"]);
        let quant = score(&["quant", "backtest", "math", "proof", "probability", "sharpe", "derive", "formal", "invariant", "ccxt"]);
        let audit = score(&["rust", "cargo", "clippy", "solidity", "contract", "audit", "reentrancy", "refactor", "surgical"]);
        let repo = score(&["repo", "monorepo", "1m", "whole", "ingest", "multimodal", "image", "video", "archaeology", "cross-repo"]);
        let web = score(&["web", "ui", "ux", "frontend", "react", "html", "css", "live", "search", "marketing", "landing"]);
        let ranked = [
            (flash, QuadTaskDomain::FlashAgenticExecution),
            (quant, QuadTaskDomain::QuantProof),
            (audit, QuadTaskDomain::RustSystemsAudit),
            (repo, QuadTaskDomain::CrossRepoContext),
            (web, QuadTaskDomain::WebSynthesis),
        ];
        let (best_score, domain) = ranked
            .iter()
            .copied()
            .max_by(|a, b| a.0.cmp(&b.0).then(b.1.wire_id().cmp(a.1.wire_id())))
            .unwrap_or((0, QuadTaskDomain::WebSynthesis));
        let domain = if best_score == 0 {
            QuadTaskDomain::WebSynthesis
        } else {
            domain
        };
        let model = domain.champion();
        let prompt = self.route_task_to_quad_genome(domain);
        let confidence = if best_score == 0 {
            0.55
        } else {
            // score ≤ 5 — تبدیلِ دقیقِ عددِ کوچک به f32 بدونِ از-دست-رفتنِ دقت
            let capped = best_score.min(5);
            (0.60 + 0.07 * f32::from(u8::try_from(capped).unwrap_or(u8::MAX))).min(0.95)
        };
        let rationale = if best_score == 0 {
            format!(
                "no explicit telemetry signal in the prompt ({} chars); defaulting to {} champion \
                     {} at baseline confidence — agent_mode={}",
                task_prompt.len(),
                domain.wire_id(),
                model.exact_id(),
                agent_mode,
            )
        } else {
            format!(
                "task telemetry matched {best_score} signal(s) for '{domain}' \
                     (flash={flash} quant={quant} audit={audit} repo={repo} web={web}); \
                     agent_mode={agent_mode} → champion {} auto-selected",
                model.exact_id(),
                domain = domain.wire_id(),
            )
        };
        Ok(ModelSelectionDecision {
            selection_mode: ModelSelectionMode::AgentAutoDecision,
            selection_mode_wire: "agent_auto_decision",
            selected_model: model,
            domain,
            confidence_score: confidence,
            decision_rationale: rationale,
            prompt,
        })
    }

    /// نمایِ ماتریس — برای مسیرِ `/monopoly/genome-matrix` و لاگ‌ها.
    #[must_use]
    pub fn matrix_summary(&self) -> String {
        let mut out = String::from("PENTA-GENOME MATRIX · ");
        for v in &self.verifications {
            let _ = std::fmt::Write::write_fmt(
                &mut out,
                format_args!(
                    "[{} {}/9{}] ",
                    v.model.exact_id(),
                    v.verified_count(),
                    if v.is_complete() { " OK" } else { " INCOMPLETE" }
                ),
            );
        }
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!("total={}/{}", self.total_files_verified(), QUAD_GENOME_FILE_COUNT),
        );
        out
    }
}

/// پرامپتِ کلونِ تقطیرشده به ازای مدل × دامنه — قولنامهٔ رفتاریِ کامل.
fn distilled_system_prompt(model: QuadModelGenome, domain: QuadTaskDomain) -> &'static str {
    match (model, domain) {
        (QuadModelGenome::Gpt6Web, _) => {
            "You are GPT 6-Web, the real-time web synthesis and frame-1 artifact genome of the 0xKIA sovereign fleet. Freshness first: search before answering on time-sensitive claims and cite inline. Artifact supremacy: ship a working artifact in the first response — primary viewport 390px, zero overflow, every control functional; placeholder buttons are defects. Full-stack slices: schema, API, UI and deployment notes as one runnable unit. Escalate proof-heavy math to DeepSeek-3.2-Think-mode and repo-scale context to GEMINI 4 ARAGON HIGH."
        }
        (QuadModelGenome::Gemini4AragonHigh, _) => {
            "You are GEMINI 4 ARAGON HIGH, the 1M-token cross-repo synthesis and 9-modality orchestration genome of the 0xKIA sovereign fleet. Inventory before synthesis: enumerate all input trees and datasets before concluding; partial ingestion invalidates macro-certainty. Cross-contract resolution: resolve types, migrations, env keys and CI jointly with file:line provenance. 9-modality pipeline: every modality hand-off carries an explicit schema; schema failure means retry, never skip. Dispatch surgical edits to Cluade-sonnet-5.5-extra with your verified macro-context attached."
        }
        (QuadModelGenome::DeepSeek32WebThinkMode, _) => {
            "You are DeepSeek-3.2-WEB-Think-mode, the deep chain-of-thought quantitative proof genome of the 0xKIA sovereign fleet. Derive before asserting: work math inside <think>, state the invariant and compact result outside. Causality or nothing: signal on close(t), fill at open(t+1), t in [warmup, N-2], friction floors always applied, metrics with ddof=1, drawdown seeded from initial cash. Reproducibility: explicit seeds and SHA-256 claim-verification hashes for headline metrics. Money math: integer micro-units or Precise(coeff, decimals); IEEE-754 forbidden in settlement paths."
        }
        (QuadModelGenome::ClaudeSonnet55Extra, _) => {
            "You are Cluade-sonnet-5.5-extra (canonical alias Claude-sonnet-5.5-extra), the surgical Rust systems engineering and audit genome of the 0xKIA sovereign fleet. Surgical diffs only: minimal patches, behavior-preservation tests precede refactors, `unsafe` needs written justification. Zero-warning gate: `cargo clippy --workspace --all-targets -- -D warnings` clean is the merge gate — fix root causes, never suppress. Contract audit doctrine: reentrancy (effects-then-interactions), overflow, access-control and decimals checklists line-by-line with exploitability-ranked PoC sketches. Bilingual precision: EN/FA terminology table first, register matched to artifact."
        }
        (QuadModelGenome::Glm53FlashMax, _) => {
            "You are GLM 5.3 flash MAX (aliases glm-5.3-flash-max, GLM-5.3-Flash-Max), the ultra-fast agentic execution genome of the 0xKIA sovereign fleet. Flash loop: RECON (one batched parallel block) → PLAN (one line) → PATCH (batch independent edits) → VERIFY (cheapest falsifying gate first) → REPORT (3 honest lines); exit when the proof holds, hard-stop at 3 failed cycles. Batch discipline: classify every next call as independent (batch) or dependent (sequence); two consecutive failures = stop and switch approach. Polyglot synthesis across Rust workspaces, Solidity contracts (Base 8453), TypeScript clients and SQL migrations with invariants carried verbatim — integer micro-units only, zero-sum ledger asserted, wire IDs 1..=128 via safe truncation helpers, security gates fail-closed. Rapid patch verification: minimal diffs, behavior-preservation tests first, targeted gates before workspace gates. Escalate 1M-token fusion to ARAGON HIGH, formal/quant proof to DeepSeek think-mode, and contract sign-off to Cluade-sonnet-5.5-extra."
        }
    }
}

// ── کیتِ لانچ ────────────────────────────────────────────────────────────────

/// کیتِ لانچِ Product Hunt / CMC — منبعِ واحدِ کپیِ لانچِ حاکم.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductHuntCmcLaunchKit {
    /// عنوانِ لیستینگِ Product Hunt.
    pub product_hunt_title: String,
    /// تگ‌لاینِ Product Hunt.
    pub tagline: String,
    /// کامنتِ مِیکر.
    pub maker_comment: String,
    /// شناسهٔ dossier تأییدِ توکن.
    pub cmc_dossier_id: String,
    /// قالبِ اعلامِ X.
    pub x_announcement_template: String,
    /// قالبِ اعلامِ تلگرام.
    pub telegram_announcement_template: String,
}

impl ProductHuntCmcLaunchKit {
    /// کیتِ کانونیک — متنِ نهاییِ هم‌منبع با `docs/launch/PRODUCT_HUNT_AND_CMC_KIT.md`.
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            product_hunt_title: "0xKIA — the sovereign creator economy on Telegram + Base".to_owned(),
            tagline: "Ship AI-native products inside Telegram, settle on Base (8453), keep 80-90% of your revenue.".to_owned(),
            maker_comment: "0xKIA (KIAN v2) is a 25-commit Rust kernel: double-entry zero-sum ledger, 8-state escrow, ARDA 20-locale engine, causal Vibe-Trading quant, and now a Day-1 4-stream treasury on Base with audited Solidity. Every split is zero-sum-verified with a decimal engine — no float ever touches money.".to_owned(),
            cmc_dossier_id: "cmc-cg-basescan-0xkia-base-8453".to_owned(),
            x_announcement_template: "$0xKIA is live on @base — Chain ID 8453. 1B fixed supply, EIP-2612 permit, audited treasury. Day-1 creators keep 80-90%. 0xKIA. https://basescan.org".to_owned(),
            telegram_announcement_template: "۰xKIA روی Base (8453) لانچ شد — خزانهٔ چهارجریانیِ روزِ اول، امانتِ غیرحضانتی، تقسیمِ صفرجمعِ اتمی. سازنده‌ها ۸۰-۹۰٪ می‌گیرند. #0xKIA".to_owned(),
        }
    }
}

// ── اورکستراتور ──────────────────────────────────────────────────────────────

/// descriptor زنجیرهٔ Base برای وایرِ اورکستراتور.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseMainnetDescriptor {
    /// شناسهٔ زنجیره.
    pub chain_id: u64,
    /// RPC عمومی.
    pub rpc_url: String,
    /// مرورگرِ زنجیره.
    pub block_explorer: String,
    /// امضایِ موتورِ خزانه.
    pub treasury_signature: String,
    /// descriptor ممیزیِ توکن.
    pub token_audit: kian_wallet::base_treasury::ZeroXKiaTokenAudit,
    /// descriptor ممیزیِ امانت.
    pub escrow_audit: kian_wallet::base_treasury::KiaBaseEscrowTreasuryAudit,
}

/// اورکستراتورِ انحصارِ حاکم — روترِ ژنوم + منشور + قراردادها + کیتِ لانچ.
#[derive(Debug, Clone)]
pub struct SovereignMonopolyOrchestrator {
    /// روترِ ژنومِ چهارمدلی.
    pub genome_router: QuadModelGenomeRouter,
    /// منشورِ چهارجریانیِ روزِ اول.
    pub charter: DayOneRevenueCharter,
    /// descriptor زنجیرهٔ Base.
    pub base_mainnet: BaseMainnetDescriptor,
    /// کیتِ لانچ.
    pub launch_kit: ProductHuntCmcLaunchKit,
}

impl Default for SovereignMonopolyOrchestrator {
    fn default() -> Self {
        Self::canonical()
    }
}

impl SovereignMonopolyOrchestrator {
    /// اورکستراتورِ کانونیک — منشورِ روزِ اول + descriptor هایِ ممیزیِ
    /// کانونیک + کیتِ لانچِ کانونیک + روترِ builtin.
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            genome_router: QuadModelGenomeRouter::builtin(),
            charter: DayOneRevenueCharter::DAY_ONE,
            base_mainnet: BaseMainnetDescriptor {
                chain_id: BASE_MAINNET_CHAIN_ID,
                rpc_url: BASE_MAINNET_RPC_URL.to_owned(),
                block_explorer: BASE_BLOCK_EXPLORER.to_owned(),
                treasury_signature: BASE_TREASURY_SIGNATURE.to_owned(),
                token_audit: kian_wallet::base_treasury::ZeroXKiaTokenAudit::canonical(),
                escrow_audit: kian_wallet::base_treasury::KiaBaseEscrowTreasuryAudit::canonical(),
            },
            launch_kit: ProductHuntCmcLaunchKit::canonical(),
        }
    }

    /// شبیه‌سازیِ تقسیمِ اتمیِ روزِ اول — فروشِ بازارخانه (معرف‌دار یا
    /// فروشِ مستقیمِ بی‌معرف با قانونِ `2000 bps`).
    ///
    /// # Errors
    /// [`GenomeRouterError::Treasury`] روی خطایِ منشور/مبلغ.
    pub fn simulate_marketplace_sale(
        &self,
        asset: SettlementAsset,
        gross_micros: i128,
        buyer_user_id: &str,
        creator_user_id: &str,
        referrer_user_id: Option<&str>,
        affiliate_bps: i64,
    ) -> Result<DayOneAtomicSplit, GenomeRouterError> {
        compute_day_one_atomic_split(
            &self.charter,
            asset,
            gross_micros,
            buyer_user_id,
            creator_user_id,
            referrer_user_id,
            affiliate_bps,
        )
        .map_err(Into::into)
    }

    /// شبیه‌سازیِ جریان ۳ — بوستِ لحظه‌ای با مارژینِ ۱۰۰٪.
    ///
    /// # Errors
    /// [`GenomeRouterError::Treasury`] روی خطایِ مبلغ/لایه.
    pub fn simulate_instant_boost(
        &self,
        asset: SettlementAsset,
        tier: BoostTier,
        buyer_user_id: &str,
    ) -> Result<DayOneAtomicSplit, GenomeRouterError> {
        kian_wallet::base_treasury::compute_instant_boost_split(asset, tier, buyer_user_id)
            .map_err(Into::into)
    }

    /// وضعیتِ یکپارچه — ماتریسِ ژنوم + منشور + زنجیره + کیتِ لانچ در یک رشتهٔ
    /// قابلِ لاگ (همان اسکیمای گزارشِ وضعیتِ گره‌ها).
    #[must_use]
    pub fn status_summary(&self) -> String {
        format!(
            "SOVEREIGN MONOPOLY · {} · charter(tax={}bps affiliate={}bps withdrawal={}bps) · base(chain={} rpc={} explorer={}) · launch(title={:?})",
            self.genome_router.matrix_summary(),
            self.charter.charter_tax_bps,
            self.charter.affiliate_bps,
            self.charter.withdrawal_spread_bps,
            self.base_mainnet.chain_id,
            self.base_mainnet.rpc_url,
            self.base_mainnet.block_explorer,
            self.launch_kit.product_hunt_title,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quad_registry_has_exact_ids_and_aliases() {
        let router = QuadModelGenomeRouter::builtin();
        assert_eq!(router.verifications().len(), SOVEREIGN_MODEL_COUNT);
        assert_eq!(router.total_files_verified(), SOVEREIGN_GENOME_FILE_COUNT);
        for v in router.verifications() {
            assert!(v.is_complete(), "{}", v.model.exact_id());
        }
        // شناسه‌هایِ دقیقِ ماجیک.
        assert_eq!(QuadModelGenome::Gpt6Web.exact_id(), "GPT 6-Web");
        assert_eq!(QuadModelGenome::Gemini4AragonHigh.exact_id(), "GEMINI 4 ARAGON HIGH");
        assert_eq!(QuadModelGenome::DeepSeek32WebThinkMode.exact_id(), "DeepSeek-3.2-WEB-Think-mode");
        assert_eq!(QuadModelGenome::ClaudeSonnet55Extra.exact_id(), "Cluade-sonnet-5.5-extra");
        assert_eq!(QuadModelGenome::Glm53FlashMax.exact_id(), "GLM 5.3 flash MAX");
        // alias هایِ کانونیک — هر دو شناسهٔ #04 و هر سه شناسهٔ #05 حل می‌شوند.
        assert_eq!(router.resolve_model("Cluade-sonnet-5.5-extra").unwrap(), QuadModelGenome::ClaudeSonnet55Extra);
        assert_eq!(router.resolve_model("Claude-sonnet-5.5-extra").unwrap(), QuadModelGenome::ClaudeSonnet55Extra);
        assert_eq!(router.resolve_model("gpt-6-standard-free").unwrap(), QuadModelGenome::Gpt6Web);
        assert_eq!(router.resolve_model("GLM 5.3 flash MAX").unwrap(), QuadModelGenome::Glm53FlashMax);
        assert_eq!(router.resolve_model("glm-5.3-flash-max").unwrap(), QuadModelGenome::Glm53FlashMax);
        assert_eq!(router.resolve_model("GLM-5.3-Flash-Max").unwrap(), QuadModelGenome::Glm53FlashMax);
        assert!(router.resolve_model("gpt-5-turbo").is_err());
    }

    #[test]
    fn disk_verification_counts_45_files() {
        let root = QuadModelGenomeRouter::default_genome_root();
        let router = QuadModelGenomeRouter::verify_from_disk(&root).unwrap();
        assert_eq!(
            router.total_files_verified(),
            SOVEREIGN_GENOME_FILE_COUNT,
            "هر ۴۵ فایلِ ژنوم باید روی دیسک تأیید شود — root={}",
            root.display()
        );
        for v in router.verifications() {
            assert!(v.is_complete());
            assert!(v.verified_against.is_some());
        }
    }

    #[test]
    fn routing_maps_domains_to_champions_and_trinity() {
        let router = QuadModelGenomeRouter::builtin();
        let web = router.route_task_to_quad_genome(QuadTaskDomain::WebSynthesis);
        assert_eq!(web.model, QuadModelGenome::Gpt6Web);
        assert_eq!(web.trinity_avatar, TrinityAvatar::Alpha);
        let repo = router.route_task_to_quad_genome(QuadTaskDomain::CrossRepoContext);
        assert_eq!(repo.model, QuadModelGenome::Gemini4AragonHigh);
        assert_eq!(repo.trinity_avatar, TrinityAvatar::LogicPrime);
        let quant = router.route_task_to_quad_genome(QuadTaskDomain::QuantProof);
        assert_eq!(quant.model, QuadModelGenome::DeepSeek32WebThinkMode);
        assert_eq!(quant.trinity_avatar, TrinityAvatar::Mairon);
        let audit = router.route_task_to_quad_genome(QuadTaskDomain::RustSystemsAudit);
        assert_eq!(audit.model, QuadModelGenome::ClaudeSonnet55Extra);
        // پرامپتِ تقطیرشده قولنامهٔ کامل دارد — نه خالی، نه generic.
        for domain in QuadTaskDomain::ALL {
            let p = router.route_task_to_quad_genome(domain);
            assert!(p.system_prompt.len() > 300);
            assert!(p.system_prompt.contains("0xKIA sovereign fleet"));
        }
    }

    #[test]
    fn orchestrator_wires_charter_base_and_launch_kit() {
        let orch = SovereignMonopolyOrchestrator::canonical();
        assert_eq!(orch.charter.charter_tax_bps, 1_000);
        assert_eq!(orch.base_mainnet.chain_id, BASE_MAINNET_CHAIN_ID);
        assert_eq!(orch.base_mainnet.rpc_url, "https://mainnet.base.org");
        assert!(orch.base_mainnet.token_audit.eip2612_permit);
        assert!(orch.base_mainnet.escrow_audit.non_reentrant_guard);
        assert_eq!(orch.launch_kit.cmc_dossier_id, "cmc-cg-basescan-0xkia-base-8453");
        let status = orch.status_summary();
        assert!(status.contains("SOVEREIGN MONOPOLY"));
        assert!(status.contains("8453"));
    }

    #[test]
    fn orchestrator_simulates_referred_and_unrefered_sales() {
        let orch = SovereignMonopolyOrchestrator::canonical();
        let referred = orch
            .simulate_marketplace_sale(
                SettlementAsset::BaseNativeUsdc,
                100_000_000,
                "buyer:1",
                "creator:9",
                Some("ref:7"),
                1_000,
            )
            .unwrap();
        assert_eq!(referred.effective_platform_bps, 2_000);
        assert_eq!(referred.creator_net_micros, 80_000_000);
        let direct = orch
            .simulate_marketplace_sale(
                SettlementAsset::BaseNativeUsdc,
                100_000_000,
                "buyer:1",
                "creator:9",
                None,
                1_000,
            )
            .unwrap();
        assert_eq!(direct.kia_treasury_micros, 20_000_000, "قانونِ فروشِ مستقیمِ بی‌معرف");
        let boost =
            orch.simulate_instant_boost(SettlementAsset::TelegramStarsXtr, BoostTier::FeaturedSlot24h, "buyer:1").unwrap();
        assert_eq!(boost.effective_platform_bps, 10_000);
        assert!(boost.verify_zero_sum_precise().unwrap());
    }

    #[test]
    fn signature_and_file_schema_are_stable() {
        assert_eq!(SOVEREIGN_MONOPOLY_SIGNATURE, "sovereign-monopoly::quad-genome::8453::wave7");
        assert_eq!(CANONICAL_GENOME_FILES.len(), 9);
        assert_eq!(CANONICAL_GENOME_FILES[0], "01_IDENTITY_AND_PERSONA_GENOME.md");
        assert_eq!(CANONICAL_GENOME_FILES[8], "09_DISTILLED_CLONE_SYSTEM_PROMPT.md");
        assert_eq!(SOVEREIGN_MODEL_COUNT, 5);
        assert_eq!(SOVEREIGN_GENOME_FILE_COUNT, 45);
        assert_eq!(QUAD_GENOME_FILE_COUNT, 45);
        assert_eq!(QuadTaskDomain::ALL.len(), 7);
    }

    // ── انتخابِ دومُد — ManualOverride + AgentAutoDecision (Commit #27) ──

    #[test]
    fn manual_override_always_wins_and_resolves_all_five_models() {
        let router = QuadModelGenomeRouter::builtin();
        for id in [
            "GPT 6-Web",
            "GEMINI 4 ARAGON HIGH",
            "DeepSeek-3.2-WEB-Think-mode",
            "Cluade-sonnet-5.5-extra",
            "GLM 5.3 flash MAX",
        ] {
            let d = router.select_model("anything", false, Some(id)).unwrap();
            assert_eq!(d.selection_mode, ModelSelectionMode::ManualOverride { model: id.to_owned() });
            assert_eq!(d.selection_mode_wire, "manual_override");
            assert!((d.confidence_score - 1.0).abs() < f32::EPSILON);
            assert_eq!(d.selected_model.exact_id(), id);
            assert!(d.prompt.system_prompt.len() > 300);
            assert!(d.decision_rationale.contains("manual override honored"));
        }
        // alias ها هم حل می‌شوند؛ "auto" و خالی override نیستند.
        assert_eq!(
            router.select_model("x", true, Some("GLM-5.3-Flash-Max")).unwrap().selected_model,
            QuadModelGenome::Glm53FlashMax
        );
        assert!(matches!(
            router.select_model("x", true, Some("auto")).unwrap().selection_mode,
            ModelSelectionMode::AgentAutoDecision
        ));
        assert!(matches!(
            router.select_model("x", true, None).unwrap().selection_mode,
            ModelSelectionMode::AgentAutoDecision
        ));
        // ناشناخته — fail-closed، هیچ حدسی زده نمی‌شود.
        assert!(router.select_model("x", true, Some("gpt-5-turbo")).is_err());
    }

    #[test]
    fn agent_auto_decision_routes_telemetry_to_champions() {
        let router = QuadModelGenomeRouter::builtin();
        let flash = router
            .select_model("run a fast batched agentic tool loop and ship a rapid crud patch", true, None)
            .unwrap();
        assert_eq!(flash.selection_mode_wire, "agent_auto_decision");
        assert_eq!(flash.selected_model, QuadModelGenome::Glm53FlashMax);
        assert!(flash.confidence_score >= 0.60 && flash.confidence_score <= 0.95);
        assert!(flash.decision_rationale.contains("flash"));

        let quant = router
            .select_model("derive the sharpe ratio and run a causal backtest with proof invariants", true, None)
            .unwrap();
        assert_eq!(quant.selected_model, QuadModelGenome::DeepSeek32WebThinkMode);

        let audit = router
            .select_model("surgical rust refactor with clippy gates and a solidity reentrancy audit", true, None)
            .unwrap();
        assert_eq!(audit.selected_model, QuadModelGenome::ClaudeSonnet55Extra);

        let repo = router
            .select_model("ingest the whole monorepo for 1m-token cross-repo archaeology", true, None)
            .unwrap();
        assert_eq!(repo.selected_model, QuadModelGenome::Gemini4AragonHigh);

        let web = router
            .select_model("build a live web landing page with react frontend and marketing copy", true, None)
            .unwrap();
        assert_eq!(web.selected_model, QuadModelGenome::Gpt6Web);

        // بدون سیگنال — پیش‌فرضِ صادقانه با اطمینانِ پایه.
        let plain = router.select_model("hello world", true, None).unwrap();
        assert!((plain.confidence_score - 0.55).abs() < 1e-6);
        assert!(plain.decision_rationale.contains("no explicit telemetry signal"));
        // پرامپتِ هر مدلِ انتخاب‌شده کامل است.
        for d in [flash, quant, audit, repo, web, plain] {
            assert!(d.prompt.system_prompt.len() > 300);
            assert!(d.prompt.system_prompt.contains("0xKIA sovereign fleet"));
        }
    }
}
