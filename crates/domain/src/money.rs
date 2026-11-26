//! ADR-002 — money is always an integer. Never a float.
//!
//! 1 Đ (Din) = one smallest unit = one integer. Legacy decimals (e.g. 187,672.8)
//! are resolved once, at migration time, with an explicit per-citizen rounding table.
//! In Rust the `Din` type is a newtype over `i64`, and the 2^53−1 safe boundary
//! (exactly JavaScript’s `Number.MAX_SAFE_INTEGER`) is deliberately preserved so
//! dual-run and adapters see the same boundaries.

use crate::errors::{DomainError, DomainErrorCode, DomainResult};
use std::fmt;

/// Maximum safe value — exactly 2^53−1 (the `Number.MAX_SAFE_INTEGER` boundary).
pub const MAX_DIN: i64 = 9_007_199_254_740_991;

/// Non-negative DNX money value — newtype equivalent of the branded `Din` in TS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Din(pub(crate) i64);

impl Din {
    /// Zero value.
    pub const ZERO: Din = Din(0);

    /// Validation like TS `din()`: negative → `NEGATIVE_AMOUNT`,
    /// above `MAX_DIN` → `AMOUNT_OUT_OF_RANGE`.
    ///
    /// # Errors
    /// If `n` is negative or above `MAX_DIN`.
    pub fn new(n: i64) -> DomainResult<Din> {
        if n < 0 {
            return Err(DomainError::new(
                DomainErrorCode::NegativeAmount,
                format!("Money cannot be negative, got {n}"),
            ));
        }
        if n > MAX_DIN {
            return Err(DomainError::new(
                DomainErrorCode::AmountOutOfRange,
                format!("Money exceeds the safe integer range, got {n}"),
            ));
        }
        Ok(Din(n))
    }

    /// Raw value — for adapters and logging only.
    #[must_use]
    pub const fn as_i64(self) -> i64 {
        self.0
    }

    /// Addition with overflow protection — equivalent to `addDin` (std naming convention).
    ///
    /// # Errors
    /// If the sum exceeds `MAX_DIN`.
    pub fn checked_add(self, other: Din) -> DomainResult<Din> {
        match self.0.checked_add(other.0) {
            Some(sum) if sum <= MAX_DIN => Ok(Din(sum)),
            _ => Err(DomainError::new(
                DomainErrorCode::AmountOutOfRange,
                format!("Sum exceeds the safe integer range: {}+{}", self.0, other.0),
            )),
        }
    }

    /// Subtraction that cannot go negative — equivalent to `subDin` (std naming convention).
    ///
    /// # Errors
    /// If `other > self`.
    pub fn checked_sub(self, other: Din) -> DomainResult<Din> {
        if other.0 > self.0 {
            return Err(DomainError::new(
                DomainErrorCode::NegativeAmount,
                format!("Cannot subtract {} from {}", other.0, self.0),
            ));
        }
        Ok(Din(self.0 - other.0))
    }

    /// The smaller of two values — equivalent to `minDin`.
    #[must_use]
    pub const fn min(self, other: Din) -> Din {
        if self.0 <= other.0 {
            self
        } else {
            other
        }
    }

    /// Multiply by basis points (1% = 100 bps), rounding down.
    /// The Charter tax engine uses this; bps outside [0, 10000] are rejected.
    ///
    /// # Errors
    /// `BPS_OUT_OF_RANGE` for invalid bps, `AMOUNT_OUT_OF_RANGE` for overflow.
    pub fn mul_bps(self, bps: i64) -> DomainResult<Din> {
        if !(0..=10_000).contains(&bps) {
            return Err(DomainError::new(
                DomainErrorCode::BpsOutOfRange,
                format!("bps must be an integer in [0, 10000], got {bps}"),
            ));
        }
        match self.0.checked_mul(bps) {
            // the product is always ≥ 0; integer division means floor.
            Some(product) => Ok(Din(product / 10_000)),
            None => Err(DomainError::new(
                DomainErrorCode::AmountOutOfRange,
                format!("Overflow computing {} * {} / 10000", self.0, bps),
            )),
        }
    }
}

impl fmt::Display for Din {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_din(*self))
    }
}

/// Format with en-US three-digit grouping + the Đ symbol — equivalent to `formatDin`.
/// example: `187,672 Đ`
#[must_use]
pub fn format_din(v: Din) -> String {
    let digits = v.as_i64().to_string();
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3 + 2);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out.push_str(" \u{110}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn d(n: i64) -> Din {
        Din(n)
    }

    #[test]
    fn accepts_safe_non_negative_integers() {
        assert_eq!(Din::new(0).unwrap().as_i64(), 0);
        assert_eq!(Din::new(1).unwrap().as_i64(), 1);
        assert_eq!(Din::new(187_672).unwrap().as_i64(), 187_672);
        assert_eq!(Din::ZERO, Din::new(0).unwrap());
    }

    #[test]
    fn rejects_negative_amounts() {
        let r = Din::new(-1);
        assert_eq!(r.unwrap_err().code, DomainErrorCode::NegativeAmount);
    }

    #[test]
    fn rejects_amounts_beyond_the_safe_integer_range() {
        // 2^53 — exactly one beyond the TS boundary
        let r = Din::new(9_007_199_254_740_992);
        assert_eq!(r.unwrap_err().code, DomainErrorCode::AmountOutOfRange);
        // the boundary itself is allowed
        assert!(Din::new(MAX_DIN).is_ok());
    }

    #[test]
    fn adds_and_subtracts() {
        let a = d(2);
        let b = d(3);
        assert_eq!(a.checked_add(b).unwrap().as_i64(), 5);
        assert_eq!(b.checked_sub(a).unwrap().as_i64(), 1);
    }

    #[test]
    fn rejects_subtraction_underflow() {
        let r = d(2).checked_sub(d(3));
        assert_eq!(r.unwrap_err().code, DomainErrorCode::NegativeAmount);
    }

    #[test]
    fn rejects_addition_overflow() {
        let a = d(9_007_199_254_740_000);
        let b = d(1_000);
        assert!(a.checked_add(b).is_err());
        assert_eq!(a.checked_add(b).unwrap_err().code, DomainErrorCode::AmountOutOfRange);
    }

    #[test]
    fn mul_bps_floors() {
        // 10% of 55 = 5, not 5.5
        assert_eq!(d(55).mul_bps(1_000).unwrap().as_i64(), 5);
        assert_eq!(d(100).mul_bps(1_000).unwrap().as_i64(), 10);
        assert_eq!(d(0).mul_bps(1_000).unwrap().as_i64(), 0);
    }

    #[test]
    fn mul_bps_rejects_out_of_range_bps() {
        assert_eq!(d(10).mul_bps(10_001).unwrap_err().code, DomainErrorCode::BpsOutOfRange);
        assert_eq!(d(10).mul_bps(-1).unwrap_err().code, DomainErrorCode::BpsOutOfRange);
    }

    #[test]
    fn orders_and_mins() {
        assert!(d(2) < d(3));
        assert_eq!(d(7).min(d(3)).as_i64(), 3);
        assert_eq!(d(1).min(d(9)).as_i64(), 1);
    }

    #[test]
    fn formats_with_the_dinar_symbol() {
        assert_eq!(format_din(Din::new(187_672).unwrap()), "187,672 \u{110}");
        assert_eq!(format_din(Din::ZERO), "0 \u{110}");
        assert_eq!(format_din(Din::new(1_234_567).unwrap()), "1,234,567 \u{110}");
        // Display prints the same
        assert_eq!(Din::new(187_672).unwrap().to_string(), "187,672 \u{110}");
    }
}
