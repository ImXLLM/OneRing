//! `kian-nation` — دولت HITL — پارلمان، آستانه ۵۰,۰۰۰ Đ، اقتصاد Đ.
//!
//! پارلمان KIAN جایی است که تصمیم‌های پولیِ نیازمند رأی به شکل «لایحه» ثبت
//! می‌شوند: هر لایحه (Bill) یک کنش واقعی روی کرنل پول را توصیف می‌کند
//! (حل اختلاف اسکرو / جابه‌جایی خزانه)، شهروندان و ایجنت‌ها به آن رأی می‌دهند
//! و بستن لایحه قطعی و deterministic است — همان الگوی ماشین وضعیت اسکرو M2.
//!
//! گره با منشور (مادهٔ ۷): لایحه‌ای که مبلغش ≥ ۵۰,۰۰۰ Đ است، هنگام بستن
//! **حداقل یک رأی انسانیِ موافق** می‌خواهد؛ ایجنت‌ها هرگز نمی‌توانند تصمیم
//! پولیِ بزرگ را تنها بگیرند (HITL — Human-in-the-Loop). آستانه از
//! [`kian_domain::requires_hitl`] می‌آید — یک منبع حقیقت، نه عدد تکراری.
//!
//! اجرا هم صادقانه است: «اجرا» فقط از دامنهٔ خالص عبور می‌کند
//! (release/refund اسکرو با `approved_by_admin: true` یا posting خزانه روی
//! دفتر دوطرفه) — هیچ مسیری پول را بی‌رأی جابه‌جا نمی‌کند.
//!
//! ماژول‌ها (فایل واحد — کرت کوچک و خالص):
//! - [`BillKind`] — دو نوع لایحه، هر دو با اثر واقعی روی دفتر
//! - [`Bill`] — ماشین وضعیت لایحه: OPEN → PASSED/REJECTED → EXECUTED
//! - [`NationPolicy`] — نصاب و آستانه (پیش‌فرض: نصاب ۳، آستانه منشور)
#![forbid(unsafe_code)]

use kian_domain::Din;

/// شناسه کرِیت — برای smoke test و لاگ‌ها.
pub const NAME: &str = "kian-nation";

/// نسخه کرِیت — از `Cargo.toml` خوانده می‌شود.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// سلام‌سنج — نشانه زنده‌بودن سیم‌کشی.
#[must_use]
pub fn hello() -> &'static str {
    "kian_nation · KIAN is alive"
}

// ── خطا ─────────────────────────────────────────────────────────────────────

/// خطای پارلمان — کدهای صریح؛ نگاشت HTTP در لایهٔ server.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NationError {
    /// عمل روی لایحه‌ای که OPEN نیست (بستن دوباره، رأی به بسته، اجرای ردشده).
    #[error("bill {0} is not OPEN")]
    BillNotOpen(String),
    /// هر رأی‌دهنده فقط یک رأی دارد.
    #[error("voter {0} already voted on bill {1}")]
    AlreadyVoted(String, String),
    /// بدنهٔ لایحه نامعتبر (عنوان خالی، مبلغ صفر برای جابه‌جایی خزانه، …).
    #[error("invalid bill: {0}")]
    InvalidBill(String),
    /// اجرا فقط از PASSED ممکن است.
    #[error("bill {0} is not PASSED — execution refused")]
    NotPassed(String),
}

/// نتیجهٔ استاندارد پارلمان.
pub type NationResult<T> = Result<T, NationError>;

// ── انواع دامنه ─────────────────────────────────────────────────────────────

/// سیاست پارلمان — نصاب و آستانه. تغییرناپذیر در عمر سرور (env).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NationPolicy {
    /// حداقل مجموع رأی‌ها برای بستن معتبر.
    pub quorum: i64,
    /// آستانه HITL به Đ — پیش‌فرض منشور ۵۰,۰۰۰؛ فقط برای تست override می‌شود.
    pub hitl_threshold_din: i64,
}

impl NationPolicy {
    /// پیش‌فرض مادهٔ ۷ — نصاب ۳، آستانه ۵۰,۰۰۰ Đ.
    #[must_use]
    pub const fn charter_default() -> Self {
        Self {
            quorum: 3,
            hitl_threshold_din: 50_000,
        }
    }

    /// مادهٔ ۷ — آیا این مبلغ رأی انسانی می‌خواهد؟ همان گیت منشور.
    #[must_use]
    pub fn requires_hitl(&self, amount_din: i64) -> bool {
        self.hitl_threshold_din > 0 && amount_din >= self.hitl_threshold_din
    }
}

/// نوع لایحه — هر نوع کنش واقعی و قابل‌اجرایی روی کرنل پول دارد.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillKind {
    /// حل اختلاف اسکرو — subject شناسهٔ اسکرو است؛ اجرا یعنی release/refund
    /// با `approved_by_admin: true` از مسیر دامنهٔ خالص M2.
    EscrowRelease,
    /// جابه‌جایی خزانه — subject حساب مقصد است؛ اجرا یعنی posting دوطرفهٔ
    /// خزانه → مقصد (مادهٔ ۶: شفافیت کامل در audit).
    TreasuryMove,
}

impl BillKind {
    /// برچسب audit/outbox.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EscrowRelease => "escrowRelease",
            Self::TreasuryMove => "treasuryMove",
        }
    }
}

/// نتیجهٔ پیشنهادی برای لایحهٔ حل اختلاف — آزادسازی به فروشنده یا بازگشت به خریدار.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscrowResolution {
    /// تسویهٔ ۹۰/۱۰ به نفع فروشنده (مادهٔ ۲).
    Release,
    /// بازگشت کامل به خریدار، بدون مالیات.
    Refund,
}

impl EscrowResolution {
    /// برچسب سیمی.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Refund => "refund",
        }
    }
}

/// وضعیت لایحه — گذارها یک‌طرفه: OPEN → PASSED/REJECTED؛ PASSED → EXECUTED.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillStatus {
    /// باز — رأی‌گیری ادامه دارد.
    Open,
    /// بسته‌شده با رأی موافق — قابل اجرا.
    Passed,
    /// بسته‌شده با رد — هرگز قابل اجرا نیست.
    Rejected,
    /// اجراشده روی دفتر — نهایی.
    Executed,
}

impl BillStatus {
    /// برچسب سیمی/دیتابیسی.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Passed => "PASSED",
            Self::Rejected => "REJECTED",
            Self::Executed => "EXECUTED",
        }
    }
}

/// یک رأی — رأی‌دهنده شهروند (نشست) یا ایجنت (A2A) است؛ `is_human` گیت مادهٔ ۷ را می‌بندد.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vote {
    /// شناسهٔ رأی‌دهنده (`citizen:…` یا `agent:…`)
    pub voter: String,
    /// انسان است؟ (نشست تلگرام = true؛ ایجنت = false)
    pub is_human: bool,
    /// موافق؟
    pub approve: bool,
    /// زمان رأی ISO
    pub at: String,
}

/// دلیل قطعیِ نتیجهٔ بستن — در ردیف دیتابیس می‌ماند (شفافیت مادهٔ ۶).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BillDecisionReason {
    /// اکثریت موافق + نصاب + (در صورت مادهٔ ۷) رأی انسانی.
    Passed,
    /// مجموع رأی‌ها کمتر از نصاب.
    QuorumNotMet,
    /// اکثریت مخالف.
    MajorityNo,
    /// مبلغ ≥ آستانه و هیچ رأی انسانیِ موافق نیست (مادهٔ ۷).
    HumanApprovalMissing,
}

impl BillDecisionReason {
    /// برچسب سیمی/دیتابیسی.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::QuorumNotMet => "quorum_not_met",
            Self::MajorityNo => "majority_no",
            Self::HumanApprovalMissing => "human_approval_missing",
        }
    }
}

/// نتیجهٔ قطعی بستن لایحه — شمارش‌ها برای شفافیت کامل نگه داشته می‌شوند.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillDecision {
    /// دلیل نهایی
    pub reason: BillDecisionReason,
    /// رأی موافق
    pub yes: i64,
    /// رأی مخالف
    pub no: i64,
    /// رأی انسانی موافق (گیت مادهٔ ۷)
    pub human_yes: i64,
}

impl BillDecision {
    /// آیا این تصمیم «گذشت» است؟
    #[must_use]
    pub fn is_passed(&self) -> bool {
        self.reason == BillDecisionReason::Passed
    }
}

/// لایحهٔ پارلمان — ثبت‌نشدنی روی دیتابیس؛ منطق گذار اینجا، ماندگاری در db.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bill {
    /// شناسه (`bill:{uuid}`) — کلید یکسان‌سازی اجرا هم هست
    pub id: String,
    /// نوع لایحه
    pub kind: BillKind,
    /// موضوع: شناسهٔ اسکرو یا حساب مقصد
    pub subject: String,
    /// مبلغ به Đ (۰ برای لایحهٔ بدون مبلغ)
    pub amount_din: i64,
    /// نتیجهٔ پیشنهادی (فقط `EscrowRelease`)
    pub resolution: Option<EscrowResolution>,
    /// عنوان (۱..=۲۰۰ کاراکتر)
    pub title: String,
    /// وضعیت فعلی
    pub status: BillStatus,
    /// تصمیم نهایی — پس از بستن
    pub decision: Option<BillDecision>,
    /// زمان باز شدن ISO
    pub opened_at: String,
    /// زمان بستن ISO
    pub closed_at: Option<String>,
    /// زمان اجرا ISO
    pub executed_at: Option<String>,
    /// رأی‌ها به ترتیب درج
    pub votes: Vec<Vote>,
}

impl Bill {
    /// باز کردن لایحه — اعتبارسنجی کامل در لبه (مثل zod؛ نه در دیتابیس).
    ///
    /// # Errors
    /// عنوان خالی/طولانی، موضوع خالی، مبلغ منفی، جابه‌جایی خزانه بدون مبلغ مثبت،
    /// حل اختلاف بدون نتیجهٔ پیشنهادی.
    pub fn open(
        id: &str,
        kind: BillKind,
        subject: &str,
        amount_din: i64,
        resolution: Option<EscrowResolution>,
        title: &str,
        opened_at: &str,
    ) -> NationResult<Self> {
        let title = title.trim();
        if title.is_empty() || title.chars().count() > 200 {
            return Err(NationError::InvalidBill(
                "title must be 1..=200 chars".into(),
            ));
        }
        let subject = subject.trim();
        if subject.is_empty() || subject.chars().count() > 128 {
            return Err(NationError::InvalidBill(
                "subject must be 1..=128 chars".into(),
            ));
        }
        if amount_din < 0 {
            return Err(NationError::InvalidBill(
                "amount must not be negative".into(),
            ));
        }
        match kind {
            BillKind::EscrowRelease => {
                if resolution.is_none() {
                    return Err(NationError::InvalidBill(
                        "escrowRelease requires a resolution (release|refund)".into(),
                    ));
                }
                if amount_din <= 0 {
                    return Err(NationError::InvalidBill(
                        "escrowRelease requires the disputed amount".into(),
                    ));
                }
            }
            BillKind::TreasuryMove => {
                if amount_din <= 0 {
                    return Err(NationError::InvalidBill(
                        "treasuryMove requires a positive amount".into(),
                    ));
                }
            }
        }
        Din::new(amount_din).map_err(|_| NationError::InvalidBill("amount out of range".into()))?;
        Ok(Self {
            id: id.to_owned(),
            kind,
            subject: subject.to_owned(),
            amount_din,
            resolution,
            title: title.to_owned(),
            status: BillStatus::Open,
            decision: None,
            opened_at: opened_at.to_owned(),
            closed_at: None,
            executed_at: None,
            votes: Vec::new(),
        })
    }

    /// ثبت رأی — فقط روی لایحهٔ باز؛ هر رأی‌دهنده یک‌بار.
    ///
    /// # Errors
    /// `BillNotOpen` / `AlreadyVoted`.
    pub fn cast_vote(
        &mut self,
        voter: &str,
        is_human: bool,
        approve: bool,
        at: &str,
    ) -> NationResult<()> {
        if self.status != BillStatus::Open {
            return Err(NationError::BillNotOpen(self.id.clone()));
        }
        let voter = voter.trim();
        if voter.is_empty() || voter.chars().count() > 128 {
            return Err(NationError::InvalidBill(
                "voter must be 1..=128 chars".into(),
            ));
        }
        if self.votes.iter().any(|v| v.voter == voter) {
            return Err(NationError::AlreadyVoted(voter.to_owned(), self.id.clone()));
        }
        self.votes.push(Vote {
            voter: voter.to_owned(),
            is_human,
            approve,
            at: at.to_owned(),
        });
        Ok(())
    }

    /// شمارش فعلی — (موافق، مخالف، موافق انسانی).
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // شمارش رأی‌ها — محدود و امن
    pub fn tally(&self) -> (i64, i64, i64) {
        let yes = self.votes.iter().filter(|v| v.approve).count() as i64;
        let no = self.votes.iter().filter(|v| !v.approve).count() as i64;
        let human_yes = self
            .votes
            .iter()
            .filter(|v| v.approve && v.is_human)
            .count() as i64;
        (yes, no, human_yes)
    }

    /// بستن لایحه — شمارش قطعی و deterministic (الگوی گیت‌های دامنهٔ M2):
    /// ۱. نصاب: مجموع رأی‌ها ≥ `policy.quorum`
    /// ۲. اکثریت: موافق > مخالف
    /// ۳. مادهٔ ۷: اگر مبلغ ≥ آستانه — حداقل یک رأی انسانیِ موافق
    /// شکست هر گیت = `REJECTED` با دلیل صریح (شفافیت مادهٔ ۶).
    ///
    /// # Errors
    /// `BillNotOpen` اگر قبلاً بسته شده باشد.
    ///
    /// # Panics
    /// هیچ‌گاه — `expect` پس از ست‌کردن همان‌جا همیشه موفق است.
    pub fn close(&mut self, policy: &NationPolicy, closed_at: &str) -> NationResult<&BillDecision> {
        if self.status != BillStatus::Open {
            return Err(NationError::BillNotOpen(self.id.clone()));
        }
        let (yes, no, human_yes) = self.tally();
        let reason = if self.requires_hitl() && human_yes == 0 {
            BillDecisionReason::HumanApprovalMissing
        } else if yes + no < policy.quorum {
            BillDecisionReason::QuorumNotMet
        } else if yes <= no {
            BillDecisionReason::MajorityNo
        } else {
            BillDecisionReason::Passed
        };
        let decision = BillDecision {
            reason,
            yes,
            no,
            human_yes,
        };
        self.status = if decision.is_passed() {
            BillStatus::Passed
        } else {
            BillStatus::Rejected
        };
        self.closed_at = Some(closed_at.to_owned());
        self.decision = Some(decision);
        Ok(self.decision.as_ref().expect("decision just set"))
    }

    /// گیت مادهٔ ۷ برای این لایحه — از سیاست منشور.
    #[must_use]
    pub fn requires_hitl(&self) -> bool {
        NationPolicy::charter_default().requires_hitl(self.amount_din)
    }

    /// علامت اجرا — فقط از PASSED؛ یک‌بار برای همیشه.
    ///
    /// # Errors
    /// `NotPassed` اگر وضعیت PASSED نباشد.
    pub fn mark_executed(&mut self, executed_at: &str) -> NationResult<()> {
        if self.status != BillStatus::Passed {
            return Err(NationError::NotPassed(self.id.clone()));
        }
        self.status = BillStatus::Executed;
        self.executed_at = Some(executed_at.to_owned());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: &str = "2026-11-21T00:00:00.000Z";

    fn open_bill(amount: i64, kind: BillKind) -> Bill {
        Bill::open(
            "bill:t1",
            kind,
            match kind {
                BillKind::EscrowRelease => "esc:1",
                BillKind::TreasuryMove => "citizen:100",
            },
            amount,
            match kind {
                BillKind::EscrowRelease => Some(EscrowResolution::Release),
                BillKind::TreasuryMove => None,
            },
            "حل اختلاف سفارش",
            NOW,
        )
        .unwrap()
    }

    fn vote(bill: &mut Bill, voter: &str, human: bool, approve: bool) {
        bill.cast_vote(voter, human, approve, NOW).unwrap();
    }

    #[test]
    fn hello_smoke() {
        assert_eq!(NAME, "kian-nation");
        assert!(hello().contains("alive"));
    }

    #[test]
    fn hitl_gate_opens_at_exactly_charter_threshold() {
        let p = NationPolicy::charter_default();
        assert!(!p.requires_hitl(49_999));
        assert!(p.requires_hitl(50_000));
        assert!(p.requires_hitl(50_001));
        // همان گیت منشور M2 — یک منبع حقیقت
        assert_eq!(
            p.requires_hitl(50_000),
            kian_domain::requires_hitl(Din::new(50_000).unwrap())
        );
    }

    #[test]
    fn open_validates_the_edges() {
        assert!(Bill::open("b", BillKind::TreasuryMove, "citizen:1", 0, None, "  ", NOW).is_err());
        assert!(Bill::open("b", BillKind::TreasuryMove, " ", 100, None, "t", NOW).is_err());
        assert!(Bill::open("b", BillKind::TreasuryMove, "citizen:1", -1, None, "t", NOW).is_err());
        assert!(
            Bill::open("b", BillKind::TreasuryMove, "citizen:1", 0, None, "t", NOW).is_err(),
            "جابه‌جایی خزانه بدون مبلغ مثبت ممنوع"
        );
        assert!(
            Bill::open("b", BillKind::EscrowRelease, "esc:1", 100, None, "t", NOW).is_err(),
            "حل اختلاف بدون نتیجهٔ پیشنهادی ممنوع"
        );
        assert!(
            Bill::open(
                "b",
                BillKind::EscrowRelease,
                "esc:1",
                0,
                Some(EscrowResolution::Refund),
                "t",
                NOW
            )
            .is_err()
        );
        let long_title = "عنوان".repeat(60); // ۲۴۰ کاراکتر
        assert!(Bill::open("b", BillKind::TreasuryMove, "c", 1, None, &long_title, NOW).is_err());
    }

    #[test]
    fn one_voter_one_vote() {
        let mut bill = open_bill(100, BillKind::TreasuryMove);
        vote(&mut bill, "citizen:1", true, true);
        assert_eq!(
            bill.cast_vote("citizen:1", true, false, NOW),
            Err(NationError::AlreadyVoted(
                "citizen:1".into(),
                "bill:t1".into()
            ))
        );
        assert_eq!(bill.tally(), (1, 0, 1));
    }

    #[test]
    fn voting_on_closed_bill_is_refused() {
        let mut bill = open_bill(100, BillKind::TreasuryMove);
        vote(&mut bill, "citizen:1", true, true);
        vote(&mut bill, "citizen:2", true, true);
        vote(&mut bill, "citizen:3", true, true);
        bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        assert_eq!(
            bill.cast_vote("citizen:4", true, true, NOW),
            Err(NationError::BillNotOpen("bill:t1".into()))
        );
        assert_eq!(bill.status, BillStatus::Passed);
    }

    #[test]
    fn quorum_not_met_rejects() {
        let mut bill = open_bill(100, BillKind::TreasuryMove);
        vote(&mut bill, "citizen:1", true, true);
        let d = bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        assert_eq!(d.reason, BillDecisionReason::QuorumNotMet);
        assert_eq!(bill.status, BillStatus::Rejected);
    }

    #[test]
    fn majority_no_rejects_even_with_quorum() {
        let mut bill = open_bill(100, BillKind::TreasuryMove);
        vote(&mut bill, "a", true, true);
        vote(&mut bill, "b", true, false);
        vote(&mut bill, "c", true, false);
        let d = bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        assert_eq!(d.reason, BillDecisionReason::MajorityNo);
        assert_eq!(bill.status, BillStatus::Rejected);
    }

    #[test]
    fn small_amounts_pass_without_human_votes() {
        let mut bill = open_bill(100, BillKind::TreasuryMove);
        vote(&mut bill, "agent:x", false, true);
        vote(&mut bill, "agent:y", false, true);
        vote(&mut bill, "agent:z", false, true);
        let d = bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        assert_eq!(d.reason, BillDecisionReason::Passed);
        assert_eq!(d.human_yes, 0);
        assert_eq!(bill.status, BillStatus::Passed);
    }

    #[test]
    fn hitl_amount_without_human_yes_is_rejected() {
        let mut bill = open_bill(50_000, BillKind::TreasuryMove);
        vote(&mut bill, "agent:x", false, true);
        vote(&mut bill, "agent:y", false, true);
        vote(&mut bill, "agent:z", false, true);
        let d = bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        assert_eq!(d.reason, BillDecisionReason::HumanApprovalMissing);
        assert_eq!(
            bill.status,
            BillStatus::Rejected,
            "ایجنت‌ها هرگز پول بزرگ را تنها تصویب نمی‌کنند"
        );
    }

    #[test]
    fn hitl_amount_passes_only_with_human_yes() {
        let mut bill = open_bill(60_000, BillKind::EscrowRelease);
        vote(&mut bill, "agent:x", false, true);
        vote(&mut bill, "agent:y", false, true);
        vote(&mut bill, "citizen:9", true, true);
        let d = bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        assert_eq!(d.reason, BillDecisionReason::Passed);
        assert_eq!(d.human_yes, 1);
        assert_eq!(bill.status, BillStatus::Passed);
    }

    #[test]
    fn human_no_vote_blocks_hitl_even_if_agents_say_yes() {
        let mut bill = open_bill(80_000, BillKind::TreasuryMove);
        vote(&mut bill, "citizen:1", true, false);
        vote(&mut bill, "agent:x", false, true);
        vote(&mut bill, "agent:y", false, true);
        let d = bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        assert_eq!(d.reason, BillDecisionReason::HumanApprovalMissing);
    }

    #[test]
    fn execution_only_from_passed_and_once() {
        let mut bill = open_bill(100, BillKind::TreasuryMove);
        assert_eq!(
            bill.mark_executed(NOW),
            Err(NationError::NotPassed("bill:t1".into())),
            "اجرای لایحهٔ باز ممنوع"
        );
        vote(&mut bill, "a", true, true);
        vote(&mut bill, "b", true, true);
        vote(&mut bill, "c", true, true);
        bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        bill.mark_executed(NOW).unwrap();
        assert_eq!(bill.status, BillStatus::Executed);
        assert!(bill.executed_at.is_some());
        assert_eq!(
            bill.mark_executed(NOW),
            Err(NationError::NotPassed("bill:t1".into()))
        );
    }

    #[test]
    fn rejected_bill_never_executes() {
        let mut bill = open_bill(100, BillKind::TreasuryMove);
        vote(&mut bill, "a", true, false);
        bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        assert_eq!(bill.status, BillStatus::Rejected);
        assert_eq!(
            bill.mark_executed(NOW),
            Err(NationError::NotPassed("bill:t1".into()))
        );
    }

    #[test]
    fn closing_twice_is_refused() {
        let mut bill = open_bill(100, BillKind::TreasuryMove);
        vote(&mut bill, "a", true, true);
        vote(&mut bill, "b", true, true);
        vote(&mut bill, "c", true, true);
        bill.close(&NationPolicy::charter_default(), NOW).unwrap();
        assert_eq!(
            bill.close(&NationPolicy::charter_default(), NOW),
            Err(NationError::BillNotOpen("bill:t1".into()))
        );
    }
}
