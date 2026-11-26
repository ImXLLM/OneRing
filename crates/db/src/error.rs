//! Persistence-layer error model — thiserror in the library, `anyhow` only in binaries.

use kian_domain::DomainError;

/// Persistence adapter error — always carries enough context to debug.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// sqlx error (connection, query, UNIQUE constraint, etc.)
    #[error("sqlx: {0}")]
    Sql(#[from] sqlx::Error),
    /// Error running the versioned schema migrations
    #[error("migrate: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    /// Pure domain error (from running a closure inside a transaction)
    #[error("domain: {0}")]
    Domain(#[from] DomainError),
    /// Invalid configuration (database URL, legacy path, etc.)
    #[error("config: {0}")]
    Config(String),
    /// Corrupt store data (e.g. an unknown account `kind`)
    #[error("corrupt store: {0}")]
    Corrupt(String),
    /// Legacy migration rejected for violating an invariant (fail-closed)
    #[error("migration blocked: {0}")]
    Migration(String),
}

/// The standard Result of the db layer.
pub type DbResult<T> = Result<T, DbError>;
