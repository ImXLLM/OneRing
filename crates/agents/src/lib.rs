//! `kian-agents` — Kian's agent fleet — roles, A2A, and MCP (M4).
//!
//! Pure parts (no database — the server executes; this is contracts only):
//! - [`roles`] — the fleet's 7 roles (all active — a 7/7 fleet) + seed rows
//! - [`agent_reach`] — the scout's zero-cost web reader + the guardian security handler
//!   (port of `Agent-Reach` + `starnet`)
//! - [`kya`] — the Know-Your-Agent identity registry (ERC-8004/ES256) + the x402 v2 gateway
//! - [`card`] — the A2A discovery card (`/.well-known/agent-card.json`)
//! - [`mcp`] — the closed tool allowlist + argument validation
//!
//! The honesty principle: no role calls itself "active" unless the kernel actually
//! uses it — the guardian is wired with the real verification handler
//! [`agent_reach::GuardianSecurityHandler`], and the full fleet is
//! now 7/7 active.
#![forbid(unsafe_code)]

pub mod agent_reach;
pub mod card;
pub mod kya;
pub mod mcp;
pub mod roles;

/// Crate ID — for the smoke test and logs.
pub const NAME: &str = "kian-agents";

/// Crate version — read from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use agent_reach::{
    attenuate_capabilities, exa_fetch_payload, exa_search_payload, GuardianSecurityHandler,
    is_antibot_challenge_page, jina_reader_url, StallDecision, SubagentStallWatchdog,
    SurfaceVerdict, ANTIBOT_SCAN_BYTES, EXA_MAX_FETCH_CHARS, EXA_MCP_URL,
    JINA_MAX_RESPONSE_BYTES, JINA_READER_PREFIX,
};
pub use card::build_agent_card;
pub use kya::{KyaMetadata, X402Pricing, KYA_SUPPORTED_JWA, X402_NETWORK, X402_VERSION};
pub use mcp::{tool_def, validate_args, McpToolDef};
pub use roles::{active_roles, role_of, seed_agents, AgentRole, ROLES};
