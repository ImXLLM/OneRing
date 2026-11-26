//! `compressor` — two-phase context compressor (port of Hermes).
//!
//! Port from the `NousResearch` treasury (Wave 2 - Step 2):
//! - `agent/context_compressor_prune.py` → phase 1, deterministic tool-output pruning
//!   ([`prune_old_tool_outputs`]): bulky outputs of old turns collapse into a
//!   one-line stub (`[tool:{name}] ran -> exit {code}, {lines} lines
//!   ({bytes}B) | last: {last_non_empty_line}`) — but **never** the active/latest turn
//!   (`keep_recent_turns >= 1`, port of `_keep_running_turn`) and **never** an output carrying
//!   a secret (Kian secret guard: [`kian_ai::redaction::redact`]) is touched.
//!   Cold-cache gate: [`should_trigger_cold_prune`] with a floor of `16_000` tokens —
//!   breaking the provider's cache for a trivial saving is never justified.
//! - `agent/micro_compaction.py` → phase 2 ([`partition_for_micro_compaction`]):
//!   micro-compaction folding boundaries — only complete `(assistant + tool*)`
//!   exchange blocks from old turns qualify for summarization, and
//!   **all `user` messages stay 100% untouched in place**.
//!
//! All functions are deterministic; the input is the lightweight [`CompressorMessage`] structure, not the
//! provider model — this crate knows nothing about the database.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Token floor of savings for pruning against the provider's cold cache
/// (`_COLD_CACHE_MIN_RECLAIM_TOKENS = 16_000` in the source).
pub const COLD_CACHE_MIN_RECLAIM_TOKENS: usize = 16_000;

/// Character floor for a tool output to count as "bulky" and be prunable.
pub const MIN_PRUNE_CHARS: usize = 400;

/// Length cap for the "last output" line inside the stub.
const LAST_LINE_ROOM: usize = 120;

/// A lightweight, serializable message from the history — chat-completion roles:
/// `system` / `user` / `assistant` / `tool`. A tool output arrives in the `content` of the
/// `tool` role, and `tool_name` keeps the name of the tool that produced it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressorMessage {
    pub role: String,
    pub content: String,
    /// Tool name for the `tool` role (for the stub and telemetry); empty string = none.
    #[serde(default)]
    pub tool_name: String,
}

impl CompressorMessage {
    #[must_use]
    pub fn new(role: &str, content: &str) -> Self {
        Self { role: role.to_owned(), content: content.to_owned(), tool_name: String::new() }
    }

    #[must_use]
    pub fn tool(role: &str, tool_name: &str, content: impl Into<String>) -> Self {
        Self { role: role.to_owned(), content: content.into(), tool_name: tool_name.to_owned() }
    }

    fn is_tool(&self) -> bool {
        self.role == "tool"
    }
}

/// Deterministic prune result: a fresh history + a few shortened messages + a few bytes saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneOutcome {
    pub messages: Vec<CompressorMessage>,
    pub pruned_count: usize,
    pub saved_bytes: usize,
}

/// Cold-cache prune gate: only when the estimated saving passes the 16,000-token floor
/// is it worth breaking the already-expired cache (port of `_COLD_CACHE_MIN_RECLAIM_TOKENS`).
#[must_use]
pub fn should_trigger_cold_prune(estimated_saved_tokens: usize) -> bool {
    estimated_saved_tokens >= COLD_CACHE_MIN_RECLAIM_TOKENS
}

/// Coarse token estimate — roughly 4 characters per token (language-neutral; for Persian/RTL
/// this stingy estimate, not a smaller one, is chosen).
#[must_use]
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// Does the tool output carry a secret? — the Kian secret guard decides deterministically: a hit from
/// the [`crate::redact`] scanner or explicit API key prefixes (`sk-`, `ghp_`,
/// `xox`, `Bearer `, JWT `eyJ…`) means "this output is never touched,
/// it stays byte-for-byte" — defense in depth on top of the base scanner.
fn contains_secret(text: &str) -> bool {
    if !crate::redact(text).findings.is_empty() {
        return true;
    }
    for marker in ["sk-", "ghp_", "gho_", "xoxb-", "Bearer ", "AKIA"] {
        if text.contains(marker) {
            return true;
        }
    }
    // Three-part JWT: eyJ…eyJ…eyJ (no regex required)
    text.contains("eyJ")
}

/// Reads the exit code from the result's JSON envelope — port of the regex
/// `"exit_code"\s*:\s*(-?\d+)` with tolerance for escaped quotes in nested
/// envelopes (`\"exit_code\":`). No envelope = `0` (success).
fn extract_exit_code(content: &str) -> String {
    let mut search_from = 0usize;
    while let Some(rel) = content[search_from..].find("exit_code") {
        let i = search_from + rel;
        let after = content[i + "exit_code".len()..].trim_start();
        // The key's closing quote (with or without a backslash) and whitespace are consumed
        let after = after.trim_start_matches(['\\', '"', '\'']).trim_start();
        let Some(after) = after.strip_prefix(':') else {
            search_from = i + "exit_code".len();
            continue;
        };
        let after = after.trim_start();
        let digits: String =
            after.chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
        if digits.is_empty() || digits == "-" {
            search_from = i + "exit_code".len();
            continue;
        }
        return digits;
    }
    "0".to_owned()
}

/// One-line stub for pruned output — port of `terminal_stub` with the treasury's generic
/// format: `[tool:{name}] ran -> exit {code}, {lines} lines ({bytes}B) | last: …`
/// The stub always stays under `MIN_PRUNE_CHARS` so the next pass does not
/// classify it again as "a result to summarize".
fn tool_stub(tool_name: &str, content: &str) -> String {
    let bytes = content.len();
    let lines = content.lines().count();
    let exit_code = extract_exit_code(content);
    let last = content
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .chars()
        .take(LAST_LINE_ROOM)
        .collect::<String>();
    let mut stub = format!(
        "[tool:{tool_name}] ran -> exit {exit_code}, {lines} lines ({bytes}B) | last: {last}"
    );
    if stub.len() > MIN_PRUNE_CHARS {
        stub = stub.chars().take(MIN_PRUNE_CHARS).collect();
    }
    stub
}

/// Protection boundary: the last `keep_recent_turns` complete turns (trailing `user … tool*`)
/// are never pruned — port of `_keep_running_turn` with extra strictness: `keep_recent_turns`
/// below 1 is meaningless and fail-closes to 1.
fn protected_boundary(messages: &[CompressorMessage], keep_recent_turns: usize) -> usize {
    let keep = keep_recent_turns.max(1);
    // Walking backward from the end, we find where the kept turns begin: the start
    // boundary of each turn is its `user` row.
    let mut turns_seen = 0usize;
    let mut boundary = 0usize;
    for (i, msg) in messages.iter().enumerate().rev() {
        if msg.role == "user" {
            turns_seen += 1;
            if turns_seen == keep {
                boundary = i;
                break;
            }
        }
    }
    if turns_seen < keep {
        // The whole history is shorter than the protection — nothing is prunable
        boundary = 0;
    }
    boundary
}

/// Phase 1 — deterministic, LLM-free pruning of old tool outputs.
///
/// Rules (port of `prune_tool_results_only`):
/// 1. The protection boundary is not "message counting" but "the start of a `user` turn", so it never cuts
///    into the middle of an assistant step's parallel results.
/// 2. Only bulky `tool` outputs (≥ [`MIN_PRUNE_CHARS`]) from **old** turns
///    become stubs; secret-bearing output (the [`kian_ai::redact`] guard) is always untouched.
/// 3. If nothing was shortened, the input is returned verbatim (no-op contract).
#[must_use]
pub fn prune_old_tool_outputs(
    messages: &[CompressorMessage],
    keep_recent_turns: usize,
) -> PruneOutcome {
    let boundary = protected_boundary(messages, keep_recent_turns);
    let mut out: Vec<CompressorMessage> = Vec::with_capacity(messages.len());
    let mut pruned_count = 0usize;
    let mut saved_bytes = 0usize;
    for (i, msg) in messages.iter().enumerate() {
        let eligible = i < boundary
            && msg.is_tool()
            && msg.content.chars().count() >= MIN_PRUNE_CHARS
            && !contains_secret(&msg.content);
        if eligible {
            let stub = tool_stub(&msg.tool_name, &msg.content);
            saved_bytes += msg.content.len().saturating_sub(stub.len());
            pruned_count += 1;
            out.push(CompressorMessage::tool("tool", &msg.tool_name, &stub));
        } else {
            out.push(msg.clone());
        }
    }
    if pruned_count == 0 {
        // no-op contract: the input verbatim — the caller gates on "no change"
        return PruneOutcome { messages: messages.to_vec(), pruned_count: 0, saved_bytes: 0 };
    }
    PruneOutcome { messages: out, pruned_count, saved_bytes }
}

/// One exchange block eligible for summarization — from an `assistant` (with no pending tool call)
/// up to just before the next `user`; contains only `assistant` and `tool` messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeBlock {
    /// The `[start, end)` range over the input history.
    pub start: usize,
    pub end: usize,
    pub tokens: usize,
}

/// Phase 2 partition output — which ranges qualify for summarization and what is preserved.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MicroCompactionPlan {
    /// Old complete-exchange ranges — never include a `user`.
    pub foldable: Vec<ExchangeBlock>,
    /// Preserved indices: all `user`s + the system head + the active turn.
    pub keep: Vec<usize>,
    /// Total summarizable indices (union of the foldable) — for telemetry.
    pub foldable_indices: Vec<usize>,
}

/// Phase 2 — partition the history for exchange-oriented micro-summarization
/// (port of `micro_compaction.py`).
///
/// Hard rules:
/// 1. **All `user` messages stay 100% untouched in place** — the summary
///    never swallows a single word of the human's real request.
/// 2. Only **complete** `(assistant + tool*)` exchange blocks from old turns
///    (before the active turn's protection boundary) qualify: the block opens with an `assistant`,
///    closes with the next `user` (or the end); the `system` head of the history
///    is never folded.
/// 3. An incomplete block (an unclosed `assistant`, or an orphan `tool` row) fail-closes
///    to preserved, not summarized.
#[must_use]
pub fn partition_for_micro_compaction(
    messages: &[CompressorMessage],
    keep_recent_turns: usize,
) -> MicroCompactionPlan {
    let boundary = protected_boundary(messages, keep_recent_turns);
    let mut plan = MicroCompactionPlan::default();
    // Manual loop: after a foldable block we jump to its end so the tool rows
    // inside the block are not mistakenly counted as "preserved" again.
    let mut i = 0usize;
    while i < messages.len() {
        if messages[i].role == "assistant" {
            // Close the block's range: from here to the first following user/system
            let mut end = i + 1;
            while end < boundary
                && matches!(messages[end].role.as_str(), "assistant" | "tool")
            {
                end += 1;
            }
            let complete =
                end < messages.len() && messages[end].role == "user" && end <= boundary;
            if complete {
                let tokens: usize = messages[i..end]
                    .iter()
                    .map(|m| estimate_tokens(&m.content))
                    .sum();
                plan.foldable.push(ExchangeBlock { start: i, end, tokens });
                plan.foldable_indices.extend(i..end);
                i = end; // the closing user becomes preserved on the next iteration
            } else {
                plan.keep.push(i);
                i += 1;
            }
        } else {
            // system / user / orphan tool row (the previous assistant was not foldable) and any
            // unknown role — all preserved fail-closed; in particular all users stay 100%
            // untouched in place
            plan.keep.push(i);
            i += 1;
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history() -> Vec<CompressorMessage> {
        vec![
            CompressorMessage::new("system", "\u{62a}\u{648} \u{647}\u{631}\u{645}\u{633} \u{6a9}\u{6cc}\u{627}\u{646} \u{647}\u{633}\u{62a}\u{6cc}."),
            CompressorMessage::new("user", "\u{633}\u{627}\u{62e}\u{62a}\u{627}\u{631} \u{67e}\u{631}\u{648}\u{698}\u{647} \u{631}\u{627} \u{628}\u{631}\u{631}\u{633}\u{6cc} \u{6a9}\u{646}"),
            CompressorMessage::new("assistant", "\u{62f}\u{627}\u{631}\u{645} \u{633}\u{627}\u{62e}\u{62a}\u{627}\u{631} \u{631}\u{627} \u{645}\u{6cc}‌\u{62e}\u{648}\u{627}\u{646}\u{645}…"),
            CompressorMessage::tool("tool", "shell", "exit 0\n".repeat(200)), // bulky
            CompressorMessage::tool("tool", "fs", "exit 0\n".repeat(150)),       // bulky
            CompressorMessage::new("user", "\u{62d}\u{627}\u{644}\u{627} \u{62a}\u{633}\u{62a}‌\u{647}\u{627} \u{631}\u{627} \u{628}\u{632}\u{646}"),
            CompressorMessage::new("assistant", "\u{62f}\u{631} \u{62d}\u{627}\u{644} \u{627}\u{62c}\u{631}\u{627}\u{6cc} \u{62a}\u{633}\u{62a}‌\u{647}\u{627}…"),
            CompressorMessage::tool("tool", "shell", "test ok\n".repeat(120)), // the active turn
        ]
    }

    /// The active/latest turn is never pruned — a port of `_keep_running_turn`.
    #[test]
    fn active_turn_is_never_pruned() {
        let out = prune_old_tool_outputs(&history(), 1);
        let active = &out.messages[out.messages.len() - 1];
        assert_eq!(active.content, "test ok\n".repeat(120), "\u{62e}\u{631}\u{648}\u{62c}\u{6cc} \u{646}\u{648}\u{628}\u{62a}\u{650} \u{641}\u{639}\u{627}\u{644} \u{62f}\u{633}\u{62a}‌\u{646}\u{62e}\u{648}\u{631}\u{62f}\u{647}");
        assert!(out.pruned_count >= 2, "\u{62e}\u{631}\u{648}\u{62c}\u{6cc}‌\u{647}\u{627}\u{6cc} \u{642}\u{62f}\u{6cc}\u{645}\u{6cc} \u{628}\u{627}\u{6cc}\u{62f} \u{647}\u{631}\u{633} \u{634}\u{648}\u{646}\u{62f}");
    }

    /// A one-line stub is built with the treasury format and saves bytes.
    #[test]
    fn stub_format_carries_exit_lines_bytes_last() {
        let content = "step a\nstep b\n\"exit_code\": 3\nlast non-empty line";
        let stub = tool_stub("shell", content);
        assert!(stub.starts_with("[tool:shell] ran -> exit 3, "), "\u{627}\u{633}\u{62a}\u{627}\u{628}: {stub}");
        assert!(stub.contains("4 lines"), "\u{62a}\u{639}\u{62f}\u{627}\u{62f} \u{62e}\u{637}: {stub}");
        assert!(stub.contains(&format!("({}B)", content.len())), "\u{628}\u{627}\u{6cc}\u{62a}: {stub}");
        assert!(stub.contains("last: last non-empty line"), "\u{622}\u{62e}\u{631}\u{6cc}\u{646} \u{62e}\u{637}: {stub}");
        assert!(stub.chars().count() < MIN_PRUNE_CHARS, "\u{627}\u{633}\u{62a}\u{627}\u{628} \u{6a9}\u{648}\u{62a}\u{627}\u{647} \u{645}\u{6cc}‌\u{645}\u{627}\u{646}\u{62f}");
        // The JSON-escaped envelope reads the exit code too
        let escaped = r#"{"output": "{\"exit_code\": 42, \"out\": 1}"}"#;
        assert_eq!(extract_exit_code(escaped), "42");
        // No envelope = success (0)
        assert_eq!(extract_exit_code("plain output"), "0");
        assert_eq!(extract_exit_code("\"exit_code\": -1"), "-1");
    }

    /// Secret-bearing output is never pruned — the Kian secret guard takes precedence over compression.
    #[test]
    fn secret_bearing_output_is_never_pruned() {
        let secret_out = format!("{}\n", "sk-or-v1-PLACEHOLDER-DUMMY-TEST-FIXTURE-not-real-aaaaaaaa".repeat(30));
        let msgs = vec![
            CompressorMessage::new("user", "\u{642}\u{62f}\u{645} \u{627}\u{648}\u{644}"),
            CompressorMessage::new("assistant", "\u{627}\u{62c}\u{631}\u{627} \u{634}\u{62f}"),
            CompressorMessage::tool("tool", "shell", secret_out.clone()),
            CompressorMessage::new("user", "\u{642}\u{62f}\u{645} \u{62f}\u{648}\u{645}"),
        ];
        let out = prune_old_tool_outputs(&msgs, 1);
        assert_eq!(out.pruned_count, 0, "\u{631}\u{627}\u{632} = \u{62f}\u{633}\u{62a}‌\u{646}\u{62e}\u{648}\u{631}\u{62f}\u{647}");
        assert_eq!(out.messages[2].content, secret_out, "\u{628}\u{627}\u{6cc}\u{62a}‌\u{628}\u{647}‌\u{628}\u{627}\u{6cc}\u{62a}");
    }

    /// Short output is never pruned (the Bulky floor) — and a no-op returns the input exactly.
    #[test]
    fn small_outputs_and_noop_return_input_unchanged() {
        let msgs = vec![
            CompressorMessage::new("user", "\u{633}\u{644}\u{627}\u{645}"),
            CompressorMessage::new("assistant", "\u{62f}\u{631}\u{648}\u{62f}"),
            CompressorMessage::tool("tool", "tiny", "ok"),
            CompressorMessage::new("user", "\u{627}\u{62f}\u{627}\u{645}\u{647}"),
        ];
        let out = prune_old_tool_outputs(&msgs, 1);
        assert_eq!(out.messages, msgs, "no-op \u{628}\u{627}\u{6cc}\u{62f} \u{639}\u{6cc}\u{646}\u{627}\u{64b} \u{648}\u{631}\u{648}\u{62f}\u{6cc} \u{628}\u{627}\u{634}\u{62f}");
        assert_eq!(out.pruned_count, 0);
        assert_eq!(out.saved_bytes, 0);
    }

    /// Cold-cache gate: the 16,000-token floor is hard.
    #[test]
    fn cold_prune_gate_enforces_16k_floor() {
        assert!(!should_trigger_cold_prune(15_999));
        assert!(should_trigger_cold_prune(16_000));
        assert!(should_trigger_cold_prune(50_000));
        assert_eq!(COLD_CACHE_MIN_RECLAIM_TOKENS, 16_000);
    }

    /// Coarse token estimate — 4 characters per token, zero = zero.
    #[test]
    fn token_estimation_is_rough_and_positive() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
        assert_eq!(estimate_tokens("abc"), 1);
    }

    /// Phase 2 partition: only old complete exchanges are foldable and all users stay untouched.
    #[test]
    fn micro_compaction_partitions_exchanges_and_keeps_users() {
        let msgs = history();
        let plan = partition_for_micro_compaction(&msgs, 1);
        // All users + system are preserved
        for (i, msg) in msgs.iter().enumerate() {
            if matches!(msg.role.as_str(), "user" | "system") {
                assert!(plan.keep.contains(&i), "\u{646}\u{642}\u{634} {} \u{62f}\u{631} \u{634}\u{627}\u{62e}\u{635} {i} \u{628}\u{627}\u{6cc}\u{62f} \u{645}\u{62d}\u{641}\u{648}\u{638} \u{628}\u{627}\u{634}\u{62f}", msg.role);
            }
        }
        // No foldable range includes a user and no row is counted twice
        for block in &plan.foldable {
            for (offset, msg) in msgs[block.start..block.end].iter().enumerate() {
                let i = block.start + offset;
                assert_ne!(msg.role, "user", "user \u{647}\u{631}\u{6af}\u{632} \u{62f}\u{631} \u{628}\u{644}\u{648}\u{6a9} \u{62c}\u{645}\u{639}‌\u{628}\u{646}\u{62f}\u{6cc} \u{646}\u{6cc}\u{633}\u{62a}");
                assert!(!plan.keep.contains(&i), "\u{631}\u{62f}\u{6cc}\u{641}\u{650} \u{62c}\u{645}\u{639}‌\u{634}\u{62f}\u{646}\u{6cc} \u{646}\u{628}\u{627}\u{6cc}\u{62f} \u{62f}\u{631} keep \u{647}\u{645} \u{628}\u{627}\u{634}\u{62f}: {i}");
            }
        }
        assert_eq!(
            plan.foldable_indices.len(),
            plan.foldable.iter().map(|b| b.end - b.start).sum::<usize>(),
            "\u{634}\u{627}\u{62e}\u{635}‌\u{647}\u{627}\u{6cc} \u{62a}\u{62c}\u{645}\u{6cc}\u{639}\u{6cc} \u{628}\u{627} \u{62c}\u{645}\u{639} \u{628}\u{644}\u{648}\u{6a9}‌\u{647}\u{627} \u{628}\u{631}\u{627}\u{628}\u{631} \u{627}\u{633}\u{62a}"
        );
        // The old exchange (assistant + 2 tool) is foldable
        assert_eq!(plan.foldable.len(), 1, "\u{641}\u{642}\u{637} \u{628}\u{644}\u{648}\u{6a9} \u{642}\u{62f}\u{6cc}\u{645}\u{6cc} \u{6a9}\u{627}\u{645}\u{644}");
        assert_eq!(plan.foldable[0].start, 2);
        assert_eq!(plan.foldable[0].end, 5);
        assert!(plan.foldable[0].tokens > 0);
        // Active turn rows (6..8) are reserved, fail-closed
        for i in 6..8 {
            assert!(plan.keep.contains(&i), "\u{646}\u{648}\u{628}\u{62a}\u{650} \u{641}\u{639}\u{627}\u{644} \u{634}\u{627}\u{62e}\u{635} {i} \u{645}\u{62d}\u{641}\u{648}\u{638}");
        }
    }

    /// An incomplete block (no next user = turn in flight) is not foldable.
    #[test]
    fn incomplete_exchange_is_kept_not_folded() {
        let msgs = vec![
            CompressorMessage::new("system", "\u{633}\u{631}"),
            CompressorMessage::new("user", "\u{633}\u{648}\u{627}\u{644} \u{642}\u{62f}\u{6cc}\u{645}\u{6cc}"),
            CompressorMessage::new("assistant", "\u{62f}\u{627}\u{631}\u{645} \u{6a9}\u{627}\u{631} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{645}…"),
            CompressorMessage::tool("tool", "shell", "\u{62e}\u{631}\u{648}\u{62c}\u{6cc}"),
        ];
        let plan = partition_for_micro_compaction(&msgs, 1);
        assert_eq!(plan.foldable, [] as [_; 0], "\u{628}\u{644}\u{648}\u{6a9}\u{650} \u{628}\u{62f}\u{648}\u{646} \u{628}\u{633}\u{62a}\u{646}\u{650} user \u{62c}\u{645}\u{639} \u{646}\u{645}\u{6cc}‌\u{634}\u{648}\u{62f}");
        assert!(plan.keep.contains(&2) && plan.keep.contains(&3));
    }

    /// History shorter than the protection — nothing is foldable (fail-closed).
    #[test]
    fn tiny_history_is_fully_protected() {
        let msgs = vec![
            CompressorMessage::new("user", "\u{62a}\u{646}\u{647}\u{627} \u{646}\u{648}\u{628}\u{62a}"),
            CompressorMessage::new("assistant", "\u{67e}\u{627}\u{633}\u{62e}"),
        ];
        let plan = partition_for_micro_compaction(&msgs, 1);
        assert_eq!(plan.foldable, [] as [_; 0]);
        let out = prune_old_tool_outputs(&msgs, 1);
        assert_eq!(out.pruned_count, 0);
    }

    /// Round-trip serialization of `CompressorMessage` — stable for stored history.
    #[test]
    fn compressor_message_roundtrips_serde() {
        let msg = CompressorMessage::tool("tool", "shell", "{\"exit_code\": 0}");
        let json = serde_json::to_string(&msg).unwrap();
        let back: CompressorMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
        let without_name: CompressorMessage =
            serde_json::from_str("{\"role\":\"user\",\"content\":\"\u{633}\u{644}\u{627}\u{645}\"}").unwrap();
        assert_eq!(without_name.tool_name, "");
    }
}
