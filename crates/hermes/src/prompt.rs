//! قواعد سخت تایپوگرافی RTL — پورت `ShopifyHermes/bot.py` (پوشهٔ ۰۳ گنجینه).
//!
//! قانون طلایی بات هرمس: **هیچ‌گاه جملهٔ فارسی با واژهٔ لاتین خاتمه نمی‌یابد** —
//! توکن‌های فنی لاتین (مثل `HTML`، `JSON`، `USDT`، `TON`، `Escrow`) باید وسط
//! جمله بیایند و جمله با واژهٔ فارسی و نشانه‌گذاری فارسی تمام شود؛ وگرنه
//! رندر دوطرفه (bidi) متن را می‌شکند و پاسخ «خراب» دیده می‌شود.
//!
//! دو سطح دفاع:
//! 1. **قفل پرامپت** — [`rtl_rule_block`] به هر پرامپت سیستمی چسبانده می‌شود
//!    تا مدل از همان مبدأ درست بنویسد ([`apply_rtl_rule`]).
//! 2. **دروازهٔ خروجی** — [`ends_on_bare_latin`] تشخیص می‌دهد جمله با توکن
//!    لاتین تنها تمام شده یا نه؛ برای تست پاسخ‌ها و FAQهای ثابت.
#![forbid(unsafe_code)]

/// بلوک قانون RTL — عیناً از `SYSTEM` بات هرمس (پوشهٔ ۰۳) پورت شده.
pub const RTL_RULE_FA: &str = "قواعد نوشتن RTL (سخت):\n\
     - فارسیِ روان و کاملاً درست بنویس؛ هیچ کلمهٔ ناقص، قاطی یا بی‌معنی نساز.\n\
     - واژهٔ انگلیسی (مثل HTML یا JSON) را وسطِ جمله بیاور و جمله را با نقطه یا \
     علامتِ فارسی تمام کن؛ هیچ‌گاه جمله با واژهٔ انگلیسی تمام نشود.";

/// قفل RTL را به پرامپت سیستمی می‌چسباند — اگر قانون از قبل حاضر باشد دست‌نخورده برمی‌گردد.
#[must_use]
pub fn apply_rtl_rule(system_prompt: &str) -> String {
    if system_prompt.contains(RTL_RULE_FA) {
        return system_prompt.to_owned();
    }
    let mut out = system_prompt.trim_end().to_owned();
    out.push_str("\n\n");
    out.push_str(RTL_RULE_FA);
    out
}

/// متن با واژهٔ فارسی یا نشانهٔ فارسی تمام می‌شود؟
///
/// حرف آخرِ مؤثر باید فارسی (`U+0600..=U+06FF` یا نیم‌فاصله) یا نشانهٔ
/// نگارشی باشد — واژهٔ لاتین تنها در پایان یعنی نقض قانون.
#[must_use]
pub fn ends_on_bare_latin(text: &str) -> bool {
    let trimmed = text.trim_end();
    let Some(last) = trimmed.chars().next_back() else {
        return false; // خالی — بی‌طرف
    };
    if matches!(
        last,
        '.' | '!' | '?' | ':' | ';' | '،' | '؛' | '؟' | '…' | '»' | ')' | ']'
    ) {
        return false; // نشانهٔ نگارشی — قانون رعایت شده
    }
    if matches!(last, '\u{200c}') {
        // نیم‌فاصله — به حرف قبلش نگاه کن
        let stem = trimmed.trim_end_matches('\u{200c}');
        return stem.chars().next_back().is_some_and(|c| c.is_ascii());
    }
    last.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// قانون یک‌بار می‌چسبد و دوبار چسباندن، آن را تکرار نمی‌کند (idempotent).
    #[test]
    fn rtl_rule_is_idempotent() {
        let base = "تو هرمس هستی.";
        let once = apply_rtl_rule(base);
        assert!(once.contains("هیچ‌گاه جمله با واژهٔ انگلیسی تمام نشود"));
        assert_eq!(apply_rtl_rule(&once), once, "دوبار چسباندن نباید قانون را تکرار کند");
    }

    /// پرامپت هرمس سازنده پس از قفل RTL، قانون را دارد و پایانش فارسی است.
    #[test]
    fn builder_system_prompt_carries_rtl_rule() {
        let prompt = crate::builder::build_system_prompt("discovery", false);
        let locked = apply_rtl_rule(&prompt);
        assert!(locked.contains(RTL_RULE_FA));
        assert!(
            !ends_on_bare_latin(&locked),
            "پرامپت باید با فارسی یا نشانه تمام شود: …{}",
            locked.chars().rev().take(12).collect::<String>()
        );
    }

    /// دروازهٔ خروجی — پایان لاتین تنها → نقض؛ پایان فارسی/نشانه → سالم.
    #[test]
    fn ends_on_bare_latin_detects_broken_rtl() {
        assert!(ends_on_bare_latin("محصول را با HTML"), "جمله با واژهٔ لاتین تمام شده");
        assert!(ends_on_bare_latin("قیمت بر حسب USDT"));
        assert!(!ends_on_bare_latin("فایل HTML را باز کنید."));
        assert!(!ends_on_bare_latin("پرداخت با تتر انجام می‌شود؛"));
        assert!(!ends_on_bare_latin("کاتالوگ به‌زودی افتتاح می‌شود؟"));
        assert!(!ends_on_bare_latin(""), "خالی بی‌طرف است");
        // نیم‌فاصلهٔ بعد از لاتین — همچنان لاتینِ تنها
        assert!(ends_on_bare_latin("escrow‌"));
    }

    /// همهٔ پاسخ‌های ثابت FAQ هرگز با توکن لاتین تنها تمام نمی‌شوند.
    #[test]
    fn instant_faq_answers_are_rtl_safe() {
        let keywords = ["خرید", "اسکرو", "پورسانت", "تتر", "هرمس", "پارلمان", "کاتالوگ", "قیمت"];
        for kw in keywords {
            if let Some(answer) = crate::faq::match_instant_faq(kw) {
                assert!(
                    !ends_on_bare_latin(answer),
                    "FAQ برای «{kw}» با واژهٔ لاتین تمام شده — RTL بشکن: …{answer}"
                );
            }
        }
    }
}
