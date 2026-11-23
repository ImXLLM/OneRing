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
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::{
    accounts, admin, agents, ai, auth, bot, builder, chains, deposits, escrows, hermes, market,
    nation, system, wallet,
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

/// لایهٔ CORS مینی‌اپ — allowlist دقیق از پیکربندی؛ اوریجین ناشناخته هرگز
/// پاسخ preflight نمی‌گیرد (fail-closed). بدون کوکی — احراز با هدر Bearer است.
fn cors_for(origins: &[String]) -> CorsLayer {
    let allowed: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
    CorsLayer::new()
        .allow_origin(allowed)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-kian-key"),
        ])
        .allow_credentials(false)
}

/// ساخت روتر کامل.
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
        .route("/wallet/mint", post(wallet::mint))
        .route("/wallet/burn", post(wallet::burn))
        .route(
            "/wallet/withdrawals",
            post(wallet::create_withdrawal).get(wallet::list_my_withdrawals),
        )
        .route("/wallet/withdrawals/{id}/confirm", post(wallet::confirm_withdrawal))
        .route("/wallet/withdrawals/{id}/reject", post(wallet::reject_withdrawal))
        .route("/admin/withdrawals", get(wallet::admin_list_withdrawals))
        // auth — M3: هویت تلگرام (HMAC رسمی) + نشست bearer apax
        .route("/auth/telegram", post(auth::auth_telegram))
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
        // builder — فاز ۴: استودیوی ساخت محصول HTML چندمرحله‌ای + انتشار یک‌کلیکه (نشست شهروند)
        .route(
            "/builder/sessions",
            post(builder::create_session).get(builder::list_sessions),
        )
        .route("/builder/sessions/{id}", get(builder::get_session))
        .route("/builder/sessions/{id}/chat", post(builder::chat))
        .route("/builder/sessions/{id}/publish", post(builder::publish))
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

    // کشف A2A — مسیر استاندارد well-known خارج از nest؛ بدون هیچ کلیدی
    Router::new()
        .nest("/api/v1", api)
        .route("/.well-known/agent-card.json", get(agents::agent_card))
        .with_state(state)
}
