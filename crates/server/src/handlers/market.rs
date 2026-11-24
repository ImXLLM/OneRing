//! هندلرهای بازار M3 — محصول، خرید اتمی روی اسکرو ۸ حالته و کنش‌های دو طرف.
//!
//! خرید = `create_escrow` + `fund_escrow` در «یک» فرمان اتمی داخل همان `apply`:
//! یا هر دو رخ می‌دهند یا هیچ‌کدام — پول خریدار فقط با FUNDED قفل می‌شود.
//! کنش‌های بعدی فقط از سمت مجاز اسکرو: فروشنده تحویل می‌دهد، خریدار آزاد
//! می‌کند (آستانهٔ HITL مادهٔ ۷ در دامنه محافظت می‌شود).
//! محتوای تحویل (`deliveryPayload`) هرگز روی وایر عمومی نمی‌آید.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::sync::Arc;

use kian_contracts::{
    CreateProductRequest, DeliverEscrowRequest, DisputeEscrowRequest, EscrowActionResponse,
    EscrowDeliveryView, MAX_DELIVERY_PAYLOAD_BYTES, MAX_PREVIEW_HTML_BYTES, MyMarketResponse,
    ProductListResponse, ProductView, PurchaseRequest, PurchaseResponse, ReleaseEscrowRequest,
};
use kian_db::{now_iso, ApplyOptions, CommandOutcome, HitlRow, ProductFilter, ProductRow};
use kian_domain::{
    create_escrow_with_affiliate, deliver_escrow, dispute_escrow, fund_escrow, release_escrow,
    AccountKind, CreateEscrowCommand, DeliverEscrowCommand, DisputeEscrowCommand, DomainError,
    DomainErrorCode, DomainEvent, DomainResult, EscrowRefCommand, EscrowRegistry, EscrowStatus,
    LedgerState, ReleaseEscrowCommand, DEFAULT_AFFILIATE_BPS, MAX_AFFILIATE_BPS,
};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::require_session;
use crate::state::AppState;
use crate::views::{escrow_status_wire, escrow_view};

/// مهلت پیش‌فرض اسکرو خرید بازار — ۷۲ ساعت.
const PURCHASE_TTL_HOURS: i64 = 72;

/// نرمال‌سازی شناسهٔ معرف به فرمت حساب شهروندی (`citizen:{tg_id}`).
///
/// ورودی می‌تواند `tg_id` عددی (`"12345"`) یا `"citizen:12345"` باشد؛
/// ورودی‌های نامعتبر/خالی `None` می‌شوند تا خرید مشتری هرگز متوقف نگردد.
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

/// نمای محصول وایر از ردیف db — بدون `deliveryPayload` (هرگز عمومی نمی‌شود).
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
    }
}

/// `POST /market/products` — ثبت محصول به نام کاربر نشست. ۲۰۱.
///
/// # Errors
/// ۴۰۱ نشست غایب؛ ۴۰۰ بدنهٔ نامعتبر (طول‌ها/قیمت/حجم HTML)؛ ۴۲۲ `BPS_OUT_OF_RANGE`؛
/// 500 خطای SQL.
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

    let row = ProductRow {
        id: format!("prod:{}", uuid::Uuid::now_v7()),
        seller_id: user.account_id.clone(),
        title: req.title,
        description: req.description,
        category: req.category,
        price_din: req.price_din.get(),
        preview_html: req.preview_html.unwrap_or_default(),
        affiliate_bps,
        artifact_kind,
        delivery_payload: req.delivery_payload,
        active: true,
        created_at: now_iso(),
    };
    state.store.insert_product(&row).await.map_err(ApiFailure::from)?;
    Ok((StatusCode::CREATED, Json(product_view(&row))))
}

/// `GET /market/products` — محصولات فعال برای همه (شفافیت مادهٔ ۶؛ بدون `deliveryPayload`).
///
/// # Errors
/// خطای SQL.
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

/// `GET /market/products/{id}` — صفحهٔ عمومی تک‌محصول برای لینک مستقیم و رفرال.
///
/// `deliveryPayload` هرگز روی این مسیر برگردانده نمی‌شود (فقط `previewHtml`).
///
/// # Errors
/// 404 محصول ناشناخته؛ 500 خطای SQL.
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

/// `POST /market/purchase` — خرید اتمی: ساخت اسکرو (با پشتیبانی رفرال) + قفل Din.
///
/// # Errors
/// ۴۰۱ نشست؛ 404 محصول ناشناخته؛ ۴۲۲ محصول غیرفعال/خرید از خود/تراز ناکافی؛
/// 500 خطای SQL.
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
                // اگر معرف معتبر در دفتر وجود داشت و حساب User بود ثبت می‌شود؛
                // در غیر این صورت خرید مشتری هرگز متوقف نمی‌شود (خودرفرالی هم در دامنه خنثی می‌گردد).
                let effective_referrer = referrer_candidate.as_ref().and_then(|r| {
                    ledger
                        .accounts
                        .get(r)
                        .filter(|a| a.kind == AccountKind::User)
                        .map(|_| r.clone())
                });
                let affiliate_bps =
                    if effective_referrer.is_some() { product_affiliate_bps } else { 0 };
                // یک فرمان اتمی: CREATED + FUNDED — هر دو یا هیچ‌کدام
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

/// `GET /market/my` — محصولات من + سفارش‌های من (خریدار یا فروشنده).
///
/// # Errors
/// ۴۰۱ نشست؛ 500 خطای SQL.
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

/// سمت مجاز کنش اسکرو — فقط مالک همان سمت حق گذار دارد.
enum EscrowSide {
    /// تحویل — فقط فروشندهٔ همان اسکرو
    Seller,
    /// آزادسازی — فقط خریدارِ همان اسکرو
    Buyer,
}

/// کنش اسکرو از سمت بازار — گوار مالکیت + اجرای دامنهٔ خالص در `apply`.
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

/// `POST /market/escrows/{id}/deliver` — فقط فروشندهٔ همان اسکرو.
///
/// # Errors
/// ۴۰۱ نشست؛ 404 اسکرو ناشناخته؛ ۴۰۳ سمت نادرست؛ ۴۲۲ گذار نامعتبر.
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

/// `GET /market/escrows/{id}/delivery` — دریافت محتوای کامل محصول پولی.
///
/// فقط خریدار یا فروشندهٔ همان اسکرو و صرفاً در وضعیت‌های پرداخت‌شده
/// (`FUNDED` / `DELIVERED` / `RELEASED`) حق دسترسی دارند. برای محصولات
/// دیجیتال آنی با `deliveryPayload` ازپیش‌ثبت‌شده، خریدار بلافاصله در حالت
/// `FUNDED` محتوا را می‌بیند و سپس می‌تواند `release` یا `dispute` بزند.
///
/// # Errors
/// ۴۰۱ نشست؛ 404 اسکرو یا محصول ناشناخته؛ ۴۰۳ سمت غیرمجاز؛ ۴۲۲ وضعیت پرداخت‌نشده.
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

/// `POST /market/escrows/{id}/release` — فقط خریدار همان اسکرو (آستانهٔ HITL در دامنه).
///
/// اگر محصول مرتبط دارای `deliveryPayload` ازپیش‌ثبت‌شده باشد (محصول دیجیتال آنی)
/// و اسکرو هنوز در وضعیت `FUNDED` باشد، تحویل آنی + آزادسازی اتمی در همان
/// تراکنش انجام می‌شود.
///
/// # Errors
/// ۴۰۱ نشست؛ 404 اسکرو ناشناخته؛ ۴۰۳ سمت نادرست؛ ۴۲۲ گذار نامعتبر یا آستانهٔ
/// ۵۰٬۰۰۰ Đ بدون تأیید ادمین (`REQUIRES_HUMAN_APPROVAL`).
pub async fn release(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<Json<EscrowActionResponse>> {
    let req: ReleaseEscrowRequest = parse_body(&body)?;
    // بررسی وجود محتوای تحویل آنی روی محصولِ مرتبط (فقط برای گذار FUNDED→DELIVERED→RELEASED)
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
    // خریدار نشست‌دار هرگز نمی‌تواند خودش را «تأییدشدهٔ ادمین» اعلام کند —
    // فلگ ادمین فقط از مسیر `POST /escrows/{id}/release` (گوار X-KIAN-Key) می‌آید.
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

    // اگر مبلغ اسکرو از آستانهٔ HITL مادهٔ ۷ گذشت، درخواست تأیید انسانی را در صف HitlRequest ثبت کن
    if let Err(ApiFailure::Domain(ref err)) = result
        && err.error.code == DomainErrorCode::RequiresHumanApproval
            && let Ok(Some(escrow)) = state.store.get_escrow(&id).await {
                let now_dt = chrono::Utc::now();
                let exp_dt = now_dt + chrono::Duration::hours(PURCHASE_TTL_HOURS);
                let hitl = HitlRow {
                    id: format!("hitl:{id}"),
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

/// `POST /market/escrows/{id}/dispute` — ثبت اختلاف توسط خریدار (قفل پول تا داوری).
///
/// فقط خریدارِ همان اسکرو (با نشست خودش، بدون نیاز به کلید ادمین) روی اسکروهای
/// `FUNDED` یا `DELIVERED` می‌تواند اختلاف ثبت کند.
///
/// # Errors
/// ۴۰۱ نشست؛ ۴۰۰ دلیل نامعتبر؛ ۴۰۳ سمت نادرست؛ 404 اسکرو ناشناخته؛ ۴۲۲ گذار نامعتبر.
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

    /// احراز کاربر تلگرام و گرفتن توکن.
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

    /// ضرب Din به حساب شهروند (مسیر ادمین M2 — بذر صحنهٔ بازار).
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

        // ثبت محصول — ۲۰۱
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({ "title": "دستور پخت نان", "description": "توضیح کامل", "priceDin": 250 })
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

        // فهرست عمومی
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

        // خرید از خود — ۴۲۲
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
    #[allow(clippy::too_many_lines)] // چرخهٔ کامل خرید اتمی + کنش‌های طرفین — خوانایی تست مهم‌تر
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
                        json!({ "title": "کلاس زبان", "description": "یک جلسه", "priceDin": 500 })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let product_id = json_body(res).await["id"].as_str().unwrap().to_owned();
        mint(&app, "citizen:333", 1_000).await;

        // خرید — امانت باید FUNDED شود
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

        // تراز خریدار پس از قفل پول
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

        // تحویل از سمت خریدار — ۴۰۳ (فقط فروشنده)
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

        // تحویل از سمت فروشنده — ۲۰۰ و DELIVERED
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

        // آزادسازی از سمت خریدار — مالیات ۱۰٪ منشور
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

        // تسویهٔ نهایی: فروشنده ۴۵۰ + خزانه ۵۰
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

        // سفارش‌های من — هر دو طرف می‌بینند
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

        // فروشنده محصولش را می‌بیند
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
                        json!({ "title": "گنج", "description": "بسیار گران", "priceDin": 900 })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let product_id = json_body(res).await["id"].as_str().unwrap().to_owned();

        // بدون ضرب — تراز صفر؛ خرید باید overdraft ۴۲۲ بدهد
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

        let full_html = "<!doctype html><html><body><h1>محصول کامل پولی</h1></body></html>";
        let preview_html = "<div><h1>دموی رایگان</h1></div>";

        // ساخت محصول با پیش‌نمایش، کد کامل HTML و ۲۰٪ پورسانت معرف
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/products")
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(
                        json!({
                            "title": "محاسبه‌گر سود کریپتو",
                            "description": "ابزار تعاملی تک‌صفحه‌ای",
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

        // صفحهٔ عمومی تک‌محصول — previewHtml دارد ولی deliveryPayload هرگز لو نمی‌رود
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

        // خرید با شناسهٔ عددی معرف ("603" → "citizen:603")
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

        // غریبه نمی‌تواند به deliveryPayload دسترسی پیدا کند → ۴۰۳
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

        // خریدار بلافاصله در حالت FUNDED محتوای HTML کامل را می‌بیند → ۲۰۰
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

        // آزادسازی مستقیم توسط خریدار (تحویل آنی + تسویه ۴پستینگی)
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

        // بررسی تراز ۳ ذی‌نفع: فروشنده ۷۰۰ + خزانه ۱۰۰ + معرف ۲۰۰ = ۱۰۰۰
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
                            "title": "قالب داشبورد",
                            "description": "HTML کامل",
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

        // معرف ناموجود در سیستم نباید خرید مشتری را خراب کند
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

        // فروشنده نمی‌تواند از مسیر خریدار dispute بزند → ۴۰۳
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/dispute"))
                    .header("authorization", format!("Bearer {seller}"))
                    .body(Body::from(json!({ "reason": "تلاش غیرمجاز" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 403);

        // خریدار اختلاف ثبت می‌کند → ۲۰۰ و DISPUTED
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/dispute"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(
                        json!({ "reason": "فایل HTML اجرا نمی‌شود" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let disputed = json_body(res).await;
        assert_eq!(disputed["escrow"]["status"], "DISPUTED");
        assert_eq!(disputed["escrow"]["disputeReason"], "فایل HTML اجرا نمی‌شود");

        // در وضعیت DISPUTED دریافت delivery رد می‌شود → ۴۲۲
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
}
