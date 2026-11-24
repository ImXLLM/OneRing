//! هندلرهای حساب‌ها — ثبت‌نام شهروند، فهرست ادمین و موجودی عمومی.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::sync::Arc;

use kian_contracts::{
    AccountListResponse, AccountView, BalanceResponse, CreateAccountRequest,
};
use kian_domain::{DomainError, DomainErrorCode};

use crate::error::{ApiFailure, ApiResult};
use crate::middleware::{require_admin, require_session};
use crate::state::AppState;
use crate::views::{account_view, wire_din};

/// احراز دسترسی به اطلاعات یک حساب: فقط ادمین (`X-KIAN-Key`) یا خودِ صاحب حساب (نشست Bearer).
async fn authorize_admin_or_owner(
    state: &AppState,
    headers: &HeaderMap,
    account_id: &str,
) -> ApiResult<()> {
    if require_admin(headers, state.config.admin_key.as_deref()).is_ok() {
        return Ok(());
    }
    let user = require_session(state, headers).await?;
    if user.account_id == account_id {
        return Ok(());
    }
    Err(ApiFailure::Http(kian_contracts::HttpFailure::forbidden_escrow_role()))
}

/// پیشوندهای سیستمی — هیچ حسابی این‌ها را نمی‌گیرد (حفاظت عرضه، مادهٔ ۴).
const RESERVED_PREFIXES: [&str; 4] = ["mint:", "treasury:", "burn:", "escrow:"];

/// `POST /accounts` — ثبت حساب کاربر (موجودی از ۰ شروع می‌شود). ادمین.
///
/// # Errors
/// ۴۲۲ `RESERVED_ID` / `DUPLICATE_ACCOUNT`؛ ۴۰۰ بدنهٔ نامعتبر.
pub async fn create_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<AccountView>)> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let req: CreateAccountRequest = crate::handlers::parse_body(&body)?;
    let id = req.id.get();

    if RESERVED_PREFIXES.iter().any(|p| id.starts_with(p)) {
        return Err(kian_contracts::HttpFailure::reserved_id().into());
    }
    if state.store.balance_of(id).await?.is_some() {
        return Err(kian_contracts::HttpFailure::duplicate_account(id).into());
    }
    state
        .store
        .create_account_if_missing(id, kian_domain::AccountKind::User, req.label.as_deref())
        .await?;

    // خواندن صادقانهٔ ردیف — createdAt واقعی دیتابیس، نه حدس پروسه
    let account = state
        .store
        .list_accounts_with_balances()
        .await?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(DomainErrorCode::UnknownAccount, format!("Account {id} vanished"))
                .into()
        })?;
    Ok((StatusCode::CREATED, Json(account_view(&account))))
}

/// `GET /accounts` — فهرست همهٔ حساب‌ها با تراز مشتق‌شده. ادمین.
///
/// # Errors
/// خطای SQL.
pub async fn list_accounts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<AccountListResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let items = state.store.list_accounts_with_balances().await?.iter().map(account_view).collect();
    Ok(Json(AccountListResponse { items }))
}

/// `GET /accounts/{id}` — جزئیات حساب؛ فقط ادمین یا خودِ صاحب حساب.
///
/// # Errors
/// ۴۰۱ بدون نشست/کلید ادمین؛ ۴۰۳ دسترسی به حساب دیگران؛ ۴۰۴ `UNKNOWN_ACCOUNT`.
pub async fn get_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> ApiResult<Json<AccountView>> {
    authorize_admin_or_owner(&state, &headers, &id).await?;
    let account = state
        .store
        .list_accounts_with_balances()
        .await?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(DomainErrorCode::UnknownAccount, format!("Account {id} does not exist"))
                .into()
        })?;
    Ok(Json(account_view(&account)))
}

/// `GET /accounts/{id}/balance` — موجودی مشتق‌شده؛ فقط ادمین یا خودِ صاحب حساب.
///
/// # Errors
/// ۴۰۱ بدون نشست/کلید ادمین؛ ۴۰۳ دسترسی به موجودی دیگران؛ ۴۰۴ `UNKNOWN_ACCOUNT`.
pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> ApiResult<Json<BalanceResponse>> {
    authorize_admin_or_owner(&state, &headers, &id).await?;
    let balance = state.store.balance_of(&id).await?.ok_or_else::<ApiFailure, _>(|| {
        DomainError::new(DomainErrorCode::UnknownAccount, format!("Account {id} does not exist"))
            .into()
    })?;
    Ok(Json(BalanceResponse { account_id: id, balance: wire_din(balance) }))
}

#[cfg(test)]
mod privacy_tests {
    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_contracts::make_test_init_data;
    use kian_db::V3Store;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    const BOT: &str = "123456:TEST-BOT-TOKEN-PRIV";
    const ADMIN: &str = "test-admin-key-0123456789";

    async fn app() -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some(ADMIN.to_owned());
        cfg.bot_token = Some(BOT.to_owned());
        crate::routes::build_router(crate::state::AppState::new(store, cfg).shared())
    }

    async fn json_body(res: axum::response::Response) -> Value {
        let b = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&b).unwrap()
    }

    async fn auth(app: &axum::Router, tg_id: &str, username: &str) -> String {
        let init = make_test_init_data(BOT, tg_id, Some(username));
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(json!({ "initData": init }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        json_body(res).await["token"].as_str().unwrap().to_owned()
    }

    #[tokio::test]
    async fn account_balance_and_details_enforce_owner_or_admin_privacy() {
        let app = app().await;
        let alice = auth(&app, "910", "alice_p").await;
        let bob = auth(&app, "911", "bob_p").await;

        // بدون توکن → ۴۰۱
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/accounts/citizen:910/balance")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);

        // شهروند دیگر (Bob) روی حساب Alice → ۴۰۳
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/accounts/citizen:910/balance")
                    .header("authorization", format!("Bearer {bob}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 403);

        // خودِ صاحب حساب (Alice) → ۲۰۰
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/accounts/citizen:910")
                    .header("authorization", format!("Bearer {alice}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(json_body(res).await["id"], "citizen:910");

        // ادمین با X-KIAN-Key → ۲۰۰
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/accounts/citizen:910/balance")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
    }
}
