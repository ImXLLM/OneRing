//! Shared server state — one `Arc<AppState>` shared by all handlers.
//!
//! Database ([`V3Store`]) — the domain's single SQLite connection point; the rate
//! limiter in memory enforces the same contractual sliding window via a Mutex; the AI
//! gateway (M4) is built once from configuration at boot — slot keys are never logged
//! here; the bot runtime (M6) starts only when `KIAN_BOT_TOKEN` and
//! `KIAN_BOT_WEBHOOK_SECRET` are both set (fail-closed).

use std::sync::Arc;
use std::time::Instant;

use kian_ai::client::{HttpModelClient, UnavailableClient};
use kian_ai::gateway::{AiGateway, ModelClient};
use kian_contracts::ShardedLimiter;
use kian_db::V3Store;

use crate::config::ServerConfig;
use crate::handlers::bot::BotRuntime;

/// Shared state for all handlers.
pub struct AppState {
    /// v3 store — the domain's single SQLite connection point.
    pub store: V3Store,
    /// Boot configuration (immutable).
    pub config: ServerConfig,
    /// Sharded sliding-window rate limiter — one per server; locks are per-shard
    /// (2025-11 load-test fix: the global Mutex was a hot spot at high RPS).
    pub limiter: ShardedLimiter,
    /// Start instant — uptime in health.
    pub started_at: Instant,
    /// AI gateway (M4) — three-slot fallback; with no configured slot, honestly `Disabled`.
    pub ai: Arc<AiGateway>,
    /// Bot runtime (M6) — `None` means the webhook is honestly disabled.
    pub bot: Option<BotRuntime>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The AI gateway prints via its own safe Debug (keys never); the bot runtime
        // holds a token — excluded from serialization.
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("ai", &self.ai)
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// Build with an injected AI client and injected bot runtime — the production path is `new`
    /// ; tests inject `ScriptedClient` and `RecordingSender`.
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

    /// Production bot runtime — only with both secrets (token + webhook secret ≥ 16 chars).
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

    /// Build state with an injected AI client — the bot runtime is built from configuration.
    #[must_use]
    pub fn with_gateway(
        store: V3Store,
        config: ServerConfig,
        client: Arc<dyn ModelClient>,
    ) -> Self {
        let bot = Self::build_bot(&store, &config);
        Self::with_parts(store, config, client, bot)
    }

    /// Build state — `started_at` is locked here; the production HTTP client uses
    /// a 90-second timeout (full HTML synthesis on free lanes takes 22–77 seconds,
    /// — a 30-second timeout killed every generation run).
    /// Client construction failure → always-rejecting client (gateway `Disabled`).
    #[must_use]
    pub fn new(store: V3Store, config: ServerConfig) -> Self {
        let client: Arc<dyn ModelClient> = match HttpModelClient::new(std::time::Duration::from_secs(90)) {
            Ok(c) => Arc::new(c),
            Err(_) => Arc::new(UnavailableClient),
        };
        Self::with_gateway(store, config, client)
    }

    /// Shared state — `Arc` so it clones into every handler.
    #[must_use]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}
