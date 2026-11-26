//! M5 four-chain wallet handlers — chain metadata, user deposit addresses and
//! USDT deposits with the official chain checksums.
//!
//! Gates:
//!  * Metadata and treasury — public (Article 6 transparency; the treasury address comes from env).
//!  * User address/deposit — session; addresses are validated with `kian-wallet`
//!    (Tron `Base58Check`, 32-byte Solana, TON `CRC16-XMODEM`, BNB Chain `EIP-55`)
//!    — a wrong address is never registered.
//!  * Deposit confirm/reject — admin; confirmation goes through the Article 3 mint path (USDT rate from env)
//!    — minting is never automatic or admin-free (the same M3 contract).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use std::sync::Arc;

use kian_contracts::{
    ChainView, ChainsResponse, DepositAddressView, MyAddressesResponse, SetDepositAddressRequest,
    TreasuryChainView, TreasuryResponse, UsdtDepositCreateRequest, UsdtDepositListResponse,
    UsdtDepositView,
};
use kian_db::{UsdtDepositRow, WalletAddressRow, now_iso};
use kian_domain::{DomainError, DomainErrorCode};
use kian_wallet::{Chain, WalletError};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::{require_admin, require_session};
use crate::state::AppState;

/// Map wallet errors → honest HTTP envelope.
fn wallet_error(e: WalletError) -> ApiFailure {
    match e {
        WalletError::InvalidAddress { chain, detail } => {
            kian_contracts::HttpFailure::invalid_address(chain, detail).into()
        }
        WalletError::InvalidAmount(_) => kian_contracts::HttpFailure::invalid_request(1).into(),
        WalletError::CustodyViolation(detail) => {
            kian_contracts::HttpFailure::custody_violation(detail).into()
        }
    }
}

/// Parse the chain from the wire — an unknown string = 400 `INVALID_REQUEST`.
fn parse_chain(raw: &str) -> ApiResult<Chain> {
    Chain::from_str_raw(raw).ok_or_else(|| kian_contracts::HttpFailure::invalid_request(1).into())
}

/// Deposit view from the db row — the single construction point for `UsdtDepositView`.
#[must_use]
fn deposit_view(d: &UsdtDepositRow) -> UsdtDepositView {
    UsdtDepositView {
        id: d.id.clone(),
        chain: d.chain.clone(),
        tx_hash: d.tx_hash.clone(),
        from_address: d.from_address.clone(),
        amount_usdt: d.amount_usdt.clone(),
        status: d.status.clone(),
        created_at: d.created_at.clone(),
        confirmed_at: d.confirmed_at.clone(),
    }
}

/// `GET /wallet/chains` — metadata of the four chains + the canonical USDT contract. Public.
///
/// # Errors
/// None — always 200.
pub async fn chains_list() -> ApiResult<Json<ChainsResponse>> {
    let items = Chain::ALL
        .iter()
        .map(|c| ChainView {
            chain: c.as_str().to_owned(),
            fa_name: c.fa_name().to_owned(),
            usdt_contract: c.usdt_contract().to_owned(),
            usdt_onchain_decimals: c.usdt_onchain_decimals(),
        })
        .collect();
    Ok(Json(ChainsResponse { items }))
}

/// `GET /wallet/treasury` — four-chain treasury transparency + custody policy. Public.
///
/// Addresses come only from env; empty means unset (honest, not hidden).
///
/// # Errors
/// None — always 200 (validation of an unset address only yields an honest flag).
pub async fn treasury(State(state): State<Arc<AppState>>) -> ApiResult<Json<TreasuryResponse>> {
    let cfg = &state.config;
    let rows = [
        (Chain::Trx, cfg.treasury_trx.as_str()),
        (Chain::Sol, cfg.treasury_sol.as_str()),
        (Chain::Ton, cfg.treasury_ton.as_str()),
        (Chain::Bsc, cfg.treasury_bsc.as_str()),
    ];
    let items = rows
        .iter()
        .map(|(chain, addr)| TreasuryChainView {
            chain: chain.as_str().to_owned(),
            address: (*addr).to_owned(),
            configured: !addr.trim().is_empty()
                && kian_wallet::validate_address(*chain, addr).is_ok(),
        })
        .collect();
    Ok(Json(TreasuryResponse {
        items,
        movement_policy: "\u{647}\u{631} \u{62c}\u{627}\u{628}\u{647}‌\u{62c}\u{627}\u{6cc}\u{6cc} \u{62e}\u{632}\u{627}\u{646}\u{647} \u{646}\u{6cc}\u{627}\u{632}\u{645}\u{646}\u{62f} \u{644}\u{627}\u{6cc}\u{62d}\u{647}\u{654} \u{62a}\u{635}\u{648}\u{6cc}\u{628}‌\u{634}\u{62f}\u{647}\u{654} \u{67e}\u{627}\u{631}\u{644}\u{645}\u{627}\u{646} \u{627}\u{633}\u{62a} \
                         (bill:PASSED)\u{61b} \u{645}\u{628}\u{627}\u{644}\u{63a} \u{628}\u{627}\u{644}\u{627}\u{6cc} \u{622}\u{633}\u{62a}\u{627}\u{646}\u{647}\u{654} \u{645}\u{646}\u{634}\u{648}\u{631} \u{631}\u{623}\u{6cc} \u{627}\u{646}\u{633}\u{627}\u{646}\u{6cc} \u{645}\u{6cc}‌\u{62e}\u{648}\u{627}\u{647}\u{646}\u{62f} \
                         — \u{645}\u{627}\u{62f}\u{647}\u{654} \u{6f6} \u{634}\u{641}\u{627}\u{641}\u{6cc}\u{62a} \u{648} \u{645}\u{627}\u{62f}\u{647}\u{654} \u{6f7} HITL."
            .to_owned(),
    }))
}

/// `POST /wallet/addresses` — upsert a user's deposit address. Session. 201.
///
/// The address is validated against the official chain checksum; a bad address → 422 `INVALID_ADDRESS`.
///
/// # Errors
/// 401 session; 400 invalid body; 422 invalid address; 500 SQL error.
pub async fn set_address(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<DepositAddressView>)> {
    let user = require_session(&state, &headers).await?;
    let req: SetDepositAddressRequest = parse_body(&body)?;
    let chain = parse_chain(&req.chain)?;
    kian_wallet::validate_address(chain, &req.address).map_err(wallet_error)?;

    let row = WalletAddressRow {
        user_id: user.id.clone(),
        chain: chain.as_str().to_owned(),
        address: req.address.trim().to_owned(),
        created_at: now_iso(),
    };
    state
        .store
        .upsert_wallet_address(&row)
        .await
        .map_err(ApiFailure::from)?;
    Ok((
        StatusCode::CREATED,
        Json(DepositAddressView {
            chain: row.chain,
            address: row.address,
            created_at: row.created_at,
        }),
    ))
}

/// `GET /wallet/addresses` — my addresses. Session.
///
/// # Errors
/// 401 session; 500 SQL error.
pub async fn my_addresses(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<MyAddressesResponse>> {
    let user = require_session(&state, &headers).await?;
    let rows = state
        .store
        .wallet_addresses_for(&user.id)
        .await
        .map_err(ApiFailure::from)?;
    let items = rows
        .into_iter()
        .map(|r| DepositAddressView {
            chain: r.chain,
            address: r.address,
            created_at: r.created_at,
        })
        .collect();
    Ok(Json(MyAddressesResponse { items }))
}

/// `POST /wallet/usdt-deposits` — register a USDT deposit proof. Session. 201 `PENDING`.
///
/// Full validation at the edge: a valid chain, sender address per chain rules, a decimal
/// amount with ≤6 decimals, a hash of 8..=128. The same hash on the same chain → 422 (replay is
/// never minted twice).
///
/// # Errors
/// 401 session; 400 invalid body; 422 invalid address/amount or duplicate hash; 500 SQL error.
pub async fn create_usdt_deposit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<UsdtDepositView>)> {
    let user = require_session(&state, &headers).await?;
    let req: UsdtDepositCreateRequest = parse_body(&body)?;
    let chain = parse_chain(&req.chain)?;
    kian_wallet::validate_address(chain, &req.from_address).map_err(wallet_error)?;
    let micro = kian_wallet::parse_usdt(&req.amount_usdt).map_err(wallet_error)?;

    let tx_hash = req.tx_hash.trim().to_owned();
    let ok_hash = (8..=128).contains(&tx_hash.chars().count())
        && tx_hash.chars().all(|c| c.is_ascii_alphanumeric());
    if !ok_hash {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }

    if state
        .store
        .get_usdt_deposit_by_hash(chain.as_str(), &tx_hash)
        .await
        .map_err(ApiFailure::from)?
        .is_some()
    {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!(
                "Deposit {tx_hash} on {} is already recorded",
                chain.as_str()
            ),
        )
        .into());
    }

    let row = UsdtDepositRow {
        id: format!("udep:{}", uuid::Uuid::now_v7()),
        user_id: user.id.clone(),
        chain: chain.as_str().to_owned(),
        tx_hash: tx_hash.clone(),
        from_address: req.from_address.trim().to_owned(),
        amount_micro: micro,
        amount_usdt: req.amount_usdt.trim().to_owned(),
        status: "PENDING".into(),
        created_at: now_iso(),
        confirmed_at: None,
        confirmed_by: None,
    };
    state
        .store
        .insert_usdt_deposit(&row)
        .await
        .map_err(ApiFailure::from)?;
    Ok((StatusCode::CREATED, Json(deposit_view(&row))))
}

/// `GET /wallet/usdt-deposits` — my deposits (last 50). Session.
///
/// # Errors
/// 401 session; 500 SQL error.
pub async fn list_my_usdt_deposits(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<UsdtDepositListResponse>> {
    let user = require_session(&state, &headers).await?;
    let items = state
        .store
        .list_usdt_deposits_for(&user.id)
        .await
        .map_err(ApiFailure::from)?
        .iter()
        .map(deposit_view)
        .collect();
    Ok(Json(UsdtDepositListResponse { items }))
}

/// `POST /wallet/usdt-deposits/{id}/confirm` — admin: verification + Article 3 mint.
///
/// The rate comes from `KIAN_USDT_TO_DIN` (default 1000 Đ/USDT); correct computation:
/// `micro_usdt × rate ÷ 1_000_000` — no float on money.
///
/// # Errors
/// 401 admin; 404 unknown; 422 already decided or below the mint minimum; 500 SQL error.
pub async fn confirm_usdt_deposit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<UsdtDepositView>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let deposit = state
        .store
        .get_usdt_deposit(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::EscrowNotFound,
                format!("Usdt deposit {id} does not exist"),
            )
            .into()
        })?;

    if deposit.status != "PENDING" {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!("Usdt deposit {id} is not PENDING anymore"),
        )
        .into());
    }

    // Din amount = microUSDT × rate ÷ 1,000,000 — computed in i128 with checked_mul to prevent i64 overflow
    let rate = state.config.usdt_to_din_rate;
    let din_128 = i128::from(deposit.amount_micro)
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

    // safe order: mint first with the idempotency key; only after the mint succeeds (or a successful
    // replay), the deposit status flips to CONFIRMED so money never burns on a mint-cap error.
    let _mint = crate::handlers::wallet::mint_to_account(
        &state,
        &user.account_id,
        din,
        Some(format!(
            "usdt deposit {} {}",
            deposit.chain, deposit.tx_hash
        )),
        Some(format!("usdt-deposit:{id}")),
    )
    .await?;

    let updated = state
        .store
        .set_usdt_deposit_status(&id, "CONFIRMED", "admin")
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(deposit_view(&updated)))
}

/// `POST /wallet/usdt-deposits/{id}/reject` — admin: reject the proof (no mint).
///
/// # Errors
/// 401 admin; 404 unknown; 422 already decided; 500 SQL error.
pub async fn reject_usdt_deposit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<UsdtDepositView>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let updated = state
        .store
        .set_usdt_deposit_status(&id, "REJECTED", "admin")
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
        cfg.treasury_bsc = "0x55d398326f99059fF775485246999027B3197955".into();
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

    async fn login(app: &axum::Router, tg: &str, username: &str) -> String {
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(
                        json!({ "initData": make_test_init_data(BOT, tg, Some(username)) })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        json_body(res).await["token"].as_str().unwrap().to_owned()
    }

    #[test]
    fn wallet_error_maps_to_honest_codes() {
        let e = wallet_error(WalletError::InvalidAddress {
            chain: "BSC",
            detail: "EIP-55 checksum mismatch".into(),
        });
        assert_eq!(e.status(), 422);
        assert_eq!(e.envelope().error.code, "INVALID_ADDRESS");
        let e = wallet_error(WalletError::CustodyViolation("no bill".into()));
        assert_eq!(e.envelope().error.code, "CUSTODY_VIOLATION");
    }

    #[tokio::test]
    async fn chains_and_treasury_are_public_and_honest() {
        let app = app().await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/wallet/chains")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        let items = body["items"].as_array().unwrap();
        assert_eq!(items.len(), 4, "four chains");
        assert_eq!(items[0]["chain"], "TRX");
        assert_eq!(
            items[0]["usdtContract"],
            "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"
        );
        let bsc = items.iter().find(|c| c["chain"] == "BSC").unwrap();
        assert_eq!(
            bsc["usdtOnchainDecimals"], 18,
            "BSC on-chain unit is honest"
        );

        // treasury — no authentication; a valid configured address gets the true flag
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/wallet/treasury")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        let bsc = body["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["chain"] == "BSC")
            .unwrap();
        assert_eq!(bsc["configured"], true);
        assert!(body["movementPolicy"].as_str().unwrap().contains("\u{67e}\u{627}\u{631}\u{644}\u{645}\u{627}\u{646}"));
    }

    #[tokio::test]
    async fn bad_addresses_are_never_registered() {
        let app = app().await;
        let token = login(&app, "40", "holder").await;

        // broken Tron checksum — 422 INVALID_ADDRESS
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/addresses")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({"chain": "TRX", "address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6u"})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
        assert_eq!(json_body(res).await["error"]["code"], "INVALID_ADDRESS");

        // unknown chain — 400
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/addresses")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({"chain": "DOGE", "address": "x12345678"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 400);

        // valid BSC address — 201 and replacement works
        for addr in [
            "0x55d398326f99059fF775485246999027B3197955",
            "55d398326f99059ff775485246999027b3197955", // all-lowercase — no checksum
        ] {
            let res = app
                .clone()
                .oneshot(
                    axum::http::Request::builder()
                        .method("POST")
                        .uri("/api/v1/wallet/addresses")
                        .header("authorization", format!("Bearer {token}"))
                        .body(Body::from(
                            json!({"chain": "BSC", "address": addr}).to_string(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), 201, "{addr} must register");
        }
        // my list — only one BSC row (the last address replaced it)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/wallet/addresses")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = json_body(res).await;
        let items = body["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0]["address"],
            "55d398326f99059ff775485246999027b3197955"
        );
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)] // full deposit→confirm→mint cycle — test readability matters more
    async fn usdt_deposit_confirm_mints_through_charter_path() {
        let app = app().await;
        let token = login(&app, "41", "staker").await;

        // valid TON address — from the jUSDT canonical contract (valid CRC)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/usdt-deposits")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "chain": "TON",
                            "txHash": "tonhash12345678",
                            "fromAddress": "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs",
                            "amountUsdt": "1.5"
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
        let dep_id = body["id"].as_str().unwrap().to_owned();

        // bad amount — 400 (wire validation at the edge, the same M3 contract)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/usdt-deposits")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "chain": "TON",
                            "txHash": "tonhash87654321",
                            "fromAddress": "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs",
                            "amountUsdt": "1.2345678"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            400,
            "more than 6 decimals must fail at the edge"
        );

        // confirm without admin — 401
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/usdt-deposits/{dep_id}/confirm"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);

        // admin confirms — default rate 1000 Đ/USDT → 1,500 Đ
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/usdt-deposits/{dep_id}/confirm"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "confirm must succeed");
        let body = json_body(res).await;
        assert_eq!(body["status"], "CONFIRMED");

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
        assert_eq!(me["balance"], 1_500, "1.5 USDT × 1000 Đ/USDT");

        // same hash on the same chain — 422 (a duplicate hash is never minted twice)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/usdt-deposits")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "chain": "TON",
                            "txHash": "tonhash12345678",
                            "fromAddress": "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs",
                            "amountUsdt": "1.5"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);

        // second resolution — 422 (one-way transition)
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/usdt-deposits/{dep_id}/reject"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
    }

    #[tokio::test]
    async fn usdt_deposit_remains_pending_if_mint_cap_fails() {
        let app = app().await;
        let token = login(&app, "42", "whale_usdt").await;

        // 2000 USDT × 1000 Đ/USDT = 2,000,000 Đ; to exceed the annual cap of 500 billion Đ:
        // 600,000,000 USDT × 1000 = 600 billion Đ > cap
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/usdt-deposits")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "chain": "TON",
                            "txHash": "usdtcapfail12345",
                            "fromAddress": "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs",
                            "amountUsdt": "600000000"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let dep_id = json_body(res).await["id"].as_str().unwrap().to_owned();

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/usdt-deposits/{dep_id}/confirm"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
        assert_eq!(json_body(res).await["error"]["code"], "MINT_CAP_EXCEEDED");

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/wallet/usdt-deposits")
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
