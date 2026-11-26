//! Phase 8 dual-run routing brain — the rollback bridge is always on top (ADR-009 §12).
//!
//! Three transition steps:
//! * **Shadow** — 100% of traffic to TS; Rust runs the same inputs in shadow mode and
//!   only records them (the Rust response never reaches the client).
//! * **Percentage** — `RUST_TRAFFIC_PCT` = 1% → 10% → 50% → 100%; each step needs 48 hours
//!   of stability + parity + p95. Allocation is **deterministic** (stable input hash) so
//!   a retry with the same idempotency key always lands on the same engine.
//! * **Stabilization** — 2 weeks at 100% green → tag `rust-core-1.0.0` → decommission Node.
//!
//! Human shield: money paths ≥ 50,000 Đ (HITL — Article 7) go only to TS until 3 days of full
//! stability — "the last layer switched over is the first thing
//! audited".
//!
//! Rollback: `RUST_TRAFFIC_PCT=0` → everything goes to TS immediately — because both engines work
//! on the **same** SQLite with the same invariants, no data rollback is needed.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Human shield threshold — charter Article 7 (HITL).
pub const HITL_SHIELD_DIN: i64 = 50_000;

/// Stability days required to lift the HITL shield.
pub const SHIELD_STABILITY_DAYS: u32 = 3;

/// Routing decision for one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Go to Rust (percentage or shadow measurement)
    ToRust,
    /// Go to TS (the real responder)
    ToTs,
    /// Shadow mode: response from TS, shadow execution on Rust + parity comparison
    Shadow,
}

/// Full dual-run configuration — read from the environment (`DualRunConfig::from_env`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualRunConfig {
    /// Rust traffic percentage (0 to 100) — the single switch variable
    pub rust_traffic_pct: u8,
    /// Shadow mode enabled? (independent of the percentage — in shadow, the percentage must be 0)
    pub shadow: bool,
    /// Recorded stability days — the HITL shield rides on them
    pub stability_days: u32,
    /// Shield threshold (default 50,000 Đ — Article 7)
    pub hitl_shield_din: i64,
}

impl Default for DualRunConfig {
    fn default() -> Self {
        Self {
            rust_traffic_pct: 0,
            shadow: true,
            stability_days: 0,
            hitl_shield_din: HITL_SHIELD_DIN,
        }
    }
}

impl DualRunConfig {
    /// Validation — percentage > 100 or shadow together with traffic is rejected.
    ///
    /// # Errors
    /// `pct > 100` or `shadow && pct > 0` — contradictory config.
    pub fn validate(&self) -> Result<(), String> {
        if self.rust_traffic_pct > 100 {
            return Err(format!("rust_traffic_pct {} > 100", self.rust_traffic_pct));
        }
        if self.shadow && self.rust_traffic_pct > 0 {
            return Err(
                "shadow mode must have rust_traffic_pct = 0 (responses never come from Rust)"
                    .into(),
            );
        }
        Ok(())
    }
}

/// Metadata of one request for the routing decision — the minimum that hashes stably.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestMeta<'a> {
    /// HTTP method
    pub method: &'a str,
    /// Full path
    pub path: &'a str,
    /// Raw body (for hashing; reading the amount is separate)
    pub body: &'a [u8],
}

/// Stable 64-bit input hash — FNV-1a over SHA-256 (two layers, stable across processes).
#[must_use]
pub fn stable_bucket(meta: &RequestMeta<'_>) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(meta.method.as_bytes());
    hasher.update(b"\x1f");
    hasher.update(meta.path.as_bytes());
    hasher.update(b"\x1f");
    hasher.update(meta.body);
    let digest = hasher.finalize();
    let mut out: u64 = 0;
    for b in &digest[0..8] {
        out = (out << 8) | u64::from(*b);
    }
    out
}

/// Is the path a transaction covered by the human shield (HITL path)?
///
/// Per ADR-009 §12: "transactions ≥ 50,000 Đ (HITL path) go to Rust only after 3 days
/// of full stability". The HITL paths are exactly these:
/// * `POST …/hitl*` — human-in-the-loop queue and decisions (its amount is the same escrow ≥ threshold)
/// * `POST …/escrows/{id}/release` and `…/refund` — escrow settlement/refund
///   (the only places money leaves the escrow; the amount lives in the shared database, not the body)
#[must_use]
pub fn hitl_shielded(meta: &RequestMeta<'_>) -> bool {
    if meta.method != "POST" {
        return false;
    }
    if meta.path.contains("/hitl") {
        return true;
    }
    meta.path.contains("/escrows")
        && (meta.path.ends_with("/release") || meta.path.ends_with("/refund"))
}

impl DualRunConfig {
    /// Routing decision for one request — the heart of the proxy.
    ///
    /// Rules in order:
    /// 1. HITL shield — transactions ≥ threshold go to TS until 3 days of stability
    /// 2. Shadow mode — everything to TS, Rust in shadow
    /// 3. Percentage — stable bucket `< pct` → Rust
    #[must_use]
    pub fn decide(&self, meta: &RequestMeta<'_>) -> Decision {
        if self.stability_days < SHIELD_STABILITY_DAYS && hitl_shielded(meta) {
            // shield — even at 100% Rust; lifting it is governed by stability_days, not the percentage
            return Decision::ToTs;
        }
        if self.shadow {
            return Decision::Shadow;
        }
        if stable_bucket(meta) % 100 < u64::from(self.rust_traffic_pct) {
            Decision::ToRust
        } else {
            Decision::ToTs
        }
    }
}

/// Step promotion gate — based on the parity report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVerdict {
    /// Green — allowed to promote to the next step
    Advance,
    /// Red — stay and roll back (pct → 0 on a financial parity-fail)
    HoldOrRollback,
}

/// Parity gate check — the report must be fully green (ADR: "every parity-fail =
/// rollback").
#[must_use]
pub fn parity_gate(green: usize, total: usize, has_financial_diff: bool) -> GateVerdict {
    if has_financial_diff || green < total {
        GateVerdict::HoldOrRollback
    } else {
        GateVerdict::Advance
    }
}

/// Transition step plan — for showing the current status in the proxy status.
#[must_use]
pub fn rollout_plan() -> [(u8, &'static str); 5] {
    [
        (0, "shadow — \u{67e}\u{627}\u{633}\u{62e} \u{641}\u{642}\u{637} \u{627}\u{632} TS\u{60c} Rust \u{62b}\u{628}\u{62a} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f}"),
        (1, "\u{6f1}\u{66a} — \u{622}\u{63a}\u{627}\u{632} \u{62a}\u{631}\u{627}\u{641}\u{6cc}\u{6a9} \u{648}\u{627}\u{642}\u{639}\u{6cc} (\u{6f4}\u{6f8} \u{633}\u{627}\u{639}\u{62a} \u{67e}\u{627}\u{6cc}\u{62f}\u{627}\u{631}\u{6cc})"),
        (10, "\u{6f1}\u{6f0}\u{66a} — \u{67e}\u{633} \u{627}\u{632} \u{633}\u{628}\u{632}\u{6cc} parity \u{648} p95 \u{67e}\u{644}\u{647}\u{654} \u{642}\u{628}\u{644}"),
        (50, "\u{6f5}\u{6f0}\u{66a} — \u{647}\u{645}\u{627}\u{646} \u{6af}\u{6cc}\u{62a}"),
        (100, "\u{6f1}\u{6f0}\u{6f0}\u{66a} — \u{633}\u{67e}\u{631} HITL \u{62a}\u{627} \u{6f3} \u{631}\u{648}\u{632} \u{67e}\u{627}\u{6cc}\u{62f}\u{627}\u{631}\u{6cc} \u{628}\u{627}\u{642}\u{6cc} \u{627}\u{633}\u{62a}"),
    ]
}

/// Dual-run status line — for `GET /_dualrun/status` and logging.
#[derive(Debug, Clone, Serialize)]
pub struct DualRunStatus {
    /// Current configuration
    pub config: DualRunConfig,
    /// Step plan
    pub plan: Vec<(u8, &'static str)>,
    /// Emergency rollback active?
    pub rollback: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    const GET: &str = "GET";

    fn meta<'a>(method: &'a str, path: &'a str, body: &'a [u8]) -> RequestMeta<'a> {
        RequestMeta { method, path, body }
    }

    #[test]
    fn bucket_is_deterministic_across_calls() {
        let m = meta("POST", "/api/v1/escrows", br#"{"amount":10}"#);
        let a = stable_bucket(&m);
        let b = stable_bucket(&m);
        assert_eq!(a, b);
        // a different body → a different bucket (equal probability 2⁻⁶⁴ — practically impossible)
        assert_ne!(stable_bucket(&meta("POST", "/api/v1/escrows", br#"{"amount":11}"#)), a);
    }

    #[test]
    fn pct_zero_sends_everything_to_ts() {
        let cfg = DualRunConfig { rust_traffic_pct: 0, shadow: false, ..DualRunConfig::default() };
        for i in 0..64 {
            let path = format!("/api/v1/r{i}");
            let m = meta(GET, &path, b"");
            assert_eq!(cfg.decide(&m), Decision::ToTs, "path {path}");
        }
    }

    #[test]
    fn pct_hundred_sends_everything_to_rust() {
        let cfg =
            DualRunConfig { rust_traffic_pct: 100, shadow: false, ..DualRunConfig::default() };
        for i in 0..64 {
            let path = format!("/api/v1/r{i}");
            let m = meta(GET, &path, b"");
            assert_eq!(cfg.decide(&m), Decision::ToRust, "path {path}");
        }
    }

    #[test]
    fn shadow_mode_flags_every_request() {
        let cfg = DualRunConfig::default(); // pct=0 + shadow
        let m = meta(GET, "/api/v1/health", b"");
        assert_eq!(cfg.decide(&m), Decision::Shadow);
    }

    #[test]
    fn hitl_shield_keeps_big_money_on_ts_until_three_days() {
        let body = br#"{"escrowId":"e1"}"#;
        let m = meta("POST", "/api/v1/hitl", body);
        // day zero — shield fully on
        let cfg = DualRunConfig {
            rust_traffic_pct: 100,
            shadow: false,
            stability_days: 0,
            ..DualRunConfig::default()
        };
        assert_eq!(cfg.decide(&m), Decision::ToTs);
        // day 2 — shield still on
        let cfg2 = DualRunConfig { stability_days: 2, ..cfg };
        assert_eq!(cfg2.decide(&m), Decision::ToTs);
        // day 3 — shield lifted → 100% Rust
        let cfg3 = DualRunConfig { stability_days: 3, ..cfg };
        assert_eq!(cfg3.decide(&m), Decision::ToRust);
    }

    #[test]
    fn release_route_is_shielded_until_three_days() {
        let cfg =
            DualRunConfig { rust_traffic_pct: 100, shadow: false, ..DualRunConfig::default() };
        let m = meta("POST", "/api/v1/escrows/e9/release", br#"{"approvedByAdmin":true}"#);
        // escrow settlement — HITL path; goes to TS until day 3 even at 100%
        assert_eq!(cfg.decide(&m), Decision::ToTs);
        let cfg3 = DualRunConfig { stability_days: 3, ..cfg };
        assert_eq!(cfg3.decide(&m), Decision::ToRust);
    }

    #[test]
    fn mint_is_not_shielded_but_refund_is() {
        let cfg =
            DualRunConfig { rust_traffic_pct: 100, shadow: false, ..DualRunConfig::default() };
        let m = meta("POST", "/api/v1/wallet/mint", br#"{"amount":50000}"#);
        // mint is not a shielded path (only hitl and release/refund are)
        assert_eq!(cfg.decide(&m), Decision::ToRust);
        let m2 = meta("POST", "/api/v1/escrows/e1/refund", br#"{}"#);
        assert_eq!(cfg.decide(&m2), Decision::ToTs);
    }

    #[test]
    fn parity_gate_holds_on_any_failure() {
        assert_eq!(parity_gate(42, 42, false), GateVerdict::Advance);
        assert_eq!(parity_gate(41, 42, false), GateVerdict::HoldOrRollback);
        assert_eq!(parity_gate(42, 42, true), GateVerdict::HoldOrRollback);
    }

    #[test]
    fn config_validation_rejects_shadow_with_traffic() {
        let ok = DualRunConfig::default();
        assert!(ok.validate().is_ok());
        let bad = DualRunConfig { rust_traffic_pct: 101, shadow: false, ..ok };
        assert!(bad.validate().is_err());
        let mixed = DualRunConfig { rust_traffic_pct: 10, shadow: true, ..ok };
        assert!(mixed.validate().is_err());
    }

    #[test]
    fn rollout_plan_has_five_steps_to_hundred() {
        let plan = rollout_plan();
        assert_eq!(plan.len(), 5);
        assert_eq!(plan[0].0, 0);
        assert_eq!(plan[4].0, 100);
    }
}
