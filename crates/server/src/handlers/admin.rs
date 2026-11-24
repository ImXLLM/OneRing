//! هندلرهای پنل ادمین (M4) — overview، صف HITL، دنبالهٔ ممیزی، واریزهای TON
//! و مدیریت محصولات.
//!
//! امنیت: همهٔ مسیرها `require_admin` با `X-KIAN-Key` — fail-closed: بدون کلید
//! سرور ۵۰۳، بدون هدر ۴۰۱، هدر نادرست ۴۰۱ (مقایسهٔ زمان‌ثابت از contracts).
//! تصمیم HITL «تأیید ⇒ تسویه» اتمی است — پول ۹۰/۱۰ دقیقاً یک‌بار جابه‌جا
//! می‌شود (`approve_hitl_and_settle`).

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use std::collections::HashMap;
use std::sync::Arc;

use kian_contracts::{
    AdminAuditEntry, AdminAuditListResponse, AdminDepositListResponse, AdminDepositView,
    AdminHitlListResponse, AdminHitlView, AdminOverviewResponse, AdminProductListResponse,
    HitlDecisionRequest, HitlDecisionResponse, HttpFailure, Id, ProductSalesView, ProductView,
    RevenuePoint, SellerSalesView,
};

use crate::error::{ApiFailure, ApiResult};
use crate::middleware::require_admin;
use crate::state::AppState;
use crate::views::wire_din;

/// `GET /admin/overview` — نمای کلی — همهٔ اعداد از کوئری‌های زندهٔ دفتر.
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۵۰۰ db.
pub async fn overview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<AdminOverviewResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let s = &state.store;
    let users = s.count_users().await.map_err(ApiFailure::from)?;
    let accounts = s.account_count().await.map_err(ApiFailure::from)?;
    let escrows = s.count_escrows().await.map_err(ApiFailure::from)?;
    let hitl_pending = s.count_hitl("PENDING").await.map_err(ApiFailure::from)?;
    let minted = s.minted_all_time().await.map_err(ApiFailure::from)?;
    let burned = s.burned_all_time().await.map_err(ApiFailure::from)?;
    let ai_calls = s.ai_calls_total().await.map_err(ApiFailure::from)?;
    let series = s.revenue_series_14d().await.map_err(ApiFailure::from)?;
    let by_seller = s.sales_by_seller().await.map_err(ApiFailure::from)?;
    let by_product = s.sales_by_product().await.map_err(ApiFailure::from)?;

    let revenue_series = series.into_iter().map(|(day, din)| RevenuePoint { day, din: wire_din(din) }).collect();
    let sales_by_seller =
        by_seller.into_iter().map(|(seller_id, din)| SellerSalesView { seller_id, din: wire_din(din) }).collect();
    let mut sales_by_product = Vec::new();
    for (product_id, din) in by_product {
        let title = s.product_title(Some(&product_id)).await.unwrap_or(None);
        sales_by_product.push(ProductSalesView { product_id, title, din: wire_din(din) });
    }

    Ok(Json(AdminOverviewResponse {
        users,
        accounts,
        escrows,
        hitl_pending,
        minted_all_time: wire_din(minted),
        burned_all_time: wire_din(burned),
        ai_calls,
        revenue_series,
        sales_by_seller,
        sales_by_product,
    }))
}

/// `GET /admin/hitl?status=PENDING` — صف HITL؛ بدون فیلتر همهٔ وضعیت‌ها.
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۵۰۰ db.
///
/// # Panics
/// هیچ‌گاه روی ورودی کاربر — `expect` فقط روی شناسه‌های wire-safe دیتابیس.
#[allow(clippy::implicit_hasher)] // Query extractor axum — hasher در اینجا بی‌معناست
pub async fn hitl_list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> ApiResult<Json<AdminHitlListResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let status = q.get("status").map(|s| s.trim()).filter(|s| !s.is_empty());
    let rows = state.store.list_hitl(status).await.map_err(ApiFailure::from)?;
    Ok(Json(AdminHitlListResponse {
        items: rows
            .into_iter()
            .map(|r| AdminHitlView {
                id: Id::new(r.id).expect("hitl ids are wire-safe"),
                kind: r.kind,
                escrow_id: r.escrow_id,
                amount: wire_din(r.amount),
                requester_id: r.requester_id,
                note: r.note,
                status: r.status,
                created_at: r.created_at,
                expires_at: r.expires_at,
                decided_by: r.decided_by,
                decided_at: r.decided_at,
                decision_note: r.decision_note,
            })
            .collect(),
    }))
}

/// `POST /admin/hitl/{id}/approve` — تأیید + تسویهٔ اتمی در همان فرمان.
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۴۰۴ ناشناخته، ۴۲۲ قبلاً تصمیم‌گیری/منقضی، ۵۰۰ db.
///
/// # Panics
/// هیچ‌گاه روی ورودی کاربر — `expect` فقط روی شناسه‌های wire-safe دیتابیس.
pub async fn hitl_approve(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    bytes: axum::body::Bytes,
) -> ApiResult<Json<HitlDecisionResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let req: HitlDecisionRequest = crate::handlers::parse_body(&bytes)?;
    let note = req.note.as_deref().map(str::trim).filter(|n| !n.is_empty());
    let decision = state
        .store
        .approve_hitl_and_settle(&id, "admin", note)
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(HitlDecisionResponse {
        id: Id::new(decision.request.id).expect("hitl ids are wire-safe"),
        status: decision.request.status,
        decided_at: decision.request.decided_at.unwrap_or_default(),
        settled: decision.escrow.is_some(),
        tx_id: decision.tx.map(|t| t.id),
        replayed: decision.replayed,
    }))
}

/// `POST /admin/hitl/{id}/reject` — رد — هیچ پولی جابه‌جا نمی‌شود.
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۴۰۴ ناشناخته، ۴۲۲ قبلاً تصمیم‌گیری/منقضی، ۵۰۰ db.
///
/// # Panics
/// هیچ‌گاه روی ورودی کاربر — `expect` فقط روی شناسه‌های wire-safe دیتابیس.
pub async fn hitl_reject(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    bytes: axum::body::Bytes,
) -> ApiResult<Json<HitlDecisionResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let req: HitlDecisionRequest = crate::handlers::parse_body(&bytes)?;
    let note = req.note.as_deref().map(str::trim).filter(|n| !n.is_empty());
    let row = state.store.reject_hitl(&id, "admin", note).await.map_err(ApiFailure::from)?;
    Ok(Json(HitlDecisionResponse {
        id: Id::new(row.id).expect("hitl ids are wire-safe"),
        status: row.status,
        decided_at: row.decided_at.unwrap_or_default(),
        settled: false,
        tx_id: None,
        replayed: false,
    }))
}

/// `GET /admin/audit?limit=50` — دنبالهٔ ممیزی؛ جدیدترین اول.
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۵۰۰ db.
///
/// # Panics
/// هیچ‌گاه روی ورودی کاربر — `expect` فقط روی شناسه‌های wire-safe دیتابیس.
#[allow(clippy::implicit_hasher)] // Query extractor axum — hasher در اینجا بی‌معناست
pub async fn audit_tail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> ApiResult<Json<AdminAuditListResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let limit = q
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(50)
        .clamp(1, 200);
    let rows = state.store.list_audit(limit).await.map_err(ApiFailure::from)?;
    Ok(Json(AdminAuditListResponse {
        items: rows
            .into_iter()
            .map(|r| AdminAuditEntry {
                id: Id::new(r.id).expect("audit ids are wire-safe"),
                actor: r.actor,
                action: r.action,
                target: r.target,
                meta: r.meta,
                at: r.at,
            })
            .collect(),
    }))
}

/// `GET /admin/deposits?status=PENDING` — واریزهای TON همهٔ کاربران.
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۵۰۰ db.
///
/// # Panics
/// هیچ‌گاه روی ورودی کاربر — `expect` فقط روی شناسه‌های wire-safe دیتابیس.
#[allow(clippy::implicit_hasher)] // Query extractor axum — hasher در اینجا بی‌معناست
pub async fn deposits_list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> ApiResult<Json<AdminDepositListResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let status = q.get("status").map(|s| s.trim()).filter(|s| !s.is_empty());
    let rows = state.store.list_ton_deposits_all(status, 100).await.map_err(ApiFailure::from)?;
    Ok(Json(AdminDepositListResponse {
        items: rows
            .into_iter()
            .map(|d| AdminDepositView {
                id: Id::new(d.id).expect("deposit ids are wire-safe"),
                user_id: d.user_id,
                tx_hash: d.tx_hash,
                wallet_address: d.wallet_address,
                amount_ton: d.amount_ton,
                status: d.status,
                created_at: d.created_at,
                decided_at: d.confirmed_at,
                decided_by: d.confirmed_by,
            })
            .collect(),
    }))
}

/// `GET /admin/products` — همهٔ محصولات شامل غیرفعال‌ها (مدیریت بازار).
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۵۰۰ db.
pub async fn products_list(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResult<Json<AdminProductListResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let rows = state
        .store
        .list_products(&kian_db::ProductFilter { active_only: false, limit: 200, offset: 0, q: None })
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(AdminProductListResponse {
        items: rows
            .iter()
            .map(|p| ProductView {
                id: p.id.clone(),
                seller_id: p.seller_id.clone(),
                title: p.title.clone(),
                description: p.description.clone(),
                price_din: wire_din(p.price_din),
                category: p.category.clone(),
                preview_html: p.preview_html.clone(),
                affiliate_bps: p.affiliate_bps,
                artifact_kind: p.artifact_kind.clone(),
                active: p.active,
                created_at: p.created_at.clone(),
                mairon_score: p.mairon_score,
                mairon_badge: p.mairon_badge.clone(),
            })
            .collect(),
    }))
}

/// `DELETE /admin/products/{id}` — حذف محصول (مدیریت بازار).
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۴۰۴ ناشناخته، ۵۰۰ db.
pub async fn product_delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<axum::http::StatusCode> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let existing = state.store.get_product(&id).await.map_err(ApiFailure::from)?;
    if existing.is_none() {
        return Err(HttpFailure::product_not_found(&id).into());
    }
    state.store.delete_product(&id).await.map_err(ApiFailure::from)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_ai::client::UnavailableClient;
    use kian_ai::gateway::AiConfig;
    use kian_db::V3Store;
    use tower::ServiceExt;

    const ADMIN: &str = "test-admin-key-0123456789";

    async fn app(admin_key: Option<&str>) -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = admin_key.map(str::to_owned);
        cfg.ai = AiConfig { slots: vec![], daily_cap_tokens: 1_000 };
        crate::routes::build_router(
            crate::state::AppState::with_gateway(store, cfg, Arc::new(UnavailableClient)).shared(),
        )
    }

    async fn body_json(res: axum::response::Response) -> serde_json::Value {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    use std::sync::Arc;

    /// ماتریس گارد ادمین — بدون کلید سرور ۵۰۳، بدون هدر ۴۰۱، هدر نادرست ۴۰۱.
    #[tokio::test]
    async fn admin_guard_matrix_over_the_wire() {
        let app_off = app(None).await;
        let res = app_off
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/overview")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 503);
        assert_eq!(body_json(res).await["error"]["code"], "ADMIN_DISABLED");

        let app_on = app(Some(ADMIN)).await;
        for path in ["/admin/overview", "/admin/hitl", "/admin/audit", "/admin/deposits", "/admin/products"] {
            let res = app_on
                .clone()
                .oneshot(
                    axum::http::Request::builder()
                        .uri(format!("/api/v1{path}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), 401, "no header on {path}");
        }
        let res = app_on
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/overview")
                    .header("x-kian-key", "wrong-key-0123456789")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }

    /// overview با کلید — شمارنده‌های زندهٔ صفر + سری ۱۴ روزه خالی.
    #[tokio::test]
    async fn overview_shape_is_live() {
        let app = app(Some(ADMIN)).await;
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/overview")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = body_json(res).await;
        assert_eq!(body["users"], 0);
        assert_eq!(body["hitlPending"], 0);
        assert_eq!(body["mintedAllTime"], 0);
        assert!(body["revenueSeries"].is_array());
    }

    /// تصمیم HITL روی درخواست ناشناخته → ۴۰۴ کد دامنه.
    #[tokio::test]
    async fn hitl_decision_on_unknown_is_404() {
        let app = app(Some(ADMIN)).await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/hitl/hitl:unknown/approve")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 404);
        assert_eq!(body_json(res).await["error"]["code"], "HITL_NOT_FOUND");

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/hitl/hitl:unknown/reject")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 404);
    }

    /// حذف محصول ناشناخته → ۴۰۴ `PRODUCT_NOT_FOUND`.
    #[tokio::test]
    async fn product_delete_unknown_is_404() {
        let app = app(Some(ADMIN)).await;
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/admin/products/prod:nope")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 404);
        assert_eq!(body_json(res).await["error"]["code"], "PRODUCT_NOT_FOUND");
    }

    /// واریزهای ادمین — فهرست خالی سالم.
    #[tokio::test]
    async fn deposits_list_empty_is_ok() {
        let app = app(Some(ADMIN)).await;
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/deposits?status=PENDING")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = body_json(res).await;
        assert_eq!(body["items"].as_array().unwrap().len(), 0);
    }
}
