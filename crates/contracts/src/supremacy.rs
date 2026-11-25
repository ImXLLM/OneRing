//! قراردادهای وایرِ موتورِ برتریِ رقابتی (Wave 8 — Commit #28).
//!
//! DTOهای روی وایر برای پنج اندپوینتِ `/api/v1/supremacy/*`:
//! - [`SupremacySurgicalPatchRequest`] / [`SupremacySurgicalPatchResponse`] —
//!   پچرِ جراحیِ `SEARCH/REPLACE` + بستهٔ بازرسِ DOM زنده (Bolt/Lovable)؛
//! - [`SupremacyCheckpointForkRequest`] / [`SupremacyCheckpointForkResponse`] —
//!   سفرِ زمانیِ وضعیتِ ایجنت و انشعاب از هر گامِ تاریخی (`LangGraph`)؛
//! - [`SupremacyWorkflowExecuteRequest`] / [`SupremacyWorkflowExecuteResponse`] —
//!   اجرایِ DAG پویا با مرتب‌سازیِ توپولوژیکِ Kahn (Dify/n8n)؛
//! - [`SupremacyMcpRequest`] / [`SupremacyMcpResponse`] — پلِ MCP JSON-RPC 2.0؛
//! - [`SupremacySweVerifyRequest`] / [`SupremacySweVerifyResponse`] —
//!   حلقهٔ `Plan → Approve → Execute → Verify → Self-Heal` با مُهرِ مایرون
//!   (OpenHands/Replit Agent).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── ستونِ ۱ — پچرِ جراحی + بازرسِ DOM زنده ─────────────────────────────────

/// درخواستِ `POST /api/v1/supremacy/surgical-patch`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySurgicalPatchRequest {
    /// منبعِ اصلی — پچ فقط روی همین متن اعمال می‌شود (صفر بازتولیدِ کلِ فایل).
    pub original_source: String,
    /// بارِ پچ — بلوک‌هایِ `<<<<<<< SEARCH … ======= … >>>>>>> REPLACE`.
    pub patch_payload: String,
    /// لوکالِ بستهٔ سندباکس زنده (`fa`, `en`, …) — پیش‌فرض `en`.
    #[serde(default)]
    pub locale: Option<String>,
    /// تزریقِ اسکریپتِ بازرسِ Click-to-Inspect — پیش‌فرض روشن.
    #[serde(default = "default_true")]
    pub inject_sandbox_inspector: bool,
}

fn default_true() -> bool {
    true
}

/// نمایِ بستهٔ سندباکس زنده.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySandboxBundleView {
    /// لوکالِ بسته.
    pub locale: String,
    /// HTML ابزارشده — هر تگِ باز `data-kia-node-id` دارد.
    pub instrumented_html: String,
    /// تعدادِ گره‌هایِ ابزارشده.
    pub node_count: usize,
    /// تایپِ پیامِ postMessage بازرس (`KIA_DOM_INSPECT`).
    pub message_type: String,
    /// بایت‌هایِ اسکریپتِ بازرسِ صفر-وابستگی.
    pub inspector_script_bytes: usize,
}

/// پاسخِ `POST /api/v1/supremacy/surgical-patch`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySurgicalPatchResponse {
    /// آیا پچ اعمال شد (بلوک‌ها همه یافته شدند).
    pub patch_applied: bool,
    /// تعدادِ بلوک‌هایِ اعمال‌شده.
    pub hunk_count: usize,
    /// حالتِ تطبیق — `exact` یا `whitespace_normalized`.
    pub patch_mode: String,
    /// SHA-256 منبعِ پیش از پچ.
    pub before_sha256: String,
    /// SHA-256 منبعِ پس از پچ.
    pub after_sha256: String,
    /// منبعِ پچ‌شده.
    pub patched_source: String,
    /// بستهٔ سندباکس زنده — وقتی `inject_sandbox_inspector` روشن باشد.
    pub sandbox_bundle: Option<SupremacySandboxBundleView>,
}

// ── ستونِ ۲ — سفرِ زمانیِ وضعیتِ ایجنت ──────────────────────────────────────

/// درخواستِ `POST /api/v1/supremacy/checkpoint/fork`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyCheckpointForkRequest {
    /// شناسهٔ گردش‌کار (`wf:*`).
    pub workflow_id: String,
    /// شناسهٔ چک‌پوینتِ تاریخیِ مبدأ انشعاب.
    pub fork_from_checkpoint_id: String,
    /// وصلهٔ وضعیت — merge رسمیِ کلیدهایِ سطحِ بالایِ JSON.
    pub state_patch_json: String,
    /// نقشِ ایجنتِ سرِ شاخه — پیش‌فرض نقشِ چک‌پوینتِ مبدأ.
    #[serde(default)]
    pub agent_role: Option<String>,
}

/// پاسخِ `POST /api/v1/supremacy/checkpoint/fork`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyCheckpointForkResponse {
    /// شناسهٔ چک‌پوینتِ انشعابیِ نو.
    pub forked_checkpoint_id: String,
    /// شناسهٔ والد (چک‌پوینتِ تاریخیِ مبدأ).
    pub parent_checkpoint_id: String,
    /// شناسهٔ گردش‌کار.
    pub workflow_id: String,
    /// شمارهٔ گامِ سرِ شاخه.
    pub step_index: u32,
    /// تعدادِ گام‌هایی که بدونِ اجرایِ مجدد جلو زده شد.
    pub rewound_steps: u32,
    /// وضعیتِ JSON پس از merge.
    pub state_json: String,
    /// SHA-256 وضعیتِ نو.
    pub state_sha256: String,
    /// اندازهٔ دفترِ چک‌پوینت پس از انشعاب.
    pub ledger_size: usize,
    /// اثباتِ سفرِ زمانی — والدِ انشعاب همان چک‌پوینتِ درخواستی است.
    pub time_travel_verified: bool,
    /// نقشِ ایجنتِ سرِ شاخه.
    pub agent_role: String,
}

// ── ستونِ ۳ — DAG پویا + MCP ────────────────────────────────────────────────

/// گرهٔ DAG روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowNodeDto {
    /// شناسهٔ یکتای گره.
    pub id: String,
    /// نوعِ گره — `trigger_cron_webhook` | `omni_genome_llm` | `trinity_judge` |
    /// `quant_vibe_strategy` | `arda_localize20` | `base_treasury_settle` |
    /// `mcp_external_tool`.
    pub kind: String,
    /// شناسهٔ وابستگی‌ها — باید پیش از این گره اجرا شده باشند.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// بارِ JSON گره.
    #[serde(default)]
    pub payload_json: String,
}

/// مشخصِ DAG روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowSpecDto {
    /// شناسهٔ گردش‌کار.
    pub workflow_id: String,
    /// گره‌ها.
    pub nodes: Vec<SupremacyWorkflowNodeDto>,
}

/// درخواستِ `POST /api/v1/supremacy/workflow/execute`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowExecuteRequest {
    /// مشخصِ DAG.
    pub workflow: SupremacyWorkflowSpecDto,
}

/// نتیجهٔ اجرای یک گره.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowNodeResultView {
    /// شناسهٔ گره.
    pub node_id: String,
    /// نوعِ گره (شناسهٔ وایر).
    pub kind: String,
    /// وضعیت — `executed`.
    pub status: String,
    /// SHA-256 خروجیِ قطعیِ گره.
    pub output_sha256: String,
    /// یادداشتِ اجرا.
    pub note: String,
}

/// پاسخِ `POST /api/v1/supremacy/workflow/execute`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyWorkflowExecuteResponse {
    /// شناسهٔ رانِ ماندگار (`dagrun:<uuid>`).
    pub run_id: String,
    /// شناسهٔ DAG.
    pub dag_id: String,
    /// ترتیبِ توپولوژیکِ اجرا.
    pub topo_order: Vec<String>,
    /// وضعیتِ ران — `completed`.
    pub status: String,
    /// تعدادِ گره‌هایِ اجراشده.
    pub executed_nodes: usize,
    /// امتیاز REA ران (مایرون) — `92` برای رانِ کامل.
    pub rea_score: u8,
    /// نتایجِ گره‌ها به ترتیبِ توپولوژیک.
    pub node_results: Vec<SupremacyWorkflowNodeResultView>,
    /// آیا در `WorkflowDagRun` ماندگار شد.
    pub persisted: bool,
}

/// درخواستِ `POST /api/v1/supremacy/mcp` — JSON-RPC 2.0 استاندارد.
#[derive(Debug, Clone, PartialEq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyMcpRequest {
    /// نسخهٔ پروتکل — باید `"2.0"` باشد.
    pub jsonrpc: String,
    /// شناسهٔ درخواست — عدد، رشته یا `null`.
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    /// متد — `initialize` | `tools/list` | `tools/call`.
    pub method: String,
    /// پارامترها.
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// پاسخِ JSON-RPC 2.0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyMcpResponse {
    /// نسخهٔ پروتکل — همیشه `"2.0"`.
    pub jsonrpc: String,
    /// شناسهٔ درخواست (echo).
    pub id: Option<serde_json::Value>,
    /// نتیجهٔ موفق.
    pub result: Option<serde_json::Value>,
    /// خطای JSON-RPC — `{code, message}`.
    pub error: Option<SupremacyMcpErrorView>,
}

/// خطای JSON-RPC 2.0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyMcpErrorView {
    /// کدِ خطا (`-32601` متد ناشناخته، `-32602` پارامتر نامعتبر).
    pub code: i32,
    /// پیامِ خطا.
    pub message: String,
}

// ── ستونِ ۴ — حلقهٔ SWE + مُهرِ مایرون ──────────────────────────────────────

/// درخواستِ `POST /api/v1/supremacy/swe-verify`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySweVerifyRequest {
    /// خلاصهٔ پلن (گامِ `Plan`).
    #[serde(default)]
    pub plan_summary: String,
    /// زبانِ آرتیفکت — `rust` | `typescript` | `solidity` | `html`.
    pub language: String,
    /// کدِ تولیدشده برای راستی‌آزمایی.
    pub artifact: String,
    /// سقفِ تلاش‌هایِ خودترمیمی — پیش‌فرض ۳.
    #[serde(default)]
    pub max_repair_attempts: Option<u32>,
}

/// نتیجهٔ یک گیتِ راستی‌آزمایی.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySweCheckView {
    /// نامِ گیت (`balanced_delimiters`, …).
    pub name: String,
    /// عبور؟
    pub passed: bool,
    /// جزئیاتِ قابلِ ممیزی.
    pub detail: String,
}

/// مُهرِ تأییدِ مایرون — فقط برای `rea_score >= 85`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacyMaironSealView {
    /// شناسهٔ مُهر (`mairon-seal:<sha16>`).
    pub seal_id: String,
    /// امتیاز REA.
    pub rea_score: u8,
    /// SHA-256 آرتیفکتِ مهرشده.
    pub artifact_sha256: String,
}

/// پاسخِ `POST /api/v1/supremacy/swe-verify`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupremacySweVerifyResponse {
    /// آیا همهٔ گیت‌ها پاس شدند.
    pub verified: bool,
    /// تعدادِ تلاش‌هایِ مصرف‌شده (۱ = بدونِ ترمیم).
    pub attempts_used: u32,
    /// امتیاز REA نهایی.
    pub rea_score: u8,
    /// گیت‌ها.
    pub checks: Vec<SupremacySweCheckView>,
    /// تشخیص‌هایِ ساختاری — ورودیِ پرامپتِ خودترمیمی.
    pub diagnostics: Vec<String>,
    /// تعدادِ پرامپت‌هایِ خودترمیمیِ صادرشده.
    pub repair_prompts_issued: u32,
    /// SHA-256 آرتیفکتِ نهایی.
    pub artifact_sha256: String,
    /// مُهرِ مایرون — وقتی `rea_score >= 85`.
    pub seal: Option<SupremacyMaironSealView>,
    /// آیا در `SweSandboxAudit` ماندگار شد.
    pub persisted: bool,
}
