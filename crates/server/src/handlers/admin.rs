//! Admin panel handlers (M4) — overview, HITL queue, audit trail, TON deposits,
//! and product management.
//!
//! Security: every route runs `require_admin` with `X-KIAN-Key` — fail-closed: no key
//! yields server 503, missing header 401, wrong header 401 (constant-time comparison from contracts).
//! The HITL "approve ⇒ settle" decision is atomic — the 90/10 funds move exactly once
//! (`approve_hitl_and_settle`).

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use std::collections::HashMap;
use std::sync::Arc;

use kian_contracts::{
    AdminAuditEntry, AdminAuditListResponse, AdminDepositListResponse, AdminDepositView,
    AdminExportTrainingDatasetRequest, AdminExportTrainingDatasetResponse,
    AdminHitlListResponse, AdminHitlView, AdminLabDashboardView, AdminLabTelemetryResponse,
    AdminOverviewResponse, AdminProductListResponse, CohortHealthView, HitlDecisionRequest,
    HitlDecisionResponse, HttpFailure, LabTelemetryEntryView, ProductSalesView, ProductView,
    RevenuePoint, SellerSalesView,
};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::safe_wire_id;
use crate::middleware::require_admin;
use crate::state::AppState;
use crate::views::wire_din;
use kian_hermes::grok_ux_and_lab_engine::{
    build_admin_telemetry_dashboard, export_rlhf_training_dataset_jsonl, GROK_LAB_SIGNATURE,
};
use crate::handlers::hermes::load_lab_telemetry_entries;

/// `GET /admin/overview` — overview — every number comes from live ledger queries.
///
/// # Errors
/// 401/503 admin, 500 db.
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

/// `GET /admin/hitl?status=PENDING` — HITL queue; without a filter, all statuses.
///
/// # Errors
/// 401/503 admin, 500 db.
///
/// # Panics
/// Never on user input — `expect` only on wire-safe database IDs.
#[allow(clippy::implicit_hasher)] // axum Query extractor — hashing is meaningless here
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
                id: safe_wire_id(r.id, "hitl"),
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

/// `POST /admin/hitl/{id}/approve` — approve + atomic settle in the same command.
///
/// # Errors
/// 401/503 admin, 404 unknown, 422 already decided/expired, 500 db.
///
/// # Panics
/// Never on user input — `expect` only on wire-safe database IDs.
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
        id: safe_wire_id(decision.request.id, "hitl"),
        status: decision.request.status,
        decided_at: decision.request.decided_at.unwrap_or_default(),
        settled: decision.escrow.is_some(),
        tx_id: decision.tx.map(|t| t.id),
        replayed: decision.replayed,
    }))
}

/// `POST /admin/hitl/{id}/reject` — reject — no funds move.
///
/// # Errors
/// 401/503 admin, 404 unknown, 422 already decided/expired, 500 db.
///
/// # Panics
/// Never on user input — `expect` only on wire-safe database IDs.
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
        id: safe_wire_id(row.id, "hitl"),
        status: row.status,
        decided_at: row.decided_at.unwrap_or_default(),
        settled: false,
        tx_id: None,
        replayed: false,
    }))
}

/// `GET /admin/audit?limit=50` — audit trail; newest first.
///
/// # Errors
/// 401/503 admin, 500 db.
///
/// # Panics
/// Never on user input — `expect` only on wire-safe database IDs.
#[allow(clippy::implicit_hasher)] // axum Query extractor — hashing is meaningless here
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
                id: safe_wire_id(r.id, "audit"),
                actor: r.actor,
                action: r.action,
                target: r.target,
                meta: r.meta,
                at: r.at,
            })
            .collect(),
    }))
}

/// `GET /admin/deposits?status=PENDING` — TON deposits of all users.
///
/// # Errors
/// 401/503 admin, 500 db.
///
/// # Panics
/// Never on user input — `expect` only on wire-safe database IDs.
#[allow(clippy::implicit_hasher)] // axum Query extractor — hashing is meaningless here
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
                id: safe_wire_id(d.id, "deposit"),
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

/// `GET /admin/products` — all products including inactive ones (market management).
///
/// # Errors
/// 401/503 admin, 500 db.
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

/// `DELETE /admin/products/{id}` — delete product (market management).
///
/// # Errors
/// 401/503 admin, 404 unknown, 500 db.
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


// ── Wave 9 (Commit #29): 10-DNA lab observatory + RLHF/SFT export ─────
//
// Two admin endpoints on the persistent `UserLabTelemetryLog` telemetry:
// - [`lab_telemetry`] — logs all 10 cohorts + a deterministic aggregate dashboard;
// - [`export_training_dataset`] — exports SFT (ChatML/ShareGPT) and DPO
//   (`prompt/chosen/rejected`) datasets for fine-tuning models on real telemetry.

/// `GET /admin/lab-telemetry` — lab log observatory + cohort health
/// dashboard (X-KIAN-Key).
///
/// # Errors
/// 401 missing admin key; 500 db.
pub async fn lab_telemetry(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<AdminLabTelemetryResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let entries = load_lab_telemetry_entries(&state)
        .await
        .map_err(ApiFailure::from)?;
    let dashboard = build_admin_telemetry_dashboard(&entries);
    Ok(Json(AdminLabTelemetryResponse {
        entries: entries
            .iter()
            .map(|e| LabTelemetryEntryView {
                id: e.id.clone(),
                dna_code: e.dna_code.clone(),
                user_handle: e.user_handle.clone(),
                hour_offset: e.hour_offset,
                ux_tier: e.ux_tier.clone(),
                locale: e.locale.clone(),
                grok_persona: e.grok_persona.clone(),
                model_slug: e.model_slug.clone(),
                routing_mode: e.routing_mode.clone(),
                prompt_text: e.prompt_text.clone(),
                thinking_trace_json: e.thinking_trace_json.clone(),
                response_text: e.response_text.clone(),
                rea_score: e.rea_score,
                latency_ms: e.latency_ms,
                usable_for_training: e.usable_for_training,
                created_at: e.created_at.clone(),
            })
            .collect(),
        dashboard: AdminLabDashboardView {
            total_entries: dashboard.total_entries,
            cohort_health: dashboard
                .cohort_health
                .iter()
                .map(|c| CohortHealthView {
                    dna_code: c.dna_code.clone(),
                    user_handle: c.user_handle.clone(),
                    turns: c.turns,
                    avg_rea_score: c.avg_rea_score,
                    unusable: c.unusable,
                    friction_events: c.friction_events,
                    friction_recovered: c.friction_recovered,
                    dominant_model: c.dominant_model.clone(),
                    locales: c.locales.clone(),
                })
                .collect(),
            by_model: dashboard.by_model.clone(),
            by_locale: dashboard.by_locale.clone(),
            by_ux_tier: dashboard.by_ux_tier.clone(),
            by_persona: dashboard.by_persona.clone(),
            avg_rea_score: dashboard.avg_rea_score,
            min_rea_score: dashboard.min_rea_score,
            max_rea_score: dashboard.max_rea_score,
            unusable_entries: dashboard.unusable_entries,
            friction_events: dashboard.friction_events,
            friction_recovered: dashboard.friction_recovered,
        },
        signature: GROK_LAB_SIGNATURE.to_owned(),
    }))
}

/// `POST /admin/export-training-dataset` — exports an RLHF dataset from real
/// telemetry; only trainable entries with `rea_score >= min_rea_score`
/// (X-KIAN-Key).
///
/// # Errors
/// 401 missing admin key; 500 db.
pub async fn export_training_dataset(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    bytes: axum::body::Bytes,
) -> ApiResult<Json<AdminExportTrainingDatasetResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let req: AdminExportTrainingDatasetRequest = crate::handlers::parse_body(&bytes)?;
    let entries = load_lab_telemetry_entries(&state)
        .await
        .map_err(ApiFailure::from)?;
    let export = export_rlhf_training_dataset_jsonl(&entries, req.min_rea_score);
    Ok(Json(AdminExportTrainingDatasetResponse {
        sft_jsonl: export.sft_jsonl,
        dpo_jsonl: export.dpo_jsonl,
        sft_records: export.sft_records,
        dpo_records: export.dpo_records,
        min_rea_score: export.min_rea_score,
        signature: export.signature,
    }))
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

    /// Admin guard matrix — no key: server 503, missing header 401, wrong header 401.
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

    /// overview with key — zero live counters + an empty 14-day series.
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

    /// HITL decision on an unknown request → 404 domain code.
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

    /// Delete unknown product → 404 `PRODUCT_NOT_FOUND`.
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


    /// Lab observatory — 401 without key; with key: the 10-cohort entries +
    /// aggregate dashboard (simulation fallback on an empty store).
    #[tokio::test]
    async fn lab_telemetry_requires_admin_and_serves_dashboard() {
        // Disabled admin → 503 fail-closed; wrong key → 401
        let app_anon = app(None).await;
        let res = app_anon
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/lab-telemetry")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 503);
        let app_wrong = app(Some(ADMIN)).await;
        let res = app_wrong
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/lab-telemetry")
                    .header("x-kian-key", "wrong-key-0123456789")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);

        let app = app(Some(ADMIN)).await;
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/admin/lab-telemetry")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = body_json(res).await;
        assert_eq!(body["signature"], "KIA-GROK-UX-LAB-W9");
        let entries = body["entries"].as_array().unwrap();
        assert!(entries.len() >= 30, "30+ telemetry entries, got {}", entries.len());
        let dashboard = &body["dashboard"];
        assert_eq!(dashboard["cohortHealth"].as_array().unwrap().len(), 10);
        assert_eq!(dashboard["totalEntries"], entries.len());
        let models = dashboard["byModel"].as_object().unwrap();
        assert_eq!(
            models.values().map(|v| v.as_u64().unwrap_or(0)).sum::<u64>(),
            u64::try_from(entries.len()).unwrap_or(0),
            "model distribution partitions entries"
        );
        assert!(dashboard["avgReaScore"].as_u64().unwrap_or(0) >= 85, "REA gate >= 85");
    }

    /// RLHF export — 401 without key; with key: valid SFT/DPO JSONL with
    /// the REA threshold; a stricter threshold → fewer records.
    #[tokio::test]
    async fn export_training_dataset_requires_admin_and_emits_jsonl() {
        let app_anon = app(None).await;
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/admin/export-training-dataset")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"minReaScore":85}"#))
            .unwrap();
        assert_eq!(app_anon.oneshot(req).await.unwrap().status(), 503);

        let app = app(Some(ADMIN)).await;
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/admin/export-training-dataset")
            .header("x-kian-key", ADMIN)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"minReaScore":85}"#))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), 200);
        let body = body_json(res).await;
        assert_eq!(body["minReaScore"], 85);
        let sft = body["sftJsonl"].as_str().unwrap_or("");
        let dpo = body["dpoJsonl"].as_str().unwrap_or("");
        let sft_count = body["sftRecords"].as_u64().unwrap_or(0);
        assert!(sft_count > 0, "SFT records exported");
        for line in sft.lines().take(3) {
            let value: serde_json::Value = serde_json::from_str(line).expect("valid SFT JSONL");
            assert_eq!(value["messages"].as_array().unwrap().len(), 3);
        }
        for line in dpo.lines().take(3) {
            let value: serde_json::Value = serde_json::from_str(line).expect("valid DPO JSONL");
            assert!(value.get("chosen").is_some());
            assert!(value.get("rejected").is_some());
        }
        // Threshold 99 → fewer or equal records
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/admin/export-training-dataset")
            .header("x-kian-key", ADMIN)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"minReaScore":99}"#))
            .unwrap();
        let body = body_json(app.oneshot(req).await.unwrap()).await;
        assert!(
            body["sftRecords"].as_u64().unwrap_or(0) <= sft_count,
            "stricter gate never grows the dataset"
        );
    }

    /// Admin deposits — healthy empty list.
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
