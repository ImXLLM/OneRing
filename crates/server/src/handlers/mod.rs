//! HTTP handlers — every handler has three stages: security (admin guard/rate from middleware
//! and [`require_admin`]), wire body validation ([`parse_body`] — like zod at
//! the HTTP edge), and domain execution via the pure path + [`V3Store`].
//!
//! No handler builds JSON directly — responses are built from the Phase 3 DTOs so that
//! the contract never drifts from the handler (ADR-005).

pub mod accounts;
pub mod admin;
pub mod agents;
pub mod ai;
pub mod arda;
pub mod auth;
pub mod bot;
pub mod builder;
pub mod chains;
pub mod deposits;
pub mod escrows;
pub mod grok_lab;
pub mod hermes;
pub mod market;
pub mod monopoly;
pub mod nation;
pub mod quant;
pub mod studio;
pub mod system;
pub mod war_room;
pub mod wallet;

use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use crate::error::ApiFailure;

/// Wire character sanitization — every character outside `^[A-Za-z0-9:_.-]+$` becomes `_`
/// so the output always passes the `Id` pattern (D2 — forensic inspection).
fn sanitize_wire_chars(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '.' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Deterministic 16-digit SHA-256 suffix for overflow ids — a pure function of the input,
/// so a fixed `raw_id` always yields the same suffix (no atomic drift).
fn deterministic_id_suffix(raw_id: &str) -> String {
    let digest = Sha256::digest(raw_id.as_bytes());
    let hex = format!("{digest:x}");
    hex.chars().take(16).collect()
}

/// D2 — safe prefixed wire id — replaces `format!("{prefix}:{id}")`.
///
/// A DB-originated id can be up to 128 characters; adding the `hitl:` prefix
/// breaks the `Id::new` cap and the `.expect` in `admin.rs` would panic
/// the whole server. This function guarantees the output is `1..=128` characters and matches
/// the `^[A-Za-z0-9:_.-]+$` pattern:
/// - if `prefix:sanitized` fits, it is returned as-is;
/// - otherwise the truncated head of ≤100 characters + the deterministic `_{sha256_hex16}`.
#[must_use]
pub fn safe_prefixed_wire_id(prefix: &str, raw_id: &str) -> String {
    let p = sanitize_wire_chars(prefix);
    let body = sanitize_wire_chars(raw_id);
    let direct = format!("{p}:{body}");
    if (1..=128).contains(&direct.chars().count()) {
        return direct;
    }
    // truncated head: at most 100 characters of `prefix:body` + a 17-character suffix
    // (`_` + 16 hex) → total always ≤ 117 and ≥ 1 — always valid for `Id::new`.
    let head: String = direct.chars().take(100).collect();
    format!("{head}_{}", deterministic_id_suffix(raw_id))
}

/// D2 — safely convert any DB-originated id to `Id` with no `.expect` on
/// malformed data. An empty/whitespace string → the static id `id:{fallback_tag}`; characters
/// are sanitized; length overflow is rescued with a deterministic 16-hex suffix — the server
/// never halts on a corrupted DB row.
///
/// `raw` is accepted as `impl Into<String>` — callers passing a `String`
/// (DB-originated ids) work unchanged.
///
/// # Panics
/// Only on the unreachable static fallback path — when even
/// `id:sanitized-fallback` (whose pattern and length are fixed and valid) is rejected;
/// impossible for any real input.
#[must_use]
pub fn safe_wire_id(raw: impl Into<String>, fallback_tag: &str) -> kian_contracts::Id {
    let raw = raw.into();
    let tag = sanitize_wire_chars(fallback_tag);
    let static_fallback = || {
        kian_contracts::Id::new(format!("id:{tag}")).unwrap_or_else(|| {
            kian_contracts::Id::new("id:sanitized-fallback").expect("valid static fallback")
        })
    };
    if raw.trim().is_empty() {
        return static_fallback();
    }
    let sanitized = sanitize_wire_chars(&raw);
    if let Some(id) = kian_contracts::Id::new(sanitized.clone()) {
        return id;
    }
    // after sanitization the only invalid case is length overflow — deterministic rescue:
    let salvaged = safe_prefixed_wire_id("id", &sanitized);
    kian_contracts::Id::new(salvaged).unwrap_or_else(static_fallback)
}

/// Wire body validation — the same spot zod holds in TS: invalid body → 400
/// `INVALID_REQUEST` with an issue count (serde yields one root error —
/// honestly at least 1). An empty body is interpreted as `{}` for all-optional shapes.
///
/// # Errors
/// [`HttpFailure::invalid_request`] — status 400 with `meta.issues`.
pub fn parse_body<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ApiFailure> {
    let raw: &[u8] = if bytes.iter().all(u8::is_ascii_whitespace) {
        b"{}"
    } else {
        bytes
    };
    serde_json::from_slice(raw).map_err(|_| kian_contracts::HttpFailure::invalid_request(1).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kian_contracts::FundEscrowRequest;

    #[test]
    fn empty_body_becomes_default_object_for_optional_forms() {
        let req: FundEscrowRequest = parse_body(b"").unwrap();
        assert_eq!(req, FundEscrowRequest::default());
    }

    #[test]
    fn invalid_body_is_invalid_request_400() {
        let err = parse_body::<FundEscrowRequest>(b"{\"idempotencyKey\":\"short\"}").unwrap_err();
        assert_eq!(err.status(), 400);
        assert_eq!(err.envelope().error.code, "INVALID_REQUEST");
    }

    // ── D2 — forensic inspection: safe wire id ────────────────────────────────

    /// A 128-character escrow id (the legal `Id::new` cap) with the `hitl:` prefix
    /// makes 133 — it must be rescued deterministically without panic and pass `Id::new`.
    #[test]
    fn hitl_wire_id_survives_128_char_escrow_id_without_panicking() {
        let escrow_id = format!("escrow:{}", "a1".repeat(61)); // 129 characters, then a cut to the legal cap of 128
        let escrow_id = escrow_id.chars().take(128).collect::<String>();
        assert_eq!(escrow_id.chars().count(), 128);
        let hitl = safe_prefixed_wire_id("hitl", &escrow_id);
        assert!(
            (1..=128).contains(&hitl.chars().count()),
            "HITL wire id must stay within 1..=128, got {}",
            hitl.chars().count()
        );
        let parsed = kian_contracts::Id::new(hitl.clone());
        assert!(parsed.is_some(), "sanitized HITL id must parse as wire Id: {hitl}");
        // determinism: fixed input → fixed output
        assert_eq!(hitl, safe_prefixed_wire_id("hitl", &escrow_id));
        // the semantic prefix is kept so the HITL operator still recognizes it
        assert!(hitl.starts_with("hitl:"));
    }

    #[test]
    fn short_ids_pass_through_unchanged() {
        assert_eq!(safe_prefixed_wire_id("hitl", "escrow-123"), "hitl:escrow-123");
        assert_eq!(safe_prefixed_wire_id("hitl", "a".repeat(122).as_str()).chars().count(), 127);
    }

    #[test]
    fn hostile_chars_are_sanitized_and_oversize_ids_recovered_deterministically() {
        let hostile = "escrow/id \u{628}\u{627} \u{641}\u{627}\u{635}\u{644}\u{647} \u{648} \u{6cc}\u{648}\u{646}\u{6cc}\u{6a9}\u{62f}!";
        let safe = safe_prefixed_wire_id("hitl", hostile);
        assert!(safe.starts_with("hitl:"));
        assert!(kian_contracts::Id::new(safe).is_some());
        // a 500-character overflow → ≤ 128 with a deterministic 16-hex suffix
        let huge = "x".repeat(500);
        let salvaged = safe_wire_id(huge.clone(), "audit");
        assert!(salvaged.get().chars().count() <= 128);
        assert!(salvaged.get().chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '.' | '-')));
        assert_eq!(safe_wire_id(huge, "audit"), salvaged, "salvage must be deterministic");
        // an empty string → valid static fallback, no panic
        assert_eq!(safe_wire_id(String::new(), "deposit").get(), "id:deposit");
    }
}
