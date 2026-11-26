//! M4 "brain + operations panel" wire contracts — AI gateway, Hermes engine,
//! A2A/MCP fleet, and admin endpoints.
//!
//! Phase 3 rules stand: `camelCase`, explicit `null` for nullable, missing key
//! for optional, integers up to `2^53−1`. This module's most sensitive security rule:
//! **AI output never returns a key or secret** — `redactions` lists what was redacted,
//! not the secret itself; and the A2A card never exposes a session or key.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::common::{Din, Id};
use crate::market::ProductView;

// ── AI gateway ───────────────────────────────────────────────────────────────

/// One message in an AI conversation — standard `system|user|assistant` roles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiMessage {
    /// Message role
    pub role: String,
    /// Message text — redacted before being sent to any slot
    pub content: String,
}

/// Chat request to the AI gateway (three-slot fallback).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiChatRequest {
    /// Conversation messages — at least 1
    pub messages: Vec<AiMessage>,
    /// Optional Hermes skill — absent means automatic routing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<Id>,
}

/// Gateway response — winning slot + exact usage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiChatResponse {
    /// Reply text (shield output also passes through it)
    pub content: String,
    /// Winning provider — "primary/secondary/tertiary"
    pub provider: String,
    /// Model used
    pub model: String,
    /// Prompt tokens
    pub prompt_tokens: i64,
    /// Completion tokens
    pub completion_tokens: i64,
    /// Total tokens
    pub total_tokens: i64,
    /// Redaction finding categories (e.g. `private_key_hex`) — the secret itself never appears
    pub redactions: Vec<String>,
    /// User's remaining daily budget in tokens
    pub remaining_tokens: i64,
}

/// User's gateway usage today — basis of the hard budget cap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiUsageResponse {
    /// Total tokens today (UTC)
    pub total_tokens: i64,
    /// Today's call count
    pub calls: i64,
    /// Configured daily cap
    pub daily_cap: i64,
}

/// One lane group in the gateway pool — honest: a group without env config reports `enabled: false`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiSlotView {
    /// Group number in pool order (1-based)
    pub slot: u8,
    /// Group name (`primary`/`secondary`/`tertiary` or a Hermes-style provider)
    pub name: String,
    /// Group models — comma-separated list; empty string without env
    pub model: String,
    /// Enabled = at least one group member has url+key+model complete
    pub enabled: bool,
}

/// AI gateway status — the honest health chip and admin dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiStatusResponse {
    /// Gateway enabled? At least one pool member configured
    pub enabled: bool,
    /// Per-user daily token cap
    pub daily_cap: i64,
    /// Pool groups in fallback order
    pub slots: Vec<AiSlotView>,
    /// Flat pool size — total `(url, key, model)` combinations after the Cartesian product
    pub pool_size: usize,
}

// ── Hermes engine ──────────────────────────────────────────────────────────────

/// Defensive shield verdict on skill input/output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShieldView {
    /// `ALLOW` / `REDACTED` / `BLOCKED`
    pub verdict: String,
    /// Blocking or redaction reason — explicit `null` for ALLOW
    pub reason: Option<String>,
}

/// Public view of a Hermes skill — the prompt template never goes out on the public wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesSkillView {
    /// Skill identifier (`wallet.balance` and so on)
    pub id: String,
    /// Persian title
    pub title: String,
    /// Category (`wallet`/`market`/`brain`/…)
    pub category: String,
    /// Description
    pub description: String,
    /// `kernel` (deterministic execution on the kernel) or `llm` (template + gateway)
    pub kind: String,
    /// Token estimate for budgeting
    pub estimate_tokens: i64,
}

/// Hermes skill list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesSkillListResponse {
    /// Enabled skills — newest-first sort by identifier
    pub items: Vec<HermesSkillView>,
}

/// Skill run request — without `skillId`, keyword-based automatic routing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesRunRequest {
    /// Skill identifier — absent means automatic
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<Id>,
    /// User input
    pub input: String,
}

/// Skill run response — includes the shield verdict and the identity score after this run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesRunResponse {
    /// Run identifier (`hrun:{uuid}`)
    pub run_id: Id,
    /// Skill that ran — under automatic routing, the selected skill is reported
    pub skill_id: String,
    /// Skill output
    pub output: String,
    /// Shield verdict on the input
    pub shield: ShieldView,
    /// Input/output redaction finding categories
    pub redactions: Vec<String>,
    /// Provider — explicit `null` for kernel skills
    pub provider: Option<String>,
    /// Model — explicit `null` for kernel skills
    pub model: Option<String>,
    /// Tokens consumed (kernel skill → 0)
    pub tokens: i64,
    /// Identity score 0..=100 after this run (honest identity RL from run counting)
    pub identity_score: i64,
}

/// One of my past runs — full output only in the run itself; the list is a short preview.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesRunView {
    /// Run identifier
    pub id: Id,
    /// Skill
    pub skill_id: String,
    /// Shield verdict (`ALLOW`/`REDACTED`/`BLOCKED`)
    pub shield: String,
    /// Tokens consumed
    pub tokens: i64,
    /// Audit outcome (`hermes.ok|blocked|failed`)
    pub outcome: String,
    /// ISO time
    pub at: String,
}

/// My run list + current identity score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesRunListResponse {
    /// Runs — newest first
    pub items: Vec<HermesRunView>,
    /// Identity score 0..=100
    pub identity_score: i64,
}

// ── A2A/MCP fleet ──────────────────────────────────────────────────────────

/// One agent in the fleet — honest status (`active`/`registered`/`disabled`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentView {
    /// Agent identifier (`agent:{role}`)
    pub id: String,
    /// Name
    pub name: String,
    /// Role — one of the 7 fleet roles
    pub role: String,
    /// Status — no agent ever lies
    pub status: String,
    /// Role description
    pub description: String,
}

/// Fleet + MCP tools — a transparent public view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FleetResponse {
    /// The 7 fleet agents
    pub agents: Vec<AgentView>,
    /// A2A gateway enabled? (the admin holds the key)
    pub a2a_enabled: bool,
    /// Registered MCP tools
    pub mcp_tools: Vec<McpToolView>,
}

/// MCP tool — closed allowlist; an unknown tool is never invoked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpToolView {
    /// Tool name
    pub name: String,
    /// Description
    pub description: String,
    /// Required arguments in order
    pub args: Vec<String>,
    /// Admin only? (public reads are false)
    pub admin_only: bool,
}

/// A2A card provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentCardProvider {
    /// Organization
    pub organization: String,
    /// URL
    pub url: String,
}

/// One skill in the A2A card — comes from the Hermes skill list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentCardSkill {
    /// Identifier
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Tags
    pub tags: Vec<String>,
}

/// A2A discovery card — `/.well-known/agent-card.json`; contains no keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentCardResponse {
    /// Agent name
    pub name: String,
    /// Description
    pub description: String,
    /// Service URL
    pub url: String,
    /// Kian version
    pub version: String,
    /// Protocol version
    pub protocol_version: String,
    /// Organization
    pub provider: AgentCardProvider,
    /// Skills (from Hermes)
    pub skills: Vec<AgentCardSkill>,
}

/// Send an A2A message between fleet agents — admin only (A2A gateway).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct A2aSendRequest {
    /// Sender — must be a registered agent
    pub from: Id,
    /// Recipient — must be a registered agent
    pub to: Id,
    /// Message kind (`handoff`/`query`/`result`)
    pub kind: String,
    /// Body
    pub body: String,
}

/// One A2A message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct A2aMessageView {
    /// Message identifier (`a2a:{uuid}`)
    pub id: Id,
    /// Sender
    pub from_agent: String,
    /// Recipient
    pub to_agent: String,
    /// Kind
    pub kind: String,
    /// Body
    pub body: String,
    /// ISO time
    pub at: String,
}

/// One agent's inbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct A2aInboxResponse {
    /// Messages — newest first
    pub items: Vec<A2aMessageView>,
}

/// MCP tool invocation — allowlist tools only, with exact arguments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpInvokeRequest {
    /// Tool name
    pub tool: Id,
    /// Arguments — JSON object
    pub arguments: Value,
}

/// MCP tool result — whatever shape the tool returns; errors come via the standard envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpInvokeResponse {
    /// Tool name
    pub tool: String,
    /// Result
    pub result: Value,
}

// ── Admin panel ───────────────────────────────────────────────────────────────

/// One point of the 14-day revenue series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevenuePoint {
    /// Day (YYYY-MM-DD)
    pub day: String,
    /// Revenue din for that day
    pub din: Din,
}

/// Sales per seller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SellerSalesView {
    /// Seller account
    pub seller_id: String,
    /// Total sales in din
    pub din: Din,
}

/// Sales per product.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductSalesView {
    /// Product identifier
    pub product_id: String,
    /// Title — explicit `null` if the product was deleted
    pub title: Option<String>,
    /// Total sales in din
    pub din: Din,
}

/// Admin overview — every counter comes from live database queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminOverviewResponse {
    /// Telegram users
    pub users: i64,
    /// Ledger accounts
    pub accounts: i64,
    /// Escrows
    pub escrows: i64,
    /// Pending HITL requests
    pub hitl_pending: i64,
    /// Total minted (Article 3)
    pub minted_all_time: Din,
    /// Total burned
    pub burned_all_time: Din,
    /// Total AI calls
    pub ai_calls: i64,
    /// 14-day revenue series
    pub revenue_series: Vec<RevenuePoint>,
    /// Sales per seller
    pub sales_by_seller: Vec<SellerSalesView>,
    /// Sales per product
    pub sales_by_product: Vec<ProductSalesView>,
}

/// One HITL request for the admin queue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminHitlView {
    /// Request identifier
    pub id: Id,
    /// Kind (`ESCROW_RELEASE`)
    pub kind: String,
    /// Related escrow
    pub escrow_id: String,
    /// Amount
    pub amount: Din,
    /// Requester
    pub requester_id: String,
    /// Request note — absent means none was recorded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Status (`PENDING`/`APPROVED`/`REJECTED`/`EXPIRED`)
    pub status: String,
    /// ISO creation time
    pub created_at: String,
    /// ISO expiry
    pub expires_at: String,
    /// Decided by — absent until decided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decided_by: Option<String>,
    /// Decision time — absent until decided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decided_at: Option<String>,
    /// Decision note — absent means none was recorded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_note: Option<String>,
}

/// Admin HITL queue list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminHitlListResponse {
    /// Requests — newest first
    pub items: Vec<AdminHitlView>,
}

/// Admin HITL decision — approve means atomic settlement in the same command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HitlDecisionRequest {
    /// Optional decision note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// The result of the HITL decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HitlDecisionResponse {
    /// Request identifier
    pub id: Id,
    /// Status after the decision (`APPROVED`/`REJECTED`)
    pub status: String,
    /// ISO decision time
    pub decided_at: String,
    /// Was money settled? (approve only)
    pub settled: bool,
    /// Settlement transaction identifier — replay always yields the original tx
    pub tx_id: Option<String>,
    /// Was the settlement a replay?
    pub replayed: bool,
}

/// One audit row for the admin trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminAuditEntry {
    /// Identifier
    pub id: Id,
    /// Actor
    pub actor: String,
    /// Action
    pub action: String,
    /// Target — absent means no target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// String JSON metadata — absent means no metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<String>,
    /// ISO time
    pub at: String,
}

/// Admin audit trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminAuditListResponse {
    /// Rows — newest first
    pub items: Vec<AdminAuditEntry>,
}

/// One TON deposit in the admin view — with its user (unlike the M3 user view).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminDepositView {
    /// Deposit identifier
    pub id: Id,
    /// Registering user
    pub user_id: String,
    /// Chain transaction hash (UNIQUE — the replay lesson)
    pub tx_hash: String,
    /// Sender address
    pub wallet_address: String,
    /// Decimal-string TON amount
    pub amount_ton: String,
    /// Status (`PENDING`/`CONFIRMED`/`REJECTED`)
    pub status: String,
    /// ISO recorded time
    pub created_at: String,
    /// ISO decision time — absent until adjudicated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decided_at: Option<String>,
    /// Decided by — absent until resolved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decided_by: Option<String>,
}

/// TON deposit list for admins (all users, filterable by status).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminDepositListResponse {
    /// Deposits — newest first
    pub items: Vec<AdminDepositView>,
}

/// Product list for admins — includes inactive ones (unlike the public market view).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminProductListResponse {
    /// All products — newest first
    pub items: Vec<ProductView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    /// Wire contract: the AI response round-trips with explicit null and camelCase.
    #[test]
    fn ai_chat_response_round_trips_camel_case() {
        let res = AiChatResponse {
            content: "\u{633}\u{644}\u{627}\u{645}".into(),
            provider: "primary".into(),
            model: "kian-model".into(),
            prompt_tokens: 12,
            completion_tokens: 34,
            total_tokens: 46,
            redactions: vec!["private_key_hex".into()],
            remaining_tokens: 49_954,
        };
        let wire = to_string(&res).unwrap();
        assert!(wire.contains("\"promptTokens\""), "camelCase wire: {wire}");
        assert!(wire.contains("\"remainingTokens\""), "camelCase wire: {wire}");
        let back: AiChatResponse = from_str(&wire).unwrap();
        assert_eq!(back, res);
    }

    /// Defensive shield: the verdict is `SCREAMING_SNAKE_CASE` on the wire — like escrow status.
    #[test]
    fn shield_verdict_is_screaming_snake() {
        let blocked = ShieldView { verdict: "BLOCKED".into(), reason: Some("injection".into()) };
        let wire = to_string(&blocked).unwrap();
        assert!(wire.contains("\"verdict\":\"BLOCKED\""), "wire: {wire}");
        let allow: ShieldView =
            from_str(&to_string(&ShieldView { verdict: "ALLOW".into(), reason: None }).unwrap())
                .unwrap();
        assert_eq!(allow.reason, None);
    }

    /// A2A card — public discovery; the protocol version and skills must always be present.
    #[test]
    fn agent_card_shape_is_a2a_like() {
        let card = AgentCardResponse {
            name: "KIAN Kernel".into(),
            description: "Money kernel + Hermes brain".into(),
            url: "http://127.0.0.1:8088/api/v1".into(),
            version: "0.1.0".into(),
            protocol_version: "a2a/1.0-kian".into(),
            provider: AgentCardProvider { organization: "KiaAgentX".into(), url: "https://kiaagentx.io".into() },
            skills: vec![AgentCardSkill {
                id: "wallet.balance".into(),
                name: "\u{62a}\u{631}\u{627}\u{632} \u{645}\u{646}".into(),
                description: "\u{62e}\u{648}\u{627}\u{646}\u{634} \u{642}\u{637}\u{639}\u{6cc} \u{62f}\u{641}\u{62a}\u{631}".into(),
                tags: vec!["kernel".into()],
            }],
        };
        let wire = to_string(&card).unwrap();
        assert!(wire.contains("\"protocolVersion\""), "wire: {wire}");
        assert!(wire.contains("\"skills\""), "wire: {wire}");
        assert!(!wire.to_lowercase().contains("key"), "no credential marker in card: {wire}");
    }

    /// MCP invocation — arguments is a free (object) value but tool is typed as Id.
    #[test]
    fn mcp_invoke_accepts_json_arguments() {
        let req: McpInvokeRequest = from_str(
            r#"{"tool":"kernel.economy","arguments":{"deep":true}}"#,
        )
        .unwrap();
        assert_eq!(req.tool.get(), "kernel.economy");
        assert_eq!(req.arguments["deep"], serde_json::json!(true));
    }
}
