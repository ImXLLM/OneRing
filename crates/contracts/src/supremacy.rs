//! Wire contracts of the competitive supremacy engine (Wave 8 — Commit #28).
//!
//! On-wire DTOs for the five `/api/v1/supremacy/*` endpoints:
//! - [`SupremacySurgicalPatchRequest`] / [`SupremacySurgicalPatchResponse`] —
//!   the `SEARCH/REPLACE` surgical patcher + the live DOM inspector bundle (Bolt/Lovable);
//! - [`SupremacyCheckpointForkRequest`] / [`SupremacyCheckpointForkResponse`] —
//!   agent-state time travel with forking from any historical step (`LangGraph`);
//! - [`SupremacyWorkflowExecuteRequest`] / [`SupremacyWorkflowExecuteResponse`] —
//!   dynamic DAG execution with Kahn's topological sort (Dify/n8n);
//! - [`SupremacyMcpRequest`] / [`SupremacyMcpResponse`] — the MCP JSON-RPC 2.0 bridge;
//! - [`SupremacySweVerifyRequest`] / [`SupremacySweVerifyResponse`] —
//!   the `Plan → Approve → Execute → Verify → Self-Heal` loop with the Mairon seal
//!   (OpenHands/Replit Agent).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── Pillar 1 — surgical patcher + live DOM inspector ─────────────────────────────────

/// Request of `POST /api/v1/supremacy/surgical-patch`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySurgicalPatchRequest {
    /// The original source — the patch applies only to this text (zero regeneration of the whole file).
    pub original_source: String,
    /// The patch payload — `<<<<<<< SEARCH … ======= … >>>>>>> REPLACE` blocks.
    pub patch_payload: String,
    /// Locale of the live sandbox bundle (`fa`, `en`, …) — default `en`.
    #[serde(default)]
    pub locale: Option<String>,
    /// Injection of the Click-to-Inspect inspector script — on by default.
    #[serde(default = "default_true")]
    pub inject_sandbox_inspector: bool,
}

fn default_true() -> bool {
    true
}

/// View of the live sandbox bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySandboxBundleView {
    /// Bundle locale.
    pub locale: String,
    /// Instrumented HTML — every open tag carries `data-kia-node-id`.
    pub instrumented_html: String,
    /// Number of instrumented nodes.
    pub node_count: usize,
    /// postMessage type of the inspector (`KIA_DOM_INSPECT`).
    pub message_type: String,
    /// Bytes of the zero-dependency inspector script.
    pub inspector_script_bytes: usize,
}

/// Response of `POST /api/v1/supremacy/surgical-patch`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySurgicalPatchResponse {
    /// Whether the patch applied (all blocks found).
    pub patch_applied: bool,
    /// Number of applied blocks.
    pub hunk_count: usize,
    /// Match mode — `exact` or `whitespace_normalized`.
    pub patch_mode: String,
    /// SHA-256 of the pre-patch source.
    pub before_sha256: String,
    /// SHA-256 of the post-patch source.
    pub after_sha256: String,
    /// The patched source.
    pub patched_source: String,
    /// The live sandbox bundle — when `inject_sandbox_inspector` is on.
    pub sandbox_bundle: Option<SupremacySandboxBundleView>,
}

// ── Pillar 2 — agent-state time travel ──────────────────────────────────────

/// Request of `POST /api/v1/supremacy/checkpoint/fork`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyCheckpointForkRequest {
    /// Workflow ID (`wf:*`).
    pub workflow_id: String,
    /// ID of the historical checkpoint to fork from.
    pub fork_from_checkpoint_id: String,
    /// State patch — a formal merge of the top-level JSON keys.
    pub state_patch_json: String,
    /// Agent role of the branch head — defaults to the origin checkpoint's role.
    #[serde(default)]
    pub agent_role: Option<String>,
}

/// Response of `POST /api/v1/supremacy/checkpoint/fork`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyCheckpointForkResponse {
    /// ID of the new forked checkpoint.
    pub forked_checkpoint_id: String,
    /// Parent ID (the origin historical checkpoint).
    pub parent_checkpoint_id: String,
    /// Workflow ID.
    pub workflow_id: String,
    /// Step number of the branch head.
    pub step_index: u32,
    /// Number of steps skipped ahead without re-execution.
    pub rewound_steps: u32,
    /// The JSON state after the merge.
    pub state_json: String,
    /// SHA-256 of the new state.
    pub state_sha256: String,
    /// Checkpoint-ledger size after the fork.
    pub ledger_size: usize,
    /// Time-travel proof — the fork's parent is exactly the requested checkpoint.
    pub time_travel_verified: bool,
    /// Agent role of the branch head.
    pub agent_role: String,
}

// ── Pillar 3 — dynamic DAG + MCP ────────────────────────────────────────────────

/// A DAG node on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowNodeDto {
    /// Unique node ID.
    pub id: String,
    /// Node type — `trigger_cron_webhook` | `omni_genome_llm` | `trinity_judge` |
    /// `quant_vibe_strategy` | `arda_localize20` | `base_treasury_settle` |
    /// `mcp_external_tool`.
    pub kind: String,
    /// Dependency IDs — they must run before this node.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// The node's JSON payload.
    #[serde(default)]
    pub payload_json: String,
}

/// A DAG spec on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowSpecDto {
    /// Workflow ID.
    pub workflow_id: String,
    /// Nodes.
    pub nodes: Vec<SupremacyWorkflowNodeDto>,
}

/// Request of `POST /api/v1/supremacy/workflow/execute`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowExecuteRequest {
    /// The DAG spec.
    pub workflow: SupremacyWorkflowSpecDto,
}

/// Result of one node's execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowNodeResultView {
    /// Node ID.
    pub node_id: String,
    /// Node type (wire ID).
    pub kind: String,
    /// Status — `executed`.
    pub status: String,
    /// SHA-256 of the node's deterministic output.
    pub output_sha256: String,
    /// Execution note.
    pub note: String,
}

/// Response of `POST /api/v1/supremacy/workflow/execute`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowExecuteResponse {
    /// Persisted run ID (`dagrun:<uuid>`).
    pub run_id: String,
    /// DAG ID.
    pub dag_id: String,
    /// Topological execution order.
    pub topo_order: Vec<String>,
    /// Run status — `completed`.
    pub status: String,
    /// Number of executed nodes.
    pub executed_nodes: usize,
    /// The run's REA score (Mairon) — `92` for a complete run.
    pub rea_score: u8,
    /// Node results in topological order.
    pub node_results: Vec<SupremacyWorkflowNodeResultView>,
    /// Whether it was persisted in `WorkflowDagRun`.
    pub persisted: bool,
}

/// Request of `POST /api/v1/supremacy/mcp` — standard JSON-RPC 2.0.
#[derive(Debug, Clone, PartialEq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyMcpRequest {
    /// Protocol version — must be `"2.0"`.
    pub jsonrpc: String,
    /// Request ID — number, string, or `null`.
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    /// Method — `initialize` | `tools/list` | `tools/call`.
    pub method: String,
    /// Parameters.
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// The JSON-RPC 2.0 response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyMcpResponse {
    /// Protocol version — always `"2.0"`.
    pub jsonrpc: String,
    /// Request ID (echo).
    pub id: Option<serde_json::Value>,
    /// The success result.
    pub result: Option<serde_json::Value>,
    /// The JSON-RPC error — `{code, message}`.
    pub error: Option<SupremacyMcpErrorView>,
}

/// A JSON-RPC 2.0 error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyMcpErrorView {
    /// Error code (`-32601` unknown method, `-32602` invalid params).
    pub code: i32,
    /// Error message.
    pub message: String,
}

// ── Pillar 4 — the SWE loop + the Mairon seal ──────────────────────────────────────

/// Request of `POST /api/v1/supremacy/swe-verify`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySweVerifyRequest {
    /// Plan summary (the `Plan` step).
    #[serde(default)]
    pub plan_summary: String,
    /// Artifact language — `rust` | `typescript` | `solidity` | `html`.
    pub language: String,
    /// The generated code to verify.
    pub artifact: String,
    /// Self-repair attempt cap — default 3.
    #[serde(default)]
    pub max_repair_attempts: Option<u32>,
}

/// Result of one verification gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySweCheckView {
    /// Gate name (`balanced_delimiters`, …).
    pub name: String,
    /// Passed?
    pub passed: bool,
    /// Auditable details.
    pub detail: String,
}

/// Mairon approval seal — only for `rea_score >= 85`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyMaironSealView {
    /// Seal ID (`mairon-seal:<sha16>`).
    pub seal_id: String,
    /// REA score.
    pub rea_score: u8,
    /// SHA-256 of the sealed artifact.
    pub artifact_sha256: String,
}

/// Response of `POST /api/v1/supremacy/swe-verify`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySweVerifyResponse {
    /// Whether all gates passed.
    pub verified: bool,
    /// Attempts consumed (1 = no repair).
    pub attempts_used: u32,
    /// Final REA score.
    pub rea_score: u8,
    /// Gates.
    pub checks: Vec<SupremacySweCheckView>,
    /// Structural findings — input to the self-repair prompt.
    pub diagnostics: Vec<String>,
    /// Number of self-repair prompts issued.
    pub repair_prompts_issued: u32,
    /// SHA-256 of the final artifact.
    pub artifact_sha256: String,
    /// The Mairon seal — when `rea_score >= 85`.
    pub seal: Option<SupremacyMaironSealView>,
    /// Whether it was persisted in `SweSandboxAudit`.
    pub persisted: bool,
}
