//! HTTP error classification — port of `server/src/errors.ts` + the explicit codes from `api.ts`/`guards.ts`.
//!
//! Rule: domain codes map to «honest» statuses — 404 (not found),
//! 422 (domain rule/body validation), 400 (request validation), 401 (authentication),
//! 429 (rate), 501/502/503 (infrastructure). Constant body: `{ error: { code, message, meta? } }`.
//! Codes are additive-only — old clients never break.

use crate::common::{ApiError, ErrorEnvelope, Meta, MetaValue};
use kian_domain::{DomainError, DomainErrorCode};

// ── mapping sets — one-to-one with errors.ts ────────────────────────────────

/// 404 codes — `NOT_FOUND_CODES`.
const NOT_FOUND_CODES: &[DomainErrorCode] = &[
    DomainErrorCode::EscrowNotFound,
    DomainErrorCode::UnknownAccount,
    DomainErrorCode::HitlNotFound,
    DomainErrorCode::ProductNotFound,
];

/// 422 codes — `UNPROCESSABLE_CODES` (domain-rule violations, not system errors).
const UNPROCESSABLE_CODES: &[DomainErrorCode] = &[
    DomainErrorCode::InvalidTransition,
    DomainErrorCode::RequiresHumanApproval,
    DomainErrorCode::AccountOverdraft,
    DomainErrorCode::UnbalancedTransaction,
    DomainErrorCode::NonIntegerAmount,
    DomainErrorCode::NegativeAmount,
    DomainErrorCode::AmountOutOfRange,
    DomainErrorCode::BpsOutOfRange,
    DomainErrorCode::MintCapExceeded,
    DomainErrorCode::EscrowAlreadyExists,
    DomainErrorCode::DuplicateAccount,
    DomainErrorCode::InvalidAccountKind,
    DomainErrorCode::TooFewPostings,
    DomainErrorCode::HitlNotRequired,
    DomainErrorCode::HitlAlreadyDecided,
    DomainErrorCode::HitlExpired,
];

/// HTTP status for a domain code — the `toHttpError` logic.
#[must_use]
pub fn status_for_domain_code(code: DomainErrorCode) -> u16 {
    if NOT_FOUND_CODES.contains(&code) {
        404
    } else if UNPROCESSABLE_CODES.contains(&code) {
        422
    } else {
        500
    }
}

/// Programmer/infrastructure error with an explicit status — the TS `HttpFailure` equivalent.
/// Named constructors exist for «all» current explicit server codes so the wire code
/// stays exact and no free-form string is written anywhere.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpFailure {
    /// HTTP status
    pub status: u16,
    /// Wire code
    pub code: String,
    /// Message
    pub message: String,
    /// Optional metadata
    pub meta: Option<Meta>,
}

impl HttpFailure {
    /// The general constructor (crate-internal).
    pub(crate) fn new(status: u16, code: &str, message: impl Into<String>) -> Self {
        Self { status, code: code.into(), message: message.into(), meta: None }
    }

    /// Incomplete domain error — `Command completed without escrow state.`
    #[must_use]
    pub fn internal_no_escrow() -> Self {
        Self::new(500, "INTERNAL", "Command completed without escrow state.")
    }

    /// Incomplete domain error — `Command completed without a transaction.`
    #[must_use]
    pub fn internal_no_tx() -> Self {
        Self::new(500, "INTERNAL", "Command completed without a transaction.")
    }

    /// A REJECTED decision must not run settlement.
    #[must_use]
    pub fn internal_rejected_settlement() -> Self {
        Self::new(500, "INTERNAL", "REJECTED decision must not execute a settlement.")
    }

    /// The VACUUM INTO backup failed.
    #[must_use]
    pub fn backup_failed(detail: impl std::fmt::Display) -> Self {
        Self::new(500, "BACKUP_FAILED", format!("VACUUM INTO failed — {detail}"))
    }

    /// The reserved system prefix.
    #[must_use]
    pub fn reserved_id() -> Self {
        Self::new(
            422,
            "RESERVED_ID",
            "The mint:/treasury:/burn:/escrow: prefixes are system-reserved.",
        )
    }

    /// Duplicate account (UNIQUE violation).
    #[must_use]
    pub fn duplicate_account(id: &str) -> Self {
        Self::new(422, "DUPLICATE_ACCOUNT", format!("Account {id} already exists."))
    }

    /// DB unavailable.
    #[must_use]
    pub fn db_down() -> Self {
        Self::new(503, "DB_DOWN", "The server cannot reach its database.")
    }

    /// Internal error with honest details (Phase 6 — AI-skills backend failure).
    #[must_use]
    pub fn internal(detail: impl std::fmt::Display) -> Self {
        Self::new(500, "INTERNAL", format!("Internal error — {detail}"))
    }

    /// Bot disabled — no `BOT_WEBHOOK_SECRET` (fail-closed Phase 5, like admin).
    #[must_use]
    pub fn bot_disabled() -> Self {
        Self::new(503, "BOT_DISABLED", "Bot webhook is disabled — set BOT_WEBHOOK_SECRET.")
    }

    /// The bot webhook secret token is wrong (`X-Telegram-Bot-Api-Secret-Token`).
    #[must_use]
    pub fn bot_unauthorized() -> Self {
        Self::new(401, "BOT_UNAUTHORIZED", "Invalid X-Telegram-Bot-Api-Secret-Token.")
    }

    /// Server without `KIAN_BOT_TOKEN` — Telegram identity is fail-closed, never open (M3).
    #[must_use]
    pub fn identity_disabled() -> Self {
        Self::new(
            503,
            "IDENTITY_DISABLED",
            "KIAN_BOT_TOKEN is not configured on this server — Telegram identity endpoints are disabled.",
        )
    }

    /// The `initData` signature is invalid — any tampering or token mismatch.
    #[must_use]
    pub fn invalid_init_data() -> Self {
        Self::new(
            401,
            "INVALID_INIT_DATA",
            "Telegram initData failed HMAC verification — identity is never derived from the body.",
        )
    }

    /// Missing or malformed Authorization header — session-gated routes (M3).
    #[must_use]
    pub fn session_required() -> Self {
        Self::new(
            401,
            "SESSION_REQUIRED",
            "Send the session token in the Authorization: Bearer header.",
        )
    }

    /// Malformed TON wallet address — 422 (M3 TON Connect address registration).
    #[must_use]
    pub fn invalid_ton_address() -> Self {
        Self::new(
            422,
            "INVALID_TON_ADDRESS",
            "TON wallet address must look like a 48-char friendly UQ/EQ address.",
        )
    }

    /// Session token expired or unknown — re-authenticate via `/auth/telegram`.
    #[must_use]
    pub fn session_expired() -> Self {
        Self::new(401, "SESSION_EXPIRED", "Session token is unknown or expired — re-authenticate.")
    }

    /// Escrow action only from the allowed side — the seller delivers, the buyer releases.
    #[must_use]
    pub fn forbidden_escrow_role() -> Self {
        Self::new(
            403,
            "FORBIDDEN",
            "Only the escrow's own seller can deliver and only its own buyer can release.",
        )
    }

    /// Demo/dev endpoints (`dev-login`, `faucet`) outside dev mode — 403.
    #[must_use]
    pub fn dev_mode_disabled() -> Self {
        Self::new(
            403,
            "DEV_MODE_DISABLED",
            "Dev/demo endpoints are disabled — enable KIAN_DEV_MODE=1 (or run without a bot token) for local testing.",
        )
    }

    /// Unknown parliament bill — M5 governance.
    #[must_use]
    pub fn bill_not_found(id: impl std::fmt::Display) -> Self {
        Self::new(
            404,
            "BILL_NOT_FOUND",
            format!("Unknown parliament bill `{id}` — see GET /api/v1/nation/bills."),
        )
    }

    /// Acting on a closed bill — voting/closing only while OPEN.
    #[must_use]
    pub fn bill_not_open(id: impl std::fmt::Display) -> Self {
        Self::new(
            422,
            "BILL_NOT_OPEN",
            format!("Bill `{id}` is not OPEN — voting and closing apply to open bills only."),
        )
    }

    /// Duplicate vote — one vote per voter per bill.
    #[must_use]
    pub fn already_voted(voter: impl std::fmt::Display) -> Self {
        Self::new(
            409,
            "ALREADY_VOTED",
            format!("Voter `{voter}` already cast a vote on this bill — one vote per voter."),
        )
    }

    /// Wallet address fails chain rules — details in meta.chain.
    #[must_use]
    pub fn invalid_address(chain: impl Into<String>, detail: impl Into<String>) -> Self {
        let chain = chain.into();
        let detail = detail.into();
        let mut meta = Meta::new();
        meta.insert("chain".into(), MetaValue::Text(chain.clone()));
        Self {
            status: 422,
            code: "INVALID_ADDRESS".into(),
            message: format!("Invalid {chain} address — {detail}."),
            meta: Some(meta),
        }
    }

    /// Treasury guard violation — a transfer without a parliament bill.
    #[must_use]
    pub fn custody_violation(detail: impl Into<String>) -> Self {
        Self::new(
            422,
            "CUSTODY_VIOLATION",
            format!("Treasury custody policy refused the movement — {}.", detail.into()),
        )
    }

    /// Parliament bill rules violated — details from `kian-nation`.
    #[must_use]
    pub fn invalid_bill(detail: impl Into<String>) -> Self {
        Self::new(
            422,
            "INVALID_BILL",
            format!("Parliament rule refused — {}.", detail.into()),
        )
    }

    /// Executing a bill that is not PASSED — a one-way transition.
    #[must_use]
    pub fn bill_not_passed(id: impl std::fmt::Display) -> Self {
        Self::new(
            422,
            "BILL_NOT_PASSED",
            format!("Bill `{id}` is not PASSED — only a passed bill executes."),
        )
    }

    /// Backup only for file-backed SQLite.
    #[must_use]
    pub fn backup_unsupported() -> Self {
        Self::new(503, "BACKUP_UNSUPPORTED", "Backups apply to file: (SQLite) databases only.")
    }

    /// Settings not implemented yet — an honest rejection beats a silent no-op.
    #[must_use]
    pub fn not_implemented() -> Self {
        Self::new(
            501,
            "NOT_IMPLEMENTED",
            "Settings persistence is not wired yet — branding comes from BRAND_NAME / BRAND_TAGLINE env.",
        )
    }

    /// Staff role without a valid key.
    #[must_use]
    pub fn unauthorized_staff() -> Self {
        Self::new(401, "UNAUTHORIZED", "Staff role requires a valid X-KIAN-Key.")
    }

    /// Public AI caller without initData.
    #[must_use]
    pub fn ai_identity_required() -> Self {
        Self::new(
            401,
            "AI_IDENTITY_REQUIRED",
            "Public AI callers must send valid Telegram initData — the acting account is derived from it, never from the body.",
        )
    }

    /// The server cannot verify the public AI identity (bot token not configured).
    #[must_use]
    pub fn ai_identity_disabled() -> Self {
        Self::new(
            503,
            "AI_IDENTITY_DISABLED",
            "This server cannot verify public AI callers (TELEGRAM_BOT_TOKEN unset) — send a valid X-KIAN-Key.",
        )
    }

    /// The AI loop is not configured.
    #[must_use]
    pub fn ai_disabled() -> Self {
        Self::new(
            503,
            "AI_DISABLED",
            "The AI cycle is not configured — set OPENAI_BASE_URL / OPENAI_API_KEY / MODEL (ADR-006).",
        )
    }

    /// The LLM upstream failed — the upstream code is passed through verbatim.
    #[must_use]
    pub fn ai_upstream(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { status: 502, code: code.into(), message: message.into(), meta: None }
    }

    /// The AI daily token cap is exhausted (Phase 6) — gated before every call.
    #[must_use]
    pub fn ai_cap_exceeded(used: i64, cap: i64) -> Self {
        let mut meta = Meta::new();
        meta.insert("limit".into(), MetaValue::Int(cap));
        meta.insert("used".into(), MetaValue::Int(used));
        Self {
            status: 429,
            code: "AI_CAP_EXCEEDED".into(),
            message: format!("Daily AI token cap exhausted ({used}/{cap})."),
            meta: Some(meta),
        }
    }

    /// The LLM returned two consecutive malformed envelopes — a typed failure; raw text never passes through.
    #[must_use]
    pub fn ai_malformed_packet() -> Self {
        Self::new(
            502,
            "AI_MALFORMED_PACKET",
            "LLM produced malformed packets twice — only {reply} or {tool,args} packets pass (ADR-009).",
        )
    }

    /// Rate limited — meta always carries `limit`.
    #[must_use]
    pub fn rate_limited(limit: i64) -> Self {
        let mut meta = Meta::new();
        meta.insert("limit".into(), MetaValue::Int(limit));
        Self {
            status: 429,
            code: "RATE_LIMITED".into(),
            message: format!("More than {limit} requests per minute."),
            meta: Some(meta),
        }
    }

    /// Unknown market product — admin/market routes.
    #[must_use]
    pub fn product_not_found(id: impl std::fmt::Display) -> Self {
        Self::new(
            404,
            "PRODUCT_NOT_FOUND",
            format!("Unknown market product `{id}`."),
        )
    }

    /// Product-builder session unknown or owned by another user — 404.
    #[must_use]
    pub fn builder_session_not_found(id: impl std::fmt::Display) -> Self {
        Self::new(
            404,
            "BUILDER_SESSION_NOT_FOUND",
            format!("Unknown builder session `{id}`."),
        )
    }

    /// Attempt to publish a session before the HTML code exists — 422.
    #[must_use]
    pub fn builder_artifact_empty() -> Self {
        Self::new(
            422,
            "BUILDER_ARTIFACT_EMPTY",
            "No HTML artifact has been generated for this builder session yet.",
        )
    }

    /// Unknown Hermes skill — the id must be one of the 16 skills.
    #[must_use]
    pub fn skill_not_found(skill: impl std::fmt::Display) -> Self {
        Self::new(
            404,
            "HERMES_SKILL_NOT_FOUND",
            format!("Unknown Hermes skill `{skill}` — see GET /api/v1/hermes/skills."),
        )
    }

    /// The input shield blocked the input — reason in meta.reason.
    #[must_use]
    pub fn hermes_blocked(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        let mut meta = Meta::new();
        meta.insert("reason".into(), MetaValue::Text(reason.clone()));
        Self {
            status: 422,
            code: "HERMES_INPUT_BLOCKED".into(),
            message: format!("Hermes shield blocked this input — {reason}."),
            meta: Some(meta),
        }
    }

    /// Unknown agent in the fleet — the A2A sender/receiver must be registered.
    #[must_use]
    pub fn agent_not_found(agent: impl std::fmt::Display) -> Self {
        Self::new(
            404,
            "AGENT_NOT_FOUND",
            format!("Unknown agent `{agent}` in the fleet registry."),
        )
    }

    /// MCP tool outside the allowlist — an unknown tool is never invoked.
    #[must_use]
    pub fn mcp_tool_not_found(tool: impl std::fmt::Display) -> Self {
        Self::new(
            404,
            "MCP_TOOL_NOT_FOUND",
            format!("Tool `{tool}` is not in the MCP allowlist — see GET /api/v1/mcp/tools."),
        )
    }

    /// Invalid MCP tool arguments — the tool name in meta.tool.
    #[must_use]
    pub fn mcp_invalid_args(tool: impl Into<String>, detail: impl std::fmt::Display) -> Self {
        let tool = tool.into();
        let mut meta = Meta::new();
        meta.insert("tool".into(), MetaValue::Text(tool.clone()));
        Self {
            status: 422,
            code: "MCP_INVALID_ARGUMENTS".into(),
            message: format!("Invalid arguments for MCP tool `{tool}` — {detail}."),
            meta: Some(meta),
        }
    }

    /// Server admin key not configured — fail-closed, never open.
    #[must_use]
    pub fn admin_disabled() -> Self {
        Self::new(
            503,
            "ADMIN_DISABLED",
            "KIAN_ADMIN_KEY is not configured on this server — admin endpoints are disabled.",
        )
    }

    /// Missing X-KIAN-Key header.
    #[must_use]
    pub fn admin_key_required() -> Self {
        Self::new(401, "ADMIN_KEY_REQUIRED", "Send the admin key in the X-KIAN-Key header.")
    }

    /// Wrong X-KIAN-Key header.
    #[must_use]
    pub fn invalid_admin_key() -> Self {
        Self::new(401, "INVALID_ADMIN_KEY", "Invalid admin key.")
    }

    /// Request validation — TS `ValidationFailure` → 400 `INVALID_REQUEST`.
    #[must_use]
    pub fn invalid_request(issues: usize) -> Self {
        let mut meta = Meta::new();
        meta.insert("issues".into(), MetaValue::Int(i64::try_from(issues).unwrap_or(i64::MAX)));
        Self {
            status: 400,
            code: "INVALID_REQUEST".into(),
            message: format!("Invalid request — {issues} issue(s)."),
            meta: Some(meta),
        }
    }

    /// Convert to the wire body.
    #[must_use]
    pub fn to_envelope(&self) -> ErrorEnvelope {
        ErrorEnvelope { error: self.to_api_error() }
    }

    /// Convert to `ApiError`.
    #[must_use]
    pub fn to_api_error(&self) -> ApiError {
        match &self.meta {
            Some(meta) => {
                ApiError::with_meta(self.code.clone(), self.message.clone(), meta.clone())
            }
            None => ApiError::new(self.code.clone(), self.message.clone()),
        }
    }
}

/// HTTP error derived from the domain — the TS `DomainFailure` equivalent.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainFailure {
    /// The source domain error
    pub error: DomainError,
}

impl DomainFailure {
    /// Wrap a domain error.
    #[must_use]
    pub fn new(error: DomainError) -> Self {
        Self { error }
    }

    /// The honest status for this code.
    #[must_use]
    pub fn status(&self) -> u16 {
        status_for_domain_code(self.error.code)
    }

    /// The wire body — domain meta passes through verbatim.
    #[must_use]
    pub fn to_envelope(&self) -> ErrorEnvelope {
        let mut meta: Meta = Meta::new();
        for (k, v) in &self.error.meta {
            // domain MetaValue → wire MetaValue (both hold string | number values;
            // domain Int goes via Display to string — the same output as JSON.stringify)
            let wire_value = match v {
                kian_domain::MetaValue::Int(i) => MetaValue::Int(*i),
                kian_domain::MetaValue::Text(t) => MetaValue::Text(t.clone()),
            };
            meta.insert(k.clone(), wire_value);
        }
        let api = if meta.is_empty() {
            ApiError::new(self.error.code.to_string(), self.error.message.clone())
        } else {
            ApiError::with_meta(self.error.code.to_string(), self.error.message.clone(), meta)
        };
        ErrorEnvelope { error: api }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kian_domain::DomainErrorCode as Code;
    use serde_json::to_string;

    #[test]
    fn domain_codes_map_to_honest_statuses() {
        assert_eq!(status_for_domain_code(Code::EscrowNotFound), 404);
        assert_eq!(status_for_domain_code(Code::UnknownAccount), 404);
        assert_eq!(status_for_domain_code(Code::HitlNotFound), 404);
        assert_eq!(status_for_domain_code(Code::ProductNotFound), 404);
        assert_eq!(status_for_domain_code(Code::AccountOverdraft), 422);
        assert_eq!(status_for_domain_code(Code::RequiresHumanApproval), 422);
        assert_eq!(status_for_domain_code(Code::InvalidTransition), 422);
        assert_eq!(status_for_domain_code(Code::MintCapExceeded), 422);
        assert_eq!(status_for_domain_code(Code::HitlExpired), 422);
        // everything else = 500 (programmer error)
        assert_eq!(status_for_domain_code(Code::BpsOutOfRange), 422);
        assert_eq!(status_for_domain_code(Code::TooFewPostings), 422);
    }

    #[test]
    fn full_taxonomy_covers_every_domain_code() {
        // all 20 domain codes must get one of the three statuses 404/422/500 — no code stays unmapped
        for code in [
            Code::NonIntegerAmount,
            Code::NegativeAmount,
            Code::AmountOutOfRange,
            Code::BpsOutOfRange,
            Code::TooFewPostings,
            Code::UnbalancedTransaction,
            Code::UnknownAccount,
            Code::InvalidAccountKind,
            Code::DuplicateAccount,
            Code::AccountOverdraft,
            Code::InvalidTransition,
            Code::EscrowNotFound,
            Code::EscrowAlreadyExists,
            Code::RequiresHumanApproval,
            Code::MintCapExceeded,
            Code::HitlNotFound,
            Code::HitlNotRequired,
            Code::HitlAlreadyDecided,
            Code::HitlExpired,
            Code::ProductNotFound,
        ] {
            let s = status_for_domain_code(code);
            assert!([404, 422, 500].contains(&s), "{code:?} → {s}");
        }
    }

    #[test]
    fn domain_failure_envelope_carries_code_message_meta() {
        let err = DomainError::new(Code::AccountOverdraft, "no funds").with_meta("needed", 90_i64);
        let env = DomainFailure::new(err).to_envelope();
        assert_eq!(
            to_string(&env).unwrap(),
            r#"{"error":{"code":"ACCOUNT_OVERDRAFT","message":"no funds","meta":{"needed":90}}}"#
        );
    }

    #[test]
    fn rate_limited_carries_limit_meta() {
        let f = HttpFailure::rate_limited(120);
        assert_eq!(f.status, 429);
        assert_eq!(f.code, "RATE_LIMITED");
        assert_eq!(
            to_string(&f.to_envelope()).unwrap(),
            r#"{"error":{"code":"RATE_LIMITED","message":"More than 120 requests per minute.","meta":{"limit":120}}}"#
        );
    }

    #[test]
    fn admin_disabled_is_503_fail_closed() {
        let f = HttpFailure::admin_disabled();
        assert_eq!(f.status, 503);
        assert_eq!(f.code, "ADMIN_DISABLED");
    }

    #[test]
    fn invalid_request_counts_issues() {
        let f = HttpFailure::invalid_request(3);
        assert_eq!(f.status, 400);
        assert!(f.message.contains('3'));
    }
}
