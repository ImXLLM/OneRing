//! `history_repair` — conversation history repair, stream-argument stabilization, and the studio self-healing loop.
//!
//! Line-by-line port from the CopilotKit/OpenBot treasury (Wave 2):
//! - `openbot/app/src/lib/copilot/repair-history.ts` → [`repair_unanswered_tool_calls`]:
//!   Every unanswered tool call (stream cut, page reload, result relocation) is immediately
//!   filled right after the assistant turn with the synthetic "UNANSWERED" result so the upstream
//!   provider never kills the whole conversation with `MissingToolResultsError` / HTTP 400.
//! - `openbot/settled-args.ts` → [`is_settled_stream_arg`]: during streaming, values
//!   mid-way (incomplete hex color, unfinished URL, number ending in "." or "-") are held
//!   back so UI components do not jump while typing (zero-jitter).
//! - `a2ui-toolkit/recovery.ts` → [`validate_and_build_self_healing_feedback`]:
//!   Validation of the studio output envelope and construction of a precise repair prompt for the model's next turn.
//!
//! All functions are deterministic (no network, no randomness) and operate on lightweight
//! serializable structures — this crate knows nothing about the database.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Synthetic result for a call whose answer never arrived — a uniform "did not run" voice
/// (port of `notRun` in `shared/approval-outcomes.ts`): what happened + two next paths.
pub const UNANSWERED: &str = "Not run: no result came back for this call before the conversation moved on. Run it again if you still need it, or say in your answer that you could not get it.";

/// Default cap on studio self-healing loop attempts (first try + retries).
pub const MAX_A2UI_ATTEMPTS: u8 = 3;

/// Announcement of a tool call inside an assistant message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallDecl {
    /// Call id — the result binds to this announcement by that same id.
    pub id: String,
    /// Tool name (for logs and audit pages).
    pub name: String,
}

/// A lightweight, serializable turn from the transcript facing the provider — AG-UI/OpenBot
/// roles: `system` / `user` / `assistant` / `tool` (a tool result carries its own
/// independent role). No dependency on any provider or database; fields are
/// serialized camelCase (AG-UI convention: `toolCalls`, `toolCallId`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase", rename_all_fields = "camelCase")]
pub enum ToolCallTurn {
    System { content: String },
    User { content: String },
    Assistant {
        content: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tool_calls: Vec<ToolCallDecl>,
    },
    /// Tool result — binds to the announcement above it via `toolCallId`.
    Tool { tool_call_id: String, content: String },
}

/// Repair output: the original message untouched, or an injected synthetic result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairedMessage {
    /// The message itself (original or synthetic).
    pub turn: ToolCallTurn,
    /// `true` means this result was produced by the repair, not by the real tool.
    pub synthesized: bool,
}

/// Tool result with its `toolCallId`.
fn tool_call_id_of(turn: &ToolCallTurn) -> Option<&str> {
    match turn {
        ToolCallTurn::Tool { tool_call_id, .. } => Some(tool_call_id),
        _ => None,
    }
}

/// Tool calls with no result anywhere in the array — regardless of ordering
/// (port of `unansweredToolCalls`). Duplicate ids are returned once.
///
/// # Errors
/// None.
#[must_use]
pub fn unanswered_tool_calls(turns: &[ToolCallTurn]) -> BTreeSet<String> {
    let answered: BTreeSet<&str> = turns.iter().filter_map(tool_call_id_of).collect();
    let mut unanswered = BTreeSet::new();
    for turn in turns {
        if let ToolCallTurn::Assistant { tool_calls, .. } = turn {
            for call in tool_calls {
                if !answered.contains(call.id.as_str()) {
                    unanswered.insert(call.id.clone());
                }
            }
        }
    }
    unanswered
}

/// The same messages, except every tool call has a result that comes "after
/// it" (port of `repairUnansweredToolCalls`).
///
/// The crux of the whole matter: the provider binds a result to the announcement above it, so
/// a result stored "before" its own call (as happens after handoff/AskPerson/MCP
/// storage) means, to the provider, an unanswered call and the death of the entire
/// conversation. The early result is "relocated", not replaced — it is real and has more
/// to say; only a call with no result anywhere receives the synthetic
/// [`UNANSWERED`] sentence. A result that answers no call is dropped.
///
/// Known answers from outside the transcript (e.g. a HITL decision still pending on a
/// request) come from `known` and take precedence over the synthetic sentence.
///
/// Empty or healthy input is returned unchanged; running twice is idempotent.
#[must_use]
pub fn repair_unanswered_tool_calls(
    given: &[ToolCallTurn],
    known: &BTreeMap<String, String>,
) -> Vec<RepairedMessage> {
    // Announced ids in order of occurrence.
    let mut called: BTreeSet<&str> = BTreeSet::new();
    // Answered from the provider's perspective: a result after its own call.
    let mut answered: BTreeSet<&str> = BTreeSet::new();
    // A real result that arrived before its own call — kept for correct placement.
    let mut early: BTreeMap<&str, &ToolCallTurn> = BTreeMap::new();
    // Indices of results that, where they sit, answer nothing.
    let mut misplaced: BTreeSet<usize> = BTreeSet::new();

    for (index, turn) in given.iter().enumerate() {
        match turn {
            ToolCallTurn::Assistant { tool_calls, .. } => {
                for call in tool_calls {
                    called.insert(call.id.as_str());
                }
            }
            ToolCallTurn::Tool { tool_call_id, .. } => {
                if called.contains(tool_call_id.as_str()) && !answered.contains(tool_call_id.as_str())
                {
                    answered.insert(tool_call_id.as_str());
                } else {
                    // before its own call, a second result for an already-answered call,
                    // or a result that answers no call.
                    early.entry(tool_call_id.as_str()).or_insert(turn);
                    misplaced.insert(index);
                }
            }
            ToolCallTurn::System { .. } | ToolCallTurn::User { .. } => {}
        }
    }

    let mut missing_any = false;
    for turn in given {
        if let ToolCallTurn::Assistant { tool_calls, .. } = turn
            && tool_calls.iter().any(|c| !answered.contains(c.id.as_str()))
        {
            missing_any = true;
            break;
        }
    }
    if !missing_any && misplaced.is_empty() {
        return given
            .iter()
            .cloned()
            .map(|turn| RepairedMessage { turn, synthesized: false })
            .collect();
    }

    let mut repaired: Vec<RepairedMessage> = Vec::with_capacity(given.len() + 4);
    let mut filled: BTreeSet<&str> = answered.clone();
    for (index, turn) in given.iter().enumerate() {
        if misplaced.contains(&index) {
            continue;
        }
        let is_assistant = matches!(turn, ToolCallTurn::Assistant { .. });
        repaired.push(RepairedMessage { turn: turn.clone(), synthesized: false });
        if !is_assistant {
            continue;
        }
        if let ToolCallTurn::Assistant { tool_calls, .. } = turn {
            for call in tool_calls {
                if filled.contains(call.id.as_str()) {
                    continue;
                }
                // immediately after the assistant message and before any later message: the provider
                // must see the result after the announcement, and the order must be the order of the
                // `tool_calls` array itself.
                let (synthesized_turn, synthesized) = match early.get(call.id.as_str()) {
                    Some(moved) => ((*moved).clone(), false),
                    None => (
                        ToolCallTurn::Tool {
                            tool_call_id: call.id.clone(),
                            content: known
                                .get(&call.id)
                                .cloned()
                                .unwrap_or_else(|| UNANSWERED.to_owned()),
                        },
                        true,
                    ),
                };
                repaired.push(RepairedMessage { turn: synthesized_turn, synthesized });
                // A duplicated call receives only one repair result.
                filled.insert(call.id.as_str());
            }
        }
    }
    repaired
}

/// Is the value being streamed "settled" and allowed to render in the live UI?
///
/// Until the call completes (`is_stream_complete == false`), mid-way values are
/// held back so components do not jump:
/// - Incomplete hex color: starts with `#` and its length is not 4/7/9 (`#abc` is complete, `#abcde`
///   is still in flight);
/// - Unfinished URL: starts with `http://` or `https://` and ends with
///   `/` or `.`, or is shorter than 12 characters;
/// - A number being typed that ends in "." or "-" ("12" on the way to "12.5").
///
/// With `is_stream_complete == true`, everything is settled immediately.
#[must_use]
pub fn is_settled_stream_arg(key: &str, raw_val: &str, is_stream_complete: bool) -> bool {
    let _ = key; // The key name is a no-op under the current contract; the value has the final say.
    if is_stream_complete {
        return true;
    }
    // Hex color — only complete lengths (3/6/8 digits + `#`).
    if raw_val.starts_with('#') {
        return matches!(raw_val.chars().count(), 4 | 7 | 9);
    }
    // Unfinished URL.
    if raw_val.starts_with("http://") || raw_val.starts_with("https://") {
        return !(raw_val.ends_with('/')
            || raw_val.ends_with('.')
            || raw_val.chars().count() < 12);
    }
    // A number ending in "." or "-" — it does not yet know how many digits come next.
    if (raw_val.ends_with('.') || raw_val.ends_with('-')) && is_all_digits(&raw_val[..raw_val.len() - 1])
    {
        return false;
    }
    true
}

/// Is the non-empty string composed only of ASCII digits?
fn is_all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Counts open `<tag …>` tags with a word boundary (port of `<(style|script)\b` in TS).
fn count_open_tags(lower_html: &str, tag: &str) -> usize {
    let pattern = format!("<{tag}");
    let mut count = 0;
    let mut cursor = 0;
    while let Some(found) = lower_html[cursor..].find(&pattern) {
        let abs = cursor + found;
        let after = &lower_html[abs + pattern.len()..];
        let boundary_ok = after.is_empty()
            || after
                .chars()
                .next()
                .is_some_and(|c| !(c.is_ascii_alphanumeric() || c == '-'));
        if boundary_ok {
            count += 1;
        }
        cursor = abs + pattern.len();
    }
    count
}

/// Defects of the studio output envelope — each defect with a code and a precise description for the repair prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioDefect {
    /// Defect code — exactly what the model sees in the repair prompt (`[missing_doctype` …).
    pub code: String,
    /// Precise one-sentence description: which tag/section is missing or extra.
    pub message: String,
}

/// Full validation of the studio envelope + construction of self-healing feedback (port of `recovery.ts`).
///
/// A complete studio output must have a full HTML envelope: a leading `<!DOCTYPE html>` and
/// a trailing `</html>`, balanced `<script>` and `<style>` tags, and no leak of the
/// markdown fence (` ``` `). If healthy, `Ok(())`; otherwise:
/// - `attempt < max_attempts` → `Err` with a structured repair prompt that states exactly
///   which tag/defect is missing (the model builds exactly that on its next turn);
/// - Attempts exhausted → `Err` with the `A2UI_RECOVERY_EXHAUSTED` envelope for
///   a safe fallback render.
///
/// This same validation must decide both the tool retry and the render suppress
/// decisions so they never conflict (the "ONE validator" principle in recovery.ts).
///
/// # Errors
/// Every invalid output is a structured `Err` — this feedback itself becomes the input
/// of the model's next turn.
pub fn validate_and_build_self_healing_feedback(
    html_or_json: &str,
    attempt: u8,
    max_attempts: u8,
) -> Result<(), String> {
    let defects = studio_defects(html_or_json);
    if defects.is_empty() {
        return Ok(());
    }
    if attempt >= max_attempts {
        return Err(format!(
            "A2UI_RECOVERY_EXHAUSTED: Failed to generate valid Studio output after {max_attempts} attempt(s). \
             Do not retry silently; render the safe fallback envelope instead."
        ));
    }
    let listed = defects
        .iter()
        .map(|d| format!("- [{}] {}", d.code, d.message))
        .collect::<Vec<_>>()
        .join("\n");
    Err(format!(
        "A2UI_REPAIR_FEEDBACK: attempt {attempt}/{max_attempts} produced an invalid Studio document — \
         fix exactly these defects and regenerate the complete document:\n{listed}"
    ))
}

/// List of structural defects of the studio output (no content judgment).
#[must_use]
pub fn studio_defects(html_or_json: &str) -> Vec<StudioDefect> {
    let mut defects = Vec::new();
    let lower = html_or_json.to_ascii_lowercase();

    if !lower.contains("<!doctype html") {
        defects.push(StudioDefect {
            code: "missing_doctype".into(),
            message: "the document must start with a complete <!DOCTYPE html> envelope".into(),
        });
    }
    if !lower.contains("</html>") {
        defects.push(StudioDefect {
            code: "missing_closing_html".into(),
            message: "the document must end with a closing </html> tag".into(),
        });
    }
    for (tag, open_code, close_code) in
        [("script", "unbalanced_script", "unclosed_script"), ("style", "unbalanced_style", "unclosed_style")]
    {
        let open = count_open_tags(&lower, tag);
        let close = lower.matches(&format!("</{tag}>")).count();
        if open > close {
            defects.push(StudioDefect {
                code: close_code.into(),
                message: format!("{open} <{tag}> block(s) opened but only {close} closed — every <{tag}> needs its closing </{tag}>"),
            });
        } else if close > open {
            defects.push(StudioDefect {
                code: open_code.into(),
                message: format!("{close} </{tag}> closer(s) but only {open} <{tag}> opener(s) — remove the extra closing tag"),
            });
        }
    }
    if lower.contains("```") {
        defects.push(StudioDefect {
            code: "markdown_fence_leak".into(),
            message: "raw markdown fence (```) leaked into the output — emit the bare HTML document with no code fences".into(),
        });
    }
    defects
}

/// Validation shortcut without a repair prompt — healthy or not.
#[must_use]
pub fn is_valid_studio_output(html_or_json: &str) -> bool {
    studio_defects(html_or_json).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assistant(calls: &[(&str, &str)]) -> ToolCallTurn {
        ToolCallTurn::Assistant {
            content: String::new(),
            tool_calls: calls
                .iter()
                .map(|(id, name)| ToolCallDecl { id: (*id).into(), name: (*name).into() })
                .collect(),
        }
    }

    fn tool(id: &str, content: &str) -> ToolCallTurn {
        ToolCallTurn::Tool { tool_call_id: id.into(), content: content.into() }
    }

    /// Unanswered call → the synthetic UNANSWERED result immediately after the assistant turn.
    #[test]
    fn orphaned_tool_call_gets_synthetic_result_right_after_assistant() {
        let turns = vec![
            ToolCallTurn::User { content: "\u{642}\u{6cc}\u{645}\u{62a} \u{631}\u{627} \u{628}\u{6af}\u{648}".into() },
            assistant(&[("c1", "market.price")]),
            ToolCallTurn::User { content: "\u{62e}\u{648}\u{628} \u{627}\u{633}\u{62a}".into() },
        ];
        let repaired = repair_unanswered_tool_calls(&turns, &BTreeMap::new());
        assert_eq!(repaired.len(), 4);
        assert!(!repaired[0].synthesized && !repaired[1].synthesized);
        assert!(repaired[2].synthesized, "synthetic result must sit at index 2");
        assert_eq!(
            repaired[2].turn,
            ToolCallTurn::Tool { tool_call_id: "c1".into(), content: UNANSWERED.into() }
        );
        assert_eq!(repaired[3].turn, turns[2], "later user turn stays after the repair");
    }

    /// A result stored before its call is relocated, not replaced; an orphaned
    /// result is dropped; running twice is idempotent.
    #[test]
    fn early_result_is_moved_orphans_dropped_and_repair_is_idempotent() {
        let turns = vec![
            tool("c9", "\u{646}\u{62a}\u{6cc}\u{62c}\u{647}\u{654} \u{628}\u{6cc}‌\u{635}\u{627}\u{62d}\u{628}"),
            tool("c1", "Handed to Knowledge — \u{67e}\u{627}\u{633}\u{62e} \u{648}\u{627}\u{642}\u{639}\u{6cc}"),
            assistant(&[("c1", "kb.search"), ("c2", "market.price")]),
        ];
        let repaired = repair_unanswered_tool_calls(&turns, &BTreeMap::new());
        // c9 dropped; c1 relocated after the assistant; c2 synthetic.
        assert_eq!(repaired.len(), 3, "orphan dropped; assistant + moved + synthetic");
        assert!(matches!(repaired[0].turn, ToolCallTurn::Assistant { .. }));
        assert!(!repaired[1].synthesized);
        assert_eq!(
            repaired[1].turn,
            tool("c1", "Handed to Knowledge — \u{67e}\u{627}\u{633}\u{62e} \u{648}\u{627}\u{642}\u{639}\u{6cc}"),
            "the real early result is moved, not replaced"
        );
        assert!(repaired[2].synthesized);
        assert_eq!(repaired[2].turn, tool("c2", UNANSWERED));

        // Healthy from the provider's perspective: every announcement has a result after it.
        let plain: Vec<ToolCallTurn> = repaired.iter().map(|m| m.turn.clone()).collect();
        let second = repair_unanswered_tool_calls(&plain, &BTreeMap::new());
        assert_eq!(
            plain,
            second.iter().map(|m| m.turn.clone()).collect::<Vec<_>>(),
            "second repair pass must be a no-op"
        );
        assert!(second.iter().all(|m| !m.synthesized));
    }

    /// A known external answer takes precedence over the synthetic sentence; the duplicated second result is dropped.
    #[test]
    fn known_answer_wins_and_duplicate_results_are_dropped() {
        let turns = vec![
            assistant(&[("c1", "escrow.create")]),
            tool("c1", "\u{646}\u{62a}\u{6cc}\u{62c}\u{647}\u{654} \u{627}\u{648}\u{644}"),
            tool("c1", "\u{646}\u{62a}\u{6cc}\u{62c}\u{647}\u{654} \u{62f}\u{648}\u{645} \u{62a}\u{6a9}\u{631}\u{627}\u{631}\u{6cc}"),
        ];
        let mut known = BTreeMap::new();
        known.insert("c1".to_owned(), "\u{62a}\u{623}\u{6cc}\u{6cc}\u{62f} \u{634}\u{62f}: \u{67e}\u{627}\u{6a9}\u{62a}‌\u{633}\u{627}\u{632} \u{628}\u{633}\u{62a}\u{647} \u{634}\u{62f}".to_owned());
        let repaired = repair_unanswered_tool_calls(&turns, &known);
        // Both existing results come after the call: the first answered, the second misplaced → dropped.
        assert_eq!(repaired.len(), 2);
        assert!(!repaired[1].synthesized);
        assert_eq!(repaired[1].turn, tool("c1", "\u{646}\u{62a}\u{6cc}\u{62c}\u{647}\u{654} \u{627}\u{648}\u{644}"));

        // And if there is no result at all, the known answer takes UNANSWERED's place.
        let orphan = vec![assistant(&[("c1", "escrow.create")])];
        let repaired = repair_unanswered_tool_calls(&orphan, &known);
        assert_eq!(repaired.len(), 2);
        assert!(repaired[1].synthesized);
        assert_eq!(repaired[1].turn, tool("c1", "\u{62a}\u{623}\u{6cc}\u{6cc}\u{62f} \u{634}\u{62f}: \u{67e}\u{627}\u{6a9}\u{62a}‌\u{633}\u{627}\u{632} \u{628}\u{633}\u{62a}\u{647} \u{634}\u{62f}"));
    }

    /// Persian text in the transcript must not break the repair path (a real user input).
    #[test]
    fn unanswered_scan_ignores_user_and_system_turns() {
        let turns = vec![
            ToolCallTurn::System { content: "\u{62a}\u{648} \u{645}\u{63a}\u{632} \u{6a9}\u{6cc}\u{627}\u{646}\u{6cc}".into() },
            ToolCallTurn::User { content: "tool \u{645}\u{62a}\u{646} \u{628}\u{627} \u{6a9}\u{644}\u{645}\u{647}\u{654} tool".into() },
            assistant(&[("c1", "kb.search"), ("c2", "kb.search")]),
        ];
        let unanswered = unanswered_tool_calls(&turns);
        assert_eq!(unanswered.iter().cloned().collect::<Vec<_>>(), vec!["c1", "c2"]);
        // A duplicated call receives only one repair result.
        let repaired = repair_unanswered_tool_calls(&turns, &BTreeMap::new());
        assert_eq!(repaired.len(), 5);
        assert!(repaired[3].synthesized && repaired[4].synthesized);
    }

    /// Hex color: complete ones (4/7/9) settled, mid-way unsettled.
    #[test]
    fn hex_colors_settle_only_on_complete_lengths() {
        for (val, expect) in [
            ("#abc", true),      // 4 characters — complete
            ("#abcdef", true),   // 7 characters — complete
            ("#aabbccdd", true), // 9 characters — complete with alpha
            ("#ab", false),      // mid-way
            ("#abcde", false),   // mid-way
            ("#", false),
        ] {
            assert_eq!(is_settled_stream_arg("color", val, false), expect, "val={val}");
        }
    }

    /// An unfinished URL (ending in "/" or ".", or shorter than 12) is held back.
    #[test]
    fn incomplete_urls_are_held_back() {
        assert!(!is_settled_stream_arg("url", "http://", false));
        assert!(!is_settled_stream_arg("href", "https://a.", false));
        assert!(!is_settled_stream_arg("src", "https://a.io/", false));
        assert!(!is_settled_stream_arg("url", "http://a.io", false), "11 chars is incomplete");
        assert!(is_settled_stream_arg("url", "https://a.io", false), "12 chars is settled");
        assert!(is_settled_stream_arg("url", "https://kian.dev/market", false));
        assert!(is_settled_stream_arg("url", "https://a.io/", true), "complete stream always settles");
    }

    /// A number ending in "." or "-" is unsettled so the UI bar does not jump.
    #[test]
    fn trailing_numeric_strings_are_held_back() {
        assert!(!is_settled_stream_arg("amount", "12.", false), "12 is on its way to 12.5");
        assert!(!is_settled_stream_arg("amount", "125-", false));
        assert!(is_settled_stream_arg("amount", "12.5", false));
        assert!(is_settled_stream_arg("amount", "125", false));
        assert!(is_settled_stream_arg("label", "\u{633}\u{644}\u{627}\u{645}.", false), "non-numeric tail is not a number");
        assert!(is_settled_stream_arg("amount", "-", false), "empty digits before '-'");
    }

    /// The full document is healthy; defects are named precisely.
    #[test]
    fn self_healing_validates_complete_envelope_and_names_exact_defects() {
        let good = "<!DOCTYPE html><html><head><style>body{color:#0ff}</style></head><body><script>const x=1;</script>\u{633}\u{644}\u{627}\u{645}</body></html>";
        assert_eq!(validate_and_build_self_healing_feedback(good, 0, MAX_A2UI_ATTEMPTS), Ok(()));
        assert!(is_valid_studio_output(good));

        // No doctype and no </html> — both defects reported with a precise code.
        let feedback = validate_and_build_self_healing_feedback("<div>\u{633}\u{644}\u{627}\u{645}</div>", 0, MAX_A2UI_ATTEMPTS)
            .expect_err("incomplete envelope must fail");
        assert!(feedback.contains("A2UI_REPAIR_FEEDBACK"), "feedback: {feedback}");
        assert!(feedback.contains("[missing_doctype]"));
        assert!(feedback.contains("[missing_closing_html]"));
        assert!(feedback.contains("attempt 0/3"));
    }

    /// An unclosed script/style and a markdown fence each get a named defect.
    #[test]
    fn self_healing_flags_unclosed_script_and_markdown_fence() {
        let feedback = validate_and_build_self_healing_feedback(
            "<!DOCTYPE html><html><body><script>let x = 1;\n```html\n",
            1,
            MAX_A2UI_ATTEMPTS,
        )
        .expect_err("unclosed script + fence must fail");
        assert!(feedback.contains("[unclosed_script]"), "feedback: {feedback}");
        assert!(feedback.contains("[markdown_fence_leak]"));

        // The extra cast-off tag is reported too (envelope leak).
        let feedback = validate_and_build_self_healing_feedback(
            "<!DOCTYPE html><html></html></style>",
            0,
            MAX_A2UI_ATTEMPTS,
        )
        .expect_err("extra closer must fail");
        assert!(feedback.contains("[unbalanced_style]"), "feedback: {feedback}");
    }

    /// Before the cap, the repair prompt; after the cap, the EXHAUSTED envelope for a safe fallback.
    #[test]
    fn self_healing_switches_to_exhausted_envelope_after_max_attempts() {
        let bad = "<div>\u{646}\u{627}\u{642}\u{635}</div>";
        let repair = validate_and_build_self_healing_feedback(bad, 2, MAX_A2UI_ATTEMPTS)
            .expect_err("attempt 2 of 3 still repairs");
        assert!(repair.starts_with("A2UI_REPAIR_FEEDBACK"));
        let exhausted = validate_and_build_self_healing_feedback(bad, 3, MAX_A2UI_ATTEMPTS)
            .expect_err("exhausted is still an Err");
        assert!(exhausted.starts_with("A2UI_RECOVERY_EXHAUSTED"), "got: {exhausted}");
        assert!(exhausted.contains("after 3 attempt(s)"));
    }

    /// The structures are lightweight and serializable, and roles are serialized with the `role` tag (AG-UI convention).
    #[test]
    fn turns_serialize_with_role_tag_and_repaired_flags() {
        let turns = vec![assistant(&[("c1", "kb.search")])];
        let json = serde_json::to_string(&turns).expect("serialize");
        assert!(json.contains("\"role\":\"assistant\""), "json: {json}");
        assert!(json.contains("\"toolCalls\""), "camelCase toolCalls: {json}");

        let repaired = repair_unanswered_tool_calls(&turns, &BTreeMap::new());
        let json = serde_json::to_string(&repaired).expect("serialize repaired");
        assert!(json.contains("\"synthesized\":true"), "json: {json}");
        assert!(json.contains("\"role\":\"tool\""));
    }
}
