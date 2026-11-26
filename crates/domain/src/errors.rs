//! Typed errors + Result pattern — the fail-closed domain boundary.
//!
//! Rule: no panic crosses the domain boundary; every operation
//! returns `Result<T, DomainError>`. Panic is a programmer error only
//! (allowed inside tests). One-to-one with the TypeScript `errors.ts`.

use std::collections::BTreeMap;
use std::fmt;

/// 20 domain error codes — one-to-one with the TS `DomainErrorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DomainErrorCode {
    /// money value is not an integer (in TS: non-integer or outside the safe range)
    NonIntegerAmount,
    /// money value is negative
    NegativeAmount,
    /// value escapes the safe integer range (2^53−1)
    AmountOutOfRange,
    /// basis points (bps) outside [0, 10000]
    BpsOutOfRange,
    /// transaction has fewer than two rows
    TooFewPostings,
    /// transaction rows do not sum to zero
    UnbalancedTransaction,
    /// unknown account
    UnknownAccount,
    /// account kind not allowed for this operation
    InvalidAccountKind,
    /// duplicate account
    DuplicateAccount,
    /// account direction rule violated (balance debit/mint and so on)
    AccountOverdraft,
    /// illegal state machine transition
    InvalidTransition,
    /// escrow not found
    EscrowNotFound,
    /// duplicate escrow
    EscrowAlreadyExists,
    /// operation ≥ 50,000 Đ without human approval
    RequiresHumanApproval,
    /// annual mint cap violated
    MintCapExceeded,
    /// HITL record not found
    HitlNotFound,
    /// this operation does not require HITL
    HitlNotRequired,
    /// this HITL record has already been decided
    HitlAlreadyDecided,
    /// HITL record has expired
    HitlExpired,
    /// product not found
    ProductNotFound,
}

impl DomainErrorCode {
    /// Canonical string — identical to the TS version (for logs, API and dual-run).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NonIntegerAmount => "NON_INTEGER_AMOUNT",
            Self::NegativeAmount => "NEGATIVE_AMOUNT",
            Self::AmountOutOfRange => "AMOUNT_OUT_OF_RANGE",
            Self::BpsOutOfRange => "BPS_OUT_OF_RANGE",
            Self::TooFewPostings => "TOO_FEW_POSTINGS",
            Self::UnbalancedTransaction => "UNBALANCED_TRANSACTION",
            Self::UnknownAccount => "UNKNOWN_ACCOUNT",
            Self::InvalidAccountKind => "INVALID_ACCOUNT_KIND",
            Self::DuplicateAccount => "DUPLICATE_ACCOUNT",
            Self::AccountOverdraft => "ACCOUNT_OVERDRAFT",
            Self::InvalidTransition => "INVALID_TRANSITION",
            Self::EscrowNotFound => "ESCROW_NOT_FOUND",
            Self::EscrowAlreadyExists => "ESCROW_ALREADY_EXISTS",
            Self::RequiresHumanApproval => "REQUIRES_HUMAN_APPROVAL",
            Self::MintCapExceeded => "MINT_CAP_EXCEEDED",
            Self::HitlNotFound => "HITL_NOT_FOUND",
            Self::HitlNotRequired => "HITL_NOT_REQUIRED",
            Self::HitlAlreadyDecided => "HITL_ALREADY_DECIDED",
            Self::HitlExpired => "HITL_EXPIRED",
            Self::ProductNotFound => "PRODUCT_NOT_FOUND",
        }
    }
}

impl fmt::Display for DomainErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Metadata value — equivalent to `string | number` in TS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaValue {
    /// text value
    Text(String),
    /// integer value
    Int(i64),
}

impl fmt::Display for MetaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(s) => f.write_str(s),
            Self::Int(i) => write!(f, "{i}"),
        }
    }
}

/// Error metadata — keys sorted (`BTreeMap`) so dual-run output is deterministic.
pub type Meta = BTreeMap<String, MetaValue>;

/// Domain error: typed code + message + optional metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError {
    /// error code
    pub code: DomainErrorCode,
    /// explanatory message (English/Persian — for logs and UI)
    pub message: String,
    /// structured metadata (e.g. `sum`, `accountId`, `newBalance`)
    pub meta: Meta,
}

impl DomainError {
    /// Simple error without metadata.
    #[must_use]
    pub fn new(code: DomainErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), meta: Meta::new() }
    }

    /// Add one metadata pair (builder).
    #[must_use]
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<MetaValue>) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DomainError {}

impl From<&str> for MetaValue {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<String> for MetaValue {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<i64> for MetaValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

/// Domain operation result — equivalent to the TS version’s custom `Result<T>`.
pub type DomainResult<T> = Result<T, DomainError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twenty_codes_match_ts_strings() {
        assert_eq!(DomainErrorCode::NonIntegerAmount.as_str(), "NON_INTEGER_AMOUNT");
        assert_eq!(DomainErrorCode::ProductNotFound.as_str(), "PRODUCT_NOT_FOUND");
        assert_eq!(DomainErrorCode::RequiresHumanApproval.to_string(), "REQUIRES_HUMAN_APPROVAL");
    }

    #[test]
    fn error_display_and_meta_builder() {
        let e = DomainError::new(DomainErrorCode::UnbalancedTransaction, "bad sum")
            .with_meta("sum", 15_i64)
            .with_meta("note", "\u{641}\u{627}\u{631}\u{633}\u{6cc}");
        assert_eq!(e.to_string(), "UNBALANCED_TRANSACTION: bad sum");
        assert_eq!(e.meta.get("sum"), Some(&MetaValue::Int(15)));
        assert_eq!(e.meta.get("note"), Some(&MetaValue::Text("\u{641}\u{627}\u{631}\u{633}\u{6cc}".into())));
    }
}
