//! `dropagentx-tooling` — tooling — legacy migration, genesis_proof, dual_run,
//! audit_ledger, parity, loadtest (phases 2/8).
//!
//! * [`parity`] — phase 8 parity suite: byte-for-byte golden scenarios + DB state
//! * [`dual_run`] — dual-run routing brain: traffic percentage, HITL shield, parity gate
//! * [`proxy`] — HTTP routing proxy with `/_dualrun` control endpoints
#![forbid(unsafe_code)]

pub mod dual_run;
pub mod parity;
pub mod proxy;

/// Crate identifier — for smoke tests and logs.
pub const NAME: &str = "dropagentx-tooling";

/// Crate version — read from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Phase 0 health probe — a sign of life for the wiring (stands in for a hello-handler).
#[must_use]
pub fn hello() -> &'static str {
    "dropagentx_tooling · Agent-One is alive"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(NAME, "dropagentx-tooling");
        assert!(!VERSION.is_empty());
        assert!(hello().ends_with("alive"));
    }
}
