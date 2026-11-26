//! budget — the hard envelope of the daily AI budget (legacy phase 6).
//!
//! Pure, correct math: `used_today + estimate ≤ cap` or rejected. This crate
//! has no database — the server brings today's usage from the ledger and only supplies the number.
//! A cap of 0 means "everything rejected" — fail-closed; a negative cap is never constructed.

/// Budget envelope — each user's daily token cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetEnvelope {
    /// Daily token cap (UTC day)
    pub daily_cap_tokens: i64,
}

/// Budget gate decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetDecision {
    /// Allowed — the remainder after subtracting today's usage + this call's estimate
    Allowed {
        /// `cap − used − estimate` (≥ 0)
        remaining_after: i64,
    },
    /// Exhausted — details for the 429 `AI_CAP_EXCEEDED`
    Exhausted {
        /// Today's usage
        used: i64,
        /// Cap
        cap: i64,
    },
}

impl BudgetEnvelope {
    /// Constructor — a negative cap is invalid (`None`) to preserve fail-closed.
    #[must_use]
    pub fn new(daily_cap_tokens: i64) -> Option<Self> {
        (daily_cap_tokens >= 0).then_some(Self { daily_cap_tokens })
    }

    /// Gate before the call — hard: usage + estimate must fit inside the envelope and
    /// Today's usage must never have reached the cap (a zero cap rejects everything).
    ///
    /// Negative input (upstream data corruption) saturates to zero — the gate never opens.
    #[must_use]
    pub fn precheck(&self, used_today: i64, estimate: i64) -> BudgetDecision {
        let used = used_today.max(0);
        let est = estimate.max(0);
        if used >= self.daily_cap_tokens {
            return BudgetDecision::Exhausted { used, cap: self.daily_cap_tokens };
        }
        if used + est <= self.daily_cap_tokens {
            BudgetDecision::Allowed { remaining_after: self.daily_cap_tokens - used - est }
        } else {
            BudgetDecision::Exhausted { used, cap: self.daily_cap_tokens }
        }
    }

    /// Envelope remainder after one call — for the wire response.
    #[must_use]
    pub fn remaining(&self, used_today: i64) -> i64 {
        (self.daily_cap_tokens - used_today.max(0)).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hard envelope: estimate larger than the remainder → rejected with exact 429 details.
    #[test]
    fn hard_envelope_refuses_when_estimate_exceeds_room() {
        let env = BudgetEnvelope::new(1_000).expect("positive cap");
        assert_eq!(env.precheck(900, 200), BudgetDecision::Exhausted { used: 900, cap: 1_000 });
    }

    /// Inside the envelope → allowed with the exact remainder; usage equal to the cap → exhausted.
    #[test]
    fn inside_envelope_is_allowed_with_exact_remaining() {
        let env = BudgetEnvelope::new(1_000).expect("positive cap");
        assert_eq!(env.precheck(300, 200), BudgetDecision::Allowed { remaining_after: 500 });
        // used == cap → gate closed (same rule as a zero cap — today's usage is spent)
        assert_eq!(env.precheck(1_000, 0), BudgetDecision::Exhausted { used: 1_000, cap: 1_000 });
    }

    /// A zero cap rejects everything — fail-closed.
    #[test]
    fn zero_cap_refuses_everything() {
        let env = BudgetEnvelope::new(0).expect("zero is valid");
        assert!(matches!(env.precheck(0, 1), BudgetDecision::Exhausted { .. }));
        assert!(matches!(env.precheck(0, 0), BudgetDecision::Exhausted { .. }));
    }

    /// A negative cap is never created; negative consumption clamps to zero.
    #[test]
    fn negative_inputs_fail_closed() {
        assert!(BudgetEnvelope::new(-1).is_none());
        let env = BudgetEnvelope::new(100).expect("cap");
        assert_eq!(env.precheck(-50, 10), BudgetDecision::Allowed { remaining_after: 90 });
        assert_eq!(env.remaining(-999), 100);
    }
}
