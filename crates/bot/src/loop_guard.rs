//! `loop_guard` — the bot-to-bot loop guard (a port of `gateway/bot_loop_guard.py` from the
//! `NousResearch` — Wave 2 - Step 2).
//!
//! Two Hermes profiles answering each other never stop on their own;
//! `{PLATFORM}_ALLOW_BOTS` only lets them in. This guard tracks the pace of
//! bot-written messages per conversation and, with three independent checks, mutes
//! the endless reply storm:
//! 1. **Consecutive bot hops** — since the last human message, more than
//!    [`MAX_CONSECUTIVE_BOT_TURNS`] consecutive bot messages in one conversation means a loop;
//!    a human message resets the counter (port of `chain_admit`).
//! 2. **Duplicate load** — the same text hash from the same sender within the sliding
//!    ([`DEFAULT_WINDOW_SECS`] seconds) = a repeated ping; rejected (port of `duplicate`).
//! 3. **Ping-pong** — identical text bouncing between two bot senders
//!    (A→B→A with the same hash) = rejected (port of `reply_to_reply`/`max_hops`).
//!
//! The guard is fully deterministic: time is injected from outside and tests need no sleep/chance.
//! A human message is never suppressed (fail-open for humans).
#![forbid(unsafe_code)]

/// Default cap on consecutive bot messages after the last human message (port of `max_hops`;
/// tighter for Kian: 3).
pub const MAX_CONSECUTIVE_BOT_TURNS: u32 = 3;

/// Default sliding window in seconds — hashes and pace are valid inside this window.
pub const DEFAULT_WINDOW_SECS: u64 = 60;

/// Cap on messages from one `(chat, sender)` in the window — a pure flood is suppressed too.
pub const MAX_EVENTS_PER_WINDOW: usize = 5;

/// The suppression reason — for logs and telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopReason {
    /// Over the consecutive-bot-message cap since the last human message.
    MaxConsecutiveBotTurns,
    /// The same text from the same sender within the sliding window.
    DuplicatePayload,
    /// Identical text bouncing between two bot senders (A→B→A).
    PingPong,
    /// A flood from one sender within the window.
    RateLimit,
}

/// The guard's decision for an inbound message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopGuardDecision {
    Allow,
    Suppress(LoopReason),
}

/// One inbound event in the conversation history.
#[derive(Debug, Clone, PartialEq, Eq)]
struct GuardEvent {
    sender: String,
    hash: u64,
    at: u64,
}

/// The bot loop guard — per-conversation state: the consecutive-bot-hop counter, the
///bounded event history, and the pace counter per `(chat, sender)`.
#[derive(Debug, Default)]
pub struct BotLoopGuard {
    window_secs: u64,
    max_consecutive_bot_turns: u32,
    /// Consecutive-bot-message counter per conversation — a human message zeroes it.
    bot_turns: std::collections::HashMap<String, u32>,
    /// The last admitted bot event per conversation — drives silent-chain expiry
    /// (port of the treasury's «chain updated»: a silent chain past the window dies).
    last_bot_at: std::collections::HashMap<String, u64>,
    /// Bounded history per conversation — the last `HISTORY_CAP` events.
    history: std::collections::HashMap<String, Vec<GuardEvent>>,
    /// Timestamps of each `(chat, sender)`'s events for pace gauging.
    velocity: std::collections::HashMap<(String, String), Vec<u64>>,
}

/// History cap per conversation — bounded memory; the sliding window never reaches it.
const HISTORY_CAP: usize = 32;

impl BotLoopGuard {
    /// A guard with the default window and cap.
    #[must_use]
    pub fn new() -> Self {
        Self::with_limits(DEFAULT_WINDOW_SECS, MAX_CONSECUTIVE_BOT_TURNS)
    }

    /// A guard with explicit bounds — for tests and per-platform tuning.
    #[must_use]
    pub fn with_limits(window_secs: u64, max_consecutive_bot_turns: u32) -> Self {
        Self {
            window_secs: window_secs.max(1),
            max_consecutive_bot_turns: max_consecutive_bot_turns.max(1),
            bot_turns: std::collections::HashMap::new(),
            last_bot_at: std::collections::HashMap::new(),
            history: std::collections::HashMap::new(),
            velocity: std::collections::HashMap::new(),
        }
    }

    /// Weighs one inbound message and decides.
    ///
    /// Check order (port of the `chain_admit` order): human → zero the hops; bot →
    /// flood, consecutive hops, duplicates, ping-pong. An admitted event is recorded; a
    /// suppressed one is not (the muted message never reaches its destination and must not
    /// count toward history or pace).
    pub fn observe(
        &mut self,
        chat_id: &str,
        sender_id: &str,
        is_bot: bool,
        text: &str,
        now_secs: u64,
    ) -> LoopGuardDecision {
        if !is_bot {
            // human — fail-open: the bot chain ends and the counter zeroes (port of `human`)
            self.bot_turns.insert(chat_id.to_owned(), 0);
            return LoopGuardDecision::Allow;
        }

        // ── sliding-window pruning (deterministic with injected time) ──
        self.sweep(chat_id, sender_id, now_secs);

        // ── check 1: flood from one sender within the window ──
        let vel_key = (chat_id.to_owned(), sender_id.to_owned());
        if self
            .velocity
            .get(&vel_key)
            .is_some_and(|stamps| stamps.len() >= MAX_EVENTS_PER_WINDOW)
        {
            return LoopGuardDecision::Suppress(LoopReason::RateLimit);
        }

        // ── silent-chain expiry — a gap beyond the window = a new chain
        // (port of the treasury's «now - chain.updated > cooldown_seconds»)
        let stale = self
            .last_bot_at
            .get(chat_id)
            .is_none_or(|&t| now_secs.saturating_sub(t) > self.window_secs);
        if stale {
            self.bot_turns.insert(chat_id.to_owned(), 0);
        }

        // ── check 2: consecutive bot hops since the last human message ──
        let turns = self.bot_turns.entry(chat_id.to_owned()).or_insert(0);
        if *turns >= self.max_consecutive_bot_turns {
            return LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns);
        }

        let hash = payload_hash(text);
        let history = self.history.entry(chat_id.to_owned()).or_default();

        // ── check 3: ping-pong — the same hash bouncing between two bot senders (A→B→A);
        // the exact signature of a bot-to-bot storm, ahead of plain duplication ──
        if history.len() >= 2 {
            let last = &history[history.len() - 1];
            let prev = &history[history.len() - 2];
            if last.sender != sender_id
                && last.hash == hash
                && prev.sender == sender_id
                && prev.hash == hash
            {
                return LoopGuardDecision::Suppress(LoopReason::PingPong);
            }
        }

        // ── check 4: duplicate load from the same sender within the window ──
        if history.iter().any(|e| e.sender == sender_id && e.hash == hash) {
            return LoopGuardDecision::Suppress(LoopReason::DuplicatePayload);
        }

        // ── admitted — record ──
        *turns += 1;
        self.last_bot_at.insert(chat_id.to_owned(), now_secs);
        if history.len() >= HISTORY_CAP {
            history.remove(0);
        }
        history.push(GuardEvent { sender: sender_id.to_owned(), hash, at: now_secs });
        self.velocity
            .entry(vel_key)
            .or_default()
            .push(now_secs);
        LoopGuardDecision::Allow
    }

    /// Prunes events older than the window from pace and history (deterministic).
    fn sweep(&mut self, chat_id: &str, sender_id: &str, now_secs: u64) {
        let cutoff = now_secs.saturating_sub(self.window_secs);
        let vel_key = (chat_id.to_owned(), sender_id.to_owned());
        if let Some(stamps) = self.velocity.get_mut(&vel_key) {
            stamps.retain(|&t| t > cutoff);
            if stamps.is_empty() {
                self.velocity.remove(&vel_key);
            }
        }
        if let Some(hist) = self.history.get_mut(chat_id) {
            hist.retain(|e| e.at > cutoff);
        }
    }

    /// The conversation's current consecutive-hop count (telemetry and tests).
    #[must_use]
    pub fn consecutive_bot_turns(&self, chat_id: &str) -> u32 {
        self.bot_turns.get(chat_id).copied().unwrap_or(0)
    }
}

/// Stable text hash — whitespace collapsing (port of `sha256(" ".join(text.split()))`
/// from the treasury, with a lighter output): multiple spaces/newlines = one space; empty = 0.
fn payload_hash(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a offset basis
    let mut last_ws = true;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if last_ws {
                continue; // consecutive spaces = one
            }
            last_ws = true;
            hash = mix(hash, b' ');
        } else {
            last_ws = false;
            let mut buf = [0u8; 4];
            let enc = ch.encode_utf8(&mut buf);
            for b in enc.bytes() {
                hash = mix(hash, b);
            }
        }
    }
    hash
}

/// Mixes one byte into the FNV-1a hash.
fn mix(mut hash: u64, byte: u8) -> u64 {
    hash ^= u64::from(byte);
    hash.wrapping_mul(0x0000_0100_0000_01b3)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fourth consecutive bot message is suppressed; a human message zeroes the chain.
    #[test]
    fn consecutive_bot_turns_are_capped_and_human_resets() {
        let mut g = BotLoopGuard::new();
        assert_eq!(g.observe("c", "bot-a", true, "\u{67e}\u{6cc}\u{627}\u{645} \u{6cc}\u{6a9}", 1), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "bot-a", true, "\u{67e}\u{6cc}\u{627}\u{645} \u{62f}\u{648}", 2), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "bot-a", true, "\u{67e}\u{6cc}\u{627}\u{645} \u{633}\u{647}", 3), LoopGuardDecision::Allow);
        assert_eq!(
            g.observe("c", "bot-a", true, "\u{67e}\u{6cc}\u{627}\u{645} \u{686}\u{647}\u{627}\u{631}", 4),
            LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns)
        );
        // a human breaks the chain — three hops allowed again
        assert_eq!(g.observe("c", "human-1", false, "\u{62e}\u{648}\u{62f}\u{62a} \u{62c}\u{648}\u{627}\u{628} \u{628}\u{62f}\u{647}", 5), LoopGuardDecision::Allow);
        assert_eq!(g.consecutive_bot_turns("c"), 0);
        assert_eq!(g.observe("c", "bot-b", true, "\u{633}\u{648}\u{627}\u{644} \u{627}\u{648}\u{644}", 6), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "bot-b", true, "\u{633}\u{648}\u{627}\u{644} \u{62f}\u{648}\u{645}", 7), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "bot-b", true, "\u{633}\u{648}\u{627}\u{644} \u{633}\u{648}\u{645}", 8), LoopGuardDecision::Allow);
        assert_eq!(
            g.observe("c", "bot-b", true, "\u{633}\u{648}\u{627}\u{644} \u{686}\u{647}\u{627}\u{631}\u{645}", 9),
            LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns)
        );
    }

    /// Same text from the same sender within the window = duplicate load; allowed again after the window.
    #[test]
    fn duplicate_payload_is_suppressed_within_window_then_allowed() {
        let mut g = BotLoopGuard::new();
        assert_eq!(g.observe("c", "bot", true, "\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{686}\u{637}\u{648}\u{631}\u{647}\u{61f}", 10), LoopGuardDecision::Allow);
        // whitespace is normalized — the same text with different spacing is a duplicate
        assert_eq!(
            g.observe("c", "bot", true, "\u{648}\u{636}\u{639}\u{6cc}\u{62a}   \u{686}\u{637}\u{648}\u{631}\u{647}\u{61f}", 20),
            LoopGuardDecision::Suppress(LoopReason::DuplicatePayload)
        );
        assert_eq!(
            g.observe("c", "bot", true, "\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{686}\u{637}\u{648}\u{631}\u{647}\u{61f}", 30),
            LoopGuardDecision::Suppress(LoopReason::DuplicatePayload)
        );
        // the 60-second window has passed — the same text is allowed again (a new round)
        assert_eq!(g.observe("c", "bot", true, "\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{686}\u{637}\u{648}\u{631}\u{647}\u{61f}", 10 + 61 + 30), LoopGuardDecision::Allow);
    }

    /// A→B→A ping-pong with identical text is suppressed — the essence of the bot-to-bot storm.
    #[test]
    fn ping_pong_between_two_bots_is_suppressed() {
        let mut g = BotLoopGuard::new();
        let msg = "\u{633}\u{644}\u{627}\u{645}\u{60c} \u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{686}\u{637}\u{648}\u{631}\u{647}\u{61f}";
        assert_eq!(g.observe("c", "a", true, msg, 1), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "b", true, msg, 2), LoopGuardDecision::Allow);
        // A repeats the same text — the bounce completed; ping-pong outranks plain duplication
        assert_eq!(
            g.observe("c", "a", true, msg, 3),
            LoopGuardDecision::Suppress(LoopReason::PingPong)
        );
        // different text from the same A — neither duplicate nor ping-pong; the third hop is allowed
        assert_eq!(g.observe("c", "a", true, "\u{645}\u{62a}\u{646} \u{62f}\u{6cc}\u{6af}\u{631}", 4), LoopGuardDecision::Allow);
        assert_eq!(g.consecutive_bot_turns("c"), 3);
        // the fourth hop — cap
        assert_eq!(
            g.observe("c", "a", true, "\u{645}\u{62a}\u{646} \u{62f}\u{6cc}\u{6af}\u{631}\u{650} \u{62a}\u{627}\u{632}\u{647}", 5),
            LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns)
        );
    }

    /// A flood from one sender within the window — even while hops reset on human messages.
    #[test]
    fn velocity_flood_is_suppressed() {
        let mut g = BotLoopGuard::new();
        // five distinct bot messages, each after a human message — hops keep resetting
        // but the (chat, spam) pace accumulates within the window
        let mut t = 1u64;
        for i in 0..MAX_EVENTS_PER_WINDOW {
            assert_eq!(g.observe("c", "human", false, "\u{633}\u{648}\u{627}\u{644} \u{62a}\u{627}\u{632}\u{647}", t), LoopGuardDecision::Allow);
            t += 1;
            assert_eq!(
                g.observe("c", "spam", true, &format!("\u{67e}\u{627}\u{633}\u{62e} \u{634}\u{645}\u{627}\u{631}\u{647}\u{654} {i}"), t),
                LoopGuardDecision::Allow
            );
            t += 10;
        }
        // the sixth message from the same sender — still inside the window; the flood is suppressed
        assert_eq!(
            g.observe("c", "spam", true, "\u{67e}\u{627}\u{633}\u{62e} \u{62a}\u{627}\u{632}\u{647}\u{654} \u{62f}\u{6cc}\u{6af}\u{631}", t),
            LoopGuardDecision::Suppress(LoopReason::RateLimit)
        );
        // another sender in the same conversation — not its flood, and the hops have reset
        assert_eq!(g.observe("c", "other", true, "\u{67e}\u{6cc}\u{627}\u{645} \u{645}\u{633}\u{62a}\u{642}\u{644}", t + 1), LoopGuardDecision::Allow);
        // a human resets; once the window passes, the first sender makes room again too
        assert_eq!(g.observe("c", "human", false, "\u{6a9}\u{627}\u{641}\u{6cc} \u{627}\u{633}\u{62a}", t + 2), LoopGuardDecision::Allow);
        assert_eq!(
            g.observe("c", "spam", true, "\u{67e}\u{627}\u{633}\u{62e} \u{628}\u{639}\u{62f} \u{627}\u{632} \u{67e}\u{646}\u{62c}\u{631}\u{647}", t + 200),
            LoopGuardDecision::Allow
        );
    }

    /// Separate conversations are independent — one conversation's guard does not catch another.
    #[test]
    fn conversations_are_isolated() {
        let mut g = BotLoopGuard::new();
        assert_eq!(g.observe("c1", "bot", true, "\u{645}\u{634}\u{62a}\u{631}\u{6a9}", 1), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c2", "bot", true, "\u{645}\u{634}\u{62a}\u{631}\u{6a9}", 2), LoopGuardDecision::Allow);
        assert_eq!(g.consecutive_bot_turns("c1"), 1);
        assert_eq!(g.consecutive_bot_turns("c2"), 1);
    }

    /// Explicit bounds — window and hop cap are tunable; a cap of 1 means a single hop,
    /// and a silent chain past the window dies (port of chain decay).
    #[test]
    fn explicit_limits_shape_the_guard() {
        let mut g = BotLoopGuard::with_limits(10, 1);
        assert_eq!(g.observe("c", "a", true, "\u{627}\u{648}\u{644}", 1), LoopGuardDecision::Allow);
        assert_eq!(
            g.observe("c", "a", true, "\u{62f}\u{648}\u{645}", 2),
            LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns)
        );
        // a silence gap beyond the window — the previous chain expired and the «first» text is allowed again
        assert_eq!(g.observe("c", "a", true, "\u{627}\u{648}\u{644}", 1 + 11), LoopGuardDecision::Allow);
    }

    /// A human is never suppressed — not even with duplicate text or a flood.
    #[test]
    fn humans_are_never_suppressed() {
        let mut g = BotLoopGuard::new();
        for i in 0u64..10 {
            assert_eq!(
                g.observe("c", "human", false, "\u{647}\u{645}\u{627}\u{646} \u{633}\u{648}\u{627}\u{644}", i + 1),
                LoopGuardDecision::Allow,
                "\u{67e}\u{6cc}\u{627}\u{645} \u{627}\u{646}\u{633}\u{627}\u{646} \u{647}\u{631}\u{6af}\u{632} \u{633}\u{631}\u{6a9}\u{648}\u{628} \u{646}\u{645}\u{6cc}‌\u{634}\u{648}\u{62f}"
            );
        }
    }

    /// Stable hash — consecutive spaces collapse and semantically equal texts hash equal.
    #[test]
    fn payload_hash_normalizes_whitespace() {
        assert_eq!(payload_hash("\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{686}\u{637}\u{648}\u{631}\u{647}\u{61f}"), payload_hash("\u{648}\u{636}\u{639}\u{6cc}\u{62a}   \n\u{686}\u{637}\u{648}\u{631}\u{647}\u{61f}"));
        assert_ne!(payload_hash("\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{686}\u{637}\u{648}\u{631}\u{647}\u{61f}"), payload_hash("\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{686}\u{637}\u{648}\u{631}\u{647}\u{61f}\u{61f}"));
        assert_eq!(payload_hash(""), payload_hash("   \t\n  "));
    }
}
