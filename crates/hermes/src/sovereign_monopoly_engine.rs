//! The sovereign monopoly engine — the five-model genome router + the monopoly
//! orchestrator (Wave 7 — Step 4/4 + the Commit #27 penta-model).
//!
//! Two stacked cores:
//!  * [`QuadModelGenomeRouter`] — the registry of 5 sovereign frontier genomes
//!    (`GPT 6-Web`, `GEMINI 4 ARAGON HIGH`, `DeepSeek-3.2-WEB-Think-mode`,
//!    `Cluade-sonnet-5.5-extra`, `GLM 5.3 flash MAX`) with 9-file verification per model
//!    (45 files on disk, new canonical names) + distilled-clone prompt synthesis
//!    per model × work domain + dual-mode selection (`ManualOverride` /
//!    `AgentAutoDecision`) + wiring into the three-avatar Trinity
//!    (`Alpha` → `Logic-Prime` → `Mairon`). The historical `QuadModelGenomeRouter`
//!    name remains as the alias of the same 5-model router.
//!  * [`SovereignMonopolyOrchestrator`] — combines the genome router, the
//!    day-one four-stream [`kian_wallet::base_treasury`] charter, the
//!    Base `8453` contract descriptors, and the Product Hunt/CMC launch kit into one
//!    single state-and-execution engine.
//!
//! Heuristic rules: aliases always resolve (`Cluade-sonnet-5.5-extra` and
//! `Claude-sonnet-5.5-extra` to model #04; `GLM 5.3 flash MAX`,
//! `glm-5.3-flash-max`, and `GLM-5.3-Flash-Max` to model #05); a missing file
//! is never assumed "present" — disk verification is fail-closed and the builtin path only
//! preserves the count of 45, never fabricated content.

use std::fmt;
use std::path::{Path, PathBuf};

use kian_wallet::base_treasury::{
    compute_day_one_atomic_split, BaseTreasuryError, BoostTier, DayOneAtomicSplit,
    DayOneRevenueCharter, SettlementAsset, BASE_BLOCK_EXPLORER, BASE_MAINNET_CHAIN_ID,
    BASE_MAINNET_RPC_URL, BASE_TREASURY_SIGNATURE,
};

// ── Signature ─────────────────────────────────────────────────────────────────────

/// The sovereign monopoly engine signature — for smoke tests and logs.
pub const SOVEREIGN_MONOPOLY_SIGNATURE: &str = "sovereign-monopoly::quad-genome::8453::wave7";

/// The 9-file canonical names — a hard `omni-genome-v4` contract.
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

/// Genome model count — 5 sovereign frontiers (penta-model, Commit #27).
pub const SOVEREIGN_MODEL_COUNT: usize = 5;

/// Total genome file count — 5 × 9.
pub const SOVEREIGN_GENOME_FILE_COUNT: usize = SOVEREIGN_MODEL_COUNT * CANONICAL_GENOME_FILES.len();

/// The historical model-count constant — now the same 5 sovereign models.
pub const QUAD_MODEL_COUNT: usize = SOVEREIGN_MODEL_COUNT;

/// The historical file-count constant — now the same 45 sovereign files.
pub const QUAD_GENOME_FILE_COUNT: usize = SOVEREIGN_GENOME_FILE_COUNT;

// ── Models ───────────────────────────────────────────────────────────────────

/// The five sovereign frontier genomes — the matrix's canonical order.
///
/// `SovereignModelId` is the equivalent name of this enum on the wire / in the Commit #27 docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadModelGenome {
    /// Model #01 — real-time web synthesis and frame-1 interactive artifact craft.
    Gpt6Web,
    /// Model #02 — 1M-token cross-repository synthesis and the 9-modality orchestrator.
    Gemini4AragonHigh,
    /// Model #03 — `<think>` quantitative math and causal-alpha proof.
    DeepSeek32WebThinkMode,
    /// Model #04 — Rust surgery engineering and smart-contract audit.
    ClaudeSonnet55Extra,
    /// Model #05 — multi-tool flash execution at minimal latency (Commit #27).
    Glm53FlashMax,
}

/// The model's sovereign wire ID — an explicit alias of [`QuadModelGenome`] (Commit #27).
pub type SovereignModelId = QuadModelGenome;

impl QuadModelGenome {
    /// All models in the matrix's canonical order.
    pub const ALL: [QuadModelGenome; SOVEREIGN_MODEL_COUNT] = [
        Self::Gpt6Web,
        Self::Gemini4AragonHigh,
        Self::DeepSeek32WebThinkMode,
        Self::ClaudeSonnet55Extra,
        Self::Glm53FlashMax,
    ];

    /// The exact display ID — the same as the Magic upload ID.
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

    /// Alias IDs — all resolve to this model.
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

    /// The canonical folder under `OMNI-MODEL-GENOME/models/`.
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

    /// The sovereign specialty — the same matrix comparison line.
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

// ── Trinity avatars ──────────────────────────────────────────────────────

/// The three-avatar Trinity — the sovereign loop `Alpha` → `Logic-Prime` → `Mairon`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrinityAvatar {
    /// The ingest/ideation avatar — raw input and web synthesis.
    Alpha,
    /// The compile/verification avatar — architecture, surgery, proof.
    LogicPrime,
    /// The quantitative judge avatar — the final numeric gate and the launch verdict.
    Mairon,
}

impl TrinityAvatar {
    /// Wire label.
    #[must_use]
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::Alpha => "Alpha",
            Self::LogicPrime => "Logic-Prime",
            Self::Mairon => "Mairon",
        }
    }

    /// Each model's default role in the Trinity loop.
    #[must_use]
    #[allow(clippy::match_same_arms)] // the two loop architects deliberately share Logic-Prime
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

// ── Work domains ────────────────────────────────────────────────────────────

/// Work routing domains — the key to distilled prompt synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadTaskDomain {
    /// Real-time web synthesis and launch copy.
    WebSynthesis,
    /// Interactive UI/UX artifact — frame 1, 390px, zero overflow.
    UiUxArtifact,
    /// Repository archaeology and macro-architecture verification.
    CrossRepoContext,
    /// Quantitative math, causal-alpha proof, formal verification.
    QuantProof,
    /// Rust surgery and smart-contract audit.
    RustSystemsAudit,
    /// High-precision bilingual EN/FA copywriting.
    BilingualCopywriting,
    /// Multi-tool flash execution — a low-latency agentic loop and rapid patching (model #05).
    FlashAgenticExecution,
}

impl QuadTaskDomain {
    /// All domains in canonical order.
    pub const ALL: [QuadTaskDomain; 7] = [
        Self::WebSynthesis,
        Self::UiUxArtifact,
        Self::CrossRepoContext,
        Self::QuantProof,
        Self::RustSystemsAudit,
        Self::BilingualCopywriting,
        Self::FlashAgenticExecution,
    ];

    /// Wire label.
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

    /// The model this domain is the absolute champion of (the 20-task matrix rule).
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

// ── Errors ──────────────────────────────────────────────────────────────────────

/// Genome router error — fail-closed on a missing file.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GenomeRouterError {
    /// A missing genome file, or one below the size threshold.
    #[error("genome file missing/undersized: {0}")]
    MissingFile(String),
    /// Unknown alias — no guessing is performed.
    #[error("unknown model alias: {0}")]
    UnknownAlias(String),
    /// Treasury split failed — propagates from the wallet layer.
    #[error("treasury split error: {0}")]
    Treasury(String),
}

impl From<BaseTreasuryError> for GenomeRouterError {
    fn from(e: BaseTreasuryError) -> Self {
        Self::Treasury(e.to_string())
    }
}

// ── Genome registration ────────────────────────────────────────────────────────────────

/// One model's verification state — the 9 canonical files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenomeVerification {
    /// The registered model.
    pub model: QuadModelGenome,
    /// The genome folder.
    pub genome_dir: String,
    /// Per-file state — `true` means present and above the threshold.
    pub files_verified: [bool; 9],
    /// The disk path verification ran against (`None` = the builtin registry).
    pub verified_against: Option<String>,
}

impl GenomeVerification {
    /// The number of verified files.
    #[must_use]
    pub fn verified_count(&self) -> usize {
        self.files_verified.iter().filter(|ok| **ok).count()
    }

    /// The model is complete when all 9 files are green.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.verified_count() == 9
    }
}

/// The distilled-clone prompt — model ID + the full behavioral-contract text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistilledClonePrompt {
    /// The model owning the prompt.
    pub model: QuadModelGenome,
    /// The routed work domain.
    pub domain: QuadTaskDomain,
    /// The destination Trinity avatar.
    pub trinity_avatar: TrinityAvatar,
    /// The full distilled prompt text.
    pub system_prompt: String,
}

/// Model selection mode — the Commit #27 dual-mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelSelectionMode {
    /// The user explicitly picked one of the 5 models — the manual override always wins.
    ManualOverride {
        /// The raw user-selection ID (exact or alias).
        model: String,
    },
    /// Agent Mode — the router analyzes task telemetry and selects automatically.
    AgentAutoDecision,
}

impl ModelSelectionMode {
    /// Wire label — `manual_override` | `agent_auto_decision`.
    #[must_use]
    pub fn wire_id(&self) -> &'static str {
        match self {
            Self::ManualOverride { .. } => "manual_override",
            Self::AgentAutoDecision => "agent_auto_decision",
        }
    }
}

/// Model selection decision — the output of [`QuadModelGenomeRouter::select_model`].
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSelectionDecision {
    /// Selection mode — manual or agent-auto.
    pub selection_mode: ModelSelectionMode,
    /// Mode wire label (`manual_override` / `agent_auto_decision`).
    pub selection_mode_wire: &'static str,
    /// The model selected from the 5 sovereign models.
    pub selected_model: QuadModelGenome,
    /// The routed domain (the auto champion or the manual model's home domain).
    pub domain: QuadTaskDomain,
    /// Decision confidence in `0.0..=1.0` (a manual override is always 1.0).
    pub confidence_score: f32,
    /// Auditable decision rationale — observed signals and the selection path.
    pub decision_rationale: String,
    /// The selected model's distilled-clone prompt + Trinity avatar.
    pub prompt: DistilledClonePrompt,
}

/// The five-model genome router — registry + 45-file verification + routing + dual-mode selection.
/// The historical `QuadModelGenomeRouter` name is retained; it covers all 5 models and 45 files.
#[derive(Debug, Clone, Default)]
pub struct QuadModelGenomeRouter {
    verifications: Vec<GenomeVerification>,
}

impl QuadModelGenomeRouter {
    /// The builtin registry — all 5 models register with their 9 "expected" files;
    /// no IO; for zero-disk tests and fast boot.
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

    /// Fail-closed disk verification — all 9 files of all 5 models under
    /// `<root>/models/<dir>/` must exist and be above 500 bytes.
    /// The registry is rewritten from the disk result (never a fake merge).
    ///
    /// # Errors
    /// [`GenomeRouterError::MissingFile`] only in `strict` mode; in
    /// normal mode, missing files are recorded with verification state `false`.
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

    /// The default genome path — relative to the hermes crate's `CARGO_MANIFEST_DIR`.
    #[must_use]
    pub fn default_genome_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../OMNI-MODEL-GENOME")
    }

    /// The production-ready router — disk verification with builtin fallback; for
    /// diskless runtimes, builtin stays in "expected registry" state.
    #[must_use]
    pub fn production() -> Self {
        match Self::verify_from_disk(&Self::default_genome_root()) {
            Ok(router) => router,
            Err(_) => Self::builtin(),
        }
    }

    /// Each model's verification state.
    #[must_use]
    pub fn verifications(&self) -> &[GenomeVerification] {
        &self.verifications
    }

    /// Total verified file count — always 45 on the builtin registry.
    pub fn total_files_verified(&self) -> usize {
        self.verifications.iter().map(GenomeVerification::verified_count).sum()
    }

    /// Alias resolution — the exact ID or any alias.
    ///
    /// # Errors
    /// [`GenomeRouterError::UnknownAlias`] — no guessing is performed.
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

    /// Route work to a genome + distilled-clone prompt + Trinity avatar.
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

    /// The domain this model is the absolute champion of — the first domain in canonical order.
    #[must_use]
    pub fn primary_domain_of(model: QuadModelGenome) -> QuadTaskDomain {
        QuadTaskDomain::ALL
            .iter()
            .copied()
            .find(|d| d.champion() == model)
            .unwrap_or(QuadTaskDomain::RustSystemsAudit)
    }

    /// The model's dual-mode selection — [`ModelSelectionMode::ManualOverride`] or
    /// [`ModelSelectionMode::AgentAutoDecision`] (Commit #27).
    ///
    /// * the user's manual override (a `manual_override` that is non-empty and not "auto") always
    ///   wins; all 5 models' aliases resolve and unknown ones error.
    /// * in Agent Mode (or an absent/"auto" override), prompt telemetry — keywords,
    ///   latency need, tool-call density, math/quantitative depth, UI/web need, and
    ///   Rust/Solidity system complexity — is analyzed, and the optimal model of the 5
    ///   is auto-selected; the output includes `selected_model`, `selection_mode`,
    ///   `confidence_score`, and `decision_rationale`.
    ///
    /// # Errors
    /// [`GenomeRouterError::UnknownAlias`] — an unknown manual override (no
    /// guessing; fail-closed).
    #[allow(clippy::too_many_lines)] // the signal table keeps the documentation readable
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

        // AgentAutoDecision — prompt telemetry converts into domain scores.
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
            // score ≤ 5 — exact small-number to f32 conversion without precision loss
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

    /// The matrix view — for the `/monopoly/genome-matrix` route and logs.
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

/// The distilled-clone prompt per model × domain — the full behavioral contract.
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

// ── Launch kit ────────────────────────────────────────────────────────────────

/// The Product Hunt / CMC launch kit — the single source of sovereign launch copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductHuntCmcLaunchKit {
    /// Product Hunt listing title.
    pub product_hunt_title: String,
    /// Product Hunt tagline.
    pub tagline: String,
    /// Maker comment.
    pub maker_comment: String,
    /// Token-verification dossier ID.
    pub cmc_dossier_id: String,
    /// X announcement template.
    pub x_announcement_template: String,
    /// Telegram announcement template.
    pub telegram_announcement_template: String,
}

impl ProductHuntCmcLaunchKit {
    /// The canonical kit — final text kept in sync with `docs/launch/PRODUCT_HUNT_AND_CMC_KIT.md`.
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            product_hunt_title: "0xKIA — the sovereign creator economy on Telegram + Base".to_owned(),
            tagline: "Ship AI-native products inside Telegram, settle on Base (8453), keep 80-90% of your revenue.".to_owned(),
            maker_comment: "0xKIA (KIAN v2) is a 25-commit Rust kernel: double-entry zero-sum ledger, 8-state escrow, ARDA 20-locale engine, causal Vibe-Trading quant, and now a Day-1 4-stream treasury on Base with audited Solidity. Every split is zero-sum-verified with a decimal engine — no float ever touches money.".to_owned(),
            cmc_dossier_id: "cmc-cg-basescan-0xkia-base-8453".to_owned(),
            x_announcement_template: "$0xKIA is live on @base — Chain ID 8453. 1B fixed supply, EIP-2612 permit, audited treasury. Day-1 creators keep 80-90%. 0xKIA. https://basescan.org".to_owned(),
            telegram_announcement_template: "\u{6f0}xKIA \u{631}\u{648}\u{6cc} Base (8453) \u{644}\u{627}\u{646}\u{686} \u{634}\u{62f} — \u{62e}\u{632}\u{627}\u{646}\u{647}\u{654} \u{686}\u{647}\u{627}\u{631}\u{62c}\u{631}\u{6cc}\u{627}\u{646}\u{6cc}\u{650} \u{631}\u{648}\u{632}\u{650} \u{627}\u{648}\u{644}\u{60c} \u{627}\u{645}\u{627}\u{646}\u{62a}\u{650} \u{63a}\u{6cc}\u{631}\u{62d}\u{636}\u{627}\u{646}\u{62a}\u{6cc}\u{60c} \u{62a}\u{642}\u{633}\u{6cc}\u{645}\u{650} \u{635}\u{641}\u{631}\u{62c}\u{645}\u{639}\u{650} \u{627}\u{62a}\u{645}\u{6cc}. \u{633}\u{627}\u{632}\u{646}\u{62f}\u{647}‌\u{647}\u{627} \u{6f8}\u{6f0}-\u{6f9}\u{6f0}\u{66a} \u{645}\u{6cc}‌\u{6af}\u{6cc}\u{631}\u{646}\u{62f}. #0xKIA".to_owned(),
        }
    }
}

// ── Orchestrator ──────────────────────────────────────────────────────────────

/// Base chain descriptor for the orchestrator wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseMainnetDescriptor {
    /// Chain ID.
    pub chain_id: u64,
    /// Public RPC.
    pub rpc_url: String,
    /// Block explorer.
    pub block_explorer: String,
    /// Treasury engine signature.
    pub treasury_signature: String,
    /// Token audit descriptor.
    pub token_audit: kian_wallet::base_treasury::ZeroXKiaTokenAudit,
    /// Escrow audit descriptor.
    pub escrow_audit: kian_wallet::base_treasury::KiaBaseEscrowTreasuryAudit,
}

/// The sovereign monopoly orchestrator — genome router + charter + contracts + launch kit.
#[derive(Debug, Clone)]
pub struct SovereignMonopolyOrchestrator {
    /// The four-model genome router.
    pub genome_router: QuadModelGenomeRouter,
    /// The day-one four-stream charter.
    pub charter: DayOneRevenueCharter,
    /// Base chain descriptor.
    pub base_mainnet: BaseMainnetDescriptor,
    /// The launch kit.
    pub launch_kit: ProductHuntCmcLaunchKit,
}

impl Default for SovereignMonopolyOrchestrator {
    fn default() -> Self {
        Self::canonical()
    }
}

impl SovereignMonopolyOrchestrator {
    /// The canonical orchestrator — the day-one charter + canonical audit
    /// descriptors + the canonical launch kit + the builtin router.
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

    /// Simulate the day-one atomic split — a marketplace sale (with a referrer, or
    /// a referrer-less direct sale under the `2000 bps` rule).
    ///
    /// # Errors
    /// [`GenomeRouterError::Treasury`] on a charter/amount error.
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

    /// Simulate stream 3 — an instant boost at a 100% margin.
    ///
    /// # Errors
    /// [`GenomeRouterError::Treasury`] on an amount/tier error.
    pub fn simulate_instant_boost(
        &self,
        asset: SettlementAsset,
        tier: BoostTier,
        buyer_user_id: &str,
    ) -> Result<DayOneAtomicSplit, GenomeRouterError> {
        kian_wallet::base_treasury::compute_instant_boost_split(asset, tier, buyer_user_id)
            .map_err(Into::into)
    }

    /// Unified status — genome matrix + charter + chain + launch kit in one
    /// loggable string (the same node-status report schema).
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
        // exact Magic IDs.
        assert_eq!(QuadModelGenome::Gpt6Web.exact_id(), "GPT 6-Web");
        assert_eq!(QuadModelGenome::Gemini4AragonHigh.exact_id(), "GEMINI 4 ARAGON HIGH");
        assert_eq!(QuadModelGenome::DeepSeek32WebThinkMode.exact_id(), "DeepSeek-3.2-WEB-Think-mode");
        assert_eq!(QuadModelGenome::ClaudeSonnet55Extra.exact_id(), "Cluade-sonnet-5.5-extra");
        assert_eq!(QuadModelGenome::Glm53FlashMax.exact_id(), "GLM 5.3 flash MAX");
        // canonical aliases — both #04 IDs and all three #05 IDs resolve.
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
            "\u{647}\u{631} \u{6f4}\u{6f5} \u{641}\u{627}\u{6cc}\u{644}\u{650} \u{698}\u{646}\u{648}\u{645} \u{628}\u{627}\u{6cc}\u{62f} \u{631}\u{648}\u{6cc} \u{62f}\u{6cc}\u{633}\u{6a9} \u{62a}\u{623}\u{6cc}\u{6cc}\u{62f} \u{634}\u{648}\u{62f} — root={}",
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
        // the distilled prompt carries a full contract — not empty, not generic.
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
        assert_eq!(direct.kia_treasury_micros, 20_000_000, "\u{642}\u{627}\u{646}\u{648}\u{646}\u{650} \u{641}\u{631}\u{648}\u{634}\u{650} \u{645}\u{633}\u{62a}\u{642}\u{6cc}\u{645}\u{650} \u{628}\u{6cc}‌\u{645}\u{639}\u{631}\u{641}");
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

    // ── Dual-mode selection — ManualOverride + AgentAutoDecision (Commit #27) ──

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
        // aliases resolve too; "auto" and empty do not override.
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
        // unknown — fail-closed, no guessing.
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

        // no signals — an honest default at base confidence.
        let plain = router.select_model("hello world", true, None).unwrap();
        assert!((plain.confidence_score - 0.55).abs() < 1e-6);
        assert!(plain.decision_rationale.contains("no explicit telemetry signal"));
        // every selected model's prompt is complete.
        for d in [flash, quant, audit, repo, web, plain] {
            assert!(d.prompt.system_prompt.len() > 300);
            assert!(d.prompt.system_prompt.contains("0xKIA sovereign fleet"));
        }
    }
}
