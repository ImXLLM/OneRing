//! هندلرهای گیت‌وی AI (M4) — وضعیت صادقانه، گفت‌وگو با پاکت بودجه، مصرف امروز.
//!
//! امنیت: `chat` و `usage` نشست bearer می‌خواهند (هویت هرگز از بدنه نمی‌آید)؛
//! `status` عمومی است ولی هیچ کلیدی در آن نیست — فقط نام/مدل/فعال بودن اسلات‌ها.
//! بودجه: گیت سخت قبل از هر فراخوانی (`used + estimate ≤ cap`) و ثبت مصرف بعد
//! از آن — سقف روزانه از پیکربندی (`KIAN_AI_DAILY_TOKENS`).

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use std::sync::Arc;

use kian_ai::budget::{BudgetDecision, BudgetEnvelope};
use kian_ai::gateway::PacketMessage;
use kian_contracts::{
    AiChatRequest, AiChatResponse, AiSlotView, AiStatusResponse, AiUsageResponse, HttpFailure,
};

use crate::error::{ApiFailure, ApiResult};
use crate::middleware::require_session;
use crate::state::AppState;

/// برآورد توکن یک فراخوانی عمومی چت — پاکت سخت؛ مهارت‌های هرمس با برآورد
/// خودشان گیت می‌خورند، اینجا فقط `/ai/chat` خام است.
const CHAT_ESTIMATE_TOKENS: i64 = 1_000;

/// `GET /ai/status` — عمومی؛ چیپ صادقانهٔ health و داشبورد.
///
/// # Errors
/// هیچ — همیشه ۲۰۰ (وضعیت صادقانه حتی وقتی Disabled).
pub async fn status(State(state): State<Arc<AppState>>) -> ApiResult<Json<AiStatusResponse>> {
    let slots = state
        .ai
        .slot_statuses()
        .into_iter()
        .map(|s| AiSlotView { slot: s.slot, name: s.name, model: s.model, enabled: s.enabled })
        .collect();
    Ok(Json(AiStatusResponse {
        enabled: state.ai.enabled(),
        daily_cap: state.config.ai.daily_cap_tokens,
        slots,
    }))
}

/// `POST /ai/chat` — نشست‌دار؛ redaction → گیت بودجه → فال‌بک سه‌اسلات →
/// ثبت مصرف. همهٔ شکست‌ها تایپ‌شده و صادقانه.
///
/// # Errors
/// ۴۰۱ نشست، ۴۲۹ سقف روزانه، ۵۰۳ گیت‌وی غیرفعال، ۵۰۲ همهٔ اسلات‌ها شکست.
pub async fn chat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    bytes: axum::body::Bytes,
) -> ApiResult<Json<AiChatResponse>> {
    let user = require_session(&state, &headers).await?;
    let req: AiChatRequest = crate::handlers::parse_body(&bytes)?;
    // اعتبارسنجی دامنه‌ای روی سیم — تعداد و اندازهٔ پیام‌ها
    if req.messages.is_empty() || req.messages.len() > 20 {
        return Err(HttpFailure::invalid_request(1).into());
    }
    if req.messages.iter().any(|m| m.content.is_empty() || m.content.len() > 8_000) {
        return Err(HttpFailure::invalid_request(1).into());
    }

    let Some(budget) = BudgetEnvelope::new(state.config.ai.daily_cap_tokens) else {
        // سقف منفی = پیکربندی خراب — صادقانه غیرفعال، هرگز باز
        return Err(HttpFailure::ai_disabled().into());
    };
    let (used_today, _calls) = state.store.ai_tokens_today(&user.id).await?;
    if let BudgetDecision::Exhausted { used, cap } = budget.precheck(used_today, CHAT_ESTIMATE_TOKENS) {
        return Err(HttpFailure::ai_cap_exceeded(used, cap).into());
    }

    let messages: Vec<PacketMessage> = req
        .messages
        .into_iter()
        .map(|m| PacketMessage { role: m.role, content: m.content })
        .collect();
    let result = state.ai.chat(messages).await.map_err(|e| ApiFailure::from(match e {
        kian_ai::gateway::GatewayError::Disabled => HttpFailure::ai_disabled(),
        kian_ai::gateway::GatewayError::AllSlotsFailed { attempts } => {
            let detail = attempts
                .iter()
                .map(|a| format!("{}: {}", a.slot, a.detail))
                .collect::<Vec<_>>()
                .join("; ");
            HttpFailure::ai_upstream("ALL_SLOTS_FAILED", detail)
        }
    }))?;

    let total = result.total_tokens;
    state
        .store
        .insert_ai_usage(&kian_db::AiUsageRow {
            user_id: user.id.clone(),
            agent: "chat".to_owned(),
            provider: result.provider.clone(),
            model: result.model.clone(),
            kind: "chat".to_owned(),
            prompt_tokens: result.prompt_tokens,
            completion_tokens: result.completion_tokens,
            total_tokens: total,
            outcome: "ai.ok".to_owned(),
        })
        .await
        .map_err(ApiFailure::from)?;

    Ok(Json(AiChatResponse {
        content: result.content,
        provider: result.provider,
        model: result.model,
        prompt_tokens: result.prompt_tokens,
        completion_tokens: result.completion_tokens,
        total_tokens: total,
        redactions: result.redactions,
        remaining_tokens: budget.remaining(used_today + total),
    }))
}

/// `GET /ai/usage` — مصرف امروز من — مبنای نمایش پاکت بودجه در UI.
///
/// # Errors
/// ۴۰۱ نشست؛ ۵۰۰ خطای db — خواندن مصرف امروز.
pub async fn usage(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<AiUsageResponse>> {
    let user = require_session(&state, &headers).await?;
    let (total_tokens, calls) = state.store.ai_tokens_today(&user.id).await?;
    Ok(Json(AiUsageResponse {
        total_tokens,
        calls,
        daily_cap: state.config.ai.daily_cap_tokens,
    }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_ai::client::ScriptedClient;
    use kian_ai::gateway::{AiConfig, SlotConfig};
    use kian_db::V3Store;
    use tower::ServiceExt;

    const BOT: &str = "123456:TEST-BOT-TOKEN-M4";
    const ADMIN: &str = "test-admin-key-0123456789";

    async fn app(slots: Vec<SlotConfig>) -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some(ADMIN.to_owned());
        cfg.bot_token = Some(BOT.to_owned());
        cfg.ai = AiConfig { slots, daily_cap_tokens: 1_000 };
        let scripted = ScriptedClient::new(vec![
            Ok(kian_ai::gateway::CompletionPacket {
                content: "پاسخ ۱".into(),
                prompt_tokens: 30,
                completion_tokens: 20,
            }),
            Ok(kian_ai::gateway::CompletionPacket {
                content: "پاسخ ۲".into(),
                prompt_tokens: 10,
                completion_tokens: 5,
            }),
        ]);
        crate::routes::build_router(
            crate::state::AppState::with_gateway(store, cfg, Arc::new(scripted)).shared(),
        )
    }

    async fn token(app: &axum::Router) -> String {
        let init_data = kian_contracts::make_test_init_data(BOT, "12345", Some("ali"));
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(format!(r#"{{"initData":"{init_data}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "helper: auth");
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["token"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    async fn body_json(res: axum::response::Response) -> serde_json::Value {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// status بدون اسلات — صادقانه disabled؛ با اسلات — enabled با مدل.
    #[tokio::test]
    async fn status_is_honest_about_slots() {
        let app_off = app(Vec::new()).await;
        let res = app_off
            .clone()
            .oneshot(axum::http::Request::builder().uri("/api/v1/ai/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = body_json(res).await;
        assert_eq!(body["enabled"], false, "no slots = disabled");
        assert_eq!(body["slots"].as_array().unwrap().len(), 0);

        let app_on = app(vec![SlotConfig {
            name: "primary".into(),
            url: "http://x".into(),
            key: "k".into(),
            model: "test-model".into(),
        }])
        .await;
        let res = app_on
            .oneshot(axum::http::Request::builder().uri("/api/v1/ai/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = body_json(res).await;
        assert_eq!(body["enabled"], true);
        assert_eq!(body["slots"][0]["model"], "test-model");
        let wire = body.to_string();
        assert!(!wire.contains("\"k\""), "slot key must never appear on the wire");
    }

    /// chat بدون نشست → ۴۰۱؛ با نشست و گیت‌وی فعال → ۲۰۰ + ثبت مصرف.
    #[tokio::test]
    async fn chat_requires_session_and_records_usage() {
        let app = app(vec![SlotConfig {
            name: "primary".into(),
            url: "http://x".into(),
            key: "k".into(),
            model: "test-model".into(),
        }])
        .await;

        // بدون نشست
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/ai/chat")
                    .body(Body::from(r#"{"messages":[{"role":"user","content":"سلام"}]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401, "no session");
        let err = body_json(res).await;
        assert_eq!(err["error"]["code"], "SESSION_REQUIRED");

        // با نشست
        let tok = token(&app).await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/ai/chat")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from(r#"{"messages":[{"role":"user","content":"سلام"}]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "chat must succeed");
        let body = body_json(res).await;
        assert_eq!(body["provider"], "primary");
        assert_eq!(body["totalTokens"], 50);
        assert_eq!(body["remainingTokens"], 950);

        // usage — مصرف ثبت شده
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/ai/usage")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = body_json(res).await;
        assert_eq!(body["totalTokens"], 50);
        assert_eq!(body["calls"], 1);
        assert_eq!(body["dailyCap"], 1_000);
    }

    /// گیت‌وی غیرفعال → ۵۰۳ `AI_DISABLED` صادقانه — هیچ دروغی نمی‌شود.
    #[tokio::test]
    async fn chat_without_slots_is_503_ai_disabled() {
        let app = app(Vec::new()).await;
        let tok = token(&app).await;
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/ai/chat")
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::from(r#"{"messages":[{"role":"user","content":"سلام"}]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 503);
        let err = body_json(res).await;
        assert_eq!(err["error"]["code"], "AI_DISABLED");
    }
}
