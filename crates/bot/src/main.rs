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
        tracing::warn!("KIAN_BOT_TOKEN \u{62a}\u{646}\u{638}\u{6cc}\u{645} \u{646}\u{634}\u{62f}\u{647} — \u{628}\u{627}\u{62a} \u{635}\u{627}\u{62f}\u{642}\u{627}\u{646}\u{647} \u{62e}\u{627}\u{645}\u{648}\u{634} \u{627}\u{633}\u{62a} (exit 0)");
        return Ok(());
    };
    if bot_token.trim().is_empty() {
        tracing::warn!("KIAN_BOT_TOKEN \u{62e}\u{627}\u{644}\u{6cc} \u{627}\u{633}\u{62a} — \u{628}\u{627}\u{62a} \u{635}\u{627}\u{62f}\u{642}\u{627}\u{646}\u{647} \u{62e}\u{627}\u{645}\u{648}\u{634} \u{627}\u{633}\u{62a} (exit 0)");
        return Ok(());
    }

    let cfg = BotRunConfig {
        bot_token: bot_token.trim().to_owned(),
        kernel_url: std::env::var("KIAN_KERNEL_URL").unwrap_or_else(|_| "http://127.0.0.1:8088".to_owned()),
        miniapp_url: std::env::var("KIAN_MINIAPP_URL").unwrap_or_else(|_| "http://localhost:3000".to_owned()),
    };
    tracing::info!(kernel = %cfg.kernel_url, "starting KIAN bot shell");
    run(cfg).await
}
