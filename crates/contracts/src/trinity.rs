//! Wire contracts of the Trinity pipeline (Wave 3 — the sovereign `0xKIA` studio).
//!
//! This module's routes operate on the citizen's Bearer session:
//! `POST /api/v1/builder/trinity/alpha` — Body 1 (Alpha; discovery and locking of the master prompt)
//! and `POST /api/v1/builder/trinity/orchestrate` — Body 2 (the command center with
//! `OpenBot` subagents in the `3×9` pattern) + Body 3 (`Mairon-150` arbitration and the
//! self-healing loop). The wire shapes mirror the deterministic `kian-hermes::trinity_pipeline`
//! engine — no logic is written here (ADR-005).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// State of the five Alpha discovery slots — returned deterministically to the client for the next turn.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinitySlotsState {
    /// Core purpose and product identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_objective: Option<String>,
    /// Target audience and UX feel in 390 pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_audience_and_ux: Option<String>,
    /// Interactive capabilities of the first frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactive_features: Option<String>,
    /// Unauthenticated data and APIs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_and_zero_auth_apis: Option<String>,
    /// Monetization and the viral loop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monetization_and_viral_loop: Option<String>,
}

/// Alpha discovery turn request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityAlphaRequest {
    /// Trinity session ID — optional; the server creates `trn:{uuid}` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// The user's raw message in this turn (`1..=4000` characters).
    pub user_message: String,
    /// Slots filled in previous turns — optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_slots: Option<TrinitySlotsState>,
}

/// Wire view of the compiled master prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityMasterPromptView {
    /// SHA-256 fingerprint of the compiled text.
    pub spec_hash: String,
    /// The exact system directive.
    pub system_directive: String,
    /// Performance spec from the five slots.
    pub functional_spec: String,
    /// UX constraints in 390 pixels.
    pub ux_390_constraints: String,
    /// Verification criteria.
    pub verification_criteria: Vec<String>,
    /// Full master prompt text.
    pub compiled_prompt: String,
    /// Derived product name.
    pub project_name: String,
    /// Mairon's price suggestion in Telegram Stars (`XTR`).
    pub price_stars_hint: u64,
}

/// Wire view of one project-structure module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityBlueprintModuleView {
    /// Module relative path (`data-module` in the artifact).
    pub path: String,
    /// Functional role.
    pub role: String,
    /// Dependencies.
    pub dependencies: Vec<String>,
    /// Target subagent specialty (wire ID).
    pub specialty: String,
    /// Persian label of the specialty.
    pub specialty_fa: String,
}

/// Wire view of the project structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityBlueprintView {
    /// Product name.
    pub project_name: String,
    /// Safe Latin name (`slug`).
    pub project_slug: String,
    /// Modules in deterministic order.
    pub modules: Vec<TrinityBlueprintModuleView>,
}

/// One Alpha discovery turn's response — either continue discovery or lock delivery to the command center.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityAlphaResponse {
    /// State (`needs_more_discovery` | `locked_for_command_center`).
    pub status: String,
    /// Trinity session ID.
    pub session_id: String,
    /// The creator citizen's account.
    pub citizen_id: String,
    /// Alpha's Persian reply (spark or delivery).
    pub alpha_reply_fa: String,
    /// Raw slots in deterministic order (wire IDs).
    pub missing_slots: Vec<String>,
    /// Readiness percentage (`0..=100`).
    pub readiness_pct: u8,
    /// Up-to-date slot states — returned for the next turn.
    pub updated_slots: TrinitySlotsState,
    /// Master prompt — only in the locked state.
    pub master_prompt: Option<TrinityMasterPromptView>,
    /// Project structure — only in the locked state.
    pub blueprint: Option<TrinityBlueprintView>,
}

/// Wire view of a subagent's deterministic random alleles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityAlleleView {
    /// Surgical density.
    pub surgical_density: f64,
    /// UX craftsmanship skill.
    pub ux_craft: f64,
    /// Edge-case paranoia.
    pub edge_case_paranoia: f64,
    /// Execution speed.
    pub execution_velocity: f64,
    /// Allele seed (deterministic reproduction).
    pub allele_seed: u64,
}

/// Wire view of an `OpenBot` subagent's DNA manifest (`3×9` pattern).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinitySubAgentDnaView {
    /// Row number of the subagent.
    pub subagent_index: u32,
    /// Specialty (wire ID).
    pub specialty: String,
    /// Persian label of the specialty.
    pub specialty_fa: String,
    /// Assigned module(s).
    pub assigned_modules: Vec<String>,
    /// Deterministic thread ID.
    pub thread_id: String,
    /// Effective command-center IQ (`139`).
    pub logic_iq: u16,
    /// Effective command-center EQ (`129`).
    pub logic_eq: u16,
    /// Stop-hook soft stall boundary (`450s`).
    pub watchdog_soft_secs: u64,
    /// Stop-hook hard timeout boundary (`1200s`).
    pub watchdog_hard_secs: u64,
    /// `Caveman` compression mode (`off|lite|full|ultra`).
    pub caveman: String,
    /// Random alleles.
    pub alleles: TrinityAlleleView,
    /// Layer 1 length (pinned constant core).
    pub layer1_chars: usize,
    /// Layer 2 length (pinned session context).
    pub layer2_chars: usize,
    /// Layer 3 length (dynamic assignment).
    pub layer3_chars: usize,
    /// Preview of the rendered prompt (first ~220 characters).
    pub prompt_preview: String,
    /// Length of the compiled genome XML (`<openbot_subagent_dna>`).
    pub xml_prompt_chars: usize,
    /// Experimental probes of factor f08.
    pub empirical_probes: Vec<String>,
    /// Delivery card of factor f09.
    pub handoff_card: String,
    /// f07 cleanup gate — zero unfinished markers/secrets/slop.
    pub sanitizer_passed: bool,
}

/// The four REA pillars on the wire (`0..=25` each).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityPillarsView {
    /// Architecture.
    pub architecture: u8,
    /// Security hardening.
    pub security_hardening: u8,
    /// UX and anti-slop.
    pub ux_and_anti_slop: u8,
    /// Verification evidence.
    pub verification_proof: u8,
}

/// Wire view of a Mairon repair command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityRepairDirectiveView {
    /// Repair pass (`1..=3`).
    pub attempt: u32,
    /// Faulted specialty (wire ID).
    pub target_specialty: String,
    /// REA pillar below threshold.
    pub pillar: String,
    /// Fix directive (English).
    pub directive_en: String,
    /// Fix directive (Persian).
    pub directive_fa: String,
}

/// One repair round on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityRepairRoundView {
    /// Repair pass.
    pub attempt: u32,
    /// Score before repair.
    pub score_before: u8,
    /// Score after repair.
    pub score_after: u8,
    /// Gate pass at `>= 85`.
    pub approved: bool,
    /// Directives of this pass.
    pub directives: Vec<TrinityRepairDirectiveView>,
}

/// Wire view of Mairon's `IQ=150 / EQ=145` strategic advisory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityAdvisoryView {
    /// Judge overclock IQ (`150`).
    pub iq: u16,
    /// Judge overclock EQ (`145`).
    pub eq: u16,
    /// Game-theoretic horizon (`8`).
    pub lookahead_horizon: u8,
    /// Theory-of-mind order (`3`).
    pub tom_order: u8,
    /// Ambition factor (`0.99`).
    pub ambition_gain: f32,
    /// Summary of the eight-step outlook.
    pub eight_step_lookahead_summary: String,
    /// Buyer psychological hook and price/referral (Persian, ToM-3).
    pub third_order_tom_market_hook_fa: String,
}

/// Command-center execution request + Mairon arbitration — the locked Alpha slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityOrchestrateRequest {
    /// The locked Trinity session ID.
    pub session_id: String,
    /// Deterministic session seed — optional; by default derived from `SHA-256(session_id)`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_seed: Option<u64>,
    /// Five slots — readiness must be `>= 80`.
    pub slots: TrinitySlotsState,
}

/// The full Trinity forging response — DNA manifest, artifact, Mairon verdict, and repair history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrinityOrchestrateResponse {
    /// Trinity session ID.
    pub session_id: String,
    /// Product name.
    pub project_name: String,
    /// The fully assembled HTML artifact.
    pub artifact_html: String,
    /// Artifact size in bytes.
    pub artifact_bytes: usize,
    /// Number of injected experimental probes.
    pub probe_count: usize,
    /// Integrated modules (`data-module`).
    pub integrated_modules: Vec<String>,
    /// DNA manifest of the `OpenBot` fleet.
    pub manifest: Vec<TrinitySubAgentDnaView>,
    /// Mairon's final score (`0..=100`).
    pub final_score: u8,
    /// Forging gate pass (`>= 85` and zero red lines).
    pub approved_for_listing: bool,
    /// Mairon's visible badge.
    pub mairon_badge: String,
    /// Breakdown of the four pillars.
    pub pillars: TrinityPillarsView,
    /// Strategic advisory `IQ=150/EQ=145`.
    pub advisory: TrinityAdvisoryView,
    /// Repair attempts consumed (`0..=3`).
    pub repair_attempts_used: u32,
    /// History of repair rounds.
    pub repair_history: Vec<TrinityRepairRoundView>,
}
