//! `tool_parser` — tool-call stream accumulator and the Hermes XML+JSON parser.
//!
//! Line-by-line port from the `NousResearch` treasury (Wave 2 - Step 2):
//! - `agent/chat_completion_helpers_tool_calls.py` → [`ToolCallStreamAccumulator`]:
//!   Accumulating `tool_calls` stream deltas with two fate-changing fixes: (1) the function name is
//!   **assigned** (`=`) instead of appended (`+=`) — some gateways (`MiniMax` on
//!   NVIDIA NIM, free `OpenRouter` models) send the full name in every delta, and
//!   blind concatenation builds "`read_fileread_file`"; (2) JSON bracket depth is tracked
//!   so that if the provider reuses the same `index` after closing the JSON object,
//!   a fresh, clean slot opens instead of continuing the previous call.
//! - `Hermes-Function-Calling/utils.py` → [`extract_xml_tool_calls`]: extracting
//!   `{"name": ..., "arguments": {...}}` blocks from the assistant text,
//!   with a lenient fallback (trailing comma, single-quoted keys) — open-weight
//!   `:free` models without the provider's native function-calling can also call Kian's
//!   tools.
//!
//! All functions are deterministic; no I/O and no randomness involved.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap, HashSet};

/// A stream input delta — the neutral shape of a chat-completions `tool_calls` delta
/// (any provider translates into these four fields).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolCallDelta {
    /// Slot index from the provider; `None` counts as `0` (port of `py`: `raw_idx`).
    pub index: Option<usize>,
    /// Call id — a fresh id at the same index means "the next call".
    pub id: Option<String>,
    /// Function name if present in this delta (some gateways send it in every delta).
    pub name: Option<String>,
    /// The next chunk of the arguments text (concatenating chunks = the full JSON).
    pub arguments: Option<String>,
}

/// A completed tool call after accumulation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccumulatedToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Incremental bracket-scan state for one slot — we never restart from the top of the text:
/// scanning from the "latest chunk" stays linear and is free for providers that never
/// reach here (port of `_arg_scan`).
#[derive(Debug, Clone, Copy, Default)]
struct ArgScanState {
    parts_scanned: usize,
    depth: i64,
    in_string: bool,
    escaped: bool,
    started: bool,
}

/// The `tool_calls` stream accumulator — port of `_ToolCallAccumulator`.
///
/// The `Ollama` key point: Ollama's compatible endpoint sends every call of a parallel
/// batch on `index=0`, separated only by `id`; so
/// a fresh id on a seen index routes to a fresh slot — but only when
/// the current call is structurally closed.
#[derive(Debug, Default)]
pub struct ToolCallStreamAccumulator {
    slots: BTreeMap<usize, AccumulatedToolCall>,
    notified: HashSet<usize>,
    last_id_at_idx: HashMap<usize, String>,
    active_slot_by_idx: HashMap<usize, usize>,
    argument_parts: HashMap<usize, Vec<String>>,
    arg_scan: HashMap<usize, ArgScanState>,
}

impl ToolCallStreamAccumulator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges one delta; the first time the function name completes, that name is returned.
    ///
    /// Rules (a line-by-line port of the Python `feed`):
    /// 1. `None` index = `0`; a numeric id comes through as a string (Poolside).
    /// 2. A fresh id on a seen index opens a fresh slot only when
    ///    [`Self::starts_new_call`] says "the current call is closed".
    /// 3. The function name is **assigned**, not appended — otherwise the repeated name of
    ///    `MiniMax`/NIM gateways builds "`read_fileread_file`".
    /// 4. Argument chunks are stored separately; [`Self::materialize`] joins them once
    ///    (delta-by-delta concatenation is quadratic on large arguments).
    pub fn feed(&mut self, delta: &ToolCallDelta) -> Option<String> {
        let raw_idx = delta.index.unwrap_or(0);
        let delta_id = delta.id.clone().unwrap_or_default();

        self.active_slot_by_idx.entry(raw_idx).or_insert(raw_idx);
        if !delta_id.is_empty()
            && self
                .last_id_at_idx
                .get(&raw_idx)
                .is_some_and(|prev| prev != &delta_id)
            && self.starts_new_call(
                delta.name.as_deref(),
                self.active_slot_by_idx[&raw_idx],
            )
        {
            let fresh = self.slots.keys().next_back().copied().unwrap_or(0) + 1;
            self.active_slot_by_idx.insert(raw_idx, fresh);
        }
        if !delta_id.is_empty() {
            self.last_id_at_idx.insert(raw_idx, delta_id);
        }
        let idx = self.active_slot_by_idx[&raw_idx];

        let entry = self.slots.entry(idx).or_default();
        if let Some(id) = delta.id.as_deref().filter(|id| !id.is_empty()) {
            entry.id = id.to_string();
        }
        if let Some(name) = delta.name.as_deref().filter(|name| !name.is_empty()) {
            // Assignment, not concatenation — the MiniMax / NVIDIA NIM / OpenRouter fix
            // This is where "read_fileread_file" comes from.
            entry.name = name.to_string();
        }
        if let Some(args) = delta.arguments.as_deref().filter(|args| !args.is_empty()) {
            self.argument_parts.entry(idx).or_default().push(args.to_owned());
        }

        let name = entry.name.clone();
        if !name.is_empty() && self.notified.insert(idx) {
            return Some(name);
        }
        None
    }

    /// Does a fresh id on a consumed index open a new call or continue the current one?
    ///
    /// Only a **named** delta may open a call (LM Studio and Kimi send a fresh id on every
    /// argument chunk without a name; some `MiniMax` paths send the
    /// name every time too) and a name opens only when the current call's
    /// arguments have closed — two parallel calls of one tool must still
    /// be separated from each other (port of `_starts_new_call`).
    fn starts_new_call(&mut self, incoming_name: Option<&str>, slot: usize) -> bool {
        let Some(name) = incoming_name.filter(|n| !n.is_empty()) else {
            return false;
        };
        if self.slots.get(&slot).map(|e| e.name.as_str()) != Some(name) {
            return true;
        }
        self.arguments_closed(slot)
    }

    /// Structural completion (bracket balance outside strings) — not strict valid JSON, so
    /// a recoverable call with a trailing comma also closes its slot. Only the chunks
    /// added since the last scan are examined (port of `_arguments_closed`).
    fn arguments_closed(&mut self, slot: usize) -> bool {
        let mut state = self.arg_scan.get(&slot).copied().unwrap_or_default();
        let parts = self.argument_parts.get(&slot).cloned().unwrap_or_default();
        for ch in parts.iter().flat_map(|p| p.chars()).skip(state.parts_scanned) {
            if state.in_string {
                if state.escaped {
                    state.escaped = false;
                } else if ch == '\\' {
                    state.escaped = true;
                } else if ch == '"' {
                    state.in_string = false;
                }
            } else if ch == '"' {
                state.in_string = true;
            } else if ch == '{' || ch == '[' {
                state.depth += 1;
                state.started = true;
            } else if ch == '}' || ch == ']' {
                state.depth -= 1;
            }
        }
        state.parts_scanned = parts.iter().map(|p| p.chars().count()).sum();
        self.arg_scan.insert(slot, state);
        state.started && state.depth == 0 && !state.in_string
    }

    /// Joins each slot's argument chunks into `arguments`; idempotent —
    /// and slots return in index order (port of `materialize`).
    #[must_use]
    pub fn materialize(&self) -> Vec<AccumulatedToolCall> {
        self.slots
            .iter()
            .map(|(idx, entry)| AccumulatedToolCall {
                id: entry.id.clone(),
                name: entry.name.clone(),
                arguments: self
                    .argument_parts
                    .get(idx)
                    .map_or_else(|| entry.arguments.clone(), |parts| parts.concat()),
            })
            .collect()
    }

    /// Number of slots opened — for telemetry and tests.
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }
}

/// A tool call extracted from the assistant text.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Extracts `{"name": ..., "arguments": ...}` blocks
/// from the assistant's textual reply (port of `validate_and_extract_tool_calls` in
/// `Hermes-Function-Calling/utils.py`).
///
/// each block is read first with strict `serde_json`; on failure, a lenient fallback:
/// trailing commas and single-quoted keys/strings (the equivalent of `ast.literal_eval`).
/// Broken blocks are discarded and the rest continue — no aggregated error is returned.
/// Text outside the blocks (analysis, the Hermes preamble) is ignored.
#[must_use]
pub fn extract_xml_tool_calls(text: &str) -> Vec<ParsedToolCall> {
    let mut calls = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("<tool_call>") {
        let after_tag = &rest[start + "<tool_call>".len()..];
        let Some(end) = after_tag.find("</tool_call>") else {
            break; // An unclosed block — unfinished stream; nothing is extracted
        };
        let body = after_tag[..end].trim();
        if let Some(call) = parse_tool_call_body(body) {
            calls.push(call);
        }
        rest = &after_tag[end + "</tool_call>".len()..];
    }
    calls
}

/// Reads one `tool_call` block's body: strict first, then lenient.
fn parse_tool_call_body(body: &str) -> Option<ParsedToolCall> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        return value_to_call(&v);
    }
    let v = lenient_json_object(body)?;
    value_to_call(&v)
}

/// Lenient fallback: consumes the trailing comma and converts single-quoted keys/strings
/// to double quotes — a light equivalent of `ast.literal_eval` without ever executing code.
fn lenient_json_object(body: &str) -> Option<serde_json::Value> {
    let trimmed = body.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    // Step 1: single-quoted key/string → double (correctly handling Python escapes)
    let chars: Vec<char> = trimmed.chars().collect();
    let mut converted = String::with_capacity(trimmed.len());
    let mut in_string: Option<char> = None;
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        match in_string {
            None => match c {
                '\'' | '"' => {
                    in_string = Some(c);
                    converted.push('"');
                }
                _ => converted.push(c),
            },
            Some(q) => {
                if c == '\\' && i + 1 < chars.len() {
                    let next = chars[i + 1];
                    match (q, next) {
                        // \' inside single quotes = the quote itself — unescaped in double-quoted JSON
                        ('\'', '\'') => converted.push('\''),
                        // \" inside single quotes = a double quote — needs escaping in JSON
                        ('\'', '"') => converted.push_str("\\\""),
                        // \\ in both = an escaped backslash
                        (_, '\\') => converted.push_str("\\\\"),
                        // The other escapes (\n, \t, …) are valid as-is in JSON
                        _ => {
                            converted.push('\\');
                            converted.push(next);
                        }
                    }
                    i += 1;
                } else if c == q {
                    in_string = None;
                    converted.push('"');
                } else {
                    converted.push(c);
                }
            }
        }
        i += 1;
    }
    // Step 2: a trailing comma before } or ] is removed (outside strings only)
    let chars: Vec<char> = converted.chars().collect();
    let mut cleaned = String::with_capacity(converted.len());
    let mut in_string = false;
    let mut j = 0usize;
    while j < chars.len() {
        let c = chars[j];
        if in_string {
            if c == '\\' && j + 1 < chars.len() {
                cleaned.push(c);
                cleaned.push(chars[j + 1]);
                j += 1;
            } else if c == '"' {
                in_string = false;
            }
            cleaned.push(c);
        } else if c == '"' {
            in_string = true;
            cleaned.push(c);
        } else if c == ',' {
            // Look at the next non-whitespace character — if it closes the block, eat the comma
            let mut k = j + 1;
            while k < chars.len() && chars[k].is_whitespace() {
                k += 1;
            }
            if !(k < chars.len() && (chars[k] == '}' || chars[k] == ']')) {
                cleaned.push(c);
            }
        } else {
            cleaned.push(c);
        }
        j += 1;
    }
    serde_json::from_str(&cleaned).ok()
}

/// A `Value` becomes a [`ParsedToolCall`] only when it has the `{"name": ..., "arguments": ...}`
/// shape; a JSON-encoded string `arguments` is unwrapped too (some models double-
/// encode).
fn value_to_call(v: &serde_json::Value) -> Option<ParsedToolCall> {
    let obj = v.as_object()?;
    let name = obj.get("name")?.as_str()?.to_owned();
    let arguments = match obj.get("arguments") {
        None => serde_json::Value::Object(serde_json::Map::new()),
        Some(serde_json::Value::String(s)) => {
            serde_json::from_str(s).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
        }
        Some(other) => other.clone(),
    };
    if name.is_empty() {
        return None;
    }
    Some(ParsedToolCall { name, arguments })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn delta(index: usize, id: &str, name: &str, args: &str) -> ToolCallDelta {
        ToolCallDelta {
            index: Some(index),
            id: (!id.is_empty()).then(|| id.to_owned()),
            name: (!name.is_empty()).then(|| name.to_owned()),
            arguments: (!args.is_empty()).then(|| args.to_owned()),
        }
    }

    /// The gateway's repeated name (MiniMax/NIM) is replaced by assignment, not concatenated.
    #[test]
    fn duplicate_name_overwrites_never_concatenates() {
        let mut acc = ToolCallStreamAccumulator::new();
        assert_eq!(
            acc.feed(&delta(0, "c1", "read_file", "")).as_deref(),
            Some("read_file")
        );
        // The notorious gateway: sends the full name in every delta
        acc.feed(&delta(0, "c1", "read_file", "{\"path\":"));
        acc.feed(&delta(0, "c1", "read_file", "\"a.rs\"}"));
        let calls = acc.materialize();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_file", "\u{647}\u{631}\u{6af}\u{632} read_fileread_file \u{646}\u{634}\u{648}\u{62f}");
        assert_eq!(calls[0].arguments, "{\"path\":\"a.rs\"}");
    }

    /// Ollama's consumed index (all on 0) routes to a fresh slot with a fresh id
    /// — when the previous call is structurally closed.
    #[test]
    fn reused_index_after_closed_object_opens_fresh_slot() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&delta(0, "a", "tool_one", "{\"x\":1}"));
        acc.feed(&delta(0, "b", "tool_one", "{\"y\":2}"));
        let calls = acc.materialize();
        assert_eq!(calls.len(), 2, "\u{62f}\u{648} \u{641}\u{631}\u{627}\u{62e}\u{648}\u{627}\u{646}\u{6cc} \u{645}\u{648}\u{627}\u{632}\u{6cc}\u{650} \u{6cc}\u{6a9} \u{627}\u{628}\u{632}\u{627}\u{631} \u{628}\u{627}\u{6cc}\u{62f} \u{62c}\u{62f}\u{627} \u{634}\u{648}\u{646}\u{62f}");
        assert_eq!(calls[0].id, "a");
        assert_eq!(calls[1].id, "b");
        assert_eq!(calls[0].arguments, "{\"x\":1}");
        assert_eq!(calls[1].arguments, "{\"y\":2}");
    }

    /// A fresh id with a repeated index while the arguments are still open is a continuation of
    /// the same call (LM Studio/Kimi send a fresh id on every chunk).
    #[test]
    fn fresh_id_on_open_object_continues_current_call() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&delta(0, "id-1", "write", "{\"content\":"));
        acc.feed(&delta(0, "id-2", "", "\"\u{633}\u{644}\u{627}\u{645}\"}"));
        let calls = acc.materialize();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "id-2", "\u{622}\u{62e}\u{631}\u{6cc}\u{646} \u{634}\u{646}\u{627}\u{633}\u{647} \u{628}\u{631}\u{646}\u{62f}\u{647} \u{627}\u{633}\u{62a}");
        assert_eq!(calls[0].arguments, "{\"content\":\"\u{633}\u{644}\u{627}\u{645}\"}");
    }

    /// Bracket depth inside strings does not count — a `}` inside a string value does not close the slot.
    #[test]
    fn braces_inside_strings_do_not_close_slot() {
        let mut acc = ToolCallStreamAccumulator::new();
        // First delta: the JSON's closing brace is "inside" the open string — the slot is still open
        acc.feed(&delta(0, "x", "echo", r#"{"text":"}"#));
        // Second delta with the same tool name: since the arguments are still open, it continues the same call
        acc.feed(&delta(0, "y", "echo", r#""}"#));
        let calls = acc.materialize();
        assert_eq!(calls.len(), 1, "\u{622}\u{6a9}\u{648}\u{644}\u{627}\u{62f}\u{650} \u{62f}\u{627}\u{62e}\u{644} \u{631}\u{634}\u{62a}\u{647} \u{646}\u{628}\u{627}\u{6cc}\u{62f} \u{641}\u{631}\u{627}\u{62e}\u{648}\u{627}\u{646}\u{6cc} \u{631}\u{627} \u{628}\u{628}\u{646}\u{62f}\u{62f}");
        assert_eq!(calls[0].arguments, r#"{"text":"}"}"#);
    }

    /// A trailing comma (recoverable JSON) also closes the slot.
    #[test]
    fn trailing_comma_still_closes_slot() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&delta(0, "a", "t", "{\"a\":1,}"));
        acc.feed(&delta(0, "b", "t", "{\"b\":2}"));
        assert_eq!(acc.slot_count(), 2);
    }

    /// A delta without an index = index 0, and a name-only stream delivers arguments chunk by chunk.
    #[test]
    fn unnamed_index_streams_argument_fragments() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&delta(0, "c", "search", "{\"q\": \""));
        acc.feed(&ToolCallDelta { index: None, id: None, name: None, arguments: Some("\u{6a9}\u{6cc}\u{627}\u{646}".into()) });
        acc.feed(&ToolCallDelta { index: None, id: None, name: None, arguments: Some("\"}".into()) });
        let calls = acc.materialize();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments, "{\"q\": \"\u{6a9}\u{6cc}\u{627}\u{646}\"}");
    }

    /// A numeric id (Poolside) comes through as a string.
    #[test]
    fn integer_id_becomes_string() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&ToolCallDelta {
            index: Some(0),
            id: Some("42".into()),
            name: Some("t".into()),
            arguments: None,
        });
        let calls = acc.materialize();
        assert_eq!(calls[0].id, "42");
    }

    /// `extract_xml_tool_calls` — the standard Hermes block with real arguments.
    #[test]
    fn extracts_standard_hermes_tool_call() {
        let text = "\u{62f}\u{631} \u{62d}\u{627}\u{644} \u{62c}\u{633}\u{62a}‌\u{648}\u{62c}\u{648}…\n<tool_call>\n{\"name\": \"web_search\", \"arguments\": {\"query\": \"KIAN\", \"limit\": 3}}\n</tool_call>\n\u{62e}\u{644}\u{627}\u{635}\u{647}\u{654} \u{646}\u{62a}\u{6cc}\u{62c}\u{647}‌\u{647}\u{627} \u{628}\u{639}\u{62f}\u{627}\u{64b}.";
        let calls = extract_xml_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "web_search");
        assert_eq!(calls[0].arguments, json!({"query": "KIAN", "limit": 3}));
    }

    /// Multiple parallel blocks in one reply — all are extracted in order.
    #[test]
    fn extracts_multiple_parallel_calls_in_order() {
        let text = "<tool_call>{\"name\": \"a\", \"arguments\": {\"i\": 1}}</tool_call>"
            .to_owned()
            + "\u{645}\u{6cc}\u{627}\u{646}‌\u{645}\u{62a}\u{646}\u{650} \u{62a}\u{62d}\u{644}\u{6cc}\u{644} \u{645}\u{62f}\u{644}"
            + "<tool_call>{\"name\": \"b\", \"arguments\": {\"i\": 2}}</tool_call>";
        let calls = extract_xml_tool_calls(&text);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "a");
        assert_eq!(calls[1].name, "b");
    }

    /// Lenient fallback: trailing comma + a single-quoted key.
    #[test]
    fn lenient_fallback_handles_trailing_comma_and_single_quotes() {
        let text = "<tool_call>{'name': 'wallet_balance', 'arguments': {'addr': 'x', }}</tool_call>";
        let calls = extract_xml_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "wallet_balance");
        assert_eq!(calls[0].arguments, json!({"addr": "x"}));
    }

    /// `arguments` as a JSON-encoded string (the model double-encoded) is unwrapped.
    #[test]
    fn string_encoded_arguments_are_decoded() {
        let text = "<tool_call>{\"name\": \"t\", \"arguments\": \"{\\\"k\\\": 7}\"}</tool_call>";
        let calls = extract_xml_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments, json!({"k": 7}));
    }

    /// A broken block is discarded, the next block stays healthy; an unclosed block yields nothing.
    #[test]
    fn broken_blocks_are_skipped_stream_safe() {
        let bad = "<tool_call>{\"name\": } NOPE</tool_call>";
        assert_eq!(extract_xml_tool_calls(bad), [] as [_; 0]);
        let mixed = format!("{bad}<tool_call>{{\"name\": \"ok\", \"arguments\": {{}}}}</tool_call>");
        let calls = extract_xml_tool_calls(&mixed);
        assert_eq!(calls.len(), 1, "\u{628}\u{644}\u{648}\u{6a9}\u{650} \u{62e}\u{631}\u{627}\u{628} \u{628}\u{644}\u{648}\u{6a9}\u{650} \u{633}\u{627}\u{644}\u{645} \u{631}\u{627} \u{646}\u{645}\u{6cc}‌\u{6a9}\u{634}\u{62f}");
        assert_eq!(calls[0].name, "ok");
        // Unfinished stream: the tag is not closed → none
        assert_eq!(extract_xml_tool_calls("{\"name\": \"x\""), [] as [_; 0]);
        // Plain text without tags → none
        assert_eq!(extract_xml_tool_calls("\u{67e}\u{627}\u{633}\u{62e} \u{633}\u{627}\u{62f}\u{647}\u{654} \u{628}\u{62f}\u{648}\u{646} \u{627}\u{628}\u{632}\u{627}\u{631}"), [] as [_; 0]);
    }
}
