//! هندلرهای هویت M3 — احراز `initData` تلگرام، «من کی‌ام» و خروج.
//!
//! جریان: `Telegram.WebApp.initData` خام → تأیید HMAC رسمی (`contracts::security`)
//! → `ensure_telegram_user` (هویت idempotent + حساب دفتر `citizen:{tg}`) →
//! توکن نشست اپاک با TTL. هویت **هرگز** از فیلدهای بدنه نمی‌آید؛ بدنه فقط
//! رشتهٔ خام initData را حمل می‌کند. ولت بدون initData فایده‌ای ندارد —
//! fail-closed مادهٔ ۱ منشور.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::sync::Arc;

use kian_contracts::{
    AuthResponse, MeResponse, TelegramAuthRequest, UserView, WalletTxView,
};
use kian_db::UserRow;

use crate::error::{ApiFailure, ApiResult};
use crate::middleware::{bearer_token, require_session};
use crate::state::AppState;
use crate::views::wire_din;

/// نمای کاربر وایر از ردیف db — نقطهٔ واحد ساخت `UserView` (هندلرها JSON مستقیم نمی‌سازند).
#[must_use]
pub fn user_view(u: &UserRow) -> UserView {
    UserView {
        id: u.id.clone(),
        telegram_id: u.telegram_id.clone(),
        username: u.username.clone(),
        account_id: u.account_id.clone(),
        ton_address: u.ton_address.clone(),
        created_at: u.created_at.clone(),
    }
}

/// توکن نشست اپاک — دو UUIDv4 رمزنگاری‌شده (CSPRNG سیستم‌عامل) پشت هم = ۶۴ hex.
pub(crate) fn new_session_token() -> String {
    let a = uuid::Uuid::new_v4().simple().to_string();
    let b = uuid::Uuid::new_v4().simple().to_string();
    format!("{a}{b}")
}

/// انقضای نشست — TTL ساعت از پیکربندی، RFC3339 UTC با millis.
pub(crate) fn session_expires_at(ttl_hours: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::hours(ttl_hours))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// `POST /auth/telegram` — احراز مینی‌اپ و باز کردن نشست. ۲۰۱.
///
/// # Errors
/// 503 `IDENTITY_DISABLED` (بدون `KIAN_BOT_TOKEN` — fail-closed)؛
/// 401 `INVALID_INIT_DATA` (امضای HMAC معتبر نیست)؛ 400 بدنهٔ نامعتبر.
pub async fn auth_telegram(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    let Some(bot_token) = state.config.bot_token.clone() else {
        return Err(ApiFailure::Http(kian_contracts::HttpFailure::identity_disabled()));
    };
    let req: TelegramAuthRequest = crate::handlers::parse_body(&body)?;
    let Some(identity) = kian_contracts::verify_telegram_init_data(&req.init_data, &bot_token) else {
        tracing::warn!("telegram initData failed HMAC verification");
        return Err(ApiFailure::Http(kian_contracts::HttpFailure::invalid_init_data()));
    };

    let user = state
        .store
        .ensure_telegram_user(&identity.user_id, identity.username.as_deref())
        .await
        .map_err(ApiFailure::from)?;
    let token = new_session_token();
    let expires_at = session_expires_at(state.config.session_ttl_hours);
    state.store.create_session(&token, &user.id, &expires_at).await.map_err(ApiFailure::from)?;
    tracing::info!(user = %user.id, "telegram identity authenticated");
    Ok((StatusCode::CREATED, Json(AuthResponse { token, expires_at, user: user_view(&user) })))
}

/// `GET /me` — هویت + تراز مشتق‌شده + قفل اسکرو + ۱۰ ردیف آخر دفتر.
///
/// # Errors
/// 401 نشست غایب/منقضی؛ 500 خطای SQL.
pub async fn me(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResult<Json<MeResponse>> {
    let user = require_session(&state, &headers).await?;
    let balance = state
        .store
        .balance_of(&user.account_id)
        .await
        .map_err(ApiFailure::from)?
        .unwrap_or(0);
    let locked = state
        .store
        .locked_for_buyer(&user.account_id)
        .await
        .map_err(ApiFailure::from)?;
    let recent = state
        .store
        .recent_postings_for(&user.account_id, 10)
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(MeResponse {
        user: user_view(&user),
        balance: wire_din(balance),
        escrow_locked: wire_din(locked),
        recent_txs: recent
            .iter()
            .map(|p| WalletTxView {
                tx_id: p.tx_id.clone(),
                amount: p.amount,
                memo: p.memo.clone(),
                at: p.at.clone(),
            })
            .collect(),
    }))
}

/// `POST /auth/logout` — حذف نشست جاری. ۲۰۴ — دوباره صدا زدن بی‌اثر.
///
/// # Errors
/// 401 `SESSION_REQUIRED` بدون هدر؛ 500 خطای SQL.
pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let Some(token) = bearer_token(&headers) else {
        return Err(ApiFailure::Http(kian_contracts::HttpFailure::session_required()));
    };
    state.store.delete_session(&token).await.map_err(ApiFailure::from)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /me/ton-address` — ثبت/به‌روزرسانی آدرس ولت TON Connect روی هویت.
///
/// فقط مالک نشست — آدرس اعتبارسنجی سبک (شکل UQ/EQ پایهٔ ۴۸ کاراکتری یا فرندلی).
///
/// # Errors
/// 401 نشست غایب/منقضی؛ 400 بدنهٔ نامعتبر؛ 422 آدرس بدشکل؛ 500 خطای SQL.
pub async fn set_ton_address(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<UserView>> {
    let user = require_session(&state, &headers).await?;
    let req: kian_contracts::SetTonAddressRequest = crate::handlers::parse_body(&body)?;
    let addr = req.address.trim();
    let shape_ok = (48..=64).contains(&addr.len())
        && addr.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && (addr.starts_with("UQ") || addr.starts_with("EQ"));
    if !shape_ok {
        return Err(ApiFailure::Http(kian_contracts::HttpFailure::invalid_ton_address()));
    }
    state
        .store
        .set_user_ton_address(&user.id, addr)
        .await
        .map_err(ApiFailure::from)?;
    let updated = state
        .store
        .get_user(&user.id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else(|| ApiFailure::Http(kian_contracts::HttpFailure::session_expired()))?;
    Ok(Json(user_view(&updated)))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_contracts::make_test_init_data;
    use kian_db::V3Store;
    use tower::ServiceExt;

    const BOT: &str = "123456:TEST-BOT-TOKEN-M3";
    const ADMIN: &str = "test-admin-key-0123456789";

    fn test_config(bot: Option<String>) -> crate::config::ServerConfig {
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some(ADMIN.to_owned());
        cfg.bot_token = bot;
        cfg
    }

    async fn app(bot: Option<String>) -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        crate::routes::build_router(crate::state::AppState::new(store, test_config(bot)).shared())
    }

    async fn json_body(res: axum::response::Response) -> serde_json::Value {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// احراز سریع برای تست‌های نشست‌دار — توکن از پاسخ ۲۰۱.
    async fn auth_token(app: &axum::Router, init_data: &str) -> String {
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"initData":"{init_data}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "helper: auth must succeed");
        let body = json_body(res).await;
        body["token"].as_str().unwrap().to_owned()
    }

    #[test]
    fn session_token_is_64_hex_csprng_without_shared_timestamp_prefix() {
        let t1 = super::new_session_token();
        let t2 = super::new_session_token();
        assert_eq!(t1.len(), 64);
        assert_eq!(t2.len(), 64);
        assert!(t1.bytes().all(|b| b.is_ascii_hexdigit()));
        assert!(t2.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_ne!(t1, t2);
        // در UUIDv7 پیشوند ۸ کاراکتر اول timestamp مشترک داشت؛ در UUIDv4 تصادفی است
        assert_ne!(&t1[..8], &t2[..8]);
    }

    #[tokio::test]
    async fn auth_me_logout_roundtrip() {
        let app = app(Some(BOT.into())).await;
        let init_data = make_test_init_data(BOT, "12345", Some("ali"));

        // احراز — ۲۰۱ با توکن
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(format!(r#"{{"initData":"{init_data}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "auth must succeed");
        let body = json_body(res).await;
        let token = body["token"].as_str().unwrap().to_owned();
        assert_eq!(body["user"]["username"], "ali");
        assert_eq!(body["user"]["accountId"], "citizen:12345");

        // /me — تراز صفر، حساب شهروند
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let me = json_body(res).await;
        assert_eq!(me["balance"], 0);
        assert_eq!(me["user"]["telegramId"], "12345");
        assert_eq!(me["escrowLocked"], 0);

        // خروج — ۲۰۴ و نشست می‌میرد
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/logout")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 204);
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401, "logged-out token must be dead");
    }

    #[tokio::test]
    async fn tampered_init_data_is_rejected() {
        let app = app(Some(BOT.into())).await;
        let init_data = make_test_init_data("OTHER-BOT-TOKEN", "12345", None); // امضای بات دیگر
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(format!(r#"{{"initData":"{init_data}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
        let body = json_body(res).await;
        assert_eq!(body["error"]["code"], "INVALID_INIT_DATA");
    }

    #[tokio::test]
    async fn identity_is_fail_closed_without_bot_token() {
        let app = app(None).await;
        let init_data = make_test_init_data(BOT, "12345", None);
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(format!(r#"{{"initData":"{init_data}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 503);
        let body = json_body(res).await;
        assert_eq!(body["error"]["code"], "IDENTITY_DISABLED");
    }

    #[tokio::test]
    async fn me_requires_bearer_header() {
        let app = app(Some(BOT.into())).await;
        let res = app
            .oneshot(
                axum::http::Request::builder().method("GET").uri("/api/v1/me").body(Body::empty()).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
        let body = json_body(res).await;
        assert_eq!(body["error"]["code"], "SESSION_REQUIRED");
    }

    /// ثبت آدرس TON روی هویت — فقط مالک نشست؛ آدرس بدشکل ۴۲۲.
    #[tokio::test]
    async fn set_ton_address_roundtrip_and_validation() {
        let app = app(Some(BOT.into())).await;
        let init_data = make_test_init_data(BOT, "55", None);
        let token = auth_token(&app, &init_data).await;

        let good = "UQAbcd1234efgh5678ijkl9012mnop3456qrst7890uvwx1234";
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/me/ton-address")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"address":"{good}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["tonAddress"], good);

        // آدرس بدشکل — ۴۲۲
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/me/ton-address")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"address":"not-ton"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);

        // بدون نشست — ۴۰۱
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/me/ton-address")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"address":"not-ton"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }

    /// هویت از بدنه نمی‌آید — فیلد userId در بدنه نادیده گرفته می‌شود.
    #[tokio::test]
    async fn identity_is_never_taken_from_body() {
        let app = app(Some(BOT.into())).await;
        // initData امضاشده برای کاربر ۹۹، اما بدنه سعی دارد کاربر ۱ شود
        let init_data = make_test_init_data(BOT, "99", None);
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(format!(r#"{{"initData":"{init_data}","userId":"1"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let body = json_body(res).await;
        assert_eq!(body["user"]["telegramId"], "99", "identity comes from initData only");
        assert_eq!(body["user"]["accountId"], "citizen:99");
    }
}
