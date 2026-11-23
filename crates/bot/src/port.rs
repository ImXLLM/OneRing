//! پورت کرنل برای orchestrator خالص — async-trait.
//!
//! پیاده‌سازی‌ها:
//! * [`crate::kernel_http::KernelHttpClient`] — HTTP واقعی کرنل (طول‌کشی).
//! * `InProcessPort` در `crates/server` — وبوک درون‌پردازشی (همان مسیر هویت).
//! * تست‌ها: `FakePort` اسکریپت‌شده — صفر شبکه.

use async_trait::async_trait;

/// خطای پورت — پیام‌های بات صادقانه از همین میان‌می‌آیند.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BotError {
    /// کرنل در دسترس نیست (شبکه/HTTP غلط).
    #[error("kernel unreachable: {0}")]
    Transport(String),
    /// کرنل پاکت خطای شکل‌دار داد — `{code}`.
    #[error("kernel error {code}: {message}")]
    Kernel { code: String, message: String },
}

impl BotError {
    /// متن کاربر-رو (فارسی، بدون نشت جزئیات فنی/راز).
    #[must_use]
    pub fn user_text(&self) -> String {
        match self {
            Self::Transport(_) => {
                "کرنل در دسترس نیست — کمی بعد دوباره امتحان کنید (/help)".to_owned()
            }
            Self::Kernel { code, .. } => match code.as_str() {
                "IDENTITY_DISABLED" => "هویت روی کرنل غیرفعال است (KIAN_BOT_TOKEN تنظیم نشده)".to_owned(),
                "INVALID_INIT_DATA" => "امضای هویت پذیرفته نشد — دوباره تلاش کنید".to_owned(),
                "SESSION_EXPIRED" | "SESSION_REQUIRED" => "نشست منقضی شد — دوباره تلاش کنید".to_owned(),
                _ => format!("کرنل خطا داد ({code}) — دوباره تلاش کنید"),
            },
        }
    }
}

/// نمای تراز شهروند — اسنپ‌شات مینیمال از `/me` (Din روی سیم عدد خام است).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotMe {
    /// username تلگرام روی پرونده.
    pub username: Option<String>,
    /// حساب دفتر (`citizen:{tg}`).
    pub account_id: String,
    /// آدرس TON ثبت‌شده — Option صریح.
    pub ton_address: Option<String>,
    /// تراز آزاد به Din.
    pub balance: i64,
    /// قفل اسکرو (خریدار) به Din.
    pub escrow_locked: i64,
}

/// نمای اقتصاد ملی — اسنپ‌شات مینیمال از `/economy` (مادهٔ ۶).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotEconomy {
    /// عرضه در گردش.
    pub circulating: i64,
    /// تراز خزانه.
    pub treasury: i64,
    /// قفل در امانت‌ها.
    pub escrow_locked: i64,
    /// کل ضرب از آغاز.
    pub minted_all: i64,
    /// کل سوزانده.
    pub burned_all: i64,
    /// شمار حساب‌ها.
    pub accounts: i64,
    /// امانت‌های باز.
    pub open_escrows: i64,
}

/// پورت کرنل — تنها دروازهٔ داده برای orchestrator.
#[async_trait]
pub trait KernelPort: Send + Sync {
    /// `GET /economy` — عمومی.
    ///
    /// # Errors
    /// [`BotError::Transport`] / [`BotError::Kernel`]
    async fn economy(&self) -> Result<BotEconomy, BotError>;

    /// نشست شهروند از مسیر رسمی هویت: ضرب initData با `KIAN_BOT_TOKEN`
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
