//! Telegram update backlog drain — port of `ShopifyHermes/bot.py::drain_backlog`
//! (vault folder 03).
//!
//! The Hermes pattern: at boot, one `getUpdates` call with `offset = -1` and
//! `timeout = 0`; Telegram returns the "latest" pending update and
//! `new offset = last update_id + 1` means "acknowledge the entire backlog without
//! processing it". Result: after a restart the bot does not answer stale messages
//! again (anti double-reply).
//!
//! The functions are pure (no network) — the real shell (teloxide / webhook) applies them
//! to the raw API output; with this shape, the logic is deterministic and testable offline.
#![forbid(unsafe_code)]

/// Backlog drain output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainOutcome {
    /// Backlog drained — start processing from this offset (`last + 1`).
    Drained(i64),
    /// No pending update — keep the previous stable offset (port of `ST["offset"]`).
    KeepCurrent,
}

impl DrainOutcome {
    /// The effective offset after draining — for logs and the next call.
    #[must_use]
    pub const fn offset(self, current: i64) -> i64 {
        match self {
            Self::Drained(o) => o,
            Self::KeepCurrent => current,
        }
    }

    /// Is this update stale and to be skipped? (port of `if uid < offset: continue`)
    #[must_use]
    pub const fn is_stale(self, current: i64, update_id: i64) -> bool {
        update_id < self.offset(current)
    }
}

/// Compute the drain offset from the raw `getUpdates {"offset": -1, "timeout": 0}` output.
///
/// Input: the returned update IDs in API order (small to large). Exactly like the Python:
/// if there was a row, `rs[-1]["update_id"] + 1`; if empty/errored → `KeepCurrent`.
#[must_use]
pub fn drain_backlog_offset(pending_update_ids: &[i64]) -> DrainOutcome {
    match pending_update_ids.iter().last() {
        Some(&last) if last >= 0 => DrainOutcome::Drained(last.saturating_add(1)),
        _ => DrainOutcome::KeepCurrent,
    }
}

/// The next offset after successfully processing one update — exactly the Python `offset = uid + 1`.
#[must_use]
pub const fn next_offset(update_id: i64) -> i64 {
    update_id.saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Hermes pattern — the last pending `update_id` + 1; empty → keep current.
    #[test]
    fn drain_uses_last_pending_plus_one() {
        assert_eq!(drain_backlog_offset(&[10, 11, 12]), DrainOutcome::Drained(13));
        assert_eq!(drain_backlog_offset(&[41]), DrainOutcome::Drained(42));
        assert_eq!(drain_backlog_offset(&[]), DrainOutcome::KeepCurrent);
        // A negative ID (error response) → honestly keeps current
        assert_eq!(drain_backlog_offset(&[-1]), DrainOutcome::KeepCurrent);
    }

    /// Effective offset and staleness — messages from before the drain are never processed.
    #[test]
    fn stale_detection_prevents_double_replies() {
        let drained = drain_backlog_offset(&[100]);
        assert_eq!(drained.offset(0), 101);
        assert!(drained.is_stale(0, 100), "\u{622}\u{67e}\u{62f}\u{6cc}\u{62a} \u{62a}\u{62e}\u{644}\u{6cc}\u{647}‌\u{634}\u{62f}\u{647} \u{6a9}\u{647}\u{646}\u{647} \u{627}\u{633}\u{62a}");
        assert!(drained.is_stale(0, 42), "\u{67e}\u{627}\u{6cc}\u{6cc}\u{646}‌\u{62a}\u{631} \u{627}\u{632} \u{62a}\u{62e}\u{644}\u{6cc}\u{647}\u{60c} \u{6a9}\u{647}\u{646}\u{647} \u{627}\u{633}\u{62a}");
        assert!(!drained.is_stale(0, 101), "\u{622}\u{67e}\u{62f}\u{6cc}\u{62a} \u{62a}\u{627}\u{632}\u{647} \u{67e}\u{631}\u{62f}\u{627}\u{632}\u{634} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f}");
        let kept = drain_backlog_offset(&[]);
        assert_eq!(kept.offset(77), 77);
        assert!(kept.is_stale(77, 76));
        assert!(!kept.is_stale(77, 77));
    }

    /// Processing chain — after each update the offset advances (no double reply).
    #[test]
    fn processing_chain_advances_offset() {
        let mut offset = drain_backlog_offset(&[5, 9]).offset(0);
        assert_eq!(offset, 10);
        for uid in [10, 11, 12] {
            assert!(!drain_backlog_offset(&[]).is_stale(offset, uid));
            offset = next_offset(uid);
        }
        assert_eq!(offset, 13);
        assert!(drain_backlog_offset(&[]).is_stale(offset, 12), "\u{62a}\u{6a9}\u{631}\u{627}\u{631} uid=12 \u{631}\u{62f} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f}");
    }
}
