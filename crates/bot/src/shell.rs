//! شِل طول‌کشی teloxide 0.17 — نازک: آپدیت → تجزیهٔ خالص → orchestrator →
//! اجرای [`BotAction`]. هر منطق واقعی در `orchestrator` است تا آفلاین تست شود.
//!
//! توپولوژی: `KIAN_BOT_TOKEN` فقط همین‌جا (و ضرب initData در کلاینت) است؛
//! لاگ‌ها هرگز توکن نمی‌شوند. TLS از rustls (هرگز openssl-system).

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

/// پیکربندی اجرا — از env در `main` می‌آید.
#[derive(Debug, Clone)]
pub struct BotRunConfig {
    /// توکن بات — از `KIAN_BOT_TOKEN`.
    pub bot_token: String,
    /// پایهٔ کرنل — از `KIAN_KERNEL_URL`.
    pub kernel_url: String,
    /// URL مینی‌اپ — از `KIAN_MINIAPP_URL`.
    pub miniapp_url: String,
}

/// اجرای طول‌کشی — تا `Ctrl+C` یا خاتمهٔ stream.
///
/// # Errors
/// توکن نامعتبر (`get_me`) یا شکست listener — با `anyhow` بالا می‌رود.
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

    // مسیر واحد تجزیه — teloxide Update → اجزا → BotUpdate (همان مسیر وبهوک)
    let mut updates = std::pin::pin!(listener.as_stream());
    while let Some(Ok(update)) = updates.next().await {
        let teloxide::types::Update { id, kind } = update;
        let teloxide::types::UpdateKind::Message(msg) = kind else { continue };
        let Some(from) = msg.from.as_ref() else { continue }; // پست کانال — بدون فرستنده
        if from.is_bot {
            continue; // بات‌ها پاسخ نمی‌گیرند
        }
        let from_id = i64::try_from(from.id.0).unwrap_or(0);
        if from_id <= 0 {
            continue;
        }
        let Some(text) = msg.text() else { continue }; // عکس/رسانه — بدون متن، بدون پاسخ
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

/// اجرای یک اکشن — فقط تلگرام API؛ شکست‌ها لاگ و ادامه (بات نمی‌میرد).
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
