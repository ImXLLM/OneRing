//! Telegram bot webhook (M6) — `POST /bot/webhook` — a second shell over the pure orchestrator.
//!
//! Guards (all fail-closed, per the Phase 5 contract anticipated in
//! `contracts::error`):
//! * Server without `KIAN_BOT_WEBHOOK_SECRET` → 503 `BOT_DISABLED` — never open.
//! * Missing/incorrect `X-Telegram-Bot-Api-Secret-Token` header → 401 `BOT_UNAUTHORIZED`
//!   (constant-time comparison via [`verify_kian_key`]).
//! * Invalid update → 200 with `handled:false` — Telegram must receive 200; an error means
//!   nothing to Telegram and is only logged.
//!
//! Identity: updates arrive directly from the trusted Telegram channel (signed webhook); for citizen
//! data, [`InProcessPort`] follows the same official mini-app path — initData
//! is minted with `build_bot_init_data` and verified with `verify_telegram_init_data`;
//! **there is no side path for identity or money.**
//!
//! Responses go through [`TelegramSender`] — in tests `RecordingSender`
//! (no network), in production `TelegramHttpSender` over the official Bot API.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use kian_bot::orchestrator::{BotAction, BotCore, BotPolicy, LinkButton};
use kian_bot::port::{BotEconomy, BotError, BotMe, KernelPort};
use kian_bot::update::BotUpdate;
use kian_contracts::{
    build_bot_init_data, verify_kian_key, verify_telegram_init_data_at,
};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::auth::{new_session_token, session_expires_at};
use crate::state::AppState;

/// Webhook secret header name — official Telegram.
pub const WEBHOOK_SECRET_HEADER: &str = "x-telegram-bot-api-secret-token";

// ── Response delivery port to Telegram ───────────────────────────────────────────────

/// Recorded message for tests — no network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedMessage {
    /// Target chat.
    pub chat_id: i64,
    /// Response text.
    pub text: String,
    /// Link buttons (label, url) — transparent multi-button row.
    pub links: Vec<(String, String)>,
}

/// Delivery port — the only output to Telegram; tests record, production sends HTTP.
#[async_trait]
pub trait TelegramSender: Send + Sync {
    /// Send one message — failure is only logged (the bot does not die).
    async fn send_message(&self, msg: &RecordedMessage) -> Result<(), String>;
}

/// Production implementation — official Bot API with rustls; the token never enters logs.
pub struct TelegramHttpSender {
    http: reqwest::Client,
    bot_token: String,
}

impl TelegramHttpSender {
    /// Build with a bot token.
    ///
    /// # Errors
    /// Build the TLS client.
    pub fn new(bot_token: &str) -> reqwest::Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()?,
            bot_token: bot_token.to_owned(),
        })
    }
}

#[async_trait]
impl TelegramSender for TelegramHttpSender {
    async fn send_message(&self, msg: &RecordedMessage) -> Result<(), String> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let mut body = serde_json::json!({ "chat_id": msg.chat_id, "text": msg.text });
        if !msg.links.is_empty() {
            let row: Vec<_> = msg
                .links
                .iter()
                .map(|(label, btn_url)| serde_json::json!({ "text": label, "url": btn_url }))
                .collect();
            body["reply_markup"] = serde_json::json!({ "inline_keyboard": [row] });
        }
        let res = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(format!("telegram sendMessage status {}", res.status().as_u16()))
        }
    }
}

/// Test recorder — zero network, zero secrets.
#[derive(Default, Clone)]
pub struct RecordingSender {
    /// Sent messages.
    pub sent: Arc<Mutex<Vec<RecordedMessage>>>,
}

#[async_trait]
impl TelegramSender for RecordingSender {
    async fn send_message(&self, msg: &RecordedMessage) -> Result<(), String> {
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(msg.clone());
        Ok(())
    }
}

// ── In-process kernel port ──────────────────────────────────────────────────

/// Kernel port without HTTP loopback — the same store data, the same identity path.
pub struct InProcessPort {
    /// v3 store — the same connection point as the handlers.
    store: kian_db::V3Store,
    /// Bot token for minting initData (never logged).
    bot_token: String,
    /// Session TTL in hours.
    session_ttl_hours: i64,
}

impl InProcessPort {
    /// Build from config + store (no Arc cycle — the store is cloned).
    #[must_use]
    pub fn new(store: kian_db::V3Store, bot_token: &str, session_ttl_hours: i64) -> Self {
        Self { store, bot_token: bot_token.to_owned(), session_ttl_hours }
    }

    /// Citizen session via the official identity path: mint → official verification → ensure → session.
    async fn session_user(
        &self,
        telegram_id: i64,
        username: Option<&str>,
        now_secs: i64,
    ) -> Result<kian_db::UserRow, BotError> {
        let tg = telegram_id.to_string();
        let init_data = build_bot_init_data(&self.bot_token, telegram_id, username, now_secs);
        let identity = verify_telegram_init_data_at(&init_data, &self.bot_token, now_secs)
            .ok_or(BotError::Kernel {
                code: "INVALID_INIT_DATA".to_owned(),
                message: "self-verification failed".to_owned(),
            })?;
        if identity.user_id != tg {
            return Err(BotError::Kernel {
                code: "INVALID_INIT_DATA".to_owned(),
                message: "identity mismatch".to_owned(),
            });
        }
        self.store
            .ensure_telegram_user(&identity.user_id, identity.username.as_deref())
            .await
            .map_err(|e| BotError::Kernel { code: "DB".to_owned(), message: e.to_string() })
    }
}

#[async_trait]
impl KernelPort for InProcessPort {
    async fn economy(&self) -> Result<BotEconomy, BotError> {
        let s = self
            .store
            .economy_summary()
            .await
            .map_err(|e| BotError::Kernel { code: "DB".to_owned(), message: e.to_string() })?;
        let minted = self
            .store
            .minted_all_time()
            .await
            .map_err(|e| BotError::Kernel { code: "DB".to_owned(), message: e.to_string() })?;
        let burned = self
            .store
            .burned_all_time()
            .await
            .map_err(|e| BotError::Kernel { code: "DB".to_owned(), message: e.to_string() })?;
        let accounts = self
            .store
            .account_count()
            .await
            .map_err(|e| BotError::Kernel { code: "DB".to_owned(), message: e.to_string() })?;
        Ok(BotEconomy {
            circulating: s.circulating_supply,
            treasury: s.treasury_balance,
            escrow_locked: s.escrow_locked,
            minted_all: minted,
            burned_all: burned,
            accounts,
            open_escrows: s.open_escrows,
        })
    }

    async fn me_via_session(
        &self,
        telegram_id: i64,
        username: Option<&str>,
        now_secs: i64,
    ) -> Result<BotMe, BotError> {
        let user = self.session_user(telegram_id, username, now_secs).await?;
        let token = new_session_token();
        let expires_at = session_expires_at(self.session_ttl_hours);
        self.store
            .create_session(&token, &user.id, &expires_at)
            .await
            .map_err(|e| BotError::Kernel { code: "DB".to_owned(), message: e.to_string() })?;
        let balance = self
            .store
            .balance_of(&user.account_id)
            .await
            .map_err(|e| BotError::Kernel { code: "DB".to_owned(), message: e.to_string() })?
            .unwrap_or(0);
        let locked = self
            .store
            .locked_for_buyer(&user.account_id)
            .await
            .map_err(|e| BotError::Kernel { code: "DB".to_owned(), message: e.to_string() })?;
        Ok(BotMe {
            username: user.username,
            account_id: user.account_id,
            ton_address: user.ton_address,
            balance,
            escrow_locked: locked,
        })
    }
}

// ── Bot runtime on AppState ───────────────────────────────────────────────

/// Webhook runtime — the pure core + two ports; only when both secrets are set.
pub struct BotRuntime {
    /// Pure core — per-chat rate cap (async lock — handle is awaited inside the lock).
    pub core: tokio::sync::Mutex<BotCore>,
    /// Delivery port to Telegram.
    pub sender: Arc<dyn TelegramSender>,
    /// In-process kernel port.
    pub port: InProcessPort,
}

impl BotRuntime {
    /// Production constructor — a real Bot API sender.
    ///
    /// # Errors
    /// Build the TLS client.
    pub fn production(
        store: kian_db::V3Store,
        bot_token: &str,
        session_ttl_hours: i64,
        miniapp_url: &str,
    ) -> reqwest::Result<Self> {
        Ok(Self {
            core: tokio::sync::Mutex::new(BotCore::new(BotPolicy {
                bot_username: None,
                miniapp_url: miniapp_url.to_owned(),
                ..BotPolicy::default()
            })),
            sender: Arc::new(TelegramHttpSender::new(bot_token)?),
            port: InProcessPort::new(store, bot_token, session_ttl_hours),
        })
    }
}

// ── Webhook handler ──────────────────────────────────────────────────────────────

/// Webhook secret guard — three-state fail-closed like the admin key.
fn require_webhook_secret(headers: &HeaderMap, secret: Option<&str>) -> Result<(), ApiFailure> {
    let header_value = headers.get(WEBHOOK_SECRET_HEADER).and_then(|v| v.to_str().ok());
    if secret.is_none() {
        return Err(kian_contracts::HttpFailure::bot_disabled().into());
    }
    if header_value.is_none() {
        return Err(kian_contracts::HttpFailure::bot_unauthorized().into());
    }
    if !verify_kian_key(header_value, secret) {
        return Err(kian_contracts::HttpFailure::bot_unauthorized().into());
    }
    Ok(())
}

/// Run actions on the sender — each message failure is logged and we continue.
async fn execute_actions(sender: &dyn TelegramSender, actions: &[BotAction]) -> usize {
    let mut sent = 0;
    for a in actions {
        let msg = RecordedMessage {
            chat_id: a.chat_id,
            text: a.text.clone(),
            links: a
                .links
                .iter()
                .map(|l| (l.label.clone(), l.url.clone()))
                .collect(),
        };
        if sender.send_message(&msg).await.is_ok() {
            sent += 1;
        } else {
            tracing::warn!(chat = a.chat_id, "bot reply send failed");
        }
    }
    sent
}

/// `POST /bot/webhook` — Telegram update → pure orchestrator → response.
///
/// # Errors
/// 503 `BOT_DISABLED`; 401 `BOT_UNAUTHORIZED`; 400 invalid JSON body.
pub async fn webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<Value>> {
    require_webhook_secret(&headers, state.config.bot_webhook_secret.as_deref())?;
    let Some(runtime) = state.bot.as_ref() else {
        // secret is set but the runtime was not built (bot token missing) — fail-closed
        return Err(kian_contracts::HttpFailure::bot_disabled().into());
    };
    let raw: Value = serde_json::from_slice(&body)
        .map_err(|_| kian_contracts::HttpFailure::invalid_request(1))?;

    let Some(update) = BotUpdate::from_json(&raw) else {
        // non-message/non-text update — Telegram gets a 200, no reply is sent
        tracing::debug!("webhook update ignored (no private text message)");
        return Ok(Json(serde_json::json!({ "ok": true, "handled": false })));
    };

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
    let actions = {
        let mut core = runtime.core.lock().await;
        core.handle(&update, now_ms, &runtime.port).await
    };
    let sent = execute_actions(runtime.sender.as_ref(), &actions).await;
    tracing::info!(update = update.update_id, sent, "webhook update handled");
    Ok(Json(serde_json::json!({ "ok": true, "handled": true, "sent": sent })))
}

/// Messages to [`RecordedMessage`] for tests — one message per [`BotAction`].
#[must_use]
pub fn recorded_of(action: &BotAction) -> RecordedMessage {
    RecordedMessage {
        chat_id: action.chat_id,
        text: action.text.clone(),
        links: action
            .links
            .iter()
            .map(|LinkButton { label, url }| (label.clone(), url.clone()))
            .collect(),
    }
}

// ── Tests — full webhook over the real router with a recording sender ────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::build_router;
    use crate::state::AppState;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use kian_db::V3Store;
    use tower::ServiceExt;

    const SECRET: &str = "test-webhook-secret-0123456789";
    const BOT: &str = "123456:TEST-BOT-TOKEN-M6";

    /// App state with the bot runtime + a recording sender (manual sender injection).
    async fn state_with_bot() -> (Arc<AppState>, RecordingSender) {
        let mut config = crate::config::ServerConfig::base();
        config.bot_token = Some(BOT.to_owned());
        config.bot_webhook_secret = Some(SECRET.to_owned());
        config.miniapp_url = "http://localhost:3000".to_owned();
        let store = V3Store::connect_in_memory().await.expect("store");
        store.migrate().await.expect("migrate");
        store.ensure_system_accounts().await.expect("system");
        // runtime with a recording sender — production stand-in for offline runs
        let sender = RecordingSender::default();
        let runtime = BotRuntime {
            core: tokio::sync::Mutex::new(BotCore::new(BotPolicy {
                miniapp_url: "http://localhost:3000".to_owned(),
                ..BotPolicy::default()
            })),
            sender: Arc::new(sender.clone()),
            port: InProcessPort::new(store.clone(), BOT, config.session_ttl_hours),
        };
        let state = AppState::with_parts(
            store,
            config,
            Arc::new(kian_ai::client::UnavailableClient),
            Some(runtime),
        );
        (state.shared(), sender)
    }

    fn update_json(text: &str) -> Value {
        serde_json::json!({
            "update_id": 1,
            "message": {
                "chat": { "id": 100, "type": "private" },
                "from": { "id": 777, "username": "kia", "is_bot": false },
                "text": text
            }
        })
    }

    async fn post_webhook(
        state: &Arc<AppState>,
        secret: Option<&str>,
        body: Value,
    ) -> axum::response::Response {
        let mut req = Request::builder()
            .method("POST")
            .uri("/api/v1/bot/webhook")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .expect("request");
        if let Some(s) = secret {
            req.headers_mut().insert(WEBHOOK_SECRET_HEADER, s.parse().expect("header"));
        }
        build_router(state.clone()).oneshot(req).await.expect("response")
    }

    #[tokio::test]
    async fn webhook_guard_is_fail_closed() {
        // 1) server without the secret → 503 BOT_DISABLED (never open)
        let config = crate::config::ServerConfig::base();
        let store = V3Store::connect_in_memory().await.expect("store");
        store.migrate().await.expect("migrate");
        store.ensure_system_accounts().await.expect("system");
        let state = AppState::new(store, config).shared();
        let res = post_webhook(&state, Some(SECRET), update_json("/start")).await;
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);

        // 2) secret set, header missing → 401
        let (state2, _) = state_with_bot().await;
        let res2 = post_webhook(&state2, None, update_json("/start")).await;
        assert_eq!(res2.status(), StatusCode::UNAUTHORIZED);

        // 3) wrong header → 401
        let res3 = post_webhook(&state2, Some("wrong-secret-0123456789"), update_json("/start")).await;
        assert_eq!(res3.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn webhook_full_balance_flow_is_offline_honest() {
        let (state, sender) = state_with_bot().await;
        let res = post_webhook(&state, Some(SECRET), update_json("/balance")).await;
        assert_eq!(res.status(), StatusCode::OK);
        let sent = sender.sent.lock().unwrap().clone();
        assert_eq!(sent.len(), 1, "one reply for one command");
        assert_eq!(sent[0].chat_id, 100);
        assert!(sent[0].text.contains("Your balance"));
        assert!(sent[0].text.contains('0'), "in-memory store starts at zero — honest");
        // the token never appears in a response
        assert!(!sent[0].text.contains(BOT));
        // identity was built via the official path — user citizen:777 now exists
        let user = state
            .store
            .ensure_telegram_user("777", Some("kia"))
            .await
            .expect("user exists");
        assert_eq!(user.account_id, "citizen:777");
    }

    #[tokio::test]
    async fn webhook_start_carries_miniapp_button() {
        let (state, sender) = state_with_bot().await;
        let res = post_webhook(&state, Some(SECRET), update_json("/start")).await;
        assert_eq!(res.status(), StatusCode::OK);
        let sent = sender.sent.lock().unwrap().clone();
        assert_eq!(sent.len(), 1);
        assert_eq!(
            sent[0].links.first().map(|(label, _)| label.as_str()),
            Some("🚀 Open the MAIRON mini-app")
        );
        assert_eq!(
            sent[0].links.first().map(|(_, url)| url.as_str()),
            Some("http://localhost:3000")
        );
        // second button: direct market shortcut
        assert_eq!(
            sent[0].links.get(1).map(|(label, url)| (label.as_str(), url.as_str())),
            Some(("🛒 MAIRON Market", "http://localhost:3000/market"))
        );
    }

    #[tokio::test]
    async fn webhook_rate_limit_stops_amplification() {
        let (state, sender) = state_with_bot().await;
        // 25 rapid messages — default cap of 20 per window
        for i in 0..25u32 {
            let mut body = update_json("/id");
            body["message"]["text"] = serde_json::json!(format!("/id {i}"));
            body["update_id"] = serde_json::json!(i);
            let res = post_webhook(&state, Some(SECRET), body).await;
            assert_eq!(res.status(), StatusCode::OK);
        }
        let sent = sender.sent.lock().unwrap().len();
        // 20 allowed replies + one rate warning = 21 — no more (no spam amplification)
        assert_eq!(sent, 21, "warn once, then silent");
    }

    #[tokio::test]
    async fn webhook_garbage_json_is_400_and_unknown_update_is_200() {
        let (state, sender) = state_with_bot().await;
        // malformed body → 400 (parse contract at the edge)
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/bot/webhook")
            .header("content-type", "application/json")
            .header(WEBHOOK_SECRET_HEADER, SECRET)
            .body(Body::from("{not-json"))
            .expect("request");
        let res = build_router(state.clone()).oneshot(req).await.expect("response");
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        // non-message update (photo) → 200 handled:false, no message is sent
        let photo = serde_json::json!({
            "update_id": 9,
            "message": { "chat": { "id": 100 }, "from": { "id": 777 }, "photo": [{}] }
        });
        let res2 = post_webhook(&state, Some(SECRET), photo).await;
        assert_eq!(res2.status(), StatusCode::OK);
        assert_eq!(sender.sent.lock().unwrap().len(), 0);
    }
}
