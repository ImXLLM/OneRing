//! M3 market handlers — products, atomic purchase over the 8-state escrow, and both parties' actions.
//!
//! a purchase = `create_escrow` + `fund_escrow` as "one" atomic command inside the same `apply`:
//! either both happen or neither — buyer funds lock only upon FUNDED.
//! subsequent actions come only from the escrow's authorized side: the seller delivers, the buyer
//! releases (the Article 7 HITL threshold is protected in the domain).
//! the delivery content (`deliveryPayload`) never goes over the public wire.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::sync::Arc;

use kian_contracts::{
    CreateProductRequest, DeliverEscrowRequest, DisputeEscrowRequest, EscrowActionResponse,
    EscrowDeliveryView, MaironConfirmResponse, MAX_DELIVERY_PAYLOAD_BYTES, MAX_PREVIEW_HTML_BYTES,
    MyMarketResponse, ProductListResponse, ProductRatingSummaryResponse, ProductReviewSubmitRequest,
    ProductReviewView, ProductSkillPackageResponse, ProductView, PurchaseRequest, PurchaseResponse,
    ReleaseEscrowRequest,
};
use kian_db::{now_iso, ApplyOptions, CommandOutcome, HitlRow, ProductFilter, ProductReviewRow, ProductRow};
use kian_domain::{
    confirm_and_publish_product, evaluate_user_product_with_mairon, submit_product_review,
    summarize_product_ratings, MaironReaPillarScores, ProductReview,
};
use kian_domain::mairon_rea_and_reviews::MARKETPLACE_APPROVAL_THRESHOLD;
use kian_domain::{
    create_escrow_with_affiliate, deliver_escrow, dispute_escrow, fund_escrow, release_escrow,
    AccountKind, CreateEscrowCommand, DeliverEscrowCommand, DisputeEscrowCommand, DomainError,
    DomainErrorCode, DomainEvent, DomainResult, EscrowRefCommand, EscrowRegistry, EscrowStatus,
    LedgerState, ReleaseEscrowCommand, DEFAULT_AFFILIATE_BPS, MAX_AFFILIATE_BPS,
};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::{parse_body, safe_prefixed_wire_id};
use crate::middleware::require_session;
use crate::state::AppState;
use crate::views::{escrow_status_wire, escrow_view};

/// Default buy-market escrow deadline — 72 hours.
const PURCHASE_TTL_HOURS: i64 = 72;

/// Normalize the referrer ID to the citizen account format (`citizen:{tg_id}`).
///
/// Input may be a numeric `tg_id` (`"12345"`) or `"citizen:12345"`;
/// invalid/empty inputs become `None` so the customer's purchase never stalls.
fn normalize_referrer_account(raw: Option<&str>) -> Option<String> {
    let s = raw?.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix("citizen:") {
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            return Some(s.to_owned());
        }
        return None;
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
        return Some(format!("citizen:{s}"));
    }
    None
}

/// Wire product view from the db row — without `deliveryPayload` (never public).
#[must_use]
fn product_view(p: &ProductRow) -> ProductView {
    ProductView {
        id: p.id.clone(),
        seller_id: p.seller_id.clone(),
        title: p.title.clone(),
        description: p.description.clone(),
        price_din: crate::views::wire_din(p.price_din),
        category: p.category.clone(),
        preview_html: p.preview_html.clone(),
        affiliate_bps: p.affiliate_bps,
        artifact_kind: p.artifact_kind.clone(),
        active: p.active,
        created_at: p.created_at.clone(),
        mairon_score: p.mairon_score,
        mairon_badge: p.mairon_badge.clone(),
    }
}

/// `POST /market/products` — register a product on behalf of the session user. 201.
///
/// # Errors
/// 401 missing session; 400 invalid body (lengths/price/HTML volume); 422 `BPS_OUT_OF_RANGE`;
/// 500 SQL error.
pub async fn create_product(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<ProductView>)> {
    let user = require_session(&state, &headers).await?;
    let req: CreateProductRequest = parse_body(&body)?;
    let title_len = req.title.trim().chars().count();
    let desc_len = req.description.trim().chars().count();
    let artifact_kind = req
        .artifact_kind
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("html")
        .to_owned();
    let kind_ok = (1..=32).contains(&artifact_kind.len())
        && artifact_kind
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !(1..=120).contains(&title_len)
        || !(1..=2000).contains(&desc_len)
        || req.price_din.get() < 1
        || !kind_ok
        || req.category.as_ref().is_some_and(|c| c.chars().count() > 40)
        || req.preview_html.as_ref().is_some_and(|p| p.len() > MAX_PREVIEW_HTML_BYTES)
        || req.delivery_payload.as_ref().is_some_and(|d| d.len() > MAX_DELIVERY_PAYLOAD_BYTES)
    {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }

    let affiliate_bps = req.affiliate_bps.unwrap_or(DEFAULT_AFFILIATE_BPS);
    if affiliate_bps > MAX_AFFILIATE_BPS {
        return Err(DomainError::new(
            DomainErrorCode::BpsOutOfRange,
            format!("affiliate_bps must be in [0, {MAX_AFFILIATE_BPS}], got {affiliate_bps}"),
        )
        .into());
    }

    let preview_html = req.preview_html.unwrap_or_default();
    let row = ProductRow {
        id: format!("prod:{}", uuid::Uuid::now_v7()),
        seller_id: user.account_id.clone(),
        title: req.title,
        description: req.description,
        category: req.category,
        price_din: req.price_din.get(),
        preview_html: preview_html.clone(),
        affiliate_bps,
        artifact_kind,
        delivery_payload: req.delivery_payload,
        active: true,
        created_at: now_iso(),
        mairon_score: 0,
        mairon_badge: String::new(),
        origin_locale: kian_hermes::arda_fishkal_engine::detect_origin_locale(&preview_html)
            .to_owned(),
        supported_locales_json: "[\"en\",\"fa\",\"ar\"]".to_owned(),
        skill_md: None,
        skill_json: None,
        arda_last_localized_at: None,
    };
    state.store.insert_product(&row).await.map_err(ApiFailure::from)?;
    Ok((StatusCode::CREATED, Json(product_view(&row))))
}

/// `GET /market/products` — active products for everyone (Article 6 transparency; no `deliveryPayload`).
///
/// # Errors
/// SQL error.
pub async fn list_products(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ProductListResponse>> {
    let filter = ProductFilter { active_only: true, limit: 50, offset: 0, q: None };
    let items = state
        .store
        .list_products(&filter)
        .await
        .map_err(ApiFailure::from)?
        .iter()
        .map(product_view)
        .collect();
    Ok(Json(ProductListResponse { items }))
}

/// `GET /market/products/{id}` — public single-product page for direct links and referrals.
///
/// `deliveryPayload` is never returned on this route (only `previewHtml`).
///
/// # Errors
/// 404 unknown product; 500 SQL error.
pub async fn get_product_one(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<ProductView>> {
    let product = state
        .store
        .get_product(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {id} does not exist"),
            )
            .into()
        })?;
    Ok(Json(product_view(&product)))
}

/// `POST /market/purchase` — atomic purchase: create escrow (with referral support) + lock Din.
///
/// # Errors
/// 401 session; 404 unknown product; 422 inactive product/self-purchase/insufficient balance;
/// 500 SQL error.
#[allow(clippy::too_many_lines)]
pub async fn purchase(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<PurchaseResponse>)> {
    let user = require_session(&state, &headers).await?;
    let req: PurchaseRequest = parse_body(&body)?;

    let product = state
        .store
        .get_product(req.product_id.get())
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {} does not exist", req.product_id.get()),
            )
            .into()
        })?;
    if !product.active {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!("Product {} is not active", product.id),
        )
        .into());
    }
    if product.seller_id == user.account_id {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            "Seller cannot buy their own product",
        )
        .into());
    }

    let escrow_id = format!("escrow:{}", uuid::Uuid::now_v7());
    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(PURCHASE_TTL_HOURS))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let amount = kian_domain::Din::new(product.price_din)?;
    let create_cmd = CreateEscrowCommand {
        id: escrow_id.clone(),
        buyer_id: user.account_id.clone(),
        seller_id: product.seller_id.clone(),
        amount,
        product_ref: Some(product.id.clone()),
        expires_at: Some(expires_at),
        at: Some(now_iso()),
    };
    let fund_cmd =
        EscrowRefCommand { escrow_id: escrow_id.clone(), idempotency_key: None, at: Some(now_iso()) };
    let key = req.idempotency_key.as_ref().map(|k| k.get().to_owned());
    let referrer_candidate = normalize_referrer_account(req.referrer.as_deref());
    let product_affiliate_bps = product.affiliate_bps;

    let result = state
        .store
        .apply(
            ApplyOptions {
                actor: user.account_id.clone(),
                action: "market.purchase".into(),
                target: Some(product.id.clone()),
                escrow_id: Some(escrow_id.clone()),
                idempotency_key: key,
            },
            move |ledger, registry| -> DomainResult<CommandOutcome> {
                // registered only if a valid referrer exists in the ledger and the account is a User;
                // otherwise the customer's purchase never stalls (self-referral is neutralized in the domain too).
                let effective_referrer = referrer_candidate.as_ref().and_then(|r| {
                    ledger
                        .accounts
                        .get(r)
                        .filter(|a| a.kind == AccountKind::User)
                        .map(|_| r.clone())
                });
                let affiliate_bps =
                    if effective_referrer.is_some() { product_affiliate_bps } else { 0 };
                // one atomic command: CREATED + FUNDED — both or neither
                let created = create_escrow_with_affiliate(
                    ledger,
                    registry,
                    &create_cmd,
                    effective_referrer,
                    affiliate_bps,
                )?;
                let funded = fund_escrow(&created.ledger, &created.registry, &fund_cmd)?;
                let events = created
                    .events
                    .iter()
                    .chain(funded.events.iter())
                    .cloned()
                    .map(DomainEvent::Escrow)
                    .collect();
                Ok(CommandOutcome {
                    ledger: funded.ledger,
                    tx: funded.tx,
                    escrow: Some(funded.escrow),
                    events,
                })
            },
        )
        .await
        .map_err(ApiFailure::from)?;

    let escrow = result
        .escrow
        .as_ref()
        .ok_or_else(|| ApiFailure::Http(kian_contracts::HttpFailure::internal_no_escrow()))?;
    Ok((
        StatusCode::CREATED,
        Json(PurchaseResponse {
            escrow: escrow_view(escrow),
            tx_id: result.tx.as_ref().map(|t| t.id.clone()),
            replayed: result.replayed,
        }),
    ))
}

/// `GET /market/my` — my products + my orders (as buyer or seller).
///
/// # Errors
/// 401 session; 500 SQL error.
pub async fn my_market(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<MyMarketResponse>> {
    let user = require_session(&state, &headers).await?;
    let account = &user.account_id;

    let products = state
        .store
        .list_products(&ProductFilter { active_only: false, limit: 100, offset: 0, q: None })
        .await
        .map_err(ApiFailure::from)?
        .iter()
        .filter(|p| p.seller_id == *account)
        .map(product_view)
        .collect();

    let orders = state
        .store
        .list_escrows(None)
        .await
        .map_err(ApiFailure::from)?
        .iter()
        .filter(|e| e.buyer_id == *account || e.seller_id == *account)
        .map(escrow_view)
        .collect();

    Ok(Json(MyMarketResponse { products, orders }))
}

/// The escrow action's authorized side — only the owner of that side may transition.
enum EscrowSide {
    /// Deliver — only that escrow's seller
    Seller,
    /// Release — only that escrow's buyer
    Buyer,
}

/// Escrow action from the market side — ownership guard + pure domain execution in `apply`.
async fn escrow_side_action<F>(
    state: &AppState,
    headers: &HeaderMap,
    escrow_id: &str,
    expected: EscrowSide,
    action: &'static str,
    key: Option<String>,
    run: F,
) -> ApiResult<Json<EscrowActionResponse>>
where
    F: FnOnce(&LedgerState, &EscrowRegistry) -> DomainResult<CommandOutcome>,
{
    let user = require_session(state, headers).await?;
    let escrow = state
        .store
        .get_escrow(escrow_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::EscrowNotFound,
                format!("Escrow {escrow_id} does not exist"),
            )
            .into()
        })?;
    let role_ok = match expected {
        EscrowSide::Seller => escrow.seller_id == user.account_id,
        EscrowSide::Buyer => escrow.buyer_id == user.account_id,
    };
    if !role_ok {
        return Err(ApiFailure::Http(kian_contracts::HttpFailure::forbidden_escrow_role()));
    }
    let result = state
        .store
        .apply(
            ApplyOptions {
                actor: user.account_id.clone(),
                action: action.to_owned(),
                target: Some(escrow_id.to_owned()),
                escrow_id: Some(escrow_id.to_owned()),
                idempotency_key: key,
            },
            run,
        )
        .await
        .map_err(ApiFailure::from)?;
    let view = result
        .escrow
        .as_ref()
        .ok_or_else(|| ApiFailure::Http(kian_contracts::HttpFailure::internal_no_escrow()))?;
    Ok(Json(EscrowActionResponse {
        escrow: escrow_view(view),
        tx_id: result.tx.as_ref().map(|t| t.id.clone()),
        replayed: result.replayed,
    }))
}

/// `POST /market/escrows/{id}/deliver` — only that escrow's seller.
///
/// # Errors
/// 401 session; 404 unknown escrow; 403 wrong side; 422 invalid transition.
pub async fn deliver(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<Json<EscrowActionResponse>> {
    let req: DeliverEscrowRequest = parse_body(&body)?;
    let key = req.idempotency_key.as_ref().map(|k| k.get().to_owned());
    let delivery_ref = req.delivery_ref.clone();
    let escrow_id = id.clone();
    let closure_key = key.clone();
    escrow_side_action(
        &state,
        &headers,
        &id,
        EscrowSide::Seller,
        "market.deliver",
        key,
        move |ledger, registry| {
            deliver_escrow(
                ledger,
                registry,
                &DeliverEscrowCommand {
                    escrow_id,
                    idempotency_key: closure_key,
                    at: Some(now_iso()),
                    delivery_ref,
                },
            )
            .map(|o| CommandOutcome {
                ledger: o.ledger,
                tx: o.tx,
                escrow: Some(o.escrow),
                events: o.events.into_iter().map(DomainEvent::Escrow).collect(),
            })
        },
    )
    .await
}

/// `GET /market/escrows/{id}/delivery` — fetch the full content of a paid product.
///
/// Only that escrow's buyer or seller, and only in the paid states
/// (`FUNDED` / `DELIVERED` / `RELEASED`) have access. For
/// instant digital products with a pre-registered `deliveryPayload`, the buyer sees the
/// content immediately in `FUNDED` state and can then `release` or `dispute`.
///
/// # Errors
/// 401 session; 404 unknown escrow or product; 403 unauthorized side; 422 unpaid state.
pub async fn delivery(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<EscrowDeliveryView>> {
    let user = require_session(&state, &headers).await?;
    let escrow = state
        .store
        .get_escrow(&id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::EscrowNotFound,
                format!("Escrow {id} does not exist"),
            )
            .into()
        })?;
    if escrow.buyer_id != user.account_id && escrow.seller_id != user.account_id {
        return Err(ApiFailure::Http(kian_contracts::HttpFailure::forbidden_escrow_role()));
    }
    if !matches!(
        escrow.status,
        EscrowStatus::Funded | EscrowStatus::Delivered | EscrowStatus::Released
    ) {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!("Delivery is not available for escrow in status {}", escrow.status),
        )
        .into());
    }
    let product_id = escrow.product_ref.as_deref().ok_or_else::<ApiFailure, _>(|| {
        DomainError::new(
            DomainErrorCode::ProductNotFound,
            format!("Escrow {id} has no associated product"),
        )
        .into()
    })?;
    let product = state
        .store
        .get_product_with_delivery(product_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {product_id} does not exist"),
            )
            .into()
        })?;
    Ok(Json(EscrowDeliveryView {
        escrow_id: escrow.id,
        product_id: product.id,
        title: product.title,
        artifact_kind: product.artifact_kind,
        state: escrow_status_wire(escrow.status),
        delivery_payload: product.delivery_payload.unwrap_or_default(),
    }))
}

/// `POST /market/escrows/{id}/release` — only that escrow's buyer (HITL threshold in the domain).
///
/// If the linked product has a pre-registered `deliveryPayload` (an instant digital product)
/// and the escrow is still in `FUNDED` state, instant delivery + atomic release happen in the
/// same transaction.
///
/// # Errors
/// 401 session; 404 unknown escrow; 403 wrong side; 422 invalid transition or the
/// 50,000 Đ threshold without admin approval (`REQUIRES_HUMAN_APPROVAL`).
pub async fn release(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<Json<EscrowActionResponse>> {
    let req: ReleaseEscrowRequest = parse_body(&body)?;
    // check for instant-delivery content on the linked product (only for the FUNDED→DELIVERED→RELEASED transitions)
    let instant_delivery = match state.store.get_escrow(&id).await.map_err(ApiFailure::from)? {
        Some(e) if e.status == EscrowStatus::Funded => match e.product_ref.as_deref() {
            Some(pid) => state
                .store
                .get_product_with_delivery(pid)
                .await
                .map_err(ApiFailure::from)?
                .and_then(|p| p.delivery_payload)
                .is_some_and(|d| !d.trim().is_empty()),
            None => false,
        },
        _ => false,
    };
    // a sessioned buyer can never self-declare "admin-approved" —
    // the admin flag arrives only via `POST /escrows/{id}/release` (the X-KIAN-Key guard).
    let key = req.idempotency_key.as_ref().map(|k| k.get().to_owned());
    let escrow_id = id.clone();
    let closure_key = key.clone();
    let result = escrow_side_action(
        &state,
        &headers,
        &id,
        EscrowSide::Buyer,
        "market.release",
        key,
        move |ledger, registry| {
            let now = now_iso();
            let (cur_ledger, cur_registry, mut events) = if instant_delivery
                && registry
                    .escrows
                    .get(&escrow_id)
                    .is_some_and(|e| e.status == EscrowStatus::Funded)
            {
                let d = deliver_escrow(
                    ledger,
                    registry,
                    &DeliverEscrowCommand {
                        escrow_id: escrow_id.clone(),
                        idempotency_key: None,
                        at: Some(now.clone()),
                        delivery_ref: Some("instant:payload".into()),
                    },
                )?;
                let evs = d.events.into_iter().map(DomainEvent::Escrow).collect::<Vec<_>>();
                (d.ledger, d.registry, evs)
            } else {
                (ledger.clone(), registry.clone(), Vec::new())
            };
            let o = release_escrow(
                &cur_ledger,
                &cur_registry,
                &ReleaseEscrowCommand {
                    escrow_id,
                    idempotency_key: closure_key,
                    at: Some(now),
                    approved_by_admin: false,
                },
            )?;
            events.extend(o.events.into_iter().map(DomainEvent::Escrow));
            Ok(CommandOutcome {
                ledger: o.ledger,
                tx: o.tx,
                escrow: Some(o.escrow),
                events,
            })
        },
    )
    .await;

    // if the escrow amount exceeds the Article 7 HITL threshold, enqueue a human-approval request on HitlRequest
    if let Err(ApiFailure::Domain(ref err)) = result
        && err.error.code == DomainErrorCode::RequiresHumanApproval
            && let Ok(Some(escrow)) = state.store.get_escrow(&id).await {
                let now_dt = chrono::Utc::now();
                let exp_dt = now_dt + chrono::Duration::hours(PURCHASE_TTL_HOURS);
                let hitl = HitlRow {
                    // D2 — the escrow ID is up to 128 characters; the raw "hitl:{id}" would break the
                    // wire cap and panic the admin — a safe deterministic conversion:
                    id: safe_prefixed_wire_id("hitl", &id),
                    kind: "ESCROW_RELEASE".to_owned(),
                    escrow_id: id.clone(),
                    amount: escrow.amount.as_i64(),
                    requester_id: escrow.buyer_id.clone(),
                    note: Some(format!(
                        "Auto-enqueued buyer release exceeding HITL threshold ({} Đ)",
                        escrow.amount.as_i64()
                    )),
                    status: "PENDING".to_owned(),
                    created_at: now_dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    expires_at: exp_dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    decided_by: None,
                    decided_at: None,
                    decision_note: None,
                };
                let _ = state.store.enqueue_hitl(hitl, &id).await;
            }
    result
}

/// `POST /market/escrows/{id}/dispute` — buyer-filed dispute (funds locked until arbitration).
///
/// Only that escrow's buyer (with their own session, no admin key needed) can dispute
/// escrows in `FUNDED` or `DELIVERED` state.
///
/// # Errors
/// 401 session; 400 invalid reason; 403 wrong side; 404 unknown escrow; 422 invalid transition.
pub async fn dispute(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<Json<EscrowActionResponse>> {
    let req: DisputeEscrowRequest = parse_body(&body)?;
    let reason = req.reason.trim().to_owned();
    let reason_len = reason.chars().count();
    if !(1..=500).contains(&reason_len) {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }
    let key = req.idempotency_key.as_ref().map(|k| k.get().to_owned());
    let escrow_id = id.clone();
    let closure_key = key.clone();
    escrow_side_action(
        &state,
        &headers,
        &id,
        EscrowSide::Buyer,
        "market.dispute",
        key,
        move |ledger, registry| {
            dispute_escrow(
                ledger,
                registry,
                &DisputeEscrowCommand {
                    escrow_id,
                    idempotency_key: closure_key,
                    at: Some(now_iso()),
                    reason,
                },
            )
            .map(|o| CommandOutcome {
                ledger: o.ledger,
                tx: o.tx,
                escrow: Some(o.escrow),
                events: o.events.into_iter().map(DomainEvent::Escrow).collect(),
            })
        },
    )
    .await
}


// ── Mairon seal and the citizens' 5-star vote (Wave 3 — Trinity marketplace gate) ──

/// `POST /market/products/{id}/mairon-confirm` — the two-key publication gate.
///
/// key 1: Mairon's deterministic verdict on the product's real artifact (`judge_market_product`);
/// key 2: the creator's explicit confirmation via the session route (same endpoint). Only the
/// joint presence of both seals the score and the `⚒️ Mairon REA Certified` badge onto the listing. 200.
///
/// # Errors
/// 401 missing session; 403 not the product's creator; 404 product missing; 422 Mairon's verdict
/// is not an acceptance (red lines or a score below 85); 500 SQL error.
pub async fn mairon_confirm(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<MaironConfirmResponse>> {
    let user = require_session(&state, &headers).await?;
    let _req: serde_json::Value = parse_body(&body)?;
    let product = state
        .store
        .get_product(&product_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {product_id} does not exist"),
            )
            .into()
        })?;
    if product.seller_id != user.account_id {
        return Err(kian_contracts::HttpFailure::forbidden_escrow_role().into());
    }
    let verdict = kian_hermes::MaironSovereignJudge::judge_market_product(&product_id, &product.preview_html);
    // key 2 — creator confirmation via the session route: `creator_confirmed = true`
    let listing = confirm_and_publish_product(&verdict, true).map_err(|msg| {
        DomainError::new(DomainErrorCode::InvalidTransition, msg.to_owned())
    })?;
    state
        .store
        .stamp_product_mairon(&product_id, listing.mairon_rea_score, &listing.mairon_visible_badge)
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(MaironConfirmResponse {
        product_id: listing.product_id,
        mairon_rea_score: listing.mairon_rea_score,
        mairon_visible_badge: listing.mairon_visible_badge,
        creator_confirmed: listing.creator_confirmed,
        published: true,
    }))
}

/// `GET /market/products/{id}/skill` — the Mairon skill bundle + the ARDA circular
/// flag ribbon (public — Article 6 transparency).
///
/// Returns the sealed bundle directly; absent a seal, Mairon's live arbitration runs on
/// `preview_html`, and if the `>= 85` gate passes, the
/// `SKILL.md` + `skill.json` bundle is forged and sealed on the spot
/// (`forged_on_the_fly = true`). Below the gate → 422.
///
/// # Errors
/// 404 product missing; 422 below Mairon's REA gate; 500 SQL.
#[allow(clippy::too_many_lines)]
pub async fn product_skill(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ProductSkillPackageResponse>> {
    let product = state
        .store
        .get_product(&product_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {product_id} does not exist"),
            )
                .into()
        })?;
    // source language and flag ribbon — the sealed provenance or deterministic script detection.
    let origin = if kian_hermes::arda_fishkal_engine::is_planetary_locale(&product.origin_locale) {
        product.origin_locale.clone()
    } else {
        kian_hermes::arda_fishkal_engine::detect_origin_locale(&product.preview_html).to_owned()
    };
    let translated: Vec<String> =
        crate::handlers::arda::parse_supported_locales(&product.supported_locales_json);
    let strip_codes: Vec<&str> = translated.iter().map(String::as_str).collect();
    let strip =
        kian_hermes::arda_fishkal_engine::render_product_flag_strip(&origin, &strip_codes)
            .unwrap_or_default();

    // sealed bundle — direct delivery (deterministic, no re-arbitration).
    if let (Some(md), Some(js)) = (&product.skill_md, &product.skill_json) {
        return Ok(Json(ProductSkillPackageResponse {
            product_id,
            rea_score: product.mairon_score,
            mairon_visible_badge: product.mairon_badge.clone(),
            origin_locale: origin,
            supported_locales: translated,
            circular_flag_strip_html: strip,
            skill_md: md.clone(),
            skill_json: js.clone(),
            forged_on_the_fly: false,
        }));
    }

    // on-the-fly forge — Mairon's live arbitration over the current content (absolute order: no
    // stale state is valid; the content itself is arbitrated).
    let verdict = kian_hermes::MaironSovereignJudge::judge_market_product(&product_id, &product.preview_html);
    if verdict.total_score < MARKETPLACE_APPROVAL_THRESHOLD {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!(
                "Mairon REA gate: score {} is below the forge bar {MARKETPLACE_APPROVAL_THRESHOLD} — skill forge refused",
                verdict.total_score
            ),
        )
            .into());
    }
    let listing = confirm_and_publish_product(&verdict, true).map_err(|msg| {
        DomainError::new(DomainErrorCode::InvalidTransition, msg.to_owned())
    })?;
    let forge_req = kian_hermes::arda_fishkal_engine::SkillForgeRequest {
        product_id: product_id.clone(),
        product_title: product.title.clone(),
        rea_score: verdict.total_score,
        origin_locale: origin.clone(),
        translated_locales: translated.clone(),
    };
    let pkg = kian_hermes::arda_fishkal_engine::MaironSkillForge::forge_skill_package(&forge_req)
        .map_err(|msg| DomainError::new(DomainErrorCode::InvalidTransition, msg.to_string()))?;
    state
        .store
        .stamp_product_arda(
            &product_id,
            &origin,
            &product.supported_locales_json,
            Some(&pkg.skill_md),
            Some(&pkg.skill_json),
            &now_iso(),
        )
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(ProductSkillPackageResponse {
        product_id,
        rea_score: listing.mairon_rea_score,
        mairon_visible_badge: listing.mairon_visible_badge,
        origin_locale: origin,
        supported_locales: translated,
        circular_flag_strip_html: strip,
        skill_md: pkg.skill_md,
        skill_json: pkg.skill_json,
        forged_on_the_fly: true,
    }))
}

/// `POST /market/products/{id}/reviews` — a real citizen vote with a validated
/// `1..=5` star; a verified purchase (escrow `RELEASED`) gets the full Bayesian weight. 201.
///
/// # Errors
/// 401 missing session; 400 invalid star/review; 404 product missing; 422 duplicate vote; 500 SQL.
pub async fn submit_review(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<ProductReviewView>)> {
    let user = require_session(&state, &headers).await?;
    let req: ProductReviewSubmitRequest = parse_body(&body)?;
    let product = state
        .store
        .get_product(&product_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {product_id} does not exist"),
            )
            .into()
        })?;
    // an instant snapshot of Mairon's score: the official seal if present, otherwise a fresh deterministic
    let mairon_score = if product.mairon_score > 0 {
        product.mairon_score
    } else {
        kian_hermes::MaironSovereignJudge::judge_market_product(&product_id, &product.preview_html)
            .total_score
    };
    let existing = state
        .store
        .list_product_reviews(&product_id)
        .await
        .map_err(ApiFailure::from)?;
    let existing_domain: Vec<ProductReview> = existing
        .iter()
        .map(|r| ProductReview {
            review_id: r.id.clone(),
            product_id: r.product_id.clone(),
            reviewer_citizen_id: r.reviewer_id.clone(),
            stars: r.stars,
            comment: r.comment.clone(),
            verified_purchase: r.verified_purchase,
            mairon_score_at_review: r.mairon_score_at_review,
        })
        .collect();
    let verified = state
        .store
        .has_verified_purchase(&product_id, &user.account_id)
        .await
        .map_err(ApiFailure::from)?;
    let review = submit_product_review(
        &existing_domain,
        &format!("rev:{}", uuid::Uuid::now_v7()),
        &product_id,
        &user.account_id,
        req.stars,
        &req.comment,
        verified,
        mairon_score,
    )
    .map_err(|msg: &'static str| {
        // invalid star/review → 400; duplicate vote → 422 (honest HTTP edge mapping)
        if msg.contains("between 1 and 5") || msg.contains("must not") {
            ApiFailure::Http(kian_contracts::HttpFailure::invalid_request(1))
        } else {
            kian_contracts::DomainFailure::new(DomainError::new(
                DomainErrorCode::InvalidTransition,
                msg.to_owned(),
            ))
            .into()
        }
    })?;
    let row = ProductReviewRow {
        id: review.review_id.clone(),
        product_id: review.product_id.clone(),
        reviewer_id: review.reviewer_citizen_id.clone(),
        stars: review.stars,
        comment: review.comment.clone(),
        verified_purchase: review.verified_purchase,
        mairon_score_at_review: review.mairon_score_at_review,
        created_at: now_iso(),
    };
    state
        .store
        .insert_product_review(&row)
        .await
        .map_err(ApiFailure::from)?;
    Ok((StatusCode::CREATED, Json(ProductReviewView {
        review_id: row.id,
        product_id: row.product_id,
        reviewer_id: row.reviewer_id,
        stars: row.stars,
        comment: row.comment,
        verified_purchase: row.verified_purchase,
        mairon_score_at_review: row.mairon_score_at_review,
        created_at: row.created_at,
    })))
}

/// `GET /market/products/{id}/reviews` — the Bayesian 5-star summary + histogram +
/// `combined_rank_score` (`0.60 * bayesian_stars * 20 + 0.40 * mairon_score`).
///
/// # Errors
/// 404 product missing; 500 SQL error.
pub async fn list_reviews(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ProductRatingSummaryResponse>> {
    let product = state
        .store
        .get_product(&product_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else::<ApiFailure, _>(|| {
            DomainError::new(
                DomainErrorCode::ProductNotFound,
                format!("Product {product_id} does not exist"),
            )
            .into()
        })?;
    let rows = state
        .store
        .list_product_reviews(&product_id)
        .await
        .map_err(ApiFailure::from)?;
    let reviews: Vec<ProductReview> = rows
        .iter()
        .map(|r| ProductReview {
            review_id: r.id.clone(),
            product_id: r.product_id.clone(),
            reviewer_citizen_id: r.reviewer_id.clone(),
            stars: r.stars,
            comment: r.comment.clone(),
            verified_purchase: r.verified_purchase,
            mairon_score_at_review: r.mairon_score_at_review,
        })
        .collect();
    // technical layer: the official seal if present, otherwise a fresh deterministic arbitration on preview_html
    let (score, badge) = if product.mairon_score > 0 {
        (product.mairon_score, product.mairon_badge.clone())
    } else {
        let v = kian_hermes::MaironSovereignJudge::judge_market_product(&product_id, &product.preview_html);
        (v.total_score, v.visible_badge)
    };
    let verdict = evaluate_user_product_with_mairon(
        &product_id,
        MaironReaPillarScores {
            architecture: 0,
            security_hardening: 0,
            ux_and_anti_slop: 0,
            verification_proof: 0,
        }
        .clamped(),
        Vec::new(),
    );
    // Mairon's official score in the summary comes from the seal/fresh arbitration — with an
    // equal-score verdict reconstruction, the domain's Bayesian engine is consumed without copying the formula.
    let verdict_with_score = evaluate_user_product_with_mairon(
        &product_id,
        pillar_scores_for_total(score),
        Vec::new(),
    );
    let _ = &verdict;
    let summary = summarize_product_ratings(&product_id, &verdict_with_score, &reviews);
    Ok(Json(ProductRatingSummaryResponse {
        product_id: summary.product_id,
        mairon_rea_score: score,
        mairon_visible_badge: if badge.is_empty() { verdict_with_score.visible_badge.clone() } else { badge },
        total_reviews: summary.total_reviews,
        verified_reviews: summary.verified_reviews,
        average_stars: summary.average_stars,
        bayesian_stars: summary.bayesian_stars,
        star_histogram: summary.star_histogram.to_vec(),
        five_star_display: summary.five_star_display.clone(),
        combined_rank_score: summary.combined_rank_score,
        reviews: rows
            .iter()
            .map(|r| ProductReviewView {
                review_id: r.id.clone(),
                product_id: r.product_id.clone(),
                reviewer_id: r.reviewer_id.clone(),
                stars: r.stars,
                comment: r.comment.clone(),
                verified_purchase: r.verified_purchase,
                mairon_score_at_review: r.mairon_score_at_review,
                created_at: r.created_at.clone(),
            })
            .collect(),
    }))
}

/// A deterministic four-column arrangement to reach the target sum — Mairon's official badge in
/// the wire summary is reproduced with the domain engine (no copying of the badge formula).
fn pillar_scores_for_total(target: u8) -> MaironReaPillarScores {
    let a = target.min(25);
    let rest = target - a;
    let s = rest.min(25);
    let rest2 = rest - s;
    let u = rest2.min(25);
    let v = rest2 - u;
    MaironReaPillarScores { architecture: a, security_hardening: s, ux_and_anti_slop: u, verification_proof: v }
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

    /// Authenticate a Telegram user and issue a token.
    async fn auth(app: &axum::Router, tg_id: &str, username: &str) -> String {
        let init_data = make_test_init_data(BOT, tg_id, Some(username));
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
        assert_eq!(res.status(), 201, "auth of {username} must succeed");
        json_body(res).await["token"].as_str().unwrap().to_owned()
    }

    /// Mint Din to a citizen account (M2 admin route — seeding the market scene).
    async fn mint(app: &axum::Router, account: &str, amount: i64) {
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/mint")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(json!({ "accountId": account, "amount": amount }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "mint to {account} must succeed");
    }

    #[tokio::test]
    async fn product_create_list_and_self_buy_rejected() {
        let app = app().await;
        let token = auth(&app, "111", "seller").await;

        // product registration — 201
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({ "title": "bread recipe", "description": "full explanation", "priceDin": 250 })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let body = json_body(res).await;
        assert_eq!(body["sellerId"], "citizen:111");
        assert_eq!(body["priceDin"], 250);

        // public listing
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/market/products")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = json_body(res).await;
        assert_eq!(body["items"].as_array().unwrap().len(), 1);

        // self-purchase — 422
        let product_id = body["items"][0]["id"].as_str().unwrap().to_owned();
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/purchase")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(json!({ "productId": product_id }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422, "self purchase must be rejected");
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)] // the full atomic-purchase cycle + both parties' actions — test readability matters more
    async fn full_purchase_cycle_through_escrow() {
        let app = app().await;
        let seller = auth(&app, "222", "seller2").await;
        let buyer = auth(&app, "333", "buyer2").await;

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({ "title": "language class", "description": "one session", "priceDin": 500 })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let product_id = json_body(res).await["id"].as_str().unwrap().to_owned();
        mint(&app, "citizen:333", 1_000).await;

        // purchase — the escrow must become FUNDED
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/purchase")
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(json!({ "productId": product_id }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let body = json_body(res).await;
        assert_eq!(body["escrow"]["status"], "FUNDED");
        assert_eq!(body["escrow"]["buyerId"], "citizen:333");
        let escrow_id = body["escrow"]["id"].as_str().unwrap().to_owned();

        // buyer balance after the funds lock
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let me = json_body(res).await;
        assert_eq!(me["balance"], 500, "1000 − 500 locked");
        assert_eq!(me["escrowLocked"], 500);

        // deliver from the buyer side — 403 (seller only)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/deliver"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 403);

        // deliver from the seller side — 200 and DELIVERED
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/deliver"))
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(json!({ "deliveryRef": "file:lesson-1" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(json_body(res).await["escrow"]["status"], "DELIVERED");

        // release from the buyer side — the 10% charter tax
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/release"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(json_body(res).await["escrow"]["status"], "RELEASED");

        // final settlement: seller 450 + treasury 50
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/economy")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let eco = json_body(res).await;
        assert_eq!(eco["trialBalanceSum"], 0, "the ledger must always balance");

        // my orders — both sides see it
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/market/my")
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let mine = json_body(res).await;
        assert_eq!(mine["orders"].as_array().unwrap().len(), 1);
        assert_eq!(mine["orders"][0]["status"], "RELEASED");

        // the seller sees their product
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/market/my")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let mine = json_body(res).await;
        assert_eq!(mine["products"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn purchase_without_balance_is_rejected_honestly() {
        let app = app().await;
        let seller = auth(&app, "444", "seller3").await;
        let buyer = auth(&app, "555", "buyer3").await;

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({ "title": "treasure", "description": "very expensive", "priceDin": 900 })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let product_id = json_body(res).await["id"].as_str().unwrap().to_owned();

        // no mint — zero balance; the purchase must yield a 422 overdraft
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/purchase")
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(json!({ "productId": product_id }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422, "overdraft must be rejected");
        let body = json_body(res).await;
        assert_eq!(body["error"]["code"], "ACCOUNT_OVERDRAFT");
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn html_artifact_preview_delivery_referral_and_access_control() {
        let app = app().await;
        let seller = auth(&app, "601", "creator").await;
        let buyer = auth(&app, "602", "customer").await;
        let referrer = auth(&app, "603", "affiliate").await;
        let stranger = auth(&app, "604", "stranger").await;
        mint(&app, "citizen:602", 2_000).await;

        let full_html = "<!doctype html><html><body><h1>Paid full product</h1></body></html>";
        let preview_html = "<div><h1>Free demo</h1></div>";

        // create a product with a preview, full HTML code, and a 20% referrer commission
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({
                            "title": "crypto profit calculator",
                            "description": "single-page interactive tool",
                            "priceDin": 1_000,
                            "previewHtml": preview_html,
                            "affiliateBps": 2_000,
                            "artifactKind": "html",
                            "deliveryPayload": full_html
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let created = json_body(res).await;
        let product_id = created["id"].as_str().unwrap().to_owned();
        assert_eq!(created["affiliateBps"], 2_000);
        assert_eq!(created["artifactKind"], "html");
        assert_eq!(created["previewHtml"], preview_html);
        assert!(created.get("deliveryPayload").is_none());

        // the public single-product page — has previewHtml but deliveryPayload never leaks
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/products/{product_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let pub_prod = json_body(res).await;
        assert_eq!(pub_prod["previewHtml"], preview_html);
        assert!(pub_prod.get("deliveryPayload").is_none());

        // purchase with a numeric referrer ID ("603" → "citizen:603")
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/purchase")
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(
                        json!({ "productId": product_id, "referrer": "603" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let purchase = json_body(res).await;
        let escrow_id = purchase["escrow"]["id"].as_str().unwrap().to_owned();
        assert_eq!(purchase["escrow"]["status"], "FUNDED");

        // a stranger cannot access deliveryPayload → 403
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/delivery"))
                    .header("authorization", format!("Bearer {stranger}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 403);

        // the buyer sees the full HTML content immediately in FUNDED state → 200
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/delivery"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let del = json_body(res).await;
        assert_eq!(del["escrowId"], escrow_id);
        assert_eq!(del["productId"], product_id);
        assert_eq!(del["artifactKind"], "html");
        assert_eq!(del["state"], "FUNDED");
        assert_eq!(del["deliveryPayload"], full_html);

        // direct release by the buyer (instant delivery + a 4-posting settlement)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/release"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(json_body(res).await["escrow"]["status"], "RELEASED");

        // check the 3 beneficiaries' balances: seller 700 + treasury 100 + referrer 200 = 1000
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["balance"], 700);

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {referrer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["balance"], 200);

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
        assert_eq!(eco["treasuryBalance"], 100);
        assert_eq!(eco["trialBalanceSum"], 0);
    }

    #[tokio::test]
    async fn buyer_can_open_dispute_and_unknown_referrer_does_not_break_purchase() {
        let app = app().await;
        let seller = auth(&app, "701", "s_disp").await;
        let buyer = auth(&app, "702", "b_disp").await;
        mint(&app, "citizen:702", 500).await;

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({
                            "title": "dashboard template",
                            "description": "full HTML",
                            "priceDin": 300,
                            "deliveryPayload": "<html>broken</html>"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let product_id = json_body(res).await["id"].as_str().unwrap().to_owned();

        // a referrer missing from the system must not break the customer's purchase
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/purchase")
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(
                        json!({ "productId": product_id, "referrer": "999999" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let escrow_id = json_body(res).await["escrow"]["id"].as_str().unwrap().to_owned();

        // the seller cannot dispute via the buyer route → 403
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/dispute"))
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(json!({ "reason": "forbidden attempt" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 403);

        // the buyer files the dispute → 200 and DISPUTED
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/dispute"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(
                        json!({ "reason": "the HTML file does not run" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let disputed = json_body(res).await;
        assert_eq!(disputed["escrow"]["status"], "DISPUTED");
        assert_eq!(disputed["escrow"]["disputeReason"], "the HTML file does not run");

        // in DISPUTED state, the delivery fetch is rejected → 422
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/delivery"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
    }
    // ── Mairon seal + citizens' 5-star vote — live HTTP routes (Wave 3) ──

    /// The forged artifact — passes Mairon's gate (RTL + 390 + 3 sections + script).
    const FORGED_PREVIEW: &str = "<!doctype html><html lang=\"fa\" dir=\"rtl\"><head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>body{font-family:'Vazirmatn',sans-serif;max-width: 390px;}</style></head><body><section aria-label=\"A\">dashboard</section><section>report</section><section>settings</section><script>/* assert: 390px zero-overflow */</script></body></html>";

    async fn create_forged_product(app: &axum::Router, token: &str, title: &str) -> String {
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({
                            "title": title,
                            "description": "single-page artifact forged for the MAIRON gate",
                            "priceDin": 500,
                            "previewHtml": FORGED_PREVIEW
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        json_body(res).await["id"].as_str().unwrap().to_owned()
    }

    /// The two-key gate: the seal lands only with Mairon's verdict + creator confirmation;
    /// the 5-star vote accumulates with Bayesian weight and an exact histogram; duplicate votes 422.
    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn mairon_confirm_two_key_gate_then_review_lifecycle() {
        let app = app().await;
        let seller = auth(&app, "321", "forge-seller").await;
        let buyer = auth(&app, "322", "star-buyer").await;
        let product_id = create_forged_product(&app, &seller, "forged product").await;

        // the unsealed product — the vote summary shows Mairon's fresh arbitration
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/products/{product_id}/reviews"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["totalReviews"], 0);
        assert!(body["maironReaScore"].as_u64().unwrap() >= 85, "body: {body}");

        // anyone but the creator → 403 (the second key cannot be stolen)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/products/{product_id}/mairon-confirm"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from("{}".to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 403);

        // key 1 + key 2 — the creator confirms → the Forge-Grade seal
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/products/{product_id}/mairon-confirm"))
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from("{}".to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["creatorConfirmed"], true);
        assert_eq!(body["published"], true);
        let score = body["maironReaScore"].as_u64().unwrap();
        assert!(score >= 85, "score: {score}");
        assert!(body["maironVisibleBadge"].as_str().unwrap().contains("Forge-Grade"));

        // the public product view now carries the seal (additive wire)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/products/{product_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = json_body(res).await;
        assert_eq!(body["maironScore"], score);
        assert!(body["maironBadge"].as_str().unwrap().contains("Forge-Grade"));

        // buyer vote — 5 stars; no verified purchase → weight 0.25 (anti-shill)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/products/{product_id}/reviews"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(
                        json!({"stars": 5, "comment": "the forge is really clean — no slop"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let review = json_body(res).await;
        assert_eq!(review["stars"], 5);
        assert_eq!(review["verifiedPurchase"], false);
        assert_eq!(review["maironScoreAtReview"], score);
        assert!(review["reviewId"].as_str().unwrap().starts_with("rev:"));

        // the one-vote lock — the same citizen → 422
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/products/{product_id}/reviews"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(
                        json!({"stars": 1, "comment": "the second vote is surreal"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);

        // a star outside 1..=5 → 400
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/products/{product_id}/reviews"))
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({"stars": 0, "comment": "invalid star"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 400);

        // the Bayesian summary — exact histogram + composite rating
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/products/{product_id}/reviews"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = json_body(res).await;
        assert_eq!(body["totalReviews"], 1);
        assert_eq!(body["verifiedReviews"], 0);
        assert_eq!(body["starHistogram"], json!([0, 0, 0, 0, 1]));
        assert_eq!(body["averageStars"], 5.0);
        let bayes = body["bayesianStars"].as_f64().unwrap();
        assert!(bayes > 4.2 && bayes < 5.0, "prior 4.20 pulls the lone 5-star down: {bayes}");
        let combined = body["combinedRankScore"].as_f64().unwrap();
        let expected = 0.60 * (bayes * 20.0) + 0.40 * f64::from(u32::try_from(score).unwrap());
        assert!((combined - expected).abs() < 1e-9, "combined: {combined} vs {expected}");
        assert!(body["fiveStarDisplay"].as_str().unwrap().contains("⚒️ MAIRON REA score"));
    }

    /// The raw product — Mairon's gate honestly rejects it (422) and no seal lands.
    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn mairon_confirm_honestly_rejects_unforged_artifact() {
        let app = app().await;
        let seller = auth(&app, "331", "raw-seller").await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({
                            "title": "raw product",
                            "description": "no real artifact",
                            "priceDin": 100
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let product_id = json_body(res).await["id"].as_str().unwrap().to_owned();
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/products/{product_id}/mairon-confirm"))
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from("{}".to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422, "unforged artifact must not pass the gate");
        let body = json_body(res).await;
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not approved"));
        // a vote on the unsealed product — the fresh arbitration score snapshot is recorded
        let buyer = auth(&app, "332", "candid-buyer").await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/products/{product_id}/reviews"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(
                        json!({"stars": 2, "comment": "not forged yet"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let review = json_body(res).await;
        let stamp = review["maironScoreAtReview"].as_u64().unwrap();
        assert!(stamp < 85, "fresh judgement must be honest: {stamp}");
    }

    /// The rich Persian preview — passes Mairon's REA gate (`>= 85`):
    /// 5 sections (architecture), `dir="rtl"` + `max-width:390px` + Vazirmatn + viewport
    /// (UX), 4 scripts + 2 probes + aria (verification).
    const RICH_FA_PREVIEW: &str = "<!doctype html><html lang=\"fa\" dir=\"rtl\"><head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>body{font-family:'Vazirmatn';max-width:390px}</style></head><body><section id=\"sod\">compound profit calculation</section><section id=\"asset\">assets and liabilities</section><section id=\"market\">live market</section><section id=\"risk\">risk management</section><section id=\"report\">monthly report</section><div role=\"main\" aria-label=\"financial calculator\"></div><div data-probe=\"p1\">data probe</div><div data-probe=\"p2\">review probe</div><script>/* calculation */</script><script>/* chart */</script><script>/* risk */</script><script>/* report */</script></body></html>";

    async fn create_rich_product(app: &axum::Router, seller: &str) -> String {
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({
                            "title": "Accounted — crypto profit calculator",
                            "description": "Persian single-page interactive tool with a monthly report",
                            "priceDin": 2_000,
                            "previewHtml": RICH_FA_PREVIEW
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        json_body(res).await["id"].as_str().unwrap().to_owned()
    }

    #[tokio::test]
    async fn skill_forges_on_the_fly_then_serves_stamped_package() {
        let app = app().await;
        let seller = auth(&app, "888001", "skill_seller").await;
        let product_id = create_rich_product(&app, &seller).await;

        // first call — an on-the-fly forge after Mairon's live arbitration.
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/products/{product_id}/skill"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["productId"], product_id);
        assert_eq!(body["forgedOnTheFly"], true);
        assert!(body["reaScore"].as_u64().unwrap() >= 85);
        assert!(body["maironVisibleBadge"].as_str().unwrap().contains("⚒️"));
        assert_eq!(body["originLocale"], "fa");
        // SKILL.md with YAML front matter and verification gates.
        let skill_md = body["skillMd"].as_str().unwrap();
        assert!(skill_md.starts_with("---\n"));
        assert!(skill_md.contains("mairon_certified: true"));
        assert!(skill_md.contains("rea_score:"));
        assert!(skill_md.contains("## Verification Gates"));
        // skill.json — the registry manifest.
        let manifest: serde_json::Value =
            serde_json::from_str(body["skillJson"].as_str().unwrap()).unwrap();
        assert_eq!(manifest["mairon_certified"], serde_json::Value::Bool(true));
        assert_eq!(manifest["origin_locale"], "fa");
        assert!(manifest["registry"]["compatible"].as_array().unwrap().len() >= 3);
        // the circular flag ribbon — provenance + the 2 default translations from migration 0008.
        let strip = body["circularFlagStripHtml"].as_str().unwrap();
        assert_eq!(strip.matches("arda-flag-badge").count(), 3);
        assert!(strip.contains("data-origin=\"fa\""));

        // second call — the sealed bundle without re-forging (deterministic).
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/products/{product_id}/skill"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let stamped = json_body(res).await;
        assert_eq!(stamped["forgedOnTheFly"], false);
        assert_eq!(stamped["skillMd"], body["skillMd"]);
        assert_eq!(stamped["skillJson"], body["skillJson"]);
    }

    #[tokio::test]
    async fn skill_refuses_below_rea_gate() {
        let app = app().await;
        let seller = auth(&app, "888002", "skill_poor").await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({
                            "title": "simple demo",
                            "description": "preview below the MAIRON gate",
                            "priceDin": 500,
                            "previewHtml": "<div><h1>Free demo</h1></div>"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let pid = json_body(res).await["id"].as_str().unwrap().to_owned();

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/products/{pid}/skill"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
        let err = json_body(res).await;
        assert!(err["error"]["message"]
            .as_str()
            .unwrap()
            .contains("below the forge bar"));
    }

    #[tokio::test]
    async fn skill_unknown_product_is_404() {
        let app = app().await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/market/products/prod:missing/skill")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 404);
    }
}
