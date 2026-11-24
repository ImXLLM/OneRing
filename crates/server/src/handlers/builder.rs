//! هندلرهای استودیوی ساخت محصول HTML با هوش مصنوعی (فاز ۴).
//!
//! همهٔ مسیرهای این ماژول با نشست Bearer شهروند (`require_session`) احراز هویت
//! می‌شوند و مالکیت جلسه (`accountId`) در تمام خواندن‌ها و نوشتن‌ها کنترل می‌شود.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;

use kian_ai::budget::{BudgetDecision, BudgetEnvelope};
use kian_contracts::{
    BuilderChatRequest, BuilderChatResponse, BuilderMessageView, BuilderSessionListResponse,
    BuilderSessionSummary, BuilderSessionView, CreateBuilderSessionRequest, HttpFailure,
    ProductView, PublishBuilderRequest, PublishBuilderResponse, MAX_DELIVERY_PAYLOAD_BYTES,
};
use kian_db::{
    now_iso, AiUsageRow, BuilderMessageRow, BuilderSessionPatch, BuilderSessionRow, ProductRow,
};
use kian_domain::{DomainError, DomainErrorCode, DEFAULT_AFFILIATE_BPS, MAX_AFFILIATE_BPS};
use kian_hermes::{
    build_preview_html, BuilderEngine, BUILDER_ESTIMATE_TOKENS, BUILDER_HISTORY_WINDOW,
    BUILDER_WELCOME_MESSAGE,
};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::require_session;
use crate::state::AppState;

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

fn message_view(m: &BuilderMessageRow) -> BuilderMessageView {
    BuilderMessageView {
        id: m.id,
        role: m.role.clone(),
        content: m.content.clone(),
        created_at: m.created_at,
    }
}

fn session_summary(s: &BuilderSessionRow) -> BuilderSessionSummary {
    BuilderSessionSummary {
        id: s.id.clone(),
        stage: s.stage.clone(),
        title: s.title.clone(),
        description: s.description.clone(),
        suggested_price: s.suggested_price,
        affiliate_bps: s.affiliate_bps,
        has_html: !s.generated_html.trim().is_empty(),
        published_product_id: s.published_product_id.clone(),
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

fn session_view(s: &BuilderSessionRow, messages: &[BuilderMessageRow]) -> BuilderSessionView {
    let preview_html = build_preview_html(&s.generated_html, &s.title);
    BuilderSessionView {
        id: s.id.clone(),
        stage: s.stage.clone(),
        title: s.title.clone(),
        description: s.description.clone(),
        suggested_price: s.suggested_price,
        affiliate_bps: s.affiliate_bps,
        generated_html: s.generated_html.clone(),
        preview_html,
        published_product_id: s.published_product_id.clone(),
        messages: messages.iter().map(message_view).collect(),
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

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

/// `POST /builder/sessions` — ساخت جلسهٔ جدید استودیو همراه با پیام خوش‌آمدگویی اولیه.
///
/// # Errors
/// ۴۰۱ نشست نامعتبر؛ ۴۰۰ بدنهٔ نامعتبر؛ ۵۰۰ خطای SQL.
pub async fn create_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<BuilderSessionView>)> {
    let user = require_session(&state, &headers).await?;
    let req: CreateBuilderSessionRequest = parse_body(&body)?;
    let initial_title = req
        .initial_idea
        .as_deref()
        .map(|s| s.trim().chars().take(120).collect::<String>())
        .unwrap_or_default();

    let now = now_unix();
    let session = BuilderSessionRow {
        id: format!("bldr:{}", uuid::Uuid::now_v7()),
        account_id: user.account_id.clone(),
        stage: "discovery".to_owned(),
        title: initial_title,
        description: String::new(),
        suggested_price: 100,
        affiliate_bps: DEFAULT_AFFILIATE_BPS,
        generated_html: String::new(),
        published_product_id: None,
        created_at: now,
        updated_at: now,
    };
    state
        .store
        .create_builder_session(&session)
        .await
        .map_err(ApiFailure::from)?;
    let msg_id = state
        .store
        .append_builder_message(&session.id, "assistant", BUILDER_WELCOME_MESSAGE, now)
        .await
        .map_err(ApiFailure::from)?;
    let welcome = BuilderMessageRow {
        id: msg_id,
        session_id: session.id.clone(),
        role: "assistant".to_owned(),
        content: BUILDER_WELCOME_MESSAGE.to_owned(),
        created_at: now,
    };
    Ok((StatusCode::CREATED, Json(session_view(&session, &[welcome]))))
}

/// `GET /builder/sessions` — فهرست جلسات سازندهٔ کاربر (جدیدترین اول).
///
/// # Errors
/// ۴۰۱ نشست نامعتبر؛ ۵۰۰ خطای SQL.
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<BuilderSessionListResponse>> {
    let user = require_session(&state, &headers).await?;
    let rows = state
        .store
        .list_builder_sessions_for_account(&user.account_id, 50)
        .await
        .map_err(ApiFailure::from)?;
    let items = rows.iter().map(session_summary).collect();
    Ok(Json(BuilderSessionListResponse { items }))
}

/// `GET /builder/sessions/{id}` — جزئیات کامل جلسهٔ متعلق به کاربر + تاریخچهٔ پیام‌ها.
///
/// # Errors
/// ۴۰۱ نشست نامعتبر؛ ۴۰۴ جلسه ناشناخته یا متعلق به کاربر دیگر؛ ۵۰۰ خطای SQL.
pub async fn get_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<BuilderSessionView>> {
    let user = require_session(&state, &headers).await?;
    let session = state
        .store
        .get_builder_session(&id, &user.account_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else(|| ApiFailure::Http(HttpFailure::builder_session_not_found(&id)))?;
    let messages = state
        .store
        .list_builder_messages(&session.id, 50)
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(session_view(&session, &messages)))
}

/// `POST /builder/sessions/{id}/chat` — یک نوبت گفتگوی چندمرحله‌ای با استودیوی AI.
///
/// پیام کاربر ثبت می‌شود، گیت بودجه (در صورت فعال بودن LLM خارجی) بررسی می‌شود،
/// موتور Builder خروجی (یا مولد تعاملی داخلی Fallback) را می‌سازد و جلسه به‌روز می‌گردد.
///
/// # Errors
/// ۴۰۱ نشست؛ ۴۰۰ پیام خالی/بلند؛ ۴۰۴ جلسه ناشناخته؛ ۴۲۹ اتمام بودجهٔ روزانهٔ AI؛ ۵۰۰ خطای SQL.
pub async fn chat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<Json<BuilderChatResponse>> {
    let user = require_session(&state, &headers).await?;
    let req: BuilderChatRequest = parse_body(&body)?;
    let user_msg = req.message.trim().to_owned();
    let msg_chars = user_msg.chars().count();
    if !(1..=4_000).contains(&msg_chars) {
        return Err(HttpFailure::invalid_request(1).into());
    }

    let session = state
        .store
        .get_builder_session(&id, &user.account_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else(|| ApiFailure::Http(HttpFailure::builder_session_not_found(&id)))?;

    // کنترل بودجهٔ روزانه فقط وقتی اسلات LLM خارجی فعال است
    if state.ai.enabled() {
        let Some(budget) = BudgetEnvelope::new(state.config.ai.daily_cap_tokens) else {
            return Err(HttpFailure::ai_disabled().into());
        };
        let (used_today, _calls) = state.store.ai_tokens_today(&user.id).await?;
        if let BudgetDecision::Exhausted { used, cap } =
            budget.precheck(used_today, BUILDER_ESTIMATE_TOKENS)
        {
            return Err(HttpFailure::ai_cap_exceeded(used, cap).into());
        }
    }

    let now = now_unix();
    state
        .store
        .append_builder_message(&session.id, "user", &user_msg, now)
        .await
        .map_err(ApiFailure::from)?;

    let history = state
        .store
        .list_builder_messages(&session.id, BUILDER_HISTORY_WINDOW)
        .await
        .map_err(ApiFailure::from)?;

    let force_generate = req.force_generate.unwrap_or(false);
    let outcome =
        BuilderEngine::run_turn(&state.ai, &session, &history, &user_msg, force_generate).await;

    let reply_ts = now_unix();
    state
        .store
        .append_builder_message(&session.id, "assistant", &outcome.reply, reply_ts)
        .await
        .map_err(ApiFailure::from)?;

    let patch = BuilderSessionPatch {
        stage: outcome.stage.clone(),
        title: outcome.title.clone(),
        description: outcome.description.clone(),
        suggested_price: outcome.suggested_price,
        affiliate_bps: outcome.affiliate_bps,
        generated_html: outcome.generated_html.clone(),
        published_product_id: session.published_product_id.clone(),
        updated_at: reply_ts,
    };
    state
        .store
        .update_builder_session_artifact(&session.id, &user.account_id, &patch)
        .await
        .map_err(ApiFailure::from)?;

    // ثبت مصرف توکن واقعی در صورت اجرای LLM خارجی
    if !outcome.used_fallback && outcome.total_tokens > 0 {
        state
            .store
            .insert_ai_usage(&AiUsageRow {
                user_id: user.id.clone(),
                agent: "builder".to_owned(),
                provider: outcome.provider.clone().unwrap_or_else(|| "primary".to_owned()),
                model: outcome.model.clone().unwrap_or_default(),
                kind: "builder".to_owned(),
                prompt_tokens: outcome.prompt_tokens,
                completion_tokens: outcome.completion_tokens,
                total_tokens: outcome.total_tokens,
                outcome: "builder.ok".to_owned(),
            })
            .await
            .map_err(ApiFailure::from)?;
    }

    let updated_session = BuilderSessionRow {
        id: session.id.clone(),
        account_id: session.account_id,
        stage: patch.stage,
        title: patch.title,
        description: patch.description,
        suggested_price: patch.suggested_price,
        affiliate_bps: patch.affiliate_bps,
        generated_html: patch.generated_html,
        published_product_id: patch.published_product_id,
        created_at: session.created_at,
        updated_at: patch.updated_at,
    };
    let full_messages = state
        .store
        .list_builder_messages(&updated_session.id, 50)
        .await
        .map_err(ApiFailure::from)?;

    Ok(Json(BuilderChatResponse {
        reply: outcome.reply,
        session: session_view(&updated_session, &full_messages),
        used_fallback: outcome.used_fallback,
        provider: outcome.provider,
        total_tokens: outcome.total_tokens,
    }))
}

/// `POST /builder/sessions/{id}/publish` — انتشار یک‌کلیکهٔ محصول ساخته‌شده در بازار کیان.
///
/// کاربر می‌تواند مقادیر پیشنهادی AI را تأیید یا در بدنه ویرایش کند. در صورت
/// انتشار قبلی، همان محصول به‌صورت idempotent همراه با `shareStartParam` برگردانده می‌شود.
///
/// # Errors
/// ۴۰۱ نشست؛ ۴۰۰ ورودی نامعتبر؛ ۴۰۴ جلسه ناشناخته؛ ۴۲۲ نبود کد HTML یا `BPS_OUT_OF_RANGE`؛ ۵۰۰ خطای SQL.
pub async fn publish(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<PublishBuilderResponse>)> {
    let user = require_session(&state, &headers).await?;
    let req: PublishBuilderRequest = parse_body(&body)?;

    let session = state
        .store
        .get_builder_session(&id, &user.account_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else(|| ApiFailure::Http(HttpFailure::builder_session_not_found(&id)))?;

    // اگر قبلاً منتشر شده و محصول در جدول Product موجود است، همان را برگردان (idempotent)
    if let Some(existing_pid) = &session.published_product_id
        && let Some(existing) = state
            .store
            .get_product(existing_pid)
            .await
            .map_err(ApiFailure::from)?
    {
        let share_start_param = format!("p_{}_r_{}", existing.id, user.telegram_id);
        return Ok((
            StatusCode::OK,
            Json(PublishBuilderResponse {
                product: product_view(&existing),
                session_id: session.id,
                share_start_param,
                already_published: true,
            }),
        ));
    }

    if session.generated_html.trim().is_empty() {
        return Err(ApiFailure::Http(HttpFailure::builder_artifact_empty()));
    }
    if session.generated_html.len() > MAX_DELIVERY_PAYLOAD_BYTES {
        return Err(HttpFailure::invalid_request(1).into());
    }

    let title = req
        .title
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map_or_else(
            || {
                if session.title.trim().is_empty() {
                    "محصول تعاملی HTML".to_owned()
                } else {
                    session.title.trim().to_owned()
                }
            },
            str::to_owned,
        );
    let description = req
        .description
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map_or_else(
            || {
                if session.description.trim().is_empty() {
                    format!("وب‌اپلیکیشن تعاملی تک‌صفحه‌ای {title} با تحویل آنی در بازار کیان.")
                } else {
                    session.description.trim().to_owned()
                }
            },
            str::to_owned,
        );
    let price_din = req
        .price_din
        .map_or_else(|| session.suggested_price.max(1), |d| d.get());
    let affiliate_bps = req.affiliate_bps.unwrap_or(session.affiliate_bps);

    if !(1..=120).contains(&title.chars().count())
        || !(1..=2_000).contains(&description.chars().count())
        || price_din < 1
        || req.category.as_ref().is_some_and(|c| c.chars().count() > 40)
    {
        return Err(HttpFailure::invalid_request(1).into());
    }
    if affiliate_bps > MAX_AFFILIATE_BPS {
        return Err(DomainError::new(
            DomainErrorCode::BpsOutOfRange,
            format!("affiliate_bps must be in [0, {MAX_AFFILIATE_BPS}], got {affiliate_bps}"),
        )
        .into());
    }

    let preview_html = build_preview_html(&session.generated_html, &title);
    let product = ProductRow {
        id: format!("prod:{}", uuid::Uuid::now_v7()),
        seller_id: user.account_id.clone(),
        title: title.clone(),
        description: description.clone(),
        category: req.category.or_else(|| Some("html-app".to_owned())),
        price_din,
        preview_html,
        affiliate_bps,
        artifact_kind: "html".to_owned(),
        delivery_payload: Some(session.generated_html.clone()),
        active: true,
        created_at: now_iso(),
    };
    state
        .store
        .insert_product(&product)
        .await
        .map_err(ApiFailure::from)?;

    let patch = BuilderSessionPatch {
        stage: "published".to_owned(),
        title,
        description,
        suggested_price: price_din,
        affiliate_bps,
        generated_html: session.generated_html,
        published_product_id: Some(product.id.clone()),
        updated_at: now_unix(),
    };
    state
        .store
        .update_builder_session_artifact(&session.id, &user.account_id, &patch)
        .await
        .map_err(ApiFailure::from)?;

    let share_start_param = format!("p_{}_r_{}", product.id, user.telegram_id);
    Ok((
        StatusCode::CREATED,
        Json(PublishBuilderResponse {
            product: product_view(&product),
            session_id: session.id,
            share_start_param,
            already_published: false,
        }),
    ))
}

#[cfg(test)]
mod builder_tests {
    use std::sync::Arc;

    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_ai::client::{ScriptedClient, UnavailableClient};
    use kian_ai::gateway::{AiConfig, CompletionPacket, SlotConfig};
    use kian_contracts::make_test_init_data;
    use kian_db::V3Store;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    const BOT: &str = "123456:TEST-BOT-TOKEN-BUILDER";
    const ADMIN: &str = "test-admin-key-0123456789";

    fn base_cfg() -> crate::config::ServerConfig {
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some(ADMIN.to_owned());
        cfg.bot_token = Some(BOT.to_owned());
        cfg
    }

    async fn app_without_llm() -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        let mut cfg = base_cfg();
        cfg.ai = AiConfig { slots: vec![], daily_cap_tokens: 10_000 };
        crate::routes::build_router(
            crate::state::AppState::with_gateway(store, cfg, Arc::new(UnavailableClient)).shared(),
        )
    }

    async fn app_with_scripted_llm(script: Vec<Result<CompletionPacket, String>>) -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        let mut cfg = base_cfg();
        cfg.ai = AiConfig {
            slots: vec![SlotConfig {
                name: "primary".into(),
                url: "http://mock-llm".into(),
                key: "k".into(),
                model: "studio-v1".into(),
            }],
            daily_cap_tokens: 10_000,
        };
        crate::routes::build_router(
            crate::state::AppState::with_gateway(store, cfg, Arc::new(ScriptedClient::new(script)))
                .shared(),
        )
    }

    async fn json_body(res: axum::response::Response) -> Value {
        let b = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&b).unwrap()
    }

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

    /// جریان کامل سرتاسری بدون کلید LLM خارجی (با Fallback داخلی):
    /// create session → discovery → generate HTML → publish → purchase with referral → delivery → release
    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn full_builder_to_market_referral_purchase_flow_with_fallback() {
        let app = app_without_llm().await;
        let creator = auth(&app, "801", "maker").await;
        let buyer = auth(&app, "802", "shopper").await;
        let referrer = auth(&app, "803", "sharer").await;
        mint(&app, "citizen:802", 1_000).await;

        // ۱) ساخت جلسهٔ استودیو با پیام خوش‌آمدگویی
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/builder/sessions")
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let created = json_body(res).await;
        let session_id = created["id"].as_str().unwrap().to_owned();
        assert_eq!(created["stage"], "discovery");
        assert_eq!(created["messages"].as_array().unwrap().len(), 1);

        // مالکیت: کاربر دیگر نمی‌تواند جلسه را ببیند → ۴۰۴
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/builder/sessions/{session_id}"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 404);

        // انتشار پیش از تولید HTML → ۴۲۲ BUILDER_ARTIFACT_EMPTY
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/publish"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
        assert_eq!(json_body(res).await["error"]["code"], "BUILDER_ARTIFACT_EMPTY");

        // ۲) نوبت اول چت (Discovery)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/chat"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(json!({ "message": "سلام ایده دارم" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let turn1 = json_body(res).await;
        assert_eq!(turn1["usedFallback"], true);
        assert_eq!(turn1["session"]["stage"], "discovery");

        // ۳) نوبت دوم چت (ساخت کامل HTML)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/chat"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(
                        json!({
                            "message": "یک ماشین حساب سود مرکب ارز دیجیتال با تم تیره بساز",
                            "forceGenerate": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let turn2 = json_body(res).await;
        assert_eq!(turn2["session"]["stage"], "ready");
        let generated_html = turn2["session"]["generatedHtml"].as_str().unwrap().to_owned();
        assert!(generated_html.contains("<!DOCTYPE html>"));
        assert!(turn2["session"]["previewHtml"]
            .as_str()
            .unwrap()
            .contains("نسخهٔ پیش‌نمایش زنده در KIAN Market"));

        // ۴) انتشار یک‌کلیکه در بازار با پورسانت ۲۰٪ و قیمت ۴۰۰ Đ
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/publish"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(
                        json!({
                            "priceDin": 400,
                            "affiliateBps": 2_000
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let published = json_body(res).await;
        let product_id = published["product"]["id"].as_str().unwrap().to_owned();
        assert_eq!(published["alreadyPublished"], false);
        assert_eq!(published["shareStartParam"], format!("p_{product_id}_r_801"));

        // انتشار مجدد idempotent است و همان محصول را برمی‌گرداند
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/publish"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(json_body(res).await["alreadyPublished"], true);

        // ۵) خرید با رفرال شهروند ۸۰۳
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/purchase")
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(
                        json!({ "productId": product_id, "referrer": "803" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let escrow_id = json_body(res).await["escrow"]["id"].as_str().unwrap().to_owned();

        // ۶) دریافت کد کامل HTML توسط خریدار
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
        assert_eq!(json_body(res).await["deliveryPayload"], generated_html);

        // ۷) آزادسازی و تسویه ۳ ذی‌نفع (۴۰۰ → فروشنده ۲۸۰ + خزانه ۴۰ + معرف ۸۰)
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

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["balance"], 280);

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {referrer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["balance"], 80);
    }

    /// اجرای واقعی با اسلات LLM اسکریپت‌شده: استخراج HTML + JSON CAST و ثبت مصرف توکن.
    #[tokio::test]
    async fn builder_chat_with_scripted_llm_extracts_artifact_and_records_tokens() {
        let llm_reply = r#"محصولت آماده شد!

```html
<!DOCTYPE html>
<html lang="fa" dir="rtl"><body><main><h1>ابزار تایمر پومودورو</h1></main></body></html>
```

```json
{
  "title": "تایمر تمرکز پومودورو",
  "description": "تمرکزت را دو برابر کن با تایمر تک‌صفحه‌ای سبک و آفلاین.",
  "suggested_price": 320,
  "affiliate_bps": 1500
}
```"#;
        let app = app_with_scripted_llm(vec![Ok(CompletionPacket {
            content: llm_reply.into(),
            prompt_tokens: 210,
            completion_tokens: 490,
        })])
        .await;
        let maker = auth(&app, "811", "llm_maker").await;

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/builder/sessions")
                    .header("authorization", format!("Bearer {maker}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let session_id = json_body(res).await["id"].as_str().unwrap().to_owned();

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/chat"))
                    .header("authorization", format!("Bearer {maker}"))
                    .body(Body::from(
                        json!({ "message": "یک تایمر پومودورو بساز", "forceGenerate": true })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["usedFallback"], false);
        assert_eq!(body["provider"], "primary");
        assert_eq!(body["totalTokens"], 700);
        assert_eq!(body["session"]["stage"], "ready");
        assert_eq!(body["session"]["title"], "تایمر تمرکز پومودورو");
        assert_eq!(body["session"]["suggestedPrice"], 320);
        assert_eq!(body["session"]["affiliateBps"], 1_500);
        assert!(body["session"]["generatedHtml"]
            .as_str()
            .unwrap()
            .contains("تایمر پومودورو"));

        // مصرف توکن در /ai/usage ثبت شده است
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/ai/usage")
                    .header("authorization", format!("Bearer {maker}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["totalTokens"], 700);
    }
}
