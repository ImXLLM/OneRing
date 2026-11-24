//! KIAN v2 Pure Rust Adapter
//! Synthesized from:
//!   - `openai/codex` (`codex-rs/apply-patch/src/seek_sequence.rs` & `execpolicy`)
//!   - `langchain-ai/langchain` (`OutputFixing` / `RetryWithError` prompts)
//!   - `langflow-ai/langflow` (`WatsonXPlaceholderMiddleware` placeholder guard)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecDecision {
    Allow,
    Prompt,
    Forbidden,
}

#[derive(Debug, Clone, Copy)]
pub struct PrefixRule {
    pub pattern: &'static [&'static str],
    pub decision: ExecDecision,
    pub justification: &'static str,
}

/// Declarative command prefix rules inspired by `codex-rs/execpolicy`.
pub const KIAN_DEFAULT_EXEC_POLICY: &[PrefixRule] = &[
    PrefixRule {
        pattern: &["git", "reset", "--hard"],
        decision: ExecDecision::Forbidden,
        justification: "Destructive working-tree reset forbidden without explicit manual shell.",
    },
    PrefixRule {
        pattern: &["rm", "-rf", "/"],
        decision: ExecDecision::Forbidden,
        justification: "Root deletion forbidden.",
    },
    PrefixRule {
        pattern: &["git", "push", "--force"],
        decision: ExecDecision::Prompt,
        justification: "Force push rewrites remote history; require HITL approval.",
    },
    PrefixRule {
        pattern: &["ls"],
        decision: ExecDecision::Allow,
        justification: "Read-only directory listing.",
    },
    PrefixRule {
        pattern: &["pwd"],
        decision: ExecDecision::Allow,
        justification: "Read-only working directory check.",
    },
];

pub fn evaluate_exec_command<'a>(argv: &[&str], rules: &'a [PrefixRule]) -> (ExecDecision, &'a str) {
    let mut best: Option<&PrefixRule> = None;
    for rule in rules {
        if argv.len() >= rule.pattern.len() && &argv[..rule.pattern.len()] == rule.pattern {
            let replace = match best {
                None => true,
                Some(prev) => rule.pattern.len() > prev.pattern.len(),
            };
            if replace {
                best = Some(rule);
            }
        }
    }
    match best {
        Some(r) => (r.decision, r.justification),
        None => (ExecDecision::Prompt, "Unmatched command requires policy check or prompt."),
    }
}

fn normalize_unicode_punctuation(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| match c {
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}' | '\u{2212}' => '-',
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
            '\u{00A0}' | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' | '\u{2006}'
            | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200A}' | '\u{202F}' | '\u{205F}'
            | '\u{3000}' => ' ',
            other => other,
        })
        .collect()
}

/// 4-pass fuzzy context finder adapted directly from `openai/codex` `apply-patch/src/seek_sequence.rs`:
/// Pass 1: exact slice | Pass 2: trim_end | Pass 3: trim | Pass 4: Unicode punctuation normalization.
pub fn seek_sequence(lines: &[String], pattern: &[String], start: usize, eof: bool) -> Option<usize> {
    if pattern.is_empty() {
        return Some(start);
    }
    if pattern.len() > lines.len() {
        return None;
    }
    let search_start = if eof && lines.len() >= pattern.len() {
        (lines.len() - pattern.len()).max(start)
    } else {
        start
    };
    let max_idx = lines.len().saturating_sub(pattern.len());

    // Pass 1: exact
    for i in search_start..=max_idx {
        if lines[i..i + pattern.len()] == *pattern {
            return Some(i);
        }
    }
    // Pass 2: rstrip
    for i in search_start..=max_idx {
        if pattern.iter().enumerate().all(|(p, pat)| lines[i + p].trim_end() == pat.trim_end()) {
            return Some(i);
        }
    }
    // Pass 3: trim
    for i in search_start..=max_idx {
        if pattern.iter().enumerate().all(|(p, pat)| lines[i + p].trim() == pat.trim()) {
            return Some(i);
        }
    }
    // Pass 4: Unicode normalization
    for i in search_start..=max_idx {
        if pattern
            .iter()
            .enumerate()
            .all(|(p, pat)| normalize_unicode_punctuation(&lines[i + p]) == normalize_unicode_punctuation(pat))
        {
            return Some(i);
        }
    }
    None
}

/// Detects literal `<result-from-...>` / `<previous-output>` placeholder hallucinations in tool args
/// (adapted from Langflow's `WatsonXPlaceholderMiddleware`).
pub fn contains_placeholder_tool_arg(arg_value: &str) -> bool {
    let trimmed = arg_value.trim();
    trimmed.starts_with('<')
        && trimmed.ends_with('>')
        && (trimmed.contains("result-from")
            || trimmed.contains("previous-output")
            || trimmed.contains("extracted-value")
            || trimmed.contains("insert-"))
}

/// Builds a LangChain-inspired `RetryWithError` self-healing prompt for structured output or tool recovery.
pub fn build_output_fixing_prompt(instructions: &str, bad_completion: &str, error: &str) -> String {
    format!(
        "Instructions:\n--------------\n{instructions}\n--------------\n\
         Completion:\n--------------\n{bad_completion}\n--------------\n\n\
         Above, the Completion did not satisfy the constraints given in the Instructions.\n\
         Error:\n--------------\n{error}\n--------------\n\n\
         Please try again. Only respond with an answer that satisfies the constraints."
    )
}
