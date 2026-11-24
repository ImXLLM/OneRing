//! فشرده‌ساز خروجی ابزار با حفظ رازها — پورت `10x/engine/token_saver.py` (پوشهٔ ۰۲).
//!
//! هدف: متن خام خروجی ابزار (دیفاگیت، لاگ بیلد، درخت پوشه‌ها…) پیش از رسیدن
//! به مدل فشرده شود تا توکن ورودی کمتری بسوزد (معمولاً ۲۰–۴۰٪ کمتر) — بدون
//! دست‌زدن به هیچ متنی که بوی راز می‌دهد (`sk-`، `PRIVATE KEY`، `password`،
//! `api_key`، `secret`، `token`، `BEGIN `، `content-type`).
//!
//! فلسفهٔ ایمنی (مانند RTK):
//! * فقط بلوک‌هایی که ۵۱۲+ نویسه‌اند و سرشان شبیه «خروجی برنامه» است؛
//! * فشرده‌سازی تقریباً بی‌اتلاف: حذف خطوط تکراری متوالی، فروپاشی خطوط خالی،
//!   کوتاه‌کردن خطوط ۳۰۰+ نویسه‌ای؛
//! * fail-safe: اگر نتیجه بزرگ‌تر یا برابر شد، متن اصلی برگردانده می‌شود.
#![forbid(unsafe_code)]

/// حداقل اندازه قبل از فشرده‌سازی — عین `_MIN_BLOCK_CHARS` پایتون.
const MIN_BLOCK_CHARS: usize = 512;
/// نشانه‌های محافظت‌شده + دو نشانهٔ مأموریتی (`sk-` و `PRIVATE KEY`).
const PROTECTIVE_MARKERS: [&str; 8] = [
    "content-type",
    "token",
    "api_key",
    "password",
    "secret",
    "begin ",
    "sk-",
    "private key",
];

/// متن «شبیه خروجی برنامه» است؟ بررسی روی سرِ ~۱KB — عین `_looks_known`.
fn looks_known(text: &str) -> bool {
    let head: String = text.chars().take(MIN_BLOCK_CHARS).collect();
    [
        // git diff
        head.contains("diff --git") || head.contains("index ---"),
        // git log
        head.contains("commit ") && (head.contains("Author:") || head.contains("Date:")),
        // git status
        head.contains("On branch ") || head.contains("nothing to commit"),
        // خروجی تست
        ["passed", "failed", " ok ", "error"]
            .iter()
            .any(|w| head.to_lowercase().contains(w))
            && head.chars().next().is_some_and(|c| c.is_ascii_digit()),
        // tracebacks
        head.contains("Traceback (most recent call last)") || head.contains("Exception in"),
        // لاگ بیلد
        ["Compiling", "Building", "Downloading", "Installing", "Collecting"]
            .iter()
            .any(|w| head.contains(w)),
        // فهرست فایل `ls -l`-مانند (دسترسی/مالک/سایز)
        head.lines().next().is_some_and(|l| {
            let cols: Vec<&str> = l.split_whitespace().collect();
            cols.len() >= 8 && cols[0].chars().all(|c| "-lrwxdc".contains(c) || c.is_ascii_digit())
        }),
    ]
    .iter()
    .any(|hit| *hit)
}

/// حذف خطوط تکراری متوالی (فقط ۲ تکرار مجاز) و فروپاشی خطوط خالی به یک.
fn compress_block(text: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let mut prev: Option<&str> = None;
    let mut empty_run = 0usize;
    let mut dup_run = 0usize;
    for line in text.lines() {
        let stripped = line.trim_end();
        if stripped.trim().is_empty() {
            empty_run += 1;
            if empty_run <= 1 {
                out.push(stripped);
            }
            continue;
        }
        empty_run = 0;
        match prev {
            Some(p) if p == stripped => {
                dup_run += 1;
                if dup_run > 2 {
                    continue;
                }
            }
            _ => dup_run = 0,
        }
        prev = Some(stripped);
        out.push(stripped);
    }
    out.join("\n")
}

/// کوتاه‌کردن خطوط مرضی (>۳۰۰ نویسه) با نشانگر ادامه.
fn trim_long_lines(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.chars().count() > 300 {
                let cut: String = line.chars().take(300).collect();
                format!("{cut} …")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// فروپاشی هر دهانهٔ ۳+ فاصله/تب به دو فاصله.
fn compress_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut ws_run = 0usize;
    for ch in text.chars() {
        if ch == ' ' || ch == '\t' {
            ws_run += 1;
            // دو نویسهٔ نخست دهانه حفظ می‌شوند؛ بقیهٔ دهانهٔ ۳+ حذف —
            // نتیجه: دقیقاً «دو فاصله» به‌جای کل دهانه (عین `WS_RUN.sub`)
            if ws_run < 3 {
                out.push(ch);
            }
        } else {
            ws_run = 0;
            out.push(ch);
        }
    }
    out
}

/// فشرده‌سازی خروجی ابزار — با حفظ مطلق رازها.
///
/// * متنی که حاوی نشانهٔ راز است **هرگز** کوتاه/تغییر نمی‌کند؛
/// * متن کوتاه (<۵۱۲) یا غیرِ «خروجی برنامه» دست‌نخورده برمی‌گردد؛
/// * اگر فشرده‌سازی بزرگ‌تر یا برابر شد، متن اصلی برمی‌گردد.
#[must_use]
pub fn compress_tool_output_preserving_secrets(text: &str, max_chars: usize) -> String {
    if text.chars().count() < MIN_BLOCK_CHARS {
        return text.to_owned();
    }
    let lowered = text.to_lowercase();
    if PROTECTIVE_MARKERS.iter().any(|m| lowered.contains(m)) {
        return text.to_owned(); // راز — هرگز دست نمی‌خورد، حتی کوتاه نمی‌شود
    }
    if !looks_known(text) {
        return text.to_owned();
    }
    let candidate = compress_whitespace(&trim_long_lines(&compress_block(text)));
    let capped: String = candidate.chars().take(max_chars.max(1)).collect();
    if capped.chars().count() >= text.chars().count() {
        return text.to_owned();
    }
    capped
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    /// خروجی بیلد بزرگ را فشرده می‌کند و هیچ رازی در آن نیست.
    #[test]
    fn compresses_noisy_build_output() {
        let mut text = String::from("Compiling kian-server v2.0.0\nBuilding kernel\n");
        for i in 0..80 {
            let _ = write!(text, "Compiling crate module_{i} v0.1.{i}\nCompiling crate module_{i} v0.1.{i}\nCompiling crate module_{i} v0.1.{i}\n");
            text.push_str("\n\n\n\n");
        }
        let out = compress_tool_output_preserving_secrets(&text, 12_000);
        assert!(out.chars().count() < text.chars().count(), "باید کوچک‌تر شود");
        assert!(out.contains("Compiling kian-server"), "محتوای واقعی حذف نمی‌شود");
    }

    /// قانون طلایی — متنِ حاوی راز بایت‌به‌بایت دست‌نخورده برمی‌گردد.
    #[test]
    fn secrets_are_never_truncated_or_mutated() {
        let noisy_secret = format!(
            "Compiling kian v2\n{}\nCompiling done zzzzz\n",
            "sk-or-v1-PLACEHOLDER-DUMMY-TEST-FIXTURE-0000-not-a-real-key  ".repeat(20)
        );
        assert!(noisy_secret.chars().count() >= MIN_BLOCK_CHARS);
        assert_eq!(
            compress_tool_output_preserving_secrets(&noisy_secret, 50),
            noisy_secret,
            "راز sk- — هرگز کوتاه نمی‌شود"
        );
        let key_block = format!("Compiling kian v2\n-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\nCompiling done\n", "a9f3\n".repeat(300));
        assert_eq!(
            compress_tool_output_preserving_secrets(&key_block, 50),
            key_block,
            "بلوک کلید خصوصی — دست‌نخورده"
        );
        for marker in ["api_key=abc123", "password: hunter2", "Authorization: Bearer token", "content-type: application/json"] {
            let big = format!("{marker}\n{}\n", "Compiling kian v2 line\n".repeat(60));
            assert_eq!(
                compress_tool_output_preserving_secrets(&big, 50),
                big,
                "نشانهٔ {marker} باید متن را محافظت کند"
            );
        }
    }

    /// متن کوتاه و متن غیرِ برنامه‌ای دست‌نخورده می‌ماند.
    #[test]
    fn small_and_non_tool_text_pass_through() {
        assert_eq!(compress_tool_output_preserving_secrets("سلام", 100), "سلام");
        let prose = "این یک متن فارسی طولانی است و به هیچ الگوی خروجی برنامه شبیه نیست. ".repeat(20);
        assert_eq!(
            compress_tool_output_preserving_secrets(&prose, 100),
            prose,
            "نثر عادی فشرده نمی‌شود"
        );
    }

    /// خطوط ۳۰۰+ نویسه‌ای کوتاه می‌شوند؛ fail-safe هرگز بزرگ‌تر نمی‌شود.
    #[test]
    fn trims_long_lines_and_never_grows() {
        let long = "x".repeat(400);
        let mut text = String::from("Traceback (most recent call last):\n");
        for _ in 0..40 {
            text.push_str(&long);
            text.push('\n');
        }
        let out = compress_tool_output_preserving_secrets(&text, 12_000);
        assert!(out.chars().count() < text.chars().count());
        for line in out.lines() {
            assert!(line.chars().count() <= 302, "خط کوتاه‌شده ≤ ۳۰۲");
        }
        // max_chars رعایت می‌شود
        let out2 = compress_tool_output_preserving_secrets(&text, 10);
        assert!(out2.chars().count() <= 10);
    }

    /// خروجی تست و git status هم الگوهای شناخته‌شده‌اند.
    #[test]
    fn recognizes_git_and_test_outputs() {
        let mut git = String::from("On branch main\nYour branch is up to date.\n");
        for i in 0..60 {
            let _ = write!(git, "modified:   src/file_{i}.rs\nmodified:   src/file_{i}.rs\n");
            git.push('\n');
        }
        let out = compress_tool_output_preserving_secrets(&git, 12_000);
        assert!(out.chars().count() < git.chars().count());

        let mut tests = String::new();
        for i in 0..30 {
            let _ = write!(tests, "3{i} passed; 0 failed; 0 ignored\n3{i} passed; 0 failed; 0 ignored\n\n");
        }
        let out2 = compress_tool_output_preserving_secrets(&tests, 12_000);
        assert!(out2.chars().count() < tests.chars().count());
    }
}
