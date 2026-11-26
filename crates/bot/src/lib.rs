//! kian-bot — the MAIRON sovereign Telegram adapter — a pure, offline-testable orchestrator.
//!
//! Architecture (the same repo-wide pattern — pure domain + thin shell):
//!
//! * [`orchestrator`] — a **100% pure** core: command parsing, rate cap with an injected
//!   clock, message splitting, [`BotAction`] construction. No I/O — every path is
//!   tested offline with `FakePort`.
//! * [`update`] — parsing a Telegram update from a plain `serde_json::Value`; **both
//!   shells** (the teloxide polling loop and the kernel webhook) parse through this single path.
//! * [`backlog`] — backlog drain at boot (port of the Hermes bot `drain_backlog`;
//!   `offset = -1, timeout = 0`) — anti double-reply after restart.
//! * [`port`] — the [`KernelPort`] port (async-trait): national economy + citizen session.
//! * [`kernel_http`] — the real port implementation over kernel HTTP.
//! * [`shell`] — the teloxide 0.17 polling loop (rustls; never openssl-system).
//!
//! Identity principle (article 1 — fail-closed): the bot holds `KIAN_BOT_TOKEN` and
//! updates come from Telegram's trusted channel. For citizen data, the bot signs initData
//! with [`kian_contracts::build_bot_init_data`] and `POST /auth/telegram`
//! follows the same official mini-app path — **there is no side path for identity or money
//! anywhere**. The bot token never appears in logs/responses (tested).

pub mod backlog;
pub mod kernel_http;
pub mod loop_guard;
pub mod orchestrator;
pub mod port;
pub mod shell;
pub mod sovereign;
pub mod telegram_api9;
pub mod update;

pub use backlog::{drain_backlog_offset, next_offset, DrainOutcome};
pub use loop_guard::{BotLoopGuard, LoopGuardDecision, LoopReason, MAX_CONSECUTIVE_BOT_TURNS};
pub use orchestrator::{BotAction, BotCore, BotPolicy, LinkButton};
pub use sovereign::{
    format_mairon, format_quant, format_sovereign_command, format_trinity, format_treasury,
    format_warroom, is_sovereign_command, BASE_USDC_CONTRACT, GROK_VOICES, KEYLESS_VENUES,
    MAIRON_MIN_SHARPE, REA_MIN_SCORE, TRINITY_IQ_ALPHA, TRINITY_IQ_COMMAND_CENTER,
    TRINITY_IQ_MAIRON, WAR_ROOM_TICKS,
};
pub use port::{BotEconomy, BotError, BotMe, KernelPort};
pub use telegram_api9::{
    validate_init_data, InitDataError, StartAppPayload, StartAppPayloadError,
    StarsInvoicePayload, StarsPayloadError, ValidatedInitData,
};
pub use update::BotUpdate;
