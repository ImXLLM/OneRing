//! وضعیت مشترک سرور — یک `Arc<AppState>` بین همهٔ هندلرها.
//!
//! دیتابیس ([`V3Store`]) نقطهٔ تنها اتصال دامنه به SQLite است؛ limiter نرخ
//! درون‌حافظه‌ای همان پنجرهٔ لغزان قراردادی را با Mutex پخش می‌کند؛ گیت‌وی AI
//! (M4) یک‌بار در بوت از پیکربندی ساخته می‌شود — کلیدهای اسلات هرگز اینجا لاگ
//! نمی‌شوند؛ ران‌تایم بات (M6) فقط وقتی `KIAN_BOT_TOKEN` و
//! `KIAN_BOT_WEBHOOK_SECRET` هر دو تنظیم شده باشند بالا می‌آید (fail-closed).

use std::sync::Arc;
use std::time::Instant;

use kian_ai::client::{HttpModelClient, UnavailableClient};
use kian_ai::gateway::{AiGateway, ModelClient};
use kian_contracts::ShardedLimiter;
use kian_db::V3Store;

use crate::config::ServerConfig;
use crate::handlers::bot::BotRuntime;

/// وضعیت مشترک همهٔ هندلرها.
pub struct AppState {
    /// فروشگاه v3 — نقطهٔ تنها اتصال دامنه به SQLite.
    pub store: V3Store,
    /// پیکربندی بوت (تغییرناپذیر).
    pub config: ServerConfig,
    /// limiter نرخ پنجرهٔ لغزان شاردشده — یکی به ازای سرور؛ قفل‌ها per-shard
    /// (اصلاح تست بار ۲۰۲۵-۱۱: Mutex سراسری در RPS بالا نقطهٔ داغ بود).
    pub limiter: ShardedLimiter,
    /// لحظهٔ شروع — uptime در health.
    pub started_at: Instant,
    /// گیت‌وی AI (M4) — فال‌بک سه‌اسلات؛ بدون اسلات کانفیگ، صادقانه `Disabled`.
    pub ai: Arc<AiGateway>,
    /// ران‌تایم بات (M6) — `None` یعنی وبهوک صادقانه غیرفعال.
    pub bot: Option<BotRuntime>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // گیت‌وی AI با Debug امن خودش چاپ می‌شود (کلیدها هرگز)؛ ران‌تایم بات
        // توکن دارد — غیرشامل می‌شود.
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("ai", &self.ai)
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// ساخت با کلاینت AI تزریقی و ران‌تایم بات تزریقی — مسیر تولیدی `new`
    /// است؛ تست‌ها `ScriptedClient` و `RecordingSender` تزریق می‌کنند.
    #[must_use]
    pub fn with_parts(
        store: V3Store,
        config: ServerConfig,
        client: Arc<dyn ModelClient>,
        bot: Option<BotRuntime>,
    ) -> Self {
        Self {
            limiter: ShardedLimiter::new(config.rate_limit),
            ai: Arc::new(AiGateway::new(config.ai.clone(), client)),
            store,
            config,
            started_at: Instant::now(),
            bot,
        }
    }

    /// ران‌تایم بات تولیدی — فقط با هر دو راز (توکن + راز وبهوک ≥ ۱۶ کاراکتر).
    fn build_bot(store: &V3Store, config: &ServerConfig) -> Option<BotRuntime> {
        let (Some(token), Some(secret)) = (&config.bot_token, &config.bot_webhook_secret) else {
            return None;
        };
        if secret.len() < 16 {
            tracing::warn!("KIAN_BOT_WEBHOOK_SECRET too short (<16) — webhook disabled");
            return None;
        }
        match BotRuntime::production(store.clone(), token, config.session_ttl_hours, &config.miniapp_url) {
            Ok(rt) => Some(rt),
            Err(e) => {
                tracing::warn!(error = %e, "telegram sender init failed — webhook disabled");
                None
            }
        }
    }

    /// ساخت وضعیت با کلاینت AI تزریقی — ران‌تایم بات از پیکربندی ساخته می‌شود.
    #[must_use]
    pub fn with_gateway(
        store: V3Store,
        config: ServerConfig,
        client: Arc<dyn ModelClient>,
    ) -> Self {
        let bot = Self::build_bot(&store, &config);
        Self::with_parts(store, config, client, bot)
    }

    /// ساخت وضعیت — `started_at` همین‌جا قفل می‌شود؛ کلاینت HTTP تولیدی با
    /// تایم‌اوت ۹۰ ثانیه (سنتز کامل HTML روی لِین‌های رایگان ۲۲ تا ۷۷ ثانیه
    /// طول می‌کشد — تایم‌اوت ۳۰ ثانیه هر نوبت سازنده را می‌کُشت).
    /// شکست ساخت کلاینت → کلاینت همیشه-رد (گیت‌وی `Disabled`).
    #[must_use]
    pub fn new(store: V3Store, config: ServerConfig) -> Self {
        let client: Arc<dyn ModelClient> = match HttpModelClient::new(std::time::Duration::from_secs(90)) {
            Ok(c) => Arc::new(c),
            Err(_) => Arc::new(UnavailableClient),
        };
        Self::with_gateway(store, config, client)
    }

    /// وضعیت اشتراکی — `Arc` برای کلون‌شدن داخل هر هندلر.
    #[must_use]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}
