//! `mairon_rea_and_reviews` — دروازهٔ داوری زندهٔ محصول و موتور ستارهٔ ۵گانهٔ جامعه
//! (Wave 2 — Step 6).
//!
//! پورت DNA [`Mairon`] (`مایرون` — معمارِ نظمِ مطلق و داورِ ارشدِ `REA`، از
//! `KIA-BRAIN-OS/10_MAIRON_REA_SOVEREIGN.md`) و استاندارد `morluto/rea`
//! نسخهٔ `6.10.1` برای بازار کیان:
//!
//! - [`MaironReaPillarScores`] + [`MaironProductVerdict`] — حکم چهارستونیِ
//!   آهنگری بازار (۴ ستون × ۲۵ = ۱۰۰؛ عبور فقط با `>= 85` و صفر خط قرمز)؛
//! - [`evaluate_user_product_with_mairon`] — داوری قطعی محصول کاربر با
//!   گزارش فارسی دوصدایی برای سازنده، پیش از هر انتشار؛
//! - [`confirm_and_publish_product`] — دروازهٔ دو-کلیدی: انتشار فقط با
//!   هم‌زمانیِ حکمِ مایرون **و** تأییدِ خودِ سازنده؛
//! - [`ProductReview`] + [`submit_product_review`] — رأی واقعی شهروندان با
//!   ستارهٔ سخت‌سنجی‌شدهٔ `1..=5`، نظر صحیح‌سنجی‌شده و قفلِ یک‌رأییِ
//!   (شهروند، محصول) — سوریل نمی‌تواند امتیاز بسازد؛
//! - [`summarize_product_ratings`] + [`ProductRatingSummary`] — میانگین
//!   بیزی (پیشین `C = 5.0` و `m = 4.20`؛ خرید تاییدشده وزن `1.0`، بدون خرید
//!   وزن `0.25`)، هیستوگرام دقیق، نمایش پنج‌ستاره و نمرهٔ رتبهٔ ترکیبی
//!   (`0.60 * bayesian_stars * 20.0 + 0.40 * mairon_rea_score`).
//!
//! این ماژول هستهٔ خالص دامنه است: صفر IO، صفر وابستگی، صفر ساعت پنهان.
#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]

/// سقف هر ستون داوری مایرون — ۴ ستون × ۲۵ = ۱۰۰.
pub const MAX_PILLAR_SCORE: u8 = 25;

/// آستانهٔ پذیرش آهنگری بازار — زیرِ این نمره، محصول از forge بیرون نمی‌آید.
pub const MARKETPLACE_APPROVAL_THRESHOLD: u8 = 85;

/// بیشینهٔ کاراکتر نظرِ رأی شهروند (پس از trim).
pub const MAX_COMMENT_CHARS: usize = 2_000;

/// وزن پیشین بیزی — `C` در فرمول `(C*m + Σw·s) / (C + Σw)`.
pub const BAYESIAN_PRIOR_WEIGHT: f64 = 5.0;

/// میانگین پیشین بیزی — `m`؛ جهانِ بی‌رأی، محصولِ متوسطِ خوب دیده می‌شود.
pub const BAYESIAN_PRIOR_MEAN: f64 = 4.20;

/// وزن رأیِ خریدِ تاییدشده (اتصال به امانتِ تسویه‌شده).
pub const VERIFIED_REVIEW_WEIGHT: f64 = 1.0;

/// وزن رأیِ بدون خرید تاییدشده — یک‌چهارمِ رأی واقعی.
pub const UNVERIFIED_REVIEW_WEIGHT: f64 = 0.25;

/// سهم جامعه در نمرهٔ رتبهٔ ترکیبی (`bayesian_stars` نرمال‌شده به صد).
pub const RANK_STAR_WEIGHT: f64 = 0.60;

/// سهم ممیزی فنی مایرون در نمرهٔ رتبهٔ ترکیبی.
pub const RANK_REA_WEIGHT: f64 = 0.40;

// ── ۱) داوری آهنگری بازار ────────────────────────────────────────────────

/// چهار ستون داوری مایرون — هر ستون در بازهٔ `0..=25`.
///
/// ورودیِ سازنده آزاد است؛ [`MaironReaPillarScores::clamped`] هر مقدار
/// سرکش را به بازهٔ قانونی می‌برد تا `total_score` هرگز از `100` فراتر
/// نرود (قرارداد دفاعی داور).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaironReaPillarScores {
    /// معماری — مرز لایه‌ای، ماژول‌بندی، صفر نشانگر ناتمام.
    pub architecture: u8,
    /// سخت‌سازی امنیت — `SSRF`، sandbox، صفر راز هاردکد.
    pub security_hardening: u8,
    /// تجربهٔ کاربری و ضد-`AI-Slop` — پشتهٔ `Vazirmatn`، صفر سرریز `390px`.
    pub ux_and_anti_slop: u8,
    /// شواهد راستی‌آزمایی — تست سبز واقعی و اجرای مشاهده‌شده.
    pub verification_proof: u8,
}

impl MaironReaPillarScores {
    /// هر ستون را به بازهٔ `0..=25` می‌بَرد — ورودی سرکش، حکم قانونی.
    #[must_use]
    pub fn clamped(&self) -> Self {
        Self {
            architecture: self.architecture.min(MAX_PILLAR_SCORE),
            security_hardening: self.security_hardening.min(MAX_PILLAR_SCORE),
            ux_and_anti_slop: self.ux_and_anti_slop.min(MAX_PILLAR_SCORE),
            verification_proof: self.verification_proof.min(MAX_PILLAR_SCORE),
        }
    }

    /// مجموع چهار ستونِ بسته‌شده — همیشه در بازهٔ `0..=100`.
    #[must_use]
    pub fn total(&self) -> u8 {
        let c = self.clamped();
        c.architecture
            .saturating_add(c.security_hardening)
            .saturating_add(c.ux_and_anti_slop)
            .saturating_add(c.verification_proof)
    }
}

impl Default for MaironReaPillarScores {
    /// داوریِ خنثیِ صفر — سازنده باید امتیاز را بشناسد، نه هدیه بگیرد.
    fn default() -> Self {
        Self {
            architecture: 0,
            security_hardening: 0,
            ux_and_anti_slop: 0,
            verification_proof: 0,
        }
    }
}

/// حکم نهایی مایرون روی یک محصول کاربر — پیش از هر انتشار.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaironProductVerdict {
    /// شناسهٔ محصول داوری‌شده.
    pub product_id: String,
    /// نمرهٔ کل `0..=100` (مجموع چهار ستونِ بسته‌شده).
    pub total_score: u8,
    /// تفکیک چهار ستون.
    pub pillars: MaironReaPillarScores,
    /// خطوط قرمز — هر یک، حتی با نمرهٔ بالا، آهنگری را رد می‌کند.
    pub hard_failures: Vec<String>,
    /// پذیرش لیستینگ — `total_score >= 85` **و** صفر خط قرمز.
    pub approved_for_listing: bool,
    /// نشان مرئی روی لیستینگ (پذیرفته: `⚒️ Mairon REA Certified: {n}/100
    /// (Forge-Grade)`؛ ردشده: `⚠️ نیازمند بازنگری مایرون (REA < 85)`).
    pub visible_badge: String,
    /// گزارش فارسی دوصدایی (صدای مهندس + صدای داور) برای سازنده.
    pub feedback_fa: String,
}

/// لیستینگ منتشرشدهٔ محصول با مُهر مایرون — خروجی [`confirm_and_publish_product`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedProductListing {
    /// شناسهٔ محصول.
    pub product_id: String,
    /// نمرهٔ `REA` مایرون روی لیستینگ (`0..=100`).
    pub mairon_rea_score: u8,
    /// نشان مرئیِ مُهرشده.
    pub mairon_visible_badge: String,
    /// تأیید صریح سازنده — کلید دومِ دروازهٔ انتشار.
    pub creator_confirmed: bool,
}

/// داوری قطعی محصول کاربر توسط مایرون — نمرهٔ دقیق + بازخورد فارسی دوصدایی.
///
/// ستون‌ها پیش از جمع، بسته می‌شوند (`0..=25`) تا نمره از `100` نگذرد؛
/// پذیرش یعنی نمرهٔ `>= 85` **و** `hard_failures` خالی.
#[must_use]
pub fn evaluate_user_product_with_mairon(
    product_id: &str,
    pillars: MaironReaPillarScores,
    hard_failures: Vec<String>,
) -> MaironProductVerdict {
    let clamped = pillars.clamped();
    let total_score = clamped.total();
    let approved_for_listing =
        total_score >= MARKETPLACE_APPROVAL_THRESHOLD && hard_failures.is_empty();
    let visible_badge = if approved_for_listing {
        format!("⚒️ Mairon REA Certified: {total_score}/100 (Forge-Grade)")
    } else {
        "⚠️ نیازمند بازنگری مایرون (REA < 85)".to_owned()
    };
    let feedback_fa = build_dual_voice_feedback(clamped, total_score, &hard_failures, approved_for_listing);
    MaironProductVerdict {
        product_id: product_id.to_owned(),
        total_score,
        pillars: clamped,
        hard_failures,
        approved_for_listing,
        visible_badge,
        feedback_fa,
    }
}

/// گزارش فارسی دوصدایی — صدای مهندس (تفکیک ستون‌ها + مسیر اصلاح) و
/// صدای داور (حکم نهایی). امتیازِ دقیق هرگز پنهان نمی‌شود.
fn build_dual_voice_feedback(
    pillars: MaironReaPillarScores,
    total_score: u8,
    hard_failures: &[String],
    approved: bool,
) -> String {
    let weakest = [
        ("معماری", pillars.architecture),
        ("امنیت و سخت‌سازی", pillars.security_hardening),
        ("UX و ضد-اسلوپ", pillars.ux_and_anti_slop),
        ("راستی‌آزمایی", pillars.verification_proof),
    ]
    .into_iter()
    .min_by_key(|&(_, s)| s)
    .map_or("—", |(name, _)| name);
    let engineer = format!(
        "🔊 صدای مهندس مایرون — معماری: {}/25، امنیت و سخت‌سازی: {}/25، \
         UX و ضد-اسلوپ: {}/25، راستی‌آزمایی: {}/25. ضعیف‌ترین ستون: {weakest}؛ \
         اصلاح را از همان‌جا شروع کن و هر ادعا را با شاهد اجرا بیاور.",
        pillars.architecture,
        pillars.security_hardening,
        pillars.ux_and_anti_slop,
        pillars.verification_proof
    );
    let judge = if approved {
        format!(
            "⚖️ حکم داور مایرون: آهنگری پذیرفته شد — نمرهٔ نهایی {total_score}/100 \
             با صفر خط قرمز. نشان «⚒️ Mairon REA Certified» پس از تأیید خودِ \
             سازنده روی لیستینگ می‌نشیند."
        )
    } else if hard_failures.is_empty() {
        format!(
            "⚖️ حکم داور مایرون: آهنگری رد شد — نمرهٔ نهایی {total_score}/100 زیر \
             آستانهٔ {MARKETPLACE_APPROVAL_THRESHOLD}. محصول منتشر نمی‌شود؛ بازخورد \
             بالا را اعمال کن و دوباره ارائه بده."
        )
    } else {
        format!(
            "⚖️ حکم داور مایرون: آهنگری رد شد — {total_score}/100 با {} خط قرمز \
             ({hard_failures_joined}). خط قرمز با نمره بخشیده نمی‌شود؛ ابتدا \
             خطوط قرمز را صفر کن.",
            hard_failures.len(),
            hard_failures_joined = hard_failures.join("؛ ")
        )
    };
    format!("{engineer}\n{judge}")
}

/// دروازهٔ دو-کلیدی انتشار به بازار کیان.
///
/// محصول **فقط** وقتی لیست می‌شود که هم حکم مایرون پذیرش باشد و هم سازنده
/// صریحاً `creator_confirmed = true` داده باشد؛ نمره و نشان مایرون به
/// لیستینگ می‌چسبند.
///
/// # Errors
/// - حکم مایرون پذیرش نیست → `Err("Mairon REA verdict has not approved this product for listing")`.
/// - حکم پذیرش است اما سازنده تأیید نکرده → `Err("Creator confirmation is required before marketplace publication")`.
pub fn confirm_and_publish_product(
    verdict: &MaironProductVerdict,
    creator_confirmed: bool,
) -> Result<PublishedProductListing, &'static str> {
    if !verdict.approved_for_listing {
        return Err("Mairon REA verdict has not approved this product for listing");
    }
    if !creator_confirmed {
        return Err("Creator confirmation is required before marketplace publication");
    }
    Ok(PublishedProductListing {
        product_id: verdict.product_id.clone(),
        mairon_rea_score: verdict.total_score,
        mairon_visible_badge: verdict.visible_badge.clone(),
        creator_confirmed: true,
    })
}

// ── ۲) ستارهٔ ۵گانهٔ جامعه ────────────────────────────────────────────────

/// یک رأی واقعی شهروند روی محصول — سخت‌سنجی‌شده در [`submit_product_review`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductReview {
    /// شناسهٔ یکتای رأی.
    pub review_id: String,
    /// شناسهٔ محصول.
    pub product_id: String,
    /// شناسهٔ شهروند رأی‌دهنده — هر شهروند به‌ازای هر محصول فقط یک رأی.
    pub reviewer_citizen_id: String,
    /// ستاره — فقط `1..=5`.
    pub stars: u8,
    /// نظر — پس از `trim` غیرخالی و حداکثر ۲۰۰۰ کاراکتر.
    pub comment: String,
    /// خرید تاییدشده — اتصال به امانتِ تسویه‌شده؛ ضد-سوریل.
    pub verified_purchase: bool,
    /// عکسِ فوریِ نمرهٔ `REA` مایرون هنگام رأی.
    pub mairon_score_at_review: u8,
}

/// خلاصهٔ رتبهٔ یک محصول — دو لایهٔ فنی و اجتماعی کنار هم.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductRatingSummary {
    /// شناسهٔ محصول.
    pub product_id: String,
    /// نمرهٔ فنی مایرون (`0..=100`).
    pub mairon_rea_score: u8,
    /// نشان مرئی مایرون.
    pub mairon_visible_badge: String,
    /// شمار همهٔ رأی‌ها.
    pub total_reviews: u32,
    /// شمار رأی‌های خرید تاییدشده.
    pub verified_reviews: u32,
    /// میانگین حسابی ستاره‌ها (`0.0..=5.0`)؛ بی‌رأی = `0.0`.
    pub average_stars: f64,
    /// میانگین بیزی با پیشین `C = 5.0` و `m = 4.20`؛ خرید تاییدشده وزن `1.0`،
    /// بدون خرید وزن `0.25`؛ بی‌رأی = پیشین `4.20`.
    pub bayesian_stars: f64,
    /// شمار دقیق `[1★, 2★, 3★, 4★, 5★]`.
    pub star_histogram: [u32; 5],
    /// نمایش پنج‌ستارهٔ دوزبانه — رأی جامعه و نمرهٔ مایرون کنار هم.
    pub five_star_display: String,
    /// نمرهٔ رتبهٔ ترکیبی: `0.60 * (bayesian_stars * 20) + 0.40 * mairon_rea_score`.
    pub combined_rank_score: f64,
}

/// ثبت رأی شهروند — سخت‌سنجی کامل پیش از پذیرش.
///
/// # Errors
/// - ستاره بیرون از `1..=5` → `Err("Star rating must be between 1 and 5")`.
/// - نظر پس از `trim` خالی → `Err("Review comment must not be empty")`.
/// - نظر بیش از ۲۰۰۰ کاراکتر → `Err("Review comment must not exceed 2000 characters")`.
/// - رأی تکراری همان شهروند روی همان محصول →
///   `Err("Citizen has already reviewed this product")`.
#[allow(clippy::too_many_arguments)] // قرارداد مشخصهٔ رأی — هشت ورودیِ نام‌دارِ صریح
pub fn submit_product_review(
    existing_reviews: &[ProductReview],
    review_id: &str,
    product_id: &str,
    reviewer_citizen_id: &str,
    stars: u8,
    comment: &str,
    verified_purchase: bool,
    mairon_score: u8,
) -> Result<ProductReview, &'static str> {
    if !(1..=5).contains(&stars) {
        return Err("Star rating must be between 1 and 5");
    }
    let trimmed = comment.trim();
    if trimmed.is_empty() {
        return Err("Review comment must not be empty");
    }
    if trimmed.chars().count() > MAX_COMMENT_CHARS {
        return Err("Review comment must not exceed 2000 characters");
    }
    if existing_reviews
        .iter()
        .any(|r| r.product_id == product_id && r.reviewer_citizen_id == reviewer_citizen_id)
    {
        return Err("Citizen has already reviewed this product");
    }
    Ok(ProductReview {
        review_id: review_id.to_owned(),
        product_id: product_id.to_owned(),
        reviewer_citizen_id: reviewer_citizen_id.to_owned(),
        stars,
        comment: trimmed.to_owned(),
        verified_purchase,
        mairon_score_at_review: mairon_score,
    })
}

/// خلاصهٔ رتبهٔ محصول — رأی‌های همان محصول فیلتر و دو لایهٔ امتیاز با هم
/// رندر می‌شوند. مایرون نمرهٔ خودش را از حکم (`verdict`) می‌گیرد.
#[must_use]
pub fn summarize_product_ratings(
    product_id: &str,
    mairon_verdict: &MaironProductVerdict,
    reviews: &[ProductReview],
) -> ProductRatingSummary {
    let mine: Vec<&ProductReview> =
        reviews.iter().filter(|r| r.product_id == product_id).collect();
    let total_reviews = u32::try_from(mine.len()).unwrap_or(0);
    let verified_reviews = u32::try_from(mine.iter().filter(|r| r.verified_purchase).count())
        .unwrap_or(0);
    let mut star_histogram = [0u32; 5];
    let mut weighted_star_sum = 0.0_f64;
    let mut weight_sum = 0.0_f64;
    let mut raw_star_sum = 0_u64;
    for r in &mine {
        let idx = usize::from(r.stars.saturating_sub(1).min(4));
        star_histogram[idx] = star_histogram[idx].saturating_add(1);
        raw_star_sum += u64::from(r.stars);
        let weight = if r.verified_purchase {
            VERIFIED_REVIEW_WEIGHT
        } else {
            UNVERIFIED_REVIEW_WEIGHT
        };
        weighted_star_sum += weight * f64::from(r.stars);
        weight_sum += weight;
    }
    let average_stars = if mine.is_empty() {
        0.0
    } else {
        f64::from(u32::try_from(raw_star_sum).unwrap_or(0)) / f64::from(total_reviews)
    };
    let bayesian_stars = (BAYESIAN_PRIOR_WEIGHT * BAYESIAN_PRIOR_MEAN + weighted_star_sum)
        / (BAYESIAN_PRIOR_WEIGHT + weight_sum);
    let filled = bayesian_stars.round();
    let bar: String = (1_i32..=5)
        .map(|i| if f64::from(i) <= filled { "★" } else { "☆" })
        .collect();
    let mairon_rea_score = mairon_verdict.total_score;
    let five_star_display = format!(
        "{bar} {bayesian_stars:.1} / 5.0 ({verified_reviews} نظر تاییدشده) | \
         ⚒️ نمره مایرون REA: {mairon_rea_score}/100"
    );
    let combined_rank_score =
        RANK_STAR_WEIGHT * (bayesian_stars * 20.0) + RANK_REA_WEIGHT * f64::from(mairon_rea_score);
    ProductRatingSummary {
        product_id: product_id.to_owned(),
        mairon_rea_score,
        mairon_visible_badge: mairon_verdict.visible_badge.clone(),
        total_reviews,
        verified_reviews,
        average_stars,
        bayesian_stars,
        star_histogram,
        five_star_display,
        combined_rank_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ستون‌های forge-grade — نمرهٔ ۹۶، صفر خط قرمز.
    fn forge_pillars() -> MaironReaPillarScores {
        MaironReaPillarScores {
            architecture: 24,
            security_hardening: 25,
            ux_and_anti_slop: 23,
            verification_proof: 24,
        }
    }

    /// پذیرش کامل: نمره >= 85 و صفر خط قرمز → لیستینگ با نشان forge و نمرهٔ شفاف.
    #[test]
    fn approved_product_publishes_with_forge_badge() {
        let verdict = evaluate_user_product_with_mairon("prod:forge-1", forge_pillars(), vec![]);
        assert_eq!(verdict.total_score, 96);
        assert!(verdict.approved_for_listing);
        assert_eq!(
            verdict.visible_badge,
            "⚒️ Mairon REA Certified: 96/100 (Forge-Grade)"
        );
        assert!(verdict.feedback_fa.contains("آهنگری پذیرفته شد"));
        assert!(verdict.feedback_fa.contains("🔊 صدای مهندس مایرون"));
        let listing =
            confirm_and_publish_product(&verdict, true).expect("approved + confirmed");
        assert_eq!(listing.mairon_rea_score, 96);
        assert!(listing.mairon_visible_badge.contains("Forge-Grade"));
        assert!(listing.creator_confirmed);
    }

    /// رد کامل: نمره زیر ۸۵ → لیستینگ بسته با بازخورد فارسی قابل‌اقدام.
    #[test]
    fn rejected_product_blocked_with_actionable_persian_feedback() {
        let weak = MaironReaPillarScores {
            architecture: 18,
            security_hardening: 20,
            ux_and_anti_slop: 19,
            verification_proof: 21,
        };
        let verdict = evaluate_user_product_with_mairon("prod:weak-1", weak, vec![]);
        assert_eq!(verdict.total_score, 78);
        assert!(!verdict.approved_for_listing);
        assert_eq!(verdict.visible_badge, "⚠️ نیازمند بازنگری مایرون (REA < 85)");
        assert!(verdict.feedback_fa.contains("آهنگری رد شد"));
        assert!(verdict.feedback_fa.contains("آستانهٔ 85"));
        assert!(verdict.feedback_fa.contains("ضعیف‌ترین ستون: معماری"));
        let err = confirm_and_publish_product(&verdict, true)
            .expect_err("rejected must not publish");
        assert!(err.contains("not approved"));
    }

    /// خط قرمز با نمرهٔ بالا بخشیده نمی‌شود — حتی ۹۸/۱۰۰ هم رد است.
    #[test]
    fn hard_failure_blocks_even_with_high_score() {
        let verdict = evaluate_user_product_with_mairon(
            "prod:leaky-1",
            MaironReaPillarScores {
                architecture: 25,
                security_hardening: 25,
                ux_and_anti_slop: 24,
                verification_proof: 24,
            },
            vec!["hardcoded secret detected in src/config.rs".to_owned()],
        );
        assert_eq!(verdict.total_score, 98);
        assert!(!verdict.approved_for_listing, "خط قرمز امنیتی را رد می‌کند");
        assert!(verdict.feedback_fa.contains("خط قرمز"));
        assert!(verdict.feedback_fa.contains("hardcoded secret"));
    }

    /// کلید دوم دروازهٔ انتشار: بدون تأیید سازنده، نمرهٔ ۹۵ هم منتشر نمی‌شود.
    #[test]
    fn creator_confirmation_is_mandatory_even_at_perfect_score() {
        let perfect = MaironReaPillarScores {
            architecture: 25,
            security_hardening: 25,
            ux_and_anti_slop: 25,
            verification_proof: 20,
        };
        let verdict = evaluate_user_product_with_mairon("prod:shy-1", perfect, vec![]);
        assert_eq!(verdict.total_score, 95);
        assert!(verdict.approved_for_listing);
        let err = confirm_and_publish_product(&verdict, false)
            .expect_err("unconfirmed must not publish");
        assert_eq!(
            err,
            "Creator confirmation is required before marketplace publication"
        );
    }

    /// ستارهٔ سخت‌سنجی: `0` و `6` رد، `1` و `5` پذیرفته — پیام قراردادی دقیق.
    #[test]
    fn stars_outside_range_rejected_and_valid_range_accepted() {
        let base = |s: u8| {
            submit_product_review(&[], "rv:x", "prod:1", "citizen:1", s, "سخت و تمیز", true, 96)
        };
        assert_eq!(base(0).expect_err("zero stars"), "Star rating must be between 1 and 5");
        assert_eq!(base(6).expect_err("six stars"), "Star rating must be between 1 and 5");
        assert!(base(1).is_ok());
        assert!(base(5).is_ok());
    }

    /// نظر خالی (حتی whitespace) رد می‌شود.
    #[test]
    fn empty_comment_rejected() {
        let err = submit_product_review(&[], "rv:e", "prod:1", "citizen:1", 5, "   ", false, 90)
            .expect_err("blank comment");
        assert_eq!(err, "Review comment must not be empty");
    }

    /// نظر بیش از ۲۰۰۰ کاراکتر رد می‌شود.
    #[test]
    fn oversized_comment_rejected() {
        let huge = "د".repeat(2_001);
        let err = submit_product_review(&[], "rv:h", "prod:1", "citizen:1", 5, &huge, false, 90)
            .expect_err("oversized comment");
        assert_eq!(err, "Review comment must not exceed 2000 characters");
        let exact = "د".repeat(2_000);
        assert!(submit_product_review(&[], "rv:ok", "prod:1", "citizen:1", 5, &exact, false, 90).is_ok());
    }

    /// یک رأی به‌ازای هر (شهروند، محصول) — سوریل راهی ندارد.
    #[test]
    fn duplicate_review_per_citizen_per_product_rejected() {
        let first = submit_product_review(
            &[],
            "rv:1",
            "prod:1",
            "citizen:ada",
            5,
            "رویایی",
            true,
            96,
        )
        .expect("first review");
        let dup_same_product = submit_product_review(
            std::slice::from_ref(&first),
            "rv:2",
            "prod:1",
            "citizen:ada",
            1,
            "عوضش دادم",
            true,
            96,
        )
        .expect_err("duplicate (product, citizen)");
        assert_eq!(dup_same_product, "Citizen has already reviewed this product");
        // همان شهروند روی محصولِ دیگر آزاد است.
        assert!(submit_product_review(
            std::slice::from_ref(&first),
            "rv:3",
            "prod:2",
            "citizen:ada",
            4,
            "محصول دیگر",
            false,
            88,
        )
        .is_ok());
    }

    /// هیستوگرام و میانگین حسابی دقیق‌اند.
    #[test]
    fn histogram_and_average_are_exact() {
        let verdict = evaluate_user_product_with_mairon("prod:hist", forge_pillars(), vec![]);
        let mk = |rv: &str, s: u8, v: bool| {
            submit_product_review(&[], rv, "prod:hist", rv, s, "رأی", v, 96).expect("review")
        };
        let reviews = [mk("a", 5, true), mk("b", 1, false), mk("c", 4, true)];
        let sum = summarize_product_ratings("prod:hist", &verdict, &reviews);
        assert_eq!(sum.total_reviews, 3);
        assert_eq!(sum.verified_reviews, 2);
        assert_eq!(sum.star_histogram, [1, 0, 0, 1, 1]);
        assert!((sum.average_stars - 10.0 / 3.0).abs() < 1e-9);
    }

    /// اثبات بیزی: خریدِ تاییدشده وزن کامل `1.0` دارد و رأی بدون خرید فقط
    /// `0.25` — همان ستاره‌ها با رأیِ بی‌خرید، نزدیکِ پیشینِ `4.20` می‌مانند.
    #[test]
    fn bayesian_weighting_proves_verified_beats_unverified() {
        let verdict = evaluate_user_product_with_mairon("prod:bayes", forge_pillars(), vec![]);
        let mk = |rv: &str, v: bool| {
            submit_product_review(&[], rv, "prod:bayes", rv, 4, "چهار ستاره", v, 96)
                .expect("review")
        };
        let all_verified = [mk("v1", true)];
        let all_unverified = [mk("u1", false)];
        let verified_summary =
            summarize_product_ratings("prod:bayes", &verdict, &all_verified);
        let unverified_summary =
            summarize_product_ratings("prod:bayes", &verdict, &all_unverified);
        // verified: (5*4.2 + 1.0*4) / (5 + 1) = 25/6 ≈ 4.1667 — به رأیِ واقعیِ ۴ می‌لغزد.
        assert!((verified_summary.bayesian_stars - 25.0 / 6.0).abs() < 1e-9);
        // unverified: (5*4.2 + 0.25*4) / (5 + 0.25) = 22/5.25 ≈ 4.1905 — به پیشینِ 4.2 می‌چسبد.
        assert!((unverified_summary.bayesian_stars - 22.0 / 5.25).abs() < 1e-9);
        assert!(
            verified_summary.bayesian_stars < unverified_summary.bayesian_stars,
            "وزن کاملِ خرید تاییدشده، میانگین را سریع‌تر به رأی واقعی می‌کشد"
        );
        // بی‌رأی — دقیقاً پیشین.
        let empty = summarize_product_ratings("prod:bayes", &verdict, &[]);
        assert!((empty.bayesian_stars - BAYESIAN_PRIOR_MEAN).abs() < 1e-9);
        assert!((empty.average_stars).abs() < 1e-9);
        assert_eq!(empty.total_reviews, 0);
    }

    /// نمایش پنج‌ستاره — رأی جامعه و نمرهٔ مایرون در یک رشته کنار هم.
    #[test]
    fn five_star_display_shows_rating_and_mairon_score_side_by_side() {
        let verdict = evaluate_user_product_with_mairon("prod:disp", forge_pillars(), vec![]);
        let r1 = submit_product_review(&[], "rv:1", "prod:disp", "c1", 5, "عالی", true, 96)
            .expect("r1");
        let r2 = submit_product_review(
            std::slice::from_ref(&r1),
            "rv:2",
            "prod:disp",
            "c2",
            4,
            "خوب",
            true,
            96,
        )
        .expect("r2");
        let sum = summarize_product_ratings("prod:disp", &verdict, &[r1, r2]);
        // (5*4.2 + 5 + 4) / 7 = 30/7 ≈ 4.2857 → گرد به ۴ ستارهٔ پر.
        assert!(sum.five_star_display.starts_with("★★★★☆"));
        assert!(sum.five_star_display.contains("4.3 / 5.0"));
        assert!(sum.five_star_display.contains("(2 نظر تاییدشده)"));
        assert!(sum.five_star_display.contains("⚒️ نمره مایرون REA: 96/100"));
    }

    /// نمرهٔ رتبهٔ ترکیبی — فرمول دقیق `0.60 * (bayes*20) + 0.40 * REA`.
    #[test]
    fn combined_rank_score_blends_bayesian_and_rea() {
        let verdict = evaluate_user_product_with_mairon("prod:rank", forge_pillars(), vec![]);
        let empty = summarize_product_ratings("prod:rank", &verdict, &[]);
        let expected = 0.60 * (BAYESIAN_PRIOR_MEAN * 20.0) + 0.40 * f64::from(96_u8);
        assert!((empty.combined_rank_score - expected).abs() < 1e-9);
        assert!(empty.combined_rank_score > 85.0, "محصول forge-grade صدر می‌ماند");
    }

    /// ستون‌های سرکش بسته می‌شوند — نمره هرگز از ۱۰۰ فراتر نمی‌رود.
    #[test]
    fn pillar_scores_are_clamped_to_25() {
        let wild = MaironReaPillarScores {
            architecture: 30,
            security_hardening: 200,
            ux_and_anti_slop: 25,
            verification_proof: 99,
        };
        let verdict = evaluate_user_product_with_mairon("prod:wild", wild, vec![]);
        assert_eq!(verdict.pillars.architecture, 25);
        assert_eq!(verdict.pillars.security_hardening, 25);
        assert_eq!(verdict.pillars.ux_and_anti_slop, 25);
        assert_eq!(verdict.pillars.verification_proof, 25);
        assert_eq!(verdict.total_score, 100);
        assert!(verdict.approved_for_listing);
    }
}
