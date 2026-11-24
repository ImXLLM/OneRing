//! `cas` — قفل خوش‌بینانهٔ CAS و کلید idempotency امانت (Wave 2 — Step 5).
//!
//! پورت `Snailclimb/JavaGuide` (`state-machine-and-workflow.md` +
//! `distributed-lock.md`) برای اسکروی کیان:
//! - [`CasEscrowTransition`] — گذارِ مشروط به `expected_version` و
//!   `expected_state` با کلید `idempotency_key`؛
//! - [`CasEscrowRecord::apply`] — برخوردِ هم‌زمان با نسخهٔ یکسان دقیقاً
//!   **یک برنده** دارد و کالرِ کهنه با [`CasTransitionOutcome::CasConflict`]
//!   رد می‌شود؛ بازپخشِ کلید تکراری نتیجهٔ کش‌شدهٔ [`CasTransitionOutcome::AlreadyApplied`]
//!   برمی‌گرداند و **هیچ دبل-بدهی micro-KIAN** رخ نمی‌دهد؛
//! - [`build_cas_update_sql`] — سازندهٔ UPDATE خوش‌بینانه برای لایهٔ
//!   `kian-db` (شرط `version` در `WHERE` — قفل خوش‌بینانهٔ واقعی روی SQLite).
//!
//! این ماژول هستهٔ خالص دامنه است: صفر IO، صفر وابستگی، صفر ساعت پنهان.
#![forbid(unsafe_code)]

use std::collections::HashMap;

/// وضعیت‌های ماشین‌حالت اسکرو (هم‌اسکیما با `crates/domain/src/escrow.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CasEscrowState {
    /// اولیه — پیش از نگه‌داشت.
    Pending,
    /// امانت نگه‌داشته شد (بدهی معوق ثبت شد).
    HeldInEscrow,
    /// آزاد شد به فروشنده.
    Released,
    /// بازگشت به خریدار.
    Refunded,
}

/// گذار خوش‌بینانهٔ امانت — کل گُردهای لازم برای `apply`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CasEscrowTransition {
    /// نسخه‌ای که کالر دیده — اگر با نسخهٔ فعلی یکی نبود، [`CasTransitionOutcome::CasConflict`].
    pub expected_version: u64,
    /// وضعیتی که کالر دیده — ناهمخوانی یعنی گذار رقیب رخ داده.
    pub expected_state: CasEscrowState,
    /// کلید پایداری کسب‌وکار — بازپخش همان کلید نتیجهٔ کش‌شده می‌دهد، نه اثر دوم.
    pub idempotency_key: String,
}

/// نتیجهٔ یک گذار CAS روی [`CasEscrowRecord`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CasTransitionOutcome {
    /// اعمال شد — نسخه و وضعیت تازه.
    Applied {
        /// نسخه پس از گذار (`expected_version + 1`).
        new_version: u64,
        /// وضعیت پس از گذار.
        new_state: CasEscrowState,
    },
    /// کلید تکراری — نتیجهٔ کش‌شدهٔ گذار نخست، بدون هیچ اثر دوم (بدون دبل-بدهی).
    AlreadyApplied {
        /// نسخهٔ کش‌شدهٔ گذار نخست.
        cached_version: u64,
        /// وضعیتِ کش‌شدهٔ گذار نخست.
        cached_state: CasEscrowState,
    },
    /// نسخهٔ کهنه — کالرِ رقیب برنده شده؛ کالر باید دوباره بخواند و تصمیم بگیرد.
    CasConflict {
        /// نسخهٔ فعلی رکورد (کالر کهنه از این دوباره می‌خواند).
        current_version: u64,
    },
    /// گذار نامعتبر ماشین‌حالت (مثل `Pending → Released` بدون نگه‌داشت).
    InvalidTransition {
        /// وضعیت فعلی.
        from: CasEscrowState,
        /// وضعیت خواسته‌شده.
        to: CasEscrowState,
    },
}

/// رکورد امانت با قفل خوش‌بینانه در حافظه — بازگوی قطعیِ سطر SQLite
/// (`Escrow`): `version` تنها منبع حقیقتِ هم‌زمانی است.
#[derive(Debug, Clone)]
pub struct CasEscrowRecord {
    state: CasEscrowState,
    version: u64,
    /// کلید → نتیجهٔ Applied (برای بازپخش idempotent).
    applied: HashMap<String, (u64, CasEscrowState)>,
    /// مجموع بدهی‌های واقعاً اعمال‌شده به دفتر (micro-KIAN) — اثباتِ
    /// «بدون دبل-بدهی» در تست: بازپخش این عدد را نمی‌لغزاند.
    total_debited_micro_kian: i64,
}

impl CasEscrowRecord {
    /// رکورد تازهٔ `Pending` با نسخهٔ ۰.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: CasEscrowState::Pending,
            version: 0,
            applied: HashMap::new(),
            total_debited_micro_kian: 0,
        }
    }

    /// وضعیت فعلی.
    #[must_use]
    pub fn state(&self) -> CasEscrowState {
        self.state
    }

    /// نسخهٔ فعلی (شمارندهٔ CAS).
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// مجموع بدهی‌های واقعاً اعمال‌شده — برای اثبات ضد-دبل-بدهی.
    #[must_use]
    pub fn total_debited_micro_kian(&self) -> i64 {
        self.total_debited_micro_kian
    }

    /// اعمال گذار: **یک برنده** در برخوردِ هم‌زمان؛ بازپخش کلید = کش؛
    /// گذار نامعتبر ماشین‌حالت رد می‌شود.
    ///
    /// نتیجهٔ [`CasTransitionOutcome::AlreadyApplied`] هیچ اثری روی نسخه،
    /// وضعیت یا بدهی نمی‌گذارد — دقیقاً رفتار «بازپخش امن شبکه».
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

/// سازندهٔ UPDATE خوش‌بینانه برای لایهٔ `kian-db` — شرط نسخه در `WHERE` یعنی
/// دو نویسندهٔ هم‌زمان با نسخهٔ یکسان، فقط یکی `rows_affected == 1` می‌گیرد
/// و دیگری صفر (= [`CasTransitionOutcome::CasConflict`] در سطح SQL).
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

    /// برخورد هم‌زمان با نسخهٔ یکسان — دقیقاً یک برنده، کالرِ کهنه `CasConflict`.
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
        // کالر کهنه هنوز با نسخهٔ ۰ می‌کوشد — ردِ قطعی
        assert_eq!(
            record.apply(&loser, CasEscrowState::HeldInEscrow, 1_000_000),
            CasTransitionOutcome::CasConflict { current_version: 1 }
        );
        assert_eq!(record.total_debited_micro_kian(), 1_000_000, "فقط برنده بدهی زد");
    }

    /// بازپخش کلید تکراری — نتیجهٔ کش، بدون دبل-بدهی micro-KIAN.
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
        // بازپخش دقیقاً همان کلید — حتی با نسخهٔ کهنه
        let replay = record.apply(&tx, CasEscrowState::HeldInEscrow, 2_500_000);
        assert_eq!(
            replay,
            CasTransitionOutcome::AlreadyApplied {
                cached_version: 1,
                cached_state: CasEscrowState::HeldInEscrow
            }
        );
        assert_eq!(record.total_debited_micro_kian(), 2_500_000, "هیچ دبل-بدهی نشد");
        assert_eq!(record.version(), 1, "نسخه هم لغزیده نشد");
        // بازپخش با نسخهٔ غلط هم باز هم کش برمی‌گرداند (idempotency مقدم بر CAS)
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

    /// گذار نامعتبر ماشین‌حالت و چرخهٔ کامل عمر اسکرو.
    #[test]
    fn invalid_transitions_rejected_and_full_lifecycle_works() {
        let mut record = CasEscrowRecord::new();
        // Pending → Released بدون نگه‌داشت — ممنوع
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
        // عمر کامل: Pending → HeldInEscrow → Released
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

    /// سازندهٔ SQL خوش‌بینانه — شرط نسخه و وضعیت در `WHERE`.
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
