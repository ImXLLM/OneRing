//! رتبه‌سنج وایرال بازار کیان — پورت دقیق `xai-org/x-algorithm` (commit 269340c).
//!
//! منابع پورت:
//! - `home-mixer/params/param.rs` — وزن‌های دقیق سیگنال تعامل (کل فایل، ۲۰۲۶)
//! - `home-mixer/scorers/author_cold_start.rs` — بوت‌استرپ سازندگان تازه‌کار
//! - `xai-value-model/scoring.rs` — فرمول `fuse_heads` (جمع وزن × احتمال)
//!
//! قانون طلایی الگوریتم X: وزن‌ها روی **احتمال پیش‌بینی‌شدهٔ** هر تعامل ضرب
//! می‌شوند، نه روی شمارش خام. `ShareViaCopyLink = +20.0` قوی‌ترین سیگنال
//! مثبت است (حدود ۲۸ برابر favorite=0.708) و `Report = -234.0` قوی‌ترین
//! سیگنال منفی — یک گزارش، ۳۰ لایک مثبت را باطل می‌کند.
//!
//! بوت‌استرپ سرد (`ColdStartPolicy`): محصول با سن ≤ ۷۲۰۰ ثانیه (۲ ساعت)،
//! سازنده با < ۵۰٬۰۰۰ فالوور و < ۲۰۰ ایمپرشن، واجد تزریق به اسلات‌های
//! کشف ۱۵..=۱۶ می‌شود — نمونه‌گیری Thompson روی `Beta(0.75, 49.25)`
//! (میانگین ۰٫۰۱۵) + پاداش اکتشافی قطعی (`SplitMix64` روی شناسهٔ محصول)،
//! بدون هیچ RNG غیرقطعی.
#![forbid(unsafe_code)]

/// وزن‌های دقیق تعامل — پورت خط‌به‌خط `home-mixer/params/param.rs`.
///
/// مقادیر منفی جریمه‌اند و هرگز به‌صورت پاداش درخشش نمی‌گیرند؛ یک سیگنال
/// منفی بزرگ (گزارش/مسدودسازی) رتبهٔ محصول را زیر صفر می‌برد.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViralEngagementWeights {
    /// `share_via_copy_link = 20.0` — قوی‌ترین سیگنال مثبت (کپی لینک محصول).
    pub share_via_copy_link: f64,
    /// `reply = 5.0`.
    pub reply: f64,
    /// `bidirectional_reply_boost = 15.0` — اگر طرفین به هم پاسخ داده باشند
    /// (مجموع پاسخ دوسویه = ۲۰٫۰، هم‌تراز کپی‌لینک).
    pub bidirectional_reply_boost: f64,
    /// `quote = 5.0` — ریمیکس/کوت محصول.
    pub quote: f64,
    /// `share_via_dm = 5.0`.
    pub share_via_dm: f64,
    /// `follow_creator = 4.0`.
    pub follow_creator: f64,
    /// `share = 2.118`.
    pub share: f64,
    /// `repost = 1.0` (`retweet_weight`).
    pub repost: f64,
    /// `favorite = 0.708`.
    pub favorite: f64,
    /// `cont_click_dwell_per_sec = 0.385` — ثانیه‌های توقف پس از کلیک.
    pub cont_click_dwell_per_sec: f64,
    /// `click = 0.274`.
    pub click: f64,
    /// `open_link = 0.2`.
    pub open_link: f64,
    /// `dwell = 0.05` — توقف سادهٔ اسکرول.
    pub dwell: f64,
    /// `not_dwelled = -0.02` — دید ولی نخواند.
    pub not_dwelled: f64,
    /// `block_creator = -31.2`.
    pub block_creator: f64,
    /// `not_interested = -53.07`.
    pub not_interested: f64,
    /// `mute_creator = -58.8`.
    pub mute_creator: f64,
    /// `report = -234.0` — قوی‌ترین سیگنال منفی.
    pub report: f64,
}

impl Default for ViralEngagementWeights {
    fn default() -> Self {
        Self {
            share_via_copy_link: 20.0,
            reply: 5.0,
            bidirectional_reply_boost: 15.0,
            quote: 5.0,
            share_via_dm: 5.0,
            follow_creator: 4.0,
            share: 2.118,
            repost: 1.0,
            favorite: 0.708,
            cont_click_dwell_per_sec: 0.385,
            click: 0.274,
            open_link: 0.2,
            dwell: 0.05,
            not_dwelled: -0.02,
            block_creator: -31.2,
            not_interested: -53.07,
            mute_creator: -58.8,
            report: -234.0,
        }
    }
}

/// احتمال‌های پیش‌بینی‌شدهٔ تعامل برای یک محصول بازار (`CandidateSignals`).
///
/// هر فیلد ∈ [0, 1] است جز `cont_click_dwell_secs` و `dwell_secs` که
/// «ثانیهٔ مورد انتظار»اند (میانگین احتمال × ثانیه).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ProductSignals {
    pub share_via_copy_link: f64,
    /// پاسخ ساده؛ `bidirectional_reply` آن را با پاداش ۱۵ جمع می‌کند.
    pub reply: f64,
    /// پاسخ دوسویه (خریدار و سازنده به هم جواب داده‌اند)؟
    pub bidirectional_reply: bool,
    pub quote: f64,
    pub share_via_dm: f64,
    pub follow_creator: f64,
    pub share: f64,
    pub repost: f64,
    pub favorite: f64,
    pub cont_click_dwell_secs: f64,
    pub click: f64,
    pub open_link: f64,
    pub dwell_secs: f64,
    pub not_dwelled: f64,
    pub block_creator: f64,
    pub not_interested: f64,
    pub mute_creator: f64,
    pub report: f64,
}

/// فرمول فیوژن `xai-value-model/scoring.rs::fuse_heads` — جمع وزن × سیگنال.
///
/// پاسخ دوسویه هم‌ارز `reply + bidirectional_reply_boost = 20.0` حساب
/// می‌شود؛ منفی‌ها همیشه کسر می‌شوند. خروجی می‌تواند منفی باشد — محصول
/// گزارش‌شده عملاً از کشف حذف می‌شود.
#[must_use]
pub fn fuse_score(w: &ViralEngagementWeights, s: &ProductSignals) -> f64 {
    let reply_w = if s.bidirectional_reply {
        w.reply + w.bidirectional_reply_boost
    } else {
        w.reply
    };
    s.share_via_copy_link * w.share_via_copy_link
        + s.reply * reply_w
        + s.quote * w.quote
        + s.share_via_dm * w.share_via_dm
        + s.follow_creator * w.follow_creator
        + s.share * w.share
        + s.repost * w.repost
        + s.favorite * w.favorite
        + s.cont_click_dwell_secs * w.cont_click_dwell_per_sec
        + s.click * w.click
        + s.open_link * w.open_link
        + s.dwell_secs * w.dwell
        + s.not_dwelled * w.not_dwelled
        + s.block_creator * w.block_creator
        + s.not_interested * w.not_interested
        + s.mute_creator * w.mute_creator
        + s.report * w.report
}

/// امتیاز فیوژن با وزن‌های پیش‌فرض الگوریتم X ۲۰۲۶.
#[must_use]
pub fn score_with_defaults(s: &ProductSignals) -> f64 {
    fuse_score(&ViralEngagementWeights::default(), s)
}

// ── Cold-Start (پورت `author_cold_start.rs`) ────────────────────────────────

/// سیاست بوت‌استرپ سرد سازنده — دقیقاً مطابق پارامترهای تست upstream:
/// `max_post_age=7200`، `beta_alpha0=0.75`، `beta_beta0=49.25`.
///
/// محصولِ واجدِ شرایط به اسلات‌های کشف `slot_min..=slot_max` (پیش‌فرض
/// ۱۵..=۱۶) تزریق می‌شود؛ آیتم‌های جابه‌جاشده **حذف نمی‌شوند**، فقط یک
/// پله پایین می‌روند (صداقت رتبه‌بندی).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColdStartPolicy {
    /// سازنده با ≥ این فالوور، دیگر تازه‌کار نیست.
    pub max_creator_followers: u64,
    /// سن محصول ≤ این مقدار (۷۲۰۰ ثانیه = ۲ ساعت).
    pub max_product_age_secs: i64,
    /// ایمپرشن ≥ این مقدار، دیگر سرد نیست.
    pub max_impressions: u64,
    /// نخستین اسلات کشف (۰-مبنا) — ۱۵.
    pub slot_min: usize,
    /// آخرین اسلات کشف (۰-مبنا) — ۱۶.
    pub slot_max: usize,
    /// `ColdStartBetaAlpha0 = 0.75`.
    pub beta_alpha0: f64,
    /// `ColdStartBetaBeta0 = 49.25`.
    pub beta_beta0: f64,
    /// دامنهٔ پاداش اکتشافی قطعی ∈ [0, این مقدار).
    pub exploration_bonus: f64,
}

impl Default for ColdStartPolicy {
    fn default() -> Self {
        Self {
            max_creator_followers: 50_000,
            max_product_age_secs: 7_200,
            max_impressions: 200,
            slot_min: 15,
            slot_max: 16,
            beta_alpha0: 0.75,
            beta_beta0: 49.25,
            exploration_bonus: 0.01,
        }
    }
}

/// محصول تازه‌کار نامزد اسلات کشف.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColdStartCandidate {
    pub product_id: u64,
    pub creator_followers: u64,
    pub product_age_secs: i64,
    pub impressions: u64,
}

impl ColdStartPolicy {
    /// واجد شرایط بوت‌استرپ سرد؟ (هیچ‌کدام از سه شرط را نباید از دست بدهد)
    #[must_use]
    pub fn is_eligible(&self, c: &ColdStartCandidate) -> bool {
        c.creator_followers < self.max_creator_followers
            && c.product_age_secs <= self.max_product_age_secs
            && c.impressions < self.max_impressions
    }

    /// میانگین توزیع پیشین `Beta(alpha0, beta0)` — مقدار قطعی ۰٫۰۱۵.
    #[must_use]
    pub fn beta_prior_mean(&self) -> f64 {
        self.beta_alpha0 / (self.beta_alpha0 + self.beta_beta0)
    }

    /// امتیاز اولویت قطعی: میانگین `Beta` + پاداش اکتشافی `SplitMix64`.
    ///
    /// Upstream از نمونه‌گیری Thompson (`Beta.sample(rng)`) استفاده می‌کند؛
    /// نسخهٔ کیان **قطعی** است تا رتبه‌بندی بازتولیدپذیر بماند — همان
    /// میانگین `0.015` به‌عنوان پایه و یک لرزش اکتشافی ∈ [0, 0.01) مشتق
    /// از شناسهٔ محصول، طوری که کاندیداهای هم‌ارز مساوی نمانند.
    #[must_use]
    pub fn priority_score(&self, c: &ColdStartCandidate) -> f64 {
        self.beta_prior_mean() + self.exploration_bonus * splitmix64_unit(c.product_id)
    }
}

/// `SplitMix64` — مولد شبه‌تصادفی قطعی و بدونوابسته (پاداش اکتشافی).
// خروجی >> 11 دقیقاً ۵۳ بیت معنادار دارد؛ تبدیل به f64 بدون خطاست اما
// clippy نمی‌تواند این را اثبات کند — اجازهٔ هدفمند طبق قرارداد ledger.rs.
#[allow(clippy::cast_precision_loss)]
fn splitmix64_unit(seed: u64) -> f64 {
    // >> 11 → عدد صحیح ۵۳ بیتی؛ تقسیم بر 2^53 → ∈ [0, 1)
    ((splitmix64(seed) >> 11) as f64) / ((1u64 << 53) as f64)
}

fn splitmix64(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// یک ردیف رتبه‌بندی‌شدهٔ بازار — ورودی/خروجی تزریق سرد.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RankedProduct {
    pub product_id: u64,
    /// امتیاز فیوژن نهایی (`fuse_score`).
    pub score: f64,
}

/// تزریق قطعی کاندیداهای سرد به اسلات‌های کشف `slot_min..=slot_max`.
///
/// * فقط کاندیداهای واجد شرایط، مرتبِ نزولیِ `priority_score`، در نظر
///   گرفته می‌شوند (تا `slot_max - slot_min + 1` مورد).
/// * لیست ورودی باید نزولیِ امتیاز باشد (قرارداد caller پس از مرتب‌سازی).
/// * طول خروجی = ورودی + تعداد تزریق‌ها — هیچ محصولی حذف نمی‌شود.
#[must_use]
pub fn inject_cold_start(
    policy: &ColdStartPolicy,
    ranked: Vec<RankedProduct>,
    candidates: &[ColdStartCandidate],
) -> Vec<RankedProduct> {
    let mut eligible: Vec<(ColdStartCandidate, f64)> = candidates
        .iter()
        .filter(|c| policy.is_eligible(c))
        .map(|c| (*c, policy.priority_score(c)))
        .collect();
    // نزولی اولویت؛ گرهٔ تساوی با شناسهٔ کوچک‌تر قطعی می‌شود تا مرتب‌سازی بازتولیدپذیر بماند.
    eligible.sort_by(|a, b| {
        b.1.total_cmp(&a.1).then_with(|| a.0.product_id.cmp(&b.0.product_id))
    });

    let slot_count = policy.slot_max.saturating_sub(policy.slot_min) + 1;
    let picked: Vec<RankedProduct> = eligible
        .into_iter()
        .take(slot_count)
        .map(|(c, priority)| RankedProduct { product_id: c.product_id, score: priority })
        .collect();

    let mut out = ranked;
    for (offset, injected) in picked.into_iter().enumerate() {
        let idx = policy.slot_min + offset;
        if idx <= out.len() {
            out.insert(idx, injected);
        } else {
            // لیست کوتاه‌تر از اسلات کشف است — در انتها می‌آید، نه حذف.
            out.push(injected);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weights() -> ViralEngagementWeights {
        ViralEngagementWeights::default()
    }

    /// وزن‌های پورت‌شده دقیقاً با `param.rs` برابرند.
    #[test]
    fn weights_match_upstream_param_rs() {
        let w = weights();
        assert_eq!(w.share_via_copy_link, 20.0);
        assert_eq!(w.reply, 5.0);
        assert_eq!(w.bidirectional_reply_boost, 15.0);
        assert_eq!(w.quote, 5.0);
        assert_eq!(w.share_via_dm, 5.0);
        assert_eq!(w.follow_creator, 4.0);
        assert_eq!(w.share, 2.118);
        assert_eq!(w.repost, 1.0);
        assert_eq!(w.favorite, 0.708);
        assert_eq!(w.cont_click_dwell_per_sec, 0.385);
        assert_eq!(w.click, 0.274);
        assert_eq!(w.open_link, 0.2);
        assert_eq!(w.dwell, 0.05);
        assert_eq!(w.not_dwelled, -0.02);
        assert_eq!(w.block_creator, -31.2);
        assert_eq!(w.not_interested, -53.07);
        assert_eq!(w.mute_creator, -58.8);
        assert_eq!(w.report, -234.0);
    }

    /// غلبهٔ کپی‌لینک: +20.0 و حدود ۲۸ برابرِ favorite=0.708.
    #[test]
    fn copy_link_dominance_is_28x_over_favorite() {
        let mut copy_sig = ProductSignals::default();
        copy_sig.share_via_copy_link = 1.0;
        let mut fav_sig = ProductSignals::default();
        fav_sig.favorite = 1.0;

        let copy = score_with_defaults(&copy_sig);
        let fav = score_with_defaults(&fav_sig);
        assert_eq!(copy, 20.0);
        assert_eq!(fav, 0.708);
        let ratio = copy / fav;
        assert!(
            (27.0..29.0).contains(&ratio),
            "copy-link must be ~28x favorite, got {ratio}"
        );
        // پاسخ دوسویه (۲۰٫۰) هم‌تراز کپی‌لینک و بالاتر از هر سیگنال تکی دیگر.
        let mut mutual = ProductSignals::default();
        mutual.reply = 1.0;
        mutual.bidirectional_reply = true;
        assert_eq!(score_with_defaults(&mutual), 20.0);
    }

    /// سرکوب جریمهٔ منفی: یک گزارش، محصول را زیر صفر می‌برد و پایین‌تر از
    /// محصولِ بی‌سیگنالِ تمیز می‌نشاند — حتی اگر لایک و کپی‌لینک هم داشته باشد.
    #[test]
    fn negative_penalty_suppresses_reported_product() {
        let mut reported = ProductSignals::default();
        reported.share_via_copy_link = 0.4; // +8.0
        reported.favorite = 0.5; // +0.354
        reported.report = 0.05; // -11.7
        let s_reported = score_with_defaults(&reported);
        assert!(s_reported < 0.0, "report must drag below zero: {s_reported}");

        let mut clean = ProductSignals::default();
        clean.favorite = 0.1; // +0.0708
        let s_clean = score_with_defaults(&clean);
        assert!(s_clean > 0.0);
        assert!(
            s_clean > s_reported,
            "clean product must outrank reported one: {s_clean} vs {s_reported}"
        );

        // ترتیب قاطعیت جریمه‌ها: report(-234) > mute(-58.8) > not_interested(-53.07)
        // > block(-31.2) > not_dwelled(-0.02)
        let mut base = ProductSignals::default();
        base.report = 1.0;
        let r = score_with_defaults(&base);
        base.report = 0.0;
        base.mute_creator = 1.0;
        let m = score_with_defaults(&base);
        base.mute_creator = 0.0;
        base.not_interested = 1.0;
        let ni = score_with_defaults(&base);
        base.not_interested = 0.0;
        base.block_creator = 1.0;
        let b = score_with_defaults(&base);
        base.block_creator = 0.0;
        base.not_dwelled = 1.0;
        let nd = score_with_defaults(&base);
        assert!(r < m && m < ni && ni < b && b < nd);
    }

    /// واجد شرایط بودن سرد: مرزهای دقیق <50k فالوور، ≤7200s، <200 ایمپرشن.
    #[test]
    fn cold_start_eligibility_boundaries() {
        let policy = ColdStartPolicy::default();
        let ok = ColdStartCandidate {
            product_id: 1,
            creator_followers: 49_999,
            product_age_secs: 7_200,
            impressions: 199,
        };
        assert!(policy.is_eligible(&ok));

        let too_famous = ColdStartCandidate { creator_followers: 50_000, ..ok };
        assert!(!policy.is_eligible(&too_famous));
        let too_old = ColdStartCandidate { product_age_secs: 7_201, ..ok };
        assert!(!policy.is_eligible(&too_old));
        let too_seen = ColdStartCandidate { impressions: 200, ..ok };
        assert!(!policy.is_eligible(&too_seen));
    }

    /// میانگین پیشین Beta(0.75, 49.25) دقیقاً ۰٫۰۱۵ و امتیاز اولویت قطعی.
    #[test]
    fn cold_start_priority_is_deterministic_and_bounded() {
        let policy = ColdStartPolicy::default();
        assert!((policy.beta_prior_mean() - 0.015).abs() < 1e-12);

        let c = ColdStartCandidate {
            product_id: 42,
            creator_followers: 10,
            product_age_secs: 100,
            impressions: 5,
        };
        let s1 = policy.priority_score(&c);
        let s2 = policy.priority_score(&c);
        assert_eq!(s1, s2, "priority must be deterministic");
        assert!(s1 >= 0.015, "score ≥ Beta mean");
        assert!(
            s1 < 0.015 + 0.01,
            "score < mean + exploration bonus ceiling"
        );

        // دو محصول متفاوت — لرزش اکتشافی آن‌ها را جدا می‌کند (تقریباً همیشه).
        let mut distinct = 0;
        for id in 1..64u64 {
            let other = ColdStartCandidate { product_id: id, ..c };
            if policy.priority_score(&other) != s1 {
                distinct += 1;
            }
        }
        assert!(distinct > 32, "exploration bonus must spread scores");
    }

    /// تزریق سرد: دو برندهٔ برتر (بالاترین اولویت قطعی) دقیقاً در اسلات‌های
    /// ۱۵ و ۱۶ می‌نشینند و هیچ آیتم رتبه‌بندی‌شده‌ای حذف نمی‌شود.
    #[test]
    #[allow(clippy::cast_precision_loss)] // فهرست ۳۰تایی نزولی امتیاز — دقت دوگانهٔ کافی
    fn cold_start_slot_injection_15_and_16() {
        let policy = ColdStartPolicy::default();
        let ranked: Vec<RankedProduct> = (0u64..30)
            .map(|i| RankedProduct {
                product_id: 1_000 + i,
                score: 100.0 - i as f64,
            })
            .collect();

        let base = ColdStartCandidate {
            product_id: 0,
            creator_followers: 0,
            product_age_secs: 60,
            impressions: 0,
        };
        let candidates = [
            ColdStartCandidate { product_id: 500, ..base },
            ColdStartCandidate { product_id: 700, ..base },
            ColdStartCandidate { product_id: 900, ..base }, // فقط ۲ اسلات هست
            ColdStartCandidate { product_id: 999, creator_followers: 500_000, ..base }, // معروف
        ];

        // انتظار = دو کاندیدای واجدِ شرایط با بالاترین اولویت قطعی
        // (همان گره‌گشاییِ تزریق: نزولی امتیاز، سپس نزولی‌نبودن بر شناسهٔ کوچک‌تر).
        let mut priorities: Vec<(f64, u64)> = [500u64, 700, 900]
            .map(|id| {
                (
                    policy.priority_score(&ColdStartCandidate { product_id: id, ..base }),
                    id,
                )
            })
            .to_vec();
        priorities
            .sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

        let out = inject_cold_start(&policy, ranked, &candidates);
        assert_eq!(out.len(), 32, "30 ranked + 2 injected (only 2 slots)");
        assert_eq!(out[15].product_id, priorities[0].1, "top cold-start at slot 15");
        assert_eq!(out[15].score, priorities[0].0);
        assert_eq!(out[16].product_id, priorities[1].1, "second cold-start at slot 16");
        assert_eq!(out[16].score, priorities[1].0);
        assert!(
            out.iter().all(|p| p.product_id != 999),
            "famous creator must never be injected"
        );
        // ترتیب نسبی آیتم‌های اصلی حفظ شده — فقط یک پله جابه‌جایی.
        let originals: Vec<u64> = out
            .iter()
            .filter(|p| p.product_id >= 1_000)
            .map(|p| p.product_id)
            .collect();
        let expected: Vec<u64> = (0u64..30).map(|i| 1_000 + i).collect();
        assert_eq!(originals, expected);
    }

    /// لیست کوتاه‌تر از اسلات کشف — تزریق در انتها، بدون هراس.
    #[test]
    fn injection_handles_short_lists() {
        let policy = ColdStartPolicy::default();
        let ranked = vec![RankedProduct { product_id: 1, score: 5.0 }];
        let candidates = [ColdStartCandidate {
            product_id: 2,
            creator_followers: 0,
            product_age_secs: 60,
            impressions: 0,
        }];
        let out = inject_cold_start(&policy, ranked, &candidates);
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].product_id, 2, "falls to the end when list < slot");
    }

    /// لرزش اکتشافی باید صفر نباشد و در بازهٔ مجاز بماند — اگر SplitMix64
    /// خراب شود، همهٔ کاندیدها مساوی می‌شوند و اکتشاف می‌میرد.
    #[test]
    fn exploration_bonus_is_real_and_bounded() {
        let policy = ColdStartPolicy::default();
        let c = ColdStartCandidate {
            product_id: 7,
            creator_followers: 0,
            product_age_secs: 60,
            impressions: 0,
        };
        let score = policy.priority_score(&c);
        let bonus = score - policy.beta_prior_mean();
        assert!(bonus > 0.0, "bonus must be strictly positive for a nonzero id");
        assert!(bonus < policy.exploration_bonus);
    }
}
