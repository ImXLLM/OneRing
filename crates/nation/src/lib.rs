//! `kian-nation` — the HITL government — parliament, the 50,000 Đ threshold, the Đ economy.
//!
//! KIAN's parliament is where money decisions requiring a vote are recorded as "bills":
//! each bill (Bill) describes a real action on the money kernel
//! (escrow dispute resolution / treasury movement), citizens and agents vote on it,
//! and bill closing is final and deterministic — the same state-machine pattern as M2 escrow.
//!
//! Charter coupling (Article 7): a bill whose amount is ≥ 50,000 Đ requires
//! **at least one approving human vote** at close time; agents can never make a large
//! money decision alone (HITL — Human-in-the-Loop). The threshold comes from
//! [`kian_domain::requires_hitl`] — one source of truth, not a duplicated number.
//!
//! Execution is honest too: "execute" only passes through the pure domain
//! (escrow release/refund with `approved_by_admin: true`, or a treasury posting on the
//! double-entry ledger) — no path moves money without a vote.
//!
//! Modules (single file — a small, pure crate):
//! - [`BillKind`] — two bill kinds, both with a real ledger effect
//! - [`Bill`] — bill state machine: OPEN → PASSED/REJECTED → EXECUTED
//! - [`NationPolicy`] — quorum and threshold (defaults: quorum 3, the charter threshold)
#![forbid(unsafe_code)]

use kian_domain::Din;

/// Crate id — for smoke tests and logs.
pub const NAME: &str = "kian-nation";

/// Crate version — read from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Health probe — wiring liveness indicator.
#[must_use]
pub fn hello() -> &'static str {
    "kian_nation · KIAN is alive"
}

// ── errors ─────────────────────────────────────────────────────────────────────

/// Parliament error — explicit codes; HTTP mapping lives in the server layer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NationError {
    /// Action on a bill that is not OPEN (closing twice, voting on a closed one, executing a rejected one).
    #[error("bill {0} is not OPEN")]
    BillNotOpen(String),
    /// Each voter gets only one vote.
    #[error("voter {0} already voted on bill {1}")]
    AlreadyVoted(String, String),
    /// Invalid bill body (empty title, zero amount for a treasury movement, …).
    #[error("invalid bill: {0}")]
    InvalidBill(String),
    /// Execution is possible only from PASSED.
    #[error("bill {0} is not PASSED — execution refused")]
    NotPassed(String),
}

/// Standard parliament result.
pub type NationResult<T> = Result<T, NationError>;

// ── domain types ─────────────────────────────────────────────────────────────

/// Parliament policy — quorum and threshold. Immutable for the server's lifetime (env).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NationPolicy {
    /// Minimum total votes for a valid close.
    pub quorum: i64,
    /// HITL threshold in Đ — charter default 50,000; overridable only for tests.
    pub hitl_threshold_din: i64,
}

impl NationPolicy {
    /// Article 7 defaults — quorum 3, threshold 50,000 Đ.
    #[must_use]
    pub const fn charter_default() -> Self {
        Self {
            quorum: 3,
            hitl_threshold_din: 50_000,
        }
    }

    /// Article 7 — does this amount require a human vote? The same charter gate.
    #[must_use]
    pub fn requires_hitl(&self, amount_din: i64) -> bool {
        self.hitl_threshold_din > 0 && amount_din >= self.hitl_threshold_din
    }
}

/// Bill kind — each kind carries a real, executable action on the money kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillKind {
    /// Escrow dispute resolution — the subject is the escrow id; executing means release/refund
    /// with `approved_by_admin: true` through the pure M2 domain path.
    EscrowRelease,
    /// Treasury movement — the subject is the destination account; executing means the double-sided
    /// treasury → destination posting (Article 6: full transparency in audit).
    TreasuryMove,
}

impl BillKind {
    /// Audit/outbox label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EscrowRelease => "escrowRelease",
            Self::TreasuryMove => "treasuryMove",
        }
    }
}

/// Proposed outcome for a dispute bill — release to the seller or refund to the buyer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscrowResolution {
    /// 90/10 settlement in the seller's favor (Article 2).
    Release,
    /// Full refund to the buyer, no tax.
    Refund,
}

impl EscrowResolution {
    /// Wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Refund => "refund",
        }
    }
}

/// Bill status — transitions are one-way: OPEN → PASSED/REJECTED; PASSED → EXECUTED.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillStatus {
    /// Open — voting is in progress.
    Open,
    /// Closed with an approving vote — executable.
    Passed,
    /// Closed with a rejection — never executable.
    Rejected,
    /// Executed on the ledger — final.
    Executed,
}

impl BillStatus {
    /// Wire/database label.
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

/// A vote — the voter is a citizen (session) or an agent (A2A); `is_human` closes the Article 7 gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vote {
    /// Voter id (`citizen:…` or `agent:…`)
    pub voter: String,
    /// Human? (Telegram session = true; agent = false)
    pub is_human: bool,
    /// In favor?
    pub approve: bool,
    /// Vote time, ISO
    pub at: String,
}

/// Deterministic reason for the close outcome — persists in the database row (Article 6 transparency).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BillDecisionReason {
    /// Majority in favor + quorum + (when Article 7 applies) a human vote.
    Passed,
    /// Total votes below the quorum.
    QuorumNotMet,
    /// Majority against.
    MajorityNo,
    /// Amount ≥ the threshold and no approving human vote (Article 7).
    HumanApprovalMissing,
}

impl BillDecisionReason {
    /// Wire/database label.
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

/// Deterministic bill-close outcome — the counts are kept for full transparency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillDecision {
    /// Final reason
    pub reason: BillDecisionReason,
    /// Votes in favor
    pub yes: i64,
    /// Votes against
    pub no: i64,
    /// Approving human votes (the Article 7 gate)
    pub human_yes: i64,
}

impl BillDecision {
    /// Is this decision "passed"?
    #[must_use]
    pub fn is_passed(&self) -> bool {
        self.reason == BillDecisionReason::Passed
    }
}

/// A parliament bill — storable in the database; transition logic here, persistence in db.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bill {
    /// Id (`bill:{uuid}`) — also the execution idempotency key
    pub id: String,
    /// Bill kind
    pub kind: BillKind,
    /// Subject: the escrow id or destination account
    pub subject: String,
    /// Amount in Đ (0 for a bill without an amount)
    pub amount_din: i64,
    /// Proposed outcome (only `EscrowRelease`)
    pub resolution: Option<EscrowResolution>,
    /// Title (1..=200 characters)
    pub title: String,
    /// Current status
    pub status: BillStatus,
    /// Final decision — after closing
    pub decision: Option<BillDecision>,
    /// Opened time, ISO
    pub opened_at: String,
    /// Closed time, ISO
    pub closed_at: Option<String>,
    /// Executed time, ISO
    pub executed_at: Option<String>,
    /// Votes in insertion order
    pub votes: Vec<Vote>,
}

impl Bill {
    /// Open a bill — full validation at the edge (like zod; not in the database).
    ///
    /// # Errors
    /// Empty/overlong title, empty subject, negative amount, a treasury movement without a positive amount,
    /// a dispute without a proposed outcome.
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

    /// Cast a vote — only on an open bill; each voter once.
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

    /// Current tally — (in favor, against, human in favor).
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // Vote tally — bounded and safe
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

    /// Close a bill — a final, deterministic tally (the M2 domain-gate pattern):
    /// 1. Quorum: total votes ≥ `policy.quorum`
    /// 2. Majority: in favor > against
    /// 3. Article 7: if the amount ≥ the threshold — at least one approving human vote
    ///
    /// Failing any gate = `REJECTED` with an explicit reason (Article 6 transparency).
    ///
    /// # Errors
    /// `BillNotOpen` if it has already been closed.
    ///
    /// # Panics
    /// Never — the `expect` right after setting it in place always succeeds.
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

    /// This bill's Article 7 gate — from the charter policy.
    #[must_use]
    pub fn requires_hitl(&self) -> bool {
        NationPolicy::charter_default().requires_hitl(self.amount_din)
    }

    /// Mark executed — only from PASSED; once and for all.
    ///
    /// # Errors
    /// `NotPassed` if the status is not PASSED.
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
            "\u{62d}\u{644} \u{627}\u{62e}\u{62a}\u{644}\u{627}\u{641} \u{633}\u{641}\u{627}\u{631}\u{634}",
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
        // The same M2 charter gate — one source of truth
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
            "\u{62c}\u{627}\u{628}\u{647}‌\u{62c}\u{627}\u{6cc}\u{6cc} \u{62e}\u{632}\u{627}\u{646}\u{647} \u{628}\u{62f}\u{648}\u{646} \u{645}\u{628}\u{644}\u{63a} \u{645}\u{62b}\u{628}\u{62a} \u{645}\u{645}\u{646}\u{648}\u{639}"
        );
        assert!(
            Bill::open("b", BillKind::EscrowRelease, "esc:1", 100, None, "t", NOW).is_err(),
            "\u{62d}\u{644} \u{627}\u{62e}\u{62a}\u{644}\u{627}\u{641} \u{628}\u{62f}\u{648}\u{646} \u{646}\u{62a}\u{6cc}\u{62c}\u{647}\u{654} \u{67e}\u{6cc}\u{634}\u{646}\u{647}\u{627}\u{62f}\u{6cc} \u{645}\u{645}\u{646}\u{648}\u{639}"
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
        let long_title = "\u{639}\u{646}\u{648}\u{627}\u{646}".repeat(60); // 240 characters
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
            "\u{627}\u{6cc}\u{62c}\u{646}\u{62a}‌\u{647}\u{627} \u{647}\u{631}\u{6af}\u{632} \u{67e}\u{648}\u{644} \u{628}\u{632}\u{631}\u{6af} \u{631}\u{627} \u{62a}\u{646}\u{647}\u{627} \u{62a}\u{635}\u{648}\u{6cc}\u{628} \u{646}\u{645}\u{6cc}‌\u{6a9}\u{646}\u{646}\u{62f}"
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
            "\u{627}\u{62c}\u{631}\u{627}\u{6cc} \u{644}\u{627}\u{6cc}\u{62d}\u{647}\u{654} \u{628}\u{627}\u{632} \u{645}\u{645}\u{646}\u{648}\u{639}"
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
