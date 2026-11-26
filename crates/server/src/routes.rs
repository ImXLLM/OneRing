//! سیم‌کشی مسیرها — زیرمجموعهٔ کرنل پول M2 («یک تعریف به ازای هر endpoint» — ADR-005).
//!
//! مسیرهای M2: health عمیق، شفافیت اقتصاد (مادهٔ ۶)، حساب‌ها/تراکنش دفتر،
//! و چرخهٔ کامل امانت ۸ حالته (مادهٔ ۸). M3: هویت تلگرام + نشست bearer، ولت
//! کاربر (`/me`)، بازار (خرید اتمی + کنش‌های طرفین) و واریز TON.
//! M4 — مغز و پنل عملیات (additive): گیت‌وی AI (`/ai/*`)، موتور هرمس
//! (`/hermes/*`)، ناوگان A2A/MCP (`/agents`، `/a2a/*`، `/mcp/*`) و ادمین
//! (`/admin/*`) — گارد ادمین/نشست درون هر هندلر.
//! M5 — حاکمیت و کیف پول چهارزنجیره (additive): پارلمان (`/nation/*` — خواندن
//! عمومی، رأی با نشست، بستن/اجرا با ادمین) و ولت USDT چهارزنجیره
//! (`/wallet/chains`، `/wallet/addresses`، `/wallet/usdt-deposits`،
//! `/wallet/treasury`) — آدرس‌ها با چک‌سام رسمی زنجیره اعتبارسنجی می‌شوند.
//! M6 — بات تلگرام (additive): وبهوک (`/bot/webhook` با راز رسمی تلگرام،
//! fail-closed) — پاسخ‌ها از orchestrator خالص kian-bot؛ هویت فقط از مسیر
//! رسمی initData (ضرب با `KIAN_BOT_TOKEN`، تأیید HMAC).
//!
//! نکتهٔ لایه‌بندی: نرخ‌سنجی قبل از همهٔ مسیرهای API اعمال می‌شود (ادمین
//! معتبر باکت مشترک دارد)؛ گارد ادمین/نشست درون هر هندلر است.

use axum::Router;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers::{
    accounts, admin, agents, ai, arda, auth, bot, builder, chains, deposits, escrows, hermes,
    market, nation, monopoly, quant, studio, system, wallet, war_room,
};
use crate::middleware::rate_limit_middleware;
use crate::state::AppState;

/// پاکت 404 مسیرهای ناشناخته — additive-only؛ کلاینت‌ها هرگز بدنهٔ بی‌شکل نمی‌گیرند.
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

/// سروِ داشبوردِ مستقلِ HTML از درونِ خودِ کرنل (Wave-10 Railway alignment):
/// مسیرِ نسبی اول نسبت به CWD (محصول: `/app`) و بعد نسبت به ریشهٔ مخزن
/// (تست‌ها: `crates/server`) امتحان می‌شود.
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

/// `GET /war-room` — اتاقِ جنگِ بنیان‌گذار، مستقیماً از کرنل.
async fn serve_war_room_html() -> Response {
    serve_tool_html("tools/war-room/war_room.html").await
}

/// `GET /neural-hud` — استودیوی HUD عصبی، مستقیماً از کرنل.
async fn serve_neural_hud_html() -> Response {
    serve_tool_html("tools/neural-hud/neural_hud.html").await
}

/// لایهٔ CORS مینی‌اپ — allowlist دقیق از پیکربندی؛ اوریجین ناشناخته هرگز
/// پاسخ preflight نمی‌گیرد (fail-closed). بدون کوکی — احراز با هدر Bearer است.
///
/// Wave-10 wildcard guard — `tower-http`'s `AllowOrigin::list` روی مقدار `"*"`
/// رفتارِ نامعجمت دارد؛ هر اوریجینِ `"*"` (یا allowlistِ خالی) به
/// [`AllowOrigin::any`] می‌افتد تا `cors_for` هرگز panic نکند.
fn cors_for(origins: &[String]) -> CorsLayer {
    let wildcard = origins.iter().any(|o| o.trim() == "*");
    let base = CorsLayer::new()
        // D3 — DELETE/PUT/PATCH هم مجازند: «حذف محصول ادمین» بدون preflight
        // مرده ماندن بود؛ مینی‌اپِ CRUD کامل اکنون از یک Allowlist واحد عبور می‌کند.
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

/// ساخت روتر کامل.
#[allow(clippy::too_many_lines)]
pub fn build_router(state: Arc<AppState>) -> Router {
    let api: Router<Arc<AppState>> = Router::new()
        // system / meta
        .route("/health", get(system::health))
        .route("/economy", get(system::get_economy))
        .route("/system/backup", post(system::create_backup))
        // accounts — دفتر دوطرفه (ادمین یا خودِ صاحب حساب برای جزئیات/موجودی)
        .route(
            "/accounts",
            post(accounts::create_account).get(accounts::list_accounts),
        )
        .route("/accounts/{id}", get(accounts::get_account))
        .route("/accounts/{id}/balance", get(accounts::get_balance))
        // escrows — ماشین وضعیت ۸ حالته (مسیرها ۱:۱ با قرارداد legacy)
        .route("/escrows", post(escrows::create).get(escrows::list))
        .route("/escrows/{id}", get(escrows::get_one))
        .route("/escrows/{id}/fund", post(escrows::fund))
        .route("/escrows/{id}/deliver", post(escrows::deliver))
        .route("/escrows/{id}/release", post(escrows::release))
        .route("/escrows/{id}/dispute", post(escrows::dispute))
        .route("/escrows/{id}/refund", post(escrows::refund))
        .route("/escrows/{id}/cancel", post(escrows::cancel))
        .route("/escrows/{id}/expire", post(escrows::expire))
        // wallet — اولیه‌های ضرب/سوزاندن Din + درخواست‌های برداشت کریپتو (فاز ۶)
        // faucet — شیر اعتبار آزمایشی (فقط حالت dev؛ KIAN_DEV_MODE=1)
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
        // auth — M3: هویت تلگرام (HMAC رسمی) + نشست bearer apax؛ dev-login فقط حالت dev
        .route("/auth/telegram", post(auth::auth_telegram))
        .route("/auth/dev-login", post(auth::auth_dev_login))
        .route("/auth/logout", post(auth::logout))
        .route("/me", get(auth::me))
        .route("/me/ton-address", post(auth::set_ton_address))
        // bot — M6: وبهوک تلگرام (راز وبهوک fail-closed؛ پاسخ از orchestrator خالص)
        .route("/bot/webhook", post(bot::webhook))
        // market — M3+: محصول/لینک مستقیم/خرید اتمی با رفرال + تحویل HTML و اختلاف خریدار
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
        // market — Wave 3: مُهر دو-کلیدی مایرون + رأی ۵-ستارهٔ بیزی شهروندان
        .route("/market/products/{id}/mairon-confirm", post(market::mairon_confirm))
        .route(
            "/market/products/{id}/reviews",
            post(market::submit_review).get(market::list_reviews),
        )
        // market — Wave 4: بستهٔ مهارتِ مایرون + نوارِ پرچمِ دایره‌ایِ ARDA
        .route("/market/products/{id}/skill", get(market::product_skill))
        // builder — فاز ۴: استودیوی ساخت محصول HTML چندمرحله‌ای + انتشار یک‌کلیکه (نشست شهروند)
        .route(
            "/builder/sessions",
            post(builder::create_session).get(builder::list_sessions),
        )
        .route("/builder/sessions/{id}", get(builder::get_session))
        .route("/builder/sessions/{id}/chat", post(builder::chat))
        .route("/builder/sessions/{id}/publish", post(builder::publish))
        // trinity — Wave 3: استودیوی حاکم سه‌کالبدی
        // (کالبد ۱ آلفا → کالبد ۲ اتاق فرمان با ساب‌ایجنت‌های OpenBot الگوی ۳×۹
        //  → کالبد ۳ مایرون IQ=150/EQ=145 با حلقهٔ خودترمیمی تا گیت REA >= 85)
        .route("/builder/trinity/alpha", post(builder::trinity_alpha))
        .route("/builder/trinity/orchestrate", post(builder::trinity_orchestrate))
        // Gate 3 (Commit #28) — aliasهایِ مستقیمِ ترامینیتی: هر دو مسیرِ
        // `/builder/trinity/*` و `/trinity/*` روی همان هندلرهایِ قطعی می‌نشینند.
        .route("/trinity/alpha", post(builder::trinity_alpha))
        .route("/trinity/orchestrate", post(builder::trinity_orchestrate))
        // arda — Wave 4: بومی‌سازیِ سیاره‌ایِ ۲۴ساعته روی معماری اصیل Fishkal
        // (کاتالوگِ عمومیِ ۲۰ زبان + اجرای زندهٔ ARDA با تزریقِ ایدمپوتنت)
        .route("/arda/flags", get(arda::flags_catalog))
        .route("/arda/localize-now", post(arda::localize_now))
        // quant — Wave 6: هستهٔ ccxt + موتور کوانتِ علّیِ Vibe-Trading
        .route("/quant/venues", get(quant::venues))
        .route("/quant/vibe-backtest", post(quant::vibe_backtest))
        .route("/quant/strategies", get(quant::strategies))
        // monopoly — Wave 7: ژنومِ پنتامدلی، منشورِ روزِ اول، خزانهٔ Base 8453
        .route("/monopoly/genome-matrix", get(monopoly::genome_matrix))
        .route("/monopoly/route-model", post(monopoly::route_model))
        .route("/monopoly/charter-and-base", get(monopoly::charter_and_base))
        .route("/monopoly/simulate-split", post(monopoly::simulate_split))
        // supremacy — Wave 8 (Commit #28): چهار ستونِ برتریِ رقابتی
        // (Bolt/Lovable live-diff · LangGraph time-travel · Dify/n8n DAG ·
        //  OpenHands SWE self-heal) + پلِ MCP JSON-RPC 2.0
        .route("/supremacy/surgical-patch", post(hermes::supremacy_surgical_patch))
        .route("/supremacy/checkpoint/fork", post(hermes::supremacy_checkpoint_fork))
        .route("/supremacy/workflow/execute", post(hermes::supremacy_workflow_execute))
        .route("/supremacy/mcp", post(hermes::supremacy_mcp))
        .route("/supremacy/swe-verify", post(hermes::supremacy_swe_verify))
        // hud — Wave 9 (Commit #29): استودیوی HUD عصبی (کاتالوگ عمومی؛
        //  سنتزِ پیش‌تن و رانِ رندر با نشستِ شهروند)
        .route("/hud/presets", get(hermes::hud_presets))
        .route("/hud/synthesize", post(hermes::hud_synthesize))
        // grok — Wave 9: امضایِ UXِ Grok (بستهٔ UI عمومی؛ چتِ نشست‌دار)
        .route("/grok/ui-bundle", get(hermes::grok_ui_bundle))
        .route("/grok/chat-turn", post(hermes::grok_chat_turn))
        // i18n — Wave 9: ماتریسِ ۲۰ لوکالِ اکوسیستم (عمومی) + پچِ داغ (نشست)
        .route("/i18n/ecosystem-matrix", get(hermes::i18n_ecosystem_matrix))
        .route("/i18n/apply-patch", post(hermes::i18n_apply_patch))
        // admin — Wave 9: رصدخانهٔ آزمایشگاهِ ۱۰-DNA + صادراتِ RLHF/SFT
        .route("/admin/lab-telemetry", get(admin::lab_telemetry))
        .route(
            "/admin/export-training-dataset",
            post(admin::export_training_dataset),
        )
        // war-room — Wave 10 (Commit #30): اتاقِ جنگِ بنیان‌گذارِ حاکم
        // (وضعیتِ روزِ صفر عمومی؛ گامِ علّی و دستورِ رئیس با نشستِ شهروند؛
        //  بازنشانیِ علایی به روزِ صفر برایِ رانشِ دوبارهٔ step)
        .route("/war-room/state", get(war_room::war_room_state))
        .route("/war-room/step", post(war_room::war_room_step))
        .route("/war-room/directive", post(war_room::war_room_directive))
        .route("/war-room/reset", post(war_room::war_room_reset))
        .route(
            "/war-room/explain-simple",
            get(war_room::war_room_explain_simple),
        )
        // studio — Wave 5: استودیوی ادراک‌سیارِ سه‌لایه + ابراستودیوی
        // ۹-مدالیتیِ صفر-هزینه + ویترین پورتفولیو (۶ درمانِ قضایی)
        .route("/studio/ux-profiles", get(studio::ux_profiles))
        .route("/studio/generate", post(studio::generate))
        .route("/studio/portfolio-catalog", get(studio::portfolio_catalog))
        // deposits — M3: واریز TON (اثبات PENDING → تأیید ادمین → ضرب مادهٔ ۳)
        .route(
            "/wallet/ton-deposits",
            post(deposits::create_deposit).get(deposits::list_mine),
        )
        .route("/wallet/ton-deposits/{id}/confirm", post(deposits::confirm))
        .route("/wallet/ton-deposits/{id}/reject", post(deposits::reject))
        // nation — M5: پارلمان HITL (خواندن عمومی؛ رأی نشست؛ بستن/اجرا ادمین)
        .route(
            "/nation/bills",
            post(nation::create_bill).get(nation::list_bills),
        )
        .route("/nation/bills/{id}", get(nation::get_bill))
        .route("/nation/bills/{id}/votes", post(nation::cast_vote))
        .route("/nation/bills/{id}/close", post(nation::close_bill))
        .route("/nation/bills/{id}/execute", post(nation::execute_bill))
        // chains — M5: کیف پول USDT چهارزنجیره (متادیتا/خزانه عمومی؛ بقیه نشست)
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
        // ai — M4: گیت‌وی AI (وضعیت عمومی؛ چت/مصرف نشست‌دار با پاکت بودجه)
        .route("/ai/status", get(ai::status))
        .route("/ai/chat", post(ai::chat))
        .route("/ai/usage", get(ai::usage))
        // hermes — M4: موتور هرمس (مهارت‌ها عمومی؛ ران نشست‌دار)
        .route("/hermes/skills", get(hermes::skills))
        .route("/hermes/run", post(hermes::run))
        .route("/hermes/runs", get(hermes::my_runs))
        // agents — M4: ناوگان + A2A + MCP (گیت‌وی ایجنت‌ها فقط ادمین)
        .route("/agents/fleet", get(agents::fleet))
        .route("/a2a/messages", post(agents::a2a_send))
        .route("/a2a/messages/{agent}", get(agents::a2a_inbox))
        .route("/mcp/tools", get(agents::mcp_tools))
        .route("/mcp/invoke", post(agents::mcp_invoke))
        // admin — M4: پنل عملیات (همهٔ مسیرها X-KIAN-Key)
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

    // کشف A2A — مسیر استاندارد well-known خارج از nest؛ بدون هیچ کلیدی.
    // D4 — کارت ایجنت هم زیر نرخ‌سنج می‌رود: این مسیر روی روتر بیرونی است،
    // پس لایهٔ نرخ‌سنج مستقیم روی زیرروترِ well-known اعمال می‌شود تا
    // `/api/v1` دوباره‌نرخ‌سنجی نشود و blind-spot بسته بماند.
    let well_known = Router::new()
        .route("/.well-known/agent-card.json", get(agents::agent_card))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ));
    // Wave-10 — داشبوردهایِ مستقلِ HTML از درونِ خودِ کرنل سرو می‌شوند:
    // بازدیدِ زندهٔ Railway روی `/war-room` و `/neural-hud` بدونِ سرورِ خارجی.
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

    /// D3 — preflight حذفِ محصول ادمین باید DELETE را تبلیغ کند.
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

    /// D3 — اوریجین ناشناخته همچنان fail-closed: بدون هدرهای CORS.
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

    /// D4 — کارت ایجنت دیگر blind-spot نیست: عبور از سقفِ نرخ روی این مسیر
    /// باید 429 بدهد (قبل از وصله، این مسیر بدون نرخ‌سنج بود).
    #[tokio::test]
    async fn agent_card_is_covered_by_rate_limit_middleware() {
        let app = hardened_app(2).await; // سقفِ ۲ درخواست در پنجره
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

    /// Gate 3 (Commit #28) — alias مستقیمِ `/api/v1/trinity/alpha` باید روی
    /// همان هندلرِ `/builder/trinity/alpha` برسد: ۴۰۱ (گوارِ نشست)، نه ۴۰۴.
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
        // orchestrate هم همان قرارداد را دارد
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/trinity/orchestrate")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_ne!(res.status(), 404, "orchestrate alias must be routed");
    }

    /// Wave 10 — wildcard CORS هرگز panic نمی‌کند: `"*"` به `AllowOrigin::any`
    /// می‌افتد و preflight با `access-control-allow-origin: *` پاس می‌گیرد.
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

    /// Wave 10 — داشبوردهایِ مستقلِ `/war-room` و `/neural-hud` از خودِ کرنل
    /// سرو می‌شوند (HTML با charset=utf-8؛ نه ۴۰۴، نه JSON).
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

    /// Wave 10 — پنج مسیرِ اتاقِ جنگ ثبت شده‌اند:
    /// state/explain-simple عمومی (200)، step/directive/reset با گوارِ نشست (401).
    #[tokio::test]
    async fn war_room_routes_are_registered() {
        let app = hardened_app(10_000).await;
        // عمومی — باید ۲۰۰ بدهد، نه ۴۰۴.
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
        // نوشتن — گوارِ نشست: ۴۰۱، نه ۴۰۴.
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
