//! `kian-db` — لایهٔ ماندگاری KIAN (کرنل پول M2).
//!
//! پل بین دامنهٔ خالص [`kian_domain`] و SQLite واقعی v3 — با «همان» اسکیمای
//! Prisma (مهاجرت نسخه‌دار `migrations/0001_init.sql`) تا dual-run بدون تبدیل
//! روی یک فایل بنشیند:
//!
//! - [`store`] — `V3Store`: بارگذاری → دامنهٔ خالص → درج، همه در
//!   `sqlx::Transaction`؛ ترازها همیشه مشتق از `Posting` (ADR-003)
//! - [`outbox`] — transactional outbox برای `DomainEvent`ها + audit
//! - [`wire`] — شکل سیمی JSON رویدادها (یکسان با `JSON.stringify` در TS)
//! - [`genesis`] — برنامهٔ خالص genesis + جدول گردکردن صریح ADR-002
//! - [`legacy`] — خوانندهٔ فقط‌خواندنی دیتابیس لگاسی (SQL خام)
//! - [`migrate`] — پورت `migrate-legacy.ts`: مهاجرت کاربران → `user:tg:<id>`،
//!   genesis یک‌بارمصرف با کلید قطعی `legacy:genesis:v1`
//! - [`proof`] — ممیزی خارجی با SQL خام — Σ کیف‌ها = 187,672 Đ · Σ ردیف‌ها = 0
//! - [`queries`] — صف HITL، محصولات داشبورد، audit و بکاپ (M4/M5 روی همین می‌نشینند)
//! - [`users`] — هویت M3: کاربران تلگرام، نشست‌های bearer، واریزهای TON
//! - [`brain`] — مغز M4: مهارت‌ها/ران‌های هرمس، حافظهٔ ۷لایه، ناوگان، A2A، ادمین
//! - [`nation`] — حاکمیت M5: لایحه‌ها و رأی‌های پارلمان HITL
//! - [`wallet_chains`] — کیف پول M5: آدرس‌های چهارزنجیره و واریزهای USDT
//!
//! اصل ساختاری: این کرِیت به `kian-domain` بسته می‌شود، اما دامنه
//! هیچ‌چیز از اینجا نمی‌داند.
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
pub mod users;
pub mod wallet_chains;
pub mod wire;

#[cfg(test)]
mod integration_tests;

/// شناسه کرِیت — برای smoke test و لاگ‌ها.
pub const NAME: &str = "kian-db";

/// نسخه کرِیت — از `Cargo.toml` خوانده می‌شود.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------- re-exports اصلی (API آرگونومیک) ----------
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

// کوئری‌های صف HITL، محصولات داشبورد، audit و بکاپ + رأی ۵-ستارهٔ ترینیتی
pub use queries::{
    deliver_order, AiUsageRow, AuditRowRead, BackupInfo, HitlDecision, HitlRow, ProductFilter,
    ProductReviewRow, ProductRow,
};

// هویت M3 — شهروندان تلگرام، نشست‌ها، واریز TON
pub use users::{citizen_account_id, TonDepositRow, UserRow};

// مغز M4 + سازندهٔ محصول HTML (فاز ۴)
pub use brain::{
    A2aRow, AgentRow, BuilderMessageRow, BuilderSessionPatch, BuilderSessionRow, HermesRunRow,
    MemoryNoteRow, SkillRow, SkillSeed,
};

// حاکمیت M5 — پارلمان HITL
pub use nation::{BillRow, VoteRow};

// استودیوی ادراک‌سیار (Wave 5) — آفرینش‌های ۹-مدالیتی
pub use studio::StudioCreationRow;

// کوانتِ علّی (Wave 6) — استراتژی‌های Vibe-Trading
pub use quant::QuantStrategyRow;

// خزانهٔ حاکم (Wave 7) — تقسیم‌های اتمیِ روزِ اول روی Base
pub use monopoly::SovereignTreasurySplitRow;

// کیف پول M5 + درخواست‌های برداشت کریپتو (فاز ۶)
pub use wallet_chains::{UsdtDepositRow, WalletAddressRow, WithdrawalRequestRow};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(NAME, "kian-db");
        assert_ne!(VERSION, "", "version string present");
        // ثابت‌های قرارداد genesis — همان legacy (ارزش نقدشوندگی آغازین ثابت است)
        assert_eq!(GENESIS_KEY, "legacy:genesis:v1");
        assert_eq!(GENESIS_TX_ID, "tx_legacy_genesis");
        assert_eq!(EXPECTED_SUPPLY, 187_672);
    }
}
