//! `kian-server` — the axum gateway of the KIAN money kernel (M2).
//!
//! HTTP layer over [`kian_db`] and [`kian_domain`]: layered security (rate + the
//! fail-closed `X-KIAN-Key` guard), wire-body validation at the edge, domain execution through
//! the pure path inside `V3Store::apply` (the single audit/outbox point) and honest
//! error mapping to `{error:{code,message,meta?}}`.
//!
//! - [`config`] — env-only configuration (no secrets in code)
//! - [`state`] — the shared `AppState`
//! - [`middleware`] — rate limit + admin guard
//! - [`routes`] — `/api/v1` wiring
//! - [`handlers`] — handlers (system/accounts/escrows)
//! - [`views`] — domain → wire
//! - [`error`] — the HTTP error envelope
#![forbid(unsafe_code)]

pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;
pub mod views;

/// Crate ID — for the smoke test and logs.
pub const NAME: &str = "kian-server";

/// Crate version — read from `Cargo.toml`.
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
