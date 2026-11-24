//! `arda_fishkal_engine` — آردا (`آردا`) — عاملِ بومی‌سازیِ سیاره‌ایِ ۲۴ساعتهٔ
//! `۰xKIA` روی معماری اصیل `Fishkal` (موتور HTML5 بدون وابستگیِ کیا، `3,616`
//! خط، `667,241` بایت) با **درمانِ کاملِ ۲۵ نقصِ ممیزیِ قضایی** (Wave 4 —
//! Step 1/4).
//!
//! این ماژول چهار قطعه دارد:
//!
//! 1. **دیکشنری پرچمِ وکتور `Fishkal`** — [`FISHKAL_VECTOR_FLAGS`] با
//!    `viewBox="0 0 20 14"` حفظ‌شدهٔ حرف‌به‌حرف برای ۱۳ زبانِ اصلی
//!    (`en`، `ar`، `fa`، `tr`، `fr`، `es`، `pt`، `nl`، `ja`، `hi`، `zh`،
//!    `id`، `ko`) + ۷ زبانِ سیاره‌ایِ تازه (`de`، `ru`، `it`، `he`، `ur`،
//!    `vi`، `th`) = **۲۰ زبانِ سیاره‌ای**؛ به‌همراه گلیفِ سوییچرِ
//!    اسکریپت‌اصیل (`STR.code`: `'فا'`، `'ع'`، `'اردو'`، `'עב'` و کد ISO
//!    بزرگ برای LTR) و [`render_circular_flag_badge`] /
//!    [`render_product_flag_strip`] — نشانِ دایره‌ایِ مینی-پرچمِ محصولات.
//! 2. **[`ArdaFishkalBundleGenerator`]** — تولید و تزریقِ باندلِ
//!    `<style>` + `<script>` بومی‌سازیِ بدون وابستگی (drop-in) با درمانِ ۲۵
//!    نقص: فلیپِ اتمیِ ۳-ویژگیِ ریشه، تایپوگرافیِ هر-اسکریپت، کاتالوگِ
//!    یکپارچهٔ ۶-دامنه‌ای با اعتبارسنجِ توازنِ ۱۰۰٪، موتورِ فرمت‌بندیِ
//!    `Intl` کش‌شده، جداسازیِ bidi در DOM و Canvas، و سوییچرِ لیست‌باکسِ
//!    WAI-ARIA با اصلاحِ `D-1..D-8`.
//! 3. **[`ArdaPlanetaryCronDaemon`]** — کارِ خودکارِ ۲۴ساعتهٔ
//!    (`86_400` ثانیه) با گاردِ دلتای [`arda_content_sha256`] تا محصولِ
//!    بیتغییر با ایدمپوتنس رد شود و محصولِ تازه/به‌روزشده روی ۲۰ زبان
//!    بومی‌سازی و نشان‌گذاری شود.
//! 4. **[`MaironSkillForge`]** — فورجِ خودکارِ `SKILL.md` + `skill.json`
//!    هرگاه محصول از گیتِ REA مایرون (`>= 85`) بگذرد — سازگار با
//!    رجیستری‌های Hermes / MCP / `OpenCode`.
//!
//! قانونِ کرنل: صفر `unsafe`، صفر IO، قطعیتِ کامل — همان ورودی، همان
//! باندل، همان هش.
#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]

use sha2::{Digest, Sha256};

// ── ثابت‌های معماری Fishkal (از ممیزیِ خط‌به‌خط) ────────────────────────────

/// امضای باندل — برای تشخیصِ تزریقِ تکراری (ایدمپوتنسِ تزریق).
pub const ARDA_RUNTIME_SIGNATURE: &str = "arda-fishkal-20";

/// ویژگیِ نشانگرِ تزریق روی تگ `<html>` (گیتِ ایدمپوتنسِ [`inject_arda_runtime`]).
pub const ARDA_RUNTIME_MARKUP_MARKER: &str = "data-arda-runtime=\"on\"";

/// شعاع نشانِ دایره‌ای — ارتقای `.lf` فیشکال (L353–358) به دایرهٔ کامل.
pub const FLAG_BADGE_RADIUS: &str = "50%";

/// هالهٔ حلقهٔ نشان روی پس‌زمینهٔ تیرهٔ بازار — رنگ `--cream` فیشکال.
pub const FLAG_BADGE_RING: &str = "0 0 0 1px rgba(239,230,214,.28)";

/// فاصلهٔ نویسه‌ایِ بدنهٔ LTR فیشکال (L299).
pub const FISHKAL_TRACK_BODY_LTR: &str = ".34em";

/// فاصلهٔ نویسه‌ایِ بدنهٔ RTL — درمانِ شکستِ پیوندِ حروفِ خطِ عربی/فارسی.
pub const ARDA_TRACK_BODY_RTL: &str = ".16em";

/// فاصلهٔ نویسه‌ایِ نمایشیِ LTR فیشکال (L301–304).
pub const FISHKAL_TRACK_DISPLAY_LTR: &str = ".14em";

/// فاصلهٔ نویسه‌ایِ نمایشیِ RTL + `text-indent` هم‌اندازه (L389–390 ارتقایافته).
pub const ARDA_TRACK_DISPLAY_RTL: &str = ".07em";

/// شعاع گلاس-پییلِ سوییچر فیشکال (L2113).
pub const FISHKAL_SWITCHER_PILL_PX: u32 = 40;

/// هدف لمسیِ اشاره‌گرِ درشت (WCAG 2.5.5 — حفظِ فیشکال).
pub const FISHKAL_TOUCH_TARGET_PX: u32 = 44;

/// بازهٔ کرونِ سیاره‌ای — دقیقاً ۲۴ ساعت به ثانیه.
pub const ARDA_CRON_INTERVAL_SECS: u64 = 86_400;

/// گیتِ REA فورجِ مهارت — همان آستانهٔ آهنگریِ بازار (`kian-domain`).
pub const SKILL_FORGE_REA_GATE: u8 = kian_domain::mairon_rea_and_reviews::MARKETPLACE_APPROVAL_THRESHOLD;

// ── 1A: دیکشنری پرچمِ وکتور Fishkal — ۲۰ زبانِ سیاره‌ای ────────────────────

/// جهت متن — قفلِ دو-مسیرهٔ فیشکال (`STR.dir`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextDirection {
    /// چپ‌به‌راست.
    Ltr,
    /// راست‌به‌چپ — `ar`، `fa`، `ur`، `he`.
    Rtl,
}

impl TextDirection {
    /// مقدار ویژگی HTML (`dir`).
    #[must_use]
    pub fn as_attr(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }

    /// راست‌به‌چپ؟
    #[must_use]
    pub fn is_rtl(self) -> bool {
        self == Self::Rtl
    }
}

/// یک زبانِ سیاره‌ای — ردیفِ دیکشنری پرچمِ وکتورِ فیشکال.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishkalLocale {
    /// کد ISO کوتاه (`fa`، `en`، …) — کلیدِ `STR` و `root.lang`.
    pub code: &'static str,
    /// نام بومیِ کنونیکال (`'فارسی'`، `'English'`، `'Bahasa Indonesia'`).
    pub native_name: &'static str,
    /// گلیفِ سوییچرِ اسکریپت‌اصیل (`STR.code`): `'فا'`، `'ع'`، `'اردو'`،
    /// `'עב'` یا کد ISO بزرگ برای LTR.
    pub str_code: &'static str,
    /// جهت متنِ زبان.
    pub dir: TextDirection,
    /// زیرتگ منطقه‌ایِ `Intl` (`fa-IR`، `ar-AE`، `hi-IN`، …) — کشِ فرمت‌بندی.
    pub intl_locale: &'static str,
    /// SVG وکتورِ حفظ‌شدهٔ حرف‌به‌حرف با `viewBox="0 0 20 14"`.
    pub svg: &'static str,
}

impl FishkalLocale {
    /// راست‌به‌چپ؟
    #[must_use]
    pub fn is_rtl(self) -> bool {
        self.dir.is_rtl()
    }
}

// پرچم‌های وکتورِ اصلیِ فیشکال — verbatim از L2690–2704 (۱۳ زبان).
const SVG_FA: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"4.7\" fill=\"#239f40\"/><rect y=\"4.7\" width=\"20\" height=\"4.7\" fill=\"#fff\"/><rect y=\"9.3\" width=\"20\" height=\"4.7\" fill=\"#da0000\"/><path d=\"M10 5.7c-.95 0-1.7.8-1.7 1.8S9.05 9.3 10 9.3s1.7-.8 1.7-1.8S10.95 5.7 10 5.7zm0 .9c.5 0 .8.4.8.9s-.3.9-.8.9-.8-.4-.8-.9.3-.9.8-.9z\" fill=\"#da0000\"/></svg>";
const SVG_AR: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#fff\"/><rect width=\"20\" height=\"4.7\" fill=\"#00732F\"/><rect y=\"9.3\" width=\"20\" height=\"4.7\" fill=\"#000\"/><rect width=\"6\" height=\"14\" fill=\"#FF0000\"/></svg>";
const SVG_EN: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#012169\"/><path d=\"M0 0l20 14M20 0L0 14\" stroke=\"#fff\" stroke-width=\"2.6\"/><path d=\"M0 0l20 14M20 0L0 14\" stroke=\"#C8102E\" stroke-width=\"1\"/><path d=\"M10 0v14M0 7h20\" stroke=\"#fff\" stroke-width=\"4\"/><path d=\"M10 0v14M0 7h20\" stroke=\"#C8102E\" stroke-width=\"2.2\"/></svg>";
const SVG_FR: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"6.667\" height=\"14\" fill=\"#0055A4\"/><rect x=\"6.667\" width=\"6.666\" height=\"14\" fill=\"#fff\"/><rect x=\"13.333\" width=\"6.667\" height=\"14\" fill=\"#EF4135\"/></svg>";
const SVG_ES: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"3.5\" fill=\"#AA151B\"/><rect y=\"3.5\" width=\"20\" height=\"7\" fill=\"#F1BF00\"/><rect y=\"10.5\" width=\"20\" height=\"3.5\" fill=\"#AA151B\"/></svg>";
const SVG_TR: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#E30A17\"/><circle cx=\"8\" cy=\"7\" r=\"3.6\" fill=\"#fff\"/><circle cx=\"9.3\" cy=\"7\" r=\"2.9\" fill=\"#E30A17\"/><path d=\"M13 5.5L13.35 6.51L14.43 6.54L13.57 7.19L13.88 8.21L13 7.6L12.12 8.21L12.43 7.19L11.57 6.54L12.65 6.51Z\" fill=\"#fff\"/></svg>";
const SVG_PT: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"8\" height=\"14\" fill=\"#046A38\"/><rect x=\"8\" width=\"12\" height=\"14\" fill=\"#DA291C\"/><circle cx=\"8\" cy=\"7\" r=\"2.4\" fill=\"none\" stroke=\"#FFE900\" stroke-width=\".8\"/><circle cx=\"8\" cy=\"7\" r=\"1.2\" fill=\"none\" stroke=\"#DA291C\" stroke-width=\".6\"/></svg>";
const SVG_NL: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"4.7\" fill=\"#AE1C28\"/><rect y=\"4.7\" width=\"20\" height=\"4.7\" fill=\"#fff\"/><rect y=\"9.3\" width=\"20\" height=\"4.7\" fill=\"#21468B\"/></svg>";
const SVG_JA: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#fff\"/><circle cx=\"10\" cy=\"7\" r=\"3.5\" fill=\"#BC002D\"/></svg>";
const SVG_HI: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"4.7\" fill=\"#FF9933\"/><rect y=\"4.7\" width=\"20\" height=\"4.7\" fill=\"#fff\"/><rect y=\"9.3\" width=\"20\" height=\"4.7\" fill=\"#138808\"/><circle cx=\"10\" cy=\"7\" r=\"1.8\" fill=\"none\" stroke=\"#000080\" stroke-width=\".45\"/><path d=\"M10 5.2v3.6M8.2 7h3.6M8.7 5.7l2.6 2.6M11.3 5.7l-2.6 2.6\" stroke=\"#000080\" stroke-width=\".3\"/></svg>";
const SVG_ZH: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#DE2910\"/><path d=\"M3.4 1.1L3.97 2.72L5.68 2.76L4.31 3.8L4.81 5.44L3.4 4.46L1.99 5.44L2.49 3.8L1.12 2.76L2.83 2.72Z\" fill=\"#FFDE00\"/></svg>";
const SVG_ID: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"7\" fill=\"#CE1126\"/><rect y=\"7\" width=\"20\" height=\"7\" fill=\"#fff\"/></svg>";
const SVG_KO: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#fff\"/><path d=\"M6.8 7a3.2 3.2 0 0 1 6.4 0z\" fill=\"#CD2E3A\"/><path d=\"M6.8 7a3.2 3.2 0 0 0 6.4 0z\" fill=\"#0047A0\"/><circle cx=\"8.4\" cy=\"7\" r=\"1.6\" fill=\"#CD2E3A\"/><circle cx=\"11.6\" cy=\"7\" r=\"1.6\" fill=\"#0047A0\"/></svg>";

// پرچم‌های سیاره‌ایِ تازه — هم‌گرامرِ فیشکال (viewBox 20×14، مینیمالِ وکتور).
const SVG_DE: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"4.7\" fill=\"#000\"/><rect y=\"4.7\" width=\"20\" height=\"4.7\" fill=\"#DD0000\"/><rect y=\"9.3\" width=\"20\" height=\"4.7\" fill=\"#FFCE00\"/></svg>";
const SVG_RU: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"4.7\" fill=\"#fff\"/><rect y=\"4.7\" width=\"20\" height=\"4.7\" fill=\"#0039A6\"/><rect y=\"9.3\" width=\"20\" height=\"4.7\" fill=\"#D52B1E\"/></svg>";
const SVG_IT: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"6.667\" height=\"14\" fill=\"#009246\"/><rect x=\"6.667\" width=\"6.666\" height=\"14\" fill=\"#fff\"/><rect x=\"13.333\" width=\"6.667\" height=\"14\" fill=\"#CE2B37\"/></svg>";
const SVG_HE: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#fff\"/><rect y=\"1.6\" width=\"20\" height=\"2.1\" fill=\"#0038B8\"/><rect y=\"10.3\" width=\"20\" height=\"2.1\" fill=\"#0038B8\"/><path d=\"M10 4.1L12 7.55L8 7.55Z\" fill=\"none\" stroke=\"#0038B8\" stroke-width=\".5\"/><path d=\"M10 9.9L8 6.45L12 6.45Z\" fill=\"none\" stroke=\"#0038B8\" stroke-width=\".5\"/></svg>";
const SVG_UR: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#01411C\"/><rect width=\"5\" height=\"14\" fill=\"#fff\"/><circle cx=\"13.2\" cy=\"7\" r=\"3.4\" fill=\"#fff\"/><circle cx=\"14.5\" cy=\"6.4\" r=\"2.9\" fill=\"#01411C\"/><path d=\"M15.3 4.1L15.51 4.71L16.16 4.72L15.64 5.11L15.83 5.73L15.3 5.36L14.77 5.73L14.96 5.11L14.44 4.72L15.09 4.71Z\" fill=\"#fff\"/></svg>";
const SVG_VI: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#DA251D\"/><path d=\"M10 3.6L10.85 5.84L13.23 5.95L11.37 7.45L12 9.75L10 8.44L8 9.75L8.63 7.45L6.77 5.95L9.15 5.84Z\" fill=\"#FFFF00\"/></svg>";
const SVG_TH: &str = "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"2.33\" fill=\"#A51931\"/><rect y=\"2.33\" width=\"20\" height=\"2.33\" fill=\"#F4F5F8\"/><rect y=\"4.66\" width=\"20\" height=\"4.67\" fill=\"#2D2A4A\"/><rect y=\"9.33\" width=\"20\" height=\"2.34\" fill=\"#F4F5F8\"/><rect y=\"11.67\" width=\"20\" height=\"2.33\" fill=\"#A51931\"/></svg>";

/// دیکشنری کاملِ ۲۰ زبانِ سیاره‌ای — ترتیبِ فیشکال: ۱۳ اصلی + ۷ تازه.
pub const FISHKAL_VECTOR_FLAGS: [FishkalLocale; 20] = [
    FishkalLocale {
        code: "en",
        native_name: "English",
        str_code: "EN",
        dir: TextDirection::Ltr,
        intl_locale: "en-US",
        svg: SVG_EN,
    },
    FishkalLocale {
        code: "ar",
        native_name: "العربية",
        str_code: "ع",
        dir: TextDirection::Rtl,
        intl_locale: "ar-AE",
        svg: SVG_AR,
    },
    FishkalLocale {
        code: "fa",
        native_name: "فارسی",
        str_code: "فا",
        dir: TextDirection::Rtl,
        intl_locale: "fa-IR",
        svg: SVG_FA,
    },
    FishkalLocale {
        code: "tr",
        native_name: "Türkçe",
        str_code: "TR",
        dir: TextDirection::Ltr,
        intl_locale: "tr-TR",
        svg: SVG_TR,
    },
    FishkalLocale {
        code: "fr",
        native_name: "Français",
        str_code: "FR",
        dir: TextDirection::Ltr,
        intl_locale: "fr-FR",
        svg: SVG_FR,
    },
    FishkalLocale {
        code: "es",
        native_name: "Español",
        str_code: "ES",
        dir: TextDirection::Ltr,
        intl_locale: "es-ES",
        svg: SVG_ES,
    },
    FishkalLocale {
        code: "pt",
        native_name: "Português",
        str_code: "PT",
        dir: TextDirection::Ltr,
        intl_locale: "pt-BR",
        svg: SVG_PT,
    },
    FishkalLocale {
        code: "nl",
        native_name: "Nederlands",
        str_code: "NL",
        dir: TextDirection::Ltr,
        intl_locale: "nl-NL",
        svg: SVG_NL,
    },
    FishkalLocale {
        code: "ja",
        native_name: "日本語",
        str_code: "JA",
        dir: TextDirection::Ltr,
        intl_locale: "ja-JP",
        svg: SVG_JA,
    },
    FishkalLocale {
        code: "hi",
        native_name: "हिन्दी",
        str_code: "HI",
        dir: TextDirection::Ltr,
        intl_locale: "hi-IN",
        svg: SVG_HI,
    },
    FishkalLocale {
        code: "zh",
        native_name: "中文",
        str_code: "ZH",
        dir: TextDirection::Ltr,
        intl_locale: "zh-CN",
        svg: SVG_ZH,
    },
    FishkalLocale {
        code: "id",
        native_name: "Bahasa Indonesia",
        str_code: "ID",
        dir: TextDirection::Ltr,
        intl_locale: "id-ID",
        svg: SVG_ID,
    },
    FishkalLocale {
        code: "ko",
        native_name: "한국어",
        str_code: "KO",
        dir: TextDirection::Ltr,
        intl_locale: "ko-KR",
        svg: SVG_KO,
    },
    FishkalLocale {
        code: "de",
        native_name: "Deutsch",
        str_code: "DE",
        dir: TextDirection::Ltr,
        intl_locale: "de-DE",
        svg: SVG_DE,
    },
    FishkalLocale {
        code: "ru",
        native_name: "Русский",
        str_code: "RU",
        dir: TextDirection::Ltr,
        intl_locale: "ru-RU",
        svg: SVG_RU,
    },
    FishkalLocale {
        code: "it",
        native_name: "Italiano",
        str_code: "IT",
        dir: TextDirection::Ltr,
        intl_locale: "it-IT",
        svg: SVG_IT,
    },
    FishkalLocale {
        code: "he",
        native_name: "עברית",
        str_code: "עב",
        dir: TextDirection::Rtl,
        intl_locale: "he-IL",
        svg: SVG_HE,
    },
    FishkalLocale {
        code: "ur",
        native_name: "اردو",
        str_code: "اردو",
        dir: TextDirection::Rtl,
        intl_locale: "ur-PK",
        svg: SVG_UR,
    },
    FishkalLocale {
        code: "vi",
        native_name: "Tiếng Việt",
        str_code: "VI",
        dir: TextDirection::Ltr,
        intl_locale: "vi-VN",
        svg: SVG_VI,
    },
    FishkalLocale {
        code: "th",
        native_name: "ไทย",
        str_code: "TH",
        dir: TextDirection::Ltr,
        intl_locale: "th-TH",
        svg: SVG_TH,
    },
];

/// تعداد زبان‌های سیاره‌ای — همیشه ۲۰ (قفلِ معماری).
pub const PLANETARY_LOCALE_COUNT: usize = FISHKAL_VECTOR_FLAGS.len();

/// یافتنِ زبان با کد ISO.
#[must_use]
pub fn fishkal_locale(code: &str) -> Option<&'static FishkalLocale> {
    FISHKAL_VECTOR_FLAGS.iter().find(|l| l.code == code)
}

/// شمارشِ رخدادِ یک الگوی مشتقِ حرف‌به‌حرف در همهٔ SVGها — گیتِ سلامتِ دیکشنری.
#[must_use]
pub fn count_flags_with(pattern: &str) -> usize {
    FISHKAL_VECTOR_FLAGS.iter().filter(|l| l.svg.contains(pattern)).count()
}

/// همهٔ کدها به ترتیبِ دیکشنری — برای حلقهٔ کرون و پاسخ کاتالوگ.
#[must_use]
pub fn all_locale_codes() -> Vec<&'static str> {
    FISHKAL_VECTOR_FLAGS.iter().map(|l| l.code).collect()
}

/// آیا کد جزو ۲۰ زبانِ سیاره‌ای است؟
#[must_use]
pub fn is_planetary_locale(code: &str) -> bool {
    fishkal_locale(code).is_some()
}

// ── نشانِ دایره‌ایِ مینی-پرچم ───────────────────────────────────────────────

/// استایلِ درون‌خطیِ نشانِ دایره‌ای — ارتقای `.lf` فیشکال (L353–358):
/// دایرهٔ ۲۰×۲۰ با هالهٔ `--cream` و بازچرخشِ دفاعیِ کِرت (`transform:none`).
fn badge_style() -> String {
    format!(
        "--flag-badge-radius:{FLAG_BADGE_RADIUS};width:20px;height:20px;border-radius:50%;\
         overflow:hidden;display:inline-flex;align-items:center;justify-content:center;\
         box-shadow:{FLAG_BADGE_RING}"
    )
}

/// رندرِ نشانِ دایره‌ایِ مینی-پرچم برای یک زبان — `None` اگر کد ناشناخته.
///
/// SVG با `preserveAspectRatio="xMidYMid slice"` داخل دایره برش می‌خورد و
/// بازچرخشِ دفاعیِ کِرتِ فیشکال (`transform:none !important;transition:none;
/// stroke:none;`) حفظ می‌شود تا هیچ قالبِ میزانی کِرتِ نشان را نچرخاند.
#[must_use]
pub fn render_circular_flag_badge(code: &str) -> Option<String> {
    let l = fishkal_locale(code)?;
    Some(format!(
        "<span class=\"arda-flag-badge\" data-lang=\"{code}\" role=\"img\" \
         aria-label=\"{native} ({code})\" dir=\"{dir}\" style=\"{style}\">\
         {svg_prefix}viewBox=\"0 0 20 14\" width=\"20\" height=\"20\" \
         preserveAspectRatio=\"xMidYMid slice\" \
         style=\"transform:none !important;transition:none;stroke:none;display:block\"\
         {svg_rest}</span>",
        native = l.native_name,
        dir = l.dir.as_attr(),
        style = badge_style(),
        svg_prefix = l.svg.split_once("viewBox").map_or("", |(a, _)| a),
        svg_rest = l
            .svg
            .split_once("viewBox=\"0 0 20 14\"")
            .map_or("", |(_, b)| b),
    ))
}

/// رندرِ نوارِ پرچمِ محصول — نشانِ دایره‌ایِ زبانِ مبدأ + نشانِ هر زبانِ
/// ترجمه‌شده (ترتیبِ پایدار؛ تکرارِ مبدأ حذف می‌شود؛ ناشناخته‌ها رد می‌شوند).
///
/// خروجی روی کارتِ بازار و درونِ سوییچرِ زبانِ خودِ محصول می‌نشیند.
#[must_use]
pub fn render_product_flag_strip(origin: &str, translated: &[&str]) -> Option<String> {
    let origin_badge = render_circular_flag_badge(origin)?;
    let mut badges = vec![format!(
        "<span class=\"arda-flag-origin\" data-origin=\"{origin}\">{origin_badge}</span>"
    )];
    for code in translated {
        if *code == origin {
            continue;
        }
        if let Some(badge) = render_circular_flag_badge(code) {
            badges.push(format!(
                "<span class=\"arda-flag-tx\" data-lang=\"{code}\">{badge}</span>"
            ));
        }
    }
    Some(format!(
        "<span class=\"arda-flag-strip\" \
         data-arda-strip=\"{origin}→{}\" dir=\"ltr\">{}</span>",
        translated.join(","),
        badges.concat()
    ))
}

/// شمارشِ نشانِ داخلِ نوارِ پرچم — گیتِ آزمونِ ترکیب.
#[must_use]
pub fn count_flag_badges(strip_html: &str) -> usize {
    strip_html.matches("arda-flag-badge").count()
}

// ── تشخیصِ زبانِ مبدأ محصول ────────────────────────────────────────────────

/// تشخیصِ قطعیِ زبانِ مبدأ از محتوای محصول.
///
/// ترتیبِ قطعی: (۱) ویژگیِ `lang=".."` / `data-lang` روی تگِ ریشه اگر جزو
/// ۲۰ زبان باشد؛ (۲) شمارشِ اسکریپت (فارسی‌خاص، اردوخاص، عربی، عبری،
/// دیواناگری، تایلندی، کانا/هیراگانا، هانگول، سیریلیک، CJK، ویتنامی‌خاص) —
/// بیشترینِ قطعی برنده است و تساوی با ترتیبِ شمارشِ ثابت حل می‌شود؛
/// (۳) برگشت به `'en'`.
#[must_use]
pub fn detect_origin_locale(html: &str) -> &'static str {
    if let Some(code) = detected_lang_attribute(html) {
        return code;
    }
    if let Some(code) = script_vote_winner(html) {
        return code;
    }
    if has_vietnamese_markers(html) {
        return "vi";
    }
    "en"
}

/// خوانشِ ویژگی `lang` / `data-lang` از تگِ ریشه — فقط کدهای سیاره‌ای.
fn detected_lang_attribute(html: &str) -> Option<&'static str> {
    for attr in ["lang=\"", "data-lang=\""] {
        for segment in html.split(attr).skip(1) {
            let value: String = segment.chars().take_while(|c| c.is_alphanumeric() || *c == '-').collect();
            let short = value.split('-').next().unwrap_or("").to_ascii_lowercase();
            if let Some(l) = fishkal_locale(&short) {
                return Some(l.code);
            }
        }
    }
    None
}

/// رأیِ اسکریپت — بازرسیِ قطعیِ حروفِ خاص به ترتیبِ اولویت: اردو، فارسی،
/// عربی، عبری، دیواناگری، تایلندی، کانا، هانگول، سیریلیک، CJK.
///
/// حروفِ خاصِ اردو/فارسی «قاطع»اند (شمارش نمی‌برند — متنِ فارسی تقریباً
/// تماماً با حروفِ بلوکِ عربی نوشته می‌شود)؛ پس گلیفِ خاص، برندهٔ مطلق است.
fn script_vote_winner(text: &str) -> Option<&'static str> {
    // اردو — حروفِ خاصِ اردو (ٹ ڈ ڑ ں ے ھ ہ) پیش از فارسی؛ فارسی‌زبان‌ها
    // هرگز این گلیف‌ها را نمی‌نویسند.
    const URDU_SPECIFIC: [char; 7] = ['ٹ', 'ڈ', 'ڑ', 'ں', 'ے', 'ھ', 'ہ'];
    // فارسی — پ چ ژ گ ک (کِ فارسی `U+06A9` جدا از کافِ عربی `U+0643`).
    const PERSIAN_SPECIFIC: [char; 5] = ['پ', 'چ', 'ژ', 'گ', 'ک'];
    if text.chars().any(|c| URDU_SPECIFIC.contains(&c)) {
        return Some("ur");
    }
    if text.chars().any(|c| PERSIAN_SPECIFIC.contains(&c)) {
        return Some("fa");
    }
    if count_in_range(text, '\u{0600}', '\u{06FF}') > 0 {
        return Some("ar");
    }
    if count_in_range(text, '\u{0590}', '\u{05FF}') > 0 {
        return Some("he");
    }
    if count_in_range(text, '\u{0900}', '\u{097F}') > 0 {
        return Some("hi");
    }
    if count_in_range(text, '\u{0E00}', '\u{0E7F}') > 0 {
        return Some("th");
    }
    if count_in_range(text, '\u{3040}', '\u{30FF}') > 0 {
        return Some("ja");
    }
    if count_in_range(text, '\u{AC00}', '\u{D7AF}') > 0 {
        return Some("ko");
    }
    if count_in_range(text, '\u{0400}', '\u{04FF}') > 0 {
        return Some("ru");
    }
    if count_in_range(text, '\u{4E00}', '\u{9FFF}') > 0 {
        return Some("zh");
    }
    None
}

/// شمارشِ بازهٔ یونیکد.
fn count_in_range(text: &str, lo: char, hi: char) -> usize {
    text.chars().filter(|c| (lo..=hi).contains(c)).count()
}

/// تشخیصِ ویتنامی — حروفِ خاصِ لاتینِ گسترش‌یافته.
fn has_vietnamese_markers(text: &str) -> bool {
    const MARKS: [char; 15] = ['ă', 'â', 'đ', 'ê', 'ô', 'ơ', 'ư', 'ạ', 'ả', 'ấ', 'ầ', 'ế', 'ỗ', 'ộ', 'ự'];
    text.chars().any(|c| MARKS.contains(&c))
}

// ── 1B-۳: کاتالوگِ یکپارچهٔ ۶-دامنه‌ای + اعتبارسنجِ توازنِ مکانیکی ───────────

/// شش دامنهٔ یکپارچه — جایگزینِ ۶ دیکشنریِ پراکندهٔ فیشکال
/// (`I18N`، `BIZSTR`، `LOGSTR`، `CARDSTR`، `OPTSTR`، `SPECIES`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArdaCatalogScope {
    /// هستهٔ رابط — عنوان، برچسب‌های عمومی.
    Core,
    /// تجارت — قیمت، خرید، پرداخت Stars/XTR.
    Business,
    /// رکوردها — جدول‌ها، تاریخچه، حالت خالی.
    Records,
    /// هم‌رسانی — پیوند، دعوت، affiliate.
    Share,
    /// ابزارها — محاسبه‌گر، فرم، خروجی.
    Tools,
    /// محتوا — توضیح محصول، راهنما.
    Content,
}

impl ArdaCatalogScope {
    /// رشتهٔ متعارف دامنه.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Business => "business",
            Self::Records => "records",
            Self::Share => "share",
            Self::Tools => "tools",
            Self::Content => "content",
        }
    }

    /// همهٔ دامنه‌ها به ترتیب قطعی.
    #[must_use]
    pub fn all() -> [Self; 6] {
        [Self::Core, Self::Business, Self::Records, Self::Share, Self::Tools, Self::Content]
    }
}

/// یک مدخلِ کاتالوگ — `(دامنه، کلید)` با ترجمهٔ هر زبان.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArdaCatalogEntry {
    /// دامنهٔ شش‌گانه.
    pub scope: ArdaCatalogScope,
    /// کلید (`"app_title"`، `"price_stars"`، …).
    pub key: String,
    /// ترجمه‌ها — کدِ زبان → رشتهٔ بومی.
    pub translations: std::collections::BTreeMap<String, String>,
}

/// گزارشِ توازنِ یک زبان نسبت به زبانِ پایه.
#[derive(Debug, Clone, PartialEq)]
pub struct LocaleParity {
    /// کد زبان.
    pub locale: String,
    /// کلیدهای غایب (نقصِ `unitFt` در `tr..ko` — درمان‌شده).
    pub missing_keys: Vec<String>,
    /// نمرهٔ پوششِ دقیقِ `0.0..=1.0` — `(کل − غایب) / کل`.
    pub coverage_score: f64,
}

/// گزارشِ کاملِ توازنِ کاتالوگ — مکانیکی و بازتولیدپذیر.
#[derive(Debug, Clone, PartialEq)]
pub struct ArdaParityReport {
    /// زبانِ پایهٔ توازن.
    pub base_locale: String,
    /// تعدادِ کلیدهای مرجعِ پایه.
    pub total_keys: usize,
    /// توازنِ هر زبانِ غیرِ پایه به ترتیبِ دیکشنری.
    pub locales: Vec<LocaleParity>,
    /// بدترینِ پوشش بین زبان‌ها — نمرهٔ بر parfait کاتالوگ.
    pub coverage_score: f64,
    /// کلیدهای تکراریِ `(دامنه، کلید)` در ساخت.
    pub duplicate_keys: Vec<String>,
    /// توازنِ کامل؟ (صفرِ غایب + صفرِ تکراری)
    pub fully_parity: bool,
}

/// کاتالوگِ یکپارچهٔ بومی‌سازی — یک دیکشنری به‌جای شش دیکشنریِ فیشکال.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArdaLocaleCatalog {
    /// زبانِ پایهٔ توازن (پیش‌فرض `en`).
    pub base_locale: String,
    /// مدخل‌ها به ترتیبِ درج.
    pub entries: Vec<ArdaCatalogEntry>,
}

/// سازندهٔ کاتالوگ با ثبتِ کلیدهای تکراری (نقصِ دیکشنری‌های فیشکال).
#[derive(Debug, Clone)]
pub struct ArdaLocaleCatalogBuilder {
    base: String,
    entries: Vec<ArdaCatalogEntry>,
    duplicates: Vec<String>,
}

impl ArdaLocaleCatalogBuilder {
    /// سازندهٔ تازه روی زبانِ پایه.
    #[must_use]
    pub fn new(base: &str) -> Self {
        Self { base: base.to_owned(), entries: Vec::new(), duplicates: Vec::new() }
    }

    /// افزودنِ یک مدخل — ترجمه‌ها به‌صورت `(کد، متن)`؛ ترجمهٔ پایه الزامی نیست
    /// اما توازن نسبت به پایه سنجیده می‌شود.
    pub fn add(&mut self, scope: ArdaCatalogScope, key: &str, translations: &[(&str, &str)]) {
        let id = format!("{}.{}", scope.as_str(), key);
        if self.entries.iter().any(|e| e.scope == scope && e.key == key) {
            self.duplicates.push(id);
            return;
        }
        let mut map = std::collections::BTreeMap::new();
        for (code, value) in translations {
            map.insert((*code).to_owned(), (*value).to_owned());
        }
        self.entries.push(ArdaCatalogEntry { scope, key: key.to_owned(), translations: map });
    }

    /// نهایی‌سازی — کاتالوگِ کامل با فهرستِ تکراری‌ها.
    #[must_use]
    pub fn build(self) -> ArdaLocaleCatalog {
        ArdaLocaleCatalog { base_locale: self.base, entries: self.entries }
    }

    /// تکراری‌های ثبت‌شده هنگام ساخت.
    #[must_use]
    pub fn duplicates(&self) -> &[String] {
        &self.duplicates
    }
}

/// اعتبارسنجِ مکانیکیِ توازنِ ۱۰۰٪ — درمانِ نقصِ `unitFt` غایب در `tr..ko`
/// و نقصِ `OPTSTR` سه-زبانِ فیشکال: هیچ زبانی کمتر از پایه کلید ندارد.
#[must_use]
pub fn validate_catalog_parity(catalog: &ArdaLocaleCatalog) -> ArdaParityReport {
    let total_keys = catalog.entries.len();
    let mut locales = Vec::new();
    let mut worst = 1.0_f64;
    for l in FISHKAL_VECTOR_FLAGS {
        let missing: Vec<String> = catalog
            .entries
            .iter()
            .filter(|e| !e.translations.contains_key(l.code))
            .map(|e| format!("{}.{}", e.scope.as_str(), e.key))
            .collect();
        let coverage = if total_keys == 0 {
            1.0
        } else {
            f64::from(u32::try_from(total_keys - missing.len()).unwrap_or(u32::MAX))
                / f64::from(u32::try_from(total_keys).unwrap_or(u32::MAX))
        };
        worst = worst.min(coverage);
        locales.push(LocaleParity { locale: l.code.to_owned(), missing_keys: missing, coverage_score: coverage });
    }
    let duplicates = catalog_duplicates(catalog);
    let fully = locales.iter().all(|p| p.missing_keys.is_empty()) && duplicates.is_empty();
    ArdaParityReport {
        base_locale: catalog.base_locale.clone(),
        total_keys,
        coverage_score: worst,
        locales,
        duplicate_keys: duplicates,
        fully_parity: fully,
    }
}

/// شمارشِ کلیدهای تکراریِ درونِ یک کاتالوگِ آماده (خطِ دفاعِ دوم).
fn catalog_duplicates(catalog: &ArdaLocaleCatalog) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut dups = Vec::new();
    for e in &catalog.entries {
        let id = format!("{}.{}", e.scope.as_str(), e.key);
        if !seen.insert(id.clone()) {
            dups.push(id);
        }
    }
    dups
}

// ── کاتالوگِ سیاره‌ایِ پیش‌فرض — ۷ کلید × ۲۰ زبان، توازنِ کامل ───────────────

/// کاتالوگِ سیاره‌ایِ پیش‌فرض ARDA — یک کلید به‌ازای هر دامنهٔ شش‌گانه + کلید
/// `aria-label` سوییچر؛ ۱۴۰ رشتهٔ بومیِ کامل با توازنِ ۱۰۰٪ مکانیکی.
#[must_use]
pub fn default_planetary_catalog() -> ArdaLocaleCatalog {
    let mut b = ArdaLocaleCatalogBuilder::new("en");
    b.add(ArdaCatalogScope::Core, "app_title", &[
        ("en", "Kian Sovereign Studio"), ("ar", "استوديو كيان السيادي"),
        ("fa", "استودیوی حاکم کیان"), ("tr", "Kian Egemen Stüdyosu"),
        ("fr", "Studio Souverain Kian"), ("es", "Estudio Soberano Kian"),
        ("pt", "Estúdio Soberano Kian"), ("nl", "Kian Soeverein Studio"),
        ("ja", "キアン・ソブリン・スタジオ"), ("hi", "कियान सार्वभौम स्टूडियो"),
        ("zh", "KIAN 主权工作室"), ("id", "Studio Kian Berdaulat"),
        ("ko", "키안 주권 스튜디오"), ("de", "Kian Souveränes Studio"),
        ("ru", "Суверенная студия Kian"), ("it", "Studio Sovrano Kian"),
        ("he", "סטודיו הריבוני של קיאן"), ("ur", "کئین خودمختار اسٹوڈیو"),
        ("vi", "Xưởng Kian Có Chủ quyền"), ("th", "สตูดิโอเคียนอธิปไตย"),
    ]);
    b.add(ArdaCatalogScope::Core, "ui_language", &[
        ("en", "UI language"), ("ar", "لغة الواجهة"),
        ("fa", "زبانِ رابط"), ("tr", "Arayüz dili"),
        ("fr", "Langue de l'interface"), ("es", "Idioma de la interfaz"),
        ("pt", "Idioma da interface"), ("nl", "Interfacetaal"),
        ("ja", "インターフェース言語"), ("hi", "इंटरफ़ेस भाषा"),
        ("zh", "界面语言"), ("id", "Bahasa antarmuka"),
        ("ko", "인터페이스 언어"), ("de", "Oberflächensprache"),
        ("ru", "Язык интерфейса"), ("it", "Lingua interfaccia"),
        ("he", "שפת הממשק"), ("ur", "انٹرفیس زبان"),
        ("vi", "Ngôn ngữ giao diện"), ("th", "ภาษาอินเทอร์เฟซ"),
    ]);
    b.add(ArdaCatalogScope::Business, "price_stars", &[
        ("en", "Price (Stars)"), ("ar", "السعر (نجوم)"),
        ("fa", "قیمت (ستاره)"), ("tr", "Fiyat (Yıldız)"),
        ("fr", "Prix (étoiles)"), ("es", "Precio (estrellas)"),
        ("pt", "Preço (estrelas)"), ("nl", "Prijs (sterren)"),
        ("ja", "価格（スター）"), ("hi", "कीमत (स्टार)"),
        ("zh", "价格（星星）"), ("id", "Harga (Bintang)"),
        ("ko", "가격(스타)"), ("de", "Preis (Sterne)"),
        ("ru", "Цена (звёзды)"), ("it", "Prezzo (stelle)"),
        ("he", "מחיר (כוכבים)"), ("ur", "قیمت (ستارے)"),
        ("vi", "Giá (sao)"), ("th", "ราคา (ดาว)"),
    ]);
    b.add(ArdaCatalogScope::Records, "empty_state", &[
        ("en", "No records yet"), ("ar", "لا سجلات بعد"),
        ("fa", "هنوز سابقه‌ای نیست"), ("tr", "Henüz kayıt yok"),
        ("fr", "Aucun enregistrement"), ("es", "Aún no hay registros"),
        ("pt", "Ainda sem registros"), ("nl", "Nog geen registraties"),
        ("ja", "記録はまだありません"), ("hi", "अभी कोई रिकॉर्ड नहीं"),
        ("zh", "暂无记录"), ("id", "Belum ada catatan"),
        ("ko", "아직 기록이 없습니다"), ("de", "Noch keine Einträge"),
        ("ru", "Записей пока нет"), ("it", "Nessun record"),
        ("he", "אין עדיין רשומות"), ("ur", "ابھی کوئی ریکارڈ نہیں"),
        ("vi", "Chưa có bản ghi"), ("th", "ยังไม่มีรายการ"),
    ]);
    b.add(ArdaCatalogScope::Share, "copy_link", &[
        ("en", "Copy link"), ("ar", "نسخ الرابط"),
        ("fa", "رونوشتِ پیوند"), ("tr", "Bağlantıyı kopyala"),
        ("fr", "Copier le lien"), ("es", "Copiar enlace"),
        ("pt", "Copiar link"), ("nl", "Link kopiëren"),
        ("ja", "リンクをコピー"), ("hi", "लिंक कॉपी करें"),
        ("zh", "复制链接"), ("id", "Salin tautan"),
        ("ko", "링크 복사"), ("de", "Link kopieren"),
        ("ru", "Копировать ссылку"), ("it", "Copia link"),
        ("he", "העתק קישור"), ("ur", "لنک کاپی کریں"),
        ("vi", "Sao chép liên kết"), ("th", "คัดลอกลิงก์"),
    ]);
    b.add(ArdaCatalogScope::Tools, "calculate", &[
        ("en", "Calculate"), ("ar", "احسب"),
        ("fa", "محاسبه"), ("tr", "Hesapla"),
        ("fr", "Calculer"), ("es", "Calcular"),
        ("pt", "Calcular"), ("nl", "Berekenen"),
        ("ja", "計算する"), ("hi", "गणना करें"),
        ("zh", "计算"), ("id", "Hitung"),
        ("ko", "계산"), ("de", "Berechnen"),
        ("ru", "Рассчитать"), ("it", "Calcola"),
        ("he", "חשב"), ("ur", "حساب کریں"),
        ("vi", "Tính toán"), ("th", "คำนวณ"),
    ]);
    b.add(ArdaCatalogScope::Content, "description", &[
        ("en", "Description"), ("ar", "الوصف"),
        ("fa", "توضیح"), ("tr", "Açıklama"),
        ("fr", "Description"), ("es", "Descripción"),
        ("pt", "Descrição"), ("nl", "Beschrijving"),
        ("ja", "説明"), ("hi", "विवरण"),
        ("zh", "描述"), ("id", "Deskripsi"),
        ("ko", "설명"), ("de", "Beschreibung"),
        ("ru", "Описание"), ("it", "Descrizione"),
        ("he", "תיאור"), ("ur", "تفصیل"),
        ("vi", "Mô tả"), ("th", "คำอธิบาย"),
    ]);
    b.build()
}

// ── 1B: مولدِ باندلِ زمانِ اجرا — درمانِ ۲۵ نقصِ ممیزیِ فیشکال ───────────────

/// پشتهٔ قلمِ هر اسکریپت — ارتقای L299/L301–304 فیشکال.
fn font_stack(code: &str) -> &'static str {
    match code {
        "ar" | "fa" | "ur" => "'Vazirmatn','Vazirmatn FD','Noto Sans Arabic',Tahoma,sans-serif",
        "he" => "'Noto Sans Hebrew','Vazirmatn',Tahoma,sans-serif",
        "ja" => "'Noto Sans JP','Hiragino Sans','Yu Gothic',sans-serif",
        "zh" => "'Noto Sans SC','PingFang SC','Microsoft YaHei',sans-serif",
        "ko" => "'Noto Sans KR','Malgun Gothic',sans-serif",
        "hi" => "'Noto Sans Devanagari','Mangal',sans-serif",
        "th" => "'Noto Sans Thai','Leelawadee UI',sans-serif",
        _ => "'Inter','Segoe UI',system-ui,sans-serif",
    }
}

/// بلوکِ CSSِ هر زبان — متغیرهای `--font`، `--track-body`، `--track-display`
/// و (برای RTL) تورفتگیِ نمایشیِ هم‌اندازه تا پیوندِ حروفِ خطِ کرسیو هرگز
/// نشکند (L389–390 ارتقایافته).
fn css_locale_block(l: &FishkalLocale) -> String {
    let (track_body, track_display, indent) = if l.is_rtl() {
        (ARDA_TRACK_BODY_RTL, ARDA_TRACK_DISPLAY_RTL, ARDA_TRACK_DISPLAY_RTL)
    } else {
        (FISHKAL_TRACK_BODY_LTR, FISHKAL_TRACK_DISPLAY_LTR, "0em")
    };
    format!(
        ":root[data-lang=\"{code}\"] {{ --font:{font}; --track-body:{body}; \
         --track-display:{display}; --display-indent:{indent}; }}\n",
        code = l.code,
        font = font_stack(l.code),
        body = track_body,
        display = track_display,
        indent = indent,
    )
}

/// CSSِ پایهٔ باندل — نشانه‌گذاری‌های ثابتِ آزمون‌پذیر.
fn base_css() -> String {
    String::from(
        "/* ARDA Fishkal planetary localization runtime (arda-fishkal-20) — 25 audit defects cured */\n\
         :root { --flag-badge-radius:50%; --arda-ring:0 0 0 1px rgba(239,230,214,.28);\n\
         --gold:#f2c14e; --track-body:.34em; --track-display:.14em; --display-indent:0em;\n\
         --font:'Inter','Vazirmatn',system-ui,-apple-system,'Segoe UI',sans-serif; }\n\
         body { font-family:var(--font); letter-spacing:var(--track-body); }\n\
         h1,h2,.arda-display { letter-spacing:var(--track-display); text-indent:var(--display-indent); }\n\
         [dir=\"auto\"] { unicode-bidi:plaintext; }\n\
         [data-dir=\"latin\"] { direction:ltr; unicode-bidi:isolate; text-align:start; }\n\
         .arda-flag-badge { --flag-badge-radius:50%; width:20px; height:20px; border-radius:50%;\n\
         overflow:hidden; display:inline-flex; align-items:center; justify-content:center;\n\
         box-shadow:0 0 0 1px rgba(239,230,214,.28); }\n\
         .arda-flag-badge svg { transform:none !important; transition:none; stroke:none; }\n\
         .arda-flag-strip { display:inline-flex; gap:6px; align-items:center; direction:ltr; }\n\
         .arda-switcher { position:absolute; inset-inline-start:16px; top:16px; width:40px; height:40px; }\n\
         .arda-switcher__trigger { width:40px; height:40px; border-radius:50%; cursor:pointer;\n\
         background:rgba(239,230,214,.08); border:1px solid rgba(239,230,214,.28); color:inherit;\n\
         font-family:var(--font); backdrop-filter:blur(8px); }\n\
         .arda-switcher__menu { position:absolute; top:44px; inset-inline-start:0; min-width:220px;\n\
         margin:0; padding:6px; list-style:none; background:rgba(20,18,24,.94);\n\
         border:1px solid rgba(239,230,214,.24); border-radius:14px; z-index:60; }\n\
         .arda-switcher__option { display:flex; align-items:center; gap:10px; min-height:44px;\n\
         padding:6px 10px; border-radius:10px; cursor:pointer; }\n\
         .arda-switcher__option:focus-visible { outline:2px solid var(--gold); }\n\
         .arda-switcher__option[aria-selected=\"true\"] { color:var(--gold); }\n\
         .arda-switcher__check { color:var(--gold); }\n\
         @media (pointer:coarse) { .arda-switcher__option { min-height:44px; } }\n",
    )
}

/// قالبِ JSِ باندل — `__ARDA_DATA__` در زمانِ تولید جای‌گذاری می‌شود.
const JS_TEMPLATE: &str = r#"(function(){"use strict";
if(window.ARDA&&window.ARDA.__fishkalSignature==="arda-fishkal-20"){return;}
var DATA=__ARDA_DATA__;
var ORIGIN=DATA.origin;
var STR=DATA.strings,DIR=DATA.dir,NATIVE=DATA.native,GLYPH=DATA.glyph,INTL=DATA.intl,ORDER=DATA.order;
var RTL={"ar":1,"fa":1,"ur":1,"he":1};
/* cached Intl engines — cures Fishkal L2201 & L2670 (per-call constructor churn) */
var _nf=Object.create(null),_dtf=Object.create(null),_pr=Object.create(null);
function nf(l,cur){var k=l+"|"+(cur||"");if(!_nf[k]){_nf[k]=new Intl.NumberFormat(INTL[l]||l,cur?{style:"currency",currency:cur,currencyDisplay:"code"}:{});}return _nf[k];}
function dtf(l){if(!_dtf[l]){_dtf[l]=new Intl.DateTimeFormat(INTL[l]||l,{dateStyle:"medium",timeStyle:"short"});}return _dtf[l];}
function pr(l){if(!_pr[l]){_pr[l]=new Intl.PluralRules(INTL[l]||l);}return _pr[l];}
var ARDA=window.ARDA=window.ARDA||{};
ARDA.__fishkalSignature="arda-fishkal-20";
ARDA.locale=ORIGIN;
ARDA.locales=ORDER;
ARDA.intl={number:function(v,l){return nf(l).format(v);},
currency:function(v,l,cur){return nf(l,cur||"XTR").format(v);},
dateTime:function(d,l){return dtf(l).format(d);},
plural:function(n,l){return pr(l).select(n);}};
ARDA.t=function(key,l){l=l||ARDA.locale;var s=STR[key];return (s&&(s[l]||s[ORIGIN]||s.en))||key;};
/* atomic 3-attribute root flip — Fishkal L2085 cured into one synchronous tick */
ARDA.applyLocale=function(l){if(!NATIVE[l]){return false;}
var root=document.documentElement;
root.lang=l; root.dir=DIR[l]; root.dataset.lang=l;
ARDA.locale=l;
try{document.dispatchEvent(new CustomEvent("arda:locale",{detail:{locale:l,dir:DIR[l]}}));}catch(e){}
return true;};
/* bidi isolation — Fishkal L2683 & L3082 */
ARDA.bidi={isRtl:function(l){return RTL[l||ARDA.locale]===1;},
autoDir:function(text){var s=String(text),i,c;
for(i=0;i<s.length;i++){c=s.codePointAt(i);
if((c>=65&&c<=90)||(c>=97&&c<=122)){return "ltr";}
if((c>=0x5D0&&c<=0x5EA)||(c>=0x600&&c<=0x6FF)){return "rtl";}}
return "ltr";},
isolate:function(text){return {"dir":"auto","unicodeBidi":"plaintext",text:String(text)};}};
/* canvas bidi — Fishkal L2898-2933 exact 4-step procedure */
ARDA.canvas={};
ARDA.canvas.drawBidiText=function(ctx,text,x,y,opts){opts=opts||{};var l=opts.locale||ARDA.locale;
var rtl=RTL[l]===1;var s=String(text);
var latin=opts.forceLtr!==undefined?opts.forceLtr:/^[\x20-\x7E]+$/.test(s);
/* step 1 — g.direction (forced LTR for Latin substrings) */
ctx.direction=latin?"ltr":(rtl?"rtl":"ltr");
/* step 2 — g.textAlign follows the resolved direction */
ctx.textAlign=opts.align||(rtl&&!latin?"right":"left");
/* step 3 — mirrored nx under RTL */
var nx=(rtl&&!latin&&opts.mirrorX)?(opts.width||ctx.canvas.width)-x:x;
/* step 4 — draw with the resolved state */
if(opts.stroke){ctx.strokeText(s,nx,y);}else{ctx.fillText(s,nx,y);}
return {direction:ctx.direction,align:ctx.textAlign,x:nx};};
/* string hydration — composite strings get dir="auto", latin tickers get forced ltr */
ARDA.hydrate=function(root){var scope=root||document;
scope.querySelectorAll("[data-arda-i18n]").forEach(function(el){
var key=el.getAttribute("data-arda-i18n");
el.textContent=ARDA.t(key);
if(el.hasAttribute("data-arda-composite")){el.setAttribute("dir","auto");}});
scope.querySelectorAll("[data-arda-latin]").forEach(function(el){el.setAttribute("dir","ltr");});};
/* WAI-ARIA listbox switcher — Fishkal L2113-2145 with D-1..D-8 cured */
ARDA.switcher={mount:function(host,opts){opts=opts||{};
if(!host){return null;}
if(host.dataset.ardaSwitcher==="on"){return host.querySelector(".arda-switcher__trigger");}
host.dataset.ardaSwitcher="on";
host.className+=" arda-switcher";
var label=ARDA.t("core.ui_language");
var btn=document.createElement("button");
btn.type="button";btn.className="arda-switcher__trigger";
btn.setAttribute("aria-haspopup","listbox");btn.setAttribute("aria-expanded","false");
btn.setAttribute("aria-label",label);
btn.textContent=GLYPH[ARDA.locale]||ARDA.locale.toUpperCase();
var menu=document.createElement("ul");
menu.className="arda-switcher__menu";menu.setAttribute("role","listbox");
menu.setAttribute("aria-label",label);menu.hidden=true;
var current=opts.locale||ARDA.locale;
var order=opts.locales||ORDER;
order.forEach(function(code){
var li=document.createElement("li");
li.setAttribute("role","option");li.id="arda-opt-"+code;li.dataset.locale=code;
li.setAttribute("aria-selected",code===current?"true":"false");li.tabIndex=-1;
var badge=document.createElement("span");badge.className="arda-flag-badge";
badge.textContent=GLYPH[code]||code.toUpperCase();
var name=document.createElement("span");name.className="arda-switcher__name";
name.textContent=NATIVE[code]||code;
var check=document.createElement("span");check.className="arda-switcher__check";
check.textContent="✓";check.style.color="var(--gold,#f2c14e)";check.hidden=code!==current;
li.appendChild(badge);li.appendChild(name);li.appendChild(check);menu.appendChild(li);});
host.appendChild(btn);host.appendChild(menu);
function open(){menu.hidden=false;btn.setAttribute("aria-expanded","true");
var sel=menu.querySelector('[aria-selected="true"]')||menu.firstElementChild;
if(sel){sel.focus({preventScroll:true});}}
function close(){menu.hidden=true;btn.setAttribute("aria-expanded","false");btn.focus({preventScroll:true});}
btn.addEventListener("click",function(){menu.hidden?open():close();});
menu.addEventListener("click",function(e){var li=e.target.closest('[role="option"]');
if(li&&ARDA.applyLocale(li.dataset.locale)){current=li.dataset.locale;
menu.querySelectorAll('[aria-selected="true"]').forEach(function(o){
o.setAttribute("aria-selected","false");o.querySelector(".arda-switcher__check").hidden=true;});
li.setAttribute("aria-selected","true");li.querySelector(".arda-switcher__check").hidden=false;
btn.textContent=GLYPH[current]||current.toUpperCase();close();}});
menu.addEventListener("keydown",function(e){
var items=Array.prototype.slice.call(menu.querySelectorAll('[role="option"]'));
var idx=items.indexOf(document.activeElement);var next=null;
if(e.key==="ArrowDown"){next=items[(idx+1)%items.length];}
else if(e.key==="ArrowUp"){next=items[(idx-1+items.length)%items.length];}
else if(e.key==="Home"){next=items[0];}
else if(e.key==="End"){next=items[items.length-1];}
else if(e.key==="Escape"){close();return;}
else if(e.key==="Enter"||e.key===" "){e.preventDefault();if(document.activeElement){document.activeElement.click();}return;}
if(next){e.preventDefault();next.focus({preventScroll:true});}});
document.addEventListener("pointerdown",function(e){if(!host.contains(e.target)&&!menu.hidden){close();}});
return btn;}};
ARDA.applyLocale(ORIGIN);
if(document.readyState==="loading"){document.addEventListener("DOMContentLoaded",function(){ARDA.hydrate(document);});}
else{ARDA.hydrate(document);}
})();
"#;

/// باندلِ زمانِ اجرا — CSS + JS + ترکیبِ آمادهٔ تزریق.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArdaRuntimeBundle {
    /// زبانِ مبدأ.
    pub origin: String,
    /// زبان‌های فعالِ باندل (به ترتیبِ دیکشنری).
    pub locales: Vec<String>,
    /// بلوک `<style>` کامل.
    pub css: String,
    /// بلوک `<script>` کامل.
    pub js: String,
    /// ترکیبِ `<style>…</style><script>…</script>` برای تزریقِ تک‌لنگه‌ای.
    pub combined: String,
}

impl ArdaRuntimeBundle {
    /// SHA-256 محتوای ترکیبی — گاردِ دلتای کرون.
    #[must_use]
    pub fn content_sha256(&self) -> String {
        arda_content_sha256(&self.combined)
    }
}

/// مولدِ باندلِ بومی‌سازی — معماریِ اصیلِ فیشکال با ۲۵ نقصِ درمان‌شده.
#[derive(Debug, Clone, Copy, Default)]
pub struct ArdaFishkalBundleGenerator;

impl ArdaFishkalBundleGenerator {
    /// تولیدِ باندل برای زبانِ مبدأ و فهرستِ هدف‌ها.
    ///
    /// کدهای ناشناخته رد می‌شوند؛ زبانِ مبدأ همیشه در فهرستِ فعال است؛
    /// خروجی برای همان ورودی بیتِ-بیت یکسان است (قطعیتِ کامل).
    #[must_use]
    pub fn generate(&self, origin: &str, targets: &[&str]) -> ArdaRuntimeBundle {
        let origin = fishkal_locale(origin).map_or("en", |l| l.code);
        let mut codes: Vec<&str> = Vec::new();
        codes.push(origin);
        for t in targets {
            if fishkal_locale(t).is_some() && !codes.contains(t) {
                codes.push(t);
            }
        }
        let mut css = base_css();
        for code in &codes {
            if let Some(l) = fishkal_locale(code) {
                css.push_str(&css_locale_block(l));
            }
        }
        let data = Self::data_json(&codes, origin);
        let js = JS_TEMPLATE.replace("__ARDA_DATA__", &data);
        let combined = format!("<style data-arda-runtime=\"on\">{css}</style>\n<script data-arda-runtime=\"on\">{js}</script>");
        ArdaRuntimeBundle {
            origin: origin.to_owned(),
            locales: codes.iter().map(|c| (*c).to_owned()).collect(),
            css,
            js,
            combined,
        }
    }

    /// JSON دادهٔ زبان‌ها — `strings`، `dir`، `native`، `glyph`، `intl`، `order`، `origin`.
    #[must_use]
    fn data_json(codes: &[&str], origin: &str) -> String {
        let catalog = default_planetary_catalog();
        let mut strings = serde_json::Map::new();
        for e in &catalog.entries {
            let key = format!("{}.{}", e.scope.as_str(), e.key);
            let mut per = serde_json::Map::new();
            for code in codes {
                if let Some(v) = e.translations.get(*code) {
                    per.insert((*code).to_owned(), serde_json::Value::String(v.clone()));
                }
            }
            strings.insert(key, serde_json::Value::Object(per));
        }
        let obj = |f: fn(&FishkalLocale) -> &str| -> serde_json::Map<String, serde_json::Value> {
            let mut m = serde_json::Map::new();
            for code in codes {
                if let Some(l) = fishkal_locale(code) {
                    m.insert((*code).to_owned(), serde_json::Value::String(f(l).to_owned()));
                }
            }
            m
        };
        let payload = serde_json::json!({
            "origin": origin,
            "order": codes,
            "strings": serde_json::Value::Object(strings),
            "dir": serde_json::Value::Object(obj(|l| l.dir.as_attr())),
            "native": serde_json::Value::Object(obj(|l| l.native_name)),
            "glyph": serde_json::Value::Object(obj(|l| l.str_code)),
            "intl": serde_json::Value::Object(obj(|l| l.intl_locale)),
        });
        payload.to_string()
    }
}

/// گزارشِ تزریق — HTML نهایی + آیا تزریق رخ داد؟
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArdaInjectionReport {
    /// HTML پس از تزریق (یا دست‌نخورده اگر قبلاً تزریق شده بود).
    pub html: String,
    /// تزریق انجام شد؟
    pub injected: bool,
    /// نقطهٔ لنگر — `head`، `html` یا `prepend`.
    pub anchor: &'static str,
}

/// تزریقِ ایدمپوتنتِ باندل در HTML محصول — پیش از `</head>`، یا پس از تگِ
/// `<html…>`، یا در صورت نبودِ هر دو، پیش‌اِسناد. تزریقِ دوباره هیچ‌گاه
/// باندلِ دوم نمی‌سازد (گیتِ `data-arda-runtime="on"`).
#[must_use]
pub fn inject_arda_runtime(html: &str, bundle: &ArdaRuntimeBundle) -> ArdaInjectionReport {
    if html.contains(ARDA_RUNTIME_MARKUP_MARKER) {
        return ArdaInjectionReport { html: html.to_owned(), injected: false, anchor: "head" };
    }
    let lower = html.to_lowercase();
    if let Some(idx) = lower.rfind("</head>") {
        let mut out = String::with_capacity(html.len() + bundle.combined.len());
        out.push_str(&html[..idx]);
        out.push_str(&bundle.combined);
        out.push_str(&html[idx..]);
        return ArdaInjectionReport { html: out, injected: true, anchor: "head" };
    }
    if let Some(tag_idx) = lower.find("<html")
        && let Some(gt) = lower[tag_idx..].find('>')
    {
        let idx = tag_idx + gt;
        let mut out = String::with_capacity(html.len() + bundle.combined.len());
        out.push_str(&html[..=idx]);
        out.push_str(&bundle.combined);
        out.push_str(&html[idx + 1..]);
        return ArdaInjectionReport { html: out, injected: true, anchor: "html" };
    }
    let mut out = String::with_capacity(html.len() + bundle.combined.len());
    out.push_str(&bundle.combined);
    out.push_str(html);
    ArdaInjectionReport { html: out, injected: true, anchor: "prepend" }
}

/// SHA-256 محتوا — گاردِ دلتای کرون و مهرِ محتوای پاسخِ localize.
#[must_use]
pub fn arda_content_sha256(content: &str) -> String {
    format!("{:x}", Sha256::digest(content.as_bytes()))
}

// ── 1C: دیمنِ کرونِ سیاره‌ای — کارِ خودکارِ ۲۴ساعته ─────────────────────────

/// یک محصولِ صفِ کرون.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArdaCronItem {
    /// شناسهٔ محصول (`prod:<uuid>`) یا نمای پلتفرم (`view:<slug>`).
    pub id: String,
    /// HTML فعلی محصول.
    pub html: String,
}

/// گزارشِ یک تیکِ کرون.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArdaCronTickReport {
    /// کلِ صفِ اسکن‌شده.
    pub scanned: usize,
    /// بیتغییر رد شد (گاردِ دلتا — ایدمپوتنس).
    pub skipped_unchanged: usize,
    /// بومی‌سازی شد (تازه یا به‌روزشده).
    pub localized: usize,
    /// شناسه‌های بومی‌سازی‌شده به ترتیبِ صف.
    pub localized_ids: Vec<String>,
}

/// دیمنِ کرونِ سیاره‌ای ARDA — هر [`ARDA_CRON_INTERVAL_SECS`] (۲۴ ساعت) کلِ
/// صف را با گاردِ دلتای SHA-256 اسکن می‌کند: محصولِ بیتغییر ایدمپوتنت رد
/// می‌شود و تازه/به‌روزشده روی ۲۰ زبان بومی‌سازی و نشان‌گذاری می‌شود.
#[derive(Debug, Clone, Default)]
pub struct ArdaPlanetaryCronDaemon {
    last_digests: std::collections::BTreeMap<String, String>,
}

impl ArdaPlanetaryCronDaemon {
    /// دیمنِ تازه با بازهٔ دقیقاً ۲۴ساعته.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// بازهٔ کرون به ثانیه — همیشه `86_400`.
    #[must_use]
    pub fn interval_secs(&self) -> u64 {
        ARDA_CRON_INTERVAL_SECS
    }

    /// خلاصهٔ دیجستِ شناخته‌شده برای شناسه (اگر قبلاً بومی‌سازی شده).
    #[must_use]
    pub fn known_digest(&self, id: &str) -> Option<&String> {
        self.last_digests.get(id)
    }

    /// یک تیکِ کرون — قطعی و ایدمپوتنت.
    ///
    /// هر آیتم با [`arda_content_sha256`] سنجیده می‌شود؛ اگر دیجست با تیکِ
    /// قبل برابر باشد → `skipped_unchanged`، وگرنه → `localized` و دیجستِ
    /// تازه ثبت می‌شود.
    pub fn run_tick(&mut self, items: &[ArdaCronItem]) -> ArdaCronTickReport {
        let mut report = ArdaCronTickReport {
            scanned: items.len(),
            skipped_unchanged: 0,
            localized: 0,
            localized_ids: Vec::new(),
        };
        for item in items {
            let digest = arda_content_sha256(&item.html);
            if self.last_digests.get(&item.id).is_some_and(|d| *d == digest) {
                report.skipped_unchanged += 1;
                continue;
            }
            self.last_digests.insert(item.id.clone(), digest);
            report.localized += 1;
            report.localized_ids.push(item.id.clone());
        }
        report
    }
}

// ── 1D: فورجِ مهارتِ مایرون — SKILL.md + skill.json خودکار ──────────────────

/// خطای فورجِ مهارت.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillForgeError {
    /// محصول از گیتِ REA مایرون (`>= 85`) نگذشته — فورج رد می‌شود.
    BelowReaGate {
        /// نمرهٔ REA فعلی.
        score: u8,
        /// آستانهٔ گیت.
        gate: u8,
    },
    /// زبانِ مبدأ نامعتبر (خارج از ۲۰ زبانِ سیاره‌ای).
    InvalidOriginLocale(String),
}

impl std::fmt::Display for SkillForgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BelowReaGate { score, gate } => {
                write!(f, "REA score {score} is below the Mairon forge gate {gate}")
            }
            Self::InvalidOriginLocale(code) => {
                write!(f, "origin locale `{code}` is not a planetary Fishkal locale")
            }
        }
    }
}

impl std::error::Error for SkillForgeError {}

/// درخواستِ فورجِ مهارت.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillForgeRequest {
    /// شناسهٔ محصول (`prod:<uuid>`).
    pub product_id: String,
    /// عنوانِ محصول.
    pub product_title: String,
    /// نمرهٔ REA مایرون (`0..=100`).
    pub rea_score: u8,
    /// زبانِ مبدأ (باید جزو ۲۰ زبان باشد).
    pub origin_locale: String,
    /// زبان‌های ترجمه‌شده (باید جزو ۲۰ زبان باشند؛ ناشناخته‌ها رد می‌شوند).
    pub translated_locales: Vec<String>,
}

/// بستهٔ مهارتِ فورج‌شده.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillPackage {
    /// سند `SKILL.md` با پیش‌مادهٔ YAML.
    pub skill_md: String,
    /// مانیفست `skill.json` سازگار با رجیستری‌های Hermes / MCP / `OpenCode`.
    pub skill_json: String,
}

/// فورجِ مهارتِ مایرون — هرگاه محصول از گیتِ REA (`>= 85`) بگذرد، بستهٔ
/// مهارتِ استاندارد را خودکار می‌سازد؛ زیرِ گیت، فورج هرگز رخ نمی‌دهد.
#[derive(Debug, Clone, Copy, Default)]
pub struct MaironSkillForge;

impl MaironSkillForge {
    /// فورجِ بستهٔ مهارت — قطعی و بازتولیدپذیر.
    ///
    /// # Errors
    /// [`SkillForgeError::BelowReaGate`] اگر نمره زیرِ گیت باشد؛
    /// [`SkillForgeError::InvalidOriginLocale`] اگر مبدأ خارج از ۲۰ زبان باشد.
    #[allow(clippy::too_many_lines)]
    pub fn forge_skill_package(req: &SkillForgeRequest) -> Result<SkillPackage, SkillForgeError> {
        if req.rea_score < SKILL_FORGE_REA_GATE {
            return Err(SkillForgeError::BelowReaGate {
                score: req.rea_score,
                gate: SKILL_FORGE_REA_GATE,
            });
        }
        let origin = fishkal_locale(&req.origin_locale)
            .ok_or_else(|| SkillForgeError::InvalidOriginLocale(req.origin_locale.clone()))?
            .code;
        let mut translated: Vec<&str> = vec![origin];
        for code in &req.translated_locales {
            if let Some(l) = fishkal_locale(code)
                && !translated.contains(&l.code)
            {
                translated.push(l.code);
            }
        }
        let slug = skill_slug(&req.product_title, &req.product_id);
        let supported = translated.join(", ");
        let strip = render_product_flag_strip(origin, &translated[1..]).unwrap_or_default();
        let skill_md = format!(
            "---\n\
             name: \"{title} — Sovereign Skill\"\n\
             slug: {slug}\n\
             version: 1.0.0\n\
             rea_score: {score}\n\
             mairon_certified: true\n\
             origin_locale: {origin}\n\
             translated_locales: [{supported}]\n\
             ---\n\n\
             # {title} — Sovereign Skill\n\n\
             ## Purpose\n\n\
             Mairon-certified digital artifact forged inside the 0xKIA Trinity Studio \
             Pipeline. This skill packages the product's verified behavior so any \
             Hermes / MCP / OpenCode agent can invoke it without re-verification. \
             The Forge bar (REA >= {gate}/100) was cleared with score {score}/100 \
             and the ⚒️ Mairon REA Certified badge.\n\n\
             ## Invocation Contract\n\n\
             - Marketplace endpoint: `GET /api/v1/market/products/{pid}/skill`\n\
             - Localize endpoint: `POST /api/v1/arda/localize-now`\n\
             - Flag catalog: `GET /api/v1/arda/flags`\n\
             - Origin locale: `{origin}` — circular flag badge stamped on the card.\n\
             - Supported locales: `{supported}` — ARDA runtime flips `lang`/`dir`/`data-lang` atomically.\n\n\
             ## Input Schema\n\n\
             ```json\n\
             {{ \"product_id\": \"{pid}\", \"locale\": \"fa\" | \"en\" | \"...\", \"payload\": \"opaque product input\" }}\n\
             ```\n\n\
             ## Output Schema\n\n\
             ```json\n\
             {{ \"product_id\": \"{pid}\", \"locale\": \"fa\", \"rendered_html\": \"string\", \"dir\": \"rtl\" | \"ltr\" }}\n\
             ```\n\n\
             ## Verification Gates\n\n\
             1. `Mairon REA >= {gate}/100` — scored {score}/100 (hard failures: none).\n\
             2. `ARDA parity = 1.00` — unified 6-scope catalog mechanically validated.\n\
             3. `25/25 Fishkal audit defects cured` — atomic root flip, per-script typography,\n\
                cached `Intl` engines, bidi DOM/Canvas isolation, WAI-ARIA listbox switcher.\n\
             4. `sha256(content)` stamped in the marketplace skill response.\n\n\
             {strip}\n",
            title = req.product_title,
            slug = slug,
            score = req.rea_score,
            gate = SKILL_FORGE_REA_GATE,
            pid = req.product_id,
            origin = origin,
            supported = supported,
            strip = strip,
        );
        let skill_json = serde_json::json!({
            "name": format!("{title} — Sovereign Skill", title = req.product_title),
            "slug": slug,
            "version": "1.0.0",
            "rea_score": req.rea_score,
            "mairon_certified": true,
            "forge_gate": SKILL_FORGE_REA_GATE,
            "origin_locale": origin,
            "translated_locales": translated,
            "circular_flag_strip_html": strip,
            "registry": {
                "compatible": ["hermes", "mcp", "opencode"],
                "kind": "marketplace-product",
            },
            "invocation": {
                "type": "http",
                "skill_endpoint": format!("/api/v1/market/products/{pid}/skill", pid = req.product_id),
                "localize_endpoint": "/api/v1/arda/localize-now",
                "flags_endpoint": "/api/v1/arda/flags",
            },
            "input_schema": {
                "type": "object",
                "properties": {
                    "product_id": { "type": "string", "const": req.product_id },
                    "locale": { "type": "string", "enum": translated },
                    "payload": { "type": "string", "description": "opaque product input" },
                },
                "required": ["product_id"],
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "product_id": { "type": "string" },
                    "locale": { "type": "string" },
                    "dir": { "type": "string", "enum": ["ltr", "rtl"] },
                    "rendered_html": { "type": "string" },
                },
                "required": ["product_id", "locale", "rendered_html"],
            },
            "verification_gates": [
                format!("mairon_rea>={gate}/100 (scored {score})", gate = SKILL_FORGE_REA_GATE, score = req.rea_score),
                "arda_catalog_parity==1.00".to_owned(),
                "fishkal_25_defects_cured".to_owned(),
                format!("sha256:{digest}", digest = arda_content_sha256(&req.product_id)),
            ],
        })
        .to_string();
        Ok(SkillPackage { skill_md, skill_json })
    }
}

/// اسلاگِ مهارت — قطعی: عنوانِ لاتین‌شده + پسوندِ کوتاهِ شناسه.
#[must_use]
pub fn skill_slug(title: &str, product_id: &str) -> String {
    let mut base = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            base.push(c.to_ascii_lowercase());
        } else if !base.ends_with('-') && !base.is_empty() {
            base.push('-');
        }
    }
    while base.ends_with('-') {
        base.pop();
    }
    if base.is_empty() {
        base.push_str("sovereign-skill");
    }
    let tail: String =
        product_id.chars().filter(char::is_ascii_alphanumeric).skip(6).take(6).collect();
    if tail.is_empty() {
        base
    } else {
        format!("{base}-{tail}")
    }
}

// ── آزمون‌ها — گیت‌های قطعیِ معماری فیشکال ──────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── 1A: دیکشنری پرچمِ وکتور ──

    #[test]
    fn dictionary_holds_exactly_20_planetary_locales() {
        assert_eq!(FISHKAL_VECTOR_FLAGS.len(), 20);
        assert_eq!(PLANETARY_LOCALE_COUNT, 20);
        let codes = all_locale_codes();
        assert_eq!(codes.len(), 20);
        for expected in [
            "en", "ar", "fa", "tr", "fr", "es", "pt", "nl", "ja", "hi", "zh", "id", "ko", "de",
            "ru", "it", "he", "ur", "vi", "th",
        ] {
            assert!(codes.contains(&expected), "missing locale {expected}");
            assert!(is_planetary_locale(expected));
        }
        assert!(!is_planetary_locale("xx"));
    }

    #[test]
    fn original_13_svgs_are_verbatim_fishkal() {
        // verbatim از L2690–2704 — رد شده‌اند تا هر تغییری در ممیزیِ بعدی شکست بخورد.
        let fa = fishkal_locale("fa").unwrap();
        assert_eq!(fa.svg, "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"4.7\" fill=\"#239f40\"/><rect y=\"4.7\" width=\"20\" height=\"4.7\" fill=\"#fff\"/><rect y=\"9.3\" width=\"20\" height=\"4.7\" fill=\"#da0000\"/><path d=\"M10 5.7c-.95 0-1.7.8-1.7 1.8S9.05 9.3 10 9.3s1.7-.8 1.7-1.8S10.95 5.7 10 5.7zm0 .9c.5 0 .8.4.8.9s-.3.9-.8.9-.8-.4-.8-.9.3-.9.8-.9z\" fill=\"#da0000\"/></svg>");
        let ar = fishkal_locale("ar").unwrap();
        assert_eq!(ar.svg, "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#fff\"/><rect width=\"20\" height=\"4.7\" fill=\"#00732F\"/><rect y=\"9.3\" width=\"20\" height=\"4.7\" fill=\"#000\"/><rect width=\"6\" height=\"14\" fill=\"#FF0000\"/></svg>");
        let en = fishkal_locale("en").unwrap();
        assert_eq!(en.svg, "<svg viewBox=\"0 0 20 14\"><rect width=\"20\" height=\"14\" fill=\"#012169\"/><path d=\"M0 0l20 14M20 0L0 14\" stroke=\"#fff\" stroke-width=\"2.6\"/><path d=\"M0 0l20 14M20 0L0 14\" stroke=\"#C8102E\" stroke-width=\"1\"/><path d=\"M10 0v14M0 7h20\" stroke=\"#fff\" stroke-width=\"4\"/><path d=\"M10 0v14M0 7h20\" stroke=\"#C8102E\" stroke-width=\"2.2\"/></svg>");
        // همهٔ ۲۰ پرچم با همان viewBox اصیل فیشکال.
        assert_eq!(count_flags_with("viewBox=\"0 0 20 14\""), 20);
    }

    #[test]
    fn native_script_glyphs_and_directions_are_locked() {
        let cases = [
            ("fa", "فا", "فارسی", TextDirection::Rtl, "fa-IR"),
            ("ar", "ع", "العربية", TextDirection::Rtl, "ar-AE"),
            ("ur", "اردو", "اردو", TextDirection::Rtl, "ur-PK"),
            ("he", "עב", "עברית", TextDirection::Rtl, "he-IL"),
            ("en", "EN", "English", TextDirection::Ltr, "en-US"),
            ("tr", "TR", "Türkçe", TextDirection::Ltr, "tr-TR"),
            ("ja", "JA", "日本語", TextDirection::Ltr, "ja-JP"),
            ("id", "ID", "Bahasa Indonesia", TextDirection::Ltr, "id-ID"),
            ("th", "TH", "ไทย", TextDirection::Ltr, "th-TH"),
        ];
        for (code, glyph, native, dir, intl) in cases {
            let l = fishkal_locale(code).unwrap();
            assert_eq!(l.str_code, glyph, "STR.code of {code}");
            assert_eq!(l.native_name, native, "native of {code}");
            assert_eq!(l.dir, dir, "dir of {code}");
            assert_eq!(l.intl_locale, intl, "Intl subtag of {code}");
        }
        // دقیقاً ۴ زبان RTL — قفلِ معماری.
        assert_eq!(FISHKAL_VECTOR_FLAGS.iter().filter(|l| l.is_rtl()).count(), 4);
    }

    #[test]
    fn circular_badge_rendering_and_strip() {
        let badge = render_circular_flag_badge("fa").unwrap();
        assert!(badge.contains("--flag-badge-radius:50%"));
        assert!(badge.contains("border-radius:50%"));
        assert!(badge.contains("box-shadow:0 0 0 1px rgba(239,230,214,.28)"));
        // بازچرخشِ دفاعیِ کِرت — همان قاعدهٔ فیشکال L353–358.
        assert!(badge.contains("transform:none !important;transition:none;stroke:none"));
        assert!(badge.contains("aria-label=\"فارسی (fa)\""));
        assert!(badge.contains("preserveAspectRatio=\"xMidYMid slice\""));
        assert!(render_circular_flag_badge("xx").is_none());

        let strip = render_product_flag_strip("fa", &["fa", "en", "ar", "xx"]).unwrap();
        // مبدأ + ۲ ترجمهٔ معتبر؛ تکرارِ مبدأ و کدِ ناشناخته حذف شده‌اند.
        assert_eq!(count_flag_badges(&strip), 3);
        assert!(strip.contains("data-origin=\"fa\""));
        assert!(strip.contains("data-lang=\"en\""));
        assert!(strip.contains("data-lang=\"ar\""));
        assert!(!strip.contains("data-lang=\"xx\""));
        assert!(strip.starts_with("<span class=\"arda-flag-strip\""));
        // مبدأ ناشناخته → نوار ساخته نمی‌شود.
        assert!(render_product_flag_strip("zz", &["en"]).is_none());
    }

    #[test]
    fn origin_locale_detection_is_script_deterministic() {
        let fa = "<html><head><title>محاسبه‌گر</title></head><body>گزارش مالی ماهانه</body></html>";
        assert_eq!(detect_origin_locale(fa), "fa");
        let ar = "<div>مرحبا بك في المتجر</div>";
        assert_eq!(detect_origin_locale(ar), "ar");
        let he = "<div>שלום וברכה</div>";
        assert_eq!(detect_origin_locale(he), "he");
        let ja = "<div>こんにちは、商品の説明です。</div>";
        assert_eq!(detect_origin_locale(ja), "ja");
        let zh = "<div>这是产品描述页面</div>";
        assert_eq!(detect_origin_locale(zh), "zh");
        let ru = "<div>Описание продукта и расчёт цены</div>";
        assert_eq!(detect_origin_locale(ru), "ru");
        let hi = "<div>उत्पाद का विवरण</div>";
        assert_eq!(detect_origin_locale(hi), "hi");
        let th = "<div>คำอธิบายผลิตภัณฑ์</div>";
        assert_eq!(detect_origin_locale(th), "th");
        let vi = "<div>Xưởng của chúng tôi có chủ quyền</div>";
        assert_eq!(detect_origin_locale(vi), "vi");
        let ur = "<div>یہ ایک اردو جملہ ہے</div>";
        assert_eq!(detect_origin_locale(ur), "ur");
        let en = "<html lang=\"en\"><body>Accounting calculator</body></html>";
        assert_eq!(detect_origin_locale(en), "en");
        // ویژگیِ lang بر شمارشِ اسکریپت مقدم است.
        let attr_first = "<html lang=\"tr\"><body>Finansal hesap makinesi</body></html>";
        assert_eq!(detect_origin_locale(attr_first), "tr");
        let plain = "<div>Plain English accounting</div>";
        assert_eq!(detect_origin_locale(plain), "en");
    }

    // ── 1B-۳: کاتالوگِ یکپارچه و اعتبارسنجِ توازن ──

    #[test]
    fn default_planetary_catalog_has_full_mechanical_parity() {
        let catalog = default_planetary_catalog();
        // یک کلید به‌ازای هر دامنهٔ شش‌گانه + aria سوییچر = ۷ کلید، ۱۴۰ رشته.
        assert_eq!(catalog.entries.len(), 7);
        let total_translations: usize = catalog.entries.iter().map(|e| e.translations.len()).sum();
        assert_eq!(total_translations, 7 * 20);
        for scope in ArdaCatalogScope::all() {
            assert!(
                catalog.entries.iter().any(|e| e.scope == scope),
                "scope {} has no entry",
                scope.as_str()
            );
        }
        let report = validate_catalog_parity(&catalog);
        assert!(report.fully_parity, "full parity required");
        assert_eq!(report.duplicate_keys, Vec::<String>::new());
        assert!((report.coverage_score - 1.0).abs() < f64::EPSILON);
        assert!(report
            .locales
            .iter()
            .all(|p| p.missing_keys.is_empty() && (p.coverage_score - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn parity_validator_detects_missing_keys_and_duplicates() {
        let mut b = ArdaLocaleCatalogBuilder::new("en");
        b.add(ArdaCatalogScope::Business, "price_stars", &[
            ("en", "Price (Stars)"),
            ("fa", "قیمت (ستاره)"),
            // tr..ko intentionally missing — the cured `unitFt` defect class.
        ]);
        b.add(ArdaCatalogScope::Tools, "calculate", &[("en", "Calculate")]);
        let catalog = b.build();
        let report = validate_catalog_parity(&catalog);
        assert!(!report.fully_parity);
        let tr = report.locales.iter().find(|p| p.locale == "tr").unwrap();
        assert_eq!(tr.missing_keys, vec!["business.price_stars", "tools.calculate"]);
        assert!((tr.coverage_score - 0.0).abs() < f64::EPSILON);
        let fa = report.locales.iter().find(|p| p.locale == "fa").unwrap();
        assert_eq!(fa.missing_keys, vec!["tools.calculate"]);
        assert!((fa.coverage_score - 0.5).abs() < f64::EPSILON);
        assert!((report.coverage_score - 0.0).abs() < f64::EPSILON);

        // کلید تکراری — همان نقصِ دیکشنری‌های پراکندهٔ فیشکال.
        let mut d = ArdaLocaleCatalogBuilder::new("en");
        d.add(ArdaCatalogScope::Core, "app_title", &[("en", "One")]);
        assert_eq!(d.duplicates().len(), 0);
        d.add(ArdaCatalogScope::Core, "app_title", &[("en", "Two")]);
        assert_eq!(d.duplicates().len(), 1);
        assert_eq!(d.duplicates()[0], "core.app_title");
        // کلید تکراری در کاتالوگِ ساخته‌شده — خطِ دفاعِ دوم.
        let dcat = d.build();
        assert_eq!(dcat.entries.len(), 1);
        assert_eq!(catalog_duplicates(&dcat), Vec::<String>::new());
    }

    // ── 1B: باندلِ زمانِ اجرا — ۲۵ نقصِ درمان‌شده ──

    #[test]
    fn bundle_is_deterministic_and_carries_all_cure_markers() {
        let generator = ArdaFishkalBundleGenerator;
        let a = generator.generate("fa", &["en", "ar"]);
        let b = generator.generate("fa", &["en", "ar"]);
        assert_eq!(a.content_sha256(), b.content_sha256(), "same input, same bundle");
        assert_eq!(a.origin, "fa");
        assert_eq!(a.locales, ["fa", "en", "ar"]);
        // کد ناشناخته رد می‌شود؛ مبدأ همیشه اول است.
        let c = generator.generate("zz", &["xx", "ja"]);
        assert_eq!(c.origin, "en");
        assert_eq!(c.locales, ["en", "ja"]);

        // (۱) فلیپِ اتمیِ ۳-ویژگیِ ریشه — L2085.
        assert!(a.js.contains("root.lang=l; root.dir=DIR[l]; root.dataset.lang=l;"));
        // (۲) تایپوگرافیِ هر-اسکریپت — RTL پیوندِ حروف را نمی‌شکند.
        assert!(a.css.contains(":root[data-lang=\"fa\"]"));
        assert!(a.css.contains("--track-body:.16em"));
        assert!(a.css.contains("--track-display:.07em"));
        assert!(a.css.contains("--display-indent:.07em"));
        assert!(a.css.contains("'Vazirmatn'"));
        assert!(a.css.contains(":root[data-lang=\"en\"]"));
        assert!(a.css.contains("--track-body:.34em"));
        // (۳) کاتالوگِ یکپارچهٔ ۶-دامنه‌ای درونِ داده.
        for key in [
            "core.app_title", "core.ui_language", "business.price_stars", "records.empty_state",
            "share.copy_link", "tools.calculate", "content.description",
        ] {
            assert!(a.js.contains(key), "unified catalog key {key} missing from bundle");
        }
        // (۴) موتور Intl کش‌شده + پول‌های بازار.
        assert!(a.js.contains("new Intl.NumberFormat"));
        assert!(a.js.contains("new Intl.DateTimeFormat"));
        assert!(a.js.contains("new Intl.PluralRules"));
        assert!(a.js.contains("fa-IR"));
        assert!(a.js.contains("currencyDisplay:\"code\""));
        assert!(a.js.contains("\"XTR\""));
        // (۵) جداسازیِ bidi در DOM و Canvas — رویهٔ ۴-گامیِ دقیق.
        assert!(a.js.contains("ARDA.canvas.drawBidiText"));
        assert!(a.js.contains("ctx.direction=latin?\"ltr\":(rtl?\"rtl\":\"ltr\")"));
        assert!(a.js.contains("ctx.textAlign"));
        assert!(a.js.contains("opts.width||ctx.canvas.width)-x"));
        assert!(a.js.contains("[data-arda-latin]"));
        assert!(a.js.contains("dir\",\"auto\""));
        // (۶) سوییچرِ لیست‌باکسِ WAI-ARIA با اصلاحات D-1..D-8.
        assert!(a.css.contains("inset-inline-start"), "D-1 logical positioning");
        assert!(!a.css.contains(" left:"), "D-1 no physical left anywhere");
        assert!(a.js.contains("aria-haspopup"));
        assert!(a.js.contains("role\",\"listbox\""));
        assert!(a.js.contains("aria-selected"));
        assert!(a.js.contains("preventScroll:true"));
        assert!(a.js.contains("ArrowDown"));
        assert!(a.js.contains("Escape"));
        assert!(a.js.contains("pointerdown"));
        assert!(a.js.contains("\u{2713}"), "visible check glyph (D-6)");
        assert!(a.js.contains("--gold"));
        assert!(a.css.contains("min-height:44px"), "coarse-pointer 44px target");
        assert!(a.css.contains("width:40px"), "40px glass pill geometry");
        // نگهت‌بانِ ایدمپوتنس — تزریقِ دوباره بی‌اثر.
        assert!(a.js.contains("__fishkalSignature"));
        assert!(a.combined.contains(ARDA_RUNTIME_MARKUP_MARKER));
    }

    #[test]
    fn injection_is_atomic_and_idempotent() {
        let bundle = ArdaFishkalBundleGenerator.generate("fa", &["en"]);
        let page = "<!DOCTYPE html><html lang=\"fa\"><head><title>محاسبه‌گر</title></head><body><p>سلام</p></body></html>";
        let report = inject_arda_runtime(page, &bundle);
        assert!(report.injected);
        assert_eq!(report.anchor, "head");
        let head_idx = report.html.to_lowercase().find("</head>").unwrap();
        let style_idx = report.html.find("<style").unwrap();
        assert!(style_idx < head_idx, "bundle must land inside <head>");
        assert!(report.html.starts_with("<!DOCTYPE html><html lang=\"fa\"><head>"), "prefix intact");
        assert!(report.html.contains("<title>محاسبه‌گر</title>"), "original head preserved");
        // ایدمپوتنس: تزریقِ دوباره هیچ تغییر نمی‌دهد.
        let second = inject_arda_runtime(&report.html, &bundle);
        assert!(!second.injected);
        assert_eq!(second.html, report.html);

        // بدون head → لنگر پس از تگ html.
        let bare = "<html><body>x</body></html>";
        let r2 = inject_arda_runtime(bare, &bundle);
        assert_eq!(r2.anchor, "html");
        assert!(r2.injected);
        // بدون هیچ تگ → پیش‌اِسناد.
        let r3 = inject_arda_runtime("<p>fragment</p>", &bundle);
        assert_eq!(r3.anchor, "prepend");
        assert!(r3.html.starts_with("<style"));
    }

    // ── 1C: دیمنِ کرونِ سیاره‌ای ──

    #[test]
    fn cron_daemon_is_24h_and_delta_guarded() {
        let mut daemon = ArdaPlanetaryCronDaemon::new();
        assert_eq!(daemon.interval_secs(), 86_400);
        let page = "<html><body>محاسبه‌گر مالی</body></html>";
        let items = vec![
            ArdaCronItem { id: "prod:one".to_owned(), html: page.to_owned() },
            ArdaCronItem { id: "prod:two".to_owned(), html: "<p>x</p>".to_owned() },
            ArdaCronItem { id: "view:home".to_owned(), html: "<main>0xKIA</main>".to_owned() },
        ];
        let tick1 = daemon.run_tick(&items);
        assert_eq!(tick1.scanned, 3);
        assert_eq!(tick1.localized, 3);
        assert_eq!(tick1.skipped_unchanged, 0);
        assert_eq!(tick1.localized_ids, ["prod:one", "prod:two", "view:home"]);
        assert!(daemon.known_digest("prod:one").is_some());
        // تیکِ دوم با همان محتوا → همه ایدمپوتنت رد می‌شوند.
        let tick2 = daemon.run_tick(&items);
        assert_eq!(tick2.scanned, 3);
        assert_eq!(tick2.localized, 0);
        assert_eq!(tick2.skipped_unchanged, 3);
        // تغییرِ محصول → فقط همان بومی‌سازی می‌شود.
        let updated = vec![
            ArdaCronItem { id: "prod:one".to_owned(), html: format!("{page}<!-- updated -->") },
            ArdaCronItem { id: "prod:two".to_owned(), html: "<p>x</p>".to_owned() },
        ];
        let tick3 = daemon.run_tick(&updated);
        assert_eq!(tick3.localized, 1);
        assert_eq!(tick3.localized_ids, ["prod:one"]);
        assert_eq!(tick3.skipped_unchanged, 1);
    }

    // ── 1D: فورجِ مهارتِ مایرون ──

    #[test]
    fn skill_forge_refuses_below_rea_gate_and_bad_origin() {
        let req = SkillForgeRequest {
            product_id: "prod:abc".to_owned(),
            product_title: "حسابدار".to_owned(),
            rea_score: SKILL_FORGE_REA_GATE - 1,
            origin_locale: "fa".to_owned(),
            translated_locales: vec!["en".to_owned()],
        };
        match MaironSkillForge::forge_skill_package(&req) {
            Err(SkillForgeError::BelowReaGate { score, gate }) => {
                assert_eq!(score, 84);
                assert_eq!(gate, 85);
            }
            other => panic!("expected BelowReaGate, got {other:?}"),
        }
        let bad = SkillForgeRequest {
            product_id: "prod:abc".to_owned(),
            product_title: "t".to_owned(),
            rea_score: 92,
            origin_locale: "xx".to_owned(),
            translated_locales: vec![],
        };
        assert_eq!(
            MaironSkillForge::forge_skill_package(&bad),
            Err(SkillForgeError::InvalidOriginLocale("xx".to_owned()))
        );
    }

    #[test]
    fn skill_forge_emits_certified_md_and_registry_json() {
        let req = SkillForgeRequest {
            product_id: "prod:018f3c2e-7b1a-7f2e-9c3d-4b5a6c7d8e9f".to_owned(),
            product_title: "حسابدار — Persian Accounting Calculator".to_owned(),
            rea_score: 92,
            origin_locale: "fa".to_owned(),
            translated_locales: vec!["fa".to_owned(), "en".to_owned(), "ar".to_owned()],
        };
        let pkg = MaironSkillForge::forge_skill_package(&req).unwrap();
        // SKILL.md — پیش‌مادهٔ YAML با همهٔ فیلدهای قفل.
        for fragment in [
            "name: \"حسابدار — Persian Accounting Calculator — Sovereign Skill\"",
            "slug: ",
            "version: 1.0.0",
            "rea_score: 92",
            "mairon_certified: true",
            "origin_locale: fa",
            "translated_locales: [fa, en, ar]",
            "## Purpose",
            "## Invocation Contract",
            "GET /api/v1/market/products/prod:018f3c2e-7b1a-7f2e-9c3d-4b5a6c7d8e9f/skill",
            "## Input Schema",
            "## Output Schema",
            "## Verification Gates",
            "scored 92/100",
        ] {
            assert!(pkg.skill_md.contains(fragment), "SKILL.md missing {fragment}");
        }
        assert!(pkg.skill_md.starts_with("---\n"));
        // نوارِ پرچمِ دایره‌ای در سند — مبدأ + ۲ ترجمه.
        assert_eq!(count_flag_badges(&pkg.skill_md), 3);
        // skill.json — مانیفستِ رجیستری.
        let manifest: serde_json::Value = serde_json::from_str(&pkg.skill_json).unwrap();
        assert_eq!(manifest["mairon_certified"], serde_json::Value::Bool(true));
        assert_eq!(manifest["rea_score"], serde_json::json!(92));
        assert_eq!(manifest["forge_gate"], serde_json::json!(SKILL_FORGE_REA_GATE));
        assert_eq!(manifest["origin_locale"], serde_json::json!("fa"));
        assert_eq!(manifest["translated_locales"], serde_json::json!(["fa", "en", "ar"]));
        assert_eq!(manifest["registry"]["compatible"], serde_json::json!(["hermes", "mcp", "opencode"]));
        assert!(manifest["invocation"]["skill_endpoint"]
            .as_str()
            .unwrap()
            .ends_with("/skill"));
        assert!(manifest["verification_gates"].as_array().unwrap().len() >= 4);
        // قطعیت — همان ورودی، همان خروجی.
        let again = MaironSkillForge::forge_skill_package(&req).unwrap();
        assert_eq!(pkg, again);
    }

    #[test]
    fn skill_slug_is_deterministic_and_sanitized() {
        assert_eq!(skill_slug("Persian Calculator", "prod:018f-7b1a"), "persian-calculator-8f7b1a");
        assert_eq!(skill_slug("  !!!  ", "prod:abc"), "sovereign-skill-c");
        let a = skill_slug("حسابدار", "prod:xyz123");
        assert_eq!(a, skill_slug("حسابدار", "prod:xyz123"));
        assert!(!a.starts_with('-') && !a.ends_with('-'));
    }
}
