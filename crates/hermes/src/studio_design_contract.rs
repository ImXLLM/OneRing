//! قرارداد طراحی استودیوی کیان — پورت `ui-ux-pro-max-skill` v2.15 +
//! `awesome-gpt-6-astra` (`website/design-qa.md`) + `androoAGI/starnet`
//! (`sidecar/compaction-fidelity.js`).
//!
//! سه قطعهٔ قطعی و تست‌شده:
//!
//! 1. [`DesignDials`] — سه اهرم طراحی `--variance` / `--motion` / `--density`
//!    از UI/UX Pro Max (دامنهٔ ۱..=۱۰ با گیرهٔ خودکار) + تشخیص خودکار RTL
//!    فارسی و پشتهٔ فونت `Vazirmatn`.
//! 2. [`AstraQualityGate`] — قانون «فریم اول تعاملی» گوگل‌شده از Astra:
//!    canvas/فرم روی فریم اول (بدون اسپلش مردهٔ بازاریابی)، سند تک‌فایلی
//!    کامل بدون ایمپورت محلی شکسته، توقف انیمیشن روی `visibilitychange`
//!    و صفر-سرریز افقی در موبایل ۳۹۰×۸۴۴.
//! 3. [`build_verbatim_user_messages_block`] — بودجهٔ ۵٪ پنجره (کف ۲٬۰۰۰ /
//!    سقف ۲۴٬۰۰۰ نویسه، کلاه Hermes) با سهم عادلانه‌ی water-fill؛ فشرده‌سازی
//!    هرگز مشخصات اصلی کاربر را از دست نمی‌دهد.
#![forbid(unsafe_code)]

// ── ۱) اهرم‌های طراحی UI/UX Pro Max v2.15 ───────────────────────────────────

/// سه اهرم طراحی — دقیقاً همان `--variance <1-10> --motion <1-10> --density <1-10>`
/// از `templates/base/skill-content.md` (خط ۱۶۷).
///
/// * variance: مرکزگرا/مینیمال (۱) ← متعادل/مدرن (۵) ← جسور/نامتقارن (۱۰)
/// * motion:  میکروتعامل ظریف (۱) ← اسکرول استاندارد (۵) ← کورئوگرافی پیچیده (۱۰)
/// * density: فضادار ۲۴-۹۶px (۱) ← استاندارد ۱۶-۶۴px (۵) ← داشبوردی ۸-۳۲px (۱۰)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesignDials {
    /// تنوع بصری (۱..=۱۰).
    pub variance: u8,
    /// شدت حرکت (۱..=۱۰).
    pub motion: u8,
    /// تراکم اطلاعات (۱..=۱۰).
    pub density: u8,
}

impl DesignDials {
    /// کران پایین دامنهٔ هر اهرم.
    pub const MIN: u8 = 1;
    /// کران بالای دامنهٔ هر اهرم.
    pub const MAX: u8 = 10;

    /// ساخت با گیرهٔ خودکار به دامنهٔ ۱..=۱۰ — ورودی خام هرگز ترک نمی‌شود.
    #[must_use]
    pub fn new(variance: u8, motion: u8, density: u8) -> Self {
        Self {
            variance: variance.clamp(Self::MIN, Self::MAX),
            motion: motion.clamp(Self::MIN, Self::MAX),
            density: density.clamp(Self::MIN, Self::MAX),
        }
    }

    /// لایهٔ variance — نگاشت سطر ۱۷۲ `skill-content.md`.
    #[must_use]
    pub fn variance_tier(&self) -> &'static str {
        match self.variance {
            1..=3 => "centered/minimal",
            4..=7 => "balanced/modern",
            _ => "bold/asymmetric",
        }
    }

    /// لایهٔ motion — نگاشت سطر ۱۷۳ `skill-content.md`.
    #[must_use]
    pub fn motion_tier(&self) -> &'static str {
        match self.motion {
            1..=3 => "subtle",
            4..=7 => "standard",
            _ => "complex",
        }
    }

    /// لایهٔ density — نگاشت سطر ۱۷۴ `skill-content.md`.
    #[must_use]
    pub fn density_tier(&self) -> &'static str {
        match self.density {
            1..=3 => "spacious",
            4..=7 => "standard",
            _ => "dense/dashboard",
        }
    }

    /// توصیهٔ زنجیرهٔ فونت RTL — خط ۹ قرارداد کیان (`Vazirmatn` برای فارسی/عربی).
    #[must_use]
    pub fn font_stack(&self, rtl: bool) -> &'static str {
        if rtl {
            RTL_FONT_STACK
        } else {
            LTR_FONT_STACK
        }
    }
}

/// پشتهٔ فونت قفل‌شدهٔ RTL فارسی (`dir="rtl"` + Vazirmatn).
pub const RTL_FONT_STACK: &str = "Vazirmatn, 'Noto Sans Arabic', system-ui, sans-serif";

/// پشتهٔ فونت پیش‌فرض LTR (فناوری/داده).
pub const LTR_FONT_STACK: &str = "'Space Grotesk', 'DM Sans', system-ui, sans-serif";

/// بلاک ویژگی‌های `<html>` برای سند فارسی — الزام خط ۹ قرارداد.
pub const RTL_HTML_ATTRS: &str = "dir=\"rtl\" lang=\"fa\"";

/// آیا پرامپت فارسی/عربی است؟ — وجود هر نویسه در بلوک
/// `U+0600..=U+06FF` (عربی/فارسی پایه) یعنی سند باید `dir="rtl"` و
/// پشتهٔ فونت [`RTL_FONT_STACK`] بگیرد.
#[must_use]
pub fn detect_persian_rtl(prompt: &str) -> bool {
    prompt
        .chars()
        .any(|c| ('\u{0600}'..='\u{06FF}').contains(&c))
}

/// تزریق قطعی `dir="rtl" lang="fa"` روی تگ `<html>` سند تولیدی.
///
/// اگر سند از قبل `dir=` داشته باشد دست‌نخورده برمی‌گردد (دوگانه‌کاری
/// ممنوع)؛ اگر تگ `<html>` ندارد، سند دست‌نخورده برمی‌گردد (فقط
/// اسناد کامل را می‌شناسیم).
#[must_use]
pub fn enforce_rtl(html: &str) -> String {
    let lower = html.to_lowercase();
    if lower.contains("dir=") || !lower.contains("<html") {
        return html.to_owned();
    }
    // اولین <html …> را پیدا کن و ویژگی‌ها را داخلش بگذار.
    match html.find('<') {
        Some(start) if html[start..].to_lowercase().starts_with("<html") => {
            let tag_end = html[start..].find('>').map_or(start, |i| start + i);
            let (tag, rest) = html.split_at(tag_end);
            format!("{tag} {RTL_HTML_ATTRS}{rest}")
        }
        _ => html.to_owned(),
    }
}

// ── ۲) دروازهٔ کیفیت Astra (فریم اول تعاملی) ────────────────────────────────

/// گزارش ممیزی استاتیک [`AstraQualityGate::audit`] — پنج پرچم بولی عمدی‌اند:
/// هر پرچم یک قانون مستقل دروازهٔ Astra است و نباید در enum/فیلد دیگر تلف شود.
#[allow(clippy::struct_excessive_bools)] // گزارش ممیزی — هر بولی یک قانون مستقل
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstraAudit {
    /// سطح تعاملی روی فریم اول (canvas/input/button/…) بدون اسپلش مرده.
    pub first_frame_interactive: bool,
    /// سند تک‌فایلی: doctype + بستن کامل + صفر ایمپورت محلی شکسته.
    pub self_contained_single_file: bool,
    /// توقف انیمیشن/تایمر روی `visibilitychange`.
    pub pauses_on_visibility_change: bool,
    /// متا‌تگ viewport موبایل با `width=device-width`.
    pub mobile_viewport_meta: bool,
    /// صفر-سرریز افقی استاتیک: هیچ عرض پیکسلی ثابت ≥ ۳۹۰px نیست.
    pub mobile_zero_overflow_static: bool,
}

impl AstraAudit {
    /// فهرست ایرادها به فارسی — برای بازخورد مستقیم به LLM استودیو.
    #[must_use]
    pub fn failures(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if !self.first_frame_interactive {
            out.push("فریم اول تعاملی نیست — سطح کار/canvas/فرم باید بی‌درنگ در viewport باشد؛ هیچ اسپلش بازاریابی مرده‌ای قبل از اپ مجاز نیست.");
        }
        if !self.self_contained_single_file {
            out.push("سند تک‌فایلی کامل نیست — <!DOCTYPE html> و </html> لازم است و هیچ ایمپورت محلی (src/href نسبی) شکسته‌ای مجاز نیست.");
        }
        if !self.pauses_on_visibility_change {
            out.push("انیمیشن/تایمر روی visibilitychange متوقف نمی‌شود.");
        }
        if !self.mobile_viewport_meta {
            out.push("متا‌تگ viewport با width=device-width پیدا نشد.");
        }
        if !self.mobile_zero_overflow_static {
            out.push("عرض پیکسلی ثابت ≥ ۳۹۰px یافت شد — در موبایل ۳۹۰×۸۴۴ سرریز افقی می‌سازد.");
        }
        out
    }

    /// عبور کامل از دروازهٔ Astra؟
    #[must_use]
    pub fn passed(&self) -> bool {
        self.first_frame_interactive
            && self.self_contained_single_file
            && self.pauses_on_visibility_change
            && self.mobile_viewport_meta
            && self.mobile_zero_overflow_static
    }
}

/// دروازهٔ کیفیت استودیو — قانون‌های گوگل‌شده از `design-qa.md` آسترا.
#[derive(Debug, Clone, Copy, Default)]
pub struct AstraQualityGate {
    _private: (),
}

impl AstraQualityGate {
    /// عرض مرجع موبایل (iPhone 14 Pro Max ≈ ۳۹۰).
    pub const MOBILE_WIDTH: u32 = 390;
    /// ارتفاع مرجع موبایل.
    pub const MOBILE_HEIGHT: u32 = 844;
    /// هر عرض پیکسلی ثابت بیش از این مقدار، در ۳۹۰px سرریز افقی می‌سازد
    /// (برابرِ ۳۹۰ حکماً سرریز نیست — `max-width:390px` الگوی استاندارد است).
    pub const OVERFLOW_WIDTH_PX: u32 = 390;

    /// ممیزی استاتیک سند HTML تولیدی — قطعی و بدون شبکه.
    ///
    /// * تعاملی‌بودن فریم اول: وجود `canvas`/`input`/`button`/`textarea`/
    ///   `select`/`contenteditable` **و** نبودِ دروازهٔ اسپلش
    ///   (`id="splash"`/`class="splash…"`).
    /// * تک‌فایلی‌بودن: `<!doctype html` + `</html>` + هیچ `src=`/`href=`
    ///   نسبی (فقط `https:`/`data:`/`blob:`/`#` مجازند).
    /// * سرریز استاتیک: اسکن `width:`/`min-width:`/`max-width:` و
    ///   `width="…"` برای مقدار پیکسلی > [`Self::OVERFLOW_WIDTH_PX`].
    #[must_use]
    pub fn audit(&self, html: &str) -> AstraAudit {
        let lower = html.to_lowercase();

        let has_surface = ["<canvas", "<input", "<button", "<textarea", "<select", "contenteditable"]
            .iter()
            .any(|m| lower.contains(m));
        let splash_gate = ["id=\"splash", "id='splash", "class=\"splash", "class='splash"]
            .iter()
            .any(|m| lower.contains(m));

        let doctype = lower.starts_with("<!doctype html") || lower.starts_with("<!doctype html ");
        let closed = lower.contains("</html>");
        let no_local_imports = !scan_local_imports(&lower);

        let viewport = lower.contains("<meta name=\"viewport\"")
            || lower.contains("<meta name='viewport'")
            || lower.contains("<meta  name=\"viewport\"");
        let device_width = lower.contains("width=device-width");

        AstraAudit {
            first_frame_interactive: has_surface && !splash_gate,
            self_contained_single_file: doctype && closed && no_local_imports,
            pauses_on_visibility_change: lower.contains("visibilitychange"),
            mobile_viewport_meta: viewport && device_width,
            mobile_zero_overflow_static: !scan_fixed_width_overflow(&lower),
        }
    }
}

/// اسکن ایمپورت محلی شکسته — هر `src=`/`href=` با مقدار نسبی (نه
/// `http`/`https`/`data:`/`blob:`/`#`/`mailto:`) یعنی سند به فایل کناری
/// وابسته است و از استودیو خارج نمی‌شود سالم.
fn scan_local_imports(lower_html: &str) -> bool {
    for attr in ["src=", "href="] {
        let mut from = 0;
        while let Some((next, value)) = find_attr_value(lower_html, attr, from) {
            from = next;
            let v = value.trim_start();
            let external = v.starts_with("http://")
                || v.starts_with("https://")
                || v.starts_with("data:")
                || v.starts_with("blob:")
                || v.starts_with('#')
                || v.starts_with("mailto:");
            if !external && !v.is_empty() {
                return true;
            }
        }
    }
    false
}

/// مقدار ویژگی بعدی `attr` از نقطهٔ `from` — بازگشت (ادامهٔ اسکن، مقدار).
fn find_attr_value<'a>(lower_html: &'a str, attr: &str, from: usize) -> Option<(usize, &'a str)> {
    let pos = lower_html[from..].find(attr)? + from;
    let after = pos + attr.len();
    let quote = lower_html.as_bytes().get(after).copied()?;
    if quote != b'"' && quote != b'\'' {
        return None; // بدون نقل‌قول → نگه‌داشتن سخت‌گیرانه امن است
    }
    let close = char::from(quote);
    let start = after + 1;
    let end = lower_html[start..].find(close)? + start;
    Some((end + 1, &lower_html[start..end]))
}

/// اسکن عرض ثابت > ۳۹۰px در CSS و ویژگی‌های `width="…"`.
fn scan_fixed_width_overflow(lower_html: &str) -> bool {
    // CSS: width:/min-width:/max-width: <digits>px
    for prop in ["width:", "min-width:", "max-width:"] {
        let mut from = 0;
        while let Some(pos) = lower_html[from..].find(prop) {
            let abs = from + pos + prop.len();
            let tail = &lower_html[abs..];
            let digits: String = tail
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            if let Ok(px) = digits.parse::<u32>() {
                // مقدار فاصلهٔ ابتدای tail که حذف شد — نه طول بخش بریده‌شده!
                let trimmed = tail.trim_start();
                let leading_ws = tail.len() - trimmed.len();
                let after_digits = abs + leading_ws + digits.len();
                let unit = lower_html[after_digits.min(lower_html.len())..]
                    .trim_start()
                    .to_lowercase();
                let is_px = unit.starts_with("px");
                let is_vw = unit.starts_with("vw");
                if is_px && px > AstraQualityGate::OVERFLOW_WIDTH_PX {
                    return true;
                }
                // vw > ۱۰۰٪ هم سرریز افقی می‌سازد (مثلاً width: 120vw).
                if is_vw && px > 100 {
                    return true;
                }
            }
            from = abs;
        }
    }
    // ویژگی HTML: width="450" / width=450
    let attr = "width=\"";
    let mut from = 0;
    while let Some(pos) = lower_html[from..].find(attr) {
        let abs = from + pos + attr.len();
        let digits: String = lower_html[abs..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if digits
            .parse::<u32>()
            .is_ok_and(|px| px > AstraQualityGate::OVERFLOW_WIDTH_PX)
        {
            return true;
        }
        from = abs;
    }
    false
}

// ── ۳) بلوک verbatim استارنت (فشرده‌سازی وفادار) ─────────────────────────────

/// تگ باز بلوک پیام‌های verbatim — عین `compaction-fidelity.js` سطر ۳۰.
pub const VERBATIM_OPEN: &str = "<user_messages_verbatim>";
/// تگ بستن بلوک پیام‌های verbatim.
pub const VERBATIM_CLOSE: &str = "</user_messages_verbatim>";

/// بودجهٔ بلوک verbatim: ~۵٪ پنجره، کف ۲٬۰۰۰ و سقف ۲۴٬۰۰۰ نویسه
/// (کلاه Hermes — سطر ۴۰ `compaction-fidelity.js`).
#[must_use]
pub fn verbatim_budget_chars(window_chars: usize) -> usize {
    (window_chars / 20).clamp(2_000, 24_000)
}

/// بلوک `<user_messages_verbatim>` با سهم عادلانهٔ water-fill.
///
/// قانون استارنت: «پیام کوتاه کامل، پیام بلند به سهمش بریده و علامت‌دار» —
/// مازاد بودجهٔ پیام‌های کوتاه به پیام‌های گرسنه بازتوزیع می‌شود (دو
/// گذر water-fill). مشخصات اصلی کاربر هرگز در فشرده‌سازی از دست نمی‌رود.
#[must_use]
pub fn build_verbatim_user_messages_block(messages: &[&str], window_chars: usize) -> String {
    use std::fmt::Write as _;

    if messages.is_empty() {
        return String::new();
    }
    let budget = verbatim_budget_chars(window_chars);
    let n = messages.len();

    let wants: Vec<usize> = messages.iter().map(|m| m.chars().count()).collect();
    let share = budget / n;
    let mut given: Vec<usize> = wants.iter().map(|w| (*w).min(share)).collect();

    // بازتوزیع مازاد سهم پیام‌های کامل به پیام‌های بریده‌شده.
    let mut leftover = budget.saturating_sub(given.iter().sum::<usize>());
    let hungry: Vec<usize> = (0..n)
        .filter(|&i| given[i] < wants[i])
        .collect();
    if !hungry.is_empty() {
        let extra = leftover / hungry.len();
        for &i in &hungry {
            let take = (wants[i] - given[i]).min(extra);
            given[i] += take;
            leftover -= take;
        }
        // گذر دوم: هر مازاد باقی‌مانده به اولین گرسنه‌ها می‌رسد.
        for &i in &hungry {
            if leftover == 0 {
                break;
            }
            let take = (wants[i] - given[i]).min(leftover);
            given[i] += take;
            leftover -= take;
        }
    }

    let mut out = String::from(VERBATIM_OPEN);
    out.push('\n');
    for (i, msg) in messages.iter().enumerate() {
        let kept: String = msg.chars().take(given[i]).collect();
        let _ = writeln!(out, "[#{} | {} chars]", i + 1, wants[i]);
        out.push_str(&kept);
        if given[i] < wants[i] {
            let _ = writeln!(out, "\n[+{} chars trimmed]", wants[i] - given[i]);
        } else {
            out.push('\n');
        }
    }
    out.push_str(VERBATIM_CLOSE);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── اهرم‌ها ──

    #[test]
    fn dials_clamp_to_1_10() {
        let d = DesignDials::new(0, 11, 255);
        assert_eq!(d.variance, 1);
        assert_eq!(d.motion, 10);
        assert_eq!(d.density, 10);
        let d = DesignDials::new(3, 7, 5);
        assert_eq!((d.variance, d.motion, d.density), (3, 7, 5));
    }

    #[test]
    fn dial_tiers_match_skill_content_md() {
        let lo = DesignDials::new(1, 2, 3);
        assert_eq!(lo.variance_tier(), "centered/minimal");
        assert_eq!(lo.motion_tier(), "subtle");
        assert_eq!(lo.density_tier(), "spacious");
        let mid = DesignDials::new(5, 5, 5);
        assert_eq!(mid.variance_tier(), "balanced/modern");
        assert_eq!(mid.motion_tier(), "standard");
        assert_eq!(mid.density_tier(), "standard");
        let hi = DesignDials::new(8, 9, 10);
        assert_eq!(hi.variance_tier(), "bold/asymmetric");
        assert_eq!(hi.motion_tier(), "complex");
        assert_eq!(hi.density_tier(), "dense/dashboard");
        // فونت RTL فقط برای فارسی — خط ۹ قرارداد.
        assert!(hi.font_stack(true).starts_with("Vazirmatn"));
        assert!(hi.font_stack(false).starts_with("'Space Grotesk'"));
    }

    // ── تشخیص RTL ──

    #[test]
    fn persian_rtl_detection_boundaries() {
        assert!(detect_persian_rtl("سلام دنیا"));
        assert!(detect_persian_rtl("ماشین‌حساب ریسک ترید با دارک نئون"));
        assert!(!detect_persian_rtl("hello world dashboard"));
        assert!(!detect_persian_rtl(""));
        // مرز پایین بلوک U+0600 (ARABIC NUMBER SIGN)
        assert!(detect_persian_rtl("\u{0600}"));
        // مرز بالای بلوک U+06FF
        assert!(detect_persian_rtl("\u{06FF}"));
        // خارج از بلوک — NFKC هم ملاک نیست، فقط این بازه
        assert!(!detect_persian_rtl("\u{0750}"));
        // ترکیبی: یک نویسهٔ فارسی در انگلیسی کافی است
        assert!(detect_persian_rtl("dark neon dashboard فارسی"));
    }

    #[test]
    fn enforce_rtl_injects_once_and_never_twice() {
        let doc = "<html><head><title>x</title></head><body>سلام</body></html>";
        let enforced = enforce_rtl(doc);
        assert!(enforced.starts_with("<html dir=\"rtl\" lang=\"fa\">"));
        assert_eq!(enforced.matches("dir=\"rtl\"").count(), 1);

        // از قبل RTL است — دست نمی‌خورد.
        let already = "<html dir=\"rtl\" lang=\"fa\"><body>سلام</body></html>";
        assert_eq!(enforce_rtl(already), already);
        // تگ html ندارد — دست نمی‌خورد.
        assert_eq!(enforce_rtl("<div>سلام</div>"), "<div>سلام</div>");
    }

    // ── دروازهٔ Astra ──

    const PASSING_DOC: &str = "<!DOCTYPE html><html dir=\"rtl\" lang=\"fa\"><head>\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
<style>body{width:100%;max-width:390px}</style></head>\
<body><canvas id=\"app\"></canvas><button>شروع</button>\
<script>document.addEventListener('visibilitychange',()=>{/*pause*/});</script>\
</body></html>";

    #[test]
    fn astra_gate_passes_good_document() {
        let a = AstraQualityGate::default().audit(PASSING_DOC);
        assert_eq!(a, AstraAudit {
            first_frame_interactive: true,
            self_contained_single_file: true,
            pauses_on_visibility_change: true,
            mobile_viewport_meta: true,
            mobile_zero_overflow_static: true,
        });
        assert!(a.passed());
        assert!(a.failures().is_empty());
    }

    #[test]
    fn astra_gate_flags_dead_marketing_splash() {
        let doc = "<!DOCTYPE html><html><head><meta name=\"viewport\" content=\"width=device-width\">\
</head><body><div id=\"splash\" class=\"splash-screen\">به سایت ما خوش آمدید…</div>\
<script>document.addEventListener('visibilitychange',()=>{});</script></body></html>";
        let a = AstraQualityGate::default().audit(doc);
        assert!(!a.first_frame_interactive, "splash gate must fail frame-1 rule");
        assert_eq!(a.failures().len(), 1);
    }

    #[test]
    fn astra_gate_flags_broken_local_import() {
        let doc = PASSING_DOC.replace("<canvas id=\"app\"></canvas>", "");
        let doc = doc.replace("</head>", "<script src=\"app.js\"></script></head>");
        let a = AstraQualityGate::default().audit(&doc);
        assert!(!a.self_contained_single_file, "relative src= must fail");
        // CDN https مجاز است — «ایمپورت محلی شکسته» فقط نسبی است.
        let cdn = PASSING_DOC.replace("</head>", "<script src=\"https://cdn.example/x.js\"></script></head>");
        assert!(AstraQualityGate::default().audit(&cdn).self_contained_single_file);
    }

    #[test]
    fn astra_gate_flags_missing_visibility_pause_and_wide_layout() {
        let no_pause = PASSING_DOC.replace("visibilitychange", "focuschange");
        let a = AstraQualityGate::default().audit(&no_pause);
        assert!(!a.pauses_on_visibility_change);

        let wide = PASSING_DOC.replace("max-width:390px", "width:720px");
        let a = AstraQualityGate::default().audit(&wide);
        assert!(!a.mobile_zero_overflow_static, "720px fixed width must fail 390 rule");

        // width="450" ویژگی‌ای هم سرریز است.
        let attr_wide = PASSING_DOC.replace("<canvas id=\"app\"></canvas>", "<canvas id=\"app\" width=\"450\"></canvas>");
        assert!(!AstraQualityGate::default().audit(&attr_wide).mobile_zero_overflow_static);
        // اما width:100% و vw ≤ ۱۰۰ مشکلی ندارند.
        assert!(AstraQualityGate::default().audit(PASSING_DOC).mobile_zero_overflow_static);
    }

    #[test]
    fn astra_gate_flags_missing_viewport_meta() {
        let no_vp = PASSING_DOC.replace("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">", "");
        let a = AstraQualityGate::default().audit(&no_vp);
        assert!(!a.mobile_viewport_meta);
    }

    // ── بلوک verbatim ──

    #[test]
    fn verbatim_budget_clamps_2000_to_24000() {
        assert_eq!(verbatim_budget_chars(10_000), 2_000, "500 → کف ۲٬۰۰۰");
        assert_eq!(verbatim_budget_chars(100_000), 5_000, "۵٪ دقیق");
        assert_eq!(verbatim_budget_chars(1_000_000), 24_000, "۵۰٬۰۰۰ → سقف ۲۴٬۰۰۰");
        assert_eq!(verbatim_budget_chars(0), 2_000);
    }

    #[test]
    fn verbatim_keeps_short_messages_whole() {
        let msgs = ["سلام", "دومی هم کوتاه است", "third is short"];
        let block = build_verbatim_user_messages_block(&msgs, 400_000);
        assert!(block.starts_with(VERBATIM_OPEN));
        assert!(block.ends_with(VERBATIM_CLOSE));
        for m in msgs {
            assert!(block.contains(m), "short message must survive whole: {m}");
        }
        assert!(!block.contains("trimmed"), "no message was cut");
    }

    #[test]
    fn verbatim_cuts_long_messages_to_fair_share() {
        let long = "x".repeat(50_000);
        let msgs = [long.as_str()];
        let block = build_verbatim_user_messages_block(&msgs, 100_000);
        let budget = verbatim_budget_chars(100_000); // ۵٬۰۰۰
        let content = block
            .lines()
            .filter(|l| !l.starts_with('[') && *l != VERBATIM_OPEN && *l != VERBATIM_CLOSE)
            .fold(0usize, |acc, l| acc + l.chars().count());
        assert!(content <= budget, "content {content} must fit budget {budget}");
        assert!(block.contains(&format!("[+{} chars trimmed]", 50_000 - content)));
    }

    #[test]
    fn verbatim_water_fill_gives_slack_to_long_messages() {
        let short = "s".repeat(200);
        let long = "L".repeat(20_000);
        let msgs = [short.as_str(), long.as_str()];
        let block = build_verbatim_user_messages_block(&msgs, 200_000);
        let budget = verbatim_budget_chars(200_000); // ۱۰٬۰۰۰
        // پیام کوتاه کامل مانده (بازتوزیع به پیام بلند رفته است)
        let short_line = block
            .lines()
            .find(|l| l.starts_with('s') && l.len() == 200)
            .expect("short message whole");
        assert_eq!(short_line.chars().count(), 200);
        // پیام بلند بیش از سهم خام (۵٬۰۰۰) گرفته — مازاد ۲۰۰ پیام کوتاه به او رسید
        let long_kept = block
            .lines()
            .find(|l| l.starts_with('L'))
            .expect("long message line")
            .chars()
            .count();
        assert!(long_kept > budget / 2, "long message must exceed raw share: {long_kept}");
        let total = block
            .lines()
            .filter(|l| !l.starts_with('[') && *l != VERBATIM_OPEN && *l != VERBATIM_CLOSE)
            .fold(0usize, |acc, l| acc + l.chars().count());
        assert!(total <= budget, "water-fill must respect budget: {total} > {budget}");
    }

    #[test]
    fn verbatim_empty_input_is_empty_string() {
        assert_eq!(build_verbatim_user_messages_block(&[], 100_000), "");
    }
}
