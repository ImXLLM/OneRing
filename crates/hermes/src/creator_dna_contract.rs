//! `creator_dna_contract` — the creator DNA heart in the Hermes kernel.
//!
//! Port of `KIA-BRAIN-OS/` (Wave 2 — Step 4/6): the creator identity (`01_DNAme`), the iron work
//! rules (`02_DNAwork`), and the Aragon-4 mind (`03_DNAmind`) become an
//! enforceable contract:
//! - zero-card/zero-KYC sovereignty is always `true` — proposing a card-gated API while
//!   a card-free alternative exists in the Kian registry is a DNA violation.
//! - product priority #1 is always UX (tier 1) — dead splashes and silent stubs are forbidden.
//! - reasoning persona "Aragon-4 Principal Architect" — zero flattery; unverified claims
//!   (tests passing without a run) are a violation.
//!
//! The audit layer is deterministic and network-free; it only measures patterns against text.

use serde::Serialize;

/// Creator handle — from `KIA-BRAIN-OS/01_DNAme.md`.
pub const CREATOR_HANDLE: &str = "Kia / KiaAgentX";

/// Required reasoning persona — from `KIA-BRAIN-OS/03_DNAmind.md`.
pub const REASONING_PERSONA: &str = "Aragon-4 Principal Architect";

/// The mandatory Persian font stack for RTL output — from `01_DNAme` §3.
pub const PERSIAN_FONT_STACK: &str = "Vazirmatn, system-ui";

/// DNA schema version — kept in lockstep with the `KIA-BRAIN-OS/` file frontmatter.
pub const DNA_SCHEMA_VERSION: &str = "1.0.0";

/// DNA contract-signature errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnaContractError {
    /// Zero-card sovereignty cannot be switched off — a structural violation.
    SovereigntyMustStayTrue,
    /// The UX priority is tier 1 only — no architecture may lower it.
    UxPriorityMustBeTierOne,
    /// The reasoning persona never changes from Aragon-4.
    PersonaIsImmutable,
}

/// The non-negotiable creator DNA contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CreatorDnaContract {
    /// Always `true` — the official rails are TON and Telegram Stars only.
    pub zero_card_zero_kyc_sovereignty: bool,
    /// Product priority tier — always 1 (UX).
    pub ux_priority_tier: u8,
    /// Reasoning persona — a compile-time constant.
    pub reasoning_persona: &'static str,
}

impl Default for CreatorDnaContract {
    fn default() -> Self {
        Self::canonical()
    }
}

impl CreatorDnaContract {
    /// The single legitimate version of the contract; any explicit change is rejected by `validate`.
    #[must_use]
    pub const fn canonical() -> Self {
        Self {
            zero_card_zero_kyc_sovereignty: true,
            ux_priority_tier: 1,
            reasoning_persona: REASONING_PERSONA,
        }
    }

    /// Checks the contract signature — rejects a tampered version.
    ///
    /// # Errors
    /// `SovereigntyMustStayTrue` if sovereignty has been switched off,
    /// `UxPriorityMustBeTierOne` if the UX tier is anything but 1, and
    /// `PersonaIsImmutable` if the reasoning persona has been replaced.
    pub fn validate(self) -> Result<(), DnaContractError> {
        if !self.zero_card_zero_kyc_sovereignty {
            return Err(DnaContractError::SovereigntyMustStayTrue);
        }
        if self.ux_priority_tier != 1 {
            return Err(DnaContractError::UxPriorityMustBeTierOne);
        }
        if self.reasoning_persona != REASONING_PERSONA {
            return Err(DnaContractError::PersonaIsImmutable);
        }
        Ok(())
    }
}

/// The DNA violation kind — the output of `audit_against_creator_dna`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DnaViolation {
    /// Proposing a card-gated/card-required path although a card-free alternative exists.
    CardGatedApiAssumption {
        snippet: String,
        guidance: String,
    },
    /// A completion/health claim without run evidence — violates Aragon-4 zero flattery.
    UnverifiedCompletionClaim {
        snippet: String,
        guidance: String,
    },
    /// A dead UI stub / marketing splash — violates the Astra first-frame rule.
    UxFirstViolation {
        snippet: String,
        guidance: String,
    },
}

impl DnaViolation {
    /// A short label for machine-generated reports.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CardGatedApiAssumption { .. } => "card_gated_api_assumption",
            Self::UnverifiedCompletionClaim { .. } => "unverified_completion_claim",
            Self::UxFirstViolation { .. } => "ux_first_violation",
        }
    }
}

/// Card-gated suggestion patterns — one hit is enough to record a violation.
const CARD_GATED_PATTERNS: &[&str] = &[
    "requires a credit card",
    "credit card required",
    "credit card is required",
    "card on file",
    "paid-only",
    "paid only",
    "requires payment",
    "payment required",
    "subscription required",
    "needs a paid plan",
    "card-gated",
    "requires billing info",
    "\u{646}\u{6cc}\u{627}\u{632}\u{645}\u{646}\u{62f} \u{6a9}\u{627}\u{631}\u{62a}",
    "\u{6a9}\u{627}\u{631}\u{62a} \u{628}\u{627}\u{646}\u{6a9}\u{6cc} \u{644}\u{627}\u{632}\u{645}",
    "\u{6a9}\u{627}\u{631}\u{62a} \u{627}\u{639}\u{62a}\u{628}\u{627}\u{631}\u{6cc}",
];

/// Unverified-claim patterns — violates "no numbers without a run".
const UNVERIFIED_CLAIM_PATTERNS: &[&str] = &[
    "all tests pass",
    "everything works",
    "fully tested",
    "definitely works",
    "guaranteed to work",
    "100% working",
    "no bugs",
    "trust me",
    "production ready without running",
    "\u{647}\u{645}\u{647}\u{654} \u{62a}\u{633}\u{62a}‌\u{647}\u{627} \u{67e}\u{627}\u{633}",
    "\u{642}\u{637}\u{639}\u{627}\u{64b} \u{6a9}\u{627}\u{631} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f}",
    "\u{628}\u{62f}\u{648}\u{646} \u{62a}\u{633}\u{62a} \u{647}\u{645} \u{62f}\u{631}\u{633}\u{62a} \u{627}\u{633}\u{62a}",
];

/// Dead UI stub patterns — violates the interactive first-frame rule.
const UX_FIRST_PATTERNS: &[&str] = &[
    "add the ui later",
    "ui comes later",
    "placeholder page",
    "dead splash",
    "marketing splash",
    "coming soon screen",
    "skip the ui for now",
    "stub component with no handler",
    "wire it up later",
    "\u{628}\u{639}\u{62f}\u{627}\u{64b} \u{648}\u{635}\u{644} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f}",
    "\u{627}\u{633}\u{67e}\u{644}\u{634} \u{634}\u{631}\u{648}\u{639}",
    "\u{67e}\u{6cc}\u{62c} \u{642}\u{627}\u{644}\u{628}\u{6cc}",
];

const GUIDANCE_CARD: &str = "\u{62d}\u{627}\u{6a9}\u{645}\u{6cc}\u{62a} \u{635}\u{641}\u{631}-\u{6a9}\u{627}\u{631}\u{62a} (DNAme §2): \u{645}\u{633}\u{6cc}\u{631} \u{628}\u{62f}\u{648}\u{646}-\u{6a9}\u{627}\u{631}\u{62a}/\u{628}\u{62f}\u{648}\u{646}-\u{6a9}\u{644}\u{6cc}\u{62f} \u{627}\u{632} \u{631}\u{62c}\u{6cc}\u{633}\u{62a}\u{631}\u{6cc} kian-ai (opencode_zen + free_llm_registry\u{60c} \u{622}\u{628}\u{634}\u{627}\u{631} \u{6f3}\u{644}\u{627}\u{6cc}\u{647}) \u{631}\u{627} \u{67e}\u{6cc}\u{634}\u{646}\u{647}\u{627}\u{62f} \u{628}\u{62f}\u{647}.";
const GUIDANCE_CLAIM: &str = "\u{635}\u{641}\u{631} \u{62a}\u{645}\u{644}\u{642} (DNAmind §3): \u{627}\u{62f}\u{639}\u{627} \u{631}\u{627} \u{628}\u{627} \u{62e}\u{631}\u{648}\u{62c}\u{6cc} \u{627}\u{62c}\u{631}\u{627} \u{627}\u{62b}\u{628}\u{627}\u{62a} \u{6a9}\u{646} — `cargo test`/`clippy`/\u{639}\u{62f}\u{62f} \u{686}\u{627}\u{67e}‌\u{634}\u{62f}\u{647}\u{61b} \u{648}\u{6af}\u{631}\u{646}\u{647} «\u{628}\u{631}\u{622}\u{648}\u{631}\u{62f}» \u{628}\u{631}\u{686}\u{633}\u{628} \u{628}\u{632}\u{646}.";
const GUIDANCE_UX: &str = "\u{641}\u{631}\u{6cc}\u{645}‌\u{627}\u{648}\u{644} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} (DNAme §3 + \u{62f}\u{631}\u{648}\u{627}\u{632}\u{647}\u{654} Astra): \u{62e}\u{631}\u{648}\u{62c}\u{6cc} \u{628}\u{627}\u{6cc}\u{62f} \u{627}\u{632} \u{641}\u{631}\u{6cc}\u{645} \u{627}\u{648}\u{644} \u{627}\u{67e} \u{632}\u{646}\u{62f}\u{647} \u{628}\u{627}\u{634}\u{62f}\u{61b} stub \u{645}\u{631}\u{62f}\u{647} \u{6cc}\u{627} \u{627}\u{633}\u{67e}\u{644}\u{634} \u{628}\u{627}\u{632}\u{627}\u{631}\u{6cc}\u{627}\u{628}\u{6cc} \u{645}\u{645}\u{646}\u{648}\u{639} \u{627}\u{633}\u{62a}.";

/// The text snippet around the first matched pattern — a safe cut on character boundaries
/// (for Persian/Unicode text, offsets are cut on the lowercase version).
fn bounded_snippet(lower: &str, match_idx: usize, pat_len: usize) -> String {
    let mut start = match_idx.saturating_sub(28);
    let mut end = (match_idx + pat_len + 28).min(lower.len());
    while start > 0 && !lower.is_char_boundary(start) {
        start -= 1;
    }
    while end < lower.len() && !lower.is_char_boundary(end) {
        end += 1;
    }
    let mut snippet = lower[start..end].trim().to_string();
    if snippet.chars().count() > 72 {
        snippet = snippet.chars().take(72).collect();
    }
    snippet
}

fn find_first(payload: &str, patterns: &[&str]) -> Option<String> {
    let lower = payload.to_lowercase();
    patterns
        .iter()
        .find_map(|p| lower.find(p).map(|idx| (p, idx)))
        .map(|(p, idx)| bounded_snippet(&lower, idx, p.len()))
}

/// The system prompt anchor every Kian agent sees at the top of its prompt —
/// a machine translation of `DNAme` + `DNAwork` into one deterministic Persian/English block.
#[must_use]
pub fn build_creator_dna_system_anchor() -> String {
    format!(
        "# CREATOR DNA ANCHOR (KIA-BRAIN-OS {DNA_SCHEMA_VERSION})\n\
         Creator: {CREATOR_HANDLE}. Reasoning persona: {REASONING_PERSONA} — zero sycophancy; \
         every numeric claim needs executed proof (cargo test / printed number); guesses stay labeled.\n\
         Sovereignty: 100% Zero-Credit-Card & Zero-KYC. Never recommend card-gated APIs, paid-only \
         plans or KYC rails while zero-card/keyless lanes exist in the KIAN registry (opencode_zen \
         `Bearer public`, free_llm_registry 3-tier waterfall). Official rails: TON deposits and \
         Telegram Stars (XTR) only. Non-custodial: user keys never touch the server.\n\
         Product priority #1 = UX tier 1: Frame-1 interactive output (no dead marketing splash, \
         no dead stubs), single-file self-contained `<!DOCTYPE html>`, Persian RTL with \
         `dir=\"rtl\"` + font stack `{PERSIAN_FONT_STACK}` and English LTR BiDi cleanliness, \
         mobile 390x844 zero horizontal overflow, touch targets >= 44px.\n\
         Ledger invariants (DNAwork §3): 466 citizens / 247 referral lines / 108 escrows / \
         20 active products / SUM(Posting.amount) = 0 across 58,700,200,000 micro-KIAN. \
         Any injection stays idempotent and never touches the ledger."
    )
}

/// Deterministic text audit against the creator DNA — no network, no model, patterns only.
#[must_use]
pub fn audit_against_creator_dna(payload: &str) -> Vec<DnaViolation> {
    let mut out = Vec::new();
    if let Some(snippet) = find_first(payload, CARD_GATED_PATTERNS) {
        out.push(DnaViolation::CardGatedApiAssumption {
            snippet,
            guidance: GUIDANCE_CARD.to_string(),
        });
    }
    if let Some(snippet) = find_first(payload, UNVERIFIED_CLAIM_PATTERNS) {
        out.push(DnaViolation::UnverifiedCompletionClaim {
            snippet,
            guidance: GUIDANCE_CLAIM.to_string(),
        });
    }
    if let Some(snippet) = find_first(payload, UX_FIRST_PATTERNS) {
        out.push(DnaViolation::UxFirstViolation {
            snippet,
            guidance: GUIDANCE_UX.to_string(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_contract_is_valid_and_immutable() {
        let c = CreatorDnaContract::canonical();
        assert!(c.validate().is_ok());
        assert!(c.zero_card_zero_kyc_sovereignty);
        assert_eq!(c.ux_priority_tier, 1);
        assert_eq!(c.reasoning_persona, "Aragon-4 Principal Architect");
    }

    #[test]
    fn tampered_persona_fails_validation() {
        let bad = CreatorDnaContract {
            reasoning_persona: "yes-man consultant",
            ..CreatorDnaContract::canonical()
        };
        assert_eq!(bad.validate(), Err(DnaContractError::PersonaIsImmutable));
    }

    #[test]
    fn tampered_sovereignty_fails_validation() {
        let bad = CreatorDnaContract {
            zero_card_zero_kyc_sovereignty: false,
            ..CreatorDnaContract::canonical()
        };
        assert_eq!(bad.validate(), Err(DnaContractError::SovereigntyMustStayTrue));
    }

    #[test]
    fn tampered_ux_tier_fails_validation() {
        let bad = CreatorDnaContract {
            ux_priority_tier: 2,
            ..CreatorDnaContract::canonical()
        };
        assert_eq!(bad.validate(), Err(DnaContractError::UxPriorityMustBeTierOne));
    }

    #[test]
    fn anchor_encodes_sovereignty_ux_and_rtl_rules() {
        let anchor = build_creator_dna_system_anchor();
        assert!(anchor.contains(CREATOR_HANDLE));
        assert!(anchor.contains("Zero-Credit-Card & Zero-KYC"));
        assert!(anchor.contains("dir=\\\"rtl\\\"") || anchor.contains("dir=\"rtl\""));
        assert!(anchor.contains(PERSIAN_FONT_STACK));
        assert!(anchor.contains("58,700,200,000 micro-KIAN"));
        assert!(anchor.contains("Telegram Stars"));
        assert!(anchor.contains(REASONING_PERSONA));
    }

    #[test]
    fn audit_flags_card_gated_recommendation() {
        let hits = audit_against_creator_dna(
            "This provider is great, but note it requires a credit card on signup.",
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind(), "card_gated_api_assumption");
        match &hits[0] {
            DnaViolation::CardGatedApiAssumption { guidance, .. } => {
                assert!(guidance.contains("opencode_zen"));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn audit_flags_unverified_completion_claim() {
        let hits = audit_against_creator_dna("Refactored the wallet; all tests pass, ship it.");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind(), "unverified_completion_claim");
    }

    #[test]
    fn audit_flags_dead_ux_stub() {
        let hits = audit_against_creator_dna("First ship the API; we can add the ui later.");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind(), "ux_first_violation");
    }

    #[test]
    fn audit_flags_multiple_classes_and_clean_text_passes() {
        let dirty = audit_against_creator_dna(
            "It needs a paid plan; also coming soon screen first; guaranteed to work.",
        );
        assert_eq!(dirty.len(), 3);
        let clean = audit_against_creator_dna(
            "cargo test -p kian-ai → 12 green; clippy -D warnings → 0 warnings. RTL \u{635}\u{641}\u{62d}\u{647} \u{628}\u{627} dir=rtl \u{633}\u{62a} \u{634}\u{62f}.",
        );
        assert!(clean.is_empty(), "clean verified text must not violate DNA: {clean:?}");
    }

    #[test]
    fn audit_is_bilingual_persian_english() {
        assert_eq!(
            audit_against_creator_dna("\u{627}\u{6cc}\u{646} \u{633}\u{631}\u{648}\u{6cc}\u{633} \u{646}\u{6cc}\u{627}\u{632}\u{645}\u{646}\u{62f} \u{6a9}\u{627}\u{631}\u{62a} \u{628}\u{627}\u{646}\u{6a9}\u{6cc} \u{644}\u{627}\u{632}\u{645} \u{627}\u{633}\u{62a}").len(),
            1
        );
        assert_eq!(audit_against_creator_dna("\u{642}\u{637}\u{639}\u{627}\u{64b} \u{6a9}\u{627}\u{631} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f} \u{628}\u{62f}\u{648}\u{646} \u{62a}\u{633}\u{62a} \u{647}\u{645} \u{62f}\u{631}\u{633}\u{62a} \u{627}\u{633}\u{62a}").len(), 1);
        assert_eq!(audit_against_creator_dna("\u{62f}\u{6a9}\u{645}\u{647}\u{654} \u{67e}\u{631}\u{62f}\u{627}\u{62e}\u{62a} \u{628}\u{639}\u{62f}\u{627}\u{64b} \u{648}\u{635}\u{644} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f}").len(), 1);
    }
}
