//! redaction — اسکن قطعی رازها روی متن ورودی/خروجی AI.
//!
//! فلسفه: هرگز راز به ارائه‌دهندهٔ بیرونی یا روی سیم پاسخ نمی‌رود. خروجی این
//! ماژول فقط «دستهٔ یافته» را گزارش می‌کند (`private_key_hex`، `bot_token`،…)
//! و متن را با جای‌نشان `[REDACTED:<kind>]` عوض می‌کند.
//!
//! الگوها (بدون regex — اسکن بایتی قطعی و replayable):
//! * `pem_key` — بلوک `-----BEGIN … -----END …`
//! * `jwt` — سه قطعهٔ base64url دور نقطه، شروع با `eyJ`
//! * `private_key_hex` — `0x` + ۶۴ کاراکتر هگز
//! * `bot_token` — قالب توکن بات تلگرام: ۸..=۱۲ رقم + `:` + ≥۳۰ کاراکتر
//! * `long_hex` — هر رشتهٔ هگز ≥۳۲ کاراکتر (توکن/نشست نشتی)
//! * `mnemonic` — ۱۲..=۲۴ واژهٔ ASCII کوچک پی‌درپی با فاصلهٔ تکی
//!
//! محدودیت اعلام‌شده: رازهای «متن‌محور» (کلید ادمین انسانی مثل
//! `kian-admin-key-...`) ساختار قابل‌تشخیص قطعی ندارند — سپر هرمس و قاعدهٔ
//! «کلید فقط در هدر» جلوی آن‌ها را می‌گیرد، نه این اسکنر.

/// یک یافته — دسته + جای بایتی در متن اصلی (برای ممیزی، بدون محتوا).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionFinding {
    /// دستهٔ راز
    pub kind: &'static str,
    /// جای شروع در متن اصلی (بایت)
    pub start: usize,
}

/// گزارش redaction — متن پاک‌شده + فهرست یافته‌ها.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionReport {
    /// متن پس از جای‌نشان‌گذاری
    pub text: String,
    /// یافته‌ها به ترتیب مکان
    pub findings: Vec<RedactionFinding>,
}

/// اولویت الگوها — پهن‌ترین/سخت‌گیرانه‌ترین اول رزرو می‌شود.
const PATTERNS: [&str; 6] =
    ["pem_key", "jwt", "private_key_hex", "bot_token", "long_hex", "mnemonic"];

/// امضای اسکنر — بازه‌های (شروع، پایان) هر الگو.
type SpanScanner = fn(&[u8]) -> Vec<(usize, usize)>;

fn is_hex(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b)
}

fn is_token_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

fn is_base64url(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

fn find_all(hay: &[u8], needle: &[u8]) -> Vec<usize> {
    let mut out = Vec::new();
    if needle.is_empty() || hay.len() < needle.len() {
        return out;
    }
    for i in 0..=(hay.len() - needle.len()) {
        if &hay[i..i + needle.len()] == needle {
            out.push(i);
        }
    }
    out
}

/// طول اجرای کاراکترهای مجاز از `start` — برای اسکن رشته‌های ساختاری.
fn run_len(hay: &[u8], start: usize, pred: impl Fn(u8) -> bool) -> usize {
    hay[start..].iter().take_while(|&&b| pred(b)).count()
}

// ── اسکنرهای هر الگو — بازگشت: (start, end_exclusive) ───────────────────────

fn scan_pem(hay: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for begin in find_all(hay, b"-----BEGIN") {
        let Some(end_rel) = hay[begin..]
            .windows(9)
            .position(|w| w == b"-----END ")
            .map(|p| begin + p)
        else {
            continue;
        };
        // تا انتهای خطِ END
        let mut end = end_rel + 9;
        while end < hay.len() && hay[end] != b'\n' {
            end += 1;
        }
        end = end.min(hay.len());
        out.push((begin, end));
    }
    out
}

fn scan_jwt(hay: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for start in find_all(hay, b"eyJ") {
        let len = run_len(hay, start, is_base64url_dot);
        let seg = &hay[start..start + len];
        #[allow(clippy::naive_bytecount)] // شمارش یک‌بارهٔ قطعی — بدون وابستگی bytecount
        let dots = seg.iter().filter(|&&b| b == b'.').count();
        // JWT = header.payload.signature — حداقل دو نقطه و طول معقول
        if dots >= 2 && len >= 32 {
            out.push((start, start + len));
        }
    }
    out
}

fn is_base64url_dot(b: u8) -> bool {
    is_base64url(b) || b == b'.'
}

fn scan_hex64(hay: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for start in find_all(hay, b"0x") {
        let after = start + 2;
        let len = run_len(hay, after, is_hex);
        if len == 64 {
            out.push((start, after + 64));
        }
    }
    out
}

fn scan_bot_token(hay: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    // رقم‌ها سپس ':' سپس بدنهٔ توکن
    for i in 0..hay.len() {
        if hay[i] != b':' {
            continue;
        }
        // شمارش رقم قبل از ':'
        let mut d = i;
        while d > 0 && hay[d - 1].is_ascii_digit() {
            d -= 1;
        }
        let digits = i - d;
        if !(8..=12).contains(&digits) {
            continue;
        }
        let body = run_len(hay, i + 1, is_token_char);
        if body >= 30 {
            out.push((d, i + 1 + body));
        }
    }
    out
}

fn scan_long_hex(hay: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < hay.len() {
        if is_hex(hay[i]) {
            let len = run_len(hay, i, is_hex);
            if len >= 32 {
                out.push((i, i + len));
            }
            i += len.max(1);
        } else {
            i += 1;
        }
    }
    out
}

fn scan_mnemonic(hay: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    // واژه = [a-z]{3,}؛ دنباله = واژه‌ها با فاصلهٔ تکی
    let mut i = 0;
    while i < hay.len() {
        // شروع واژهٔ کوچک ASCII
        let is_word_start = hay[i].is_ascii_lowercase()
            && (i == 0 || hay[i - 1] == b' ' || !hay[i - 1].is_ascii_alphanumeric());
        if !is_word_start {
            i += 1;
            continue;
        }
        // اندازه‌گیری طول دنباله از این واژه
        let mut j = i;
        let mut words = 0usize;
        let mut end = i;
        loop {
            let len = run_len(hay, j, |b| b.is_ascii_lowercase());
            if len < 3 {
                break;
            }
            words += 1;
            end = j + len;
            j = end;
            // فقط فاصلهٔ تکی واژهٔ بعدی
            if j + 1 < hay.len() && hay[j] == b' ' && hay[j + 1].is_ascii_lowercase() {
                j += 1;
            } else {
                break;
            }
        }
        if (12..=24).contains(&words) {
            out.push((i, end));
            i = end;
        } else {
            i = end.max(i + 1);
        }
    }
    out
}

/// متن را اسکن می‌کند و هر راز را با `[REDACTED:<kind>]` عوض می‌کند.
///
/// اسکن یک‌بارمصرف و قطعی است: الگوها به ترتیب اولویت رزرو می‌شوند و
/// هم‌پوشانی‌ها به نفع الگوی سخت‌گیرانه‌تر حذف می‌شوند.
#[must_use]
pub fn redact(input: &str) -> RedactionReport {
    let hay = input.as_bytes();
    let scanners: [SpanScanner; 6] =
        [scan_pem, scan_jwt, scan_hex64, scan_bot_token, scan_long_hex, scan_mnemonic];

    // رزرو با اولویت؛ هم‌پوشانی = رد
    let mut reserved: Vec<(usize, usize, &'static str)> = Vec::new();
    for (pattern_idx, scanner) in scanners.iter().enumerate() {
        'spans: for (start, end) in scanner(hay) {
            for (s, e, _) in &reserved {
                if start < *e && *s < end {
                    continue 'spans;
                }
            }
            reserved.push((start, end, PATTERNS[pattern_idx]));
        }
    }
    reserved.sort_by_key(|(s, e, _)| (*s, *e));

    // بازسازی متن با جای‌نشان — لبه‌ها به مرز کاراکتر امن می‌روند
    let mut text = String::with_capacity(input.len());
    let mut findings = Vec::new();
    let mut cursor = 0usize;
    for (start, end, kind) in reserved {
        let start = floor_boundary(input, start);
        let end = ceil_boundary(input, end);
        if start < cursor {
            continue; // مرزِ پرشده — هم‌پوشانی پس از تنظیم مرز
        }
        text.push_str(&input[cursor..start]);
        text.push_str("[REDACTED:");
        text.push_str(kind);
        text.push(']');
        findings.push(RedactionFinding { kind, start });
        cursor = end;
    }
    text.push_str(&input[cursor.min(input.len())..]);

    RedactionReport { text, findings }
}

fn floor_boundary(s: &str, mut i: usize) -> usize {
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn ceil_boundary(s: &str, mut i: usize) -> usize {
    let max = s.len();
    while i < max && !s.is_char_boundary(i) {
        i += 1;
    }
    i.min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// کلید خصوصی EVM — دقیقاً ۶۴ هگز پس از 0x.
    #[test]
    fn private_key_hex_is_redacted_without_echo() {
        let key = "0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318";
        let r = redact(format!("send {key} now").as_str());
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].kind, "private_key_hex");
        assert!(r.text.contains("[REDACTED:private_key_hex]"));
        assert!(!r.text.contains("4c0883"), "secret must not echo");
    }

    /// توکن بات تلگرام — الگوی رسمی رقم+دو+بدنهٔ بلند.
    #[test]
    fn bot_token_is_redacted() {
        let r = redact("token 123456789:AAHdqTcvbCHB1WUqptGffZfjHlXgMbGfRcE broken");
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].kind, "bot_token");
        assert!(!r.text.contains("AAHdqTcv"));
    }

    /// بلوک PEM — از BEGIN تا انتهای خط END کامل پوشیده می‌شود.
    #[test]
    fn pem_block_is_redacted() {
        let pem = "-----BEGIN RSA PRIVATE KEY-----\nMIIBOgIBAAJBAK\n-----END RSA PRIVATE KEY-----";
        let r = redact(pem);
        assert_eq!(r.findings[0].kind, "pem_key");
        assert!(!r.text.contains("MIIBOg"), "body must go: {}", r.text);
    }

    /// JWT — سه قطعه دور نقطه.
    #[test]
    fn jwt_is_redacted() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jGWNHk2ZjBzd3AQA1n9cFSTOvQncf4o";
        let r = redact(jwt);
        assert_eq!(r.findings[0].kind, "jwt");
        assert!(!r.text.contains("eyJhbGci"));
    }

    /// عبارت حافظهٔ ۱۲واژه‌ای — هشدار رایج کیف‌ها.
    #[test]
    fn twelve_word_mnemonic_is_redacted() {
        let m = "legal winner thank year wave sausage worth useful legal winner thank yellow";
        let r = redact(m);
        assert_eq!(r.findings[0].kind, "mnemonic");
        assert!(!r.text.contains("sausage"), "seed words must go: {}", r.text);
    }

    /// متن سالم فارسی — بدون یافتهٔ کاذب.
    #[test]
    fn clean_persian_text_is_untouched() {
        let r = redact("سلام، تراز کیف من چقدر است؟ محصول بازار را نشان بده.");
        assert!(r.findings.is_empty(), "no false positives: {:?}", r.findings);
        assert_eq!(r.text, "سلام، تراز کیف من چقدر است؟ محصول بازار را نشان بده.");
    }

    /// هگز بلند غیر-EVM — `long_hex`.
    #[test]
    fn long_hex_session_token_is_redacted() {
        let r = redact("session=5f1b2c3d4e5f60718293a4b5c6d7e8f90112233445566778899aabbccddeeff0");
        assert_eq!(r.findings[0].kind, "long_hex");
    }

    /// اولویت: hex64 داخل بازهٔ `long_hex` — فقط یک یافتهٔ سخت‌گیرانه‌تر.
    #[test]
    fn overlapping_spans_deduplicate() {
        let key = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let r = redact(key);
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].kind, "private_key_hex");
    }
}
