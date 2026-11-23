//! `kian-agents` — ناوگان ایجنت‌های کیان — نقش‌ها، A2A و MCP (M4).
//!
//! سه قطعهٔ خالص (بدون دیتابیس — سرور اجرا می‌کند، اینجا فقط قرارداد است):
//! - [`roles`] — ۷ نقش ناوگان + ردیف‌های بذر
//! - [`card`] — کارت کشف A2A (`/.well-known/agent-card.json`)
//! - [`mcp`] — allowlist بستهٔ ابزارها + اعتبارسنجی آرگومان‌ها
//!
//! اصل صادق‌بودن: هیچ نقشی خودش را «فعال» نمی‌نامد مگر کرنل واقعاً از آن
//! استفاده کند — orchestrator فعال است (مغز همین کرنل)، بقیه `registered`
//! تا وایر شدن واقعی در M5.
#![forbid(unsafe_code)]

pub mod card;
pub mod mcp;
pub mod roles;

/// شناسه کرِیت — برای smoke test و لاگ‌ها.
pub const NAME: &str = "kian-agents";

/// نسخه کرِیت — از `Cargo.toml` خوانده می‌شود.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use card::build_agent_card;
pub use mcp::{tool_def, validate_args, McpToolDef};
pub use roles::{role_of, seed_agents, AgentRole, ROLES};
