//! HTTP error envelope — the bridge between layer errors and the wire body `{error:{code,message,meta?}}`.
//!
//! Honest mapping (port of `errors.ts` + the explicit codes of `api.ts`/`guards.ts`):
//! * [`HttpFailure`] (phase 3) — carries an explicit status and code.
//! * [`DomainFailure`] — domain code → 404/422/500 via `status_for_domain_code`.
//! * [`DbError`] — SQL/corrupt → 500 `INTERNAL`; a half-finished persistence never corrupts
//!   state and the client gets an honest message.
//!
//! The 404 response for unknown paths is enveloped too (additive-only) — the SPA and bot never
//! receive a shapeless body.

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use kian_contracts::{DomainFailure, ErrorEnvelope, HttpFailure};
use kian_db::DbError;

/// Ready-made HTTP error — each variant yields the exact wire body.
#[derive(Debug)]
pub enum ApiFailure {
    /// Explicit server error (`HttpFailure` phase 3).
    Http(HttpFailure),
    /// Pure domain error — mapped to 404/422/500 in the contract layer.
    Domain(DomainFailure),
    /// Persistence infrastructure error.
    Db(DbError),
}

impl From<HttpFailure> for ApiFailure {
    fn from(f: HttpFailure) -> Self {
        Self::Http(f)
    }
}

impl From<DomainFailure> for ApiFailure {
    fn from(f: DomainFailure) -> Self {
        Self::Domain(f)
    }
}

impl From<kian_domain::DomainError> for ApiFailure {
    fn from(e: kian_domain::DomainError) -> Self {
        Self::Domain(DomainFailure::new(e))
    }
}

impl From<DbError> for ApiFailure {
    fn from(e: DbError) -> Self {
        match e {
            // a pure domain error from inside the transaction — the honest 404/422 mapping is preserved
            DbError::Domain(domain_error) => Self::Domain(DomainFailure::new(domain_error)),
            other => Self::Db(other),
        }
    }
}

impl ApiFailure {
    /// Honest status.
    #[must_use]
    pub fn status(&self) -> u16 {
        match self {
            Self::Http(f) => f.status,
            Self::Domain(f) => f.status(),
            Self::Db(_) => 500,
        }
    }

    /// Wire body — exactly the `{error:{…}}` shape.
    #[must_use]
    pub fn envelope(&self) -> ErrorEnvelope {
        match self {
            Self::Http(f) => f.to_envelope(),
            Self::Domain(f) => f.to_envelope(),
            Self::Db(e) => ErrorEnvelope {
                error: kian_contracts::ApiError::new(
                    "INTERNAL",
                    format!("Persistence layer failed — {e}"),
                ),
            },
        }
    }
}

impl std::fmt::Display for ApiFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let env = self.envelope();
        write!(f, "{} — {}: {}", self.status(), env.error.code, env.error.message)
    }
}

impl IntoResponse for ApiFailure {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = self.envelope();
        // structured log — the database audit row owns history; here only a trace
        let (code, message) = (&body.error.code, &body.error.message);
        tracing::warn!(status = self.status(), code = %code, message = %message, "api failure");
        (
            status,
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::to_string(&body).unwrap_or_default(),
        )
            .into_response()
    }
}

/// The handlers' standard Result.
pub type ApiResult<T> = Result<T, ApiFailure>;

#[cfg(test)]
mod tests {
    use super::*;
    use kian_contracts::HttpFailure;
    use kian_db::DbError;
    use kian_domain::{DomainError, DomainErrorCode};

    #[test]
    fn http_failure_keeps_explicit_status() {
        let f = ApiFailure::Http(HttpFailure::admin_disabled());
        assert_eq!(f.status(), 503);
        assert_eq!(f.envelope().error.code, "ADMIN_DISABLED");
    }

    #[test]
    fn domain_failure_maps_honest_status() {
        let f = ApiFailure::from(DomainError::new(DomainErrorCode::EscrowNotFound, "x"));
        assert_eq!(f.status(), 404);
        let f = ApiFailure::from(DomainError::new(DomainErrorCode::AccountOverdraft, "x"));
        assert_eq!(f.status(), 422);
    }

    #[test]
    fn db_failure_is_internal_500() {
        let f = ApiFailure::Db(DbError::Corrupt("bad row".into()));
        assert_eq!(f.status(), 500);
        assert_eq!(f.envelope().error.code, "INTERNAL");
    }
}
