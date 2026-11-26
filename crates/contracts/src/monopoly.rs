//! Wire contracts of the sovereign monopoly — the penta-model genome, the Day-1 charter, and the
//! Base treasury (Wave 7 — Step 4/4 + Commit #27 penta-model and dual-mode selection).
//!
//! On-wire DTOs:
//! - [`MonopolyGenomeMatrixResponse`] — the 5-model × 9-file matrix with the
//!   45-file verification status, routing rules, and both dual-mode selection capabilities;
//! - [`MonopolyRouteModelRequest`] / [`MonopolyRouteModelResponse`] — running
//!   `ManualOverride` or `AgentAutoDecision` across the 5 sovereign models with the
//!   distilled clone prompt, the 9-file verification proof, and the decision rationale;
//! - [`MonopolyCharterAndBaseResponse`] — the four-stream Day-1 charter +
//!   Base Mainnet configuration (`8453`) + the audit summary of both Solidity contracts;
//! - [`MonopolySimulateSplitRequest`] / [`MonopolySimulateSplitResponse`] —
//!   running `compute_day_one_atomic_split` (a referred sale, a direct
//!   unreferred sale under the `2000 bps` rule, or a 100% boost) + the zero-sum proof +
//!   the EIP-681 payment URI on Base.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── Genome matrix ─────────────────────────────────────────────────────────────

/// Verification status of one model in the genome matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyGenomeModelView {
    /// Exact model ID (`GPT 6-Web`, …).
    pub model_id: String,
    /// Alias IDs — all resolve.
    pub aliases: Vec<String>,
    /// Genome folder under `OMNI-MODEL-GENOME/models/`.
    pub genome_dir: String,
    /// Sovereign specialty.
    pub specialty: String,
    /// Names of the 9 canonical files.
    pub canonical_files: Vec<String>,
    /// Number of verified files (0 to 9).
    pub files_verified: usize,
    /// Full model when `9/9`.
    pub is_complete: bool,
    /// Default Traminity avatar (`Alpha` | `Logic-Prime` | `Mairon`).
    pub trinity_avatar: String,
    /// Verification source — disk path or `builtin-registry`.
    pub verified_against: String,
}

/// Domain → champion routing rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyRoutingRuleView {
    /// Work domain (`web-synthesis`, …).
    pub domain: String,
    /// The absolute champion model.
    pub champion_model_id: String,
    /// Destination Traminity avatar.
    pub trinity_avatar: String,
}

/// Response of `GET /api/v1/monopoly/genome-matrix`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyGenomeMatrixResponse {
    /// Monopoly engine signature.
    pub signature: String,
    /// The five sovereign models with 9-file verification status.
    pub models: Vec<MonopolyGenomeModelView>,
    /// Domain → champion routing rules.
    pub routing_rules: Vec<MonopolyRoutingRuleView>,
    /// Total verified files (target: 45).
    pub total_files_verified: usize,
    /// Hard contract target.
    pub expected_file_count: usize,
    /// Model selection modes on this matrix — always both:
    /// `manual_override` and `agent_auto_decision` (Commit #27).
    pub selection_modes: Vec<String>,
}

// ── Dual-mode model selection (Commit #27) ───────────────────────────────────────

/// Request of `POST /api/v1/monopoly/route-model` — dual-mode selection across the 5 sovereign models.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyRouteModelRequest {
    /// Task text — in `AgentAutoDecision` its telemetry is analyzed.
    pub task_prompt: String,
    /// Agent Mode — when `true` (or the actor is absent/`"auto"`), automatic selection runs.
    #[serde(default)]
    pub agent_mode: bool,
    /// The user's manual actor — exact or alias of any of the 5 models
    /// (`GLM 5.3 flash MAX` | `GPT 6-Web` | …); `null`/`"auto"` = automatic.
    pub manual_model_override: Option<String>,
}

/// Response of `POST /api/v1/monopoly/route-model` — the selected model + verification proof.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyRouteModelResponse {
    /// Selected model — exact ID (`GLM 5.3 flash MAX`, …).
    pub selected_model: String,
    /// Selection mode — `manual_override` | `agent_auto_decision`.
    pub selection_mode: String,
    /// Decision confidence in `0.0..=1.0` (manual actor always 1.0).
    pub confidence_score: f64,
    /// Auditable explanation of the decision — observed signals and the selection path.
    pub decision_rationale: String,
    /// Routed domain (`flash-agentic-execution`, …).
    pub routed_domain: String,
    /// Destination Traminity avatar.
    pub trinity_avatar: String,
    /// The selected model's distilled clone prompt (from the 9-file genome).
    pub distilled_system_prompt: String,
    /// Genome folder of the selected model (`models/05-glm-5.3-flash-max`, …).
    pub genome_dir: String,
    /// Verified files of the selected model — the 9-file proof.
    pub files_verified: usize,
    /// Hard contract target — always 9.
    pub expected_files: usize,
}

// ── Charter + Base ─────────────────────────────────────────────────────────────

/// View of one charter stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyCharterStreamView {
    /// Stream number (1 to 4).
    pub stream: u8,
    /// Stream name.
    pub name: String,
    /// Declared bps rate.
    pub bps: i64,
    /// Destination sovereign account.
    pub destination_account: String,
    /// One-line description.
    pub note: String,
}

/// Audit summary of one Solidity contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyContractAuditView {
    /// Contract name.
    pub contract_name: String,
    /// Source path in the repository.
    pub source_path: String,
    /// Solidity version.
    pub solidity_version: String,
    /// Architecture properties — key/value pairs.
    pub audit_points: Vec<MonopolyAuditPoint>,
}

/// One audit clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyAuditPoint {
    /// Key (`total_supply`, `post_constructor_mint`, …).
    pub key: String,
    /// Declared value.
    pub value: String,
}

/// Response of `GET /api/v1/monopoly/charter-and-base`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyCharterAndBaseResponse {
    /// Treasury engine signature.
    pub signature: String,
    /// Charter motto — "war first in exchange for peace last".
    pub charter_motto: String,
    /// The four Day-1 revenue streams.
    pub streams: Vec<MonopolyCharterStreamView>,
    /// Base Mainnet configuration.
    pub base_chain_id: u64,
    /// Public Base RPC.
    pub base_rpc_url: String,
    /// Block explorer.
    pub base_block_explorer: String,
    /// Base-native USDC contract.
    pub base_usdc_contract: String,
    /// Audit summary of `ZeroXKiaToken.sol`.
    pub token_audit: MonopolyContractAuditView,
    /// Audit summary of `KiaBaseEscrowTreasury.sol`.
    pub escrow_audit: MonopolyContractAuditView,
}

// ── Split simulation ─────────────────────────────────────────────────────────

/// Request of `POST /api/v1/monopoly/simulate-split`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolySimulateSplitRequest {
    /// Scenario — `marketplace_sale` or `instant_boost`.
    pub scenario: String,
    /// Settlement asset — `telegram-stars-xtr` | `base-native-eth` |
    /// `base-native-usdc` | `zero-x-kia-base-token` | `internal-micro-kian`.
    pub settlement_asset: String,
    /// Gross micro amount (10^-6 display units) — ignored for the boost,
    /// where the layer's canonical price applies.
    pub gross_amount_micros: i64,
    /// Buyer ID.
    pub buyer_user_id: String,
    /// Creator ID.
    pub creator_user_id: String,
    /// Referrer ID — empty/absent means a direct unreferred sale (the 2000 bps rule).
    pub referrer_user_id: Option<String>,
    /// Affiliate watch — `1000..=5000`; charter default `1000`.
    pub affiliate_bps: i64,
    /// Boost tier for the `instant_boost` scenario — `featured-slot-24h` |
    /// `mairon-fast-track-audit` | `viral-swarm-amplifier`.
    pub boost_tier: Option<String>,
    /// EIP-681 recipient address (optional) — when present, the Base payment URI
    /// is built.
    pub recipient_address: Option<String>,
}

/// One ledger leg on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolySplitLegView {
    /// Leg kind (`buyer_debit` | `kia_treasury_credit` | …).
    pub leg_kind: String,
    /// Ledger account.
    pub account_id: String,
    /// Signed micro amount.
    pub amount_micros: i64,
    /// Declared bps rate.
    pub bps: i64,
    /// Charter note.
    pub note: String,
}

/// Split-simulation response — zero-sum proof + persistence + EIP-681.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolySimulateSplitResponse {
    /// Persisted row ID (`split:<hash>`).
    pub split_id: String,
    /// Executed scenario.
    pub scenario: String,
    /// Settlement asset.
    pub settlement_asset: String,
    /// Gross micro amount.
    pub gross_amount_micros: i64,
    /// Sovereign treasury micros.
    pub kia_treasury_micros: i64,
    /// Referrer micros (0 when unreferred).
    pub referrer_micros: i64,
    /// Creator net.
    pub creator_net_micros: i64,
    /// Effective platform share in bps (2000 on a direct sale).
    pub effective_platform_bps: i64,
    /// The two-sided ledger legs.
    pub legs: Vec<MonopolySplitLegView>,
    /// Zero-sum proof — auditable text.
    pub zero_sum_proof: String,
    /// Whether the row was persisted in `sovereign_treasury_splits`.
    pub persisted: bool,
    /// EIP-681 payment URI on Base — only for on-chain assets and when
    /// `recipient_address` is provided.
    pub eip681_payment_uri: Option<String>,
    /// Orchestrator status summary.
    pub orchestrator_status: String,
}
