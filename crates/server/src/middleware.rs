//! Layered wire security — port of `guards.ts` as Axum middleware.
//!
//! Two layers, in the same order as TS:
//! 1. **Rate** — a 60-second sliding window; a caller with a valid admin key gets the
//!    shared `admin` bucket; the rest key by ip (X-Forwarded-For first, then
//!    `ConnectInfo`, otherwise `local`). The 429 body is exactly Phase 3 (`meta.limit`).
//! 2. **Admin** — fail-closed: server key unset → 503 `ADMIN_DISABLED`;
//!    missing header → 401 `ADMIN_KEY_REQUIRED`; wrong → 401 `INVALID_ADMIN_KEY`.
//!    Constant-time comparison from `contracts::security` (subtle).
//!
//! Mini-app user identity (`initData` → bearer session) is verified in the M3 handlers —
//! the two general layers apply to every route right here.

use axum::extract::{ConnectInfo, Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use kian_db::UserRow;
use std::net::SocketAddr;
use std::sync::Arc;

use kian_contracts::{verify_kian_key, HttpFailure};

use crate::error::ApiFailure;
use crate::state::AppState;

/// Admin bucket key — all trusted callers share one bucket (as in TS).
const ADMIN_BUCKET: &str = "admin";

/// Admin guard — invoked by handlers (every route `Security::Admin`).
///
/// # Errors
/// The three explicit Phase 3 errors: `ADMIN_DISABLED`/`ADMIN_KEY_REQUIRED`/`INVALID_ADMIN_KEY`.
pub fn require_admin(headers: &HeaderMap, admin_key: Option<&str>) -> Result<(), ApiFailure> {
    let header_value = headers.get("x-kian-key").and_then(|v| v.to_str().ok());
    if admin_key.is_none() {
        return Err(HttpFailure::admin_disabled().into());
    }
    if header_value.is_none() {
        return Err(HttpFailure::admin_key_required().into());
    }
    if !verify_kian_key(header_value, admin_key) {
        return Err(HttpFailure::invalid_admin_key().into());
    }
    Ok(())
}

/// Bucket key for this request — valid admin → shared bucket; everyone else → ip.
///
/// Security fix (2025-11 load test): X-Forwarded-For is read only when
/// `trust_proxy` is enabled (deployment behind a trusted proxy like Railway). With
/// `trust_proxy=false` only the socket IP counts — in a direct deployment any client
/// could previously bypass the rate cap with a forged XFF.
fn bucket_key(
    headers: &HeaderMap,
    admin_key: Option<&str>,
    connect_info: Option<SocketAddr>,
    trust_proxy: bool,
) -> String {
    let header_value = headers.get("x-kian-key").and_then(|v| v.to_str().ok());
    if verify_kian_key(header_value, admin_key) {
        return ADMIN_BUCKET.to_owned();
    }
    if trust_proxy
        && let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            // first ip in the chain = the real client (behind a proxy)
            if let Some(first) = xff.split(',').next() {
                let trimmed = first.trim();
                if !trimmed.is_empty() {
                    return format!("ip:{trimmed}");
                }
            }
        }
    if let Some(addr) = connect_info {
        return format!("ip:{}", addr.ip());
    }
    // oneshot tests and in-process calls — no ip
    "local".to_owned()
}

/// Rate middleware — applied before every `/api/v1` route.
///
/// # Errors
/// 429 with the exact Phase 3 body (`RATE_LIMITED` + `meta.limit`).
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiFailure> {
    let connect_info = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|ci| ci.0);
    let key = bucket_key(
        request.headers(),
        state.config.admin_key.as_deref(),
        connect_info,
        state.config.trust_proxy,
    );
    // sharded limiter — per-shard lock inside check; no global lock
    let decision = state.limiter.check(&key);
    if let Err(failure) = decision {
        tracing::warn!(bucket = %key, "rate limited");
        return Err(failure.into());
    }
    Ok(next.run(request).await)
}

// ── M3 bearer session ──────────────────────────────────────────────────────────

/// Extract the session token from `Authorization: Bearer <token>` — missing/malformed → `None`.
#[must_use]
pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("authorization")?.to_str().ok()?;
    let token = raw.strip_prefix("Bearer ")?.trim();
    if token.is_empty() { None } else { Some(token.to_owned()) }
}

/// M3 session guard — returns a valid user or yields 401.
///
/// fail-closed like the admin guard: missing token → `SESSION_REQUIRED`; unknown or
/// expired → `SESSION_EXPIRED`. Identity is never read from the request body.
///
/// # Errors
/// `SESSION_REQUIRED` (401) / `SESSION_EXPIRED` (401) / SQL error (500).
pub async fn require_session(state: &AppState, headers: &HeaderMap) -> Result<UserRow, ApiFailure> {
    let token =
        bearer_token(headers).ok_or_else(kian_contracts::HttpFailure::session_required)?;
    match state.store.session_user(&token).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(kian_contracts::HttpFailure::session_expired().into()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    const KEY: &str = "test-admin-key-0123456789";

    #[test]
    fn admin_guard_fail_closed_matrix() {
        let mut headers = HeaderMap::new();
        // server without a key → 503 always (never open)
        assert_eq!(require_admin(&headers, None).unwrap_err().status(), 503);
        // missing header → 401
        assert_eq!(require_admin(&headers, Some(KEY)).unwrap_err().status(), 401);
        // wrong header → 401
        headers.insert("x-kian-key", "wrong-key-0123456789".parse().unwrap());
        assert_eq!(require_admin(&headers, Some(KEY)).unwrap_err().status(), 401);
        // correct header → pass
        headers.insert("x-kian-key", KEY.parse().unwrap());
        assert!(require_admin(&headers, Some(KEY)).is_ok());
    }

    #[test]
    fn bucket_key_prefers_admin_then_xff_then_connect_info() {
        let mut headers = HeaderMap::new();
        assert_eq!(bucket_key(&headers, Some(KEY), None, true), "local");
        headers.insert("x-kian-key", KEY.parse().unwrap());
        assert_eq!(bucket_key(&headers, Some(KEY), None, true), "admin");
        headers.remove("x-kian-key");
        headers.insert("x-forwarded-for", "1.2.3.4, 5.6.7.8".parse().unwrap());
        assert_eq!(bucket_key(&headers, Some(KEY), None, true), "ip:1.2.3.4");
        assert_eq!(
            bucket_key(&headers, Some(KEY), Some("9.9.9.9:80".parse().unwrap()), true),
            "ip:1.2.3.4"
        );
    }

    /// Security fix (2025-11 load test): with `trust_proxy=false` a forged XFF is
    /// ignored — the bucket comes from the socket IP, not from a client-controlled header.
    #[test]
    fn bucket_key_ignores_xff_when_proxy_untrusted() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        // without trust: the header is inert — no socket → local, socket → socket ip
        assert_eq!(bucket_key(&headers, None, None, false), "local");
        assert_eq!(
            bucket_key(&headers, None, Some("9.9.9.9:80".parse().unwrap()), false),
            "ip:9.9.9.9"
        );
        // with trust: the same header is authoritative (behavior behind a proxy)
        assert_eq!(bucket_key(&headers, None, Some("9.9.9.9:80".parse().unwrap()), true), "ip:1.2.3.4");
    }
}
