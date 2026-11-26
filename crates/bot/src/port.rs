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
    /// User-facing text (Persian, no technical/secret leakage).
    #[must_use]
    pub fn user_text(&self) -> String {
        match self {
            Self::Transport(_) => {
                "\u{6a9}\u{631}\u{646}\u{644} \u{62f}\u{631} \u{62f}\u{633}\u{62a}\u{631}\u{633} \u{646}\u{6cc}\u{633}\u{62a} — \u{6a9}\u{645}\u{6cc} \u{628}\u{639}\u{62f} \u{62f}\u{648}\u{628}\u{627}\u{631}\u{647} \u{627}\u{645}\u{62a}\u{62d}\u{627}\u{646} \u{6a9}\u{646}\u{6cc}\u{62f} (/help)".to_owned()
            }
            Self::Kernel { code, .. } => match code.as_str() {
                "IDENTITY_DISABLED" => "\u{647}\u{648}\u{6cc}\u{62a} \u{631}\u{648}\u{6cc} \u{6a9}\u{631}\u{646}\u{644} \u{63a}\u{6cc}\u{631}\u{641}\u{639}\u{627}\u{644} \u{627}\u{633}\u{62a} (KIAN_BOT_TOKEN \u{62a}\u{646}\u{638}\u{6cc}\u{645} \u{646}\u{634}\u{62f}\u{647})".to_owned(),
                "INVALID_INIT_DATA" => "\u{627}\u{645}\u{636}\u{627}\u{6cc} \u{647}\u{648}\u{6cc}\u{62a} \u{67e}\u{630}\u{6cc}\u{631}\u{641}\u{62a}\u{647} \u{646}\u{634}\u{62f} — \u{62f}\u{648}\u{628}\u{627}\u{631}\u{647} \u{62a}\u{644}\u{627}\u{634} \u{6a9}\u{646}\u{6cc}\u{62f}".to_owned(),
                "SESSION_EXPIRED" | "SESSION_REQUIRED" => "\u{646}\u{634}\u{633}\u{62a} \u{645}\u{646}\u{642}\u{636}\u{6cc} \u{634}\u{62f} — \u{62f}\u{648}\u{628}\u{627}\u{631}\u{647} \u{62a}\u{644}\u{627}\u{634} \u{6a9}\u{646}\u{6cc}\u{62f}".to_owned(),
                _ => format!("\u{6a9}\u{631}\u{646}\u{644} \u{62e}\u{637}\u{627} \u{62f}\u{627}\u{62f} ({code}) — \u{62f}\u{648}\u{628}\u{627}\u{631}\u{647} \u{62a}\u{644}\u{627}\u{634} \u{6a9}\u{646}\u{6cc}\u{62f}"),
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
