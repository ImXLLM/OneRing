//! `cas` — CAS optimistic locking and escrow idempotency keys (Wave 2 — Step 5).
//!
//! Port of `Snailclimb/JavaGuide` (`state-machine-and-workflow.md` +
//! `distributed-lock.md`) for the Kian escrow:
//! - [`CasEscrowTransition`] — a transition conditioned on `expected_version` and
//!   `expected_state` with an `idempotency_key`;
//! - [`CasEscrowRecord::apply`] — a concurrent collision on the same version has exactly
//!   **one winner** and the stale caller is rejected with [`CasTransitionOutcome::CasConflict`];
//!   replaying a duplicate key returns the cached [`CasTransitionOutcome::AlreadyApplied`]
//!   result and **no micro-KIAN double-debit** ever occurs;
//! - [`build_cas_update_sql`] — optimistic UPDATE builder for the `kian-db`
//!   layer (`version` condition in `WHERE` — a real optimistic lock on SQLite).
//!
//! This module is a pure domain core: zero IO, zero dependencies, zero hidden hours.
#![forbid(unsafe_code)]

use std::collections::HashMap;

/// Escrow state machine states (schema-aligned with `crates/domain/src/escrow.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CasEscrowState {
    /// Initial — before holding.
    Pending,
    /// Collateral held (the outstanding debit was recorded).
    HeldInEscrow,
    /// Released to the seller.
    Released,
    /// Refunded to the buyer.
    Refunded,
}

/// Optimistic collateral transition — all the guards `apply` needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CasEscrowTransition {
    /// The version the caller saw — if it does not match the current version, [`CasTransitionOutcome::CasConflict`].
    pub expected_version: u64,
    /// The state the caller saw — a mismatch means a competing transition happened.
    pub expected_state: CasEscrowState,
    /// Business stability key — replaying the same key returns the cached result, not a second effect.
    pub idempotency_key: String,
}

/// Outcome of one CAS transition on a [`CasEscrowRecord`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CasTransitionOutcome {
    /// Applied — with the fresh version and state.
    Applied {
        /// Version after the transition (`expected_version + 1`).
        new_version: u64,
        /// State after the transition.
        new_state: CasEscrowState,
    },
    /// Duplicate key — the cached result of the first transition, with no second effect (no double-debit).
    AlreadyApplied {
        /// Version cached from the first transition.
        cached_version: u64,
        /// State cached from the first transition.
        cached_state: CasEscrowState,
    },
    /// Stale version — a competing caller won; the caller must re-read and decide again.
    CasConflict {
        /// Current record version (the stale caller re-reads from here).
        current_version: u64,
    },
    /// Invalid state machine transition (e.g. `Pending → Released` without holding).
    InvalidTransition {
        /// Current state.
        from: CasEscrowState,
        /// Requested state.
        to: CasEscrowState,
    },
}

/// In-memory collateral record with optimistic locking — the deterministic mirror of a SQLite row
/// (`Escrow`): `version` is the single source of truth for concurrency.
#[derive(Debug, Clone)]
pub struct CasEscrowRecord {
    state: CasEscrowState,
    version: u64,
    /// Key → Applied result (for idempotent replay).
    applied: HashMap<String, (u64, CasEscrowState)>,
    /// Total debits actually applied to the ledger (micro-KIAN) — the test proof of
    /// "no double-debit": replay never moves this number.
    total_debited_micro_kian: i64,
}

impl CasEscrowRecord {
    /// A fresh `Pending` record with version 0.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: CasEscrowState::Pending,
            version: 0,
            applied: HashMap::new(),
            total_debited_micro_kian: 0,
        }
    }

    /// Current state.
    #[must_use]
    pub fn state(&self) -> CasEscrowState {
        self.state
    }

    /// Current version (CAS counter).
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Total debits actually applied — for the anti-double-debit proof.
    #[must_use]
    pub fn total_debited_micro_kian(&self) -> i64 {
        self.total_debited_micro_kian
    }

    /// Apply a transition: **one winner** on concurrent collision; key replay = cache;
    /// invalid state machine transitions are rejected.
    ///
    /// An [`CasTransitionOutcome::AlreadyApplied`] result leaves no trace on the version,
    /// state, or debit — exactly "safe network replay" behavior.
    pub fn apply(
        &mut self,
        transition: &CasEscrowTransition,
        next_state: CasEscrowState,
        debit_micro_kian: i64,
    ) -> CasTransitionOutcome {
        if let Some(&(cached_version, cached_state)) = self.applied.get(&transition.idempotency_key) {
            return CasTransitionOutcome::AlreadyApplied {
                cached_version,
                cached_state,
            };
        }
        if self.version != transition.expected_version || self.state != transition.expected_state {
            return CasTransitionOutcome::CasConflict { current_version: self.version };
        }
        let allowed = matches!(
            (self.state, next_state),
            (CasEscrowState::Pending, CasEscrowState::HeldInEscrow)
                | (
                    CasEscrowState::HeldInEscrow,
                    CasEscrowState::Released | CasEscrowState::Refunded
                )
        );
        if !allowed {
            return CasTransitionOutcome::InvalidTransition { from: self.state, to: next_state };
        }
        self.state = next_state;
        self.version += 1;
        self.total_debited_micro_kian += debit_micro_kian;
        let outcome = CasTransitionOutcome::Applied {
            new_version: self.version,
            new_state: self.state,
        };
        self.applied
            .insert(transition.idempotency_key.clone(), (self.version, self.state));
        outcome
    }
}

impl Default for CasEscrowRecord {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimistic UPDATE builder for the `kian-db` layer — the version condition in `WHERE` means
/// two concurrent writers with the same version: only one gets `rows_affected == 1`
/// and the other gets zero (= [`CasTransitionOutcome::CasConflict`] at the SQL level).
#[must_use]
pub fn build_cas_update_sql(table: &str, key_column: &str) -> String {
    format!(
        "UPDATE \"{table}\" SET \"state\" = :new_state, \"version\" = \"version\" + 1 \
         WHERE \"{key_column}\" = :key AND \"version\" = :expected_version \
         AND \"state\" = :expected_state"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Concurrent collision on the same version — exactly one winner, the stale caller gets `CasConflict`.
    #[test]
    fn concurrent_same_version_transitions_allow_exactly_one_winner() {
        let mut record = CasEscrowRecord::new();
        let winner = CasEscrowTransition {
            expected_version: 0,
            expected_state: CasEscrowState::Pending,
            idempotency_key: "hold-tx-1".to_owned(),
        };
        let loser = CasEscrowTransition {
            expected_version: 0,
            expected_state: CasEscrowState::Pending,
            idempotency_key: "hold-tx-2".to_owned(),
        };
        let first = record.apply(&winner, CasEscrowState::HeldInEscrow, 1_000_000);
        assert_eq!(
            first,
            CasTransitionOutcome::Applied {
                new_version: 1,
                new_state: CasEscrowState::HeldInEscrow
            }
        );
        // the stale caller still tries with version 0 — deterministically rejected
        assert_eq!(
            record.apply(&loser, CasEscrowState::HeldInEscrow, 1_000_000),
            CasTransitionOutcome::CasConflict { current_version: 1 }
        );
        assert_eq!(record.total_debited_micro_kian(), 1_000_000, "\u{641}\u{642}\u{637} \u{628}\u{631}\u{646}\u{62f}\u{647} \u{628}\u{62f}\u{647}\u{6cc} \u{632}\u{62f}");
    }

    /// Replaying a duplicate key — the cached result, with no micro-KIAN double-debit.
    #[test]
    fn idempotency_key_replay_returns_cached_result_without_double_debit() {
        let mut record = CasEscrowRecord::new();
        let tx = CasEscrowTransition {
            expected_version: 0,
            expected_state: CasEscrowState::Pending,
            idempotency_key: "hold-escrow-42".to_owned(),
        };
        let first = record.apply(&tx, CasEscrowState::HeldInEscrow, 2_500_000);
        assert!(matches!(first, CasTransitionOutcome::Applied { .. }));
        // replaying exactly the same key — even with a stale version
        let replay = record.apply(&tx, CasEscrowState::HeldInEscrow, 2_500_000);
        assert_eq!(
            replay,
            CasTransitionOutcome::AlreadyApplied {
                cached_version: 1,
                cached_state: CasEscrowState::HeldInEscrow
            }
        );
        assert_eq!(record.total_debited_micro_kian(), 2_500_000, "\u{647}\u{6cc}\u{686} \u{62f}\u{628}\u{644}-\u{628}\u{62f}\u{647}\u{6cc} \u{646}\u{634}\u{62f}");
        assert_eq!(record.version(), 1, "\u{646}\u{633}\u{62e}\u{647} \u{647}\u{645} \u{644}\u{63a}\u{632}\u{6cc}\u{62f}\u{647} \u{646}\u{634}\u{62f}");
        // replaying with a wrong version still returns the cache (idempotency precedes CAS)
        let stale_replay = record.apply(
            &CasEscrowTransition {
                expected_version: 0,
                expected_state: CasEscrowState::Pending,
                idempotency_key: "hold-escrow-42".to_owned(),
            },
            CasEscrowState::HeldInEscrow,
            2_500_000,
        );
        assert_eq!(
            stale_replay,
            CasTransitionOutcome::AlreadyApplied {
                cached_version: 1,
                cached_state: CasEscrowState::HeldInEscrow
            }
        );
        assert_eq!(record.total_debited_micro_kian(), 2_500_000);
    }

    /// Invalid state machine transitions and the full escrow lifecycle.
    #[test]
    fn invalid_transitions_rejected_and_full_lifecycle_works() {
        let mut record = CasEscrowRecord::new();
        // Pending → Released without holding — forbidden
        assert_eq!(
            record.apply(
                &CasEscrowTransition {
                    expected_version: 0,
                    expected_state: CasEscrowState::Pending,
                    idempotency_key: "skip-hold".to_owned(),
                },
                CasEscrowState::Released,
                0,
            ),
            CasTransitionOutcome::InvalidTransition {
                from: CasEscrowState::Pending,
                to: CasEscrowState::Released
            }
        );
        // full lifecycle: Pending → HeldInEscrow → Released
        let hold = record.apply(
            &CasEscrowTransition {
                expected_version: 0,
                expected_state: CasEscrowState::Pending,
                idempotency_key: "life-hold".to_owned(),
            },
            CasEscrowState::HeldInEscrow,
            700_000,
        );
        assert!(matches!(hold, CasTransitionOutcome::Applied { new_version: 1, .. }));
        let release = record.apply(
            &CasEscrowTransition {
                expected_version: 1,
                expected_state: CasEscrowState::HeldInEscrow,
                idempotency_key: "life-release".to_owned(),
            },
            CasEscrowState::Released,
            0,
        );
        assert!(matches!(release, CasTransitionOutcome::Applied { new_version: 2, .. }));
        assert_eq!(record.state(), CasEscrowState::Released);
        assert_eq!(record.version(), 2);
    }

    /// Optimistic SQL builder — version and state conditions in `WHERE`.
    #[test]
    fn cas_update_sql_carries_version_and_state_guard() {
        let sql = build_cas_update_sql("Escrow", "id");
        assert!(sql.contains("UPDATE \"Escrow\""));
        assert!(sql.contains("\"version\" = \"version\" + 1"));
        assert!(sql.contains("WHERE \"id\" = :key"));
        assert!(sql.contains("\"version\" = :expected_version"));
        assert!(sql.contains("\"state\" = :expected_state"));
    }
}
