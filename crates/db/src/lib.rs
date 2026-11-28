//! `kian-db` — the KIAN persistence layer (the M2 money kernel).
//!
//! The bridge between the pure domain [`kian_domain`] and real v3 SQLite — with the "same"
//! Prisma schema (versioned migration `migrations/0001_init.sql`) so dual-run can sit
//! on one file without conversion:
//!
//! - [`store`] — `V3Store`: load → pure domain → insert, all inside
//!   a `sqlx::Transaction`; balances always derived from `Posting` (ADR-003)
//! - [`outbox`] — transactional outbox for `DomainEvent`s + audit
//! - [`wire`] — the JSON wire shape of events (identical to `JSON.stringify` in TS)
//! - [`genesis`] — the pure genesis plan + the ADR-002 explicit rounding table
//! - [`legacy`] — read-only reader of the legacy database (raw SQL)
//! - [`migrate`] — port of `migrate-legacy.ts`: user migration → `user:tg:<id>`,
//!   one-shot genesis with the deterministic key `legacy:genesis:v1`
//! - [`proof`] — external audit with raw SQL — Σ wallets = 187,672 Đ · Σ rows = 0
//! - [`queries`] — HITL queue, dashboard products, audit and backup (M4/M5 sit on this)
//! - [`users`] — M3 identity: Telegram users, bearer sessions, TON deposits
//! - [`brain`] — the M4 brain: Hermes skills/runs, 7-layer memory, fleet, A2A, admin
//! - [`nation`] — M5 governance: HITL parliament bills and votes
//! - [`wallet_chains`] — M5 wallet: four-chain addresses and USDT deposits
//!
//! Structural principle: this crate depends on `kian-domain`, but the domain
//! knows nothing from here.
#![forbid(unsafe_code)]

pub mod brain;
pub mod error;
pub mod genesis;
pub mod legacy;
pub mod migrate;
pub mod monopoly;
pub mod nation;
pub mod outbox;
pub mod quant;
pub mod proof;
pub mod queries;
pub mod sqlsafe;
pub mod store;
pub mod studio;
pub mod supremacy;
pub mod users;
pub mod wallet_chains;
pub mod wave10;
pub mod wave9;
pub mod wire;

#[cfg(test)]
mod integration_tests;

/// Crate id — for smoke tests and logs.
pub const NAME: &str = "kian-db";

/// Crate version — read from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------- main re-exports (ergonomic API) ----------
pub use error::{DbError, DbResult};
pub use genesis::{
    plan_genesis, GenesisCitizen, GenesisPlan, LegacyProductRow, LegacyUserRow, EXPECTED_SUPPLY,
    GENESIS_KEY, GENESIS_TX_ID,
};
pub use legacy::{read_legacy, LegacySnapshot};
pub use migrate::{migrate_legacy, reconcile, MigrateReport, ReconcileReport};
pub use proof::{genesis_proof, ProofReport};
pub use store::{
    now_iso, AccountBalance, ApplyOptions, ApplyResult, CommandOutcome, EconomySummary, OutboxRow,
    RecentPosting, V3Store,
};
pub use wire::{event_kind, event_payload};

// HITL queue, dashboard product, audit and backup queries + the Trinity 5-star vote
pub use queries::{
    deliver_order, AiUsageRow, AuditRowRead, BackupInfo, HitlDecision, HitlRow, ProductFilter,
    ProductReviewRow, ProductRow,
};

// M3 identity — Telegram citizens, sessions, TON deposits
pub use users::{citizen_account_id, TonDepositRow, UserRow};

// M4 brain + HTML product builder (Phase 4)
pub use brain::{
    A2aRow, AgentRow, BuilderMessageRow, BuilderSessionPatch, BuilderSessionRow, HermesRunRow,
    MemoryNoteRow, SkillRow, SkillSeed,
};

// M5 governance — the HITL parliament
pub use nation::{BillRow, VoteRow};

// Mobile-perception studio (Wave 5) — 9-modality creations
pub use studio::StudioCreationRow;
pub use supremacy::{AgentCheckpointRow, SweSandboxAuditRow, WorkflowDagRunRow};

// Wave 9 — lab log, local registration and the HUD render run
pub use wave9::{LocalePatchRegistryRow, NeuralHudRenderJobRow, UserLabTelemetryLogRow};

// Founder war room (Wave 10) — day-zero steps + boss directives
pub use wave10::{
    ChairmanDirectiveLogRow, FounderWarRoomTickRow, WarRoomStateRow, WarRoomTickEventRow,
};

// Causal quant (Wave 6) — Vibe-Trading strategies
pub use quant::QuantStrategyRow;

// Sovereign treasury (Wave 7) — day-one atomic splits on Base
pub use monopoly::SovereignTreasurySplitRow;

// M5 wallet + crypto withdrawal requests (Phase 6)
pub use wallet_chains::{UsdtDepositRow, WalletAddressRow, WithdrawalRequestRow};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(NAME, "kian-db");
        assert_ne!(VERSION, "", "version string present");
        // genesis contract constants — same as legacy (the initial liquidity value is fixed)
        assert_eq!(GENESIS_KEY, "legacy:genesis:v1");
        assert_eq!(GENESIS_TX_ID, "tx_legacy_genesis");
        assert_eq!(EXPECTED_SUPPLY, 187_672);
    }
}
