//! The teloxide 0.17 polling shell — thin: update → pure parse → orchestrator →
//! [`BotAction`] execution. All real logic lives in `orchestrator` so it tests offline.
//!
//! Topology: `KIAN_BOT_TOKEN` lives only here (and in the client's initData signing);
//! logs never become tokens. TLS via rustls (never openssl-system).

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use teloxide::payloads::SendMessageSetters;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::update_listeners::{AsUpdateStream, UpdateListener, polling_default};
use teloxide::{Bot, prelude::Requester};

use crate::kernel_http::KernelHttpClient;
use crate::orchestrator::{BotAction, BotCore, BotPolicy};
use crate::update::BotUpdate;

/// Run configuration — comes from env in `main`.
#[derive(Debug, Clone)]
pub struct BotRunConfig {
    /// Bot token — from `KIAN_BOT_TOKEN`.
    pub bot_token: String,
    /// Kernel base — from `KIAN_KERNEL_URL`.
    pub kernel_url: String,
    /// Mini-app URL — from `KIAN_MINIAPP_URL`.
    pub miniapp_url: String,
}

/// Run the polling loop — until `Ctrl+C` or stream termination.
///
/// # Errors
/// Invalid token (`get_me`) or listener failure — bubbles up with `anyhow`.
pub async fn run(cfg: BotRunConfig) -> anyhow::Result<()> {
    let bot = Bot::new(cfg.bot_token.clone());
    let me = bot.get_me().await?;
    let bot_username = me.user.username.clone();
    tracing::info!(bot = %me.user.username.as_deref().unwrap_or("?"), "KIAN bot long-polling started");

    let port = Arc::new(KernelHttpClient::new(&cfg.kernel_url, &cfg.bot_token)?);
    let core = tokio::sync::Mutex::new(BotCore::new(BotPolicy {
        bot_username,
        miniapp_url: cfg.miniapp_url,
        ..BotPolicy::default()
    }));

    let mut listener = polling_default(bot.clone()).await;
    let stop_token = listener.stop_token();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("ctrl-c — polling stops after current batch");
        stop_token.stop();
    });

    // single parse path — teloxide Update → parts → BotUpdate (same path as the webhook)
    let mut updates = std::pin::pin!(listener.as_stream());
    while let Some(Ok(update)) = updates.next().await {
        let teloxide::types::Update { id, kind } = update;
        let teloxide::types::UpdateKind::Message(msg) = kind else { continue };
        let Some(from) = msg.from.as_ref() else { continue }; // channel post — no sender
        if from.is_bot {
            continue; // bots get no replies
        }
        let from_id = i64::try_from(from.id.0).unwrap_or(0);
        if from_id <= 0 {
            continue;
        }
        let Some(text) = msg.text() else { continue }; // photo/media — no text, no reply
        let bot_update =
            BotUpdate::from_parts(i64::from(id.0), msg.chat.id.0, from_id, from.username.as_deref(), text);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
        let actions = {
            let mut core = core.lock().await;
            core.handle(&bot_update, now_ms, port.as_ref()).await
        };
        for action in actions {
            execute(&bot, &action).await;
        }
    }
    tracing::info!("polling stream ended — shell exits");
    Ok(())
}

/// Execute one action — Telegram API only; failures log and continue (the bot never dies).
async fn execute(bot: &Bot, action: &BotAction) {
    let chat = ChatId(action.chat_id);
    let mut req = bot.send_message(chat, action.text.clone());
    if !action.links.is_empty() {
        let mut row = Vec::new();
        for link in &action.links {
            match link.url.parse::<reqwest::Url>() {
                Ok(url) => row.push(InlineKeyboardButton::url(link.label.clone(), url)),
                Err(e) => tracing::warn!(url = %link.url, error = %e, "link button skipped"),
            }
        }
        if !row.is_empty() {
            req = req.reply_markup(InlineKeyboardMarkup::new(vec![row]));
        }
    }
    if let Err(e) = req.await {
        tracing::warn!(chat = action.chat_id, error = %e, "send_message failed");
    }
}
