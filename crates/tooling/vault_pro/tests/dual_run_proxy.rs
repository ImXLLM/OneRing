//! Phase 8 dual-run proxy integration test — two simulated upstreams on a
//! temporary port, the full shadow → pct → rollback cycle (ADR-009 §12).

// integration test — unwrap/expect are intentional
#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use dropagentx_tooling::dual_run::DualRunConfig;
use dropagentx_tooling::proxy::{proxy_router, ProxyState};
use http_body_util::BodyExt;
use tokio::net::TcpListener;
use tower::ServiceExt;

/// Simulated upstream — `GET /api/v1/health` and `GET /api/v1/echo/{what}`
/// with a tag prefix (TS/RUST) so response differences are visible.
async fn spawn_upstream(tag: &'static str) -> String {
    let app: Router = Router::new()
        .route("/api/v1/health", axum::routing::get(move || async move { format!("{tag}:health") }))
        .route(
            "/api/v1/echo/{what}",
            axum::routing::get(
                move |axum::extract::Path(what): axum::extract::Path<String>| async move {
                    format!("{tag}:{what}")
                },
            ),
        )
        // same-response path — both upstreams return identical bodies (parity match test)
        .route("/api/v1/same", axum::routing::get(|| async { "ok:1" }));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// One GET request to the proxy — response (status, body).
async fn send(router: &Router, uri: &str) -> (StatusCode, String) {
    let req = Request::builder().method("GET").uri(uri).body(Body::empty()).unwrap();
    let res = router.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// POST with a JSON body to the proxy.
async fn post(router: &Router, uri: &str, body: &str) -> StatusCode {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap();
    router.clone().oneshot(req).await.unwrap().status()
}

#[tokio::test]
async fn proxy_full_cycle_shadow_pct_rollback() {
    let ts = spawn_upstream("TS").await;
    let rust = spawn_upstream("RUST").await;
    // step 1 — shadow: TS is the responder + parity comparison happens
    let state = ProxyState::new(DualRunConfig::default(), ts.clone(), rust.clone());
    let router = proxy_router(state.clone());

    let (status, body) = send(&router, "/api/v1/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "TS:health");
    let snap = state.stats.snapshot();
    assert_eq!(snap["shadow"], 1);
    assert_eq!(snap["parityDiff"], 1, "\u{62a}\u{6af} \u{622}\u{67e}‌\u{627}\u{633}\u{62a}\u{631}\u{6cc}\u{645}‌\u{647}\u{627} \u{645}\u{62a}\u{641}\u{627}\u{648}\u{62a} \u{627}\u{633}\u{62a} — \u{646}\u{631}\u{645}\u{627}\u{644}‌\u{633}\u{627}\u{632} \u{628}\u{627}\u{6cc}\u{62f} \u{645}\u{63a}\u{627}\u{6cc}\u{631}\u{62a} \u{631}\u{627} \u{628}\u{634}\u{645}\u{627}\u{631}\u{62f}");

    // same-response path — the parity match is counted
    let (status, body) = send(&router, "/api/v1/same").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok:1");
    let snap = state.stats.snapshot();
    assert_eq!(snap["parityMatch"], 1, "{snap}");

    // step 2 — step promotion: pct=100 → everything to Rust
    assert_eq!(post(&router, "/_dualrun/pct", r#"{"pct":100}"#).await, StatusCode::OK);
    let (status, body) = send(&router, "/api/v1/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "RUST:health");
    let snap = state.stats.snapshot();
    assert_eq!(snap["rust"], 1);

    // invalid control — pct > 100 is rejected
    assert_eq!(
        post(&router, "/_dualrun/pct", r#"{"pct":150}"#).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // step 3 — instant rollback: rollback → everything to TS
    assert_eq!(post(&router, "/_dualrun/rollback", "").await, StatusCode::OK);
    let (_, body) = send(&router, "/api/v1/health").await;
    assert_eq!(body, "TS:health");
    let snap = state.stats.snapshot();
    assert_eq!(snap["ts"], 1, "{snap} — \u{628}\u{631}\u{6af}\u{634}\u{62a} \u{6cc}\u{639}\u{646}\u{6cc} TS \u{62e}\u{627}\u{644}\u{635} (\u{628}\u{62f}\u{648}\u{646} \u{633}\u{627}\u{6cc}\u{647})");
}

#[tokio::test]
async fn pct_split_is_deterministic_and_bounded() {
    let ts = spawn_upstream("TS").await;
    let rust = spawn_upstream("RUST").await;
    let state = ProxyState::new(
        DualRunConfig { rust_traffic_pct: 50, shadow: false, ..DualRunConfig::default() },
        ts,
        rust,
    );
    let router = proxy_router(state.clone());
    let mut rust_hits = 0u32;
    for i in 0..200 {
        let (_, body) = send(&router, &format!("/api/v1/echo/r{i}")).await;
        if body.starts_with("RUST:") {
            rust_hits += 1;
        }
    }
    // allocation is deterministic — the same path always goes to the same side; with 200 unique paths
    // the distribution is near 50% (no hard bound; just a chaos guard)
    assert!(rust_hits > 40 && rust_hits < 160, "rust_hits = {rust_hits}");
    // status endpoint — shows the counters
    let (_, body) = send(&router, "/_dualrun/status").await;
    assert!(body.contains("\"rust\""), "{body}");
}

#[tokio::test]
async fn dead_upstream_is_fail_closed_502() {
    // TS upstream alive, Rust dead (closed port)
    let ts = spawn_upstream("TS").await;
    let rust = "http://127.0.0.1:1".to_string(); // closed
    let state = ProxyState::new(
        DualRunConfig { rust_traffic_pct: 100, shadow: false, ..DualRunConfig::default() },
        ts,
        rust,
    );
    let router = proxy_router(state);
    let (status, body) = send(&router, "/api/v1/health").await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert!(body.contains("UPSTREAM_DOWN"), "{body}");
}

#[tokio::test]
async fn shadow_parity_diff_is_counted() {
    // the upstreams deliberately return different responses — the parity diff must be counted
    let ts = spawn_upstream("TS").await;
    let rust = spawn_upstream("RUST").await;
    let state = ProxyState::new(DualRunConfig::default(), ts, rust);
    let router = proxy_router(state.clone());
    // echo path — TS and RUST return different bodies (different tag)
    let (_, body) = send(&router, "/api/v1/echo/x").await;
    assert_eq!(body, "TS:x");
    let snap = state.stats.snapshot();
    assert_eq!(snap["parityDiff"], 1, "{snap}");
}
