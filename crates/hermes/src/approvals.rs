//! `approvals` — six-class HITL governance with dual voice + subagent and routine guards.
//!
//! A Wave 2 port from the CopilotKit/OpenBot treasure trove:
//! - `shared/approval-outcomes.ts` → [`ApprovalOutcome`]: every decision has two voices —
//!   a sentence for the model (`outcome_for_model`) so it reports honestly and does not hunt
//!   for a workaround, and a sentence for the human (`outcome_for_person_fa`) that is blunt and short.
//!   Changing one never leaves the other behind, because both are defined side by side and both are tested.
//! - `docs/approvals.md` + `shared/what-if.ts` → [`simulate_approval_policy`]:
//!   The deterministic "what if…" policy simulator — the same layer order the product
//!   enforces, with no network and no randomness.
//! - `shared/subagent.ts` → [`deterministic_subagent_thread_id`]: thread
//!   The subagent is "derived", not "multiplied" — from parent + tool call, as a `UUIDv8`, so
//!   the subagent transcript is findable even after a restart and rebirth never occurs.
//! - `docs/routines.md` → [`RoutineFatigueGuard`]: routine anti-spam — a 15-minute floor,
//!   a cap of 20 active routines, a circuit breaker after 10 consecutive failures, and
//!   skipping cron windows missed after a restart (instead of a firing burst).
//!
//! The "secret box" pattern is layered here as well: credential and access management is
//! never auto-approved — it is the human's own job.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

/// Human-approval threshold from the Kian charter (Article 7) — 50,000 dinar.
pub use kian_domain::charter::Charter;

/// The six action classes that, per `docs/approvals.md`, "get asked in every release" — the
/// display order is the same as the policy priority order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalCategory {
    /// Sending a message/email/invitation to others.
    SendMessages,
    /// Buying, paying, or moving money.
    MakePurchasesOrTransfers,
    /// Deleting or altering existing data.
    DeleteOrModifyData,
    /// Publishing public content (a post, a tweet, a product launch).
    PublishPublicContent,
    /// Managing credentials or access (password, token, permission) — the secret box.
    ManageCredentialsOrPermissions,
    /// Submitting a form or a formal request.
    SubmitFormsOrApplications,
}

impl ApprovalCategory {
    /// Persian label for the approval card and the policy page.
    #[must_use]
    pub const fn label_fa(self) -> &'static str {
        match self {
            Self::SendMessages => "\u{627}\u{631}\u{633}\u{627}\u{644} \u{67e}\u{6cc}\u{627}\u{645} \u{628}\u{647} \u{62f}\u{6cc}\u{6af}\u{631}\u{627}\u{646}",
            Self::MakePurchasesOrTransfers => "\u{62e}\u{631}\u{6cc}\u{62f} \u{6cc}\u{627} \u{62c}\u{627}\u{628}\u{647}‌\u{62c}\u{627}\u{6cc}\u{6cc} \u{67e}\u{648}\u{644}",
            Self::DeleteOrModifyData => "\u{62d}\u{630}\u{641} \u{6cc}\u{627} \u{62a}\u{63a}\u{6cc}\u{6cc}\u{631} \u{62f}\u{627}\u{62f}\u{647}",
            Self::PublishPublicContent => "\u{627}\u{646}\u{62a}\u{634}\u{627}\u{631} \u{645}\u{62d}\u{62a}\u{648}\u{627}\u{6cc} \u{639}\u{645}\u{648}\u{645}\u{6cc}",
            Self::ManageCredentialsOrPermissions => "\u{645}\u{62f}\u{6cc}\u{631}\u{6cc}\u{62a} \u{627}\u{639}\u{62a}\u{628}\u{627}\u{631}\u{646}\u{627}\u{645}\u{647} \u{6cc}\u{627} \u{62f}\u{633}\u{62a}\u{631}\u{633}\u{6cc}",
            Self::SubmitFormsOrApplications => "\u{62b}\u{628}\u{62a} \u{641}\u{631}\u{645} \u{6cc}\u{627} \u{62f}\u{631}\u{62e}\u{648}\u{627}\u{633}\u{62a}",
        }
    }
}

/// The bilingual outcome of an approval card — the model voice and the human voice side by side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalOutcome {
    /// The human allowed it.
    Approved,
    /// The human explicitly denied it — the model may not retry, amend, or bypass.
    DeclinedByPerson,
    /// The decision deadline expired and nothing was done.
    TimedOut,
}

impl ApprovalOutcome {
    /// The sentence the model reads — it states who decided and what to do next,
    /// so its report stays honest and seeks no workaround (a port of `OpenBot`).
    #[must_use]
    pub const fn outcome_for_model(self) -> &'static str {
        match self {
            Self::Approved => {
                "APPROVED: The person allowed this action. Proceed, and report the result truthfully in your answer."
            }
            Self::DeclinedByPerson => {
                "DECLINED_BY_PERSON: The user explicitly declined this action. Do not retry, rephrase, or work around it."
            }
            Self::TimedOut => {
                "TIMED_OUT: The approval request expired without a decision, so it was not done. Ask again only if it is still needed, or say in your answer that it timed out."
            }
        }
    }

    /// The sentence the human reads — blunt, short, no model-facing wordiness.
    #[must_use]
    pub const fn outcome_for_person_fa(self) -> &'static str {
        match self {
            Self::Approved => "\u{627}\u{646}\u{62c}\u{627}\u{645} \u{634}\u{62f}.",
            Self::DeclinedByPerson => "\u{62f}\u{631}\u{62e}\u{648}\u{627}\u{633}\u{62a} \u{62a}\u{648}\u{633}\u{637} \u{634}\u{645}\u{627} \u{631}\u{62f} \u{634}\u{62f}.",
            Self::TimedOut => "\u{632}\u{645}\u{627}\u{646} \u{62a}\u{623}\u{6cc}\u{6cc}\u{62f} \u{628}\u{647} \u{67e}\u{627}\u{6cc}\u{627}\u{646} \u{631}\u{633}\u{6cc}\u{62f} \u{648} \u{6a9}\u{627}\u{631}\u{6cc} \u{627}\u{646}\u{62c}\u{627}\u{645} \u{646}\u{634}\u{62f}.",
        }
    }
}

/// The context of a hypothetical action for the policy simulator — the same chips as the what-if page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalContext {
    /// A monetary amount in dinar (None means a non-monetary action).
    pub amount_din: Option<i64>,
    /// An app-labeled destructive tool — prompts regardless of the tool name.
    pub destructive: bool,
    /// A short action description for the card (e.g. "a 2-million-dinar transfer to wallet X").
    pub detail: String,
}

impl ApprovalContext {
    /// A plain non-monetary, non-destructive context.
    #[must_use]
    pub fn plain(detail: impl Into<String>) -> Self {
        Self { amount_din: None, destructive: false, detail: detail.into() }
    }
}

/// The policy verdict for a hypothetical action — a deterministic tri-state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyVerdict {
    /// Runs without a card.
    AutoApprove,
    /// Needs a human approval card (dual voice).
    RequiresHumanConfirmation,
    /// The policy rejects outright — even a human cannot pass this route.
    Blocked,
}

impl PolicyVerdict {
    /// A one-sentence Persian rationale for the what-if card and the audit log.
    #[must_use]
    pub const fn reason_fa(self) -> &'static str {
        match self {
            Self::AutoApprove => "\u{633}\u{6cc}\u{627}\u{633}\u{62a} \u{627}\u{6cc}\u{646} \u{627}\u{642}\u{62f}\u{627}\u{645} \u{631}\u{627} \u{628}\u{62f}\u{648}\u{646} \u{6a9}\u{627}\u{631}\u{62a} \u{62a}\u{623}\u{6cc}\u{6cc}\u{62f} \u{627}\u{62c}\u{631}\u{627} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f}.",
            Self::RequiresHumanConfirmation => "\u{627}\u{6cc}\u{646} \u{627}\u{642}\u{62f}\u{627}\u{645} \u{6a9}\u{627}\u{631}\u{62a} \u{62a}\u{623}\u{6cc}\u{6cc}\u{62f} \u{627}\u{646}\u{633}\u{627}\u{646}\u{6cc} \u{645}\u{6cc}‌\u{62e}\u{648}\u{627}\u{647}\u{62f}.",
            Self::Blocked => "\u{633}\u{6cc}\u{627}\u{633}\u{62a} \u{627}\u{6cc}\u{646} \u{627}\u{642}\u{62f}\u{627}\u{645} \u{631}\u{627} \u{628}\u{647}‌\u{6a9}\u{644}\u{6cc} \u{645}\u{6cc}‌\u{628}\u{646}\u{62f}\u{62f} — \u{6a9}\u{627}\u{631}\u{650} \u{62e}\u{648}\u{62f} \u{627}\u{646}\u{633}\u{627}\u{646} \u{627}\u{633}\u{62a}.",
        }
    }
}

/// The "what if…" policy simulator — a pure, deterministic function (a port of what-if-check).
///
/// The layer order is the same order the product enforces; the first deterministic answer wins:
/// 1. **Secret box:** `ManageCredentialsOrPermissions` is always [`PolicyVerdict::Blocked`]
///    — only the human ever enters the password/token, and no answer may sit in on
///    their behalf (the "I'll do it myself" pattern in `OpenBot`).
/// 2. **Destructive-labeled:** `destructive` wants a card regardless of the tool name.
/// 3. **Kian charter gate:** purchases/transfers above the charter HITL threshold (50,000 Đ)
///    need human approval; below the threshold, the automated escrow lane releases
///    ([`PolicyVerdict::AutoApprove`]) and a negative amount is fully blocked.
/// 4. The remaining classes always want a card — send, delete, publish, and form submission.
///
/// No input causes I/O or randomness; the same function runs both for the "what
/// if…" button and for the real gate, so the simulator and reality can never
/// disagree.
#[must_use]
pub fn simulate_approval_policy(category: ApprovalCategory, context: &ApprovalContext) -> PolicyVerdict {
    // 1. Secret box — even a human does not pass this gate.
    if matches!(category, ApprovalCategory::ManageCredentialsOrPermissions) {
        return PolicyVerdict::Blocked;
    }
    // 2. A destructive-labeled tool — wants a card whatever its name.
    if context.destructive {
        return PolicyVerdict::RequiresHumanConfirmation;
    }
    // 3. The money gate — the Kian charter threshold.
    if matches!(category, ApprovalCategory::MakePurchasesOrTransfers) {
        return match context.amount_din {
            None => PolicyVerdict::RequiresHumanConfirmation,
            // an invalid amount (negative or beyond the safe 2^53−1 bound) is fully blocked.
            Some(amount) => match kian_domain::Din::new(amount) {
                Ok(din) if kian_domain::requires_hitl(din) => PolicyVerdict::RequiresHumanConfirmation,
                Ok(_) => PolicyVerdict::AutoApprove,
                Err(_) => PolicyVerdict::Blocked,
            },
        };
    }
    // 4. The remaining classes always want a human card.
    PolicyVerdict::RequiresHumanConfirmation
}

/// Is the string in standard UUID shape (36 characters, hyphens at 8/13/18/23)?
fn looks_like_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, b) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *b != b'-' {
                    return false;
                }
            }
            _ => {
                if !b.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

/// The subagent's deterministic thread — derived, not multiplied (a port of `subagentThreadId`).
///
/// A SHA-256 hash over `subagent\n{parent_thread_id}\n{tool_call_id}`; if the parent
/// is a UUID, the first 6 bytes are replaced with the parent's bytes so the subagent carries
/// the conversation's own deployment fingerprint. The version locks to 8 (`UUIDv8`:
/// these bits are not random) and the RFC variant is honored too.
///
/// Both sides (server and client) know the same derivation, so the client can locate
/// the subagent transcript "before the result arrives", and after a restart the
/// same thread is recovered — rebirth and duplicate spawning are impossible.
///
/// Nothing here is a secret; knowing the ID grants nothing.
#[must_use]
pub fn deterministic_subagent_thread_id(parent_thread_id: &str, tool_call_id: &str) -> String {
    use std::fmt::Write as _;

    let digest = Sha256::digest(
        format!("subagent\n{parent_thread_id}\n{tool_call_id}").as_bytes(),
    );
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    if looks_like_uuid(parent_thread_id) {
        let hex: String = parent_thread_id.chars().filter(|c| *c != '-').collect();
        for (index, byte) in bytes.iter_mut().take(6).enumerate() {
            *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap_or(0);
        }
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x80; // version 8
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // RFC variant
    let mut hex = String::with_capacity(bytes.len() * 2);
    for b in &bytes {
        let _ = write!(hex, "{b:02x}");
    }
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// The routine fatigue guard's verdict for one scheduling window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FireDecision {
    /// Firing is allowed.
    Fire,
    /// The 15-minute floor has not elapsed yet — defer to the next turn.
    Throttled,
    /// The active-routine cap is full.
    CapReached,
    /// The circuit breaker is open — the routine was auto-disabled after consecutive failures.
    DisabledAfterFailures,
    /// The cron window was missed after a restart — skip, no firing burst.
    SkippedMissedWindow,
}

impl FireDecision {
    /// Persian explanation for the log and the routines page.
    #[must_use]
    pub const fn reason_fa(self) -> &'static str {
        match self {
            Self::Fire => "\u{631}\u{648}\u{62a}\u{6cc}\u{646} \u{62f}\u{631} \u{67e}\u{646}\u{62c}\u{631}\u{647}\u{654} \u{62e}\u{648}\u{62f}\u{634} \u{634}\u{644}\u{6cc}\u{6a9} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f}.",
            Self::Throttled => "\u{641}\u{627}\u{635}\u{644}\u{647}\u{654} \u{62d}\u{62f}\u{627}\u{642}\u{644}\u{6cc} \u{6f1}\u{6f5} \u{62f}\u{642}\u{6cc}\u{642}\u{647} \u{647}\u{646}\u{648}\u{632} \u{62a}\u{645}\u{627}\u{645} \u{646}\u{634}\u{62f}\u{647} \u{627}\u{633}\u{62a}.",
            Self::CapReached => "\u{633}\u{642}\u{641} \u{6f2}\u{6f0} \u{631}\u{648}\u{62a}\u{6cc}\u{646} \u{641}\u{639}\u{627}\u{644} \u{67e}\u{631} \u{627}\u{633}\u{62a}\u{61b} \u{62a}\u{627} \u{62e}\u{627}\u{644}\u{6cc} \u{634}\u{62f}\u{646} \u{635}\u{628}\u{631} \u{6a9}\u{646}.",
            Self::DisabledAfterFailures => "\u{631}\u{648}\u{62a}\u{6cc}\u{646} \u{67e}\u{633} \u{627}\u{632} \u{6f1}\u{6f0} \u{634}\u{6a9}\u{633}\u{62a} \u{67e}\u{6cc}\u{627}\u{67e}\u{6cc} \u{62e}\u{648}\u{62f}\u{6a9}\u{627}\u{631} \u{62e}\u{627}\u{645}\u{648}\u{634} \u{634}\u{62f}.",
            Self::SkippedMissedWindow => "\u{67e}\u{646}\u{62c}\u{631}\u{647}\u{654} \u{627}\u{632} \u{62f}\u{633}\u{62a} \u{631}\u{641}\u{62a}\u{647} \u{67e}\u{633} \u{627}\u{632} \u{631}\u{6cc}‌\u{627}\u{633}\u{62a}\u{627}\u{631}\u{62a} \u{67e}\u{631}\u{634} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f}\u{60c} \u{646}\u{647} \u{634}\u{644}\u{6cc}\u{6a9} \u{627}\u{646}\u{628}\u{648}\u{647}.",
        }
    }
}

/// The routine anti-spam guard — a port of the `OpenBot` routines policy with mission constants:
/// - [`Self::MIN_INTERVAL_SECS`] = 900 (a 15-minute floor between two firings of one routine);
/// - [`Self::MAX_ACTIVE_ROUTINES`] = 20 (the active-routine cap);
/// - [`Self::MAX_CONSECUTIVE_FAILURES`] = 10 (circuit breaker: auto-disable).
///
/// The guard is fully deterministic: the caller supplies time (testability without sleeps),
/// and windows missed after a restart are skipped instead of bursting every
/// backlogged window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoutineFatigueGuard {
    /// Each routine's state keyed by its ID.
    state: BTreeMap<String, RoutineState>,
}

/// A routine's internal state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RoutineState {
    /// The last permitted firing (epoch seconds).
    last_fired_secs: Option<u64>,
    /// The consecutive-failure count without a success.
    consecutive_failures: u32,
}

impl RoutineFatigueGuard {
    /// The floor between two firings of one routine: 15 minutes.
    pub const MIN_INTERVAL_SECS: u64 = 900;
    /// The cap on concurrently active routines.
    pub const MAX_ACTIVE_ROUTINES: usize = 20;
    /// The circuit-breaker threshold: auto-disable after this many consecutive failures.
    pub const MAX_CONSECUTIVE_FAILURES: u32 = 10;

    /// A fresh, empty guard.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// This scheduling window's verdict for the given routine.
    ///
    /// The layer order (the first deterministic answer wins):
    /// 1. Circuit breaker open → [`FireDecision::DisabledAfterFailures`];
    /// 2. Missed window (scheduled before now, after a restart) →
    ///    [`FireDecision::SkippedMissedWindow`] — skip, no firing burst;
    /// 3. Active-routine cap → [`FireDecision::CapReached`];
    /// 4. 15-minute floor → [`FireDecision::Throttled`];
    /// 5. Otherwise [`FireDecision::Fire`] and record the firing time.
    pub fn evaluate(
        &mut self,
        routine_id: &str,
        now_secs: u64,
        scheduled_for_secs: u64,
        active_count: usize,
    ) -> FireDecision {
        let state = self.state.entry(routine_id.to_owned()).or_default();
        if state.consecutive_failures >= Self::MAX_CONSECUTIVE_FAILURES {
            return FireDecision::DisabledAfterFailures;
        }
        if scheduled_for_secs < now_secs {
            // after a restart, all backlogged windows do not fire at once;
            // the routine waits for the next scheduled window.
            return FireDecision::SkippedMissedWindow;
        }
        if active_count >= Self::MAX_ACTIVE_ROUTINES {
            return FireDecision::CapReached;
        }
        if let Some(last) = state.last_fired_secs
            && now_secs.saturating_sub(last) < Self::MIN_INTERVAL_SECS
        {
            return FireDecision::Throttled;
        }
        state.last_fired_secs = Some(now_secs);
        FireDecision::Fire
    }

    /// Record a success — the consecutive-failure count resets to zero.
    pub fn record_success(&mut self, routine_id: &str) {
        let state = self.state.entry(routine_id.to_owned()).or_default();
        state.consecutive_failures = 0;
    }

    /// Record a failure — at the threshold, the circuit breaker opens (auto-disable).
    pub fn record_failure(&mut self, routine_id: &str) {
        let state = self.state.entry(routine_id.to_owned()).or_default();
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
    }

    /// Is this routine's circuit breaker open?
    #[must_use]
    pub fn is_disabled(&self, routine_id: &str) -> bool {
        self.state
            .get(routine_id)
            .is_some_and(|s| s.consecutive_failures >= Self::MAX_CONSECUTIVE_FAILURES)
    }

    /// The current consecutive-failure count (for the routines page).
    #[must_use]
    pub fn consecutive_failures(&self, routine_id: &str) -> u32 {
        self.state
            .get(routine_id)
            .map_or(0, |s| s.consecutive_failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The model voice and the human voice are defined side by side — a human denial is an
    /// explicit "do not bypass" for the model and a short Persian sentence for the human.
    #[test]
    fn dual_voice_outcomes_never_leak_model_voice_to_person() {
        let declined = ApprovalOutcome::DeclinedByPerson;
        assert_eq!(
            declined.outcome_for_model(),
            "DECLINED_BY_PERSON: The user explicitly declined this action. Do not retry, rephrase, or work around it."
        );
        assert_eq!(declined.outcome_for_person_fa(), "\u{62f}\u{631}\u{62e}\u{648}\u{627}\u{633}\u{62a} \u{62a}\u{648}\u{633}\u{637} \u{634}\u{645}\u{627} \u{631}\u{62f} \u{634}\u{62f}.");
        assert!(declined.outcome_for_model().starts_with("DECLINED_BY_PERSON"));
        assert!(!declined.outcome_for_person_fa().contains("DECLINED"));

        assert!(ApprovalOutcome::Approved.outcome_for_model().starts_with("APPROVED"));
        assert!(ApprovalOutcome::TimedOut.outcome_for_model().starts_with("TIMED_OUT"));
        assert_eq!(ApprovalOutcome::TimedOut.outcome_for_person_fa(), "\u{632}\u{645}\u{627}\u{646} \u{62a}\u{623}\u{6cc}\u{6cc}\u{62f} \u{628}\u{647} \u{67e}\u{627}\u{6cc}\u{627}\u{646} \u{631}\u{633}\u{6cc}\u{62f} \u{648} \u{6a9}\u{627}\u{631}\u{6cc} \u{627}\u{646}\u{62c}\u{627}\u{645} \u{646}\u{634}\u{62f}.");
    }

    /// The simulator's layer order: the secret box blocks, the destructive tool wants a card,
    /// money passes the charter gate, and the rest always get a human card.
    #[test]
    fn what_if_policy_layers_match_the_product_order() {
        // secret box — even a zero, non-destructive amount is blocked.
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::ManageCredentialsOrPermissions, &ApprovalContext::plain("\u{62a}\u{63a}\u{6cc}\u{6cc}\u{631} \u{631}\u{645}\u{632} \u{6a9}\u{6cc}\u{641}")),
            PolicyVerdict::Blocked
        );
        // destructive-labeled — wants a card in any class.
        let destructive = ApprovalContext { amount_din: Some(100), destructive: true, detail: "\u{67e}\u{627}\u{6a9}‌\u{633}\u{627}\u{632}\u{6cc} \u{622}\u{631}\u{634}\u{6cc}\u{648}".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::DeleteOrModifyData, &destructive),
            PolicyVerdict::RequiresHumanConfirmation
        );
        // money: below the charter threshold auto-approve, at the threshold human, negative blocked.
        let buy_small = ApprovalContext { amount_din: Some(Charter::HITL_THRESHOLD_DIN.as_i64() - 1), destructive: false, detail: "\u{62e}\u{631}\u{6cc}\u{62f} \u{645}\u{62d}\u{635}\u{648}\u{644}".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::MakePurchasesOrTransfers, &buy_small),
            PolicyVerdict::AutoApprove
        );
        let buy_big = ApprovalContext { amount_din: Some(Charter::HITL_THRESHOLD_DIN.as_i64()), destructive: false, detail: "\u{627}\u{646}\u{62a}\u{642}\u{627}\u{644} \u{628}\u{632}\u{631}\u{6af}".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::MakePurchasesOrTransfers, &buy_big),
            PolicyVerdict::RequiresHumanConfirmation
        );
        let buy_negative = ApprovalContext { amount_din: Some(-1), destructive: false, detail: "\u{645}\u{628}\u{644}\u{63a} \u{646}\u{627}\u{645}\u{639}\u{62a}\u{628}\u{631}".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::MakePurchasesOrTransfers, &buy_negative),
            PolicyVerdict::Blocked
        );
        let buy_unknown = ApprovalContext { amount_din: None, destructive: false, detail: "\u{645}\u{628}\u{644}\u{63a} \u{646}\u{627}\u{645}\u{639}\u{644}\u{648}\u{645}".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::MakePurchasesOrTransfers, &buy_unknown),
            PolicyVerdict::RequiresHumanConfirmation
        );
        // the remaining classes always want a human card.
        for category in [
            ApprovalCategory::SendMessages,
            ApprovalCategory::DeleteOrModifyData,
            ApprovalCategory::PublishPublicContent,
            ApprovalCategory::SubmitFormsOrApplications,
        ] {
            assert_eq!(
                simulate_approval_policy(category, &ApprovalContext::plain("\u{627}\u{642}\u{62f}\u{627}\u{645} \u{641}\u{631}\u{636}\u{6cc}")),
                PolicyVerdict::RequiresHumanConfirmation,
                "{category:?}"
            );
        }
    }

    /// The subagent `UUIDv8`: deterministic, well-formed, parent-fingerprinted, and tool-call sensitive.
    #[test]
    fn subagent_thread_id_is_deterministic_uuidv8_with_parent_fingerprint() {
        let parent = "11111111-2222-3333-4444-555555555555";
        let a = deterministic_subagent_thread_id(parent, "call-1");
        let b = deterministic_subagent_thread_id(parent, "call-1");
        assert_eq!(a, b, "same parent + call → same thread, no duplicate spawn");

        // UUIDv8 shape: version on the high 8 bits and the RFC variant.
        assert_eq!(a.len(), 36);
        assert_eq!(a.chars().nth(14), Some('8'), "version nibble must be 8: {a}");
        assert!(
            matches!(a.chars().nth(19), Some('8' | '9' | 'a' | 'b')),
            "variant nibble must be RFC: {a}"
        );
        let c = deterministic_subagent_thread_id(parent, "call-2");
        assert_ne!(a, c, "different tool call → different thread");

        // the first 6 bytes are inherited from the parent (deployment fingerprint); the
        // version-8 nibble is locked in the third group (8 in the high nibble).
        assert!(a.starts_with("11111111-2222-8"), "parent fingerprint + version 8: {a}");

        // non-UUID parents work too and stay deterministic.
        let d1 = deterministic_subagent_thread_id("chat:fa:900", "c1");
        let d2 = deterministic_subagent_thread_id("chat:fa:900", "c1");
        assert_eq!(d1, d2);
        assert_eq!(d1.chars().nth(14), Some('8'));
    }

    /// The 15-minute floor: throttled at 899 s, fires at 900 s.
    #[test]
    fn routine_fatigue_enforces_fifteen_minute_floor() {
        let mut guard = RoutineFatigueGuard::new();
        assert_eq!(guard.evaluate("r1", 1_000, 1_000, 0), FireDecision::Fire);
        assert_eq!(guard.evaluate("r1", 1_000 + 899, 1_899, 0), FireDecision::Throttled);
        let fired = guard.evaluate("r1", 1_000 + 900, 1_900, 0);
        assert_eq!(fired, FireDecision::Fire);
        assert_eq!(fired.reason_fa(), "\u{631}\u{648}\u{62a}\u{6cc}\u{646} \u{62f}\u{631} \u{67e}\u{646}\u{62c}\u{631}\u{647}\u{654} \u{62e}\u{648}\u{62f}\u{634} \u{634}\u{644}\u{6cc}\u{6a9} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f}.");
        // again immediately? throttled — the 15-minute floor counts from the fresh firing.
        assert_eq!(guard.evaluate("r1", 1_900, 1_900, 0), FireDecision::Throttled);
        // the second routine is independent.
        assert_eq!(guard.evaluate("r2", 1_001, 1_001, 0), FireDecision::Fire);
    }

    /// The 20-active-routine cap and skipping windows missed after a restart.
    #[test]
    fn routine_cap_and_missed_window_skip_work() {
        let mut guard = RoutineFatigueGuard::new();
        assert_eq!(guard.evaluate("fresh", 5_000, 5_000, RoutineFatigueGuard::MAX_ACTIVE_ROUTINES), FireDecision::CapReached);
        assert_eq!(
            guard.evaluate("fresh", 5_000, 5_000, RoutineFatigueGuard::MAX_ACTIVE_ROUTINES - 1),
            FireDecision::Fire,
            "19 active leaves one seat"
        );
        // the backlogged window after a restart: skipped, even if the interval floor has elapsed.
        assert_eq!(guard.evaluate("fresh", 100_000, 90_000, 0), FireDecision::SkippedMissedWindow);
    }

    /// Circuit breaker: disabled after 10 consecutive failures; one success resets the count.
    #[test]
    fn circuit_breaker_auto_disables_after_ten_failures() {
        let mut guard = RoutineFatigueGuard::new();
        for round in 0..RoutineFatigueGuard::MAX_CONSECUTIVE_FAILURES - 1 {
            guard.record_failure("r1");
            assert_eq!(guard.consecutive_failures("r1"), round + 1);
            assert!(!guard.is_disabled("r1"));
        }
        guard.record_failure("r1");
        assert!(guard.is_disabled("r1"), "10th failure opens the breaker");
        assert_eq!(guard.evaluate("r1", 50_000, 50_000, 0), FireDecision::DisabledAfterFailures);
        // success closes the circuit and the routine fires again.
        guard.record_success("r1");
        assert!(!guard.is_disabled("r1"));
        assert_eq!(guard.evaluate("r1", 50_000, 50_000, 0), FireDecision::Fire);
    }

    /// Six classes and three verdicts — honors the `SCREAMING_SNAKE` serialization contract.
    #[test]
    fn categories_and_verdicts_serialize_stable() {
        let json = serde_json::to_string(&ApprovalCategory::MakePurchasesOrTransfers).expect("serialize");
        assert_eq!(json, "\"MAKE_PURCHASES_OR_TRANSFERS\"");
        let json = serde_json::to_string(&PolicyVerdict::RequiresHumanConfirmation).expect("serialize");
        assert_eq!(json, "\"REQUIRES_HUMAN_CONFIRMATION\"");
        let json = serde_json::to_string(&ApprovalOutcome::DeclinedByPerson).expect("serialize");
        assert_eq!(json, "\"DECLINED_BY_PERSON\"");
        assert_eq!(ApprovalCategory::ManageCredentialsOrPermissions.label_fa(), "\u{645}\u{62f}\u{6cc}\u{631}\u{6cc}\u{62a} \u{627}\u{639}\u{62a}\u{628}\u{627}\u{631}\u{646}\u{627}\u{645}\u{647} \u{6cc}\u{627} \u{62f}\u{633}\u{62a}\u{631}\u{633}\u{6cc}");
    }
}
