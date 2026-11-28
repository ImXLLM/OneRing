//! TON deposit handlers (M3) — user proof registration and admin adjudication with the Article 3 mint.
//!
//! Honest money flow from chain to ledger: the user sends via TON Connect to the KIAN treasury,
//! submits the hash (`PENDING`), the admin verifies it on-chain and marks it
//! `CONFIRMED`, and the Article 3 minting path (annual cap + mint account)
//! credits Din to the citizen account at that very moment. Minting is **never**
//! automatic and admin-free — even if the frontend waits.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::sync::Arc;

use kian_contracts::{TonDepositCreateRequest, TonDepositListResponse, TonDepositView};
use kian_db::{now_iso, TonDepositRow};
use kian_domain::{DomainError, DomainErrorCode};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::{require_admin, require_session};
use crate::state::AppState;

/// Wire deposit view from a db row — the single construction point for `TonDepositView`.
#[must_use]
fn deposit_view(d: &TonDepositRow) -> TonDepositView {
    TonDepositView {
        id: d.id.clone(),
        tx_hash: d.tx_hash.clone(),
        wallet_address: d.wallet_address.clone(),
        amount_ton: d.amount_ton.clone(),
        status: d.status.clone(),
        created_at: d.created_at.clone(),
        confirmed_at: d.confirmed_at.clone(),
    }
}

/// TON amount string validation — decimal with at most 6 fractional digits and value > 0.
/// Output: value in microTON scale (1 TON = 1,000,000).
fn parse_micro_ton(amount_ton: &str) -> Result<i64, ApiFailure> {
    let invalid = || kian_contracts::HttpFailure::invalid_request(1);
    let (whole, frac) = amount_ton.split_once('.').map_or((amount_ton, ""), |(w, f)| (w, f));
    if whole.is_empty() && frac.is_empty() {
        return Err(invalid().into());
    }
    if whole.chars().any(|c| !c.is_ascii_digit()) || frac.chars().any(|c| !c.is_ascii_digit()) {
        return Err(invalid().into());
    }
    if frac.len() > 6 {
        return Err(invalid().into()); // TON has 9 fractional digits; our wire cuts at 6
    }
    let whole_val: u64 = if whole.is_empty() { 0 } else { whole.parse().map_err(|_| invalid())? };
    let mut micro: u128 = u128::from(whole_val) * 1_000_000;
    if !frac.is_empty() {
        let padded = format!("{frac:0<6}");
        micro += padded.parse::<u128>().map_err(|_| invalid())?;
    }
    if micro == 0 || micro > i64::MAX as u128 {
        return Err(invalid().into());
    }
    #[allow(clippy::cast_possible_truncation)] // checked above i64
    Ok(micro as i64)
}

/// Transaction hash and wallet address validation — TON chain length and allowed characters.
fn validate_deposit_fields(
    tx_hash: &str,
    wallet_address: &str,
) -> Result<(), ApiFailure> {
    let ok_hash = (8..=128).contains(&tx_hash.chars().count())
        && tx_hash.chars().all(|c| c.is_ascii_alphanumeric());
    let ok_addr = (8..=128).contains(&wallet_address.chars().count())
        && wallet_address.chars().all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '_');
    if ok_hash && ok_addr {
        Ok(())
    } else {
        Err(kian_contracts::HttpFailure::invalid_request(1).into())
    }
}

/// `POST /wallet/ton-deposits` — register a deposit as user proof. 201 `PENDING`.
///
/// # Errors
/// 401 session; 400 invalid body; 422 duplicate hash (`InvalidTransition` — the
/// same deposit never gets minted twice); 500 SQL error.
pub async fn create_deposit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<TonDepositView>)> {
    let user = require_session(&state, &headers).await?;
    let req: TonDepositCreateRequest = parse_body(&body)?;
    validate_deposit_fields(&req.tx_hash, &req.wallet_address)?;
    let _micro = parse_micro_ton(&req.amount_ton)?; // shape validation only — the mint rate is the admin’s business

    if state
        .store
        .get_ton_deposit_by_hash(&req.tx_hash)
        .await
        .map_err(ApiFailure::from)?
        .is_some()
    {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!("Deposit {} is already recorded", req.tx_hash),
        )
        .into());
    }

    let row = TonDepositRow {
        id: format!("dep:{}", uuid::Uuid::now_v7()),
        user_id: user.id.clone(),
        tx_hash: req.tx_hash,
        wallet_address: req.wallet_address,
        amount_ton: req.amount_ton,
        status: "PENDING".into(),
        created_at: now_iso(),
        confirmed_at: None,
        confirmed_by: None,
    };
    state.store.insert_ton_deposit(&row).await.map_err(ApiFailure::from)?;
    Ok((StatusCode::CREATED, Json(deposit_view(&row))))
}

/// `GET /wallet/ton-deposits` — my deposits (latest 50).
///
/// # Errors
/// 401 session; 500 SQL error.
pub async fn list_mine(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<TonDepositListResponse>> {
    let user = require_session(&state, &headers).await?;
    let items = state
        .store
        .list_ton_deposits_for(&user.id)
        .await
        .map_err(ApiFailure::from)?
        .iter()
        .map(deposit_view)
        .collect();
    Ok(Json(TonDepositListResponse { items }))
}

/// `POST /wallet/ton-deposits/{id}/confirm` — admin: verification + Article 3 mint.
///
/// Conversion rate from `KIAN_TON_TO_DIN` (Din per TON; default 1000 Đ) —
/// conversion in microTON integers, no floats on money.
///
/// # Errors
/// 404 unknown deposit; 422 already decided or below the mint minimum;
/// 422 `MINT_CAP_EXCEEDED` (Article 3); 500 SQL error.
pub async fn confirm(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<TonDepositView>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let deposit = state
        .store
        .get_ton_deposit(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::EscrowNotFound,
                format!("Ton deposit {id} does not exist"),
            )
            .into()
        })?;

    if deposit.status != "PENDING" {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!("Ton deposit {id} is not PENDING anymore"),
        )
        .into());
    }

    // Din amount = microTON × rate ÷ 1,000,000 — in i128 with checked_mul to prevent i64 overflow
    let micro = parse_micro_ton(&deposit.amount_ton)?;
    let rate = state.config.ton_to_din_rate;
    let din_128 = i128::from(micro)
        .checked_mul(i128::from(rate))
        .map(|v| v / 1_000_000)
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::AmountOutOfRange,
                format!("Deposit {id} conversion overflow"),
            )
            .into()
        })?;
    if din_128 < 1 {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!("Deposit {id} is below the minimum mintable amount"),
        )
        .into());
    }
    if din_128 > i128::from(kian_domain::MAX_DIN) {
        return Err(DomainError::new(
            DomainErrorCode::AmountOutOfRange,
            format!("Deposit {id} exceeds MAX_DIN"),
        )
        .into());
    }
    let din = i64::try_from(din_128).map_err(|_| -> ApiFailure {
        DomainError::new(
            DomainErrorCode::AmountOutOfRange,
            format!("Deposit {id} exceeds MAX_DIN"),
        )
        .into()
    })?;

    let user = state
        .store
        .get_user(&deposit.user_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::UnknownAccount,
                format!("User {} does not exist", deposit.user_id),
            )
            .into()
        })?;

    // Safe ordering: mint first with the idempotency key; only after the mint succeeds (or a successful
    // replay) does the deposit become CONFIRMED, so money never burns on a mint-cap error.
    let _mint = crate::handlers::wallet::mint_to_account(
        &state,
        &user.account_id,
        din,
        Some(format!("ton deposit {}", deposit.tx_hash)),
        Some(format!("ton-deposit:{id}")),
    )
    .await?;

    let updated = state
        .store
        .set_ton_deposit_status(&id, "CONFIRMED", "admin")
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(deposit_view(&updated)))
}

/// `POST /wallet/ton-deposits/{id}/reject` — admin: reject the proof (no mint).
///
/// # Errors
/// 404 unknown deposit; 422 already decided; 500 SQL error.
pub async fn reject(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<TonDepositView>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let updated = state
        .store
        .set_ton_deposit_status(&id, "REJECTED", "admin")
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(deposit_view(&updated)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_contracts::make_test_init_data;
    use kian_db::V3Store;
    use serde_json::json;
    use tower::ServiceExt;

    const BOT: &str = "123456:TEST-BOT-TOKEN-M3";
    const ADMIN: &str = "test-admin-key-0123456789";

    fn test_config() -> crate::config::ServerConfig {
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some(ADMIN.to_owned());
        cfg.bot_token = Some(BOT.into());
        cfg
    }

    async fn app() -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        crate::routes::build_router(crate::state::AppState::new(store, test_config()).shared())
    }

    async fn json_body(res: axum::response::Response) -> serde_json::Value {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn micro_ton_parsing_is_strict() {
        assert_eq!(parse_micro_ton("0.5").unwrap(), 500_000);
        assert_eq!(parse_micro_ton("2").unwrap(), 2_000_000);
        assert_eq!(parse_micro_ton("1.234567").unwrap(), 1_234_567);
        assert!(parse_micro_ton("0.1234567").is_err(), "more than 6 decimal places");
        assert!(parse_micro_ton("0").is_err(), "zero is meaningless");
        assert!(parse_micro_ton("-1").is_err(), "negative never");
        assert!(parse_micro_ton("1e3").is_err());
        assert!(parse_micro_ton("").is_err());
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)] // the full deposit→confirm→mint cycle — test readability beats breaking
    async fn deposit_confirm_mints_through_charter_path() {
        let app = app().await;
        let init_data = make_test_init_data(BOT, "7", Some("buyer"));
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(json!({ "initData": init_data }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let token = json_body(res).await["token"].as_str().unwrap().to_owned();

        // deposit registration
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/ton-deposits")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "txHash": "abcd1234abcd1234",
                            "walletAddress": "UQ12345678",
                            "amountTon": "0.5"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let body = json_body(res).await;
        assert_eq!(body["status"], "PENDING");
        assert_eq!(body["confirmedAt"], serde_json::Value::Null);
        let dep_id = body["id"].as_str().unwrap().to_owned();

        // no admin — 401
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/ton-deposits/{dep_id}/confirm"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);

        // admin confirms — default rate 1000 Đ/TON → 500 Đ
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/ton-deposits/{dep_id}/confirm"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "confirm must succeed");
        let body = json_body(res).await;
        assert_eq!(body["status"], "CONFIRMED");

        // citizen balance must become 500
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let me = json_body(res).await;
        assert_eq!(me["balance"], 500, "0.5 TON × 1000 Đ/TON");

        // duplicate hash — 422
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/ton-deposits")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "txHash": "abcd1234abcd1234",
                            "walletAddress": "UQ12345678",
                            "amountTon": "0.5"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422, "same chain hash must never mint twice");

        // deciding again — 422 (one-way transition)
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/ton-deposits/{dep_id}/reject"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
    }

    #[tokio::test]
    async fn deposit_remains_pending_if_mint_cap_fails() {
        let app = app().await;
        let init_data = make_test_init_data(BOT, "8", Some("whale"));
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(json!({ "initData": init_data }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let token = json_body(res).await["token"].as_str().unwrap().to_owned();

        // 2000 TON × 1000 Đ/TON = 2,000,000 Đ; to exceed the 500 billion Đ annual cap:
        // 600,000,000 TON × 1000 = 600 billion Đ > cap
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/ton-deposits")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "txHash": "capfail12345678",
                            "walletAddress": "UQ12345678",
                            "amountTon": "600000000"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let dep_id = json_body(res).await["id"].as_str().unwrap().to_owned();

        // confirm must be rejected with the mint-cap error
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/ton-deposits/{dep_id}/confirm"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
        assert_eq!(json_body(res).await["error"]["code"], "MINT_CAP_EXCEEDED");

        // deposit status must still be PENDING (not a burned CONFIRMED)
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/wallet/ton-deposits")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let list = json_body(res).await;
        assert_eq!(list["items"][0]["status"], "PENDING");
    }
}
