//! `dual-run` — the phase 8 transition proxy: TS and Rust side by side on **the same** SQLite.
//!
//! Environment variables:
//! | Variable | Default | Role |
//! |---|---|---|
//! | `PORT` | `3001` | Proxy port |
//! | `TS_URL` | required | The Node upstream (e.g. `http://127.0.0.1:3000`) |
//! | `RUST_URL` | required | The Rust upstream |
//! | `RUST_TRAFFIC_PCT` | `0` | The single switch variable (0=shadow/rollback, 100=full) |
//! | `SHADOW` | `1` | `1` = shadow execution active (valid only with pct=0) |
//! | `STABILITY_DAYS` | `0` | Stability days — the HITL shield opens at 3 |
//!
//! Live control points: `GET /_dualrun/status` · `POST /_dualrun/pct {"pct":10}`
//! · `POST /_dualrun/rollback` — every change is logged as a JSON line on stderr.
//!
//! Boot with contradictory configuration is rejected (fail-closed — like the phase 4 ServerConfig).

use dropagentx_tooling::dual_run::DualRunConfig;
use dropagentx_tooling::proxy::{proxy_router, ProxyState};

/// Read an optional env var with a default.
fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.into())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = env_or("PORT", "3001").parse()?;
    let ts_url = std::env::var("TS_URL").ok().filter(|v| !v.trim().is_empty());
    let rust_url = std::env::var("RUST_URL").ok().filter(|v| !v.trim().is_empty());
    let (Some(ts_url), Some(rust_url)) = (ts_url, rust_url) else {
        eprintln!("dual-run: TS_URL \u{648} RUST_URL \u{627}\u{644}\u{632}\u{627}\u{645}\u{6cc}‌\u{627}\u{646}\u{62f} — \u{628}\u{648}\u{62a} \u{631}\u{62f} \u{634}\u{62f} (fail-closed)");
        std::process::exit(2);
    };

    let cfg = DualRunConfig {
        rust_traffic_pct: env_or("RUST_TRAFFIC_PCT", "0").parse()?,
        shadow: env_or("SHADOW", "1") != "0",
        stability_days: env_or("STABILITY_DAYS", "0").parse()?,
        hitl_shield_din: dropagentx_tooling::dual_run::HITL_SHIELD_DIN,
    };
    if let Err(msg) = cfg.validate() {
        eprintln!("dual-run: \u{6a9}\u{627}\u{646}\u{641}\u{6cc}\u{6af} \u{645}\u{62a}\u{646}\u{627}\u{642}\u{636} — \u{628}\u{648}\u{62a} \u{631}\u{62f} \u{634}\u{62f}: {msg}");
        std::process::exit(2);
    }

    let state = ProxyState::new(cfg, ts_url.clone(), rust_url.clone());
    let app = proxy_router(state);
    let addr = format!("0.0.0.0:{port}");
    println!("dual-run proxy → {addr}  (TS={ts_url} · RUST={rust_url})");
    println!("  \u{6a9}\u{646}\u{62a}\u{631}\u{644}: GET /_dualrun/status · POST /_dualrun/pct · POST /_dualrun/rollback");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
