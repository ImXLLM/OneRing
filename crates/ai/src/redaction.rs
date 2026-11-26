//! redaction — deterministic secret scanning over AI input/output text.
//!
//! Philosophy: a secret never reaches an external provider or the wire. This module's
//! output only reports the finding category (`private_key_hex`, `bot_token`, …)
//! and replaces the text with the `[REDACTED:<kind>]` placeholder.
//!
//! Patterns (no regex — deterministic, replayable byte scanning):
//! * `pem_key` — a `-----BEGIN … -----END …` block
//! * `jwt` — three base64url segments around dots, starting with `eyJ`
//! * `private_key_hex` — `0x` + 64 hex characters
//! * `bot_token` — the Telegram bot token format: 8..=12 digits + `:` + ≥30 characters
//! * `long_hex` — any hex string of ≥32 characters (a leaked token/session)
//! * `mnemonic` — 12..=24 consecutive lowercase ASCII words with single spaces
//!
//! Declared limitation: text-shaped secrets (a human admin key such as
//! `kian-admin-key-...`) have no deterministically detectable structure — the Hermes shield
//! and the "key only in headers" rule stop those, not this scanner.

/// One finding — category + byte offset in the original text (for audit, no content).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionFinding {
    /// Secret category
    pub kind: &'static str,
    /// Start offset in the original text (bytes)
    pub start: usize,
}

/// The redaction report — sanitized text + the findings list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionReport {
    /// Text after placeholder substitution
    pub text: String,
    /// Findings in positional order
    pub findings: Vec<RedactionFinding>,
}

/// Pattern priority — the broadest/strictest reserves first.
const PATTERNS: [&str; 6] =
    ["pem_key", "jwt", "private_key_hex", "bot_token", "long_hex", "mnemonic"];

/// Scanner signature — the (start, end) ranges per pattern.
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

/// The run length of allowed characters from `start` — for scanning structural strings.
fn run_len(hay: &[u8], start: usize, pred: impl Fn(u8) -> bool) -> usize {
    hay[start..].iter().take_while(|&&b| pred(b)).count()
}

// ── Per-pattern scanners — returns: (start, end_exclusive) ───────────────────────

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
        // Up to the end of the END line
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
        #[allow(clippy::naive_bytecount)] // A single-pass deterministic count — no bytecount dependency
        let dots = seg.iter().filter(|&&b| b == b'.').count();
        // JWT = header.payload.signature — at least two dots and a reasonable length
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
    // Digits, then ':', then the token body
    for i in 0..hay.len() {
        if hay[i] != b':' {
            continue;
        }
        // Count the digits before ':'
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
    // Word = [a-z]{3,}; sequence = words with single spaces
    let mut i = 0;
    while i < hay.len() {
        // The start of a lowercase ASCII word
        let is_word_start = hay[i].is_ascii_lowercase()
            && (i == 0 || hay[i - 1] == b' ' || !hay[i - 1].is_ascii_alphanumeric());
        if !is_word_start {
            i += 1;
            continue;
        }
        // Measure the sequence length from this word
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
            // Only a single space to the next word
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

/// Scans the text and replaces every secret with `[REDACTED:<kind>]`.
///
/// The scan is single-pass and deterministic: patterns reserve in priority order and
/// overlaps are pruned in favor of the stricter pattern.
#[must_use]
pub fn redact(input: &str) -> RedactionReport {
    let hay = input.as_bytes();
    let scanners: [SpanScanner; 6] =
        [scan_pem, scan_jwt, scan_hex64, scan_bot_token, scan_long_hex, scan_mnemonic];

    // Reserve by priority; overlap = reject
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

    // Rebuild the text with placeholders — edges move to safe character boundaries
    let mut text = String::with_capacity(input.len());
    let mut findings = Vec::new();
    let mut cursor = 0usize;
    for (start, end, kind) in reserved {
        let start = floor_boundary(input, start);
        let end = ceil_boundary(input, end);
        if start < cursor {
            continue; // Occupied boundary — overlap after boundary adjustment
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

    /// EVM private key — exactly 64 hex after 0x.
    #[test]
    fn private_key_hex_is_redacted_without_echo() {
        let key = "0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318";
        let r = redact(format!("send {key} now").as_str());
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].kind, "private_key_hex");
        assert!(r.text.contains("[REDACTED:private_key_hex]"));
        assert!(!r.text.contains("4c0883"), "secret must not echo");
    }

    /// Telegram bot token — the official digits+colon+long-body pattern.
    #[test]
    fn bot_token_is_redacted() {
        let r = redact("token 123456789:AAHdqTcvbCHB1WUqptGffZfjHlXgMbGfRcE broken");
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].kind, "bot_token");
        assert!(!r.text.contains("AAHdqTcv"));
    }

    /// PEM block — fully covered from BEGIN to the end of the END line.
    #[test]
    fn pem_block_is_redacted() {
        let pem = "-----BEGIN RSA PRIVATE KEY-----\nMIIBOgIBAAJBAK\n-----END RSA PRIVATE KEY-----";
        let r = redact(pem);
        assert_eq!(r.findings[0].kind, "pem_key");
        assert!(!r.text.contains("MIIBOg"), "body must go: {}", r.text);
    }

    /// JWT — three segments around dots.
    #[test]
    fn jwt_is_redacted() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jGWNHk2ZjBzd3AQA1n9cFSTOvQncf4o";
        let r = redact(jwt);
        assert_eq!(r.findings[0].kind, "jwt");
        assert!(!r.text.contains("eyJhbGci"));
    }

    /// A 12-word seed phrase — the common wallet warning.
    #[test]
    fn twelve_word_mnemonic_is_redacted() {
        let m = "legal winner thank year wave sausage worth useful legal winner thank yellow";
        let r = redact(m);
        assert_eq!(r.findings[0].kind, "mnemonic");
        assert!(!r.text.contains("sausage"), "seed words must go: {}", r.text);
    }

    /// Healthy Persian text — no false positives.
    #[test]
    fn clean_persian_text_is_untouched() {
        let r = redact("\u{633}\u{644}\u{627}\u{645}\u{60c} \u{62a}\u{631}\u{627}\u{632} \u{6a9}\u{6cc}\u{641} \u{645}\u{646} \u{686}\u{642}\u{62f}\u{631} \u{627}\u{633}\u{62a}\u{61f} \u{645}\u{62d}\u{635}\u{648}\u{644} \u{628}\u{627}\u{632}\u{627}\u{631} \u{631}\u{627} \u{646}\u{634}\u{627}\u{646} \u{628}\u{62f}\u{647}.");
        assert!(r.findings.is_empty(), "no false positives: {:?}", r.findings);
        assert_eq!(r.text, "\u{633}\u{644}\u{627}\u{645}\u{60c} \u{62a}\u{631}\u{627}\u{632} \u{6a9}\u{6cc}\u{641} \u{645}\u{646} \u{686}\u{642}\u{62f}\u{631} \u{627}\u{633}\u{62a}\u{61f} \u{645}\u{62d}\u{635}\u{648}\u{644} \u{628}\u{627}\u{632}\u{627}\u{631} \u{631}\u{627} \u{646}\u{634}\u{627}\u{646} \u{628}\u{62f}\u{647}.");
    }

    /// Long non-EVM hex — `long_hex`.
    #[test]
    fn long_hex_session_token_is_redacted() {
        let r = redact("session=5f1b2c3d4e5f60718293a4b5c6d7e8f90112233445566778899aabbccddeeff0");
        assert_eq!(r.findings[0].kind, "long_hex");
    }

    /// Priority: hex64 inside a `long_hex` range — only the stricter finding survives.
    #[test]
    fn overlapping_spans_deduplicate() {
        let key = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let r = redact(key);
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].kind, "private_key_hex");
    }
}
