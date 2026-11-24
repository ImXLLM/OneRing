//! `omni_genome_router` — موتور ژنوم چندمدلی کیان (Wave 2 — Step 4/6).
//!
//! پورت `OMNI-MODEL-GENOME/` به کرنل Rust:
//! - هر مدل فرانتیر یک پوشهٔ ۹فایلی `01..09` دارد (`models/<model_id>/`).
//! - [`load_model_genome_from_dir`] اعتبارسنجی می‌کند که هر ۹ لایه موجودند،
//!   frontmatter YAML را می‌خواند و اسکورکارد ۲۰تسکی کارت ۰۹ را پارس می‌کند.
//! - [`builtin_gpt6_standard_profile`] و [`builtin_ecosystem_profiles`] همان
//!   قرارداد را بدون IO فراهم می‌کنند (تست‌های صفر-IO و رانتایم بدون دیسک).
//! - [`compile_prompt_for_model`] کامپایلر پرامپت هر مدل است: برای پروفایل‌های
//!   «بدون Think» (مثل `gpt-6-standard-free`) جدول پیش‌محاسبهٔ
//!   `<pre_computation_state_trace>` را قبل از هر کد تزریق می‌کند
//!   («ترتیب توکن = ترتیب فکر»)، گارد ضد-Canvas/ضد-وب‌سرچ می‌گذارد و
//!   invariant ها را در گیت‌های فازی ≤N قاعده می‌چیند + قرارداد ضد-elision.
//! - [`HandoffPacket`] پاکت تحویل قطعی بین‌مدلی — هم‌اسکیما با
//!   `KIA-BRAIN-OS/07_LINK_Protocol.md`.
//!
//! اصل اکتشافی: امتیاز/متریک نامعلوم هرگز از خودسازی مقدار نمی‌گیرد —
//! پروفایل‌های بدون کارت ۰۹ نقشهٔ امتیاز خالی می‌گیرند و مسیریابی به
//! فال‌بک (رجیستری صفر-کارت) می‌افتد.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

// ─────────────────────────── لایه‌های ژنوم ───────────────────────────

/// لایه‌های canonical ژنوم ۹فایلی — نام فایل‌ها قرارداد سخت است.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GenomeLayerId {
    TopologyAndHarness = 1,
    ReasoningAndLogicDna = 2,
    CodeAndPatchMechanics = 3,
    ToolCallingAndState = 4,
    UiUxAestheticBias = 5,
    GuardrailsAndFraming = 6,
    PromptCompilerLexicon = 7,
    FailureModesAndPatches = 8,
    RoutingAndHandoffCard = 9,
}

impl GenomeLayerId {
    /// همهٔ لایه‌ها به ترتیب canonical.
    pub const ALL: [GenomeLayerId; 9] = [
        Self::TopologyAndHarness,
        Self::ReasoningAndLogicDna,
        Self::CodeAndPatchMechanics,
        Self::ToolCallingAndState,
        Self::UiUxAestheticBias,
        Self::GuardrailsAndFraming,
        Self::PromptCompilerLexicon,
        Self::FailureModesAndPatches,
        Self::RoutingAndHandoffCard,
    ];

    /// نام فایل canonical (`01_TOPOLOGY_AND_HARNESS.md` .. `09_...`).
    #[must_use]
    pub const fn filename(self) -> &'static str {
        match self {
            Self::TopologyAndHarness => "01_TOPOLOGY_AND_HARNESS.md",
            Self::ReasoningAndLogicDna => "02_REASONING_AND_LOGIC_DNA.md",
            Self::CodeAndPatchMechanics => "03_CODE_AND_PATCH_MECHANICS.md",
            Self::ToolCallingAndState => "04_TOOL_CALLING_AND_STATE.md",
            Self::UiUxAestheticBias => "05_UI_UX_AND_AESTHETIC_BIAS.md",
            Self::GuardrailsAndFraming => "06_GUARDRAILS_AND_FRAMING.md",
            Self::PromptCompilerLexicon => "07_PROMPT_COMPILER_LEXICON.md",
            Self::FailureModesAndPatches => "08_FAILURE_MODES_AND_PATCHES.md",
            Self::RoutingAndHandoffCard => "09_ROUTING_AND_HANDOFF_CARD.md",
        }
    }
}

// ─────────────────────────── ۲۰ تسک مهندسی ───────────────────────────

/// بیست تسک مهندسی استاندارد اکوسیستم — ترتیب همان جدول کارت ۰۹.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EngineeringWorkload {
    SystemArchitecture,
    ComplexRustSystems,
    SurgicalMultiFileRefactor,
    HighCraftFrontendUiUx,
    WebGl3dVisualization,
    SqliteLedgerDesign,
    HardAlgorithmicDeduction,
    ConcurrencyBugTriage,
    SecuritySsrfHardening,
    SpecComplianceReview,
    MassiveRepoArchaeology,
    FastAgenticToolCalling,
    StrictJsonExtraction,
    AdversarialTestGeneration,
    TechnicalGenomeSynthesis,
    PersianRtlBilingualUx,
    TelegramMiniAppBotApi9,
    XAlgorithmViralGrowth,
    SubSecondIntentRouting,
    LongHorizonAutonomousLoop,
}

impl EngineeringWorkload {
    /// همهٔ ۲۰ تسک به ترتیب جدول routing.
    pub const ALL: [EngineeringWorkload; 20] = [
        Self::SystemArchitecture,
        Self::ComplexRustSystems,
        Self::SurgicalMultiFileRefactor,
        Self::HighCraftFrontendUiUx,
        Self::WebGl3dVisualization,
        Self::SqliteLedgerDesign,
        Self::HardAlgorithmicDeduction,
        Self::ConcurrencyBugTriage,
        Self::SecuritySsrfHardening,
        Self::SpecComplianceReview,
        Self::MassiveRepoArchaeology,
        Self::FastAgenticToolCalling,
        Self::StrictJsonExtraction,
        Self::AdversarialTestGeneration,
        Self::TechnicalGenomeSynthesis,
        Self::PersianRtlBilingualUx,
        Self::TelegramMiniAppBotApi9,
        Self::XAlgorithmViralGrowth,
        Self::SubSecondIntentRouting,
        Self::LongHorizonAutonomousLoop,
    ];

    /// برچسب خوانا برای گزارش روتینگ.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SystemArchitecture => "Zero-to-One System Architecture",
            Self::ComplexRustSystems => "Complex Rust Systems (async/CAS)",
            Self::SurgicalMultiFileRefactor => "Surgical Multi-File Refactor",
            Self::HighCraftFrontendUiUx => "High-Craft Frontend UI/UX (Anti-Slop)",
            Self::WebGl3dVisualization => "3D/WebGL/Shader Visualization",
            Self::SqliteLedgerDesign => "SQLite Ledger & Double-Entry Design",
            Self::HardAlgorithmicDeduction => "Hard Algorithmic Deduction & Formal Logic",
            Self::ConcurrencyBugTriage => "Concurrency / Heisenbug RCA",
            Self::SecuritySsrfHardening => "Security / SSRF Hardening",
            Self::SpecComplianceReview => "Spec-Compliance Review Gatekeeping",
            Self::MassiveRepoArchaeology => "Massive Repo Archaeology (100k+)",
            Self::FastAgenticToolCalling => "Fast Agentic Tool-Calling",
            Self::StrictJsonExtraction => "Strict JSON Schema Extraction",
            Self::AdversarialTestGeneration => "Adversarial Test Generation",
            Self::TechnicalGenomeSynthesis => "Technical Genome Synthesis",
            Self::PersianRtlBilingualUx => "Persian RTL + English Bilingual UX",
            Self::TelegramMiniAppBotApi9 => "Telegram Mini-App & Bot API 9.0",
            Self::XAlgorithmViralGrowth => "X-Algorithm Viral Growth Engineering",
            Self::SubSecondIntentRouting => "Sub-Second Intent Routing",
            Self::LongHorizonAutonomousLoop => "Long-Horizon Autonomous Loop",
        }
    }
}

// ─────────────────────────── پروفایل ژنوم ───────────────────────────

/// پروفایل مهندسی معکوس یک مدل — پورت فایل‌های `01..09` به ساختار ماشینی.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelGenomeProfile {
    /// شناسهٔ مدل = نام پوشه (`gpt-6-standard-free`).
    pub model_id: String,
    /// پارادایم استدلال از frontmatter فایل ۰۱.
    pub reasoning_paradigm: String,
    /// آیا حالت Think زنده است؟ (`gpt-6-standard-free`: false)
    pub think_mode_active: bool,
    /// اگر Think خاموش است — جدول `<pre_computation_state_trace>` الزامی.
    pub requires_external_state_trace: bool,
    /// گارد ضد-Canvas / ضد-وب‌سرچ برای هارنس وب.
    pub suppress_web_canvas_hijack: bool,
    /// سقف قواعد هر گیت فازی (پیش‌فرض ۶).
    pub max_simultaneous_rules_per_phase: usize,
    /// اسکورکارد ۲۰تسکی کارت ۰۹ — امتیاز نامعلوم هرگز ساخته نمی‌شود.
    pub workload_scores: BTreeMap<EngineeringWorkload, f32>,
    /// کالیبراسیون کارت ۰۹ (برای گزارش صادقانه).
    pub calibration_status: String,
}

impl ModelGenomeProfile {
    /// بهترین مدلِ موجود برای یک تسک از میان پروفایل‌ها — قطعی:
    /// بالاترین امتیاز؛ تساوی → ترتیب الفبایی `model_id`؛ بدون امتیاز → رد.
    #[must_use]
    pub fn best_for_workload<'a>(
        profiles: impl IntoIterator<Item = &'a ModelGenomeProfile>,
        workload: EngineeringWorkload,
    ) -> Option<&'a ModelGenomeProfile> {
        profiles
            .into_iter()
            .filter_map(|p| p.workload_scores.get(&workload).map(|s| (s, p)))
            .max_by(|(s1, p1), (s2, p2)| {
                s1.partial_cmp(s2).unwrap_or(std::cmp::Ordering::Equal).then(
                    p2.model_id.cmp(&p1.model_id),
                )
            })
            .map(|(_, p)| p)
    }
}

/// خطاهای بارگذاری ژنوم از دیسک.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenomeError {
    /// پوشهٔ مدل وجود ندارد یا خوانا نیست.
    UnreadableDirectory { path: String },
    /// یکی از ۹ لایهٔ canonical غایب است.
    MissingLayer { path: String, layer: &'static str },
    /// فایل موجود است اما frontmatter YAML ندارد.
    MissingFrontmatter { path: String },
}

impl std::fmt::Display for GenomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnreadableDirectory { path } => write!(f, "unreadable model dir: {path}"),
            Self::MissingLayer { path, layer } => write!(f, "missing genome layer {layer} in {path}"),
            Self::MissingFrontmatter { path } => write!(f, "missing YAML frontmatter in {path}"),
        }
    }
}

impl std::error::Error for GenomeError {}

/// frontmatter سادهٔ `key: value` را می‌کشد (بدون وابستگی YAML).
fn parse_frontmatter(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Some(rest) = text.strip_prefix("---\n") else {
        return out;
    };
    for line in rest.lines().take_while(|line| line.trim() != "---") {
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim().trim_matches('"').to_string();
            if !value.is_empty() {
                out.insert(key.trim().to_string(), value);
            }
        }
    }
    out
}

/// اسکورکارد ۲۰تسکی کارت ۰۹ را پارس می‌کند — سطرهای `| n | ... | 0.xx |`.
/// سطرهای کمتر از ۲۰ یعنی امتیاز ناقص → فقط سطرهای معتبر ثبت می‌شوند؛
/// این تابع هرگز عدد از خودسازی نمی‌سازد.
fn parse_workload_scores(routing_card: &str) -> BTreeMap<EngineeringWorkload, f32> {
    let mut out = BTreeMap::new();
    for line in routing_card.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = trimmed.split('|').map(str::trim).collect();
        // cells[0] خالی (جداکنندهٔ ابتدای سطر)؛ cells[1] شمارهٔ ردیف است.
        let Some(row_no) = cells.get(1).and_then(|c| c.parse::<usize>().ok()) else {
            continue;
        };
        if row_no == 0 || row_no > EngineeringWorkload::ALL.len() {
            continue;
        }
        let Some(score) = cells.iter().skip(2).find_map(|c| {
            let parsed = c.parse::<f32>().ok()?;
            if (0.0..=1.0).contains(&parsed) {
                Some(parsed)
            } else {
                None
            }
        }) else {
            continue;
        };
        out.insert(EngineeringWorkload::ALL[row_no - 1], score);
    }
    out
}

/// بارگذاری ژنوم یک مدل از پوشهٔ `models/<model_id>/` روی دیسک:
/// هر ۹ فایل `01..09` باید موجود باشند؛ frontmatter فایل ۰۱ خوانده می‌شود و
/// کارت ۰۹ به نقشهٔ امتیاز ۲۰تسکی تبدیل می‌گردد.
///
/// # Errors
/// `UnreadableDirectory`/`MissingLayer`/`MissingFrontmatter`.
pub fn load_model_genome_from_dir(dir: &Path) -> Result<ModelGenomeProfile, GenomeError> {
    if !dir.is_dir() {
        return Err(GenomeError::UnreadableDirectory { path: dir.display().to_string() });
    }
    let model_id = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown-model".to_string());

    let mut first_frontmatter: Option<BTreeMap<String, String>> = None;
    for layer in GenomeLayerId::ALL {
        let path = dir.join(layer.filename());
        if !path.is_file() {
            return Err(GenomeError::MissingLayer {
                path: dir.display().to_string(),
                layer: layer.filename(),
            });
        }
        if first_frontmatter.is_none() {
            let text = std::fs::read_to_string(&path).map_err(|_| GenomeError::UnreadableDirectory {
                path: dir.display().to_string(),
            })?;
            let fm = parse_frontmatter(&text);
            if fm.is_empty() {
                return Err(GenomeError::MissingFrontmatter { path: path.display().to_string() });
            }
            first_frontmatter = Some(fm);
        }
    }

    let routing_text = std::fs::read_to_string(dir.join(GenomeLayerId::RoutingAndHandoffCard.filename()))
        .map_err(|_| GenomeError::UnreadableDirectory { path: dir.display().to_string() })?;
    let calibration = parse_frontmatter(&routing_text)
        .get("calibration_status")
        .cloned()
        .unwrap_or_else(|| "unspecified".to_string());
    let workload_scores = parse_workload_scores(&routing_text);

    let fm = first_frontmatter.unwrap_or_default();
    let reasoning_paradigm = fm
        .get("reasoning_paradigm")
        .cloned()
        .unwrap_or_else(|| "unknown_paradigm".to_string());
    let think_mode_active = match fm.get("think_mode_active").map(String::as_str) {
        Some("true") => true,
        Some("false") => false,
        _ => {
            // استنتاج محافظه‌کارانه: پارادایم «hidden reasoning» یعنی CoT مرئی نیست.
            let p = reasoning_paradigm.to_ascii_lowercase();
            if p.contains("hidden") {
                false
            } else {
                p.contains("thinking") || p.contains("reasoner") || p.contains("visible")
            }
        }
    };
    let max_rules = fm
        .get("max_simultaneous_rules_per_phase")
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| (1..=24).contains(n))
        .unwrap_or(6);

    Ok(ModelGenomeProfile {
        model_id,
        reasoning_paradigm,
        think_mode_active,
        requires_external_state_trace: !think_mode_active,
        suppress_web_canvas_hijack: fm
            .get("suppress_web_canvas_hijack")
            .map(|v| v == "true")
            .unwrap_or(!think_mode_active),
        max_simultaneous_rules_per_phase: max_rules,
        workload_scores,
        calibration_status: calibration,
    })
}

// ─────────────────────────── پروفایل‌های داخلی ───────────────────────────

/// امتیازهای اسکورکارد `gpt-6-standard-free` — کپیِ کارت ۰۹ مونت‌شده
/// (کالیبراسیون: برآورد کیفی، نه تلمتری اندازه‌گیری‌شده).
const GPT6_STANDARD_SCORES: [f32; 20] = [
    0.87, 0.68, 0.73, 0.70, 0.55, 0.83, 0.88, 0.70, 0.82, 0.86, 0.52, 0.67, 0.84, 0.85, 0.91,
    0.67, 0.72, 0.43, 0.56, 0.39,
];

/// پروفایل داخلی هدف #01 — `gpt-6-standard-free` (وب رسمی، Think Mode OFF):
/// جدول `<pre_computation_state_trace>` اجباری + گارد ضد-Canvas.
#[must_use]
pub fn builtin_gpt6_standard_profile() -> ModelGenomeProfile {
    scores_to_profile(
        "gpt-6-standard-free",
        "autoregressive_generation_with_hidden_reasoning",
        false,
        &GPT6_STANDARD_SCORES,
        "qualitative_estimates_not_benchmark_measurements",
    )
}

/// امتیازهای provision al ۱۱ اسلات plug-and-play آیندهٔ `00_ECOSYSTEM_INDEX.md`.
/// برآورد کیفی آرکتایپی — تا وقتی کارت ۰۹ واقعی مونت نشود provisional هستند.
const ECOSYSTEM_SLOT_SCORES: &[(&str, bool, [f32; 20])] = &[
    ("gpt-6-thinking", true, [0.88, 0.82, 0.78, 0.74, 0.66, 0.85, 0.90, 0.78, 0.84, 0.88, 0.62, 0.74, 0.86, 0.87, 0.92, 0.70, 0.78, 0.55, 0.62, 0.58]),
    ("claude-opus-thinking", true, [0.87, 0.83, 0.82, 0.90, 0.70, 0.80, 0.84, 0.76, 0.82, 0.90, 0.58, 0.76, 0.84, 0.86, 0.90, 0.74, 0.74, 0.50, 0.55, 0.60]),
    ("claude-sonnet-standard", true, [0.80, 0.74, 0.80, 0.84, 0.62, 0.74, 0.76, 0.68, 0.74, 0.82, 0.55, 0.78, 0.80, 0.80, 0.84, 0.70, 0.70, 0.46, 0.56, 0.55]),
    ("gemini-pro-longcontext", true, [0.84, 0.72, 0.74, 0.72, 0.66, 0.80, 0.83, 0.72, 0.78, 0.84, 0.88, 0.72, 0.82, 0.82, 0.88, 0.72, 0.72, 0.50, 0.60, 0.56]),
    ("deepseek-r1-reasoner", true, [0.85, 0.78, 0.72, 0.60, 0.50, 0.78, 0.92, 0.78, 0.76, 0.82, 0.55, 0.66, 0.86, 0.88, 0.84, 0.58, 0.64, 0.52, 0.56, 0.50]),
    ("grok-frontier", true, [0.78, 0.70, 0.70, 0.68, 0.58, 0.72, 0.80, 0.68, 0.72, 0.76, 0.56, 0.72, 0.76, 0.78, 0.80, 0.62, 0.64, 0.62, 0.58, 0.46]),
    ("zai-glm-harness", true, [0.80, 0.72, 0.74, 0.74, 0.56, 0.76, 0.80, 0.70, 0.74, 0.80, 0.60, 0.80, 0.82, 0.80, 0.84, 0.72, 0.76, 0.50, 0.62, 0.66]),
    ("qwen-coder-agent", true, [0.78, 0.80, 0.82, 0.76, 0.60, 0.76, 0.82, 0.70, 0.72, 0.78, 0.58, 0.80, 0.80, 0.82, 0.80, 0.64, 0.70, 0.48, 0.56, 0.58]),
    ("openai-codex-agent", true, [0.82, 0.82, 0.86, 0.74, 0.58, 0.78, 0.84, 0.72, 0.76, 0.82, 0.60, 0.84, 0.80, 0.84, 0.84, 0.62, 0.68, 0.44, 0.54, 0.62]),
    ("llama4-cerebras-fast", false, [0.70, 0.62, 0.64, 0.62, 0.48, 0.68, 0.74, 0.60, 0.64, 0.70, 0.50, 0.80, 0.70, 0.72, 0.72, 0.56, 0.58, 0.40, 0.80, 0.42]),
    ("opencode-zen-flagship", false, [0.78, 0.72, 0.74, 0.70, 0.54, 0.72, 0.80, 0.68, 0.72, 0.76, 0.56, 0.76, 0.76, 0.78, 0.80, 0.62, 0.66, 0.44, 0.58, 0.46]),
];

/// کل اکوسیستم: هدف #01 + ۱۱ اسلات plug-and-play — همهٔ پروفایل‌ها
/// قابل‌گسترش از دیسک (`models/<id>/`) بدون تغییر کرنل.
#[must_use]
pub fn builtin_ecosystem_profiles() -> Vec<ModelGenomeProfile> {
    let mut out = vec![builtin_gpt6_standard_profile()];
    for (model_id, think, scores) in ECOSYSTEM_SLOT_SCORES {
        out.push(scores_to_profile(
            model_id,
            if *think { "extended_thinking_with_visible_cot" } else { "autoregressive_generation_with_hidden_reasoning" },
            *think,
            scores,
            "provisional_archetype_estimate_pending_09_card",
        ));
    }
    out
}

#[allow(clippy::cast_precision_loss)] // امتیازهای ثابت 0..=1 — دقت f32 کافی
fn scores_to_profile(
    model_id: &str,
    paradigm: &str,
    think: bool,
    scores: &[f32; 20],
    calibration: &str,
) -> ModelGenomeProfile {
    let mut map = BTreeMap::new();
    for (workload, score) in EngineeringWorkload::ALL.iter().zip(scores.iter()) {
        map.insert(*workload, *score);
    }
    ModelGenomeProfile {
        model_id: model_id.to_string(),
        reasoning_paradigm: paradigm.to_string(),
        think_mode_active: think,
        requires_external_state_trace: !think,
        suppress_web_canvas_hijack: !think,
        max_simultaneous_rules_per_phase: 6,
        workload_scores: map,
        calibration_status: calibration.to_string(),
    }
}

// ─────────────────────────── کامپایلر پرامپت ───────────────────────────

/// کامپایل پرامپت اختصاصی مدل: برای پروفایل‌های بدون Think جدول
/// `<pre_computation_state_trace>` قبل از کد اجباری می‌شود، گارد ضد-Canvas
/// می‌آید، invariant ها گیت‌فازی ≤N می‌شوند و قرارداد ضد-elision می‌بندد.
#[must_use]
pub fn compile_prompt_for_model(
    profile: &ModelGenomeProfile,
    task: &str,
    invariants: &[&str],
) -> String {
    let mut out = String::with_capacity(2048);
    if profile.suppress_web_canvas_hijack {
        out.push_str("## HARNESS GUARDS (STRICT)\n");
        out.push_str("- Do NOT invoke Canvas / canmore / artifacts tools; emit inline fenced Markdown only.\n");
        out.push_str("- Do NOT invoke web search unless the task explicitly requires live data; otherwise offline deterministic synthesis.\n\n");
    }
    out.push_str("## TASK\n");
    out.push_str(task.trim());
    out.push_str("\n\n");

    if profile.requires_external_state_trace {
        out.push_str("## MANDATORY PRE-COMPUTATION (Token Order = Thought Order)\n");
        out.push_str("Your Think Mode is OFF: asking for code or a verdict first causes Premature \
            Commitment in tokens 1..30. BEFORE any code, emit a `<pre_computation_state_trace>` \
            block containing: (1) an invariant trace table — each invariant below and how it is \
            honored, (2) the edge cases you will defend, (3) the failure modes you ruled out.\n\n");
    }

    let batch = profile.max_simultaneous_rules_per_phase.max(1);
    let phases = if invariants.is_empty() {
        0
    } else {
        invariants.len().div_ceil(batch)
    };
    out.push_str("## PHASE GATES (emit phases in order; preserve every rule verbatim)\n");
    for phase in 0..phases {
        out.push_str(&format!("### Phase {} — Invariant Gate\n", phase + 1));
        for rule in &invariants[phase * batch..((phase + 1) * batch).min(invariants.len())] {
            out.push_str(&format!("- {rule}\n"));
        }
    }
    if phases == 0 {
        out.push_str("(no invariant gates supplied)\n");
    }
    out.push('\n');

    out.push_str("## OUTPUT CONTRACT (ANTI-ELISION)\n");
    out.push_str("- Never emit `// ... existing code`, `...` or elided placeholders; write complete code.\n");
    out.push_str("- For patches prefer `<<<<<<< SEARCH` / `=======` / `>>>>>>> REPLACE` blocks with >= 3-line context anchors.\n");
    out.push_str("- End with an explicit verification statement separating proven facts from estimates.\n");
    out
}

// ─────────────────────────── پاکت تحویل ───────────────────────────

/// پاکت تحویل بین‌مدلی — هم‌اسکیما با `KIA-BRAIN-OS/07_LINK_Protocol.md`.
/// تحویل بدون `base_sha` یا با ادعای بی‌اثبات، در سطح سازنده رد می‌شود.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffPacket {
    pub handoff_id: String,
    pub from_model: String,
    pub to_model: String,
    pub base_sha: String,
    pub task: String,
    pub invariants: Vec<String>,
    pub files_touched: Vec<String>,
    pub failed_hypotheses_do_not_repeat: Vec<String>,
    pub verification_proof: Vec<String>,
    pub target_contract: String,
}

impl HandoffPacket {
    /// پاکت کیان‌کیان برای گام جاری — الگوی آمادهٔ سازنده.
    #[must_use]
    pub fn kian_step4_template(base_sha: &str) -> Self {
        Self {
            handoff_id: format!("ho_step4_{base_sha}"),
            from_model: "kian-hermes".to_string(),
            to_model: "next-executor".to_string(),
            base_sha: base_sha.to_string(),
            task: "Continue Wave 2 re-architected kernel work without breaking ledger invariants".to_string(),
            invariants: vec![
                "KbEntry idempotent; trial balance SUM(Posting.amount) = 0".to_string(),
                "20 active products, 466 citizens, 247 referral lines, 108 escrows untouched".to_string(),
                "all crates forbid(unsafe_code) and clippy -D warnings clean".to_string(),
            ],
            files_touched: Vec::new(),
            failed_hypotheses_do_not_repeat: Vec::new(),
            verification_proof: Vec::new(),
            target_contract: "green cargo test + clippy on touched crates, zero secrets in git diff --cached".to_string(),
        }
    }

    /// سریال‌سازی JSON قطعی (برای A2A و لاگ شاهد).
    ///
    /// # Errors
    /// در عمل غیرممکن (ساختار فقط String/Vec<String>)؛ برای امضای صادقانهٔ
    /// فیلد، خطا تایپ‌شده برمی‌گردد.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// رمزگشایی JSON قطعی.
    ///
    /// # Errors
    /// `serde_json::Error` روی ساختار نامشروع.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ─────────────────────────── تست‌ها ───────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const MOUNTED_GENOME_DIR: &str = "OMNI-MODEL-GENOME/models/gpt-6-standard-free";

    #[test]
    fn genome_layer_filenames_are_canonical() {
        assert_eq!(GenomeLayerId::ALL.len(), 9);
        assert_eq!(GenomeLayerId::TopologyAndHarness.filename(), "01_TOPOLOGY_AND_HARNESS.md");
        assert_eq!(GenomeLayerId::RoutingAndHandoffCard.filename(), "09_ROUTING_AND_HANDOFF_CARD.md");
        for (idx, layer) in GenomeLayerId::ALL.iter().enumerate() {
            let name = layer.filename();
            assert_eq!(name.get(0..2).unwrap(), format!("{:02}", idx + 1));
            assert!(name.ends_with(".md"));
        }
    }

    #[test]
    fn engineering_workloads_cover_the_twenty_tasks() {
        assert_eq!(EngineeringWorkload::ALL.len(), 20);
        assert_eq!(EngineeringWorkload::ALL[0].label(), "Zero-to-One System Architecture");
        assert_eq!(EngineeringWorkload::ALL[19].label(), "Long-Horizon Autonomous Loop");
    }

    #[test]
    fn builtin_gpt6_standard_profile_matches_mounted_scorecard() {
        let p = builtin_gpt6_standard_profile();
        assert_eq!(p.model_id, "gpt-6-standard-free");
        assert!(!p.think_mode_active, "Think Mode OFF for the free standard tier");
        assert!(p.requires_external_state_trace);
        assert!(p.suppress_web_canvas_hijack);
        assert_eq!(p.workload_scores.len(), 20);
        assert_eq!(p.workload_scores[&EngineeringWorkload::SystemArchitecture], 0.87);
        assert_eq!(p.workload_scores[&EngineeringWorkload::HardAlgorithmicDeduction], 0.88);
        assert_eq!(p.workload_scores[&EngineeringWorkload::TechnicalGenomeSynthesis], 0.91);
        assert_eq!(p.workload_scores[&EngineeringWorkload::LongHorizonAutonomousLoop], 0.39);
    }

    #[test]
    fn mounted_genome_loads_from_disk_with_full_scorecard() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(MOUNTED_GENOME_DIR);
        let profile = load_model_genome_from_dir(&dir).expect("mounted genome must load");
        assert_eq!(profile.model_id, "gpt-6-standard-free");
        assert!(
            profile.reasoning_paradigm.contains("hidden_reasoning"),
            "paradigm from frontmatter: {}",
            profile.reasoning_paradigm
        );
        assert!(!profile.think_mode_active);
        assert_eq!(profile.workload_scores.len(), 20, "20-row scorecard must parse");
        assert_eq!(profile.workload_scores[&EngineeringWorkload::SqliteLedgerDesign], 0.83);
    }

    #[test]
    fn missing_layer_fails_loudly() {
        let tmp = std::env::temp_dir().join("kian_genome_missing_layer_test");
        std::fs::create_dir_all(&tmp).expect("tmp dir");
        std::fs::write(tmp.join("01_TOPOLOGY_AND_HARNESS.md"), "---\nreasoning_paradigm: x\n---\nbody")
            .expect("write layer 1");
        let err = load_model_genome_from_dir(&tmp).unwrap_err();
        assert!(matches!(err, GenomeError::MissingLayer { layer: "02_REASONING_AND_LOGIC_DNA.md", .. }));
        assert!(err.to_string().contains("missing genome layer"));
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn missing_frontmatter_fails_loudly() {
        let tmp = std::env::temp_dir().join("kian_genome_no_frontmatter_test");
        std::fs::create_dir_all(&tmp).expect("tmp dir");
        for layer in GenomeLayerId::ALL {
            std::fs::write(tmp.join(layer.filename()), "no frontmatter here").expect("write");
        }
        let err = load_model_genome_from_dir(&tmp).unwrap_err();
        assert!(matches!(err, GenomeError::MissingFrontmatter { .. }));
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn compile_injects_state_trace_and_canvas_suppression_for_no_think_model() {
        let profile = builtin_gpt6_standard_profile();
        let prompt = compile_prompt_for_model(
            &profile,
            "کد ماژول دفتر دوبل را بنویس",
            &["SUM(Posting.amount) = 0", "466 citizens untouched", "20 active products"],
        );
        assert!(prompt.contains("<pre_computation_state_trace>"), "state trace gate: {prompt}");
        assert!(prompt.contains("Token Order = Thought Order"));
        assert!(prompt.contains("Do NOT invoke Canvas / canmore"), "anti-canvas guard: {prompt}");
        assert!(prompt.contains("Anti-Elision") || prompt.contains("ANTI-ELISION"));
        assert!(prompt.contains("SUM(Posting.amount) = 0"));
        // ترتیب: جدول پیش‌محاسبه قبل از گیت‌های فازی می‌آید
        let trace_pos = prompt.find("MANDATORY PRE-COMPUTATION").expect("trace section");
        let gates_pos = prompt.find("PHASE GATES").expect("gates section");
        assert!(trace_pos < gates_pos, "state trace MUST precede phase gates");
    }

    #[test]
    fn compile_chunks_invariants_by_max_rules_per_phase() {
        let profile = builtin_gpt6_standard_profile();
        let invariants = ["i1", "i2", "i3", "i4", "i5", "i6", "i7"];
        let prompt = compile_prompt_for_model(&profile, "task", &invariants);
        assert_eq!(prompt.matches("### Phase ").count(), 2, "7 rules @ max 6 → 2 phases: {prompt}");
        let thinking = ModelGenomeProfile {
            think_mode_active: true,
            requires_external_state_trace: false,
            suppress_web_canvas_hijack: false,
            ..builtin_gpt6_standard_profile()
        };
        let prompt2 = compile_prompt_for_model(&thinking, "task", &["only rule"]);
        assert!(!prompt2.contains("<pre_computation_state_trace>"), "think ON: no trace gate");
        assert!(!prompt2.contains("Do NOT invoke Canvas"), "think ON: no canvas guard");
        assert!(prompt2.contains("- only rule"));
    }

    #[test]
    fn think_on_profiles_skip_state_trace_but_keep_phase_gates() {
        let profiles = builtin_ecosystem_profiles();
        assert_eq!(profiles.len(), 12, "target #01 + 11 plug-and-play slots");
        let thinking = profiles.iter().find(|p| p.model_id == "claude-opus-thinking").expect("slot");
        assert!(thinking.think_mode_active);
        let prompt = compile_prompt_for_model(thinking, "refactor crates", &["clippy clean"]);
        assert!(!prompt.contains("<pre_computation_state_trace>"));
        assert!(prompt.contains("### Phase 1"));
    }

    #[test]
    fn routing_picks_the_right_specialist_per_workload() {
        let profiles = builtin_ecosystem_profiles();
        let best_frontend = ModelGenomeProfile::best_for_workload(&profiles, EngineeringWorkload::HighCraftFrontendUiUx)
            .expect("frontend candidate");
        assert_eq!(best_frontend.model_id, "claude-opus-thinking");
        let best_deduction =
            ModelGenomeProfile::best_for_workload(&profiles, EngineeringWorkload::HardAlgorithmicDeduction)
                .expect("deduction candidate");
        assert_eq!(best_deduction.model_id, "deepseek-r1-reasoner");
        let best_fast = ModelGenomeProfile::best_for_workload(&profiles, EngineeringWorkload::SubSecondIntentRouting)
            .expect("fast candidate");
        assert_eq!(best_fast.model_id, "llama4-cerebras-fast");
        // هدف #01 هرگز برای حلقهٔ بی‌پایان روت می‌شود (0.39 = Do Not Route)
        let horizon = builtin_gpt6_standard_profile()
            .workload_scores
            .get(&EngineeringWorkload::LongHorizonAutonomousLoop)
            .copied()
            .unwrap_or(1.0);
        assert!(horizon < 0.45, "long horizon must stay Do-Not-Route for gpt-6-standard-free");
    }

    #[test]
    fn handoff_packet_roundtrips_json_with_invariants() {
        let packet = HandoffPacket::kian_step4_template("040d42a");
        let json = packet.to_json().expect("serialize");
        let back = HandoffPacket::from_json(&json).expect("deserialize");
        assert_eq!(packet, back);
        assert!(json.contains("\"base_sha\":\"040d42a\""));
        assert!(json.contains("failed_hypotheses_do_not_repeat"));
    }

    #[test]
    fn ecosystem_profiles_never_invent_unknown_scores() {
        // پروفایل از پوشه‌ای با کارت ۰۹ بی‌جدول → نقشهٔ امتیاز خالی، نه عدد ساختگی
        let tmp = std::env::temp_dir().join("kian_genome_no_scorecard_test");
        std::fs::create_dir_all(&tmp).expect("tmp dir");
        for layer in GenomeLayerId::ALL {
            std::fs::write(tmp.join(layer.filename()), "---\nreasoning_paradigm: unknown\n---\nno table")
                .expect("write");
        }
        let profile = load_model_genome_from_dir(&tmp).expect("loads with empty scores");
        assert!(profile.workload_scores.is_empty(), "no fabricated scores");
        assert!(
            ModelGenomeProfile::best_for_workload([profile.clone()].iter(), EngineeringWorkload::SystemArchitecture)
                .is_none(),
            "score-less profile must not be routed"
        );
        std::fs::remove_dir_all(&tmp).ok();
    }
}
