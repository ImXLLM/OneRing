//! M2 wire integration tests — the full money cycle over real HTTP (no network).
//!
//! Each scenario exercises three layers at once: the fail-closed `X-KIAN-Key` guard,
//! wire validation, and the escrow state machine over the double-entry ledger with the Charter 10% tax.
//! The equivalent of Fastify’s `app.inject` in legacy — `tower::ServiceExt::oneshot`.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use kian_db::V3Store;
use kian_server::{build_router, AppState, ServerConfig};
use serde_json::{json, Value};
use tower::ServiceExt;

const KEY: &str = "kian-admin-key-0123456789";

/// Build the router with the migrated in-memory store + system accounts.
async fn router_with_key(admin_key: Option<&str>) -> axum::Router {
    let store = V3Store::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store.ensure_system_accounts().await.unwrap();
    let mut config = ServerConfig::base();
    config.admin_key = admin_key.map(str::to_owned);
    // M3 Telegram identity — disabled in the M2 tests; M4 AI slot off
    config.bot_token = None;
    config.cors_origins = vec!["http://localhost:3000".into(), "http://127.0.0.1:3000".into()];
    config.rate_limit = 120;
    config.backup_dir = "data/backups".into();
    config.keep_backups = 3;
    build_router(AppState::new(store, config).shared())
}

async fn call(
    router: axum::Router,
    method: &str,
    path: &str,
    key: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    let req = match key {
        Some(k) => req.header("x-kian-key", k),
        None => req,
    };
    let response = router
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1_048_576).await.unwrap();
    let json: Value =
        if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json)
}

async fn call_empty(
    router: axum::Router,
    method: &str,
    path: &str,
    key: Option<&str>,
) -> (StatusCode, Value) {
    let req = Request::builder().method(method).uri(path);
    let req = match key {
        Some(k) => req.header("x-kian-key", k),
        None => req,
    };
    let response = router.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1_048_576).await.unwrap();
    let json: Value =
        if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json)
}

// ── Admin guard — fail-closed matrix over the real wire ────────────────────────────

#[tokio::test]
async fn admin_guard_fail_closed_over_the_wire() {
    // server without a key → 503 always (never open)
    let (status, body) =
        call_empty(router_with_key(None).await, "POST", "/api/v1/wallet/mint", None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "ADMIN_DISABLED");

    let router = router_with_key(Some(KEY)).await;
    // missing header → 401
    let (status, body) = call_empty(router.clone(), "POST", "/api/v1/wallet/mint", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "ADMIN_KEY_REQUIRED");
    // wrong header → 401
    let (status, body) =
        call_empty(router.clone(), "POST", "/api/v1/wallet/mint", Some("wrong-key-0123456")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "INVALID_ADMIN_KEY");
    // unknown path → 404 additive-only envelope
    let (status, body) = call_empty(router.clone(), "GET", "/api/v1/nope", Some(KEY)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

// ── Full escrow cycle with the Charter 10% tax ─────────────────────────────────────

#[tokio::test]
async fn full_escrow_lifecycle_with_charter_tax_split() {
    let router = router_with_key(Some(KEY)).await;

    // buyer and seller accounts
    let (status, _) = call(
        router.clone(),
        "POST",
        "/api/v1/accounts",
        Some(KEY),
        json!({"id": "user:kia", "label": "Kia"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = call(
        router.clone(),
        "POST",
        "/api/v1/accounts",
        Some(KEY),
        json!({"id": "user:ada", "label": "Ada"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // mint 1000 Đ for the buyer (within the Charter annual cap)
    let (status, minted) = call(
        router.clone(),
        "POST",
        "/api/v1/wallet/mint",
        Some(KEY),
        json!({"accountId": "user:kia", "amount": 1000, "idempotencyKey": "mint:kia:1"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "mint body: {minted}");

    // create a 400 Đ escrow — CREATED; no money moved yet
    let (status, escrow) = call(
        router.clone(),
        "POST",
        "/api/v1/escrows",
        Some(KEY),
        json!({"id": "e1", "buyerId": "user:kia", "sellerId": "user:ada", "amount": 400, "productRef": "p-9"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create body: {escrow}");
    assert_eq!(escrow["escrow"]["status"], "CREATED");

    // fund → deliver → release
    let (status, funded) = call_empty(router.clone(), "POST", "/api/v1/escrows/e1/fund", Some(KEY)).await;
    assert_eq!(status, StatusCode::OK, "fund body: {funded}");
    assert_eq!(funded["escrow"]["status"], "FUNDED");
    let buyer_locked = call_empty(router.clone(), "GET", "/api/v1/accounts/user:kia/balance", Some(KEY)).await;
    assert_eq!(buyer_locked.1["balance"], 600, "funding: 1000 − 400 locked in escrow");

    let (status, delivered) = call_empty(router.clone(), "POST", "/api/v1/escrows/e1/deliver", Some(KEY)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(delivered["escrow"]["status"], "DELIVERED");

    let (status, released) = call_empty(router.clone(), "POST", "/api/v1/escrows/e1/release", Some(KEY)).await;
    assert_eq!(status, StatusCode::OK, "release body: {released}");
    assert_eq!(released["escrow"]["status"], "RELEASED");

    // Article 8 — settlement: seller net (90%) + treasury tax 10%
    let (_, seller) = call_empty(router.clone(), "GET", "/api/v1/accounts/user:ada/balance", Some(KEY)).await;
    assert_eq!(seller["balance"], 360);
    let (_, treasury) =
        call_empty(router.clone(), "GET", "/api/v1/accounts/treasury:nation/balance", Some(KEY)).await;
    assert_eq!(treasury["balance"], 40);

    // economy transparency — the trial balance must remain inviolate
    let (status, economy) = call_empty(router.clone(), "GET", "/api/v1/economy", None).await;
    assert_eq!(status, StatusCode::OK, "economy is public and keyless (article 6)");
    assert_eq!(economy["balanced"], true);
    assert_eq!(economy["circulatingSupply"], 1000);
}

// ── Invalid transition → honest 422 ──────────────────────────────────────────────

#[tokio::test]
async fn invalid_transition_is_422() {
    let router = router_with_key(Some(KEY)).await;
    call(router.clone(), "POST", "/api/v1/accounts", Some(KEY), json!({"id": "user:kia"})).await;
    call(router.clone(), "POST", "/api/v1/accounts", Some(KEY), json!({"id": "user:ada"})).await;
    call(
        router.clone(),
        "POST",
        "/api/v1/wallet/mint",
        Some(KEY),
        json!({"accountId": "user:kia", "amount": 100}),
    )
    .await;
    call(
        router.clone(),
        "POST",
        "/api/v1/escrows",
        Some(KEY),
        json!({"id": "e2", "buyerId": "user:kia", "sellerId": "user:ada", "amount": 50}),
    )
    .await;

    // release before funding — a state machine violation
    let (status, body) = call_empty(router.clone(), "POST", "/api/v1/escrows/e2/release", Some(KEY)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "INVALID_TRANSITION");
}

// ── Idempotency key — a replay never moves money twice ──────────────────

#[tokio::test]
async fn idempotent_fund_replays_without_double_money() {
    let router = router_with_key(Some(KEY)).await;
    call(router.clone(), "POST", "/api/v1/accounts", Some(KEY), json!({"id": "user:kia"})).await;
    call(router.clone(), "POST", "/api/v1/accounts", Some(KEY), json!({"id": "user:ada"})).await;
    call(
        router.clone(),
        "POST",
        "/api/v1/wallet/mint",
        Some(KEY),
        json!({"accountId": "user:kia", "amount": 100}),
    )
    .await;
    call(
        router.clone(),
        "POST",
        "/api/v1/escrows",
        Some(KEY),
        json!({"id": "e3", "buyerId": "user:kia", "sellerId": "user:ada", "amount": 60}),
    )
    .await;

    let (s1, r1) = call(
        router.clone(),
        "POST",
        "/api/v1/escrows/e3/fund",
        Some(KEY),
        json!({"idempotencyKey": "fund:e3:once"}),
    )
    .await;
    assert_eq!(s1, StatusCode::OK);
    let (s2, r2) = call(
        router.clone(),
        "POST",
        "/api/v1/escrows/e3/fund",
        Some(KEY),
        json!({"idempotencyKey": "fund:e3:once"}),
    )
    .await;
    assert_eq!(s2, StatusCode::OK);
    // same transaction — a replay returns exactly the same result
    assert_eq!(r1["txId"], r2["txId"]);
    assert_eq!(r2["replayed"], true);
    let (_, buyer) = call_empty(router.clone(), "GET", "/api/v1/accounts/user:kia/balance", Some(KEY)).await;
    assert_eq!(buyer["balance"], 40, "fund twice = one 60 deduction");
}

// ── SQL injection on the status filter — the sqlsafe guard rejects the contract ────────────

#[tokio::test]
async fn injection_attempt_on_status_filter_is_400() {
    let router = router_with_key(Some(KEY)).await;
    let (status, body) =
        call_empty(router, "GET", "/api/v1/escrows?status=%27%20OR%201%3D1%20--", Some(KEY)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "INVALID_REQUEST");
}

// ── Rate limiting — the sliding window on the wire ──────────────────────────────────────────

#[tokio::test]
async fn rate_limit_429_with_exact_meta() {
    let store = V3Store::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store.ensure_system_accounts().await.unwrap();
    let mut config = ServerConfig::base();
    config.admin_key = Some(KEY.into());
    config.rate_limit = 2; // tight cap for the test
    config.backup_dir = "data/backups".into();
    let router = build_router(AppState::new(store, config).shared());
    // admin bucket — the first two calls pass, the third gets 429 with the exact meta.limit
    let (_, _) = call_empty(router.clone(), "GET", "/api/v1/economy", Some(KEY)).await;
    let (_, _) = call_empty(router.clone(), "GET", "/api/v1/economy", Some(KEY)).await;
    let (status, body) = call_empty(router.clone(), "GET", "/api/v1/economy", Some(KEY)).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "RATE_LIMITED");
    assert_eq!(body["error"]["meta"]["limit"], 2);
}

/// CORS for allowlisted origins only — an unknown origin’s preflight gets no ACAO (M3).
#[tokio::test]
async fn cors_preflight_only_for_allowed_origins() {
    let router = router_with_key(Some(KEY)).await;

    let preflight = |router: axum::Router, origin: &'static str| {
        let req = Request::builder()
            .method("OPTIONS")
            .uri("/api/v1/health")
            .header("origin", origin)
            .header("access-control-request-method", "POST")
            .header("access-control-request-headers", "authorization,content-type")
            .body(Body::empty())
            .unwrap();
        router.clone().oneshot(req)
    };

    let res = preflight(router.clone(), "http://127.0.0.1:3000").await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "allowed origin preflight must pass");
    let acao = res
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    assert_eq!(acao, "http://127.0.0.1:3000");

    let res = preflight(router, "http://evil.example.com").await.unwrap();
    let acao = res.headers().get("access-control-allow-origin");
    assert!(acao.is_none(), "unknown origin must not receive ACAO header");
}
