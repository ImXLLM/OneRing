//! budget — پاکت سخت بودجهٔ روزانهٔ AI (فاز ۶ لگاسی).
//!
//! ریاضی صحیح و خالص: `used_today + estimate ≤ cap` وگرنه رد. این کرِیت
//! دیتابیس ندارد — سرور مصرف امروز را از دفتر می‌آورد و فقط عدد می‌دهد.
//! سقف ۰ یعنی «همه‌چیز رد» — fail-closed؛ سقف منفی هرگز ساخته نمی‌شود.

/// پاکت بودجه — سقف روزانهٔ توکن هر کاربر.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetEnvelope {
    /// سقف روزانهٔ توکن (UTC روز)
    pub daily_cap_tokens: i64,
}

/// تصمیم گیت بودجه.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetDecision {
    /// مجاز — باقی‌مانده پس از کسر مصرف امروز + برآورد این فراخوانی
    Allowed {
        /// `cap − used − estimate` (≥ ۰)
        remaining_after: i64,
    },
    /// تمام‌شده — جزئیات برای ۴۲۹ `AI_CAP_EXCEEDED`
    Exhausted {
        /// مصرف امروز
        used: i64,
        /// سقف
        cap: i64,
    },
}

impl BudgetEnvelope {
    /// سازنده — سقف منفی نامعتبر است (`None`) تا fail-closed حفظ شود.
    #[must_use]
    pub fn new(daily_cap_tokens: i64) -> Option<Self> {
        (daily_cap_tokens >= 0).then_some(Self { daily_cap_tokens })
    }

    /// گیت قبل از فراخوانی — سخت: مصرف + برآورد باید داخل پاکت جا شود و
    /// مصرف امروز هرگز نباید به سقف رسیده باشد (سقف صفر = همه‌چیز رد).
    ///
    /// ورودی منفی (خرابی دادهٔ بالا) به صفر سیر می‌شود — گیت هرگز باز نمی‌شود.
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

    /// باقی‌ماندهٔ پاکت پس از یک فراخوانی — برای پاسخ سیم.
    #[must_use]
    pub fn remaining(&self, used_today: i64) -> i64 {
        (self.daily_cap_tokens - used_today.max(0)).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// پاکت سخت: برآورد بزرگ‌تر از باقی‌مانده → رد با جزئیات دقیق ۴۲۹.
    #[test]
    fn hard_envelope_refuses_when_estimate_exceeds_room() {
        let env = BudgetEnvelope::new(1_000).expect("positive cap");
        assert_eq!(env.precheck(900, 200), BudgetDecision::Exhausted { used: 900, cap: 1_000 });
    }

    /// داخل پاکت → مجاز با باقی‌ماندهٔ دقیق؛ مصرفِ مساویِ سقف → تمام‌شده.
    #[test]
    fn inside_envelope_is_allowed_with_exact_remaining() {
        let env = BudgetEnvelope::new(1_000).expect("positive cap");
        assert_eq!(env.precheck(300, 200), BudgetDecision::Allowed { remaining_after: 500 });
        // used == cap → گیت بسته (همان قانون سقف صفر — مصرف امروز تمام است)
        assert_eq!(env.precheck(1_000, 0), BudgetDecision::Exhausted { used: 1_000, cap: 1_000 });
    }

    /// سقف صفر = همه‌چیز رد — fail-closed.
    #[test]
    fn zero_cap_refuses_everything() {
        let env = BudgetEnvelope::new(0).expect("zero is valid");
        assert!(matches!(env.precheck(0, 1), BudgetDecision::Exhausted { .. }));
        assert!(matches!(env.precheck(0, 0), BudgetDecision::Exhausted { .. }));
    }

    /// سقف منفی هرگز ساخته نمی‌شود؛ مصرف منفی به صفر سیر می‌شود.
    #[test]
    fn negative_inputs_fail_closed() {
        assert!(BudgetEnvelope::new(-1).is_none());
        let env = BudgetEnvelope::new(100).expect("cap");
        assert_eq!(env.precheck(-50, 10), BudgetDecision::Allowed { remaining_after: 90 });
        assert_eq!(env.remaining(-999), 100);
    }
}
