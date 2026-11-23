//! هندلرهای ناوگان A2A/MCP (M4) — کارت کشف، ناوگان، پیام A2A، ابزار MCP.
//!
//! امنیت: کارت و ناوگان و فهرست ابزارها عمومی (شفافیت — هیچ کلیدی در پاسخ
//! نیست)؛ ارسال/صندوق A2A و invoke ابزارهای MCP **ادمین** با `X-KIAN-Key` —
//! ایجنت‌ها زیرساخت قابل‌اعتمادند، نه شهروند.
//!
//! بوت: [`seed_agents`] — ۷ نقش از `kian_agents::ROLES`؛ seed فقط بار اول
//! می‌نویسد و وضعیت ادمین را هرگز بازنویسی نمی‌کند.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use std::sync::Arc;

use kian_agents::mcp::{tool_def, validate_args, TOOLS};
use kian_agents::roles::{role_of, seed_agents as role_seed};
use kian_contracts::{
    A2aInboxResponse, A2aMessageView, A2aSendRequest, AgentCardProvider, AgentCardResponse,
    AgentCardSkill, AgentView, FleetResponse, HttpFailure, Id, McpInvokeRequest, McpInvokeResponse,
    McpToolView,
};
use kian_db::A2aRow;

use crate::error::{ApiFailure, ApiResult};
use crate::middleware::require_admin;
use crate::state::AppState;

/// بذر ناوگان — از بوت (`main`)؛ فقط ردیف‌های غایب.
///
/// # Errors
/// خطای SQL.
pub async fn seed_agents(store: &kian_db::V3Store) -> Result<(), kian_db::DbError> {
    for (id, name, role, status, description) in role_seed() {
        store
            .seed_agent(&kian_db::AgentRow {
                id: id.to_owned(),
                name: name.to_owned(),
                role: role.to_owned(),
                status: status.to_owned(),
                description: description.to_owned(),
            })
            .await?;
    }
    Ok(())
}

/// `GET /.well-known/agent-card.json` — کشف عمومی A2A؛ مهارت‌ها از هرمس.
///
/// # Errors
/// ۵۰۰ خطای db — خواندن مهارت‌های فعال.
pub async fn agent_card(State(state): State<Arc<AppState>>) -> ApiResult<Json<AgentCardResponse>> {
    let skills = state
        .store
        .list_skills(true)
        .await
        .map_err(ApiFailure::from)?
        .into_iter()
        .map(|s| AgentCardSkill {
            id: s.id,
            name: s.title,
            description: s.description,
            tags: vec![s.kind],
        })
        .collect();
    let base = format!("http://{}:{}/api/v1", state.config.host, state.config.port);
    Ok(Json(AgentCardResponse {
        name: "KIAN Kernel".into(),
        description: "Money kernel + Hermes brain — the KIAN ecosystem brain".into(),
        url: base.clone(),
        version: crate::VERSION.to_owned(),
        // نسخهٔ پروتکل صادقانه — پیاده‌سازی خودِ کیان از مفهوم A2A
        protocol_version: kian_agents::card::PROTOCOL_VERSION.to_owned(),
        provider: AgentCardProvider { organization: "KiaAgentX".into(), url: base },
        skills,
    }))
}

/// اطمینان از حضور ناوگان — بوت اصلی می‌کارد؛ برای دیتابیس‌های تازه هم idempotent.
async fn ensure_fleet(state: &AppState) -> Result<(), ApiFailure> {
    seed_agents(&state.store).await.map_err(ApiFailure::from)
}

/// `GET /agents/fleet` — عمومی؛ ۷ ایجنت + ابزارهای MCP، وضعیت صادقانه.
///
/// # Errors
/// ۵۰۰ خطای db — `ensure_fleet` یا خواندن ناوگان.
pub async fn fleet(State(state): State<Arc<AppState>>) -> ApiResult<Json<FleetResponse>> {
    ensure_fleet(&state).await?;
    let rows = state.store.list_agents().await.map_err(ApiFailure::from)?;
    Ok(Json(FleetResponse {
        agents: rows
            .into_iter()
            .map(|a| AgentView {
                id: a.id,
                name: a.name,
                role: a.role,
                status: a.status,
                description: a.description,
            })
            .collect(),
        a2a_enabled: state.config.admin_key.is_some(),
        mcp_tools: TOOLS
            .iter()
            .map(|t| McpToolView {
                name: t.name.to_owned(),
                description: t.description.to_owned(),
                args: t.args.iter().map(|s| (*s).to_owned()).collect(),
                admin_only: t.admin_only,
            })
            .collect(),
    }))
}

/// `POST /a2a/messages` — ادمین؛ فرستنده/گیرنده باید ایجنت ثبت‌شده باشند.
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۴۰۴ ایجنت ناشناخته، ۴۲۲ نوع پیام نامعتبر.
///
/// # Panics
/// هیچ‌گاه روی ورودی کاربر — `expect` فقط روی شناسه‌های wire-safe دیتابیس.
pub async fn a2a_send(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    bytes: axum::body::Bytes,
) -> ApiResult<Json<A2aMessageView>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let req: A2aSendRequest = crate::handlers::parse_body(&bytes)?;
    if role_of(req.from.get()).is_none() {
        return Err(HttpFailure::agent_not_found(req.from.get()).into());
    }
    if role_of(req.to.get()).is_none() {
        return Err(HttpFailure::agent_not_found(req.to.get()).into());
    }
    if !matches!(req.kind.as_str(), "handoff" | "query" | "result") {
        return Err(HttpFailure::invalid_request(1).into());
    }
    if req.body.trim().is_empty() || req.body.len() > 2_000 {
        return Err(HttpFailure::invalid_request(1).into());
    }
    let row = A2aRow {
        id: format!("a2a:{}", uuid::Uuid::now_v7()),
        from_agent: req.from.get().to_owned(),
        to_agent: req.to.get().to_owned(),
        kind: req.kind,
        body: req.body,
        created_at: None,
    };
    state.store.insert_a2a_message(&row).await.map_err(ApiFailure::from)?;
    Ok(Json(A2aMessageView {
        id: Id::new(row.id.clone()).expect("a2a ids are wire-safe"),
        from_agent: row.from_agent,
        to_agent: row.to_agent,
        kind: row.kind,
        body: row.body,
        at: kian_db::now_iso(),
    }))
}

/// `GET /a2a/messages/{agent}` — ادمین؛ صندوق ورودی ایجنت.
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۴۰۴ ایجنت ناشناخته.
///
/// # Panics
/// هیچ‌گاه روی ورودی کاربر — `expect` فقط روی شناسه‌های wire-safe دیتابیس.
pub async fn a2a_inbox(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(agent): Path<String>,
) -> ApiResult<Json<A2aInboxResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let agent_id = format!("agent:{agent}");
    if role_of(&agent_id).is_none() {
        return Err(HttpFailure::agent_not_found(&agent_id).into());
    }
    let rows = state.store.list_a2a_inbox(&agent_id, 50).await.map_err(ApiFailure::from)?;
    Ok(Json(A2aInboxResponse {
        items: rows
            .into_iter()
            .map(|r| A2aMessageView {
                id: Id::new(r.id).expect("a2a ids are wire-safe"),
                from_agent: r.from_agent,
                to_agent: r.to_agent,
                kind: r.kind,
                body: r.body,
                at: r.created_at.unwrap_or_default(),
            })
            .collect(),
    }))
}

/// `GET /mcp/tools` — عمومی؛ allowlist شفاف.
///
/// # Errors
/// هیچ — همیشه ۲۰۰ (allowlist ثابت کد).
pub async fn mcp_tools(State(state): State<Arc<AppState>>) -> ApiResult<Json<Vec<McpToolView>>> {
    let _ = &state;
    Ok(Json(TOOLS
        .iter()
        .map(|t| McpToolView {
            name: t.name.to_owned(),
            description: t.description.to_owned(),
            args: t.args.iter().map(|s| (*s).to_owned()).collect(),
            admin_only: t.admin_only,
        })
        .collect()))
}

/// `POST /mcp/invoke` — ادمین؛ فقط ابزارهای allowlist با آرگومان کامل.
///
/// # Errors
/// ۴۰۱/۵۰۳ ادمین، ۴۰۴ ابزار ناشناخته، ۴۲۲ آرگومان نقص، ۵۰۰ db.
pub async fn mcp_invoke(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    bytes: axum::body::Bytes,
) -> ApiResult<Json<McpInvokeResponse>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let req: McpInvokeRequest = crate::handlers::parse_body(&bytes)?;
    let tool = tool_def(req.tool.get()).ok_or_else(|| HttpFailure::mcp_tool_not_found(req.tool.get()))?;
    if let Some(detail) = validate_args(tool, &req.arguments) {
        return Err(HttpFailure::mcp_invalid_args(tool.name, detail).into());
    }
    let arg = |k: &str| req.arguments.get(k).and_then(|v| v.as_str()).map(str::to_owned);
    let limit_arg = |k: &str| {
        req.arguments
            .get(k)
            .and_then(serde_json::Value::as_i64)
            .or_else(|| req.arguments.get(k).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()))
            .unwrap_or(20)
            .clamp(1, 100)
    };

    let result = match tool.name {
        "kernel.economy" => {
            let s = state.store.economy_summary().await.map_err(ApiFailure::from)?;
            serde_json::json!({
                "circulatingSupply": s.circulating_supply,
                "treasuryBalance": s.treasury_balance,
                "escrowLocked": s.escrow_locked,
                "usersSum": s.users_sum,
            })
        }
        "kernel.health" => {
            // دفتر پاسخ می‌دهد؟ — همان liveness عمیق health
            state.store.economy_summary().await.map_err(ApiFailure::from)?;
            serde_json::json!({ "status": "ok", "service": "kian-server" })
        }
        "market.products" => {
            let rows = state
                .store
                .list_products(&kian_db::ProductFilter {
                    active_only: true,
                    limit: limit_arg("limit"),
                    offset: 0,
                    q: None,
                })
                .await
                .map_err(ApiFailure::from)?;
            serde_json::json!(rows
                .iter()
                .map(|p| serde_json::json!({"id": p.id, "title": p.title, "priceDin": p.price_din}))
                .collect::<Vec<_>>())
        }
        "escrow.get" => {
            let id = arg("id").unwrap_or_default();
            let e = state.store.get_escrow(&id).await.map_err(ApiFailure::from)?;
            match e {
                Some(e) => serde_json::json!({
                    "id": e.id,
                    "status": format!("{:?}", e.status),
                    "amountDin": e.amount.as_i64(),
                    "buyerId": e.buyer_id,
                    "sellerId": e.seller_id,
                }),
                None => serde_json::json!(null),
            }
        }
        "kb.search" => {
            let q = arg("q").unwrap_or_default();
            let rows = state.store.search_kb(&q, 5).await.map_err(ApiFailure::from)?;
            serde_json::json!(rows
                .into_iter()
                .map(|(id, title, body)| serde_json::json!({"id": id, "title": title, "body": body}))
                .collect::<Vec<_>>())
        }
        "audit.tail" => {
            let rows = state.store.list_audit(limit_arg("limit")).await.map_err(ApiFailure::from)?;
            serde_json::json!(rows
                .into_iter()
                .map(|r| serde_json::json!({"id": r.id, "actor": r.actor, "action": r.action, "at": r.at}))
                .collect::<Vec<_>>())
        }
        "wallet.balance_of" => {
            let account = arg("account_id").unwrap_or_default();
            let b = state.store.balance_of(&account).await.map_err(ApiFailure::from)?;
            serde_json::json!({ "account": account, "balanceDin": b.unwrap_or(0) })
        }
        "deposits.list" => {
            let status = arg("status");
            let rows = state
                .store
                .list_ton_deposits_all(status.as_deref(), 50)
                .await
                .map_err(ApiFailure::from)?;
            serde_json::json!(rows
                .iter()
                .map(|d| serde_json::json!({"id": d.id, "status": d.status, "amountTon": d.amount_ton}))
                .collect::<Vec<_>>())
        }
        other => return Err(HttpFailure::mcp_tool_not_found(other).into()),
    };
    Ok(Json(McpInvokeResponse { tool: tool.name.to_owned(), result }))
}

#[cfg(test)]
mod tests {
    use super::seed_agents;
    use std::sync::Arc;

    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_ai::client::UnavailableClient;
    use kian_db::V3Store;
    use tower::ServiceExt;

    const ADMIN: &str = "test-admin-key-0123456789";

    async fn app() -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        seed_agents(&store).await.unwrap();
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some(ADMIN.to_owned());
        crate::routes::build_router(
            crate::state::AppState::with_gateway(store, cfg, Arc::new(UnavailableClient)).shared(),
        )
    }

    async fn body_json(res: axum::response::Response) -> serde_json::Value {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// ناوگان عمومی — ۷ ایجنت با وضعیت صادقانه + ۸ ابزار MCP.
    #[tokio::test]
    async fn fleet_is_public_and_honest() {
        let app = app().await;
        let res = app
            .oneshot(axum::http::Request::builder().uri("/api/v1/agents/fleet").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = body_json(res).await;
        let agents = body["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 7);
        assert_eq!(body["mcpTools"].as_array().unwrap().len(), 8);
        // ناوگان ۶+۱ (پوشهٔ ۰۵ گنجینه): مغز + پنج متخصص فعال، نگهبان صادقانه
        // registered — ترتیب دیتابیس الفبایی است
        let orch = agents.iter().find(|a| a["role"] == "orchestrator").expect("orchestrator");
        assert_eq!(orch["status"], "active");
        let merchant = agents.iter().find(|a| a["role"] == "merchant").expect("merchant");
        assert_eq!(merchant["status"], "active");
        let guardian = agents.iter().find(|a| a["role"] == "guardian").expect("guardian");
        assert_eq!(guardian["status"], "registered");
        let active_count = agents.iter().filter(|a| a["status"] == "active").count();
        assert_eq!(active_count, 6, "ناوگان ۶عاملی فعال");
        let wire = body.to_string().to_lowercase();
        assert!(!wire.contains("test-admin-key"), "fleet must never leak the admin key");
    }

    /// کارت A2A در مسیر استاندارد well-known — عمومی، بدون کلید.
    #[tokio::test]
    async fn agent_card_is_public() {
        let app = app().await;
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/.well-known/agent-card.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = body_json(res).await;
        assert_eq!(body["protocolVersion"], "a2a/1.0-kian");
        assert_eq!(body["provider"]["organization"], "KiaAgentX");
    }

    /// گیت‌وی ایجنت‌ها فقط ادمین — بدون کلید ۴۰۱؛ ایجنت ناشناخته ۴۰۴.
    #[tokio::test]
    async fn a2a_is_admin_only_with_agent_guard() {
        let app = app().await;
        let payload = r#"{"from":"agent:orchestrator","to":"agent:support","kind":"handoff","body":"سلام"}"#;

        // بدون کلید
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/a2a/messages")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);

        // با کلید — ارسال سالم
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/a2a/messages")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = body_json(res).await;
        assert!(body["id"].as_str().unwrap().starts_with("a2a:"));

        // صندوق گیرنده
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/a2a/messages/support")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = body_json(res).await;
        assert_eq!(body["items"].as_array().unwrap().len(), 1);

        // ایجنت ناشناخته → ۴۰۴
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/a2a/messages/nope")
                    .header("x-kian-key", ADMIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 404);
    }

    /// MCP — ابزار ناشناخته ۴۰۴؛ آرگومان نقص ۴۲۲؛ economy با کلید ۲۰۰.
    #[tokio::test]
    async fn mcp_invoke_is_guarded_and_validated() {
        let app = app().await;

        // بدون کلید
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/mcp/invoke")
                    .body(Body::from(r#"{"tool":"kernel.economy","arguments":{}}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);

        // ابزار ناشناخته
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/mcp/invoke")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(r#"{"tool":"drop.tables","arguments":{}}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 404);
        assert_eq!(body_json(res).await["error"]["code"], "MCP_TOOL_NOT_FOUND");

        // آرگومان نقص
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/mcp/invoke")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(r#"{"tool":"escrow.get","arguments":{}}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
        assert_eq!(body_json(res).await["error"]["code"], "MCP_INVALID_ARGUMENTS");

        // خوانش عمومی با کلید
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/mcp/invoke")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(r#"{"tool":"kernel.economy","arguments":{}}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = body_json(res).await;
        assert_eq!(body["tool"], "kernel.economy");
        assert!(body["result"]["circulatingSupply"].is_number());
    }
}
