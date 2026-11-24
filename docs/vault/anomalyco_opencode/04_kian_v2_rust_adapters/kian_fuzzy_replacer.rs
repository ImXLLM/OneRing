//! KIAN v2 Multi-Pass Fuzzy Code Replacer (`crates/ai` & `crates/hermes`)
//!
//! Pure Rust port of OpenCode's surgical edit & fuzzy patch matching engines:
//! - `packages/core/src/tool/plugin/edit.ts` (Unicode normalization, trailing-whitespace trimming,
//!   CRLF-preserving LCS line endings, and miss-hint line voting)
//! - `packages/util/src/patch.ts` (`seek`: Exact -> Rstrip -> Trim -> UnicodeNormalized)
//! - Indentation-normalized fallback so KIAN Studio can patch HTML/CSS/JS/Rust files cleanly
//!   without regenerating the entire artifact.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchPass {
    Exact,
    UnicodeNormalized,
    LineTrimmedEnd,
    BlockAnchorTrimmed,
    IndentationNormalized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacementOutcome {
    pub content: String,
    pub pass: MatchPass,
    pub replacements: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplaceError {
    IdenticalInputs,
    EmptyOldString,
    NotFound { hint: Option<String> },
    MultipleMatches { count: usize, pass: MatchPass },
}

/// Normalizes typographic quotes, dashes, and non-breaking/CJK spaces (from `edit.ts` & `patch.ts`).
pub fn normalize_typographic(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => out.push('\''),
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => out.push('"'),
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}' | '\u{2212}' => {
                out.push('-')
            }
            '\u{00A0}' | '\u{2002}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => {
                out.push(' ')
            }
            _ => out.push(ch),
        }
    }
    out
}

fn dominant_line_ending(text: &str) -> &'static str {
    let crlf = text.matches("\r\n").count();
    let lf = text.matches('\n').count().saturating_sub(crlf);
    if crlf > lf { "\r\n" } else { "\n" }
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
    for (idx, b) in text.bytes().enumerate() {
        if b == b'\n' {
            lines.push((start, idx, &text[start..idx]));
            start = idx + 1;
        }
    }
    if start < text.len() {
        lines.push((start, text.len(), &text[start..]));
    }
    lines
}

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
        let matched = (0..pat_lines.len()).all(|k| eq(src_lines[i + k].2, pat_lines[k]));
        if matched {
            let start = src_lines[i].0;
            let end = src_lines[i + pat_lines.len() - 1].1;
            spans.push((start, end));
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
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                String::new()
            } else {
                l.chars().skip(min_indent).collect::<String>()
            }
        })
        .collect()
}

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
        let ok = win_norm
            .iter()
            .zip(pat_norm.iter())
            .all(|(a, b)| normalize_typographic(a.trim_end()) == normalize_typographic(b.trim_end()));
        if ok {
            spans.push((src_lines[i].0, src_lines[i + pat_raw.len() - 1].1));
            i += pat_raw.len();
        } else {
            i += 1;
        }
    }
    spans
}

/// Port of `edit.ts` `missHint`: detects already-applied `new_string` or votes closest matching lines.
fn miss_hint(view_lf: &str, old_lf: &str, new_lf: &str) -> Option<String> {
    let norm_view = normalize_typographic(view_lf);
    let norm_new = normalize_typographic(new_lf);
    if norm_new.trim().len() >= 20 && norm_view.contains(&norm_new) {
        return Some("newString is already in the file, so this edit may already be applied.".to_string());
    }
    let key = |s: &str| -> String {
        normalize_typographic(s)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let lines: Vec<&str> = view_lf.split('\n').collect();
    let search: Vec<&str> = old_lf.split('\n').collect();
    let mut best_start = 0usize;
    let mut best_votes = 0usize;
    for start in 0..lines.len() {
        let mut votes = 0usize;
        for (offset, sline) in search.iter().enumerate() {
            let k = key(sline);
            if k.is_empty() {
                continue;
            }
            if let Some(target) = lines.get(start + offset) {
                if key(target) == k {
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
        .map(|(idx, line)| format!("{}: {}", best_start + idx + 1, line))
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!("The closest lines in the file:\n{snippet}"))
}

/// Applies OpenCode's multi-pass fuzzy replacement while preserving file CRLF/LF line endings.
pub fn fuzzy_replace(
    source: &str,
    old_string: &str,
    new_string: &str,
    replace_all: bool,
) -> Result<ReplacementOutcome, ReplaceError> {
    if old_string == new_string {
        return Err(ReplaceError::IdenticalInputs);
    }
    if old_string.is_empty() {
        return Err(ReplaceError::EmptyOldString);
    }

    let ending = dominant_line_ending(source);
    let view = to_lf(source);
    let old_lf = to_lf(old_string);
    let new_lf = to_lf(new_string);

    // Pass 1: Exact LF match
    let mut pass = MatchPass::Exact;
    let mut spans = find_exact_spans(&view, &old_lf);

    // Pass 2: Unicode punctuation/space normalized (safe when byte lengths match, else line-level)
    if spans.is_empty() {
        let norm_view = normalize_typographic(&view);
        let norm_old = normalize_typographic(&old_lf);
        if norm_view.len() == view.len() && norm_old.len() == old_lf.len() {
            let candidate = find_exact_spans(&norm_view, &norm_old);
            if !candidate.is_empty() {
                pass = MatchPass::UnicodeNormalized;
                spans = candidate;
            }
        }
    }

    // Pass 3: Line trailing-whitespace trimmed + typographic normalized
    if spans.is_empty() {
        let candidate = find_line_spans(&view, &old_lf, |a, b| {
            normalize_typographic(a.trim_end()) == normalize_typographic(b.trim_end())
        });
        if !candidate.is_empty() {
            pass = MatchPass::LineTrimmedEnd;
            spans = candidate;
        }
    }

    // Pass 4: Block-anchor full line trim (mirrors `patch.ts` `trim`/`normalized`)
    if spans.is_empty() {
        let candidate = find_line_spans(&view, &old_lf, |a, b| {
            normalize_typographic(a.trim()) == normalize_typographic(b.trim())
        });
        if !candidate.is_empty() {
            pass = MatchPass::BlockAnchorTrimmed;
            spans = candidate;
        }
    }

    // Pass 5: Indentation-normalized multi-line match
    if spans.is_empty() {
        let candidate = find_indent_normalized_spans(&view, &old_lf);
        if !candidate.is_empty() {
            pass = MatchPass::IndentationNormalized;
            spans = candidate;
        }
    }

    if spans.is_empty() {
        return Err(ReplaceError::NotFound {
            hint: miss_hint(&view, &old_lf, &new_lf),
        });
    }
    if spans.len() > 1 && !replace_all {
        return Err(ReplaceError::MultipleMatches {
            count: spans.len(),
            pass,
        });
    }

    let targets = if replace_all { &spans[..] } else { &spans[..1] };
    let mut out = view;
    for &(start, end) in targets.iter().rev() {
        let rep = new_lf.trim_end_matches('\n');
        let slice_rep = if old_lf.ends_with('\n') && pass <= MatchPass::UnicodeNormalized {
            new_lf.as_str()
        } else {
            rep
        };
        out.replace_range(start..end, slice_rep);
    }

    Ok(ReplacementOutcome {
        content: restore_line_endings(&out, ending),
        pass,
        replacements: targets.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_and_crlf_preservation() {
        let src = "<div>\r\n  <h1>Hello</h1>\r\n</div>\r\n";
        let res = fuzzy_replace(src, "  <h1>Hello</h1>", "  <h1>KIAN</h1>", false).unwrap();
        assert_eq!(res.pass, MatchPass::Exact);
        assert!(res.content.contains("\r\n  <h1>KIAN</h1>\r\n"));
    }

    #[test]
    fn handles_unicode_quotes_and_indentation_drift() {
        let src = "<section>\n        <p>It's ready — now</p>\n</section>\n";
        let res = fuzzy_replace(
            src,
            "  <p>It’s ready – now</p>",
            "        <p>Ready!</p>",
            false,
        )
        .unwrap();
        assert_eq!(res.pass, MatchPass::BlockAnchorTrimmed);
        assert!(res.content.contains("<p>Ready!</p>"));
    }

    #[test]
    fn miss_hint_detects_already_applied_edit() {
        let src = "function renderStudioBanner() {\n  return 'KIAN Studio v2';\n}\n";
        let err = fuzzy_replace(
            src,
            "function oldBanner() {}",
            "function renderStudioBanner() {\n  return 'KIAN Studio v2';\n}",
            false,
        )
        .unwrap_err();
        match err {
            ReplaceError::NotFound { hint: Some(h) } => assert!(h.contains("already be applied")),
            other => panic!("unexpected: {other:?}"),
        }
    }
}
