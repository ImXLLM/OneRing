//! KIAN bot binary — Telegram long-polling entry point.
//!
//! Honest fail-closed: without `KIAN_BOT_TOKEN` the bot is **off** — it exits
//! cleanly with a log (exit 0), never starting with a fake token.

#![forbid(unsafe_code)]

use kian_bot::shell::{BotRunConfig, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,teloxide=warn")),
        )
        .init();

    let Ok(bot_token) = std::env::var("KIAN_BOT_TOKEN") else {
        tracing::warn!("KIAN_BOT_TOKEN is not set — the bot is honestly off (exit 0)");
        return Ok(());
    };
    if bot_token.trim().is_empty() {
        tracing::warn!("KIAN_BOT_TOKEN is empty — the bot is honestly off (exit 0)");
        return Ok(());
    }

    let cfg = BotRunConfig {
        bot_token: bot_token.trim().to_owned(),
        kernel_url: std::env::var("KIAN_KERNEL_URL").unwrap_or_else(|_| "http://127.0.0.1:8088".to_owned()),
        miniapp_url: std::env::var("KIAN_MINIAPP_URL").unwrap_or_else(|_| "http://localhost:3000".to_owned()),
    };
    tracing::info!(kernel = %cfg.kernel_url, "starting the MAIRON bot shell");
    run(cfg).await
}
