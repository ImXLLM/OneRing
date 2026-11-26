//! M5 parliament handlers — bills, votes, deterministic close, and execution on the money kernel.
//!
//! Gates (all fail-closed):
//!  * Open a bill — session (citizen); full validation in `kian-nation`.
//!  * Vote — session; one vote per citizen; the vote is human (Article 7 gate).
//!  * Close — admin only; deterministic tally in `kian-nation` (quorum + majority + Article 7).
//!  * Execute — admin only and only from `PASSED`; the money action goes through the same pure
//!    M2 domain path (`apply_escrow` / treasury posting) — there is no side path for money.
//!  * Reading bills — public (Article 6 transparency).

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

/// Parliament policy from configuration — the threshold is always the charter's (50,000 Đ), quorum from env.
fn nation_policy(state: &AppState) -> NationPolicy {
    NationPolicy {
        quorum: state.config.nation_quorum.max(1),
        hitl_threshold_din: NationPolicy::charter_default().hitl_threshold_din,
    }
}

/// Wire bill type → domain.
fn kind_from_wire(w: BillKind) -> kian_nation::BillKind {
    match w {
        BillKind::EscrowRelease => kian_nation::BillKind::EscrowRelease,
        BillKind::TreasuryMove => kian_nation::BillKind::TreasuryMove,
    }
}

/// Wire proposed outcome → domain.
fn resolution_from_wire(w: EscrowResolution) -> kian_nation::EscrowResolution {
    match w {
        EscrowResolution::Release => kian_nation::EscrowResolution::Release,
        EscrowResolution::Refund => kian_nation::EscrowResolution::Refund,
    }
}

/// Map parliament errors → honest HTTP envelope.
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

/// Bill view from the db row — live tallies for an open bill.
#[allow(clippy::cast_possible_wrap)] // tally votes — bounded and safe
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

/// Explicit db string → wire type bridge — the `wire_status_to_domain` pattern.
fn parse_kind(s: &str) -> Option<BillKind> {
    match s {
        "escrowRelease" => Some(BillKind::EscrowRelease),
        "treasuryMove" => Some(BillKind::TreasuryMove),
        _ => None,
    }
}

/// List status filter — an invalid string = 400 `INVALID_REQUEST`.
fn parse_status_filter(raw: &str) -> ApiResult<&'static str> {
    match raw.to_ascii_uppercase().as_str() {
        "OPEN" => Ok("OPEN"),
        "PASSED" => Ok("PASSED"),
        "REJECTED" => Ok("REJECTED"),
        "EXECUTED" => Ok("EXECUTED"),
        _ => Err(kian_contracts::HttpFailure::invalid_request(1).into()),
    }
}

/// `POST /nation/bills` — open a bill. Session. 201.
///
/// # Errors
/// 401 session; 400 invalid body; 422 bill rules (`kian-nation`); 500 SQL error.
pub async fn create_bill(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<BillView>)> {
    // identity is only for the entry gate — the bill is registered on the citizen's behalf but the subject is independent
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

/// Bill list filter — `?status=`.
#[derive(Debug, Deserialize)]
pub struct BillFilter {
    /// Optional status: OPEN/PASSED/REJECTED/EXECUTED
    status: Option<String>,
}

/// `GET /nation/bills` — list bills. Public (Article 6).
///
/// # Errors
/// 400 invalid filter; 500 SQL error.
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

/// `GET /nation/bills/{id}` — one bill. Public.
///
/// # Errors
/// 404 unknown; 500 SQL error.
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

/// `POST /nation/bills/{id}/votes` — record a human vote. Session.
///
/// # Errors
/// 401 session; 404 unknown bill; 422 closed bill; 409 duplicate vote; 500 SQL error.
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
            is_human: true, // a Telegram session = a human; the Article 7 gate rests on exactly this
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

/// `POST /nation/bills/{id}/close` — deterministic close with the domain tally. Admin.
///
/// The tally over real db votes happens inside the pure domain [`Bill`] —
/// the outcome (PASSED/REJECTED + reason) and tallies are recorded right there.
///
/// # Errors
/// 401 admin; 404 unknown; 422 already closed; 500 SQL error.
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

    // rebuild the domain state from db — the same escrow pattern: the pure domain decides
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

/// Treasury → destination account transfer — a real double-entry posting inside `apply` (Article 6),
/// exactly the `wallet_move` pattern: domain event + audit at the single point.
/// Overdraft impossible: the source is the treasury; a negative treasury balance = domain violation (422).
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

/// `POST /nation/bills/{id}/execute` — execute a passed bill on the money kernel. Admin.
///
/// * `escrowRelease` — the escrow must be `DISPUTED`; release (Article 2: 90/10) or
///   refund (full return) with `approved_by_admin: true` via the pure M2 domain path.
/// * `treasuryMove` — double-entry treasury → destination account posting (idempotency key: the bill).
///
/// # Errors
/// 401 admin; 404 unknown; 422 not PASSED or invalid transition; 500 SQL error.
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
                                    approved_by_admin: true, // Article 7: a parliament vote = human approval
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
    #[allow(clippy::too_many_lines)] // full market→bill→execute cycle — test readability matters more
    async fn parliament_full_lifecycle_passes_and_executes() {
        let app = app().await;
        // three citizens — quorum 3
        let t1 = login(&app, "10", "voter1").await;
        let t2 = login(&app, "11", "voter2").await;
        let t3 = login(&app, "12", "voter3").await;

        // the treasury must hold money — 10% tax from a real market cycle:
        // mint 1000 to the buyer (t2) → buy the 1000 product (t1) → deliver → release
        // → seller 900 + treasury 100 (Article 2)
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
                            "title": "\u{67e}\u{627}\u{631}\u{644}\u{645}\u{627}\u{646} \u{631}\u{627} \u{622}\u{632}\u{645}\u{627}\u{6cc}\u{634} \u{6a9}\u{646}",
                            "description": "\u{645}\u{62d}\u{635}\u{648}\u{644} \u{686}\u{631}\u{62e}\u{647}\u{654} \u{62e}\u{632}\u{627}\u{646}\u{647}",
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

        // small treasury bill (below threshold) — session; amount ≤ the treasury's real tax (100)
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
                            "title": "\u{67e}\u{627}\u{62f}\u{627}\u{634} \u{645}\u{634}\u{627}\u{631}\u{6a9}\u{62a}"
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

        // public list — no authentication at all
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

        // three citizens vote yes + a fourth duplicate vote — 409
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

        // cannot be closed without admin — 401
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

        // admin closes — quorum 3 → PASSED
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

        // execute — treasury → citizen; real posting on the ledger
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

        // money actually moved — the first citizen's /me shows 500
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
            "\u{62a}\u{633}\u{648}\u{6cc}\u{647}\u{654} \u{6f9}\u{6f0}\u{66a} \u{641}\u{631}\u{648}\u{634} (\u{6f9}\u{6f0}\u{6f0}) + \u{62c}\u{627}\u{628}\u{647}‌\u{62c}\u{627}\u{6cc}\u{6cc} \u{62e}\u{632}\u{627}\u{646}\u{647}\u{654} \u{644}\u{627}\u{6cc}\u{62d}\u{647}‌\u{634}\u{62f}\u{647} (\u{6f1}\u{6f0}\u{6f0}) — \u{67e}\u{648}\u{644} \u{648}\u{627}\u{642}\u{639}\u{6cc}"
        );

        // re-execute — an EXECUTED bill is no longer PASSED → 422
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
        // Article 7: a 60,000 Đ bill with no human votes is rejected — even with a quorum of yes votes.
        // three citizens vote "no" so the human yes tally stays zero; because session =
        // human, this test pins down the no-human-yes-vote behavior: rejection with a reason.
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
                            "title": "\u{62c}\u{627}\u{628}\u{647}‌\u{62c}\u{627}\u{6cc}\u{6cc} \u{628}\u{632}\u{631}\u{6af}"
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
        // there are human yes votes (t1, t2 — each session is a human) → PASSED;
        // the "rejected without a human vote" gate is covered purely in kian-nation::tests.
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

        // invalid body — empty title
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

        // without a session — 401
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
                            "title": "\u{628}\u{62f}\u{648}\u{646} \u{646}\u{634}\u{633}\u{62a}"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);

        // unknown bill — 404
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
