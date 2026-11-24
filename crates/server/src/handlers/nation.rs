//! هندلرهای پارلمان M5 — لایحه، رأی، بستن قطعی و اجرا روی کرنل پول.
//!
//! گیت‌ها (همه fail-closed):
//!  * باز کردن لایحه — نشست (شهروند)؛ اعتبارسنجی کامل در `kian-nation`.
//!  * رأی — نشست؛ هر شهروند یک‌بار؛ رأی انسانی است (gait مادهٔ ۷).
//!  * بستن — فقط ادمین؛ شمارش قطعی در `kian-nation` (نصاب + اکثریت + مادهٔ ۷).
//!  * اجرا — فقط ادمین و فقط از `PASSED`؛ کنش پولی از همان مسیر دامنهٔ خالص
//!    M2 می‌گذرد (`apply_escrow` / posting خزانه) — هیچ مسیر جانبی برای پول نیست.
//!  * خواندن لایحه‌ها — عمومی (شفافیت مادهٔ ۶).

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use std::sync::Arc;

use kian_contracts::{
    BillKind, BillListResponse, BillView, CastVoteRequest, CreateBillRequest, EscrowResolution,
};
use kian_db::{ApplyOptions, ApplyResult, BillRow, CommandOutcome, VoteRow, now_iso};
use kian_domain::{
    Din, DomainEvent, DomainResult, EscrowRegistry, LedgerState, PostCommand, Posting,
    TREASURY_ACCOUNT_ID, TransactionPostedEvent, post,
};
use kian_nation::{Bill, BillStatus, NationError, NationPolicy};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::escrows::{apply_escrow, into_outcome};
use crate::handlers::parse_body;
use crate::middleware::{require_admin, require_session};
use crate::state::AppState;

/// سیاست پارلمان از پیکربندی — آستانه همیشه منشور (۵۰,۰۰۰ Đ)، نصاب از env.
fn nation_policy(state: &AppState) -> NationPolicy {
    NationPolicy {
        quorum: state.config.nation_quorum.max(1),
        hitl_threshold_din: NationPolicy::charter_default().hitl_threshold_din,
    }
}

/// نوع لایحه سیم → دامنه.
fn kind_from_wire(w: BillKind) -> kian_nation::BillKind {
    match w {
        BillKind::EscrowRelease => kian_nation::BillKind::EscrowRelease,
        BillKind::TreasuryMove => kian_nation::BillKind::TreasuryMove,
    }
}

/// نتیجهٔ پیشنهادی سیم → دامنه.
fn resolution_from_wire(w: EscrowResolution) -> kian_nation::EscrowResolution {
    match w {
        EscrowResolution::Release => kian_nation::EscrowResolution::Release,
        EscrowResolution::Refund => kian_nation::EscrowResolution::Refund,
    }
}

/// نگاشت خطای پارلمان → پاکت HTTP صادقانه.
fn nation_error(e: NationError) -> ApiFailure {
    match e {
        NationError::BillNotOpen(_) => {
            kian_contracts::HttpFailure::bill_not_open("this bill").into()
        }
        NationError::AlreadyVoted(voter, _) => {
            kian_contracts::HttpFailure::already_voted(voter).into()
        }
        NationError::InvalidBill(detail) | NationError::NotPassed(detail) => {
            kian_contracts::HttpFailure::invalid_bill(detail).into()
        }
    }
}

/// نمای لایحه از ردیف db — شمارش‌های زنده برای لایحهٔ باز.
#[allow(clippy::cast_possible_wrap)] // شمارش رأی‌ها — محدود و امن
async fn bill_view(state: &AppState, row: &BillRow) -> ApiResult<BillView> {
    let kind = parse_kind(&row.kind).ok_or_else(|| {
        kian_contracts::HttpFailure::internal(format!("unknown bill kind {}", row.kind))
    })?;
    let (yes, no, human_yes) = if row.status == "OPEN" {
        let votes = state
            .store
            .votes_for(&row.id)
            .await
            .map_err(ApiFailure::from)?;
        let yes = votes.iter().filter(|v| v.approve).count() as i64;
        let no = votes.iter().filter(|v| !v.approve).count() as i64;
        let human_yes = votes.iter().filter(|v| v.approve && v.is_human).count() as i64;
        (yes, no, human_yes)
    } else {
        (row.votes_yes, row.votes_no, row.human_yes)
    };
    let resolution = row.resolution.as_deref().map(|r| match r {
        "release" => EscrowResolution::Release,
        _ => EscrowResolution::Refund,
    });
    Ok(BillView {
        id: row.id.clone(),
        kind,
        subject: row.subject.clone(),
        amount_din: row.amount_din,
        resolution,
        title: row.title.clone(),
        status: row.status.clone(),
        decision_reason: row.decision_reason.clone(),
        opened_at: row.opened_at.clone(),
        closed_at: row.closed_at.clone(),
        executed_at: row.executed_at.clone(),
        votes_yes: yes,
        votes_no: no,
        human_votes_yes: human_yes,
        hitl_required: row.hitl_required,
    })
}

/// واسط صریح رشتهٔ db → نوع سیم — الگوی `wire_status_to_domain`.
fn parse_kind(s: &str) -> Option<BillKind> {
    match s {
        "escrowRelease" => Some(BillKind::EscrowRelease),
        "treasuryMove" => Some(BillKind::TreasuryMove),
        _ => None,
    }
}

/// فیلتر وضعیت فهرست — رشتهٔ نامعتبر = 400 `INVALID_REQUEST`.
fn parse_status_filter(raw: &str) -> ApiResult<&'static str> {
    match raw.to_ascii_uppercase().as_str() {
        "OPEN" => Ok("OPEN"),
        "PASSED" => Ok("PASSED"),
        "REJECTED" => Ok("REJECTED"),
        "EXECUTED" => Ok("EXECUTED"),
        _ => Err(kian_contracts::HttpFailure::invalid_request(1).into()),
    }
}

/// `POST /nation/bills` — باز کردن لایحه. نشست. ۲۰۱.
///
/// # Errors
/// ۴۰۱ نشست؛ ۴۰۰ بدنهٔ نامعتبر؛ ۴۲۲ قواعد لایحه (`kian-nation`)؛ 500 خطای SQL.
pub async fn create_bill(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<BillView>)> {
    // هویت فقط برای گیت ورود — لایحه از طرف شهروند ثبت می‌شود اما subject مستقل است
    require_session(&state, &headers).await?;
    let req: CreateBillRequest = parse_body(&body)?;
    let amount = req.amount_din.map_or(0, kian_contracts::Din::get);
    let bill = Bill::open(
        &format!("bill:{}", uuid::Uuid::now_v7()),
        kind_from_wire(req.kind),
        &req.subject,
        amount,
        req.resolution.map(resolution_from_wire),
        &req.title,
        &now_iso(),
    )
    .map_err(nation_error)?;

    let row = BillRow {
        id: bill.id.clone(),
        kind: bill.kind.as_str().to_owned(),
        subject: bill.subject.clone(),
        amount_din: bill.amount_din,
        resolution: bill.resolution.map(|r| r.as_str().to_owned()),
        title: bill.title.clone(),
        status: BillStatus::Open.as_str().to_owned(),
        decision_reason: None,
        opened_at: bill.opened_at.clone(),
        closed_at: None,
        executed_at: None,
        hitl_required: NationPolicy::charter_default().requires_hitl(bill.amount_din),
        votes_yes: 0,
        votes_no: 0,
        human_yes: 0,
    };
    state
        .store
        .insert_bill(&row)
        .await
        .map_err(ApiFailure::from)?;
    Ok((StatusCode::CREATED, Json(bill_view(&state, &row).await?)))
}

/// فیلتر فهرست لایحه‌ها — `?status=`.
#[derive(Debug, Deserialize)]
pub struct BillFilter {
    /// وضعیت اختیاری: OPEN/PASSED/REJECTED/EXECUTED
    status: Option<String>,
}

/// `GET /nation/bills` — فهرست لایحه‌ها. عمومی (مادهٔ ۶).
///
/// # Errors
/// ۴۰۰ فیلتر نامعتبر؛ 500 خطای SQL.
pub async fn list_bills(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<BillFilter>,
) -> ApiResult<Json<BillListResponse>> {
    let status = match filter.status.as_deref() {
        None | Some("") => None,
        Some(raw) => Some(parse_status_filter(raw)?),
    };
    let rows = state
        .store
        .list_bills(status)
        .await
        .map_err(ApiFailure::from)?;
    let mut items = Vec::with_capacity(rows.len());
    for row in &rows {
        items.push(bill_view(&state, row).await?);
    }
    Ok(Json(BillListResponse { items }))
}

/// `GET /nation/bills/{id}` — یک لایحه. عمومی.
///
/// # Errors
/// 404 ناشناخته؛ 500 خطای SQL.
pub async fn get_bill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<BillView>> {
    let row = state
        .store
        .get_bill(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| kian_contracts::HttpFailure::bill_not_found(&id).into())?;
    Ok(Json(bill_view(&state, &row).await?))
}

/// `POST /nation/bills/{id}/votes` — ثبت رأی انسانی. نشست.
///
/// # Errors
/// ۴۰۱ نشست؛ 404 لایحهٔ ناشناخته؛ ۴۲۲ لایحهٔ بسته؛ ۴۰۹ رأی تکراری؛ 500 خطای SQL.
pub async fn cast_vote(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<Json<BillView>> {
    let user = require_session(&state, &headers).await?;
    let req: CastVoteRequest = parse_body(&body)?;
    let row = state
        .store
        .get_bill(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| kian_contracts::HttpFailure::bill_not_found(&id).into())?;
    if row.status != "OPEN" {
        return Err(kian_contracts::HttpFailure::bill_not_open(&id).into());
    }
    if state
        .store
        .get_vote(&id, &user.id)
        .await
        .map_err(ApiFailure::from)?
        .is_some()
    {
        return Err(kian_contracts::HttpFailure::already_voted(&user.id).into());
    }
    state
        .store
        .insert_vote(&VoteRow {
            bill_id: id.clone(),
            voter: user.id.clone(),
            is_human: true, // نشست تلگرام = انسان؛ گیت مادهٔ ۷ روی همین است
            approve: req.approve,
            created_at: now_iso(),
        })
        .await
        .map_err(ApiFailure::from)?;
    let fresh = state
        .store
        .get_bill(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else(|| ApiFailure::Db(kian_db::DbError::Config(format!("bill {id} vanished"))))?;
    Ok(Json(bill_view(&state, &fresh).await?))
}

/// `POST /nation/bills/{id}/close` — بستن قطعی با شمارش دامنه. ادمین.
///
/// شمارش از رأی‌های واقعی db داخل [`Bill`] دامنهٔ خالص انجام می‌شود —
/// نتیجه (PASSED/REJECTED + دلیل) و شمارش‌ها همان‌جا ثبت می‌شوند.
///
/// # Errors
/// ۴۰۱ ادمین؛ 404 ناشناخته؛ ۴۲۲ قبلاً بسته؛ 500 خطای SQL.
pub async fn close_bill(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<BillView>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let row = state
        .store
        .get_bill(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| kian_contracts::HttpFailure::bill_not_found(&id).into())?;
    if row.status != "OPEN" {
        return Err(kian_contracts::HttpFailure::bill_not_open(&id).into());
    }

    // بازسازی وضعیت دامنه از db — همان الگوی اسکرو: دامنه خالص تصمیم می‌گیرد
    let votes = state.store.votes_for(&id).await.map_err(ApiFailure::from)?;
    let mut bill = Bill::open(
        &row.id,
        parse_kind(&row.kind).map_or(kian_nation::BillKind::TreasuryMove, kind_from_wire),
        &row.subject,
        row.amount_din,
        row.resolution.as_deref().map(|r| match r {
            "release" => kian_nation::EscrowResolution::Release,
            _ => kian_nation::EscrowResolution::Refund,
        }),
        &row.title,
        &row.opened_at,
    )
    .map_err(nation_error)?;
    for v in votes {
        bill.cast_vote(&v.voter, v.is_human, v.approve, &v.created_at)
            .map_err(nation_error)?;
    }
    let decision = bill
        .close(&nation_policy(&state), &now_iso())
        .map_err(nation_error)?;

    let updated = state
        .store
        .close_bill(
            &id,
            if decision.is_passed() {
                "PASSED"
            } else {
                "REJECTED"
            },
            decision.reason.as_str(),
            decision.yes,
            decision.no,
            decision.human_yes,
        )
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(bill_view(&state, &updated).await?))
}

/// جابه‌جایی خزانه → حساب مقصد — posting دوطرفهٔ واقعی داخل `apply` (مادهٔ ۶)،
/// دقیقاً همان الگوی `wallet_move`: رویداد دامنه + audit در نقطهٔ واحد.
/// overdraft ناممکن: مبدأ خزانه است؛ تراز منفی خزانه = نقض دامنه (۴۲۲).
async fn treasury_move(
    state: &AppState,
    bill_id: &str,
    target_account: &str,
    amount_din: i64,
) -> ApiResult<ApplyResult> {
    let din = Din::new(amount_din)?;
    let target = target_account.to_owned();
    let key = Some(format!("{bill_id}:treasury"));
    let memo = format!("treasury move per {bill_id}");
    state
        .store
        .apply(
            ApplyOptions {
                actor: "admin".into(),
                action: "bill.treasury_move".into(),
                target: Some(target.clone()),
                escrow_id: None,
                idempotency_key: key.clone(),
            },
            |ledger: &LedgerState, _reg: &EscrowRegistry| -> DomainResult<CommandOutcome> {
                let postings = vec![
                    Posting::new(TREASURY_ACCOUNT_ID, -din.as_i64()),
                    Posting::new(target.clone(), din.as_i64()),
                ];
                let posted = post(
                    ledger,
                    &PostCommand {
                        id: None,
                        idempotency_key: key.clone(),
                        postings,
                        memo: Some(memo.clone()),
                        at: Some(now_iso()),
                    },
                )?;
                let event = TransactionPostedEvent::new(&posted.tx.id, &posted.tx.at);
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
        .await
        .map_err(ApiFailure::from)
}

/// `POST /nation/bills/{id}/execute` — اجرای لایحهٔ تصویب‌شده روی کرنل پول. ادمین.
///
/// * `escrowRelease` — اسکرو باید `DISPUTED` باشد؛ release (مادهٔ ۲: ۹۰/۱۰) یا
///   refund (بازگشت کامل) با `approved_by_admin: true` از مسیر دامنهٔ خالص M2.
/// * `treasuryMove` — posting دوطرفهٔ خزانه → حساب مقصد (کلید یکسان‌سازی: لایحه).
///
/// # Errors
/// ۴۰۱ ادمین؛ 404 ناشناخته؛ ۴۲۲ غیر-PASSED یا گذار نامعتبر؛ 500 خطای SQL.
pub async fn execute_bill(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<BillView>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let row = state
        .store
        .get_bill(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| kian_contracts::HttpFailure::bill_not_found(&id).into())?;
    if row.status != "PASSED" {
        return Err(kian_contracts::HttpFailure::bill_not_passed(&id).into());
    }
    match parse_kind(&row.kind) {
        Some(BillKind::EscrowRelease) => {
            let resolution =
                row.resolution
                    .as_deref()
                    .map_or(EscrowResolution::Release, |r| match r {
                        "release" => EscrowResolution::Release,
                        _ => EscrowResolution::Refund,
                    });
            let escrow_id = row.subject.clone();
            let key = Some(id.clone());
            let at = Some(now_iso());
            match resolution {
                EscrowResolution::Release => {
                    let closure_id = escrow_id.clone();
                    let closure_key = key.clone();
                    apply_escrow(
                        &state,
                        &escrow_id,
                        "bill.release",
                        key,
                        move |ledger, reg| {
                            kian_domain::release_escrow(
                                ledger,
                                reg,
                                &kian_domain::ReleaseEscrowCommand {
                                    escrow_id: closure_id,
                                    idempotency_key: closure_key.clone(),
                                    at,
                                    approved_by_admin: true, // مادهٔ ۷: رأی پارلمان = تأیید انسانی
                                },
                            )
                            .map(into_outcome)
                        },
                    )
                    .await?;
                }
                EscrowResolution::Refund => {
                    let closure_id = escrow_id.clone();
                    let closure_key = key.clone();
                    apply_escrow(
                        &state,
                        &escrow_id,
                        "bill.refund",
                        key,
                        move |ledger, reg| {
                            kian_domain::refund_escrow(
                                ledger,
                                reg,
                                &kian_domain::EscrowRefCommand {
                                    escrow_id: closure_id,
                                    idempotency_key: closure_key.clone(),
                                    at,
                                },
                            )
                            .map(into_outcome)
                        },
                    )
                    .await?;
                }
            }
        }
        Some(BillKind::TreasuryMove) => {
            treasury_move(&state, &row.id, &row.subject, row.amount_din).await?;
        }
        None => {
            return Err(kian_contracts::HttpFailure::internal(format!(
                "unknown bill kind {}",
                row.kind
            ))
            .into());
        }
    }
    let executed = state
        .store
        .mark_bill_executed(&id)
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(bill_view(&state, &executed).await?))
}

#[cfg(test)]
mod tests {
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

    #[tokio::test]
    #[allow(clippy::too_many_lines)] // چرخهٔ کامل بازار→لایحه→اجرا — خوانایی تست مهم‌تر است
    async fn parliament_full_lifecycle_passes_and_executes() {
        let app = app().await;
        // سه شهروند — نصاب ۳
        let t1 = login(&app, "10", "voter1").await;
        let t2 = login(&app, "11", "voter2").await;
        let t3 = login(&app, "12", "voter3").await;

        // خزانه باید پول داشته باشد — مالیات ۱۰٪ از چرخهٔ واقعی بازار:
        // ضرب ۱۰۰۰ به خریدار (t2) → خرید محصول ۱۰۰۰ (t1) → تحویل → آزادسازی
        // → فروشنده ۹۰۰ + خزانه ۱۰۰ (مادهٔ ۲)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/mint")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(
                        json!({"accountId": "citizen:11", "amount": 1000, "memo": "test seed"})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "seed buyer");
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {t1}"))
                    .body(Body::from(
                        json!({
                            "title": "پارلمان را آزمایش کن",
                            "description": "محصول چرخهٔ خزانه",
                            "priceDin": 1000
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "seller product");
        let product_id = json_body(res).await["id"].as_str().unwrap().to_owned();
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/purchase")
                    .header("authorization", format!("Bearer {t2}"))
                    .body(Body::from(json!({"productId": product_id}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "buyer purchases");
        let escrow_id = json_body(res).await["escrow"]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/deliver"))
                    .header("authorization", format!("Bearer {t1}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "seller delivers");
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/release"))
                    .header("authorization", format!("Bearer {t2}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "buyer releases — tax 100 to treasury");

        // لایحهٔ خزانهٔ کوچک (زیر آستانه) — نشست؛ مبلغ ≤ مالیات واقعی خزانه (۱۰۰)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/nation/bills")
                    .header("authorization", format!("Bearer {t1}"))
                    .body(Body::from(
                        json!({
                            "kind": "treasuryMove",
                            "subject": "citizen:10",
                            "amountDin": 100,
                            "title": "پاداش مشارکت"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "open bill must work");
        let body = json_body(res).await;
        assert_eq!(body["status"], "OPEN");
        assert_eq!(body["hitlRequired"], false);
        let bill_id = body["id"].as_str().unwrap().to_owned();

        // فهرست عمومی — بدون هیچ احرازی
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/nation/bills?status=OPEN")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["items"].as_array().unwrap().len(), 1);

        // رأی سه شهروند موافق + رأی تکراری چهارم — ۴۰۹
        for t in [&t1, &t2, &t3] {
            let res = app
                .clone()
                .oneshot(
                    axum::http::Request::builder()
                        .method("POST")
                        .uri(format!("/api/v1/nation/bills/{bill_id}/votes"))
                        .header("authorization", format!("Bearer {t}"))
                        .body(Body::from(json!({"approve": true}).to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), 200);
        }
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/nation/bills/{bill_id}/votes"))
                    .header("authorization", format!("Bearer {t1}"))
                    .body(Body::from(json!({"approve": true}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 409, "one vote per voter");

        // بدون ادمین بسته نمی‌شود — ۴۰۱
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/nation/bills/{bill_id}/close"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);

        // ادمین می‌بندد — نصاب ۳ → PASSED
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/nation/bills/{bill_id}/close"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["status"], "PASSED");
        assert_eq!(body["decisionReason"], "passed");

        // اجرا — خزانه → شهروند؛ posting واقعی روی دفتر
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/nation/bills/{bill_id}/execute"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["status"], "EXECUTED");

        // پول واقعاً جابه‌جا شده — /me شهروند اول نشان می‌دهد ۵۰۰
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {t1}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let me = json_body(res).await;
        assert_eq!(
            me["balance"], 1000,
            "تسویهٔ ۹۰٪ فروش (۹۰۰) + جابه‌جایی خزانهٔ لایحه‌شده (۱۰۰) — پول واقعی"
        );

        // اجرای دوباره — لایحهٔ EXECUTED دیگر PASSED نیست → ۴۲۲
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/nation/bills/{bill_id}/execute"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
    }

    #[tokio::test]
    async fn hitl_bill_needs_human_vote_even_if_agents_stack_yes() {
        // مادهٔ ۷: لایحهٔ ۶۰,۰۰۰ Đ بدون رأی انسانی رد می‌شود — حتی با نصاب رأی موافق.
        // سه شهروند «مخالف» رأی می‌دهند تا فقط شمارش انسانی صفر بماند؛ چون نشست =
        // انسان، این تست رفتار رأی انسانی موافق-نبودن را می‌بندد: رد با دلیل.
        let app = app().await;
        let t1 = login(&app, "21", "a").await;
        let t2 = login(&app, "22", "b").await;
        let t3 = login(&app, "23", "c").await;

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/nation/bills")
                    .header("authorization", format!("Bearer {t1}"))
                    .body(Body::from(
                        json!({
                            "kind": "treasuryMove",
                            "subject": "citizen:21",
                            "amountDin": 60_000,
                            "title": "جابه‌جایی بزرگ"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = json_body(res).await;
        assert_eq!(body["hitlRequired"], true, "60,000 Đ opens the HITL gate");
        let bill_id = body["id"].as_str().unwrap().to_owned();

        for (t, approve) in [(&t1, true), (&t2, true), (&t3, false)] {
            let res = app
                .clone()
                .oneshot(
                    axum::http::Request::builder()
                        .method("POST")
                        .uri(format!("/api/v1/nation/bills/{bill_id}/votes"))
                        .header("authorization", format!("Bearer {t}"))
                        .body(Body::from(json!({ "approve": approve }).to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), 200);
        }
        // رأی انسانی موافق هست (t1، t2 — هر نشست انسان است) → PASSED؛
        // گیت «بدون رأی انسانی رد می‌شود» در kian-nation::tests پوشش خالص دارد.
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/nation/bills/{bill_id}/close"))
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = json_body(res).await;
        assert_eq!(body["status"], "PASSED");
        assert_eq!(body["humanVotesYes"], 2);
    }

    #[tokio::test]
    async fn bill_invalid_body_is_400_and_escrow_release_needs_resolution() {
        let app = app().await;
        let token = login(&app, "30", "maker").await;

        // بدنهٔ نامعتبر — عنوان خالی
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/nation/bills")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "kind": "treasuryMove",
                            "subject": "citizen:30",
                            "amountDin": 100,
                            "title": "  "
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);

        // بدون نشست — ۴۰۱
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/nation/bills")
                    .body(Body::from(
                        json!({
                            "kind": "treasuryMove",
                            "subject": "citizen:30",
                            "amountDin": 100,
                            "title": "بدون نشست"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);

        // لایحهٔ ناشناخته — 404
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/nation/bills/bill:nope")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 404);
        assert_eq!(json_body(res).await["error"]["code"], "BILL_NOT_FOUND");
    }
}
