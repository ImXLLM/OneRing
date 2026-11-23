//! هندلرهای کیف پول چهارزنجیره M5 — متادیتا زنجیره‌ها، آدرس واریز کاربر و
//! واریز USDT با چک‌سام‌های رسمی زنجیره.
//!
//! گیت‌ها:
//!  * متادیتا و خزانه — عمومی (شفافیت مادهٔ ۶؛ آدرس خزانه از env می‌آید).
//!  * آدرس/واریز کاربر — نشست؛ آدرس‌ها با `kian-wallet` اعتبارسنجی می‌شوند
//!    (`Base58Check` ترون، ۳۲بایت سولانا، `CRC16-XMODEM` تون، `EIP-55` بایننس‌چین)
//!    — آدرس اشتباه هرگز ثبت نمی‌شود.
//!  * تأیید/رد واریز — ادمین؛ تأیید از مسیر ضرب مادهٔ ۳ می‌گذرد (نرخ USDT از env)
//!    — ضرب هرگز خودکار و بی‌ادمین نیست (همان قرارداد M3).

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

/// نگاشت خطای کیف پول → پاکت HTTP صادقانه.
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

/// تجزیهٔ زنجیره از سیم — رشتهٔ ناشناخته = 400 `INVALID_REQUEST`.
fn parse_chain(raw: &str) -> ApiResult<Chain> {
    Chain::from_str_raw(raw).ok_or_else(|| kian_contracts::HttpFailure::invalid_request(1).into())
}

/// نمای واریز از ردیف db — نقطهٔ واحد ساخت `UsdtDepositView`.
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

/// `GET /wallet/chains` — متادیتای چهار زنجیره + قرارداد کانونیک USDT. عمومی.
///
/// # Errors
/// هیچ — همیشه ۲۰۰.
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

/// `GET /wallet/treasury` — شفافیت خزانهٔ چهار زنجیره + سیاست حراست. عمومی.
///
/// آدرس‌ها فقط از env می‌آیند؛ خالی یعنی تنظیم‌نشده (صادقانه، نه مخفی).
///
/// # Errors
/// هیچ — همیشه ۲۰۰ (اعتبارسنجی آدرس تنظیم‌نشده فقط فلگ صادقانه می‌دهد).
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
        movement_policy: "هر جابه‌جایی خزانه نیازمند لایحهٔ تصویب‌شدهٔ پارلمان است \
                         (bill:PASSED)؛ مبالغ بالای آستانهٔ منشور رأی انسانی می‌خواهند \
                         — مادهٔ ۶ شفافیت و مادهٔ ۷ HITL."
            .to_owned(),
    }))
}

/// `POST /wallet/addresses` — ثبت/به‌روزرسانی آدرس واریز کاربر. نشست. ۲۰۱.
///
/// آدرس با چک‌سام رسمی زنجیره اعتبارسنجی می‌شود؛ آدرس بد → ۴۲۲ `INVALID_ADDRESS`.
///
/// # Errors
/// ۴۰۱ نشست؛ ۴۰۰ بدنهٔ نامعتبر؛ ۴۲۲ آدرس نامعتبر؛ 500 خطای SQL.
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

/// `GET /wallet/addresses` — آدرس‌های من. نشست.
///
/// # Errors
/// ۴۰۱ نشست؛ 500 خطای SQL.
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

/// `POST /wallet/usdt-deposits` — ثبت اثبات واریز USDT. نشست. ۲۰۱ `PENDING`.
///
/// اعتبارسنجی کامل در لبه: زنجیرهٔ معتبر، آدرس فرستنده با قواعد زنجیره، مبلغ
/// دهدهی ≤۶ اعشار، هش ۸..=۱۲۸. همان هش روی همان زنجیره → ۴۲۲ (replay هرگز
/// دوبار ضرب نمی‌شود).
///
/// # Errors
/// ۴۰۱ نشست؛ ۴۰۰ بدنهٔ نامعتبر؛ ۴۲۲ آدرس/مبلغ نامعتبر یا هش تکراری؛ 500 خطای SQL.
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

/// `GET /wallet/usdt-deposits` — واریزهای من (۵۰ آخر). نشست.
///
/// # Errors
/// ۴۰۱ نشست؛ 500 خطای SQL.
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

/// `POST /wallet/usdt-deposits/{id}/confirm` — ادمین: راستی‌آزمایی + ضرب مادهٔ ۳.
///
/// نرخ از `KIAN_USDT_TO_DIN` (پیش‌فرض ۱۰۰۰ Đ/USDT)؛ محاسبهٔ صحیح:
/// `micro_usdt × rate ÷ 1_000_000` — بدون float روی پول.
///
/// # Errors
/// ۴۰۱ ادمین؛ 404 ناشناخته؛ ۴۲۲ قبلاً تصمیم‌گرفته یا زیر حداقل ضرب؛ 500 خطای SQL.
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

    // مبلغ دیین = microUSDT × نرخ ÷ ۱٬۰۰۰٬۰۰۰ — در i128 با checked_mul برای جلوگیری از overflow i64
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

    // ترتیب ایمن: اول ضرب با کلید یکسان‌سازی؛ فقط پس از موفقیت ضرب (یا replay
    // موفق)، وضعیت واریز به CONFIRMED تغییر می‌کند تا در خطای سقف ضرب پول نسوزد.
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

/// `POST /wallet/usdt-deposits/{id}/reject` — ادمین: رد اثبات (بدون ضرب).
///
/// # Errors
/// ۴۰۱ ادمین؛ 404 ناشناخته؛ ۴۲۲ قبلاً تصمیم‌گرفته؛ 500 خطای SQL.
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

        // خزانه — بدون احراز؛ آدرس تنظیم‌شدهٔ معتبر فلگ true می‌گیرد
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
        assert!(body["movementPolicy"].as_str().unwrap().contains("پارلمان"));
    }

    #[tokio::test]
    async fn bad_addresses_are_never_registered() {
        let app = app().await;
        let token = login(&app, "40", "holder").await;

        // چک‌سام خراب ترون — ۴۲۲ INVALID_ADDRESS
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

        // زنجیرهٔ ناشناخته — ۴۰۰
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

        // آدرس معتبر BSC — ۲۰۱ و جایگزینی کار می‌کند
        for addr in [
            "0x55d398326f99059fF775485246999027B3197955",
            "55d398326f99059ff775485246999027b3197955", // همه‌کوچک — بدون چک‌سام
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
        // فهرست من — فقط یک ردیف BSC (آخرین آدرس جایگزین شد)
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
    #[allow(clippy::too_many_lines)] // چرخهٔ کامل واریز→تأیید→ضرب — خوانایی تست مهم‌تر است
    async fn usdt_deposit_confirm_mints_through_charter_path() {
        let app = app().await;
        let token = login(&app, "41", "staker").await;

        // آدرس معتبر تون — از قرارداد کانونیک jUSDT (CRC معتبر)
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

        // مبلغ بد — ۴۰۰ (اعتبارسنجی وایر در لبه، همان قرارداد M3)
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

        // تأیید بدون ادمین — ۴۰۱
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

        // ادمین تأیید می‌کند — نرخ پیش‌فرض ۱۰۰۰ Đ/USDT → ۱٬۵۰۰ Đ
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

        // همان هش روی همان زنجیره — ۴۲۲ (هش تکراری دوبار ضرب نمی‌شود)
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

        // تعیین تکلیف دوباره — ۴۲۲ (گذار یک‌طرفه)
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

        // ۲۰۰۰ USDT × ۱۰۰۰ Đ/USDT = ۲٬۰۰۰٬۰۰۰ Đ > سقف سالانه ۱٬۰۰۰٬۰۰۰ Đ
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
                            "amountUsdt": "2000"
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
