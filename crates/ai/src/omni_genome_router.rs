//! `omni_genome_router` — the Kian multi-model genome engine (Wave 2 — Step 4/6).
//!
//! Port of `OMNI-MODEL-GENOME/` to the Rust kernel:
//! - Every frontier model has a 9-file `01..09` folder (`models/<model_id>/`).
//! - [`load_model_genome_from_dir`] validates that all 9 layers exist,
//!   reads the YAML frontmatter, and parses card 09's 20-task scorecard.
//! - [`builtin_gpt6_standard_profile`] and [`builtin_ecosystem_profiles`] provide the
//!   same contract without IO (zero-IO tests and a diskless runtime).
//! - [`compile_prompt_for_model`] is each model's prompt compiler: for the
//!   'no-Think' profiles (e.g. `gpt-6-standard-free`) it injects the pre-computed
//!   `<pre_computation_state_trace>` table before any code
//!   ('token order = thought order'), adds the anti-Canvas/anti-web-search guard, and
//!   stacks the invariants into fuzzy gates ≤N + the anti-elision contract.
//! - [`HandoffPacket`] the deterministic cross-model handoff packet — schema-aligned with
//!   `KIA-BRAIN-OS/07_LINK_Protocol.md`.
//!
//! Discovery principle: an unknown score/metric never receives a fabricated value —
//! profiles without card 09 get an empty score map and routing falls
//! back to the fallback (the zero-card registry).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};

// ─────────────────────────── Genome layers ───────────────────────────

/// Canonical layers of the 9-file genome — the file names are a hard contract.
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
    /// All layers in canonical order.
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

    /// Canonical file name (`01_TOPOLOGY_AND_HARNESS.md` .. `09_...`).
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

    /// The canonical v4 file name of the sovereign matrix — the equivalent of each layer;
    /// (`01_IDENTITY_AND_PERSONA_GENOME.md` .. `09_DISTILLED_CLONE_SYSTEM_PROMPT.md`).
    /// the disk loader accepts both names (Commit #27).
    #[must_use]
    pub const fn canonical_v4_filename(self) -> &'static str {
        match self {
            Self::TopologyAndHarness => "01_IDENTITY_AND_PERSONA_GENOME.md",
            Self::ReasoningAndLogicDna => "02_SYSTEM_ARCHITECTURE_AND_CONSTRAINTS.md",
            Self::CodeAndPatchMechanics => "03_REASONING_AND_DECISION_TREE.md",
            Self::ToolCallingAndState => "04_TOOL_ORCHESTRATION_AND_AGENTIC_LOOP.md",
            Self::UiUxAestheticBias => "05_CODING_AND_ENGINEERING_DOCTRINE.md",
            Self::GuardrailsAndFraming => "06_UI_UX_AND_ARTIFACT_CRAFT.md",
            Self::PromptCompilerLexicon => "07_SAFETY_GUARDRAILS_AND_EDGE_CASES.md",
            Self::FailureModesAndPatches => "08_FAILURE_MODES_AND_BLINDSPOTS.md",
            Self::RoutingAndHandoffCard => "09_DISTILLED_CLONE_SYSTEM_PROMPT.md",
        }
    }
}

/// The five canonical sovereign-model folders — matrix order (Commit #27, 45 files).
pub const SOVEREIGN_MODEL_DIRS: [&str; 5] = [
    "01-gpt-6-web",
    "02-gemini-4-aragon-high",
    "03-deepseek-3.2-web-think-mode",
    "04-claude-sonnet-5.5-extra",
    "05-glm-5.3-flash-max",
];

/// Resolves a sovereign-model alias — exact or any alias to the canonical folder.
/// (`Cluade-sonnet-5.5-extra` and `Claude-sonnet-5.5-extra` both → `04-...`;
/// `GLM 5.3 flash MAX` / `glm-5.3-flash-max` / `GLM-5.3-Flash-Max` → `05-...`.)
#[must_use]
pub fn resolve_sovereign_model_alias(id: &str) -> Option<&'static str> {
    /// The alias table — canonical folder + all accepted spellings.
    const TABLE: &[(&str, &[&str])] = &[
        ("01-gpt-6-web", &["GPT 6-Web", "gpt-6-web", "GPT-6", "gpt-6-standard-free"]),
        ("02-gemini-4-aragon-high", &["GEMINI 4 ARAGON HIGH", "gemini-4-aragon-high", "ARAGON HIGH"]),
        ("03-deepseek-3.2-web-think-mode", &["DeepSeek-3.2-WEB-Think-mode", "deepseek-3.2-web-think-mode", "DS-3.2-Think"]),
        ("04-claude-sonnet-5.5-extra", &["Cluade-sonnet-5.5-extra", "Claude-sonnet-5.5-extra", "claude-sonnet-5-5", "Claude Sonnet 5.5"]),
        ("05-glm-5.3-flash-max", &["GLM 5.3 flash MAX", "glm-5.3-flash-max", "GLM-5.3-Flash-Max", "GLM 5.3 Flash Max"]),
    ];
    let needle = id.trim();
    if needle.is_empty() {
        return None;
    }
    TABLE.iter().find_map(|(dir, aliases)| {
        if dir.eq_ignore_ascii_case(needle)
            || aliases.iter().any(|a| a.eq_ignore_ascii_case(needle))
        {
            Some(*dir)
        } else {
            None
        }
    })
}

// ─────────────────────────── 20 engineering tasks ───────────────────────────

/// The ecosystem's 20 standard engineering tasks — the same order as the card 09 table.
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
    /// All 20 tasks in routing-table order.
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

    /// A human-readable label for routing reports.
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

// ─────────────────────────── Genome profile ───────────────────────────

/// A model's reverse-engineered engineering profile — the `01..09` files ported into a machine-readable structure.
/// The many boolean flags are the domain's nature: each flag is an independent card 09 contract
/// (`Think`, anti-filler, zero-copy, backtick, anti-Canvas) and their combination forms the meaning.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelGenomeProfile {
    /// Model ID = folder name (`gpt-6-standard-free`).
    pub model_id: String,
    /// Official display name (`GEMINI 4 Aragon HIGH`) — defaults to the `model_id`.
    pub display_name: String,
    /// Reasoning paradigm from file 01's frontmatter.
    pub reasoning_paradigm: String,
    /// Is Think mode live? (`gpt-6-standard-free`: false)
    pub think_mode_active: bool,
    /// When Think is off — the `<pre_computation_state_trace>` table is mandatory.
    pub requires_external_state_trace: bool,
    /// Declared effort level (`EFFORT LEVEL`) — `None` = unknown (never fabricated).
    pub effort_level: Option<f32>,
    /// Chatty-filler ban (`Sure!`, `I hope this helps`, etc.).
    pub forbids_chatty_filler: bool,
    /// Preference for zero-copy Rust synthesis (borrow over clone).
    pub supports_zero_copy_rust_synthesis: bool,
    /// The `clippy-1.100` backtick contract in doc comments.
    pub enforce_clippy_1_100_backticks: bool,
    /// Anti-Canvas / anti-web-search guard for the web harness.
    pub suppress_web_canvas_hijack: bool,
    /// Rule cap per fuzzy gate (default 6).
    pub max_simultaneous_rules_per_phase: usize,
    /// Card 09's 20-task scorecard — unknown scores are never invented.
    pub workload_scores: BTreeMap<EngineeringWorkload, f32>,
    /// Card 09 calibration (for honest reporting).
    pub calibration_status: String,
}

impl ModelGenomeProfile {
    /// The best available model for a task among the profiles — deterministic:
    /// highest score; ties → alphabetical `model_id` order; no score → excluded.
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

/// Genome load-from-disk errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenomeError {
    /// The model folder is missing or unreadable.
    UnreadableDirectory { path: String },
    /// One of the 9 canonical layers is missing.
    MissingLayer { path: String, layer: &'static str },
    /// The file exists but has no YAML frontmatter.
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

/// Scrapes simple `key: value` frontmatter (no YAML dependency).
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

/// Parses card 09's 20-task scorecard — rows of `| n | ... | 0.xx |`;
/// Markdown decorations (`**bold**` and `` `code` ``) are stripped before parsing so
/// cards like `**01**` / `` **`0.99`** `` also parse. Fewer than 20 rows means
/// a partial score → only valid rows are recorded; this function never
/// fabricates a number.
fn parse_workload_scores(routing_card: &str) -> BTreeMap<EngineeringWorkload, f32> {
    let mut out = BTreeMap::new();
    for line in routing_card.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cells: Vec<String> = trimmed
            .split('|')
            .map(|c| c.trim().trim_matches(['*', '`']).trim().to_string())
            .collect();
        // cells[0] is empty (the row-leading separator); cells[1] is the row number.
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

/// Every layer starts with the legacy name; canonical v4 substitutes for a missing file —
/// the dual-naming loader (Commit #27).
fn resolve_layer_path(dir: &Path, layer: GenomeLayerId) -> Option<std::path::PathBuf> {
    let legacy = dir.join(layer.filename());
    if legacy.is_file() {
        return Some(legacy);
    }
    let canonical = dir.join(layer.canonical_v4_filename());
    if canonical.is_file() {
        return Some(canonical);
    }
    None
}

/// Loads one model's genome from disk under `models/<model_id>/`:
/// all 9 layers must exist — under **both namings** (Commit #27):
/// the legacy names (`01_TOPOLOGY_AND_HARNESS.md`..`09_ROUTING_AND_HANDOFF_CARD.md`)
/// or the canonical v4 names of the sovereign matrix
/// (`01_IDENTITY_AND_PERSONA_GENOME.md`..`09_DISTILLED_CLONE_SYSTEM_PROMPT.md`)
/// — legacy takes priority and canonical substitutes for a missing file.
/// File 01's frontmatter is read and card 09 becomes the 20-task score
/// map.
///
/// # Errors
/// `UnreadableDirectory`/`MissingLayer`/`MissingFrontmatter`.
pub fn load_model_genome_from_dir(dir: &Path) -> Result<ModelGenomeProfile, GenomeError> {
    if !dir.is_dir() {
        return Err(GenomeError::UnreadableDirectory { path: dir.display().to_string() });
    }
    let model_id = dir.file_name().map_or_else(
        || "unknown-model".to_string(),
        |n| n.to_string_lossy().to_string(),
    );

    let mut first_frontmatter: Option<BTreeMap<String, String>> = None;
    for layer in GenomeLayerId::ALL {
        let Some(path) = resolve_layer_path(dir, layer) else {
            return Err(GenomeError::MissingLayer {
                path: dir.display().to_string(),
                layer: layer.filename(),
            });
        };
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

    let routing_path = resolve_layer_path(dir, GenomeLayerId::RoutingAndHandoffCard).ok_or_else(|| {
        GenomeError::MissingLayer {
            path: dir.display().to_string(),
            layer: GenomeLayerId::RoutingAndHandoffCard.filename(),
        }
    })?;
    let routing_text = std::fs::read_to_string(routing_path)
        .map_err(|_| GenomeError::UnreadableDirectory { path: dir.display().to_string() })?;
    let routing_fm = parse_frontmatter(&routing_text);
    let calibration = routing_fm
        .get("calibration_status")
        .or_else(|| routing_fm.get("empirical_verification_status"))
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
            // Conservative inference: the 'hidden reasoning' paradigm means the CoT is not visible.
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
        display_name: fm.get("display_name").cloned().unwrap_or_else(|| model_id.clone()),
        effort_level: fm
            .get("effort_level")
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|e| (0.0..=1.0).contains(e)),
        forbids_chatty_filler: fm.get("forbids_chatty_filler").is_some_and(|v| v == "true"),
        supports_zero_copy_rust_synthesis: fm
            .get("supports_zero_copy_rust_synthesis")
            .is_some_and(|v| v == "true"),
        enforce_clippy_1_100_backticks: fm
            .get("enforce_clippy_1_100_backticks")
            .is_some_and(|v| v == "true"),
        model_id,
        reasoning_paradigm,
        think_mode_active,
        requires_external_state_trace: !think_mode_active,
        suppress_web_canvas_hijack: fm
            .get("suppress_web_canvas_hijack")
            .map_or(!think_mode_active, |v| v == "true"),
        max_simultaneous_rules_per_phase: max_rules,
        workload_scores,
        calibration_status: calibration,
    })
}

// ─────────────────────────── Builtin profiles ───────────────────────────

/// `gpt-6-standard-free` scorecard scores — a mounted copy of card 09
/// (calibration: a qualitative estimate, not measured telemetry).
const GPT6_STANDARD_SCORES: [f32; 20] = [
    0.87, 0.68, 0.73, 0.70, 0.55, 0.83, 0.88, 0.70, 0.82, 0.86, 0.52, 0.67, 0.84, 0.85, 0.91,
    0.67, 0.72, 0.43, 0.56, 0.39,
];

/// Builtin target #01 profile — `gpt-6-standard-free` (official web, Think Mode OFF):
/// a mandatory `<pre_computation_state_trace>` table + the anti-Canvas guard.
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

/// The canonical ID of the v2 dual-model registry's target #02.
pub const GEMINI_ARAGON_MODEL_ID: &str = "gemini-4-aragon-high";

/// `gemini-4-aragon-high` scorecard scores — a mounted copy of card 09
/// (calibration: a qualitative user estimate, no live session, no benchmark).
const GEMINI_ARAGON_SCORES: [f32; 20] = [
    0.99, 0.97, 0.96, 0.96, 0.96, 0.99, 0.99, 0.98, 0.98, 0.99, 0.99, 0.92, 0.98, 0.97, 1.00,
    0.99, 0.98, 0.97, 0.65, 0.99,
];

/// Builtin target #02 profile — `gemini-4-aragon-high` (`GEMINI 4 Aragon HIGH`,
/// Think Mode ON with `EFFORT LEVEL` 1.00, anti-filler, zero-copy Rust synthesis, and
/// the `clippy-1.100` backtick contract).
#[must_use]
pub fn builtin_gemini_aragon_high_profile() -> ModelGenomeProfile {
    let mut p = scores_to_profile(
        GEMINI_ARAGON_MODEL_ID,
        "hybrid_thinking (native_deep_cot_scratchpad + aragon4_first_principles_deduction)",
        true,
        &GEMINI_ARAGON_SCORES,
        "user_supplied_qualitative_estimates_not_benchmark_measurements",
    );
    p.display_name = "GEMINI 4 Aragon HIGH".to_string();
    p.effort_level = Some(1.00);
    p.forbids_chatty_filler = true;
    p.supports_zero_copy_rust_synthesis = true;
    p.enforce_clippy_1_100_backticks = true;
    p
}

/// Provisional scores of the 11 future plug-and-play slots from `00_ECOSYSTEM_INDEX.md`.
/// Archetypal qualitative estimates — provisional until a real card 09 is mounted.
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

/// The whole ecosystem: target #01 + target #02 + 11 plug-and-play slots — every profile
/// is extendable from disk (`models/<id>/`) without kernel changes.
#[must_use]
pub fn builtin_ecosystem_profiles() -> Vec<ModelGenomeProfile> {
    let mut out = vec![builtin_gpt6_standard_profile(), builtin_gemini_aragon_high_profile()];
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

// ── The sovereign pentamodel registry (Commit #27) ─────────────────────────────────

/// Model #03 scorecard scores — `DeepSeek-3.2-WEB-Think-mode` (a qualitative estimate
/// from card 09; quantitative math and proof at the ceiling, UI/web at the floor).
const DEEPSEEK_32_THINK_SCORES: [f32; 20] = [
    0.80, 0.72, 0.68, 0.58, 0.50, 0.78, 0.93, 0.75, 0.72, 0.80, 0.62, 0.66, 0.88, 0.88, 0.86,
    0.60, 0.66, 0.52, 0.58, 0.52,
];

/// Model #04 scorecard scores — `Cluade-sonnet-5.5-extra` (a mounted copy of card 00;
/// surgical refactor, JSON extraction, and agentic tooling at the ceiling).
const CLAUDE_SONNET_55_EXTRA_SCORES: [f32; 20] = [
    0.85, 0.70, 0.80, 0.75, 0.65, 0.82, 0.70, 0.55, 0.75, 0.75, 0.65, 0.85, 0.88, 0.82, 0.85,
    0.75, 0.65, 0.40, 0.40, 0.65,
];

/// Model #05 scorecard scores — `GLM 5.3 flash MAX` (the canonical card 09 copy
/// in standard 20-task order; flash agentic routing at the ceiling, 1M repo
/// archaeology at the floor).
const GLM_53_FLASH_MAX_SCORES: [f32; 20] = [
    0.90, 0.88, 0.88, 0.85, 0.75, 0.88, 0.62, 0.55, 0.70, 0.85, 0.45, 0.93, 0.90, 0.83, 0.85,
    0.85, 0.82, 0.78, 0.92, 0.65,
];

/// Sovereign model #03 builtin profile — `03-deepseek-3.2-web-think-mode`.
#[must_use]
pub fn builtin_deepseek_32_think_profile() -> ModelGenomeProfile {
    let mut p = scores_to_profile(
        "03-deepseek-3.2-web-think-mode",
        "explicit_chain_of_thought_with_think_blocks",
        true,
        &DEEPSEEK_32_THINK_SCORES,
        "self_reported_09_scorecard_qualitative_provisional",
    );
    p.display_name = "DeepSeek-3.2-WEB-Think-mode".to_string();
    p
}

/// Sovereign model #04 builtin profile — `04-claude-sonnet-5.5-extra`
/// (both the `Cluade-...` and `Claude-...` IDs resolve in the alias resolver).
#[must_use]
pub fn builtin_claude_sonnet_55_extra_profile() -> ModelGenomeProfile {
    let mut p = scores_to_profile(
        "04-claude-sonnet-5.5-extra",
        "reflective_single_pass_with_self_verification_loops",
        false,
        &CLAUDE_SONNET_55_EXTRA_SCORES,
        "legacy_00_ecosystem_index_card_normalized",
    );
    p.display_name = "Cluade-sonnet-5.5-extra".to_string();
    p
}

/// Sovereign model #05 builtin profile — `05-glm-5.3-flash-max`
/// (aliases: `GLM 5.3 flash MAX`, `glm-5.3-flash-max`, `GLM-5.3-Flash-Max`).
#[must_use]
pub fn builtin_glm_53_flash_max_profile() -> ModelGenomeProfile {
    let mut p = scores_to_profile(
        "05-glm-5.3-flash-max",
        "single_pass_flash_with_parallel_tool_batching",
        false,
        &GLM_53_FLASH_MAX_SCORES,
        "canonical_09_scorecard_self_reported",
    );
    p.display_name = "GLM 5.3 flash MAX".to_string();
    p
}

/// All 5 sovereign models in matrix order — builtin profiles (no IO; for the
/// diskless runtime). The IDs equal the canonical folder names so the disk
/// registry and the builtin one share names. Disk registry: [`load_sovereign_penta_registry`].
#[must_use]
pub fn builtin_sovereign_penta_profiles() -> Vec<ModelGenomeProfile> {
    let mut gpt = builtin_gpt6_standard_profile();
    gpt.model_id = "01-gpt-6-web".to_string();
    gpt.display_name = "GPT 6-Web".to_string();
    let mut gemini = builtin_gemini_aragon_high_profile();
    gemini.model_id = "02-gemini-4-aragon-high".to_string();
    gemini.display_name = "GEMINI 4 ARAGON HIGH".to_string();
    vec![
        gpt,
        gemini,
        builtin_deepseek_32_think_profile(),
        builtin_claude_sonnet_55_extra_profile(),
        builtin_glm_53_flash_max_profile(),
    ]
}

/// The disk-backed pentamodel registry — loads all 5 [`SOVEREIGN_MODEL_DIRS`] folders
/// under `<genome_root>/models/` (the dual-naming loader; 45 files).
///
/// # Errors
/// [`GenomeError`] for an unreadable folder / a missing layer / missing frontmatter —
/// fail-closed; no incomplete model enters the registry.
pub fn load_sovereign_penta_registry(genome_root: &Path) -> Result<Vec<ModelGenomeProfile>, GenomeError> {
    let models_root = genome_root.join("models");
    SOVEREIGN_MODEL_DIRS
        .iter()
        .map(|dir| load_model_genome_from_dir(&models_root.join(dir)))
        .collect()
}

// ── Deterministic dual-mode selection — ManualOverride / AgentAutoDecision (Commit #28) ──

/// The model-selection mode over the pentamodel matrix — both exact wire symbols:
/// `ManualOverride { model }` (the user explicitly locks one of the 5 models;
/// aliases resolve; unknown fails closed) and `AgentAutoDecision`
/// (the router analyzes the task's telemetry and picks the optimal model itself).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelSelectionMode {
    /// Manual override — an exact model or alias (`GLM 5.3 flash MAX`, `gpt-6-web`, …).
    ManualOverride {
        /// The user's selected model ID.
        model: String,
    },
    /// The agent's own decision — prompt-telemetry analysis and automatic selection.
    AgentAutoDecision,
}

/// The model-selection error — the only possible failure: an unknown alias/ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterModeError {
    /// No sovereign model matches this ID/alias — no guessing.
    UnknownModelAlias {
        /// The rejected ID.
        model: String,
    },
}

impl std::fmt::Display for RouterModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownModelAlias { model } => {
                write!(f, "unknown sovereign model alias: {model}")
            }
        }
    }
}

impl std::error::Error for RouterModeError {}

/// The dual-mode selection decision — the deterministic output of [`manual_override`] and
/// [`agent_auto_decision`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DualModeDecision {
    /// canonical folder of the selected model (`05-glm-5.3-flash-max`, …).
    pub selected_dir: &'static str,
    /// Model display name (`GLM 5.3 flash MAX`, …).
    pub selected_model: String,
    /// Selection mode — `manual_override` | `agent_auto_decision`.
    pub selection_mode: &'static str,
    /// Confidence — a manual override is always `1.0`; automatic is `0.55..=0.95`.
    pub confidence_score: f32,
    /// Auditable decision rationale.
    pub decision_rationale: String,
    /// Matched alias — meaningful only for a non-exact manual override.
    pub matched_alias: Option<String>,
}

/// The automatic router's telemetry signals — `(signal, keywords, champion_dir)`.
/// Table order = tie-break order (the first matrix entry wins).
const ROUTER_TELEMETRY_SIGNALS: &[(&str, &[&str], &str)] = &[
    (
        "flash_agentic_tool_density",
        &["flash", "fast", "quick", "agentic", "tool", "batch", "rapid", "patch", "multi-tool", "instant"],
        "05-glm-5.3-flash-max",
    ),
    (
        "quant_math_backtest_depth",
        &["quant", "backtest", "math", "sharpe", "candle", "strategy", "numeric", "derivation", "proof"],
        "03-deepseek-3.2-web-think-mode",
    ),
    (
        "rust_solidity_systems_audit",
        &["rust", "solidity", "audit", "borrow", "gas", "reentrancy", "refactor", "unsafe", "compiler"],
        "04-claude-sonnet-5.5-extra",
    ),
    (
        "long_context_multimodal",
        &["1m", "context", "multimodal", "vision", "audio", "document", "video", "long-form"],
        "02-gemini-4-aragon-high",
    ),
    (
        "live_web_and_ui_craft",
        &["web", "live", "search", "browser", "ui", "html", "css", "landing", "frame"],
        "01-gpt-6-web",
    ),
];

/// **`manual_override`** — the user's manual override over the 5 sovereign models: aliases resolve,
/// unknown models fail closed, confidence is always `1.0`.
///
/// # Errors
/// [`RouterModeError::UnknownModelAlias`] for an unknown ID/alias.
pub fn manual_override(model: &str) -> Result<DualModeDecision, RouterModeError> {
    let Some(dir) = resolve_sovereign_model_alias(model) else {
        return Err(RouterModeError::UnknownModelAlias {
            model: model.trim().to_owned(),
        });
    };
    let exact = SOVEREIGN_MODEL_DIRS.contains(&dir) && dir.eq_ignore_ascii_case(model.trim());
    let matched_alias = (!exact || !dir.eq(model.trim())).then(|| model.trim().to_owned());
    Ok(DualModeDecision {
        selected_dir: dir,
        selected_model: display_name_for_dir(dir).to_owned(),
        selection_mode: "manual_override",
        confidence_score: 1.0,
        decision_rationale: format!(
            "manual override: user pinned `{}` → resolved to `{dir}` across the 5-model 45-file genome matrix",
            model.trim()
        ),
        matched_alias: if dir.eq(model.trim()) { None } else { matched_alias },
    })
}

/// **`agent_auto_decision`** — the automatic router: prompt telemetry (keywords,
/// tool density, math/quant depth, UI/web need, Rust/Solidity complexity) is analyzed
/// and the dominant signal's champion model is selected; ties → matrix order;
/// no signal → the general web/UI champion (`01-gpt-6-web`) with an honest rationale.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)] // a small bounded score
pub fn agent_auto_decision(task_prompt: &str) -> DualModeDecision {
    let needle = task_prompt.to_ascii_lowercase();
    let mut scores: [i64; 5] = [0; 5];
    let mut matched_signals: Vec<&str> = Vec::new();
    let mut total_hits = 0i64;
    for (signal, keywords, dir) in ROUTER_TELEMETRY_SIGNALS {
        let hits: i64 = keywords
            .iter()
            .filter(|kw| needle.contains(**kw))
            .count()
            .try_into()
            .unwrap_or(0);
        if hits > 0 {
            matched_signals.push(signal);
            total_hits += hits;
        }
        if let Some(idx) = SOVEREIGN_MODEL_DIRS.iter().position(|d| *d == *dir) {
            scores[idx] += hits;
        }
    }
    let mut best_idx = 0usize;
    for (idx, &score) in scores.iter().enumerate() {
        if score > scores[best_idx] {
            best_idx = idx;
        }
    }
    let dir = SOVEREIGN_MODEL_DIRS[best_idx];
    let rationale = if matched_signals.is_empty() {
        format!(
            "agent auto decision: no telemetry signals matched in prompt — defaulting to general \
             web/UI champion `{dir}` (matrix fallback, fail-honest)"
        )
    } else {
        format!(
            "agent auto decision: telemetry signals [{}] with {total_hits} keyword \
             hit(s) → champion `{dir}` (ties broken by matrix order)",
            matched_signals.join(", ")
        )
    };
    let confidence = if matched_signals.is_empty() {
        0.55_f32
    } else {
        (0.55 + 0.08 * f64::from(u32::try_from(total_hits).unwrap_or(u32::MAX)).min(5.0)) as f32
    };
    DualModeDecision {
        selected_dir: dir,
        selected_model: display_name_for_dir(dir).to_owned(),
        selection_mode: "agent_auto_decision",
        confidence_score: confidence.min(0.95),
        decision_rationale: rationale,
        matched_alias: None,
    }
}

/// Executes [`ModelSelectionMode`] — the only on-wire entry point (`route-model`).
///
/// # Errors
/// [`RouterModeError::UnknownModelAlias`] only for an unknown manual override.
pub fn route_with_selection_mode(
    mode: &ModelSelectionMode,
    task_prompt: &str,
) -> Result<DualModeDecision, RouterModeError> {
    match mode {
        ModelSelectionMode::ManualOverride { model } => manual_override(model),
        ModelSelectionMode::AgentAutoDecision => Ok(agent_auto_decision(task_prompt)),
    }
}

/// The canonical folder's display name — consistent with the builtin registry.
fn display_name_for_dir(dir: &str) -> &'static str {
    match dir {
        "01-gpt-6-web" => "GPT 6-Web",
        "02-gemini-4-aragon-high" => "GEMINI 4 ARAGON HIGH",
        "03-deepseek-3.2-web-think-mode" => "DeepSeek-3.2-WEB-Think-mode",
        "04-claude-sonnet-5.5-extra" => "Cluade-sonnet-5.5-extra",
        "05-glm-5.3-flash-max" => "GLM 5.3 flash MAX",
        _ => "unknown-sovereign-model",
    }
}

#[allow(clippy::cast_precision_loss)] // fixed 0..=1 scores — `f32` precision suffices
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
        display_name: model_id.to_string(),
        effort_level: None,
        forbids_chatty_filler: false,
        supports_zero_copy_rust_synthesis: false,
        enforce_clippy_1_100_backticks: false,
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

// ─────────────────────────── Prompt compiler ───────────────────────────

/// Model-specific prompt compilation: for no-Think profiles the
/// `<pre_computation_state_trace>` table becomes mandatory before code, the anti-Canvas guard
/// follows, invariants become fuzzy gates ≤N, and the anti-elision contract closes it out.
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
        let _ = writeln!(out, "### Phase {} — Invariant Gate", phase + 1);
        for rule in &invariants[phase * batch..((phase + 1) * batch).min(invariants.len())] {
            let _ = writeln!(out, "- {rule}");
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

// ─────────────────────────── Target #02: the `gemini-4-aragon-high` compiler and scrubber ───────────────────────────

/// The target #02 prompt compiler — `EFFORT LEVEL` 1.00 with live Think (no pre-computation
/// table), the Kian `SYSTEM DNA` anchor, the chatty-filler ban, and the
/// `clippy-1.100` backtick contract when `strict_rust_clippy` is on.
#[must_use]
pub fn compile_prompt_for_gemini_aragon_high(
    system_dna: &str,
    user_task: &str,
    strict_rust_clippy: bool,
) -> String {
    let mut out = String::with_capacity(2048);
    out.push_str("# TASK CONTRACT — GEMINI 4 Aragon HIGH (effort 1.00)\n\n");
    out.push_str("## SYSTEM DNA (verbatim anchor — never contradict)\n");
    out.push_str(system_dna.trim());
    out.push_str("\n\n");
    out.push_str("## EFFORT LEVEL: 1.00 — maximum reasoning depth\n");
    out.push_str("- Think Mode is ACTIVE (hybrid_thinking): reason to full depth internally; \
        never emit hidden chain-of-thought or `<thought>` markers.\n");
    out.push_str("- No chatty filler: no `Sure!`, `Certainly!`, `Of course!`, \
        `I hope this helps`, `Let me know if` — answer starts with substance.\n");
    out.push_str("- Prefer zero-copy Rust synthesis: borrows over clones, `get_mut` over \
        direct indexing, no unnecessary `String` allocation.\n\n");
    out.push_str("## TASK\n");
    out.push_str(user_task.trim());
    out.push_str("\n\n");
    if strict_rust_clippy {
        out.push_str("## RUST CONTRACT (clippy 1.100 pedantic, `-D warnings`)\n");
        out.push_str("- `#![forbid(unsafe_code)]`; every pub item documented; technical \
            identifiers/paths/types in doc comments wrapped in backticks (e.g. `IPv4`, `CAS`).\n");
        out.push_str("- Compiles on first write: no `unwrap()` outside tests, `# Errors` \
            section on fallible public functions.\n\n");
    }
    out.push_str("## OUTPUT CONTRACT (ANTI-ELISION)\n");
    out.push_str("- Never emit `// ... existing code` or elided placeholders; complete code only.\n");
    out.push_str("- End with a verification statement separating proven facts from estimates.\n");
    out
}

/// Meta markers that 'leaked' from the prompt into the output — their line is stripped.
const GEMINI_LEAKED_MARKERS: &[&str] =
    &["SPECIAL INSTRUCTION: think silently", "EFFORT LEVEL:"];

/// Conversational fillers forbidden by the profile's `forbids_chatty_filler` — their
/// leading/trailing line is stripped (fenced code content is untouched).
const GEMINI_CHATTY_FILLERS: &[&str] = &[
    "sure!",
    "certainly!",
    "of course!",
    "great question",
    "i hope this helps",
    "let me know if you",
    "feel free to ask",
    "here is what i came up with",
];

/// The `gemini-4-aragon-high` output scrubber: strips leaked meta markers
/// (`SPECIAL INSTRUCTION: think silently`, `EFFORT LEVEL:`), hidden
/// `<thought>`/`<think>` blocks, and conversational filler —
/// ```rust``` fence content and valid code stay untouched.
#[must_use]
pub fn sanitize_gemini_aragon_output(raw: &str) -> String {
    // 1) hidden think blocks — multi-line, non-greedy
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(open) = rest.find("<thought>").or_else(|| rest.find("<think>")) {
        out.push_str(&rest[..open]);
        let tag_len = if rest[open..].starts_with("<thought>") { 9 } else { 7 };
        let after_open = &rest[open + tag_len..];
        let close_tag = if tag_len == 9 { "</thought>" } else { "</think>" };
        rest = match after_open.find(close_tag) {
            Some(close) => &after_open[close + close_tag.len()..],
            None => "", // open without a close — stripped to the end
        };
    }
    out.push_str(rest);
    let scrubbed = out;

    // 2) leaked meta lines are removed entirely; conversational filler is stripped only as
    // a line prefix so the meaningful remainder of the line (the real answer) stays.
    let mut lines: Vec<String> = Vec::new();
    for line in scrubbed.lines() {
        let lower = line.trim_start().to_ascii_lowercase();
        let leaked = GEMINI_LEAKED_MARKERS
            .iter()
            .any(|m| lower.contains(&m.to_ascii_lowercase()));
        if leaked {
            continue;
        }
        let mut work = line.trim_start().to_string();
        let mut stripped = false;
        for filler in GEMINI_CHATTY_FILLERS {
            if work.to_ascii_lowercase().starts_with(*filler) {
                work = work[filler.len()..]
                    .trim_start_matches(['!', '.', ',', ':', ' '])
                    .to_string();
                stripped = true;
            }
        }
        if stripped {
            let remainder = work.trim_end();
            if remainder.is_empty() {
                continue;
            }
            lines.push(remainder.to_string());
        } else {
            lines.push(line.to_string());
        }
    }
    let joined = lines.join("\n");
    joined.trim().to_string()
}

/// The `caveman` compression-mode intensity (the `mattpocock/skills` vault) —
/// telegraphic output with 100% technical/code fidelity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CavemanIntensity {
    /// Off — full prose allowed.
    Off,
    /// Light — no pleasantries or task restatement; bullets over paragraphs.
    Lite,
    /// Full — telegraphic fragments; no adjectives/articles; ~75% output-token reduction.
    Full,
    /// Ultra — word fragments; code and invariants only; maximum compression.
    Ultra,
}

/// The system directive per `caveman` intensity — for injection into the prompt compiler.
/// Target rate: `Full`/`Ultra` ≈ ~75% output-token reduction with 100% technical fidelity.
#[must_use]
pub fn build_caveman_directive(intensity: CavemanIntensity) -> &'static str {
    match intensity {
        CavemanIntensity::Off => "",
        CavemanIntensity::Lite => {
            "CAVEMAN LITE: no pleasantries, no restating the task, no summaries of what you \
             are about to do; bullets over paragraphs; keep all code, tests, and error paths \
             verbatim."
        }
        CavemanIntensity::Full => {
            "CAVEMAN FULL: telegraphic fragments only — drop articles, adjectives and \
             transitions; zero filler; ~75% output token reduction; code, invariants and \
             error paths stay 100% verbatim and technically exact."
        }
        CavemanIntensity::Ultra => {
            "CAVEMAN ULTRA: single words and fragments; emit only code, tables and invariants; \
             explanations only when explicitly asked; ~75%+ output token reduction; 100% \
             technical/code accuracy preserved — never truncate code."
        }
    }
}

// ─────────────────────────── Handoff packet ───────────────────────────

/// The cross-model handoff packet — schema-aligned with `KIA-BRAIN-OS/07_LINK_Protocol.md`.
/// A delivery without `base_sha` or with unverified claims is rejected at the builder level.
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
    /// The Kian-to-Kian packet for the current step — the builder-ready template.
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

    /// Deterministic JSON serialization (for A2A and the witness log).
    ///
    /// # Errors
    /// Impossible in practice (the structure is only `String`/`Vec<String>`); for an honest
    /// field signature, a typed error is returned.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deterministic JSON decoding.
    ///
    /// # Errors
    /// `serde_json::Error` on a malformed structure.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ─────────────────────────── Tests ───────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const MOUNTED_GENOME_DIR: &str = "OMNI-MODEL-GENOME/legacy/models/gpt-6-standard-free";

    #[test]
    fn genome_layer_filenames_are_canonical() {
        assert_eq!(GenomeLayerId::ALL.len(), 9);
        assert_eq!(GenomeLayerId::TopologyAndHarness.filename(), "01_TOPOLOGY_AND_HARNESS.md");
        assert_eq!(GenomeLayerId::RoutingAndHandoffCard.filename(), "09_ROUTING_AND_HANDOFF_CARD.md");
        for (idx, layer) in GenomeLayerId::ALL.iter().enumerate() {
            let name = layer.filename();
            assert_eq!(name.get(0..2).unwrap(), format!("{:02}", idx + 1));
            assert!(
                std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("md")),
                "canonical genome layer file must end with .md"
            );
        }
    }

    // ── The pentamodel registry + dual-naming loader (Commit #27) ─────────────

    /// The canonical v4 names align with the layers and follow `01..09`.
    #[test]
    fn canonical_v4_filenames_mirror_layers() {
        assert_eq!(
            GenomeLayerId::TopologyAndHarness.canonical_v4_filename(),
            "01_IDENTITY_AND_PERSONA_GENOME.md"
        );
        assert_eq!(
            GenomeLayerId::RoutingAndHandoffCard.canonical_v4_filename(),
            "09_DISTILLED_CLONE_SYSTEM_PROMPT.md"
        );
        for (idx, layer) in GenomeLayerId::ALL.iter().enumerate() {
            let name = layer.canonical_v4_filename();
            assert_eq!(name.get(0..2).unwrap(), format!("{:02}", idx + 1));
        }
    }

    /// The alias resolver covers all 5 models — both #04 spellings and all three #05 spellings —
    /// and maps them to the canonical folder; unknown → `None` (no guessing).
    #[test]
    fn sovereign_alias_resolver_covers_all_five_models() {
        assert_eq!(SOVEREIGN_MODEL_DIRS.len(), 5);
        for (id, dir) in [
            ("GPT 6-Web", "01-gpt-6-web"),
            ("gpt-6-web", "01-gpt-6-web"),
            ("gpt-6-standard-free", "01-gpt-6-web"),
            ("GEMINI 4 ARAGON HIGH", "02-gemini-4-aragon-high"),
            ("DeepSeek-3.2-WEB-Think-mode", "03-deepseek-3.2-web-think-mode"),
            ("Cluade-sonnet-5.5-extra", "04-claude-sonnet-5.5-extra"),
            ("Claude-sonnet-5.5-extra", "04-claude-sonnet-5.5-extra"),
            ("GLM 5.3 flash MAX", "05-glm-5.3-flash-max"),
            ("glm-5.3-flash-max", "05-glm-5.3-flash-max"),
            ("GLM-5.3-Flash-Max", "05-glm-5.3-flash-max"),
        ] {
            assert_eq!(resolve_sovereign_model_alias(id), Some(dir), "{id}");
        }
        assert_eq!(resolve_sovereign_model_alias("gpt-5-turbo"), None);
        assert_eq!(resolve_sovereign_model_alias(""), None);
    }

    /// The builtin pentamodel profiles — 5 models with full 20-task scorecards.
    #[test]
    fn builtin_sovereign_penta_profiles_cover_five_models() {
        let profiles = builtin_sovereign_penta_profiles();
        assert_eq!(profiles.len(), 5);
        for (p, dir) in profiles.iter().zip(SOVEREIGN_MODEL_DIRS) {
            assert_eq!(p.model_id, dir);
            assert_eq!(p.workload_scores.len(), 20, "{}", p.model_id);
        }
        let glm = &profiles[4];
        assert_eq!(glm.display_name, "GLM 5.3 flash MAX");
        assert_eq!(glm.workload_scores[&EngineeringWorkload::FastAgenticToolCalling], 0.93);
        assert_eq!(glm.workload_scores[&EngineeringWorkload::MassiveRepoArchaeology], 0.45);
        // sub-second flash routing: sovereign model #05 (0.92) displaces the legacy
        // `llama4-cerebras-fast` (0.80) slot and ARAGON (0.65)
        let best_fast = ModelGenomeProfile::best_for_workload(&profiles, EngineeringWorkload::SubSecondIntentRouting)
            .expect("fast candidate");
        assert_eq!(best_fast.model_id, "05-glm-5.3-flash-max", "0.92 beats ARAGON 0.65");
    }

    /// The disk-backed pentamodel registry — all 5 folders load under canonical v4 names
    /// (the dual-naming loader; the legacy file is absent and canonical substitutes).
    #[test]
    fn sovereign_penta_registry_loads_from_disk_via_canonical_names() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../OMNI-MODEL-GENOME");
        let registry = load_sovereign_penta_registry(&root).expect("all 5 sovereign genomes must load");
        assert_eq!(registry.len(), 5);
        assert_eq!(
            registry.iter().map(|p| p.model_id.as_str()).collect::<Vec<_>>(),
            SOVEREIGN_MODEL_DIRS.to_vec()
        );
        for p in &registry {
            assert!(!p.reasoning_paradigm.is_empty(), "{}", p.model_id);
            assert!(p.display_name.len() > 3, "{}", p.model_id);
        }
        let glm = registry.iter().find(|p| p.model_id == "05-glm-5.3-flash-max").unwrap();
        assert_eq!(glm.display_name, "GLM 5.3 flash MAX");
        assert_eq!(
            glm.reasoning_paradigm, "single_pass_flash_with_parallel_tool_batching",
            "frontmatter of canonical 01 file must be read via dual-name loader"
        );
        assert_eq!(
            glm.workload_scores[&EngineeringWorkload::FastAgenticToolCalling], 0.93,
            "GLM 09 scorecard row 1 parsed from the canonical card"
        );
    }

    /// the dual-naming loader: a legacy folder (`01_TOPOLOGY..09_ROUTING` names) still
    /// loads unchanged — old behavior sees no regression
    #[test]
    fn legacy_layer_names_still_load_through_dual_name_loader() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../")
            .join("OMNI-MODEL-GENOME/legacy/models/gemini-4-aragon-high");
        let profile = load_model_genome_from_dir(&dir).expect("legacy genome must load unchanged");
        assert_eq!(profile.model_id, "gemini-4-aragon-high");
        assert_eq!(profile.workload_scores.len(), 20);
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
            "\u{6a9}\u{62f} \u{645}\u{627}\u{698}\u{648}\u{644} \u{62f}\u{641}\u{62a}\u{631} \u{62f}\u{648}\u{628}\u{644} \u{631}\u{627} \u{628}\u{646}\u{648}\u{6cc}\u{633}",
            &["SUM(Posting.amount) = 0", "466 citizens untouched", "20 active products"],
        );
        assert!(prompt.contains("<pre_computation_state_trace>"), "state trace gate: {prompt}");
        assert!(prompt.contains("Token Order = Thought Order"));
        assert!(prompt.contains("Do NOT invoke Canvas / canmore"), "anti-canvas guard: {prompt}");
        assert!(prompt.contains("Anti-Elision") || prompt.contains("ANTI-ELISION"));
        assert!(prompt.contains("SUM(Posting.amount) = 0"));
        // order: the pre-computation table comes before the fuzzy gates
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
        assert_eq!(profiles.len(), 13, "target #01 + target #02 + 11 plug-and-play slots");
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
        assert_eq!(best_frontend.model_id, "gemini-4-aragon-high", "0.96 beats claude 0.90");
        let best_deduction =
            ModelGenomeProfile::best_for_workload(&profiles, EngineeringWorkload::HardAlgorithmicDeduction)
                .expect("deduction candidate");
        assert_eq!(best_deduction.model_id, "gemini-4-aragon-high", "0.99 beats deepseek 0.92");
        let best_fast = ModelGenomeProfile::best_for_workload(&profiles, EngineeringWorkload::SubSecondIntentRouting)
            .expect("fast candidate");
        assert_eq!(best_fast.model_id, "llama4-cerebras-fast");
        // target #01 is never routed an infinite loop (0.39 = Do Not Route)
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
        // a profile from a folder with a tableless card 09 → an empty score map, not a fabricated number
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

    /// The builtin target #02 profile — the `EFFORT LEVEL` 1.00, anti-filler,
    /// zero-copy synthesis, and `clippy-1.100` backtick contracts.
    #[test]
    fn builtin_gemini_aragon_high_profile_matches_mounted_scorecard() {
        let p = builtin_gemini_aragon_high_profile();
        assert_eq!(p.model_id, "gemini-4-aragon-high");
        assert_eq!(p.display_name, "GEMINI 4 Aragon HIGH");
        assert!(p.think_mode_active, "hybrid thinking ON");
        assert!(!p.requires_external_state_trace, "Think ON → no trace table");
        assert_eq!(p.effort_level, Some(1.00));
        assert!(p.forbids_chatty_filler);
        assert!(p.supports_zero_copy_rust_synthesis);
        assert!(p.enforce_clippy_1_100_backticks);
        assert_eq!(p.workload_scores.len(), 20);
        assert_eq!(p.workload_scores[&EngineeringWorkload::TechnicalGenomeSynthesis], 1.00);
        assert_eq!(p.workload_scores[&EngineeringWorkload::HardAlgorithmicDeduction], 0.99);
        assert_eq!(p.workload_scores[&EngineeringWorkload::MassiveRepoArchaeology], 0.99);
        assert_eq!(p.workload_scores[&EngineeringWorkload::SubSecondIntentRouting], 0.65);
        assert_eq!(p.workload_scores[&EngineeringWorkload::LongHorizonAutonomousLoop], 0.99);
    }

    /// The disk-mounted `gemini-4-aragon-high` genome parses fully with the decorated
    /// card 09 (`**01**` / `` **`0.99`** ``).
    #[test]
    fn mounted_gemini_genome_loads_from_disk_with_full_scorecard() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../")
            .join("OMNI-MODEL-GENOME/legacy/models/gemini-4-aragon-high");
        let profile = load_model_genome_from_dir(&dir).expect("mounted gemini genome must load");
        assert_eq!(profile.model_id, "gemini-4-aragon-high");
        assert!(profile.reasoning_paradigm.contains("hybrid_thinking"), "{}", profile.reasoning_paradigm);
        assert!(profile.think_mode_active, "paradigm contains 'thinking' → Think ON");
        assert!(!profile.requires_external_state_trace);
        assert_eq!(profile.workload_scores.len(), 20, "decorated 20-row scorecard must parse");
        assert_eq!(profile.workload_scores[&EngineeringWorkload::SystemArchitecture], 0.99);
        assert_eq!(profile.workload_scores[&EngineeringWorkload::SecuritySsrfHardening], 0.98);
        assert_eq!(
            profile.calibration_status, "SOURCE_CLAIM_NOT_INDEPENDENTLY_VERIFIED",
            "honest calibration from 09 frontmatter"
        );
    }

    /// Target #02 routing: formal `UNSAT`/`CAS` reasoning, >128k context, and spec-compliance
    /// review → `gemini-4-aragon-high`; sub-second routing still goes
    /// to `llama4-cerebras-fast`.
    #[test]
    fn gemini_aragon_routes_for_formal_verification_and_massive_context() {
        let profiles = builtin_ecosystem_profiles();
        for workload in [
            EngineeringWorkload::HardAlgorithmicDeduction,
            EngineeringWorkload::MassiveRepoArchaeology,
            EngineeringWorkload::SpecComplianceReview,
            EngineeringWorkload::SqliteLedgerDesign,
            EngineeringWorkload::SecuritySsrfHardening,
        ] {
            let best = ModelGenomeProfile::best_for_workload(&profiles, workload)
                .unwrap_or_else(|| panic!("no candidate for {workload:?}"));
            assert_eq!(
                best.model_id, "gemini-4-aragon-high",
                "workload {workload:?} must route to target #02"
            );
        }
        let fast = ModelGenomeProfile::best_for_workload(&profiles, EngineeringWorkload::SubSecondIntentRouting)
            .expect("fast candidate");
        assert_eq!(fast.model_id, "llama4-cerebras-fast", "0.80 beats gemini 0.65");
    }

    /// The target #02 prompt compiler: the DNA anchor verbatim, `EFFORT LEVEL` 1.00, anti-filler,
    /// the `clippy-1.100` contract only when strict is requested, and no state-trace table.
    #[test]
    fn compile_gemini_prompt_includes_dna_effort_and_clippy_contract() {
        let dna = "sovereignty: zero-card zero-KYC; ux_priority_tier=1";
        let strict = compile_prompt_for_gemini_aragon_high(dna, "design CAS escrow", true);
        assert!(strict.contains(dna), "system DNA anchor verbatim: {strict}");
        assert!(strict.contains("EFFORT LEVEL: 1.00"));
        assert!(strict.contains("No chatty filler"));
        assert!(strict.contains("clippy 1.100 pedantic"));
        assert!(strict.contains("ANTI-ELISION"));
        assert!(!strict.contains("<pre_computation_state_trace>"), "Think ON: no trace gate");
        let loose = compile_prompt_for_gemini_aragon_high(dna, "t", false);
        assert!(!loose.contains("clippy 1.100"), "non-strict: no rust contract");
    }

    /// The scrubber: leaked meta (`SPECIAL INSTRUCTION`/`EFFORT LEVEL`),
    /// `<thought>` blocks, and filler are removed — ```rust``` fences and code stay untouched.
    #[test]
    fn sanitize_strips_leaked_markers_thought_blocks_and_filler_preserving_code() {
        let raw = "Sure! Here is my answer.\n\
                   SPECIAL INSTRUCTION: think silently about the ledger\n\
                   EFFORT LEVEL: 1.00 leaked into the reply\n\
                   <thought>secret chain of thought must vanish</thought>\n\
                   <think>also hidden</think>\n\
                   I hope this helps!\n\
                   ```rust\nfn cas_apply() -> u64 { 42 }\n```\n\
                   Let me know if you need more.";
        let clean = sanitize_gemini_aragon_output(raw);
        assert!(clean.contains("fn cas_apply"), "code fence preserved: {clean}");
        assert!(clean.contains("my answer"), "substantive prose preserved");
        assert!(!clean.contains("think silently"), "leaked meta marker stripped: {clean}");
        assert!(!clean.contains("EFFORT LEVEL"));
        assert!(!clean.contains("<thought>"));
        assert!(!clean.contains("<think>"));
        assert!(!clean.contains("secret chain of thought"));
        assert!(!clean.contains("Sure!"), "chatty filler stripped");
        assert!(!clean.contains("I hope this helps"));
        assert!(!clean.contains("Let me know if"));
        // an open think block without a close — stripped to the end
        let open = sanitize_gemini_aragon_output("ok<think>leak until end");
        assert_eq!(open, "ok");
    }

    /// The `caveman` mode: off = an empty string; intensities are distinct; `Full`/`Ultra`
    /// carry the ~75%-reduction-with-100%-fidelity contract.
    #[test]
    fn caveman_directives_are_graduated_and_off_is_empty() {
        assert_eq!(build_caveman_directive(CavemanIntensity::Off), "");
        let lite = build_caveman_directive(CavemanIntensity::Lite);
        let full = build_caveman_directive(CavemanIntensity::Full);
        let ultra = build_caveman_directive(CavemanIntensity::Ultra);
        assert!(lite.contains("CAVEMAN LITE") && !lite.contains("75%"));
        assert!(full.contains("CAVEMAN FULL") && full.contains("~75% output token reduction"));
        assert!(full.contains("100% verbatim"), "accuracy contract");
        assert!(ultra.contains("CAVEMAN ULTRA") && ultra.contains("~75%+"));
        assert!(ultra.contains("never truncate code"));
        // the intensities are strictly harsher than each other — a distinct string for all four states
        assert_ne!(lite, full);
        assert_ne!(full, ultra);
    }

    // ── Deterministic dual-mode selection — ManualOverride / AgentAutoDecision (Commit #28) ─

    /// `manual_override` — aliases resolve for all 5 models; an exact folder stays
    /// (`matched_alias` = `None`); unknown fails closed; confidence always 1.0.
    #[test]
    fn manual_override_resolves_all_five_models_and_fails_closed() {
        for (alias, dir) in [
            ("gpt-6-web", "01-gpt-6-web"),
            ("GLM 5.3 flash MAX", "05-glm-5.3-flash-max"),
            ("glm-5.3-flash-max", "05-glm-5.3-flash-max"),
            ("Cluade-sonnet-5.5-extra", "04-claude-sonnet-5.5-extra"),
            ("Claude-sonnet-5.5-extra", "04-claude-sonnet-5.5-extra"),
            ("DeepSeek-3.2-WEB-Think-mode", "03-deepseek-3.2-web-think-mode"),
            ("GEMINI 4 ARAGON HIGH", "02-gemini-4-aragon-high"),
        ] {
            let d = manual_override(alias).expect("known alias");
            assert_eq!(d.selected_dir, dir, "{alias}");
            assert_eq!(d.selection_mode, "manual_override");
            assert!((d.confidence_score - 1.0).abs() < f32::EPSILON);
            if d.selected_dir.eq_ignore_ascii_case(alias) {
                assert!(d.matched_alias.is_none(), "exact dir pin has no alias");
            } else {
                assert_eq!(d.matched_alias.as_deref(), Some(alias));
            }
        }
        // an exact folder path is also accepted
        let exact = manual_override("05-glm-5.3-flash-max").expect("exact dir");
        assert_eq!(exact.selected_dir, "05-glm-5.3-flash-max");
        assert!(exact.matched_alias.is_none());
        // unknown → fail-closed
        assert_eq!(
            manual_override("gpt-5-turbo"),
            Err(RouterModeError::UnknownModelAlias {
                model: "gpt-5-turbo".to_owned()
            })
        );
        assert!(matches!(
            manual_override("   "),
            Err(RouterModeError::UnknownModelAlias { .. })
        ));
    }

    /// `agent_auto_decision` — prompt telemetry is deterministic across all 5 champions;
    /// ties → matrix order; no signal → an honest fallback; the confidence cap is 0.95.
    #[test]
    fn agent_auto_decision_routes_telemetry_deterministically_across_five() {
        let cases = [
            (
                "batch agentic tool loop with rapid patch verification, keep it fast",
                "05-glm-5.3-flash-max",
            ),
            (
                "backtest this quant strategy: sharpe, candles and numeric proof",
                "03-deepseek-3.2-web-think-mode",
            ),
            (
                "audit this rust borrow checker issue and the solidity gas reentrancy",
                "04-claude-sonnet-5.5-extra",
            ),
            (
                "summarize a long-form multimodal document with vision and audio context",
                "02-gemini-4-aragon-high",
            ),
            (
                "build a live web ui landing frame with html css search",
                "01-gpt-6-web",
            ),
        ];
        for (prompt, dir) in cases {
            let d = agent_auto_decision(prompt);
            assert_eq!(d.selected_dir, dir, "prompt: {prompt}");
            assert_eq!(d.selection_mode, "agent_auto_decision");
            assert!(d.confidence_score > 0.55, "signal confidence: {}", d.confidence_score);
            assert!(
                d.decision_rationale.contains("telemetry signals"),
                "{}",
                d.decision_rationale
            );
        }
        // determinism — same input, same decision
        let a = agent_auto_decision("agentic tool batch");
        let b = agent_auto_decision("agentic tool batch");
        assert_eq!(a, b);
        // no signal → the matrix fallback with an honest rationale
        let fallback = agent_auto_decision("hello world");
        assert_eq!(fallback.selected_dir, "01-gpt-6-web");
        assert!(fallback.decision_rationale.contains("no telemetry signals"));
        assert!((fallback.confidence_score - 0.55).abs() < f32::EPSILON);
        // the confidence cap — 0.95
        let saturated =
            agent_auto_decision("fast flash quick agentic tool batch rapid patch multi-tool instant");
        assert!((saturated.confidence_score - 0.95).abs() < 1e-6);
    }

    /// `route_with_selection_mode` — `ManualOverride` dominates telemetry and
    /// `AgentAutoDecision` runs the same deterministic router; the wire is `snake_case`.
    #[test]
    fn route_with_selection_mode_manual_beats_auto_and_serializes() {
        let manual = route_with_selection_mode(
            &ModelSelectionMode::ManualOverride {
                model: "GLM 5.3 flash MAX".to_owned(),
            },
            "quant backtest please",
        )
        .expect("manual wins");
        assert_eq!(manual.selected_dir, "05-glm-5.3-flash-max");
        assert_eq!(manual.selection_mode, "manual_override");
        let auto = route_with_selection_mode(
            &ModelSelectionMode::AgentAutoDecision,
            "quant backtest please",
        )
        .expect("auto");
        assert_eq!(auto.selected_dir, "03-deepseek-3.2-web-think-mode");
        assert_eq!(auto.selection_mode, "agent_auto_decision");
        // the manual-override error also stays fail-closed on the wire
        assert!(matches!(
            route_with_selection_mode(
                &ModelSelectionMode::ManualOverride { model: "nope".to_owned() },
                "anything"
            ),
            Err(RouterModeError::UnknownModelAlias { .. })
        ));
        // `snake_case` serialization on the wire
        assert_eq!(
            serde_json::to_value(&ModelSelectionMode::AgentAutoDecision).unwrap(),
            serde_json::json!("agent_auto_decision")
        );
        assert_eq!(
            serde_json::to_value(&ModelSelectionMode::ManualOverride {
                model: "GLM 5.3 flash MAX".to_owned()
            })
            .unwrap(),
            serde_json::json!({ "manual_override": { "model": "GLM 5.3 flash MAX" } })
        );
    }
}
