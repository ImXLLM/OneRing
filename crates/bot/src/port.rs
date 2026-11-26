//! Kernel port for the pure orchestrator — async-trait.
//!
//! Implementations:
//! * [`crate::kernel_http::KernelHttpClient`] — real kernel HTTP (long-polling).
//! * `InProcessPort` in `crates/server` — in-process webhook (the same identity path).
//! * Tests: the scripted `FakePort` — zero network.

use async_trait::async_trait;

/// Port error — the bot’s honest messages come out of this enum.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BotError {
    /// Kernel unreachable (network/wrong HTTP).
    #[error("kernel unreachable: {0}")]
    Transport(String),
    /// The kernel returned a shaped error envelope — `{code}`.
    #[error("kernel error {code}: {message}")]
    Kernel { code: String, message: String },
}

impl BotError {
    /// User-facing text (international English, no technical/secret leakage).
    #[must_use]
    pub fn user_text(&self) -> String {
        match self {
            Self::Transport(_) => {
                "The kernel is not reachable right now — please try again shortly (/help)".to_owned()
            }
            Self::Kernel { code, .. } => match code.as_str() {
                "IDENTITY_DISABLED" => {
                    "Identity is currently disabled on the kernel (KIAN_BOT_TOKEN is not set)".to_owned()
                }
                "INVALID_INIT_DATA" => {
                    "Identity signature was rejected — please try again".to_owned()
                }
                "SESSION_EXPIRED" | "SESSION_REQUIRED" => {
                    "Your session has expired — please try again".to_owned()
                }
                _ => format!("The kernel reported an error ({code}) — please try again"),
            },
        }
    }
}

/// Citizen balance view — minimal snapshot from `/me` (Din is a raw number on the wire).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotMe {
    /// Telegram username on the profile.
    pub username: Option<String>,
    /// Ledger account (`citizen:{tg}`).
    pub account_id: String,
    /// Registered TON address — explicit Option.
    pub ton_address: Option<String>,
    /// Free balance in Din.
    pub balance: i64,
    /// Escrow-locked (buyer) amount in Din.
    pub escrow_locked: i64,
}

/// National economy view — minimal snapshot from `/economy` (Article 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotEconomy {
    /// Circulating supply.
    pub circulating: i64,
    /// Treasury balance.
    pub treasury: i64,
    /// Locked in escrows.
    pub escrow_locked: i64,
    /// Total minted since inception.
    pub minted_all: i64,
    /// Total burned.
    pub burned_all: i64,
    /// Account count.
    pub accounts: i64,
    /// Open escrows.
    pub open_escrows: i64,
}

/// Kernel port — the only data gateway for the orchestrator.
#[async_trait]
pub trait KernelPort: Send + Sync {
    /// `GET /economy` — public.
    ///
    /// # Errors
    /// [`BotError::Transport`] / [`BotError::Kernel`]
    async fn economy(&self) -> Result<BotEconomy, BotError>;

    /// Citizen session via the official identity path: HMAC of the initData with `KIAN_BOT_TOKEN`
    /// (`build_bot_init_data`) → `POST /auth/telegram` → `GET /me`.
    ///
    /// # Errors
    /// [`BotError::Transport`] / [`BotError::Kernel`]
    async fn me_via_session(
        &self,
        telegram_id: i64,
        username: Option<&str>,
        now_secs: i64,
    ) -> Result<BotMe, BotError>;
}
