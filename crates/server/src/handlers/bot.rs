//! وبهوک بات تلگرام (M6) — `POST /bot/webhook` — شِل دوم روی orchestrator خالص.
//!
//! گاردها (همه fail-closed، قرارداد فاز ۵ که در `contracts::error` پیش‌بینی
//! شده بود):
//! * سرور بدون `KIAN_BOT_WEBHOOK_SECRET` → 503 `BOT_DISABLED` — هرگز باز.
//! * هدر `X-Telegram-Bot-Api-Secret-Token` غایب/نادرست → 401 `BOT_UNAUTHORIZED`
//!   (مقایسهٔ زمان‌ثابت با [`verify_kian_key`]).
//! * آپدیت نامعتبر → 200 با `handled:false` — تلگرام باید ۲۰۰ بگیرد؛ خطا برای
//!   تلگرام معنا ندارد، فقط لاگ می‌شود.
//!
//! هویت: آپدیت مستقیم از کانال معتمد تلگرام (وبهوک امضاشده) می‌آید؛ برای دادهٔ
//! شهروند، [`InProcessPort`] همان مسیر رسمی مینی‌اپ را طی می‌کند — initData
//! با `build_bot_init_data` ضرب و با `verify_telegram_init_data` تأیید می‌شود؛
//! **هیچ مسیر جانبی برای هویت و پول وجود ندارد.**
//!
//! پاسخ‌ها از [`TelegramSender`] می‌روند — در تست‌ها `RecordingSender`
//! (بدون شبکه)، در تولید `TelegramHttpSender` روی Bot API رسمی.

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

/// نام هدر راز وبهوک — رسمی تلگرام.
pub const WEBHOOK_SECRET_HEADER: &str = "x-telegram-bot-api-secret-token";

// ── پورت ارسال پاسخ به تلگرام ───────────────────────────────────────────────

/// پیام ثبت‌شده برای تست — بدون شبکه.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedMessage {
    /// چت مقصد.
    pub chat_id: i64,
    /// متن پاسخ.
    pub text: String,
    /// دکمه‌های لینک (label, url) — ردیف شیشه‌ای چنددکمه‌ای.
    pub links: Vec<(String, String)>,
}

/// پورت ارسال — تنها خروجی به تلگرام؛ تست‌ها ضبط می‌کنند، تولید HTTP می‌زند.
#[async_trait]
pub trait TelegramSender: Send + Sync {
    /// ارسال یک پیام — شکست فقط لاگ می‌شود (بات نمی‌میرد).
    async fn send_message(&self, msg: &RecordedMessage) -> Result<(), String>;
}

/// پیاده‌سازی تولید — Bot API رسمی با rustls؛ توکن هرگز در لاگ نمی‌رود.
pub struct TelegramHttpSender {
    http: reqwest::Client,
    bot_token: String,
}

impl TelegramHttpSender {
    /// ساخت با توکن بات.
    ///
    /// # Errors
    /// ساخت کلاینت TLS.
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

/// ضبط‌کنندهٔ تست — صفر شبکه، صفر راز.
#[derive(Default, Clone)]
pub struct RecordingSender {
    /// پیام‌های ارسال‌شده.
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

// ── پورت کرنل درون‌پردازشی ──────────────────────────────────────────────────

/// پورت کرنل بدون HTTP loopback — همان دیتای store، همان مسیر هویت.
pub struct InProcessPort {
    /// فروشگاه v3 — همان نقطهٔ اتصال هندلرها.
    store: kian_db::V3Store,
    /// توکن بات برای ضرب initData (هرگز لاگ نمی‌شود).
    bot_token: String,
    /// TTL نشست ساعت.
    session_ttl_hours: i64,
}

impl InProcessPort {
    /// ساخت از پیکربندی + store (بدون چرخهٔ Arc — store کلون می‌شود).
    #[must_use]
    pub fn new(store: kian_db::V3Store, bot_token: &str, session_ttl_hours: i64) -> Self {
        Self { store, bot_token: bot_token.to_owned(), session_ttl_hours }
    }

    /// نشست شهروند از مسیر رسمی هویت: ضرب → تأیید رسمی → ensure → session.
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

// ── ران‌تایم بات روی AppState ───────────────────────────────────────────────

/// ران‌تایم وبهوک — هستهٔ خالص + دو پورت؛ فقط وقتی هر دو راز تنظیم شده باشند.
pub struct BotRuntime {
    /// هستهٔ خالص — سقف نرخ per-chat (قفل async — handle درون قفل await می‌شود).
    pub core: tokio::sync::Mutex<BotCore>,
    /// پورت ارسال به تلگرام.
    pub sender: Arc<dyn TelegramSender>,
    /// پورت کرنل درون‌پردازشی.
    pub port: InProcessPort,
}

impl BotRuntime {
    /// ساخت تولیدی — sender واقعی Bot API.
    ///
    /// # Errors
    /// ساخت کلاینت TLS.
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

// ── هندلر وبهوک ──────────────────────────────────────────────────────────────

/// گوار راز وبهوک — سه‌وضعیتی fail-closed مثل ادمین.
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

/// اجرای اکشن‌ها روی sender — شکست هر پیام لاگ می‌شود و ادامه.
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

/// `POST /bot/webhook` — آپدیت تلگرام → orchestrator خالص → پاسخ.
///
/// # Errors
/// 503 `BOT_DISABLED`؛ 401 `BOT_UNAUTHORIZED`؛ 400 بدنهٔ نامعتبر JSON.
pub async fn webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<Value>> {
    require_webhook_secret(&headers, state.config.bot_webhook_secret.as_deref())?;
    let Some(runtime) = state.bot.as_ref() else {
        // راز تنظیم است اما ران‌تایم ساخته نشده (توکن بات غایب) — fail-closed
        return Err(kian_contracts::HttpFailure::bot_disabled().into());
    };
    let raw: Value = serde_json::from_slice(&body)
        .map_err(|_| kian_contracts::HttpFailure::invalid_request(1))?;

    let Some(update) = BotUpdate::from_json(&raw) else {
        // آپدیت غیرپیام/غیرمتنی — به تلگرام ۲۰۰ می‌دهیم، هیچ پاسخی نمی‌رود
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

/// پیام به [`RecordedMessage`] برای تست‌ها — هر [`BotAction`] یک پیام.
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

// ── تست‌ها — وبهوک کامل روی روتر واقعی با sender ضبط‌شده ────────────────────

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

    /// وضعیت اپ با ران‌تایم بات + sender ضبط‌شده (تزریق دستی sender).
    async fn state_with_bot() -> (Arc<AppState>, RecordingSender) {
        let mut config = crate::config::ServerConfig::base();
        config.bot_token = Some(BOT.to_owned());
        config.bot_webhook_secret = Some(SECRET.to_owned());
        config.miniapp_url = "http://localhost:3000".to_owned();
        let store = V3Store::connect_in_memory().await.expect("store");
        store.migrate().await.expect("migrate");
        store.ensure_system_accounts().await.expect("system");
        // ران‌تایم با sender ضبط‌شده — جایگزین تولیدی برای آفلاین‌بودن
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
        // ۱) سرور بدون راز → 503 BOT_DISABLED (هرگز باز)
        let config = crate::config::ServerConfig::base();
        let store = V3Store::connect_in_memory().await.expect("store");
        store.migrate().await.expect("migrate");
        store.ensure_system_accounts().await.expect("system");
        let state = AppState::new(store, config).shared();
        let res = post_webhook(&state, Some(SECRET), update_json("/start")).await;
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);

        // ۲) راز تنظیم شده، هدر غایب → 401
        let (state2, _) = state_with_bot().await;
        let res2 = post_webhook(&state2, None, update_json("/start")).await;
        assert_eq!(res2.status(), StatusCode::UNAUTHORIZED);

        // ۳) هدر نادرست → 401
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
        assert!(sent[0].text.contains("تراز شما"));
        assert!(sent[0].text.contains('0'), "in-memory store starts at zero — honest");
        // توکن هرگز در پاسخ نیست
        assert!(!sent[0].text.contains(BOT));
        // هویت از مسیر رسمی ساخته شده — کاربر citizen:777 اکنون وجود دارد
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
            Some("🚀 ورود به مینی‌اپ KIAN")
        );
        assert_eq!(
            sent[0].links.first().map(|(_, url)| url.as_str()),
            Some("http://localhost:3000")
        );
        // دکمهٔ دوم: میان‌بر مستقیم بازار
        assert_eq!(
            sent[0].links.get(1).map(|(label, url)| (label.as_str(), url.as_str())),
            Some(("🛒 بازار KIAN", "http://localhost:3000/market"))
        );
    }

    #[tokio::test]
    async fn webhook_rate_limit_stops_amplification() {
        let (state, sender) = state_with_bot().await;
        // ۲۵ پیام سریع — سقف پیش‌فرض ۲۰ در پنجره
        for i in 0..25u32 {
            let mut body = update_json("/id");
            body["message"]["text"] = serde_json::json!(format!("/id {i}"));
            body["update_id"] = serde_json::json!(i);
            let res = post_webhook(&state, Some(SECRET), body).await;
            assert_eq!(res.status(), StatusCode::OK);
        }
        let sent = sender.sent.lock().unwrap().len();
        // ۲۰ پاسخ مجاز + یک هشدار نرخ = ۲۱ — نه بیشتر (بدون تقویت اسپم)
        assert_eq!(sent, 21, "warn once, then silent");
    }

    #[tokio::test]
    async fn webhook_garbage_json_is_400_and_unknown_update_is_200() {
        let (state, sender) = state_with_bot().await;
        // بدنهٔ خراب → 400 (قرارداد parse در لبه)
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/bot/webhook")
            .header("content-type", "application/json")
            .header(WEBHOOK_SECRET_HEADER, SECRET)
            .body(Body::from("{not-json"))
            .expect("request");
        let res = build_router(state.clone()).oneshot(req).await.expect("response");
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        // آپدیت غیرپیامی (photo) → 200 handled:false، هیچ پیامی نمی‌رود
        let photo = serde_json::json!({
            "update_id": 9,
            "message": { "chat": { "id": 100 }, "from": { "id": 777 }, "photo": [{}] }
        });
        let res2 = post_webhook(&state, Some(SECRET), photo).await;
        assert_eq!(res2.status(), StatusCode::OK);
        assert_eq!(sender.sent.lock().unwrap().len(), 0);
    }
}
