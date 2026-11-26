//! The phase 8 parity replay test — the in-CI gate.
//!
//! All golden scenarios run on the Rust router (a fresh in-memory database per
//! scenario) and the response + database state are compared **byte-for-byte** with the
//! `tests/golden/*.json` goldens. Any byte drift means the contract changed —
//! either a bug or an intentional contract change, in which case `parity --record`
//! must be run knowingly and the diff reviewed.

// integration test — unwrap/expect are intentional
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dropagentx_tooling::parity;

/// The main gate — all scenarios green or the test fails.
#[tokio::test]
async fn every_golden_scenario_replays_byte_for_byte() {
    let report = parity::verify_all(|_| "sqlite::memory:".into()).await.unwrap();
    let (green, total) = report.summary();
    assert_eq!(green, total, "parity drift detected:\n{}", report.render());
    // ADR-009 phase 8 — covers the "~30 golden scenarios": the registry must be ≥30
    assert!(total >= 30, "golden scenario registry shrank: {total}");
}

/// Normalizer — volatile timestamp and uptime must collapse to fixed tokens.
#[test]
fn normalizer_flattens_volatile_fields() {
    let body = serde_json::json!({
        "createdAt": "2026-11-12T09:41:00.123Z",
        "uptime": 41.5,
        "nested": { "decidedAt": "2026-11-12T09:41:03+03:30", "amount": 10 }
    });
    let out = parity::normalize_value(&body);
    assert_eq!(out["createdAt"], "<TS>");
    assert_eq!(out["uptime"], "<UPTIME>");
    assert_eq!(out["nested"]["decidedAt"], "<TS>");
    assert_eq!(out["nested"]["amount"], 10);
    // normalizing again — idempotent
    assert_eq!(parity::normalize_value(&out), out);
}

/// The parity money gate — breaking 90/10 is a mismatch even under normalization.
#[tokio::test]
async fn release_scenario_db_state_is_locked_in_golden() {
    let scenarios = parity::scenarios();
    let release = scenarios
        .iter()
        .find(|s| s.name == "escrow_release_settles_90_10")
        .unwrap_or_else(|| panic!("release scenario missing"));
    let result = parity::run_scenario(release, "sqlite::memory:").await.unwrap();
    assert_eq!(result.db.user_wallet_sum, Some(199_000));
    assert_eq!(result.db.treasury, Some(1_000));
    assert_eq!(result.db.escrow_status.get("escrow:escrow:e3"), Some(&"RELEASED".to_string()));
}
