//! امنیت چندلایه روی سیم — پورت `guards.ts` به‌صورت middleware آکسوم.
//!
//! دو لایه، به همان ترتیب TS:
//! ۱. **نرخ** — پنجرهٔ لغزان ۶۰ثانیه‌ای؛ تماس‌گیرندهٔ دارای کلید ادمین معتبر باکت
//!    مشترک `admin` می‌گیرد، بقیه با کلید ip (X-Forwarded-For اول، سپس
//!    `ConnectInfo`، وگرنه `local`). بدنهٔ ۴۲۹ عین فاز ۳ است (`meta.limit`).
//! ۲. **ادمین** — fail-closed: کلید سرور تنظیم‌نشده → 503 `ADMIN_DISABLED`؛
//!    هدر غایب → 401 `ADMIN_KEY_REQUIRED`؛ نادرست → 401 `INVALID_ADMIN_KEY`.
//!    مقایسهٔ زمان‌ثابت از `contracts::security` (subtle).
//!
//! هویت کاربر مینی‌اپ (`initData` → نشست bearer) در هندلرهای M3 بررسی می‌شود —
//! همین‌جا دو لایهٔ عمومی هر مسیر اعمال می‌شوند.

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

/// کلید باکت ادمین — همهٔ تماس‌های معتمد یک باکت مشترک دارند (مثل TS).
const ADMIN_BUCKET: &str = "admin";

/// گارد ادمین — از هندلرها صدا زده می‌شود (هر مسیر `Security::Admin`).
///
/// # Errors
/// سه خطای صریح فاز ۳: `ADMIN_DISABLED`/`ADMIN_KEY_REQUIRED`/`INVALID_ADMIN_KEY`.
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

/// کلید باکت برای این درخواست — ادمین معتبر → باکت مشترک؛ بقیه → ip.
///
/// اصلاح امنیتی (تست بار ۲۰۲۵-۱۱): X-Forwarded-For فقط وقتی خوانده می‌شود که
/// `trust_proxy` فعال باشد (استقرار پشت پروکسی معتبر مثل Railway). با
/// `trust_proxy=false` فقط IP سوکت ملاک است — در استقرار مستقیم، هر کلاینت
/// با XFF جعلی می‌توانست سقف نرخ را دور بزند.
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
    if trust_proxy {
        if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            // اولین ip زنجیره = کلاینت واقعی (پشت proxy)
            if let Some(first) = xff.split(',').next() {
                let trimmed = first.trim();
                if !trimmed.is_empty() {
                    return format!("ip:{trimmed}");
                }
            }
        }
    }
    if let Some(addr) = connect_info {
        return format!("ip:{}", addr.ip());
    }
    // oneshot تست‌ها و فراخوانی درون‌پردازشی — بدون ip
    "local".to_owned()
}

/// middleware نرخ — قبل از همهٔ مسیرهای `/api/v1` اعمال می‌شود.
///
/// # Errors
/// ۴۲۹ با بدنهٔ دقیق فاز ۳ (`RATE_LIMITED` + `meta.limit`).
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
    // limiter شاردشده — قفل per-shard درون check؛ بدون قفل سراسری
    let decision = state.limiter.check(&key);
    if let Err(failure) = decision {
        tracing::warn!(bucket = %key, "rate limited");
        return Err(failure.into());
    }
    Ok(next.run(request).await)
}

// ── نشست bearer M3 ──────────────────────────────────────────────────────────

/// استخراج توکن نشست از `Authorization: Bearer <token>` — غایب/بدشکل → `None`.
#[must_use]
pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("authorization")?.to_str().ok()?;
    let token = raw.strip_prefix("Bearer ")?.trim();
    if token.is_empty() { None } else { Some(token.to_owned()) }
}

/// گوار نشست M3 — کاربرِ معتبر را برمی‌گرداند یا 401 می‌دهد.
///
/// fail-closed همان گارد ادمین: توکن غایب → `SESSION_REQUIRED`؛ ناشناخته یا
/// منقضی → `SESSION_EXPIRED`. هویت هرگز از بدنهٔ درخواست خوانده نمی‌شود.
///
/// # Errors
/// `SESSION_REQUIRED` (۴۰۱) / `SESSION_EXPIRED` (۴۰۱) / خطای SQL (۵۰۰).
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
        // سرور بدون کلید → 503 همیشه (هرگز open)
        assert_eq!(require_admin(&headers, None).unwrap_err().status(), 503);
        // هدر غایب → 401
        assert_eq!(require_admin(&headers, Some(KEY)).unwrap_err().status(), 401);
        // هدر نادرست → 401
        headers.insert("x-kian-key", "wrong-key-0123456789".parse().unwrap());
        assert_eq!(require_admin(&headers, Some(KEY)).unwrap_err().status(), 401);
        // هدر درست → عبور
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

    /// اصلاح امنیتی (تست بار ۲۰۲۵-۱۱): با trust_proxy=false، XFF جعلی نادیده
    /// می‌شود — باکت از IP سوکت گرفته می‌شود، نه از هدر کنترل‌شدهٔ کلاینت.
    #[test]
    fn bucket_key_ignores_xff_when_proxy_untrusted() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        // بدون اعتماد: هدر بی‌اثر — سوکت غایب → local، سوکت → ip سوکت
        assert_eq!(bucket_key(&headers, None, None, false), "local");
        assert_eq!(
            bucket_key(&headers, None, Some("9.9.9.9:80".parse().unwrap()), false),
            "ip:9.9.9.9"
        );
        // با اعتماد: همان هدر ملاک است (رفتار پشت پروکسی)
        assert_eq!(bucket_key(&headers, None, Some("9.9.9.9:80".parse().unwrap()), true), "ip:1.2.3.4");
    }
}
