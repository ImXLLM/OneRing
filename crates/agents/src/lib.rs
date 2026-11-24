//! `kian-agents` — ناوگان ایجنت‌های کیان — نقش‌ها، A2A و MCP (M4).
//!
//! قطعات خالص (بدون دیتابیس — سرور اجرا می‌کند، اینجا فقط قرارداد است):
//! - [`roles`] — ۷ نقش ناوگان (همگی فعال — ناوگان ۷/۷) + ردیف‌های بذر
//! - [`agent_reach`] — خوانندهٔ وب صفر-هزینهٔ کاوشگر + هندلر امنیت نگهبان
//!   (پورت `Agent-Reach` + `starnet`)
//! - [`kya`] — شناسنامهٔ Know-Your-Agent (ERC-8004/ES256) + دروازهٔ x402 v2
//! - [`card`] — کارت کشف A2A (`/.well-known/agent-card.json`)
//! - [`mcp`] — allowlist بستهٔ ابزارها + اعتبارسنجی آرگومان‌ها
//!
//! اصل صادق‌بودن: هیچ نقشی خودش را «فعال» نمی‌نامد مگر کرنل واقعاً از آن
//! استفاده کند — نگهبان با هندلر راستی‌آزمایی واقعی
//! [`agent_reach::GuardianSecurityHandler`] سیم شده و اینک ناوگان کامل
//! ۷/۷ فعال است.
#![forbid(unsafe_code)]

pub mod agent_reach;
pub mod card;
pub mod kya;
pub mod mcp;
pub mod roles;

/// شناسه کرِیت — برای smoke test و لاگ‌ها.
pub const NAME: &str = "kian-agents";

/// نسخه کرِیت — از `Cargo.toml` خوانده می‌شود.
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
