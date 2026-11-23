//! `kian-server` — دروازهٔ axum کرنل پول KIAN (M2).
//!
//! لایهٔ HTTP روی [`kian_db`] و [`kian_domain`]: امنیت چندلایه (نرخ + گارد
//! fail-closed `X-KIAN-Key`)، اعتبارسنجی بدنهٔ وایر در لبه، اجرای دامنه از
//! مسیر خالص داخل `V3Store::apply` (نقطهٔ واحد audit/outbox) و نگاشت صادقانهٔ
//! خطاها به `{error:{code,message,meta?}}`.
//!
//! - [`config`] — پیکربندی فقط-env (بدون secret در کد)
//! - [`state`] — `AppState` مشترک
//! - [`middleware`] — نرخ + گارد ادمین
//! - [`routes`] — سیم‌کشی `/api/v1`
//! - [`handlers`] — هندلرها (system/accounts/escrows)
//! - [`views`] — دامنه → وایر
//! - [`error`] — پاکت خطای HTTP
#![forbid(unsafe_code)]

pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;
pub mod views;

/// شناسه کرِیت — برای smoke test و لاگ‌ها.
pub const NAME: &str = "kian-server";

/// نسخه کرِیت — از `Cargo.toml` خوانده می‌شود.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use config::ServerConfig;
pub use error::{ApiFailure, ApiResult};
pub use routes::build_router;
pub use state::AppState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(NAME, "kian-server");
        assert_ne!(VERSION, "");
    }
}
