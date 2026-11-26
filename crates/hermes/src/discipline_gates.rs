//! `discipline_gates` — Kian builder discipline gates (Wave 2 — Step 5).
//!
//! A port of three behavioral treasuries into one deterministic module:
//! - `ultraworkers/claw-code` → [`SYSTEM_PROMPT_DYNAMIC_BOUNDARY`]: the system prompt
//!   cache boundary — cacheable static blocks before it, dynamic state after;
//!   any byte moved breaks the prompt cache and multiplies token cost.
//! - `anthropics/skills` (frontend-design + web-artifacts-builder) →
//!   [`audit_frontend_slop`]: five "AI-slop" markers in front-end output —
//!   generic font stack, purple-blue gradient, bouncy easing, neon on unrequested
//!   dark themes, and templated hero copy.
//! - `obra/superpowers` v7.0.2 → [`verify_completion_claim`] + [`ReviewStage`]:
//!   The iron rule «no completion claim without fresh execution evidence» and the
//!   two-pass review (`SpecCompliance` before `CodeQuality`).
//! - `affaan-m/everything-claude-code` v2.2.3 → [`should_suggest_strategic_compact`]
//!   and [`AtomicInstinct`]: the strategic `/compact` suggestion at the 50 tool-call
//!   boundary and every 25 calls after, or the token threshold; atomic instincts with
//!   confidence clamped to `0.3..=0.9`.
//!
//! All functions are deterministic; no IO and no hidden clocks are involved.
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

/// System-prompt cache boundary — cacheable static blocks must end before this marker;
/// dynamic blocks (session state, turn memory, execution evidence) come after it so
/// the static prefix stays stable in the provider cache.
pub const SYSTEM_PROMPT_DYNAMIC_BOUNDARY: &str = "__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__";

// ── 1. Front-end anti-slop validator (`anthropics/skills`) ────────────────────

/// Distinctive font names — their presence means deliberate pairing, not the default `SaaS` stack.
/// Generic `serif`/`monospace` are deliberately absent — `sans-serif` would spoof them.
const DISTINCTIVE_FONT_HINTS: &[&str] = &[
    "vazirmatn", "ibm plex", "jetbrains", "geist", "playfair", "crimson", "fraunces",
    "space grotesk", "instrument", "sora", "outfit", "rubik", "lemonada", "vazir",
    "iransans", "kalameh",
];

/// Templated hero phrases — the marker of stock copy without a dedicated brief.
const HERO_STOCK_COPY: &[&str] = &[
    "empowering your journey",
    "unlock your potential",
    "take your project to the next level",
    "supercharge your workflow",
    "blazing fast",
    "seamlessly integrate",
    "the future of",
    "revolutionize",
];

/// Finds the five forbidden AI-slop markers in HTML/CSS/TSX output — the return
/// codes are static and feed the builder quality gate directly:
/// `SLOP_GENERIC_FONT_STACK`, `SLOP_PURPLE_BLUE_GRADIENT`,
/// `SLOP_BOUNCE_ELASTIC_EASING`, `SLOP_UNREQUESTED_NEON_GLOW`, and
/// `SLOP_HERO_STOCK_COPY`.
pub fn audit_frontend_slop(html_or_css: &str) -> Vec<&'static str> {
    let lower = html_or_css.to_ascii_lowercase();
    let mut out = Vec::new();

    // 1) generic Inter/Roboto font stack with no distinctive pairing
    let font_families: Vec<String> = lower
        .split([';', '{', '\n'])
        .filter(|seg| seg.contains("font-family") || seg.contains("font-sans"))
        .map(str::to_string)
        .collect();
    let generic_font = font_families.iter().any(|f| f.contains("inter") || f.contains("roboto"));
    let distinctive = font_families
        .iter()
        .any(|f| DISTINCTIVE_FONT_HINTS.iter().any(|h| f.contains(h)));
    if generic_font && !distinctive {
        out.push("SLOP_GENERIC_FONT_STACK");
    }

    // 2) purple-blue gradient on a white card — the SaaS cliché
    let purple = lower.contains("purple") || lower.contains("indigo")
        || lower.contains("#8b5cf6") || lower.contains("#7c3aed") || lower.contains("#a855f7");
    let blue_target = lower.contains("to-blue") || lower.contains("#3b82f6")
        || lower.contains("#2563eb") || lower.contains("blue-");
    let has_gradient = lower.contains("linear-gradient") || lower.contains("bg-gradient");
    let white_card = lower.contains("#fff") || lower.contains("white") || lower.contains("bg-white");
    if has_gradient && purple && (blue_target || white_card) {
        out.push("SLOP_PURPLE_BLUE_GRADIENT");
    }

    // 3) bouncy/elastic easing — bounce/elastic or cubic-bezier with overshoot
    let bounce_ease = lower.contains("bounce") || lower.contains("elastic")
        || lower.contains("cubic-bezier(0.68, -0.55") || lower.contains("cubic-bezier(.68,-.55");
    if bounce_ease && (lower.contains("@keyframes") || lower.contains("animation") || lower.contains("transition")) {
        out.push("SLOP_BOUNCE_ELASTIC_EASING");
    }

    // 4) neon glow on an unrequested dark background
    let dark_bg = lower.contains("#0a0a0a") || lower.contains("#000")
        || lower.contains("background:#0") || lower.contains("background-color:#0")
        || lower.contains("bg-black") || lower.contains("bg-[#0");
    let neon_glow = lower.contains("text-shadow") || lower.contains("box-shadow");
    let neon_color = lower.contains("#0f0") || lower.contains("#0ff") || lower.contains("#f0f")
        || lower.contains("#39ff14") || lower.contains("#00ffff") || lower.contains("#ff00ff")
        || lower.contains("neon");
    if dark_bg && neon_glow && neon_color {
        out.push("SLOP_UNREQUESTED_NEON_GLOW");
    }

    // 5) templated hero copy
    if HERO_STOCK_COPY.iter().any(|p| lower.contains(p)) {
        out.push("SLOP_HERO_STOCK_COPY");
    }

    out
}

// ── 2. Completion verification gate (`obra/superpowers`) ──────────────────────────

/// «Red-flag» phrases — reasoning/wishful self-soothing instead of fresh command evidence.
const HEDGE_RED_FLAGS: &[&str] = &[
    "should work now",
    "should pass",
    "should work",
    "probably fixed",
    "probably",
    "seems to",
    "looks correct",
    "i'm confident",
    "i am confident",
    "just this once",
];

/// The `VerificationBeforeCompletion` iron rule: a completion claim without fresh execution
/// evidence in the same turn, or paired with a wishful phrase (`should work now`,
/// `probably fixed`, etc.) is rejected — `Ok(())` only with fresh evidence and
/// argument-free text.
///
/// # Errors
/// `Err(&'static str)` on the first red flag or a missing fresh witness — the message is the
/// iron rule itself so it lands straight in the builder's ledger.
pub fn verify_completion_claim(
    claim_text: &str,
    has_fresh_command_evidence: bool,
) -> Result<(), &'static str> {
    let lower = claim_text.to_ascii_lowercase();
    for flag in HEDGE_RED_FLAGS {
        if lower.contains(flag) {
            return Err("Red-flag rationalization detected: replace speculation with fresh command output.");
        }
    }
    if !has_fresh_command_evidence {
        return Err(
            "NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE — run the command, read the full output, and check the exit code.",
        );
    }
    Ok(())
}

/// Two-pass review — pass 1 (`SpecCompliance`) must clear before pass 2
/// (`CodeQuality`) even opens; the reverse order is never allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewStage {
    /// Pass 1 — contract/spec compliance (first).
    SpecCompliance,
    /// Pass 2 — code quality (only after pass 1 clears).
    CodeQuality,
}

/// Two-pass review state machine — status + each pass's retained findings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwoStageReview {
    stage: ReviewStage,
    spec_findings: Vec<String>,
    quality_findings: Vec<String>,
}

impl Default for TwoStageReview {
    fn default() -> Self {
        Self { stage: ReviewStage::SpecCompliance, spec_findings: Vec::new(), quality_findings: Vec::new() }
    }
}

impl TwoStageReview {
    /// Adjudicates pass 1 — non-compliance stays in `SpecCompliance`.
    pub fn record_spec_verdict(&mut self, compliant: bool, findings: Vec<String>) -> ReviewStage {
        self.spec_findings = findings;
        self.stage = if compliant && self.spec_findings.is_empty() {
            ReviewStage::CodeQuality
        } else {
            ReviewStage::SpecCompliance
        };
        self.stage
    }

    /// Adjudicates pass 2 — if pass 1 has not cleared, `Err` (reverse order forbidden).
    ///
    /// # Errors
    /// `Err(&'static str)` while pass 1 has not cleared — quality findings before
    /// spec compliance are moot.
    pub fn record_quality_verdict(
        &mut self,
        approved: bool,
        critical_findings: Vec<String>,
    ) -> Result<ReviewStage, &'static str> {
        if self.stage != ReviewStage::CodeQuality {
            return Err("Stage 2 (CodeQuality) cannot run before Stage 1 (SpecCompliance) passes.");
        }
        self.quality_findings = critical_findings;
        if approved && self.quality_findings.is_empty() {
            Ok(ReviewStage::CodeQuality)
        } else {
            Err("CodeQuality rejected: critical findings must be fixed and re-reviewed.")
        }
    }

    /// The current pass — for the ledger and HITL.
    pub fn stage(&self) -> ReviewStage {
        self.stage
    }
}

// ── 3. Strategic compaction and atomic instincts (`affaan-ecc`) ────────────────────

/// `/compact` suggestion token threshold — port of `suggest-compact.js` (160k).
pub const STRATEGIC_COMPACT_TOKEN_THRESHOLD: usize = 160_000;
/// The first tool-call boundary (50) and the repeat stride after it (25).
pub const STRATEGIC_COMPACT_TOOL_CALL_THRESHOLD: u32 = 50;
pub const STRATEGIC_COMPACT_TOOL_CALL_INTERVAL: u32 = 25;

/// Should the strategic `/compact` suggestion fire? Exactly at call 50 and
/// every 25 calls after (50, 75, 100, ...), or once the token estimate reaches 160k.
pub fn should_suggest_strategic_compact(tool_calls: u32, estimated_tokens: usize) -> bool {
    if estimated_tokens >= STRATEGIC_COMPACT_TOKEN_THRESHOLD {
        return true;
    }
    tool_calls >= STRATEGIC_COMPACT_TOOL_CALL_THRESHOLD
        && (tool_calls - STRATEGIC_COMPACT_TOOL_CALL_THRESHOLD)
            .is_multiple_of(STRATEGIC_COMPACT_TOOL_CALL_INTERVAL)
}

/// The `continuous-learning-v2` atomic instinct — "when you see a trigger, take the action" with
/// confidence clamped to `0.3..=0.9` (tentative 0.3 up to core 0.9).
#[derive(Debug, Clone, PartialEq)]
pub struct AtomicInstinct {
    /// Stable id (`inst:parameterize-sql`).
    pub id: String,
    /// The observed trigger.
    pub trigger: String,
    /// The mandatory action.
    pub action: String,
    /// Confidence in the closed interval `0.3..=0.9` — any out-of-range number is clamped.
    pub confidence: f32,
}

/// Instinct confidence floor and ceiling — the tentative/core thresholds per the ECC treasury convention.
pub const INSTINCT_MIN_CONFIDENCE: f32 = 0.3;
pub const INSTINCT_MAX_CONFIDENCE: f32 = 0.9;

impl AtomicInstinct {
    /// A fresh instinct — always tentative at confidence 0.3.
    pub fn new(id: impl Into<String>, trigger: impl Into<String>, action: impl Into<String>) -> Self {
        Self { id: id.into(), trigger: trigger.into(), action: action.into(), confidence: INSTINCT_MIN_CONFIDENCE }
    }

    /// Clamps confidence into `0.3..=0.9` and returns it.
    pub fn set_confidence(&mut self, value: f32) -> f32 {
        self.confidence = value.clamp(INSTINCT_MIN_CONFIDENCE, INSTINCT_MAX_CONFIDENCE);
        self.confidence
    }

    /// Reinforcement (+0.15) or violation (−0.15) — always clamped into the allowed range.
    pub fn reinforce(&mut self, confirmed: bool) -> f32 {
        let delta = if confirmed { 0.15 } else { -0.15 };
        self.set_confidence(self.confidence + delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_boundary_is_a_stable_sentinel() {
        // the cache boundary must stay fixed and lexeme-independent — builders weave it between blocks;
        // any string change breaks the provider cache.
        assert_eq!(SYSTEM_PROMPT_DYNAMIC_BOUNDARY, "__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__");
        let static_prefix = format!("DNA blocks\n{SYSTEM_PROMPT_DYNAMIC_BOUNDARY}\nuser turn state");
        assert!(static_prefix.split_once(SYSTEM_PROMPT_DYNAMIC_BOUNDARY).is_some());
    }

    #[test]
    fn anti_slop_flags_all_five_tells() {
        let bad = "body{background:#000;font-family:'Inter',sans-serif;box-shadow:0 0 40px #39ff14;\
                   background:linear-gradient(to right,#8b5cf6,#3b82f6);\
                   @keyframes bounce{0%{transform:translateY(0)}100%{transform:translateY(-8px)}}\
                   <h1>Unlock your potential</h1>";
        let codes = audit_frontend_slop(bad);
        assert_eq!(codes.len(), 5, "all five tells: {codes:?}");
        assert!(codes.contains(&"SLOP_GENERIC_FONT_STACK"));
        assert!(codes.contains(&"SLOP_PURPLE_BLUE_GRADIENT"));
        assert!(codes.contains(&"SLOP_BOUNCE_ELASTIC_EASING"));
        assert!(codes.contains(&"SLOP_UNREQUESTED_NEON_GLOW"));
        assert!(codes.contains(&"SLOP_HERO_STOCK_COPY"));
    }

    #[test]
    fn intentional_design_passes_the_slop_audit() {
        let good = "body{background:#0E1715;color:#EBF2EE;font-family:'Vazirmatn','IBM Plex Sans';\
                    box-shadow:0 1px 2px rgba(0,0,0,.3)}\
                    <h1>\u{62a}\u{631}\u{627}\u{632}\u{645}\u{62a} \u{634}\u{645}\u{627}: \u{6f5}\u{6f8}\u{66c}\u{6f7}\u{6f0}\u{6f0} \u{645}\u{6cc}\u{644}\u{6cc}‌KIAN</h1>";
        assert!(audit_frontend_slop(good).is_empty(), "distinctive pairing + restrained chrome");
    }

    #[test]
    fn completion_claims_require_fresh_evidence_and_no_hedging() {
        assert_eq!(
            verify_completion_claim("It should work now.", true),
            Err("Red-flag rationalization detected: replace speculation with fresh command output.")
        );
        assert_eq!(
            verify_completion_claim("probably fixed the bug", true),
            Err("Red-flag rationalization detected: replace speculation with fresh command output.")
        );
        assert!(verify_completion_claim("It should work now.", false).is_err());
        assert!(verify_completion_claim("All 244 tests pass; clippy exit 0.", false).is_err());
        assert!(verify_completion_claim("cargo test: 95 passed; 0 failed (fresh this turn)", true).is_ok());
    }

    #[test]
    fn two_stage_review_blocks_quality_before_spec() {
        let mut review = TwoStageReview::default();
        assert_eq!(review.stage(), ReviewStage::SpecCompliance);
        // pass 2 before pass 1 — forbidden
        assert!(review.record_quality_verdict(true, Vec::new()).is_err());

        // pass 1 failed → still in SpecCompliance
        assert_eq!(
            review.record_spec_verdict(false, vec!["missing CAS guard".into()]),
            ReviewStage::SpecCompliance
        );
        assert!(review.record_quality_verdict(true, Vec::new()).is_err());

        // pass 1 cleared → pass 2 opens and approves
        assert_eq!(review.record_spec_verdict(true, Vec::new()), ReviewStage::CodeQuality);
        assert!(review.record_quality_verdict(true, Vec::new()).is_ok());
        assert_eq!(review.stage(), ReviewStage::CodeQuality);

        // pass 2 with a critical finding — rejected
        let mut review2 = TwoStageReview::default();
        review2.record_spec_verdict(true, Vec::new());
        assert!(review2.record_quality_verdict(true, vec!["unwrap in prod".into()]).is_err());
    }

    #[test]
    fn strategic_compact_fires_at_50_then_every_25_or_token_threshold() {
        // before 50 — off even with a low token count
        assert!(!should_suggest_strategic_compact(49, 10_000));
        // exactly 50 — it fires
        assert!(should_suggest_strategic_compact(50, 10_000));
        // 51..74 — off
        assert!(!should_suggest_strategic_compact(51, 10_000));
        assert!(!should_suggest_strategic_compact(74, 10_000));
        // 75 and 100 — the next boundaries
        assert!(should_suggest_strategic_compact(75, 10_000));
        assert!(should_suggest_strategic_compact(100, 10_000));
        // the token threshold is call-count independent
        assert!(should_suggest_strategic_compact(3, 160_000));
        assert!(should_suggest_strategic_compact(3, 200_001));
        assert!(!should_suggest_strategic_compact(3, 159_999));
    }

    #[test]
    fn atomic_instinct_clamps_confidence_in_03_to_09_band() {
        let mut inst = AtomicInstinct::new(
            "inst:parameterize-sql",
            "when writing SQL with user input",
            "always bind parameters",
        );
        assert_eq!(inst.confidence, 0.3);
        // reinforced up to the 0.9 ceiling
        for _ in 0..10 {
            inst.reinforce(true);
        }
        assert_eq!(inst.confidence, INSTINCT_MAX_CONFIDENCE, "capped at 0.9");
        // violated down to the 0.3 floor
        for _ in 0..10 {
            inst.reinforce(false);
        }
        assert_eq!(inst.confidence, INSTINCT_MIN_CONFIDENCE, "floored at 0.3");
        // out-of-range inputs are clamped
        assert_eq!(inst.set_confidence(0.05), 0.3);
        assert_eq!(inst.set_confidence(0.99), 0.9);
        assert_eq!(inst.set_confidence(0.75), 0.75);
    }
}
