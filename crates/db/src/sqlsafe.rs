//! Safe SQL guard — the single human-audit point for dynamic SQL in `kian-db`.
//!
//! sqlx 0.9 blocks every non-literal SQL string with `E0277` so that every possible
//! injection point is explicitly audited. In KIAN all 8 dynamic SQL points (in
//! `queries.rs` only) go through this single passage so the audit is centralized and searchable:
//! `rg "audited\(" crates/db/src`.
//!
//! Audit contract (every change here must re-confirm these five clauses):
//! 1. The dynamic string is built only from "static fragments inside the crate" — table/column
//!    names come from enum whitelists (`HitlRequest`, `Product`, `AuditLog`, …);
//! 2. Every value derived from user input must go through `.bind()` and is never
//!    interpolated into SQL with `format!`;
//! 3. Status filters come from serializable in-crate enums (`as_str`
//!    on `HitlStatus`/`ProductStatus` — not a raw client string);
//! 4. Backup file paths are built only inside `data/backups/` with server-generated
//!    names (no path input from the client);
//! 5. The `integration_tests.rs` integration tests reject classic injection
//!    paths (`' OR 1=1 --`) through these same passages.
//!
//! Port of the `sqlsafe` guard philosophy from the 10x repo — a legacy lesson from security stewards.
#![forbid(unsafe_code)]

use sqlx::AssertSqlSafe;

/// Human audit of a dynamic SQL string — the only way a non-literal string passes the gate.
///
/// # Calling contract
/// The caller must guarantee clauses 1–3 of the document above for `sql`; this function
/// adds no runtime validation because the rule "never interpolate user input
/// into SQL" is enforced across all call sites.
pub(crate) fn audited(sql: String) -> AssertSqlSafe<String> {
    AssertSqlSafe(sql)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The contract is sacred — output equals input; the guard is only a point audit.
    #[test]
    fn audited_passes_through_the_string_verbatim() {
        let sql = r#"SELECT "id" FROM "HitlRequest" WHERE "status" = ?"#;
        let wrapped = audited(sql.to_owned());
        assert_eq!(wrapped.0, sql);
    }
}
