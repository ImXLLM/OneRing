//! KIAN v2 Rust Adapter — Extracted & Adapted from `langgenius/dify` & `Snailclimb/JavaGuide`
//! Patterns:
//!   1. `FixedRecursiveSplitter` (`dify/api/core/rag/splitter/fixed_text_splitter.py`):
//!      Hierarchical separator cascade `["\n\n", "\n", "。", ". ", " ", ""]` with bounded chunking.
//!   2. `StreamingOutputModerator` (`dify/api/core/moderation/output_moderation.py`):
//!      Lock-free cursor-based streaming moderation with terminal error latch.
//!   3. `IdempotentStateTransition` (`JavaGuide/docs/system-design/state-machine-and-workflow.md`):
//!      Deterministic CAS transition + stable business-operation idempotency key for escrow/ledger.

use std::collections::HashSet;

pub const DIFY_DEFAULT_SEPARATORS: &[&str] = &["\n\n", "\n", "。", ". ", " ", ""];

/// Splits text using Dify's hierarchical separator cascade so semantic boundaries stay intact.
pub fn recursive_split_text(text: &str, chunk_size: usize, separators: &[&str]) -> Vec<String> {
    if text.chars().count() <= chunk_size || separators.is_empty() {
        return vec![text.to_string()];
    }

    let mut chosen_sep = "";
    let mut remaining_seps: &[&str] = &[];
    for (idx, sep) in separators.iter().enumerate() {
        if sep.is_empty() {
            chosen_sep = "";
            remaining_seps = &[];
            break;
        }
        if text.contains(sep) {
            chosen_sep = sep;
            remaining_seps = &separators[idx + 1..];
            break;
        }
    }

    let parts: Vec<String> = if chosen_sep.is_empty() {
        text.chars().map(|c| c.to_string()).collect()
    } else {
        text.split(chosen_sep)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    };

    let mut chunks = Vec::new();
    let mut current = String::new();
    for part in parts {
        let part_len = part.chars().count();
        if part_len > chunk_size && !remaining_seps.is_empty() {
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
            }
            chunks.extend(recursive_split_text(&part, chunk_size, remaining_seps));
            continue;
        }

        let joiner_len = if current.is_empty() || chosen_sep.is_empty() {
            0
        } else {
            chosen_sep.chars().count()
        };
        if !current.is_empty() && current.chars().count() + joiner_len + part_len > chunk_size {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() && !chosen_sep.is_empty() {
            current.push_str(chosen_sep);
        }
        current.push_str(&part);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Lock-free cursor state machine modeled after Dify's `OutputModeration`.
#[derive(Debug, Default)]
pub struct StreamingOutputModerator {
    buffer: String,
    checked_len: usize,
    halted: bool,
    latched_error: Option<String>,
}

impl StreamingOutputModerator {
    pub fn append_token<F>(&mut self, token: &str, mut check_segment: F) -> Result<&str, String>
    where
        F: FnMut(&str) -> Result<bool, String>,
    {
        if let Some(err) = &self.latched_error {
            return Err(err.clone());
        }
        if self.halted {
            return Ok("");
        }
        self.buffer.push_str(token);
        let unchecked = &self.buffer[self.checked_len..];
        match check_segment(unchecked) {
            Ok(true) => {
                self.checked_len = self.buffer.len();
                Ok(unchecked)
            }
            Ok(false) => {
                self.halted = true;
                Err("moderation_flagged".to_string())
            }
            Err(e) => {
                self.latched_error = Some(e.clone());
                Err(e)
            }
        }
    }
}

/// JavaGuide-inspired Escrow/Ledger CAS + Idempotency Guard:
/// - Same `idempotency_key` on retry returns `AlreadyApplied` without double-charging.
/// - Mismatched `expected_version` rejects concurrent racing writes (optimistic lock).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerState {
    Pending,
    HeldInEscrow,
    Released,
    Refunded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionOutcome {
    Applied { new_version: u64, state: LedgerState },
    AlreadyApplied,
    VersionConflict,
    InvalidTransition,
}

pub struct IdempotentLedgerRecord {
    pub state: LedgerState,
    pub version: u64,
    pub applied_keys: HashSet<String>,
}

impl IdempotentLedgerRecord {
    pub fn apply_transition(
        &mut self,
        idempotency_key: &str,
        expected_version: u64,
        next_state: LedgerState,
    ) -> TransitionOutcome {
        if self.applied_keys.contains(idempotency_key) {
            return TransitionOutcome::AlreadyApplied;
        }
        if self.version != expected_version {
            return TransitionOutcome::VersionConflict;
        }
        let allowed = matches!(
            (self.state, next_state),
            (LedgerState::Pending, LedgerState::HeldInEscrow)
                | (LedgerState::HeldInEscrow, LedgerState::Released)
                | (LedgerState::HeldInEscrow, LedgerState::Refunded)
        );
        if !allowed {
            return TransitionOutcome::InvalidTransition;
        }
        self.state = next_state;
        self.version += 1;
        self.applied_keys.insert(idempotency_key.to_string());
        TransitionOutcome::Applied {
            new_version: self.version,
            state: self.state,
        }
    }
}
