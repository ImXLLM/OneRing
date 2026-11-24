//! `discipline_gates` — گیت‌های انضباطی سازندهٔ کیان (Wave 2 — Step 5).
//!
//! پورت سه گنجینهٔ رفتاری در یک ماژول قطعی:
//! - `ultraworkers/claw-code` → [`SYSTEM_PROMPT_DYNAMIC_BOUNDARY`]: مرزِ کش
//!   پرامپت سیستم — بلوک‌های ایستای قابل‌کش قبل از مرز، وضعیت پویا پس از آن؛
//!   هر بایتِ جابه‌جاشده کشِ پرامپت را می‌شکند و هزینهٔ توکن را چندبرابر می‌کند.
//! - `anthropics/skills` (frontend-design + web-artifacts-builder) →
//!   [`audit_frontend_slop`]: پنج نشانهٔ «AI-slop» در خروجی فرانت‌اند —
//!   فونت‌استک عمومی، گرادیان بنفش-آبی، easing فنری، نئون روی تیرهٔ
//!   ناخواسته و متن قهرمان قالبی.
//! - `obra/superpowers` v7.0.2 → [`verify_completion_claim`] + [`ReviewStage`]:
//!   قانون آهنین «هیچ ادعای تکمیل بدون شاهد اجرای تازه» و دوگذرهٔ
//!   رِیو (`SpecCompliance` قبل از `CodeQuality`).
//! - `affaan-m/everything-claude-code` v2.2.3 → [`should_suggest_strategic_compact`]
//!   و [`AtomicInstinct`]: پیشنهاد `/compact` استراتژیک در مرز ۵۰ فراخوان
//!   ابزار و هر ۲۵ فراخوان بعد، یا آستانهٔ توکن؛ غریزهٔ اتمی با اطمینانِ
//!   گیریده در `0.3..=0.9`.
//!
//! همهٔ توابع قطعی‌اند؛ هیچ IO و هیچ ساعت پنهانی در کار نیست.
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

/// مرزِ کشِ پرامپت سیستم — بلوک‌های ایستای قابل‌کش باید قبل از این نشانه
/// تمام شوند؛ بلوک‌های پویا (وضعیت نشست، حافظهٔ نوبت، شاهد اجرا) بعد از آن
/// می‌آیند تا پیشوند ایستا در کش ارائه‌دهنده پایدار بماند.
pub const SYSTEM_PROMPT_DYNAMIC_BOUNDARY: &str = "__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__";

// ── ۱. اعتبارسنج ضد-slop فرانت‌اند (`anthropics/skills`) ────────────────────

/// نام فونت‌های متمایز — حضورشان یعنی جفت‌کردن عامدانه، نه استک پیش‌فرض `SaaS`.
/// `serif`/`monospace` عمومی عمداً اینجا نیستند — `sans-serif` آن‌ها را قلابی می‌کرد.
const DISTINCTIVE_FONT_HINTS: &[&str] = &[
    "vazirmatn", "ibm plex", "jetbrains", "geist", "playfair", "crimson", "fraunces",
    "space grotesk", "instrument", "sora", "outfit", "rubik", "lemonada", "vazir",
    "iransans", "kalameh",
];

/// عبارت‌های قهرمانِ قالبی — نشانهٔ متنِ stock بدون بریف اختصاصی.
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

/// پنج نشانهٔ ممنوعِ AI-slop را در خروجی HTML/CSS/TSX می‌یابد — کدهای
/// بازگشتی static اند و مستقیم در گیت کیفیت سازنده استفاده می‌شوند:
/// `SLOP_GENERIC_FONT_STACK`، `SLOP_PURPLE_BLUE_GRADIENT`،
/// `SLOP_BOUNCE_ELASTIC_EASING`، `SLOP_UNREQUESTED_NEON_GLOW` و
/// `SLOP_HERO_STOCK_COPY`.
pub fn audit_frontend_slop(html_or_css: &str) -> Vec<&'static str> {
    let lower = html_or_css.to_ascii_lowercase();
    let mut out = Vec::new();

    // ۱) فونت‌استک عمومی Inter/Roboto بدون جفت متمایز
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

    // ۲) گرادیان بنفش-آبی روی کارت سفید — کلیشهٔ SaaS
    let purple = lower.contains("purple") || lower.contains("indigo")
        || lower.contains("#8b5cf6") || lower.contains("#7c3aed") || lower.contains("#a855f7");
    let blue_target = lower.contains("to-blue") || lower.contains("#3b82f6")
        || lower.contains("#2563eb") || lower.contains("blue-");
    let has_gradient = lower.contains("linear-gradient") || lower.contains("bg-gradient");
    let white_card = lower.contains("#fff") || lower.contains("white") || lower.contains("bg-white");
    if has_gradient && purple && (blue_target || white_card) {
        out.push("SLOP_PURPLE_BLUE_GRADIENT");
    }

    // ۳) easing فنری/کشسانی — bounce/elastic یا cubic-bezier با overshoot
    let bounce_ease = lower.contains("bounce") || lower.contains("elastic")
        || lower.contains("cubic-bezier(0.68, -0.55") || lower.contains("cubic-bezier(.68,-.55");
    if bounce_ease && (lower.contains("@keyframes") || lower.contains("animation") || lower.contains("transition")) {
        out.push("SLOP_BOUNCE_ELASTIC_EASING");
    }

    // ۴) نئون‌گلو روی زمینهٔ تیرهٔ ناخواسته
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

    // ۵) متن قهرمان قالبی
    if HERO_STOCK_COPY.iter().any(|p| lower.contains(p)) {
        out.push("SLOP_HERO_STOCK_COPY");
    }

    out
}

// ── ۲. گیت راستی‌آزمایی تکمیل (`obra/superpowers`) ──────────────────────────

/// عبارت‌های «پرچم قرمز» — استدلال/دل‌خوش‌کنی به‌جای شاهد فرمان تازه.
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

/// قانون آهنین `VerificationBeforeCompletion`: ادعای تکمیل بدون شاهد اجرای
/// تازه در همین نوبت، یا همراه عبارت دله‌دل‌خوش (`should work now`,
/// `probably fixed` و …) رد می‌شود — `Ok(())` فقط با شاهد تازه و متنِ
/// بی‌استدلال.
///
/// # Errors
/// `Err(&'static str)` روی نخستین پرچم قرمز یا نبودِ شاهد تازه — پیام همان
/// قانون آهنین است تا در لَگر سازنده مستقیم نشیند.
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

/// دوگذرهٔ رِیو — گذرهٔ ۱ (`SpecCompliance`) باید پاس شود تا گذرهٔ ۲
/// (`CodeQuality`) حتی باز شود؛ ترتیب وارونه هرگز مجاز نیست.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewStage {
    /// گذرهٔ ۱ — تطابق با قرارداد/spec (اول).
    SpecCompliance,
    /// گذرهٔ ۲ — کیفیت کد (فقط پس از پاسِ گذرهٔ ۱).
    CodeQuality,
}

/// ماشین‌حالت دوگذرهٔ رِیو — وضعیت + نگه‌داشتن یافته‌های هر گذره.
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
    /// گذرهٔ ۱ را داوری می‌کند — عدم تطابق در `SpecCompliance` می‌ماند.
    pub fn record_spec_verdict(&mut self, compliant: bool, findings: Vec<String>) -> ReviewStage {
        self.spec_findings = findings;
        self.stage = if compliant && self.spec_findings.is_empty() {
            ReviewStage::CodeQuality
        } else {
            ReviewStage::SpecCompliance
        };
        self.stage
    }

    /// گذرهٔ ۲ را داوری می‌کند — اگر گذرهٔ ۱ پاس نشده، `Err` (ترتیب وارونه ممنوع).
    ///
    /// # Errors
    /// `Err(&'static str)` وقتی گذرهٔ ۱ هنوز پاس نشده — یافتهٔ کیفیت قبل از
    /// تطابق spec بلاموضوع است.
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

    /// گذرهٔ فعلی — برای لَگر و HITL.
    pub fn stage(&self) -> ReviewStage {
        self.stage
    }
}

// ── ۳. فشرده‌سازی استراتژیک و غریزهٔ اتمی (`affaan-ecc`) ────────────────────

/// آستانهٔ توکن پیشنهاد `/compact` — پورت `suggest-compact.js` (۱۶۰k).
pub const STRATEGIC_COMPACT_TOKEN_THRESHOLD: usize = 160_000;
/// نخستین مرز فراخوان ابزار (۵۰) و گام تکرار پس از آن (۲۵).
pub const STRATEGIC_COMPACT_TOOL_CALL_THRESHOLD: u32 = 50;
pub const STRATEGIC_COMPACT_TOOL_CALL_INTERVAL: u32 = 25;

/// آیا پیشنهاد `/compact` استراتژیک باید شعله بکشد؟ دقیقاً در فراخوان ۵۰ و
/// هر ۲۵ فراخوان پس از آن (۵۰، ۷۵، ۱۰۰، …)، یا وقتی برآورد توکن به ۱۶۰k رسیده.
pub fn should_suggest_strategic_compact(tool_calls: u32, estimated_tokens: usize) -> bool {
    if estimated_tokens >= STRATEGIC_COMPACT_TOKEN_THRESHOLD {
        return true;
    }
    tool_calls >= STRATEGIC_COMPACT_TOOL_CALL_THRESHOLD
        && (tool_calls - STRATEGIC_COMPACT_TOOL_CALL_THRESHOLD)
            .is_multiple_of(STRATEGIC_COMPACT_TOOL_CALL_INTERVAL)
}

/// غریزهٔ اتمی `continuous-learning-v2` — «وقتی trigger دیدی، action بزن» با
/// اطمینانِ گیریده در `0.3..=0.9` (tentative 0.3 تا core 0.9).
#[derive(Debug, Clone, PartialEq)]
pub struct AtomicInstinct {
    /// شناسهٔ پایدار (`inst:parameterize-sql`).
    pub id: String,
    /// محرک مشاهده‌شده.
    pub trigger: String,
    /// کنش الزامی.
    pub action: String,
    /// اطمینان در بازهٔ بستهٔ `0.3..=0.9` — هر عدد بیرونی گیریده می‌شود.
    pub confidence: f32,
}

/// کف و سقف اطمینان غریزه — tentative/آستانهٔ core قرارداد گنجینهٔ ECC.
pub const INSTINCT_MIN_CONFIDENCE: f32 = 0.3;
pub const INSTINCT_MAX_CONFIDENCE: f32 = 0.9;

impl AtomicInstinct {
    /// غریزهٔ تازه — همیشه tentative با اطمینان 0.3.
    pub fn new(id: impl Into<String>, trigger: impl Into<String>, action: impl Into<String>) -> Self {
        Self { id: id.into(), trigger: trigger.into(), action: action.into(), confidence: INSTINCT_MIN_CONFIDENCE }
    }

    /// اطمینان را در بازهٔ `0.3..=0.9` گیریده می‌کند و برمی‌گرداند.
    pub fn set_confidence(&mut self, value: f32) -> f32 {
        self.confidence = value.clamp(INSTINCT_MIN_CONFIDENCE, INSTINCT_MAX_CONFIDENCE);
        self.confidence
    }

    /// تقویت (+0.15) یا نقض (−0.15) — همیشه در بازهٔ مجاز گیریده.
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
        // مرز کش باید ثابت و بی‌واژه‌بست باشد — سازنده‌ها آن را بین بلوک‌ها
        // می‌کارند؛ هر تغییر رشته، کش ارائه‌دهنده را می‌شکند.
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
                    <h1>ترازمت شما: ۵۸٬۷۰۰ میلی‌KIAN</h1>";
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
        // گذرهٔ ۲ قبل از گذرهٔ ۱ — ممنوع
        assert!(review.record_quality_verdict(true, Vec::new()).is_err());

        // گذرهٔ ۱ رد → هنوز در SpecCompliance
        assert_eq!(
            review.record_spec_verdict(false, vec!["missing CAS guard".into()]),
            ReviewStage::SpecCompliance
        );
        assert!(review.record_quality_verdict(true, Vec::new()).is_err());

        // گذرهٔ ۱ پاس → گذرهٔ ۲ باز می‌شود و تأیید می‌شود
        assert_eq!(review.record_spec_verdict(true, Vec::new()), ReviewStage::CodeQuality);
        assert!(review.record_quality_verdict(true, Vec::new()).is_ok());
        assert_eq!(review.stage(), ReviewStage::CodeQuality);

        // گذرهٔ ۲ با یافتهٔ بحرانی — رد
        let mut review2 = TwoStageReview::default();
        review2.record_spec_verdict(true, Vec::new());
        assert!(review2.record_quality_verdict(true, vec!["unwrap in prod".into()]).is_err());
    }

    #[test]
    fn strategic_compact_fires_at_50_then_every_25_or_token_threshold() {
        // قبل از ۵۰ — خاموش حتی با توکن کم
        assert!(!should_suggest_strategic_compact(49, 10_000));
        // دقیقاً ۵۰ — شعله می‌کشد
        assert!(should_suggest_strategic_compact(50, 10_000));
        // ۵۱..۷۴ — خاموش
        assert!(!should_suggest_strategic_compact(51, 10_000));
        assert!(!should_suggest_strategic_compact(74, 10_000));
        // ۷۵ و ۱۰۰ — مرزهای بعدی
        assert!(should_suggest_strategic_compact(75, 10_000));
        assert!(should_suggest_strategic_compact(100, 10_000));
        // آستانهٔ توکن مستقل از فراخوان
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
        // تقویت تا سقف 0.9
        for _ in 0..10 {
            inst.reinforce(true);
        }
        assert_eq!(inst.confidence, INSTINCT_MAX_CONFIDENCE, "capped at 0.9");
        // نقض تا کف 0.3
        for _ in 0..10 {
            inst.reinforce(false);
        }
        assert_eq!(inst.confidence, INSTINCT_MIN_CONFIDENCE, "floored at 0.3");
        // ورودی‌های خارج از بازه گیریده می‌شوند
        assert_eq!(inst.set_confidence(0.05), 0.3);
        assert_eq!(inst.set_confidence(0.99), 0.9);
        assert_eq!(inst.set_confidence(0.75), 0.75);
    }
}
