//! Secret-preserving tool output compressor — port of `10x/engine/token_saver.py` (folder 02).
//!
//! Goal: compress raw tool output text (diffs, build logs, folder trees...) before it
//! reaches the model so it burns fewer input tokens (typically 20–40% less) — without
//! touching any text that smells like a secret (`sk-`, `PRIVATE KEY`, `password`,
//! `api_key`, `secret`, `token`, `BEGIN `, `content-type`).
//!
//! Safety philosophy (like RTK):
//! * only blocks of 512+ characters whose head looks like "program output";
//! * near-lossless compression: remove consecutive duplicate lines, collapse blank lines,
//!   truncate 300+ character lines;
//! * fail-safe: if the result is larger or equal, the original text is returned.
#![forbid(unsafe_code)]

/// Minimum size before compression — exactly the Python `_MIN_BLOCK_CHARS`.
const MIN_BLOCK_CHARS: usize = 512;
/// Protected markers + two mission-critical ones (`sk-` and `PRIVATE KEY`).
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

/// Does the text look like "program output"? Checked on the ~1KB head — exactly `_looks_known`.
fn looks_known(text: &str) -> bool {
    let head: String = text.chars().take(MIN_BLOCK_CHARS).collect();
    [
        // git diff
        head.contains("diff --git") || head.contains("index ---"),
        // git log
        head.contains("commit ") && (head.contains("Author:") || head.contains("Date:")),
        // git status
        head.contains("On branch ") || head.contains("nothing to commit"),
        // test output
        ["passed", "failed", " ok ", "error"]
            .iter()
            .any(|w| head.to_lowercase().contains(w))
            && head.chars().next().is_some_and(|c| c.is_ascii_digit()),
        // tracebacks
        head.contains("Traceback (most recent call last)") || head.contains("Exception in"),
        // build log
        ["Compiling", "Building", "Downloading", "Installing", "Collecting"]
            .iter()
            .any(|w| head.contains(w)),
        // `ls -l`-style file listing (permissions/owner/size)
        head.lines().next().is_some_and(|l| {
            let cols: Vec<&str> = l.split_whitespace().collect();
            cols.len() >= 8 && cols[0].chars().all(|c| "-lrwxdc".contains(c) || c.is_ascii_digit())
        }),
    ]
    .iter()
    .any(|hit| *hit)
}

/// Remove consecutive duplicate lines (only 2 repeats allowed) and collapse blank lines to one.
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

/// Truncate pathological lines (>300 characters) with a continuation marker.
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

/// Collapse any run of 3+ spaces/tabs down to two spaces.
fn compress_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut ws_run = 0usize;
    for ch in text.chars() {
        if ch == ' ' || ch == '\t' {
            ws_run += 1;
            // the first two characters of the run are kept; the rest of a 3+ run is removed —
            // result: exactly "two spaces" in place of the whole run (exactly `WS_RUN.sub`)
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

/// Compress tool output — with absolute secret preservation.
///
/// * text containing a secret marker is **never** truncated/modified;
/// * short (<512) or non-"program output" text is returned untouched;
/// * if compression is larger or equal, the original text is returned.
#[must_use]
pub fn compress_tool_output_preserving_secrets(text: &str, max_chars: usize) -> String {
    if text.chars().count() < MIN_BLOCK_CHARS {
        return text.to_owned();
    }
    let lowered = text.to_lowercase();
    if PROTECTIVE_MARKERS.iter().any(|m| lowered.contains(m)) {
        return text.to_owned(); // secret — never touched, not even truncated
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

    /// Compresses a large build output that contains no secrets.
    #[test]
    fn compresses_noisy_build_output() {
        let mut text = String::from("Compiling kian-server v2.0.0\nBuilding kernel\n");
        for i in 0..80 {
            let _ = write!(text, "Compiling crate module_{i} v0.1.{i}\nCompiling crate module_{i} v0.1.{i}\nCompiling crate module_{i} v0.1.{i}\n");
            text.push_str("\n\n\n\n");
        }
        let out = compress_tool_output_preserving_secrets(&text, 12_000);
        assert!(out.chars().count() < text.chars().count(), "\u{628}\u{627}\u{6cc}\u{62f} \u{6a9}\u{648}\u{686}\u{6a9}‌\u{62a}\u{631} \u{634}\u{648}\u{62f}");
        assert!(out.contains("Compiling kian-server"), "\u{645}\u{62d}\u{62a}\u{648}\u{627}\u{6cc} \u{648}\u{627}\u{642}\u{639}\u{6cc} \u{62d}\u{630}\u{641} \u{646}\u{645}\u{6cc}‌\u{634}\u{648}\u{62f}");
    }

    /// Golden rule — secret-bearing text comes back byte-for-byte untouched.
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
            "\u{631}\u{627}\u{632} sk- — \u{647}\u{631}\u{6af}\u{632} \u{6a9}\u{648}\u{62a}\u{627}\u{647} \u{646}\u{645}\u{6cc}‌\u{634}\u{648}\u{62f}"
        );
        let key_block = format!("Compiling kian v2\n-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\nCompiling done\n", "a9f3\n".repeat(300));
        assert_eq!(
            compress_tool_output_preserving_secrets(&key_block, 50),
            key_block,
            "\u{628}\u{644}\u{648}\u{6a9} \u{6a9}\u{644}\u{6cc}\u{62f} \u{62e}\u{635}\u{648}\u{635}\u{6cc} — \u{62f}\u{633}\u{62a}‌\u{646}\u{62e}\u{648}\u{631}\u{62f}\u{647}"
        );
        for marker in ["api_key=abc123", "password: hunter2", "Authorization: Bearer token", "content-type: application/json"] {
            let big = format!("{marker}\n{}\n", "Compiling kian v2 line\n".repeat(60));
            assert_eq!(
                compress_tool_output_preserving_secrets(&big, 50),
                big,
                "\u{646}\u{634}\u{627}\u{646}\u{647}\u{654} {marker} \u{628}\u{627}\u{6cc}\u{62f} \u{645}\u{62a}\u{646} \u{631}\u{627} \u{645}\u{62d}\u{627}\u{641}\u{638}\u{62a} \u{6a9}\u{646}\u{62f}"
            );
        }
    }

    /// Short text and non-program text stay untouched.
    #[test]
    fn small_and_non_tool_text_pass_through() {
        assert_eq!(compress_tool_output_preserving_secrets("\u{633}\u{644}\u{627}\u{645}", 100), "\u{633}\u{644}\u{627}\u{645}");
        let prose = "\u{627}\u{6cc}\u{646} \u{6cc}\u{6a9} \u{645}\u{62a}\u{646} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{637}\u{648}\u{644}\u{627}\u{646}\u{6cc} \u{627}\u{633}\u{62a} \u{648} \u{628}\u{647} \u{647}\u{6cc}\u{686} \u{627}\u{644}\u{6af}\u{648}\u{6cc} \u{62e}\u{631}\u{648}\u{62c}\u{6cc} \u{628}\u{631}\u{646}\u{627}\u{645}\u{647} \u{634}\u{628}\u{6cc}\u{647} \u{646}\u{6cc}\u{633}\u{62a}. ".repeat(20);
        assert_eq!(
            compress_tool_output_preserving_secrets(&prose, 100),
            prose,
            "\u{646}\u{62b}\u{631} \u{639}\u{627}\u{62f}\u{6cc} \u{641}\u{634}\u{631}\u{62f}\u{647} \u{646}\u{645}\u{6cc}‌\u{634}\u{648}\u{62f}"
        );
    }

    /// 300+ character lines get truncated; fail-safe never returns anything larger.
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
            assert!(line.chars().count() <= 302, "\u{62e}\u{637} \u{6a9}\u{648}\u{62a}\u{627}\u{647}‌\u{634}\u{62f}\u{647} ≤ \u{6f3}\u{6f0}\u{6f2}");
        }
        // max_chars is respected
        let out2 = compress_tool_output_preserving_secrets(&text, 10);
        assert!(out2.chars().count() <= 10);
    }

    /// Test output and git status are also recognized patterns.
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
