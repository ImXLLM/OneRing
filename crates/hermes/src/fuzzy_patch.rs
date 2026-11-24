//! `fuzzy_patch` — ویرایشگر جراحی ۵گذر کیان (Wave 2 — Step 4/6).
//!
//! تلفیق دو موتور مرجع:
//! - `anomalyco/opencode` (`packages/util/src/patch.ts` + `edit.rs`): نرمال‌سازی
//!   تایپوگرافی، حفظ CRLF/LF و رأی‌گیری خط نزدیک برای hint.
//! - `openai/codex` (`apply-patch/src/seek_sequence.rs` + `execpolicy`):
//!   تطبیق multi-pass خط‌محور و سیاست اجرای دستوری (Allow/Prompt/Forbidden).
//!
//! ترتیب گذرها (قرارداد سخت):
//! ۱. تطبیق دقیق بایت‌به‌بایت؛ ۲. trim انتهای خط؛ ۳. trim دو طرف؛
//! ۴. نرمال‌سازی یونیکد (گیومهٔ فرفری/دَش/فاصلهٔ non-breaking)؛
//! ۵. انکرِ بلوکی (≥۳ خط: خط اول/آخر + شباهت میانی) و نرمال‌سازی تورفتگی.
//! خروجی همیشه نشانه‌گذاری سطر اصلی فایل را حفظ می‌کند (CRLF/LF).
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

/// نتیجهٔ سیاست اجرا برای یک دستور.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecDecision {
    /// اجرای مستقیم مجاز — فقط تشخیص‌های read-only امن.
    Allow,
    /// نیازمند تأیید انسانی (HITL) — پیش‌فرض امن برای هر چیز نامطابق.
    Prompt,
    /// مطلقاً ممنوع — مخرب یا برگشت‌ناپذیر.
    Forbidden,
}

/// خطای پچ جراحی.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchError {
    /// `old_str` و `new_str` یکسان‌اند — هیچ تغییر لازم نیست.
    IdenticalInputs,
    /// `old_str` خالی است — پچ جراحی جست‌وجوی خالی را نمی‌پذیرد.
    EmptyOldString,
    /// هیچ گذری تطبیق نداد؛ شمارهٔ خط نزدیک‌ترین ناحیه و hint گنجانده شده.
    NotFound { closest_line: usize, hint: String },
    /// چند تطبیق بدون تعیین تکلیف — پچ مبهم اجرا نمی‌شود.
    MultipleMatches { count: usize },
}

/// گذر تطبیقی که پچ را یافت — برای گزارش و تست.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchPass {
    Exact,
    LineEndTrimmed,
    BothSidesTrimmed,
    UnicodeNormalized,
    IndentationNormalized,
    BlockAnchor,
}

/// نرمال‌سازی تایپوگرافی یونیکد (پورت `edit.ts`/`seek_sequence.rs`):
/// گیومهٔ فرفری → ASCII، دَش‌ها → `-`، فاصله‌های عرض‌دار → فاصلهٔ ساده.
#[must_use]
pub fn normalize_typographic(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => out.push('\''),
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => out.push('"'),
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}'
            | '\u{2212}' => out.push('-'),
            '\u{00A0}' | '\u{2002}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => {
                out.push(' ');
            }
            _ => out.push(ch),
        }
    }
    out
}

fn dominant_line_ending(text: &str) -> &'static str {
    let crlf = text.matches("\r\n").count();
    let lf = text.matches('\n').count().saturating_sub(crlf);
    if crlf > lf {
        "\r\n"
    } else {
        "\n"
    }
}

fn to_lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn restore_line_endings(text_lf: &str, ending: &str) -> String {
    if ending == "\n" {
        text_lf.to_string()
    } else {
        text_lf.replace('\n', ending)
    }
}

fn find_exact_spans(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut pos = 0;
    while let Some(idx) = haystack[pos..].find(needle) {
        let start = pos + idx;
        let end = start + needle.len();
        spans.push((start, end));
        pos = end;
    }
    spans
}

fn split_lines_with_offsets(text: &str) -> Vec<(usize, usize, &str)> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (idx, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            lines.push((start, idx, &text[start..idx]));
            start = idx + 1;
        }
    }
    if start < text.len() {
        lines.push((start, text.len(), &text[start..]));
    }
    lines
}

/// تطبیق خط‌محور با گزارهٔ تساوی سفارشی — قلب گذرهای ۲ و ۳ و ۴.
fn find_line_spans<F>(haystack: &str, needle: &str, eq: F) -> Vec<(usize, usize)>
where
    F: Fn(&str, &str) -> bool,
{
    let src_lines = split_lines_with_offsets(haystack);
    let pat_lines: Vec<&str> = needle.trim_end_matches('\n').split('\n').collect();
    if pat_lines.is_empty() || src_lines.len() < pat_lines.len() {
        return Vec::new();
    }
    let mut spans = Vec::new();
    let mut i = 0;
    while i + pat_lines.len() <= src_lines.len() {
        if (0..pat_lines.len()).all(|k| eq(src_lines[i + k].2, pat_lines[k])) {
            spans.push((src_lines[i].0, src_lines[i + pat_lines.len() - 1].1));
            i += pat_lines.len();
        } else {
            i += 1;
        }
    }
    spans
}

fn strip_common_indent(lines: &[&str]) -> Vec<String> {
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line.chars().skip(min_indent).collect::<String>()
            }
        })
        .collect()
}

/// گذر ۵ الف — نرمال‌سازی تورفتگی: پنجرهٔ هم‌اندازه با الگو پس از حذف
/// بیشینهٔ تورفتگی مشترک باید خط‌به‌خط (با نرمال‌سازی تایپوگرافی) برابر شود.
fn find_indent_normalized_spans(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    let src_lines = split_lines_with_offsets(haystack);
    let pat_raw: Vec<&str> = needle.trim_end_matches('\n').split('\n').collect();
    if pat_raw.is_empty() || src_lines.len() < pat_raw.len() {
        return Vec::new();
    }
    let pat_norm = strip_common_indent(&pat_raw);
    let mut spans = Vec::new();
    let mut i = 0;
    while i + pat_raw.len() <= src_lines.len() {
        let window: Vec<&str> = src_lines[i..i + pat_raw.len()].iter().map(|l| l.2).collect();
        let win_norm = strip_common_indent(&window);
        if win_norm
            .iter()
            .zip(pat_norm.iter())
            .all(|(src, pat)| normalize_typographic(src.trim_end()) == normalize_typographic(pat.trim_end()))
        {
            spans.push((src_lines[i].0, src_lines[i + pat_raw.len() - 1].1));
            i += pat_raw.len();
        } else {
            i += 1;
        }
    }
    spans
}

/// شباهت دو خط پس از نرمال‌سازی — نسبت طول LCS نویسه‌ای به طول بلندتر.
#[allow(clippy::cast_precision_loss)] // شباهت فازی خطوط — دقت f32 کاملاً کافی است
fn line_similarity(left: &str, right: &str) -> f32 {
    let left: Vec<char> = normalize_typographic(left.trim()).chars().collect();
    let right: Vec<char> = normalize_typographic(right.trim()).chars().collect();
    if left.is_empty() && right.is_empty() {
        return 1.0;
    }
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    // LCS کلاسیک روی نویسه‌ها؛ برای خط‌های کوتاهِ بلوک‌های واقعی کافی است.
    let mut dp = vec![0u32; right.len() + 1];
    for lch in &left {
        let mut prev_diag = 0u32;
        for (j, rch) in right.iter().enumerate() {
            let temp = dp[j + 1];
            dp[j + 1] = if lch == rch {
                prev_diag + 1
            } else {
                dp[j].max(dp[j + 1])
            };
            prev_diag = temp;
        }
    }
    let lcs = dp[right.len()] as f32;
    lcs / left.len().max(right.len()) as f32
}

/// گذر ۵ ب — انکرِ بلوکی: برای الگوهای ≥۳ خط، خط اول و آخر (نرمال‌شدهٔ trim)
/// باید عیناً برابر باشند و شباهت خطوط میانی ≥ ۰٫۵ شود.
fn find_block_anchor_spans(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    let src_lines = split_lines_with_offsets(haystack);
    let pat: Vec<&str> = needle.trim_end_matches('\n').split('\n').collect();
    if pat.len() < 3 || src_lines.len() < pat.len() {
        return Vec::new();
    }
    let first = normalize_typographic(pat[0].trim());
    let last = normalize_typographic(pat[pat.len() - 1].trim());
    let middle: &[&str] = &pat[1..pat.len() - 1];
    let mut spans = Vec::new();
    let mut i = 0;
    while i + pat.len() <= src_lines.len() {
        let src_first = normalize_typographic(src_lines[i].2.trim());
        let src_last = normalize_typographic(src_lines[i + pat.len() - 1].2.trim());
        if src_first == first && src_last == last {
            let window: Vec<&str> = src_lines[i + 1..i + pat.len() - 1].iter().map(|l| l.2).collect();
            let score = window
                .iter()
                .zip(middle.iter())
                .map(|(src, pat)| if line_similarity(src, pat) >= 0.5 { 1.0 } else { 0.0 })
                .sum::<f32>()
                / middle.len() as f32;
            if score >= 0.5 {
                spans.push((src_lines[i].0, src_lines[i + pat.len() - 1].1));
                i += pat.len();
                continue;
            }
        }
        i += 1;
    }
    spans
}

/// رأی‌گیری خطوط نزدیک (پورت `edit.ts` `missHint`) — نزدیک‌ترین شمارهٔ خط
/// (۱-مبنا) به محلِ بیشترین هم‌رأیی با الگو را برمی‌گرداند؛ بدون هم‌رأیی → `None`.
#[must_use]
pub fn miss_hint(source: &str, old_str: &str) -> Option<(usize, String)> {
    let view = to_lf(source);
    let old_lf = to_lf(old_str);
    let lines: Vec<&str> = view.split('\n').collect();
    let search: Vec<&str> = old_lf.trim_end_matches('\n').split('\n').collect();
    let key = |line: &str| -> String {
        normalize_typographic(line)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut best_start = 0usize;
    let mut best_votes = 0usize;
    for start in 0..lines.len() {
        let mut votes = 0usize;
        for (offset, search_line) in search.iter().enumerate() {
            let needle_key = key(search_line);
            if needle_key.is_empty() {
                continue;
            }
            if let Some(target) = lines.get(start + offset) {
                if key(target) == needle_key {
                    votes += 1;
                }
            }
        }
        if votes > best_votes {
            best_votes = votes;
            best_start = start;
        }
    }
    if best_votes == 0 {
        return None;
    }
    let end = (best_start + search.len().min(12)).min(lines.len());
    let snippet = lines[best_start..end]
        .iter()
        .enumerate()
        .map(|(idx, line)| format!("{}: {line}", best_start + idx + 1))
        .collect::<Vec<_>>()
        .join("\n");
    Some((best_start + 1, format!("The closest lines in the file:\n{snippet}")))
}

/// اعمال پچ جراحی با آبشار ۵گذر و حفظ نشانه‌گذاری سطر فایل.
///
/// ترتیب گذرها: Exact → LineEndTrimmed → BothSidesTrimmed →
/// UnicodeNormalized → IndentationNormalized → BlockAnchor(≥۳ خط).
/// اگر الگو در چند ناحیه تطبیق کند، پچ اجرا نمی‌شود (کد مبهم ممنوع).
///
/// # Errors
/// `IdenticalInputs`/`EmptyOldString` برای ورودی نامشروع؛ `NotFound` با
/// شمارهٔ خط نزدیک و hint (شامل «از قبل اعمال‌شده» اگر `new_str` موجود باشد)؛
/// `MultipleMatches` برای تطبیق مبهم.
pub fn apply_surgical_patch(source: &str, old_str: &str, new_str: &str) -> Result<String, PatchError> {
    if old_str == new_str {
        return Err(PatchError::IdenticalInputs);
    }
    if old_str.is_empty() {
        return Err(PatchError::EmptyOldString);
    }

    let ending = dominant_line_ending(source);
    let view = to_lf(source);
    let old_lf = to_lf(old_str);
    let new_lf = to_lf(new_str);

    // گذر ۱ — دقیق بایت‌به‌بایت
    let mut spans = find_exact_spans(&view, &old_lf);

    // گذر ۲ — trim انتهای هر خط
    if spans.is_empty() {
        spans = find_line_spans(&view, &old_lf, |src, pat| src.trim_end() == pat.trim_end());
    }

    // گذر ۳ — trim دو طرف
    if spans.is_empty() {
        spans = find_line_spans(&view, &old_lf, |src, pat| src.trim() == pat.trim());
    }

    // گذر ۴ — نرمال‌سازی تایپوگرافی یونیکد (ترکیب با trim انتهای خط)
    if spans.is_empty() {
        spans = find_line_spans(&view, &old_lf, |src, pat| {
            normalize_typographic(src.trim_end()) == normalize_typographic(pat.trim_end())
        });
    }

    // گذر ۵ الف — نرمال‌سازی تورفتگی
    if spans.is_empty() {
        spans = find_indent_normalized_spans(&view, &old_lf);
    }

    // گذر ۵ ب — انکرِ بلوکی (≥۳ خط)
    if spans.is_empty() {
        spans = find_block_anchor_spans(&view, &old_lf);
    }

    if spans.is_empty() {
        let (closest_line, hint) = match miss_hint(source, old_str) {
            Some((line, hint)) => (line, hint),
            None => {
                let norm_view = normalize_typographic(&view);
                let norm_new = normalize_typographic(new_lf.trim());
                if norm_new.chars().count() >= 20 && norm_view.contains(&norm_new) {
                    (0, "newString is already present in the file, so this edit may already be applied.".to_string())
                } else {
                    (0, "no overlapping lines found".to_string())
                }
            }
        };
        return Err(PatchError::NotFound { closest_line, hint });
    }
    if spans.len() > 1 {
        return Err(PatchError::MultipleMatches { count: spans.len() });
    }

    let (start, end) = spans[0];
    let mut out = view;
    let replacement = new_lf.trim_end_matches('\n');
    out.replace_range(start..end, replacement);
    Ok(restore_line_endings(&out, ending))
}

/// قواعد پیشوندی سیاست اجرا (الهام از `codex-rs/execpolicy`) — بلندترین
/// پیشوندِ منطبق برنده است؛ دستور نامطابق = `Prompt`.
pub const KIAN_DEFAULT_EXEC_POLICY: &[(&[&str], ExecDecision, &str)] = &[
    (
        &["git", "reset", "--hard"],
        ExecDecision::Forbidden,
        "Destructive working-tree reset is forbidden in the KIAN builder sandbox.",
    ),
    (&["rm", "-rf", "/"], ExecDecision::Forbidden, "Root deletion is forbidden."),
    (
        &["git", "push", "--force"],
        ExecDecision::Prompt,
        "Force push rewrites remote history; requires HITL approval.",
    ),
    (
        &["cargo", "test"],
        ExecDecision::Allow,
        "Read-only verification build with artifacts in the isolated target dir.",
    ),
    (&["cargo", "clippy"], ExecDecision::Allow, "Lint-only diagnostics."),
    (&["cargo", "check"], ExecDecision::Allow, "Type-check only."),
    (&["ls"], ExecDecision::Allow, "Read-only directory listing."),
    (&["pwd"], ExecDecision::Allow, "Read-only working directory check."),
    (
        &["python3", "-c"],
        ExecDecision::Prompt,
        "Inline python can mutate state; require approval.",
    ),
];

/// ارزیابی سیاست اجرا برای یک دستور (`argv`) با قواعد پیش‌فرض کیان.
#[must_use]
pub fn evaluate_exec_policy(argv: &[&str]) -> ExecDecision {
    let (decision, _why) = evaluate_exec_policy_with_rules(argv, KIAN_DEFAULT_EXEC_POLICY);
    decision
}

/// همان ارزیابی با مجموعهٔ قواعد دلخواه + دلیل برنده (برای تست و گزارش).
#[must_use]
pub fn evaluate_exec_policy_with_rules<'r>(
    argv: &[&str],
    rules: &'r [(&'r [&'r str], ExecDecision, &'r str)],
) -> (ExecDecision, &'r str) {
    let mut best: Option<&'r (&'r [&'r str], ExecDecision, &'r str)> = None;
    for rule in rules {
        let (pattern, _, _) = *rule;
        if argv.len() >= pattern.len() && argv[..pattern.len()] == *pattern {
            let replace = match best {
                None => true,
                Some(prev) => pattern.len() > prev.0.len(),
            };
            if replace {
                best = Some(rule);
            }
        }
    }
    match best {
        Some((_, decision, why)) => (*decision, why),
        None => (
            ExecDecision::Prompt,
            "Unmatched command requires policy check or HITL prompt.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass1_exact_match_preserves_crlf() {
        let src = "<div>\r\n  <h1>Hello</h1>\r\n</div>\r\n";
        let out = apply_surgical_patch(src, "  <h1>Hello</h1>", "  <h1>KIAN</h1>").unwrap();
        assert!(out.contains("\r\n  <h1>KIAN</h1>\r\n"), "CRLF must survive: {out:?}");
    }

    #[test]
    fn pass2_line_end_trim_rescues_trailing_whitespace() {
        let src = "fn a() {\n    return 1;   \n}\n";
        let out = apply_surgical_patch(src, "    return 1;\n}", "    return 2;\n}").unwrap();
        assert!(out.contains("    return 2;"), "pass 2 must hit: {out}");
    }

    #[test]
    fn pass3_both_sides_trim_rescues_indent_drift() {
        let src = "if ok {\n        launch();\n    } else {\n        abort();\n    }\n";
        let out = apply_surgical_patch(src, "    abort();", "    abort_all();").unwrap();
        assert!(out.contains("abort_all();"), "pass 3 must hit: {out}");
    }

    #[test]
    fn pass4_unicode_normalized_matches_curly_quotes_and_dashes() {
        let src = "<p>It’s ready — now</p>\n";
        let out = apply_surgical_patch(src, "<p>It's ready - now</p>", "<p>Shipped</p>").unwrap();
        assert!(out.contains("<p>Shipped</p>"), "pass 4 must hit: {out}");
    }

    #[test]
    fn pass5_indentation_normalized_matches_reindented_block() {
        let src = "class A:\n        def go(self):\n            return 1\n";
        let out = apply_surgical_patch(
            src,
            "    def go(self):\n        return 1",
            "    def go(self):\n        return 42",
        )
        .unwrap();
        assert!(out.contains("return 42"), "pass 5 must hit: {out}");
    }

    #[test]
    fn pass5_block_anchor_rescues_drifted_middle_lines() {
        let src = "fn handle() {\n    let a = drifted_line_1();\n    let b = drifted_2();\n    finish(a, b)\n}\n";
        let out = apply_surgical_patch(
            src,
            "fn handle() {\n    let a = drifted_line_2();\n    let b = drifted_3();\n    finish(a, b)\n}",
            "fn handle() {\n    let a = drifted_line_1();\n    let b = drifted_2();\n    finish_all(a, b)\n}",
        )
        .unwrap();
        assert!(out.contains("drifted_line_1"), "block anchor must keep source: {out}");
        assert!(out.contains("finish_all"), "block anchor must apply replacement: {out}");
    }

    #[test]
    fn failure_returns_closest_line_hint() {
        let src = "fn main() {\n    println!(\"kian\");\n}\n";
        let err = apply_surgical_patch(src, "fn missing() {\n    println!(\"kian\");\n}", "x").unwrap_err();
        match err {
            PatchError::NotFound { closest_line, hint } => {
                assert!(closest_line >= 1, "must report a 1-based line, got {closest_line}");
                assert!(hint.contains("closest"), "hint must be human-readable: {hint}");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn already_applied_new_string_is_detected_in_hint() {
        let src = "value = compute_kian_v2();\n";
        let err =
            apply_surgical_patch(src, "value = old_impl();", "value = compute_kian_v2();").unwrap_err();
        match err {
            PatchError::NotFound { hint, .. } => {
                assert!(hint.contains("already be applied"), "hint: {hint}");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn ambiguous_multiple_matches_are_rejected() {
        let src = "x = 1;\nx = 1;\n";
        let err = apply_surgical_patch(src, "x = 1;", "x = 2;").unwrap_err();
        assert_eq!(err, PatchError::MultipleMatches { count: 2 });
    }

    #[test]
    fn identical_and_empty_inputs_are_typed_errors() {
        assert_eq!(apply_surgical_patch("a\n", "a", "a").unwrap_err(), PatchError::IdenticalInputs);
        assert_eq!(apply_surgical_patch("a\n", "", "b").unwrap_err(), PatchError::EmptyOldString);
    }

    #[test]
    fn exec_policy_blocks_destructive_allows_diagnostics() {
        assert_eq!(evaluate_exec_policy(&["rm", "-rf", "/"]), ExecDecision::Forbidden);
        assert_eq!(evaluate_exec_policy(&["git", "reset", "--hard"]), ExecDecision::Forbidden);
        assert_eq!(evaluate_exec_policy(&["git", "push", "--force", "main"]), ExecDecision::Prompt);
        assert_eq!(evaluate_exec_policy(&["ls", "-la"]), ExecDecision::Allow);
        assert_eq!(evaluate_exec_policy(&["pwd"]), ExecDecision::Allow);
        assert_eq!(evaluate_exec_policy(&["cargo", "test", "-p", "kian-ai"]), ExecDecision::Allow);
        assert_eq!(
            evaluate_exec_policy(&["cargo", "clippy", "--", "-D", "warnings"]),
            ExecDecision::Allow
        );
        assert_eq!(evaluate_exec_policy(&["python3", "-c", "print(1)"]), ExecDecision::Prompt);
        assert_eq!(evaluate_exec_policy(&["curl", "http://x"]), ExecDecision::Prompt);
    }

    #[test]
    fn longest_prefix_rule_wins() {
        let rules = [
            (&["git"] as &[&str], ExecDecision::Allow, "generic git"),
            (&["git", "push", "--force"] as &[&str], ExecDecision::Prompt, "Force push needs approval"),
        ];
        let (decision, why) = evaluate_exec_policy_with_rules(&["git", "push", "--force", "x"], &rules);
        assert_eq!(decision, ExecDecision::Prompt);
        assert!(why.contains("Force push"));
    }

    #[test]
    fn miss_hint_reports_line_and_snippet() {
        let src = "fn a() {\n    let x = 1;\n    let y = kian_v2;\n}\n";
        let (line, hint) = miss_hint(src, "let z = missing;\nlet y = kian_v2;").expect("hint");
        assert!(line >= 2 && line <= 4, "closest region line, got {line}");
        assert!(hint.contains("closest"), "hint: {hint}");
        assert!(miss_hint("alpha\nbeta\ngamma", "unrelated tokens only").is_none());
    }
}
