//! Route wiring — a subset of the M2 money kernel («one definition per endpoint» — ADR-005).
//!
//! M2 routes: deep health, economy transparency (Article 6), accounts/ledger transactions,
//! and the full 8-state escrow cycle (Article 8). M3: Telegram identity + bearer sessions, the user
//! wallet (`/me`), the market (atomic purchase + counterparty actions) and TON deposits.
//! M4 — brain and operations panel (additive): the AI gateway (`/ai/*`), the Hermes engine
//! (`/hermes/*`), the A2A/MCP fleet (`/agents`, `/a2a/*`, `/mcp/*`) and admin
//! (`/admin/*`) — admin/session guards inside each handler.
//! M5 — governance and the four-chain wallet (additive): parliament (`/nation/*` — public
//! reads, session-gated voting, admin close/execute) and the four-chain USDT wallet
//! (`/wallet/chains`, `/wallet/addresses`, `/wallet/usdt-deposits`,
//! `/wallet/treasury`) — addresses are validated with the chain's official checksum.
//! M6 — Telegram bot (additive): webhook (`/bot/webhook` with the official Telegram secret,
//! fail-closed) — replies come from the pure kian-bot orchestrator; identity only via the
//! official initData path (multiplied with `KIAN_BOT_TOKEN`, HMAC-verified).
//!
//! Layering note: rate limiting applies before all API routes (valid admin
//! shares one bucket); the admin/session guard lives inside each handler.

use axum::Router;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers::{
    accounts, admin, agents, ai, arda, auth, bot, builder, chains, deposits, escrows, grok_lab,
    hermes, market, nation, monopoly, quant, studio, system, wallet, war_room,
};
use crate::middleware::rate_limit_middleware;
use crate::state::AppState;

/// 404 envelope for unknown routes — additive-only; clients never receive an unshaped body.
async fn api_not_found() -> Response {
    let envelope = serde_json::json!({
        "error": { "code": "NOT_FOUND", "message": "No such route under /api/v1 — M4 kernel surface." }
    });
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "application/json")],
        envelope.to_string(),
    )
        .into_response()
}

/// Serving the standalone HTML dashboard from inside the kernel itself (Wave-10 Railway alignment):
/// relative paths are tried first against CWD (production: `/app`) and then against the repo root
/// (tests: `crates/server`).
async fn serve_tool_html(rel: &str) -> Response {
    for candidate in [rel.to_owned(), format!("../../{rel}")] {
        if let Ok(bytes) = tokio::fs::read(&candidate).await {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(axum::body::Body::from(bytes))
                .unwrap();
        }
    }
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(axum::body::Body::from(
            "<pre>dashboard artifact not found in this deployment</pre>",
        ))
        .unwrap()
}

/// `GET /war-room` — the founder war room, served straight from the kernel.
async fn serve_war_room_html() -> Response {
    serve_tool_html("tools/war-room/war_room.html").await
}

/// `GET /neural-hud` — the neural HUD studio, served straight from the kernel.
async fn serve_neural_hud_html() -> Response {
    serve_tool_html("tools/neural-hud/neural_hud.html").await
}

/// Mini-app CORS layer — exact allowlist from configuration; unknown origins never
/// receive a preflight response (fail-closed). No cookies — auth is via the Bearer header.
///
/// Wave-10 wildcard guard — `tower-http`'s `AllowOrigin::list` on the `"*"` value
/// has ambiguous behavior; any `"*"` origin (or an empty allowlist) falls
/// into [`AllowOrigin::any`] so `cors_for` never panics.
fn cors_for(origins: &[String]) -> CorsLayer {
    let wildcard = origins.iter().any(|o| o.trim() == "*");
    let base = CorsLayer::new()
        // D3 — DELETE/PUT/PATCH are allowed too: without preflight, «admin product deletion»
        // would have stayed dead; the full-CRUD mini app now passes through a single Allowlist.
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::PUT,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-kian-key"),
        ])
        .allow_credentials(false);
    if origins.is_empty() || wildcard {
        base.allow_origin(AllowOrigin::any())
    } else {
        let allowed: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
        base.allow_origin(allowed)
    }
}

/// Build the full router.
#[allow(clippy::too_many_lines)]
pub fn build_router(state: Arc<AppState>) -> Router {
    let api: Router<Arc<AppState>> = Router::new()
        // system / meta
        .route("/health", get(system::health))
        .route("/economy", get(system::get_economy))
        .route("/system/backup", post(system::create_backup))
        // accounts — double-entry ledger (admin, or the account owner for details/balance)
        .route(
            "/accounts",
            post(accounts::create_account).get(accounts::list_accounts),
        )
        .route("/accounts/{id}", get(accounts::get_account))
        .route("/accounts/{id}/balance", get(accounts::get_balance))
        // escrows — 8-state machine (routes 1:1 with the legacy contract)
        .route("/escrows", post(escrows::create).get(escrows::list))
        .route("/escrows/{id}", get(escrows::get_one))
        .route("/escrows/{id}/fund", post(escrows::fund))
        .route("/escrows/{id}/deliver", post(escrows::deliver))
        .route("/escrows/{id}/release", post(escrows::release))
        .route("/escrows/{id}/dispute", post(escrows::dispute))
        .route("/escrows/{id}/refund", post(escrows::refund))
        .route("/escrows/{id}/cancel", post(escrows::cancel))
        .route("/escrows/{id}/expire", post(escrows::expire))
        // wallet — Din mint/burn initiations + crypto withdrawal requests (Phase 6)
        // faucet — trial-credit tap (dev mode only; KIAN_DEV_MODE=1)
        .route("/wallet/mint", post(wallet::mint))
        .route("/wallet/burn", post(wallet::burn))
        .route("/wallet/faucet", post(wallet::faucet))
        .route(
            "/wallet/withdrawals",
            post(wallet::create_withdrawal).get(wallet::list_my_withdrawals),
        )
        .route("/wallet/withdrawals/{id}/confirm", post(wallet::confirm_withdrawal))
        .route("/wallet/withdrawals/{id}/reject", post(wallet::reject_withdrawal))
        .route("/admin/withdrawals", get(wallet::admin_list_withdrawals))
        .route("/admin/wallet/withdrawals", get(wallet::admin_list_withdrawals))
        // auth — M3: Telegram identity (official HMAC) + bearer session issuance; dev-login in dev mode only
        .route("/auth/telegram", post(auth::auth_telegram))
        .route("/auth/dev-login", post(auth::auth_dev_login))
        .route("/auth/logout", post(auth::logout))
        .route("/me", get(auth::me))
        .route("/me/ton-address", post(auth::set_ton_address))
        // bot — M6: Telegram webhook (fail-closed webhook secret; replies from the pure orchestrator)
        .route("/bot/webhook", post(bot::webhook))
        // market — M3+: product/direct link/atomic purchase with referral + HTML delivery and buyer disputes
        .route(
            "/market/products",
            post(market::create_product).get(market::list_products),
        )
        .route("/market/products/{id}", get(market::get_product_one))
        .route("/market/purchase", post(market::purchase))
        .route("/market/my", get(market::my_market))
        .route("/market/escrows/{id}/delivery", get(market::delivery))
        .route("/market/escrows/{id}/deliver", post(market::deliver))
        .route("/market/escrows/{id}/release", post(market::release))
        .route("/market/escrows/{id}/dispute", post(market::dispute))
        // market — Wave 3: the two-key Mairon seal + citizens' Bayesian 5-star votes
        .route("/market/products/{id}/mairon-confirm", post(market::mairon_confirm))
        .route(
            "/market/products/{id}/reviews",
            post(market::submit_review).get(market::list_reviews),
        )
        // market — Wave 4: the Mairon skill pack + ARDA circular flag banner
        .route("/market/products/{id}/skill", get(market::product_skill))
        // builder — Phase 4: multi-stage HTML product-builder studio + one-click publish (citizen session)
        .route(
            "/builder/sessions",
            post(builder::create_session).get(builder::list_sessions),
        )
        .route("/builder/sessions/{id}", get(builder::get_session))
        .route("/builder/sessions/{id}/chat", post(builder::chat))
        .route("/builder/sessions/{id}/publish", post(builder::publish))
        // trinity — Wave 3: the three-body sovereign studio
        // (body 1 alpha → body 2 command room with OpenBot subagents in the 3×9 pattern
        //  → body 3 Mairon IQ=150/EQ=145 with a self-heal loop up to the REA gate >= 85)
        .route("/builder/trinity/alpha", post(builder::trinity_alpha))
        .route("/builder/trinity/orchestrate", post(builder::trinity_orchestrate))
        // Gate 3 (Commit #28) — direct Trinity aliases: both
        // `/builder/trinity/*` and `/trinity/*` sit on the same deterministic handlers.
        .route("/trinity/alpha", post(builder::trinity_alpha))
        .route("/trinity/orchestrate", post(builder::trinity_orchestrate))
        // arda — Wave 4: 24-hour planetary localization on the authentic Fishkal architecture
        // (public 20-language catalog + live ARDA runs with idempotent injection)
        .route("/arda/flags", get(arda::flags_catalog))
        .route("/arda/localize-now", post(arda::localize_now))
        // quant — Wave 6: the ccxt core + the Vibe-Trading causal quant engine
        .route("/quant/venues", get(quant::venues))
        .route("/quant/vibe-backtest", post(quant::vibe_backtest))
        .route("/quant/strategies", get(quant::strategies))
        // monopoly — Wave 7: pentamodel genome, Day One charter, Base 8453 treasury
        .route("/monopoly/genome-matrix", get(monopoly::genome_matrix))
        .route("/monopoly/route-model", post(monopoly::route_model))
        .route("/monopoly/charter-and-base", get(monopoly::charter_and_base))
        .route("/monopoly/simulate-split", post(monopoly::simulate_split))
        // supremacy — Wave 8 (Commit #28): four pillars of competitive supremacy
        // (Bolt/Lovable live-diff · LangGraph time-travel · Dify/n8n DAG ·
        //  OpenHands SWE self-heal) + the MCP JSON-RPC 2.0 bridge
        .route("/supremacy/surgical-patch", post(hermes::supremacy_surgical_patch))
        .route("/supremacy/checkpoint/fork", post(hermes::supremacy_checkpoint_fork))
        .route("/supremacy/workflow/execute", post(hermes::supremacy_workflow_execute))
        .route("/supremacy/mcp", post(hermes::supremacy_mcp))
        .route("/supremacy/swe-verify", post(hermes::supremacy_swe_verify))
        // hud — Wave 9 (Commit #29): neural HUD studio (public catalog;
        //  pre-tuned synthesis and render runs with a citizen session)
        .route("/hud/presets", get(hermes::hud_presets))
        .route("/hud/synthesize", post(hermes::hud_synthesize))
        // grok — Wave 9: the Grok UX signature (public UI pack; session-gated chat)
        .route("/grok/ui-bundle", get(hermes::grok_ui_bundle))
        .route("/grok/chat-turn", post(hermes::grok_chat_turn))
        // grok-lab — the MAIRON 10-DNA telemetry surface (packages/analytics client)
        .route("/grok-lab/telemetry", post(grok_lab::insert_telemetry))
        .route("/grok-lab/export-sft-dpo", get(grok_lab::export_sft_dpo))
        .route("/grok-lab/summary", get(grok_lab::summary))
        // i18n — Wave 9: the ecosystem's 20-locale matrix (public) + hot patches (session)
        .route("/i18n/ecosystem-matrix", get(hermes::i18n_ecosystem_matrix))
        .route("/i18n/apply-patch", post(hermes::i18n_apply_patch))
        // admin — Wave 9: the 10-DNA lab observatory + RLHF/SFT export
        .route("/admin/lab-telemetry", get(admin::lab_telemetry))
        .route(
            "/admin/export-training-dataset",
            post(admin::export_training_dataset),
        )
        // war-room — Wave 10 (Commit #30): the sovereign founder war room
        // (public day-zero state; causal steps and founder directives with a citizen session;
        //  idempotent reset back to day zero to re-drive step)
        .route("/war-room/state", get(war_room::war_room_state))
        .route("/war-room/step", post(war_room::war_room_step))
        .route("/war-room/directive", post(war_room::war_room_directive))
        .route("/war-room/reset", post(war_room::war_room_reset))
        .route(
            "/war-room/explain-simple",
            get(war_room::war_room_explain_simple),
        )
        // studio — Wave 5: the three-layer mobile-perception studio + the
        // zero-cost 9-modality super-studio + portfolio showcase (6 legal remedies)
        .route("/studio/ux-profiles", get(studio::ux_profiles))
        .route("/studio/generate", post(studio::generate))
        .route("/studio/portfolio-catalog", get(studio::portfolio_catalog))
        // deposits — M3: TON deposits (PENDING proof → admin confirmation → Article 3 mint)
        .route(
            "/wallet/ton-deposits",
            post(deposits::create_deposit).get(deposits::list_mine),
        )
        .route("/wallet/ton-deposits/{id}/confirm", post(deposits::confirm))
        .route("/wallet/ton-deposits/{id}/reject", post(deposits::reject))
        // nation — M5: HITL parliament (public reads; session voting; admin close/execute)
        .route(
            "/nation/bills",
            post(nation::create_bill).get(nation::list_bills),
        )
        .route("/nation/bills/{id}", get(nation::get_bill))
        .route("/nation/bills/{id}/votes", post(nation::cast_vote))
        .route("/nation/bills/{id}/close", post(nation::close_bill))
        .route("/nation/bills/{id}/execute", post(nation::execute_bill))
        // chains — M5: four-chain USDT wallet (metadata/treasury public; the rest session-gated)
        .route("/wallet/chains", get(chains::chains_list))
        .route("/wallet/treasury", get(chains::treasury))
        .route(
            "/wallet/addresses",
            post(chains::set_address).get(chains::my_addresses),
        )
        .route(
            "/wallet/usdt-deposits",
            post(chains::create_usdt_deposit).get(chains::list_my_usdt_deposits),
        )
        .route(
            "/wallet/usdt-deposits/{id}/confirm",
            post(chains::confirm_usdt_deposit),
        )
        .route(
            "/wallet/usdt-deposits/{id}/reject",
            post(chains::reject_usdt_deposit),
        )
        // ai — M4: AI gateway (public status; session-gated chat/consumption with a budget envelope)
        .route("/ai/status", get(ai::status))
        .route("/ai/chat", post(ai::chat))
        .route("/ai/usage", get(ai::usage))
        // hermes — M4: Hermes engine (skills public; session-gated runs)
        .route("/hermes/skills", get(hermes::skills))
        .route("/hermes/run", post(hermes::run))
        .route("/hermes/runs", get(hermes::my_runs))
        // agents — M4: fleet + A2A + MCP (agent gateway admin-only)
        .route("/agents/fleet", get(agents::fleet))
        .route("/a2a/messages", post(agents::a2a_send))
        .route("/a2a/messages/{agent}", get(agents::a2a_inbox))
        .route("/mcp/tools", get(agents::mcp_tools))
        .route("/mcp/invoke", post(agents::mcp_invoke))
        // admin — M4: operations panel (all routes X-KIAN-Key)
        .route("/admin/overview", get(admin::overview))
        .route("/admin/hitl", get(admin::hitl_list))
        .route("/admin/hitl/{id}/approve", post(admin::hitl_approve))
        .route("/admin/hitl/{id}/reject", post(admin::hitl_reject))
        .route("/admin/audit", get(admin::audit_tail))
        .route("/admin/deposits", get(admin::deposits_list))
        .route("/admin/products", get(admin::products_list))
        .route("/admin/products/{id}", delete(admin::product_delete))
        .fallback(api_not_found)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(cors_for(&state.config.cors_origins))
        .layer(TraceLayer::new_for_http());

    // A2A discovery — the standard well-known path outside the nest; no keys at all.
    // D4 — the agent card also goes under the rate limiter: this route sits on the outer router,
    // so the rate-limit layer is applied directly on the well-known subrouter to keep
    // `/api/v1` from being rate-limited twice and the blind spot closed.
    let well_known = Router::new()
        .route("/.well-known/agent-card.json", get(agents::agent_card))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ));
    // Wave-10 — standalone HTML dashboards are served from inside the kernel itself:
    // live Railway viewing of `/war-room` and `/neural-hud` with no external server.
    let dashboards = Router::new()
        .route("/war-room", get(serve_war_room_html))
        .route("/neural-hud", get(serve_neural_hud_html))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ));
    Router::new()
        .nest("/api/v1", api)
        .merge(well_known)
        .merge(dashboards)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use kian_db::V3Store;
    use tower::ServiceExt;

    use super::*;

    async fn hardened_app(rate_limit: usize) -> Router {
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some("test-admin-key-0123456789".to_owned());
        cfg.rate_limit = rate_limit;
        cfg.cors_origins = vec!["https://mini.0xkia.app".to_owned()];
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        build_router(crate::state::AppState::new(store, cfg).shared())
    }

    /// D3 — the admin product-deletion preflight must advertise DELETE.
    #[tokio::test]
    async fn preflight_for_admin_product_delete_advertises_delete() {
        let app = hardened_app(10_000).await;
        let req = axum::http::Request::builder()
            .method("OPTIONS")
            .uri("/api/v1/admin/products/p1")
            .header("origin", "https://mini.0xkia.app")
            .header("access-control-request-method", "DELETE")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), 200, "preflight must succeed for allowlisted origin");
        let allow = res
            .headers()
            .get("access-control-allow-methods")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        for method in ["DELETE", "PUT", "PATCH", "POST", "GET"] {
            assert!(allow.contains(method), "preflight must advertise {method}, got: {allow}");
        }
    }

    /// D3 — unknown origins remain fail-closed: no CORS headers.
    #[tokio::test]
    async fn preflight_from_unknown_origin_stays_fail_closed() {
        let app = hardened_app(10_000).await;
        let req = axum::http::Request::builder()
            .method("OPTIONS")
            .uri("/api/v1/admin/products/p1")
            .header("origin", "https://evil.example")
            .header("access-control-request-method", "DELETE")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert!(res.headers().get("access-control-allow-origin").is_none());
    }

    /// D4 — the agent card is no longer a blind spot: exceeding the rate cap on this route
    /// must return 429 (before the patch, this route had no rate limiting).
    #[tokio::test]
    async fn agent_card_is_covered_by_rate_limit_middleware() {
        let app = hardened_app(2).await; // cap of 2 requests per window
        let mut last = None;
        for i in 0..4 {
            let req = axum::http::Request::builder()
                .method("GET")
                .uri("/.well-known/agent-card.json")
                .body(Body::empty())
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            last = Some((i, res.status().as_u16()));
        }
        let (idx, status) = last.expect("at least one probe");
        assert_eq!(status, 429, "request #{idx} over the limit must be 429");
    }

    /// Gate 3 (Commit #28) — the direct `/api/v1/trinity/alpha` alias must land on
    /// the same `/builder/trinity/alpha` handler: 401 (session guard), not 404.
    #[tokio::test]
    async fn trinity_alpha_direct_alias_is_registered() {
        let app = hardened_app(10_000).await;
        for path in ["/api/v1/trinity/alpha", "/api/v1/builder/trinity/alpha"] {
            let req = axum::http::Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"vision":"probe"}"#))
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            let status = res.status().as_u16();
            assert_ne!(status, 404, "{path} must be routed");
            assert_eq!(status, 401, "{path} must hit the same session guard");
        }
        // orchestrate carries the same contract
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/trinity/orchestrate")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_ne!(res.status(), 404, "orchestrate alias must be routed");
    }

    /// Wave 10 — wildcard CORS never panics: `"*"` falls through to `AllowOrigin::any`
    /// and the preflight passes with `access-control-allow-origin: *`.
    #[tokio::test]
    async fn cors_for_with_wildcard_origin_does_not_panic_and_allows_any() {
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some("test-admin-key-0123456789".to_owned());
        cfg.rate_limit = 10_000;
        cfg.cors_origins = vec!["*".to_owned()];
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        let app = build_router(crate::state::AppState::new(store, cfg).shared());
        let req = axum::http::Request::builder()
            .method("OPTIONS")
            .uri("/api/v1/war-room/state")
            .header("origin", "https://any.example")
            .header("access-control-request-method", "GET")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), 200, "wildcard preflight must succeed without panicking");
        let allow = res
            .headers()
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        assert_eq!(allow, "*", "wildcard allowlist must answer access-control-allow-origin: *");
    }

    /// Wave 10 — the standalone `/war-room` and `/neural-hud` dashboards are served
    /// by the kernel itself (HTML with charset=utf-8; not 404, not JSON).
    #[tokio::test]
    async fn standalone_dashboards_are_served_by_the_kernel() {
        let app = hardened_app(10_000).await;
        for path in ["/war-room", "/neural-hud"] {
            let req = axum::http::Request::builder()
                .method("GET")
                .uri(path)
                .body(Body::empty())
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            let status = res.status().as_u16();
            assert_ne!(status, 404, "{path} must be routed by the kernel");
            assert_eq!(status, 200, "{path} must serve the committed dashboard");
            let ctype = res
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_owned();
            assert!(
                ctype.starts_with("text/html") && ctype.contains("charset=utf-8"),
                "{path} must be text/html; charset=utf-8, got {ctype}"
            );
        }
    }

    /// Wave 10 — five war-room routes are registered:
    /// state/explain-simple public (200); step/directive/reset with a session guard (401).
    #[tokio::test]
    async fn war_room_routes_are_registered() {
        let app = hardened_app(10_000).await;
        // public — must return 200, not 404.
        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/v1/war-room/state")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status().as_u16(), 200, "war-room/state is public");
        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/v1/war-room/explain-simple?locale=fa")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status().as_u16(), 200, "explain-simple is public");
        // write — session guard: 401, not 404.
        for (method, path, body) in [
            ("POST", "/api/v1/war-room/step", r#"{"tickCursor":0,"speedMultiplier":1}"#),
            (
                "POST",
                "/api/v1/war-room/directive",
                r#"{"directiveText":"probe","locale":"fa","triggeredTick":0}"#,
            ),
            ("POST", "/api/v1/war-room/reset", ""),
        ] {
            let req = axum::http::Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            let status = res.status().as_u16();
            assert_ne!(status, 404, "{path} must be routed");
            assert_eq!(status, 401, "{path} must be session-guarded");
        }
    }
}
