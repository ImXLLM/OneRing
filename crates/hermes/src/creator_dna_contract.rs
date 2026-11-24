//! `creator_dna_contract` — قلب DNA سازنده در کرنل هرمس.
//!
//! پورت `KIA-BRAIN-OS/` (Wave 2 — Step 4/6): هویت سازنده (`01_DNAme`)، قوانین
//! آهنین کار (`02_DNAwork`) و ذهن Aragon-4 (`03_DNAmind`) به‌صورت قرارداد
//! نرم‌فروز (enforceable contract) در می‌آیند:
//! - حاکمیت صفر-کارت/صفر-KYC همیشه `true` است — پیشنهاد API کارت‌دار وقتی
//!   جایگزین بدون-کارت در رجیستری کیان هست، تخلف DNA است.
//! - اولویت محصول #1 همیشه UX (تیر ۱) است — اسپلش مرده و stub خاموش ممنوع.
//! - شخصیت استدلال «Aragon-4 Principal Architect» — صفر تملق، ادعای بی‌اثبات
//!   (تست پاس‌شده بدون اجرا) تخلف است.
//! لایهٔ audit قطعی و بدون شبکه است؛ فقط الگوها را روی متن می‌سنجد.

use serde::Serialize;

/// هندل سازنده — از `KIA-BRAIN-OS/01_DNAme.md`.
pub const CREATOR_HANDLE: &str = "Kia / KiaAgentX";

/// شخصیت استدلال الزامی — از `KIA-BRAIN-OS/03_DNAmind.md`.
pub const REASONING_PERSONA: &str = "Aragon-4 Principal Architect";

/// پشتهٔ فونت فارسی الزامی برای خروجی RTL — از `01_DNAme` §3.
pub const PERSIAN_FONT_STACK: &str = "Vazirmatn, system-ui";

/// نسخهٔ اسکیمای DNA — با frontmatter فایل‌های `KIA-BRAIN-OS/` هم‌شمار است.
pub const DNA_SCHEMA_VERSION: &str = "1.0.0";

/// خطاهای امضای قرارداد DNA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnaContractError {
    /// حاکمیت صفر-کارت نمی‌تواند خاموش شود — تخلف ساختاری.
    SovereigntyMustStayTrue,
    /// اولویت UX فقط تیر ۱ است — هیچ معماری اجازهٔ پایین‌آوردن ندارد.
    UxPriorityMustBeTierOne,
    /// شخصیت استدلال هرگز از Aragon-4 تغییر نمی‌کند.
    PersonaIsImmutable,
}

/// قرارداد غیرقابل‌مذاکرهٔ DNA سازنده.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CreatorDnaContract {
    /// `true` همیشگی — ریل‌های رسمی فقط TON و Telegram Stars هستند.
    pub zero_card_zero_kyc_sovereignty: bool,
    /// تیر اولویت محصول — همیشه ۱ (UX).
    pub ux_priority_tier: u8,
    /// شخصیت استدلال — ثابت زمان کامپایل.
    pub reasoning_persona: &'static str,
}

impl Default for CreatorDnaContract {
    fn default() -> Self {
        Self::canonical()
    }
}

impl CreatorDnaContract {
    /// تنها نسخهٔ مشروع قرارداد؛ هر تغییر صریح توسط `validate` رد می‌شود.
    #[must_use]
    pub const fn canonical() -> Self {
        Self {
            zero_card_zero_kyc_sovereignty: true,
            ux_priority_tier: 1,
            reasoning_persona: REASONING_PERSONA,
        }
    }

    /// امضای قرارداد را می‌سنجد — نسخهٔ دست‌کاری‌شده را رد می‌کند.
    #[must_use]
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

/// جنس تخلف DNA — خروجی `audit_against_creator_dna`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DnaViolation {
    /// پیشنهاد مسیرِ کارت‌دار/کارت‌نیاز با وجود وجود جایگزین بدون-کارت.
    CardGatedApiAssumption {
        snippet: String,
        guidance: String,
    },
    /// ادعای تکمیل/سالمی بدون شاهد اجرا — نقض صفر تملق Aragon-4.
    UnverifiedCompletionClaim {
        snippet: String,
        guidance: String,
    },
    /// stub مردهٔ UI / اسپلش بازاریابی — نقض قانون فریم‌اول Astra.
    UxFirstViolation {
        snippet: String,
        guidance: String,
    },
}

impl DnaViolation {
    /// برچسب کوتاه برای گزارش‌های ماشینی.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CardGatedApiAssumption { .. } => "card_gated_api_assumption",
            Self::UnverifiedCompletionClaim { .. } => "unverified_completion_claim",
            Self::UxFirstViolation { .. } => "ux_first_violation",
        }
    }
}

/// الگوهای پیشنهادِ کارت‌دار — کافی است یکی ظاهر شود تا تخلف ثبت شود.
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
    "نیازمند کارت",
    "کارت بانکی لازم",
    "کارت اعتباری",
];

/// الگوهای ادعای بی‌اثبات — نقض «عدد بدون اجرا ممنوع».
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
    "همهٔ تست‌ها پاس",
    "قطعاً کار می‌کند",
    "بدون تست هم درست است",
];

/// الگوهای stub مردهٔ UI — نقض قانون فریم‌اول تعاملی.
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
    "بعداً وصل می‌شود",
    "اسپلش شروع",
    "پیج قالبی",
];

const GUIDANCE_CARD: &str = "حاکمیت صفر-کارت (DNAme §2): مسیر بدون-کارت/بدون-کلید از رجیستری kian-ai (opencode_zen + free_llm_registry، آبشار ۳لایه) را پیشنهاد بده.";
const GUIDANCE_CLAIM: &str = "صفر تملق (DNAmind §3): ادعا را با خروجی اجرا اثبات کن — `cargo test`/`clippy`/عدد چاپ‌شده؛ وگرنه «برآورد» برچسب بزن.";
const GUIDANCE_UX: &str = "فریم‌اول تعاملی (DNAme §3 + دروازهٔ Astra): خروجی باید از فریم اول اپ زنده باشد؛ stub مرده یا اسپلش بازاریابی ممنوع است.";

/// اسنیپت متنی اطراف نخستین الگوی یافته‌شده — برش امن روی مرز نویسه‌ها
/// (برای متن فارسی/یونیکد، offsets روی نسخهٔ lowercase بریده می‌شوند).
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

/// لنگر سیستم (system prompt anchor) که هر ایجنت کیان بالای پرامپت می‌بیند —
/// ترجمهٔ ماشینی `DNAme` + `DNAwork` به یک بلوک قطعی فارسی/انگلیسی.
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

/// ممیزی قطعی متن علیه DNA سازنده — بدون شبکه، بدون مدل، فقط الگو.
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
            "cargo test -p kian-ai → 12 green; clippy -D warnings → 0 warnings. RTL صفحه با dir=rtl ست شد.",
        );
        assert!(clean.is_empty(), "clean verified text must not violate DNA: {clean:?}");
    }

    #[test]
    fn audit_is_bilingual_persian_english() {
        assert_eq!(
            audit_against_creator_dna("این سرویس نیازمند کارت بانکی لازم است").len(),
            1
        );
        assert_eq!(audit_against_creator_dna("قطعاً کار می‌کند بدون تست هم درست است").len(), 1);
        assert_eq!(audit_against_creator_dna("دکمهٔ پرداخت بعداً وصل می‌شود").len(), 1);
    }
}
