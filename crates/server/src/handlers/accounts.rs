//! Account handlers — citizen registration, the admin list, and public balance.

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

/// Authorizing access to one account's info: admin only (`X-KIAN-Key`) or the account owner themselves (Bearer session).
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

/// System prefixes — no account may take these (supply protection, article 4).
const RESERVED_PREFIXES: [&str; 4] = ["mint:", "treasury:", "burn:", "escrow:"];

/// `POST /accounts` — register a user account (balance starts at 0). Admin.
///
/// # Errors
/// 422 `RESERVED_ID` / `DUPLICATE_ACCOUNT`; 400 invalid body.
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

    // honest row read — the database's real createdAt, not the process's guess
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

/// `GET /accounts` — list all accounts with derived balances. Admin.
///
/// # Errors
/// SQL error.
pub async fn list_accounts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<AccountListResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let items = state.store.list_accounts_with_balances().await?.iter().map(account_view).collect();
    Ok(Json(AccountListResponse { items }))
}

/// `GET /accounts/{id}` — account details; admin or the account owner only.
///
/// # Errors
/// 401 without a session/admin key; 403 accessing someone else's account; 404 `UNKNOWN_ACCOUNT`.
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

/// `GET /accounts/{id}/balance` — derived balance; admin or the account owner only.
///
/// # Errors
/// 401 without a session/admin key; 403 accessing someone else's balance; 404 `UNKNOWN_ACCOUNT`.
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

        // without a token → 401
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

        // another citizen (Bob) on Alice's account → 403
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

        // the account owner themselves (Alice) → 200
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

        // admin with X-KIAN-Key → 200
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
