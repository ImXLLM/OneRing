//! هندلرهای کیف پول — ضرب (مادهٔ ۳: سقف سالانه) و سوزاندن (مادهٔ ۴: overdraft
//! ناممکن). هر فرمان از مسیر دامنهٔ خالص + `apply` می‌گذرد؛ replay هرگز دوبار
//! پول جابه‌جا نمی‌کند (قید UNIQUE + پیش‌چک).

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use kian_contracts::{
    BurnRequest, ConfirmWithdrawalRequest, CreateWithdrawalRequest, FaucetRequest, MintRequest,
    WalletMoveResponse, WithdrawalListResponse, WithdrawalRequestView, FAUCET_DEFAULT_MICRO,
    FAUCET_MAX_MICRO,
};
use kian_db::{now_iso, ApplyOptions, ApplyResult, CommandOutcome, V3Store, WithdrawalRequestRow};
use kian_domain::{
    assert_mint_within_cap, post, Din, DomainError, DomainErrorCode, DomainEvent, DomainResult,
    EscrowRegistry, LedgerState, PostCommand, Posting, TransactionPostedEvent, BURN_ACCOUNT_ID,
    MINT_ACCOUNT_ID,
};
use kian_wallet::{Chain, WalletError};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::{require_admin, require_session};
use crate::state::AppState;
use crate::views::wire_din;

/// شروع سال UTC جاری — قرارداد سقف منشور (مادهٔ ۳). مقایسهٔ رشته‌ای RFC3339
/// با فرمت یکسان millis+Z معادل مقایسهٔ زمانی است.
fn year_start_iso() -> String {
    use chrono::{Datelike, TimeZone};
    let y = chrono::Utc::now().year();
    chrono::Utc.with_ymd_and_hms(y, 1, 1, 0, 0, 0).single().map_or_else(
        || format!("{y}-01-01T00:00:00.000Z"),
        |t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    )
}

/// ضرب سال جاری از وضعیت لودشده داخل تراکنش — دقیق و بدون مسابقه: هر فرمان
/// کل دفتر را در همان تراکنش می‌بیند (الگوی فاز ۲).
fn minted_this_year_locked(ledger: &LedgerState) -> i64 {
    let year_start = year_start_iso();
    let mut sum = 0i64;
    for tx in ledger.txs.values() {
        if tx.at.as_str() >= year_start.as_str() {
            for p in &tx.postings {
                if p.account_id == MINT_ACCOUNT_ID && p.amount < 0 {
                    sum += -p.amount;
                }
            }
        }
    }
    sum
}

/// بدنهٔ فرمان ضرب/سوزاندن.
struct MoveCmd {
    account_id: String,
    amount: Din,
    memo: Option<String>,
    key: Option<String>,
}

async fn wallet_move(store: &V3Store, cmd: MoveCmd, mint: bool) -> Result<ApplyResult, ApiFailure> {
    let key = cmd.key.clone();
    let account_id = cmd.account_id.clone();
    let amount = cmd.amount;
    let memo = cmd.memo.clone().unwrap_or_else(|| {
        if mint {
            format!("mint {amount}")
        } else {
            format!("burn {amount}")
        }
    });
    let action = if mint { "wallet.mint" } else { "wallet.burn" };
    let outcome = store
        .apply(
            ApplyOptions {
                actor: "admin".into(),
                action: action.into(),
                target: Some(account_id.clone()),
                escrow_id: None,
                idempotency_key: key.clone(),
            },
            |ledger: &LedgerState, _reg: &EscrowRegistry| -> DomainResult<CommandOutcome> {
                // مادهٔ ۳ — سقف ضرب سالانه فقط روی مسیر mint؛ خوانش از همان
                // وضعیتِ لودشده داخل تراکنش — دو ضرب هم‌زمان هرگز از سقف می‌گذرند.
                if mint {
                    let minted = minted_this_year_locked(ledger);
                    assert_mint_within_cap(Din::new(minted)?, amount)?;
                }
                let postings = if mint {
                    vec![
                        Posting::new(MINT_ACCOUNT_ID, -amount.as_i64()),
                        Posting::new(account_id.clone(), amount.as_i64()),
                    ]
                } else {
                    vec![
                        Posting::new(account_id.clone(), -amount.as_i64()),
                        Posting::new(BURN_ACCOUNT_ID, amount.as_i64()),
                    ]
                };
                let posted = post(
                    ledger,
                    &PostCommand {
                        id: None,
                        idempotency_key: key.clone(),
                        postings,
                        memo: Some(memo),
                        at: Some(now_iso()),
                    },
                )?;
                let event = TransactionPostedEvent::new(&posted.tx.id, &posted.tx.at);
                // memo/key/correlation را کامل می‌کنیم — شکل سیمی رویداد فاز ۵
                let event = TransactionPostedEvent {
                    memo: posted.tx.memo.clone(),
                    idempotency_key: posted.tx.idempotency_key.clone(),
                    ..event
                };
                Ok(CommandOutcome {
                    ledger: posted.state,
                    tx: Some(posted.tx),
                    escrow: None,
                    events: vec![DomainEvent::TransactionPosted(event)],
                })
            },
        )
        .await?;
    Ok(outcome)
}

/// `POST /wallet/mint` — ضرب دیین به یک حساب. ادمین.
///
/// # Errors
/// ۴۲۲ `MINT_CAP_EXCEEDED` (مادهٔ ۳)؛ 404 `UNKNOWN_ACCOUNT`؛ 400 بدنهٔ نامعتبر.
pub async fn mint(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<WalletMoveResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let req: MintRequest = crate::handlers::parse_body(&body)?;
    let amount = Din::new(req.amount.get())?;
    let cmd = MoveCmd {
        account_id: req.account_id.get().to_owned(),
        amount,
        memo: req.memo,
        key: req.idempotency_key.as_ref().map(|k| k.get().to_owned()),
    };
    let result = wallet_move(&state.store, cmd, true).await?;
    to_move_response(&result, req.account_id.get())
}

/// `POST /wallet/burn` — سوزاندن دیین از یک حساب؛ overdraft ناممکن (مادهٔ ۴).
///
/// # Errors
/// ۴۲۲ `ACCOUNT_OVERDRAFT`؛ 404 `UNKNOWN_ACCOUNT`؛ 400 بدنهٔ نامعتبر.
pub async fn burn(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<WalletMoveResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let req: BurnRequest = crate::handlers::parse_body(&body)?;
    let amount = Din::new(req.amount.get())?;
    let cmd = MoveCmd {
        account_id: req.account_id.get().to_owned(),
        amount,
        memo: req.memo,
        key: req.idempotency_key.as_ref().map(|k| k.get().to_owned()),
    };
    let result = wallet_move(&state.store, cmd, false).await?;
    to_move_response(&result, req.account_id.get())
}

/// ضرب داخلی M3 — فقط برای تأیید واریز TON (مادهٔ ۳ با نرخ تبدیل). بدون هدر
/// ادمین؛ هویت فراخواننده سمت سرور ثابت است. همان مسیر دامنهٔ ضرب با سقف سالانه.
///
/// # Errors
/// ۴۲۲ `MINT_CAP_EXCEEDED`؛ ۴۲۲ `DIN_OUT_OF_RANGE` اگر amount ≤ 0؛ 500 خطای داخلی.
pub(crate) async fn mint_to_account(
    state: &Arc<AppState>,
    account_id: &str,
    amount: i64,
    memo: Option<String>,
    key: Option<String>,
) -> ApiResult<Json<WalletMoveResponse>> {
    let din = Din::new(amount)?;
    let cmd = MoveCmd {
        account_id: account_id.to_owned(),
        amount: din,
        memo,
        key,
    };
    let result = wallet_move(&state.store, cmd, true).await?;
    to_move_response(&result, account_id)
}

/// `POST /wallet/faucet` — شیرِ اعتبار آزمایشی برای تستِ محلی (فقط حالت dev).
///
/// ۵٬۰۰۰ KIAN پیش‌فرض (سقف ۵۰٬۰۰۰ KIAN) از مسیر رسمی ضرب مادهٔ ۳ به حساب
/// شهروند نشست‌دار می‌رسد — همان `wallet::mint_to_account`؛ خارج از حالت dev
/// صادقانه ۴۰۳ می‌دهد تا هیچ شیری به تولید نشت کند.
///
/// # Errors
/// ۴۰۱ نشست؛ ۴۰۳ `DEV_MODE_DISABLED`؛ ۴۰۰ مبلغ خارج از بازه؛ ۴۲۲ `MINT_CAP_EXCEEDED`.
pub async fn faucet(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<WalletMoveResponse>> {
    let user = require_session(&state, &headers).await?;
    if !state.config.dev_mode {
        return Err(ApiFailure::Http(kian_contracts::HttpFailure::dev_mode_disabled()));
    }
    let req: FaucetRequest = parse_body(&body)?;
    let amount = req.amount_micro.unwrap_or(FAUCET_DEFAULT_MICRO);
    if (1..=FAUCET_MAX_MICRO).contains(&amount) {
        let memo = Some(format!("faucet demo credit — {amount} micro-KIAN"));
        mint_to_account(&state, &user.account_id, amount, memo, None).await
    } else {
        Err(ApiFailure::Http(kian_contracts::HttpFailure::invalid_request(1)))
    }
}

fn to_move_response(result: &ApplyResult, account_id: &str) -> ApiResult<Json<WalletMoveResponse>> {
    let tx = result.tx.as_ref().ok_or(kian_contracts::HttpFailure::internal_no_tx())?;
    let balance = result.ledger.balances.get(account_id).copied().unwrap_or(0);
    Ok(Json(WalletMoveResponse {
        tx_id: tx.id.clone(),
        account_id: account_id.to_owned(),
        balance,
        replayed: result.replayed,
    }))
}

fn withdrawal_view(w: &WithdrawalRequestRow) -> WithdrawalRequestView {
    WithdrawalRequestView {
        id: w.id.clone(),
        account_id: w.account_id.clone(),
        chain: w.chain.clone(),
        asset: w.asset.clone(),
        destination_address: w.destination_address.clone(),
        amount_din: wire_din(w.amount_din),
        tx_hash: w.tx_hash.clone(),
        status: w.status.clone(),
        created_at: w.created_at,
        resolved_at: w.resolved_at,
    }
}

fn map_wallet_err(e: WalletError) -> ApiFailure {
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

/// بازگرداندن وجه یک درخواست برداشت ردشده از `burn:void` به حساب شهروند بدون مصرف سقف ضرب سالانه.
async fn refund_burned_withdrawal(
    store: &V3Store,
    account_id: &str,
    amount: Din,
    withdrawal_id: &str,
) -> Result<ApplyResult, ApiFailure> {
    let key = format!("withdrawal:{withdrawal_id}:refund");
    let target = account_id.to_owned();
    let memo = format!("withdrawal refund {withdrawal_id}");
    let outcome = store
        .apply(
            ApplyOptions {
                actor: "admin".into(),
                action: "withdrawal.refund".into(),
                target: Some(target.clone()),
                escrow_id: None,
                idempotency_key: Some(key.clone()),
            },
            move |ledger: &LedgerState, _reg: &EscrowRegistry| -> DomainResult<CommandOutcome> {
                let posted = post(
                    ledger,
                    &PostCommand {
                        id: None,
                        idempotency_key: Some(key),
                        postings: vec![
                            Posting::new(BURN_ACCOUNT_ID, -amount.as_i64()),
                            Posting::new(target, amount.as_i64()),
                        ],
                        memo: Some(memo),
                        at: Some(now_iso()),
                    },
                )?;
                let event = TransactionPostedEvent {
                    memo: posted.tx.memo.clone(),
                    idempotency_key: posted.tx.idempotency_key.clone(),
                    ..TransactionPostedEvent::new(&posted.tx.id, &posted.tx.at)
                };
                Ok(CommandOutcome {
                    ledger: posted.state,
                    tx: Some(posted.tx),
                    escrow: None,
                    events: vec![DomainEvent::TransactionPosted(event)],
                })
            },
        )
        .await?;
    Ok(outcome)
}

/// `POST /wallet/withdrawals` — ثبت درخواست برداشت کریپتو با قفل/سوزاندن اتمی موجودی (ضد دابل‌اسپند).
///
/// # Errors
/// ۴۰۱ نشست؛ ۴۰۰ ورودی نامعتبر؛ ۴۲۲ آدرس نامعتبر یا موجودی ناکافی (`ACCOUNT_OVERDRAFT`).
pub async fn create_withdrawal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<WithdrawalRequestView>)> {
    let user = require_session(&state, &headers).await?;
    let req: CreateWithdrawalRequest = parse_body(&body)?;
    if req.amount_din.get() < 1 {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }
    let chain = Chain::from_str_raw(req.chain.trim())
        .ok_or_else(|| ApiFailure::Http(kian_contracts::HttpFailure::invalid_request(1)))?;
    let asset = req.asset.trim().to_ascii_uppercase();
    let asset_ok = match asset.as_str() {
        "USDT" => true,
        "TON" => chain == Chain::Ton,
        _ => false,
    };
    if !asset_ok {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }
    let dest = req.destination_address.trim();
    kian_wallet::validate_address(chain, dest).map_err(map_wallet_err)?;

    let id = format!("wdr:{}", uuid::Uuid::now_v7());
    let amount = Din::new(req.amount_din.get())?;

    // قفل/سوزاندن اتمی Din در لحظهٔ ثبت — اگر موجودی ناکافی باشد ۴۲۲ و هیچ رکوردی ساخته نمی‌شود
    let _burn = wallet_move(
        &state.store,
        MoveCmd {
            account_id: user.account_id.clone(),
            amount,
            memo: Some(format!("withdrawal lock {id} ({} {})", chain.as_str(), asset)),
            key: Some(format!("withdrawal:{id}:lock")),
        },
        false,
    )
    .await?;

    let row = WithdrawalRequestRow {
        id,
        account_id: user.account_id,
        chain: chain.as_str().to_owned(),
        asset,
        destination_address: dest.to_owned(),
        amount_din: amount.as_i64(),
        tx_hash: None,
        status: "PENDING".to_owned(),
        created_at: chrono::Utc::now().timestamp(),
        resolved_at: None,
    };
    state
        .store
        .insert_withdrawal_request(&row)
        .await
        .map_err(ApiFailure::from)?;
    Ok((StatusCode::CREATED, Json(withdrawal_view(&row))))
}

/// `GET /wallet/withdrawals` — فهرست درخواست‌های برداشت شهروند نشست‌دار.
///
/// # Errors
/// ۴۰۱ نشست؛ ۵۰۰ خطای SQL.
pub async fn list_my_withdrawals(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<WithdrawalListResponse>> {
    let user = require_session(&state, &headers).await?;
    let rows = state
        .store
        .list_withdrawals_for_account(&user.account_id)
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(WithdrawalListResponse {
        items: rows.iter().map(withdrawal_view).collect(),
    }))
}

/// پارامتر کوئری فهرست ادمین برداشت‌ها.
#[derive(Debug, Default, Deserialize)]
pub struct AdminWithdrawalQuery {
    /// فیلتر وضعیت (`PENDING` / `CONFIRMED` / `REJECTED`)
    pub status: Option<String>,
}

/// `GET /admin/withdrawals` — فهرست همهٔ درخواست‌های برداشت برای پنل ادمین.
///
/// # Errors
/// ۴۰۱ کلید ادمین؛ ۵۰۰ خطای SQL.
pub async fn admin_list_withdrawals(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<AdminWithdrawalQuery>,
) -> ApiResult<Json<WithdrawalListResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let rows = state
        .store
        .list_withdrawals_all(q.status.as_deref(), 200)
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(WithdrawalListResponse {
        items: rows.iter().map(withdrawal_view).collect(),
    }))
}

/// `POST /wallet/withdrawals/{id}/confirm` — تأیید پرداخت روی زنجیره توسط ادمین + ثبت `txHash`.
///
/// # Errors
/// ۴۰۱ کلید ادمین؛ ۴۰۰ هش نامعتبر؛ ۴۰۴ ناشناخته؛ ۴۲۲ قبلاً تصمیم‌گرفته‌شده.
pub async fn confirm_withdrawal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<Json<WithdrawalRequestView>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let req: ConfirmWithdrawalRequest = parse_body(&body)?;
    let tx_hash = req.tx_hash.trim();
    let hash_ok = (8..=128).contains(&tx_hash.chars().count())
        && tx_hash
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !hash_ok {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }
    let updated = state
        .store
        .resolve_withdrawal_request(&id, "CONFIRMED", Some(tx_hash), chrono::Utc::now().timestamp())
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(withdrawal_view(&updated)))
}

/// `POST /wallet/withdrawals/{id}/reject` — رد درخواست برداشت و بازگشت اتمی Din به حساب شهروند.
///
/// ترتیب ایمن: اول بازگشت وجه با کلید یکتای `withdrawal:{id}:refund`، سپس تغییر وضعیت به `REJECTED`.
///
/// # Errors
/// ۴۰۱ کلید ادمین؛ ۴۰۴ ناشناخته؛ ۴۲۲ قبلاً تصمیم‌گرفته‌شده.
pub async fn reject_withdrawal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<WithdrawalRequestView>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let current = state
        .store
        .get_withdrawal_request(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::EscrowNotFound,
                format!("Withdrawal request {id} does not exist"),
            )
            .into()
        })?;
    if current.status != "PENDING" {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!("Withdrawal request {id} is already {}", current.status),
        )
        .into());
    }
    let amount = Din::new(current.amount_din)?;
    let _refund = refund_burned_withdrawal(&state.store, &current.account_id, amount, &id).await?;
    let updated = state
        .store
        .resolve_withdrawal_request(&id, "REJECTED", None, chrono::Utc::now().timestamp())
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(withdrawal_view(&updated)))
}

#[cfg(test)]
mod withdrawal_tests {
    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_contracts::make_test_init_data;
    use kian_db::V3Store;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    const BOT: &str = "123456:TEST-BOT-TOKEN-WDR";
    const ADMIN: &str = "test-admin-key-0123456789";
    // آدرس معتبر بایننس‌چین EIP-55 (تمام حروف کوچک نیز در EIP-55 پذیرفته می‌شود)
    const VALID_BSC: &str = "0x742d35cc6634c0532925a3b844bc454e4438f44e";

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
    #[allow(clippy::too_many_lines)]
    async fn withdrawal_flow_locks_balance_reject_refunds_and_confirm_finalizes() {
        let app = app().await;
        let tok = auth(&app, "901", "seller_wdr").await;

        // شارژ ۵۰۰ Đ
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/mint")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(json!({ "accountId": "citizen:901", "amount": 500 }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);

        // آدرس نامعتبر → ۴۲۲ INVALID_ADDRESS
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/withdrawals")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from(
                        json!({
                            "chain": "BSC",
                            "asset": "USDT",
                            "destinationAddress": "not-an-address",
                            "amountDin": 200
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);

        // برداشت ۱ (۳۰۰ Đ) → موجودی بلافاصله به ۲۰۰ Đ کاهش می‌یابد
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/withdrawals")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from(
                        json!({
                            "chain": "BSC",
                            "asset": "USDT",
                            "destinationAddress": VALID_BSC,
                            "amountDin": 300
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let w1 = json_body(res).await;
        let w1_id = w1["id"].as_str().unwrap().to_owned();
        assert_eq!(w1["status"], "PENDING");

        // تلاش برای دابل‌اسپند (برداشت دوبارهٔ ۳۰۰ Đ از موجودی ۲۰۰ Đ) → ۴۲۲ ACCOUNT_OVERDRAFT
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/withdrawals")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from(
                        json!({
                            "chain": "BSC",
                            "asset": "USDT",
                            "destinationAddress": VALID_BSC,
                            "amountDin": 300
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
        assert_eq!(json_body(res).await["error"]["code"], "ACCOUNT_OVERDRAFT");

        // رد توسط ادمین → بازگشت ۳۰۰ Đ به شهروند (موجودی دوباره ۵۰۰ Đ)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/withdrawals/{w1_id}/reject"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(json_body(res).await["status"], "REJECTED");

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["balance"], 500);

        // برداشت ۲ (۴۰۰ Đ) + تأیید ادمین با txHash → موجودی نهایی ۱۰۰ Đ و دفتر متوازن
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/withdrawals")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from(
                        json!({
                            "chain": "BSC",
                            "asset": "USDT",
                            "destinationAddress": VALID_BSC,
                            "amountDin": 400
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let w2_id = json_body(res).await["id"].as_str().unwrap().to_owned();

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/wallet/withdrawals/{w2_id}/confirm"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(json!({ "txHash": "0xabc1234567890def" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let confirmed = json_body(res).await;
        assert_eq!(confirmed["status"], "CONFIRMED");
        assert_eq!(confirmed["txHash"], "0xabc1234567890def");

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/economy")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let eco = json_body(res).await;
        assert_eq!(eco["trialBalanceSum"], 0);
        assert_eq!(eco["balanced"], true);
    }

    // ── شیرِ اعتبار آزمایشی (faucet — فقط حالت dev) + فهرست ادمین برداشت‌ها ──

    /// ورود دمو برای تست‌های فاست — ۲۰۱ و توکن.
    async fn dev_auth(app: &axum::Router, tg_id: &str, username: &str) -> String {
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/dev-login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "tgUserId": tg_id, "username": username }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "helper: dev-login must succeed");
        json_body(res).await["token"].as_str().unwrap().to_owned()
    }

    #[tokio::test]
    async fn faucet_mints_default_five_kian_and_validates_range() {
        let app = app().await; // dev_mode = true (پایهٔ تست)
        let tok = dev_auth(&app, "2002", "buyer_demo").await;

        // فاست پیش‌فرض — ۵٬۰۰۰ KIAN = ۵٬۰۰۰٬۰۰۰٬۰۰۰ micro
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/faucet")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["accountId"], "citizen:2002");
        assert_eq!(body["balance"], 5_000_000_000i64);
        assert_eq!(body["replayed"], false);

        // /me همین تراز را می‌بیند
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["balance"], 5_000_000_000i64);

        // مبلغ سفارشی معتبر — ۱٬۰۰۰ KIAN دیگر
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/faucet")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from(json!({ "amountMicro": 1_000_000_000i64 }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(json_body(res).await["balance"], 6_000_000_000i64);

        // بالای سقف — ۴۰۰
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/faucet")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from(
                        json!({ "amountMicro": 50_000_000_001i64 }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 400);

        // بدون نشست — ۴۰۱
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/faucet")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }

    /// فاست خارج از حالت dev — ۴۰۳ `DEV_MODE_DISABLED` (هیچ شیری به تولید نشت نمی‌کند).
    #[tokio::test]
    async fn faucet_is_forbidden_when_dev_mode_off() {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some(ADMIN.to_owned());
        cfg.bot_token = Some(BOT.to_owned());
        cfg.dev_mode = false;
        let app = crate::routes::build_router(crate::state::AppState::new(store, cfg).shared());

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/faucet")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401, "no session yet");
    }

    /// مسیر مستقیم پنل بنیان‌گذار — GET /admin/wallet/withdrawals همهٔ درخواست‌ها را می‌بیند.
    #[tokio::test]
    async fn admin_wallet_withdrawals_route_lists_all_requests() {
        let app = app().await;
        let tok = dev_auth(&app, "901", "creator_wdr").await;

        // شارژ ۵۰۰ Đ با کلید ادمین
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/mint")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(
                        json!({ "accountId": "citizen:901", "amount": 500 }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);

        // برداشت ۳۰۰ Đ
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/withdrawals")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from(
                        json!({
                            "chain": "BSC",
                            "asset": "USDT",
                            "destinationAddress": VALID_BSC,
                            "amountDin": 300
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);

        // فهرست ادمین — مسیر جدید /admin/wallet/withdrawals
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/wallet/withdrawals")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let list = json_body(res).await;
        assert_eq!(list["items"].as_array().map(Vec::len), Some(1));
        assert_eq!(list["items"][0]["status"], "PENDING");

        // فیلتر وضعیت
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/wallet/withdrawals?status=PENDING")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["items"].as_array().map(Vec::len), Some(1));

        // بدون کلید ادمین — ۴۰۱
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/wallet/withdrawals")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }
}
