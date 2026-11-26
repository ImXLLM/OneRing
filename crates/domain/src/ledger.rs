//! ADR-003 — the immutable double-entry ledger (pure operational kernel).
//!
//! Contracts:
//!  - posting value > 0 → credit (into the account), value < 0 → debit (out of the account).
//!  - every transaction has at least 2 postings and their sum is **always** zero.
//!  - direction rules per account kind (fail-closed):
//!    user/escrow/treasury → the balance can never go negative,
//!    mint → the balance can never go positive (it only emits supply),
//!    burn → the balance can never go negative (it only receives destroyed money).
//!  - idempotencyKey: posting again with the same key = a no-op replay returning
//!    the original transaction (safe for bot/API retries).
//!  - invariant (sanctity): Σ of all account balances = 0, always.
//!
//! Immutability: every operation returns a **new** `LedgerState`
//! (clone-on-write) — an error never corrupts the state.

use crate::errors::{DomainError, DomainErrorCode, DomainResult, MetaValue};
use crate::events::EPOCH_ISO;
use crate::money::MAX_DIN;
use std::collections::BTreeMap;

/// Account kind — drives the fail-closed rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccountKind {
    /// Citizen account — can never go negative
    User,
    /// Escrow account — can never go negative
    Escrow,
    /// National treasury account — can never go negative
    Treasury,
    /// Mint account — can never go positive (only emits supply)
    Mint,
    /// Burn account — can never go negative (only receives destroyed money)
    Burn,
}

impl AccountKind {
    /// Canonical string — identical to the TS version.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Escrow => "escrow",
            Self::Treasury => "treasury",
            Self::Mint => "mint",
            Self::Burn => "burn",
        }
    }
}

impl std::fmt::Display for AccountKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Account id in the ledger (e.g. `user:buyer`, `citizen:123`, or `escrow:e1`).
pub type AccountId = String;

/// An account in the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// Unique id (e.g. `user:buyer` or `escrow:e1`)
    pub id: AccountId,
    /// Account kind
    pub kind: AccountKind,
    /// Optional label (e.g. a Persian name)
    pub label: Option<String>,
}

/// One transaction posting — a signed integer amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    /// Target account
    pub account_id: String,
    /// Signed amount (credit > 0, debit < 0)
    pub amount: i64,
}

impl Posting {
    /// Posting construction shortcut.
    #[must_use]
    pub fn new(account_id: impl Into<String>, amount: i64) -> Self {
        Self { account_id: account_id.into(), amount }
    }
}

/// A recorded transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostedTransaction {
    /// Unique id (`tx_N` or explicit)
    pub id: String,
    /// Optional idempotency key
    pub idempotency_key: Option<String>,
    /// Postings (independent copy)
    pub postings: Vec<Posting>,
    /// Optional note
    pub memo: Option<String>,
    /// ISO-8601 timestamp (supplied by the caller/adapter — no clock domain)
    pub at: String,
}

/// Full ledger state — immutable from the caller's point of view.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LedgerState {
    /// Accounts keyed by id
    pub accounts: BTreeMap<String, Account>,
    /// Balances keyed by id
    pub balances: BTreeMap<String, i64>,
    /// Recorded transactions
    pub txs: BTreeMap<String, PostedTransaction>,
    /// Idempotency key → transaction id
    pub idempotency: BTreeMap<String, String>,
}

/// Known system accounts (shared contract with the adapters).
pub const MINT_ACCOUNT_ID: &str = "mint:genesis";
/// National treasury account.
pub const TREASURY_ACCOUNT_ID: &str = "treasury:nation";
/// Burn account.
pub const BURN_ACCOUNT_ID: &str = "burn:void";

/// Opens an empty ledger.
#[must_use]
pub fn open_ledger() -> LedgerState {
    LedgerState::default()
}

/// Add an account — duplicate ids are rejected.
///
/// # Errors
/// `DUPLICATE_ACCOUNT` if the account already exists.
pub fn add_account(state: &LedgerState, account: Account) -> DomainResult<LedgerState> {
    if state.accounts.contains_key(&account.id) {
        return Err(DomainError::new(
            DomainErrorCode::DuplicateAccount,
            format!("Account {} already exists", account.id),
        ));
    }
    let mut accounts = state.accounts.clone();
    let id = account.id.clone();
    accounts.insert(id.clone(), account);
    let mut balances = state.balances.clone();
    balances.insert(id, 0);
    Ok(LedgerState {
        accounts,
        balances,
        txs: state.txs.clone(),
        idempotency: state.idempotency.clone(),
    })
}

/// Transaction post command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostCommand {
    /// Optional explicit id (default: `tx_N`)
    pub id: Option<String>,
    /// Optional idempotency key
    pub idempotency_key: Option<String>,
    /// Postings (>= 2, summing to zero)
    pub postings: Vec<Posting>,
    /// Optional note
    pub memo: Option<String>,
    /// Optional ISO timestamp — defaults to the deterministic epoch
    pub at: Option<String>,
}

/// Transaction post result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostOutcome {
    /// New ledger state (on replay, the input state unchanged)
    pub state: LedgerState,
    /// Recorded transaction (on replay, the original transaction)
    pub tx: PostedTransaction,
    /// Was this a replay?
    pub replayed: bool,
}

/// Direction rule violation — `mint` cannot go positive; the rest cannot go negative.
#[must_use]
pub const fn is_overdraft_violation(kind: AccountKind, balance: i64) -> bool {
    match kind {
        AccountKind::Mint => balance > 0,
        AccountKind::Burn | AccountKind::User | AccountKind::Escrow | AccountKind::Treasury => {
            balance < 0
        }
    }
}

/// Post a transaction — four steps: replay → shape → balance → direction rules, then commit.
///
/// # Errors
/// Domain codes: `TOO_FEW_POSTINGS`, `NON_INTEGER_AMOUNT`,
/// `UNKNOWN_ACCOUNT`, `UNBALANCED_TRANSACTION`, `ACCOUNT_OVERDRAFT`.
pub fn post(state: &LedgerState, cmd: &PostCommand) -> DomainResult<PostOutcome> {
    // 1) Idempotent replay — returning the original transaction is allowed, not an error.
    if let Some(key) = &cmd.idempotency_key
        && let Some(existing_id) = state.idempotency.get(key)
        && let Some(existing) = state.txs.get(existing_id)
    {
        return Ok(PostOutcome {
            state: state.clone(),
            tx: existing.clone(),
            replayed: true,
        });
    }

    // 2) Shape validation.
    if cmd.postings.len() < 2 {
        return Err(DomainError::new(
            DomainErrorCode::TooFewPostings,
            format!("A transaction needs at least 2 postings, got {}", cmd.postings.len()),
        ));
    }
    for p in &cmd.postings {
        // TS-safe boundary: |amount| ≤ 2^53−1
        if p.amount.unsigned_abs() > MAX_DIN as u64 {
            return Err(DomainError::new(
                DomainErrorCode::NonIntegerAmount,
                format!("Posting to {} must be a safe integer, got {}", p.account_id, p.amount),
            ));
        }
        if !state.accounts.contains_key(&p.account_id) {
            return Err(DomainError::new(
                DomainErrorCode::UnknownAccount,
                format!("Unknown account {}", p.account_id),
            ));
        }
    }
    let mut sum: i64 = 0;
    for p in &cmd.postings {
        sum = match sum.checked_add(p.amount) {
            Some(s) => s,
            None => {
                return Err(DomainError::new(
                    DomainErrorCode::UnbalancedTransaction,
                    "Postings must sum to zero (overflow while summing)",
                )
                .with_meta("sum", MetaValue::Text("overflow".into())));
            }
        };
    }
    if sum != 0 {
        return Err(DomainError::new(
            DomainErrorCode::UnbalancedTransaction,
            format!("Postings must sum to zero, got {sum}"),
        )
        .with_meta("sum", sum));
    }

    // 3) Direction rules (fail-closed) — evaluated on the draft; commit only if clean.
    let mut draft = state.balances.clone();
    for p in &cmd.postings {
        *draft.entry(p.account_id.clone()).or_insert(0) += p.amount;
    }
    for (account_id, new_balance) in &draft {
        if let Some(account) = state.accounts.get(account_id)
            && is_overdraft_violation(account.kind, *new_balance)
        {
            return Err(DomainError::new(
                DomainErrorCode::AccountOverdraft,
                format!(
                    "Account {} ({}) cannot go to {}",
                    account_id, account.kind, new_balance
                ),
            )
            .with_meta("accountId", account_id.clone())
            .with_meta("kind", account.kind.as_str())
            .with_meta("newBalance", *new_balance));
        }
    }

    // 4) Commit — a new immutable state.
    let tx_id = cmd.id.clone().unwrap_or_else(|| format!("tx_{}", state.txs.len() + 1));
    let tx = PostedTransaction {
        id: tx_id,
        idempotency_key: cmd.idempotency_key.clone(),
        postings: cmd.postings.clone(),
        memo: cmd.memo.clone(),
        at: cmd.at.clone().unwrap_or_else(|| EPOCH_ISO.to_owned()),
    };
    let mut txs = state.txs.clone();
    txs.insert(tx.id.clone(), tx.clone());
    let mut idempotency = state.idempotency.clone();
    if let Some(key) = &cmd.idempotency_key {
        idempotency.insert(key.clone(), tx.id.clone());
    }
    Ok(PostOutcome {
        state: LedgerState { accounts: state.accounts.clone(), balances: draft, txs, idempotency },
        tx,
        replayed: false,
    })
}

/// The balance of one account.
///
/// # Errors
/// `UNKNOWN_ACCOUNT` if the account does not exist.
pub fn balance_of(state: &LedgerState, account_id: &str) -> DomainResult<i64> {
    state.balances.get(account_id).copied().ok_or_else(|| {
        DomainError::new(DomainErrorCode::UnknownAccount, format!("Unknown account {account_id}"))
    })
}

/// Trial-balance row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialBalanceRow {
    /// Account id
    pub account_id: String,
    /// Account kind
    pub kind: AccountKind,
    /// Balance
    pub balance: i64,
}

/// Sanity balance — sacred invariant: `balanced == true` always.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialBalance {
    /// Rows (sorted by id — deterministic)
    pub rows: Vec<TrialBalanceRow>,
    /// Σ of all balances
    pub sum: i64,
    /// Is Σ = 0?
    pub balanced: bool,
}

/// Compute the trial balance.
#[must_use]
pub fn trial_balance(state: &LedgerState) -> TrialBalance {
    let mut rows = Vec::new();
    let mut sum: i64 = 0;
    for (account_id, balance) in &state.balances {
        if let Some(account) = state.accounts.get(account_id) {
            rows.push(TrialBalanceRow {
                account_id: account_id.clone(),
                kind: account.kind,
                balance: *balance,
            });
            sum += balance;
        }
    }
    TrialBalance { sum, balanced: sum == 0, rows }
}

/// Money in citizens' hands (user + escrow + treasury) = minted − burned.
#[must_use]
pub fn circulating_supply(state: &LedgerState) -> i64 {
    let mut total: i64 = 0;
    for (account_id, balance) in &state.balances {
        if let Some(account) = state.accounts.get(account_id)
            && matches!(
                account.kind,
                AccountKind::User | AccountKind::Escrow | AccountKind::Treasury
            )
        {
            total += balance;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINT: &str = MINT_ACCOUNT_ID;
    const TREASURY: &str = TREASURY_ACCOUNT_ID;
    const BURN: &str = BURN_ACCOUNT_ID;

    fn new_ledger() -> LedgerState {
        let l = open_ledger();
        let l = add_account(&l, Account { id: MINT.into(), kind: AccountKind::Mint, label: None })
            .unwrap();
        let l = add_account(
            &l,
            Account { id: TREASURY.into(), kind: AccountKind::Treasury, label: None },
        )
        .unwrap();
        let l = add_account(&l, Account { id: BURN.into(), kind: AccountKind::Burn, label: None })
            .unwrap();
        let l = add_account(
            &l,
            Account {
                id: "user:buyer".into(),
                kind: AccountKind::User,
                label: Some("\u{62e}\u{631}\u{6cc}\u{62f}\u{627}\u{631}".into()),
            },
        )
        .unwrap();
        let l = add_account(
            &l,
            Account {
                id: "user:seller".into(),
                kind: AccountKind::User,
                label: Some("\u{641}\u{631}\u{648}\u{634}\u{646}\u{62f}\u{647}".into()),
            },
        )
        .unwrap();
        add_account(&l, Account { id: "user:c".into(), kind: AccountKind::User, label: None })
            .unwrap()
    }

    fn mint_to(state: &LedgerState, account_id: &str, amount: i64) -> LedgerState {
        post(
            state,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![Posting::new(MINT, -amount), Posting::new(account_id, amount)],
                memo: Some(format!("mint {amount} → {account_id}")),
                at: None,
            },
        )
        .unwrap()
        .state
    }

    #[test]
    fn opens_empty_and_keeps_accounts_unique() {
        let l = new_ledger();
        assert_eq!(trial_balance(&l).sum, 0);
        let dup =
            add_account(&l, Account { id: MINT.into(), kind: AccountKind::Mint, label: None });
        assert!(dup.is_err());
        assert_eq!(dup.unwrap_err().code, DomainErrorCode::DuplicateAccount);
    }

    #[test]
    fn mints_supply_and_keeps_conservation() {
        let mut l = new_ledger();
        l = mint_to(&l, "user:buyer", 1_000);
        assert_eq!(balance_of(&l, "user:buyer").unwrap(), 1_000);
        assert_eq!(balance_of(&l, MINT).unwrap(), -1_000);
        let tb = trial_balance(&l);
        assert!(tb.balanced);
        assert_eq!(circulating_supply(&l), 1_000);
    }

    #[test]
    fn rejects_unbalanced_transactions() {
        let l = new_ledger();
        let r = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![Posting::new("user:buyer", 10), Posting::new("user:seller", 5)],
                memo: None,
                at: None,
            },
        );
        let e = r.unwrap_err();
        assert_eq!(e.code, DomainErrorCode::UnbalancedTransaction);
        assert_eq!(e.meta.get("sum"), Some(&MetaValue::Int(15)));
    }

    #[test]
    fn rejects_single_posting_transactions() {
        let l = new_ledger();
        let r = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![Posting::new("user:buyer", 10)],
                memo: None,
                at: None,
            },
        );
        assert_eq!(r.unwrap_err().code, DomainErrorCode::TooFewPostings);
    }

    #[test]
    fn rejects_unknown_accounts_and_out_of_range_postings() {
        let l = new_ledger();
        let unknown = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![Posting::new("user:buyer", -10), Posting::new("user:ghost", 10)],
                memo: None,
                at: None,
            },
        );
        assert_eq!(unknown.unwrap_err().code, DomainErrorCode::UnknownAccount);
        // in TS a fractional post is rejected; in Rust the same code guards the safe boundary
        let beyond = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![
                    Posting::new("user:buyer", -9_007_199_254_740_992),
                    Posting::new("user:seller", 9_007_199_254_740_992),
                ],
                memo: None,
                at: None,
            },
        );
        assert_eq!(beyond.unwrap_err().code, DomainErrorCode::NonIntegerAmount);
    }

    #[test]
    fn never_lets_a_user_go_negative() {
        let mut l = new_ledger();
        l = mint_to(&l, "user:buyer", 500);
        let r = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![Posting::new("user:buyer", -600), Posting::new("user:seller", 600)],
                memo: None,
                at: None,
            },
        );
        let e = r.unwrap_err();
        assert_eq!(e.code, DomainErrorCode::AccountOverdraft);
        assert_eq!(e.meta.get("newBalance"), Some(&MetaValue::Int(-100)));
        // state untouched after an error
        assert_eq!(balance_of(&l, "user:buyer").unwrap(), 500);
        assert_eq!(balance_of(&l, "user:seller").unwrap(), 0);
    }

    #[test]
    fn enforces_direction_rules_mint_cannot_gain_burn_cannot_lose() {
        let mut l = new_ledger();
        l = mint_to(&l, "user:buyer", 1_000);
        assert_eq!(balance_of(&l, MINT).unwrap(), -1_000);
        // a credit that pushes mint above zero is rejected
        let bad_mint = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![Posting::new("user:buyer", -1_100), Posting::new(MINT, 1_100)],
                memo: None,
                at: None,
            },
        );
        let e = bad_mint.unwrap_err();
        assert_eq!(e.code, DomainErrorCode::AccountOverdraft);
        assert_eq!(e.meta.get("accountId"), Some(&MetaValue::Text(MINT.into())));
        // receipt by burn is allowed
        let burn_ok = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![Posting::new("user:buyer", -100), Posting::new(BURN, 100)],
                memo: None,
                at: None,
            },
        );
        assert!(burn_ok.is_ok());
        // rules matrix
        assert!(is_overdraft_violation(AccountKind::Mint, 1));
        assert!(is_overdraft_violation(AccountKind::Burn, -1));
        assert!(is_overdraft_violation(AccountKind::User, -1));
        assert!(is_overdraft_violation(AccountKind::Escrow, -1));
        assert!(is_overdraft_violation(AccountKind::Treasury, -1));
        assert!(!is_overdraft_violation(AccountKind::Mint, -1));
        assert!(!is_overdraft_violation(AccountKind::Burn, 1));
    }

    #[test]
    fn replays_idempotently_same_key_same_tx_state_untouched() {
        let mut l = new_ledger();
        l = mint_to(&l, "user:buyer", 500);
        let cmd = PostCommand {
            id: None,
            idempotency_key: Some("order:42:settle".into()),
            postings: vec![Posting::new("user:buyer", -100), Posting::new("user:seller", 100)],
            memo: Some("pay order 42".into()),
            at: None,
        };
        let first = post(&l, &cmd).unwrap();
        assert!(!first.replayed);
        let second = post(&first.state, &cmd).unwrap();
        assert!(second.replayed);
        assert_eq!(second.tx.id, first.tx.id);
        assert_eq!(second.state, first.state); // identical immutable state
        assert_eq!(balance_of(&second.state, "user:buyer").unwrap(), 400);
        assert_eq!(balance_of(&second.state, "user:seller").unwrap(), 100);
    }

    #[test]
    fn does_not_dedupe_postings_without_idempotency_key() {
        let mut l = new_ledger();
        l = mint_to(&l, "user:buyer", 500);
        let cmd = PostCommand {
            id: None,
            idempotency_key: None,
            postings: vec![Posting::new("user:buyer", -10), Posting::new("user:seller", 10)],
            memo: None,
            at: None,
        };
        l = post(&l, &cmd).unwrap().state;
        l = post(&l, &cmd).unwrap().state;
        assert_eq!(balance_of(&l, "user:seller").unwrap(), 20);
    }

    #[test]
    fn assigns_unique_transaction_ids() {
        let mut l = new_ledger();
        l = mint_to(&l, "user:buyer", 100);
        let r1 = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![Posting::new("user:buyer", -1), Posting::new("user:seller", 1)],
                memo: None,
                at: None,
            },
        )
        .unwrap();
        let r2 = post(
            &r1.state,
            &PostCommand {
                id: None,
                idempotency_key: None,
                postings: vec![Posting::new("user:buyer", -1), Posting::new("user:seller", 1)],
                memo: None,
                at: None,
            },
        )
        .unwrap();
        assert_ne!(r1.tx.id, r2.tx.id);
        assert_ne!(r2.tx.id, "tx_1");
    }

    /// Deterministic PRNG — an exact mulberry32 port so it stays identical to the TS version.
    #[allow(clippy::many_single_char_names)] // 1:1 port of the reference algorithm — the names are intentional
    #[allow(clippy::unreadable_literal)] // reference PRNG constants — one-to-one with TS
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
    fn mulberry32(seed: u32) -> impl FnMut() -> f64 {
        let mut t = seed;
        move || {
            t = t.wrapping_add(0x6d2b79f5);
            let mut x = t;
            x = (x ^ (x >> 15)).wrapping_mul(x | 1);
            x ^= x.wrapping_add((x ^ (x >> 7)).wrapping_mul(x | 0x3d));
            f64::from(x ^ (x >> 14)) / 4294967296.0
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
    fn survives_300_seeded_random_operations_with_conservation_intact() {
        let mut l = new_ledger();
        for u in ["user:buyer", "user:seller", "user:c"] {
            l = mint_to(&l, u, 300);
        }
        let mut rand = mulberry32(1405);
        let users = ["user:buyer", "user:seller", "user:c"];
        let mut minted_extra: i64 = 0;
        let mut burned_total: i64 = 0;
        for i in 0..300 {
            let from = users[(rand() * users.len() as f64) as usize];
            let mut to = users[(rand() * users.len() as f64) as usize];
            if to == from {
                to = "user:seller";
            }
            let amount = 1 + (rand() * 50.0) as i64;
            let roll = rand();
            let postings = if roll < 0.6 {
                vec![Posting::new(from, -amount), Posting::new(to, amount)]
            } else if roll < 0.8 {
                vec![Posting::new(from, -amount), Posting::new(BURN, amount)]
            } else {
                vec![Posting::new(MINT, -amount), Posting::new(to, amount)]
            };
            let r = post(
                &l,
                &PostCommand {
                    id: None,
                    idempotency_key: None,
                    postings,
                    memo: Some(format!("seeded op {i}")),
                    at: None,
                },
            );
            match r {
                Ok(out) => {
                    l = out.state;
                    assert!(trial_balance(&l).balanced, "op {i} broke conservation");
                    if roll >= 0.8 {
                        minted_extra += amount;
                    }
                    if (0.6..0.8).contains(&roll) {
                        burned_total += amount;
                    }
                }
                Err(e) => {
                    // a failed operation is a policy error only, never an invariant breach
                    assert!(
                        matches!(
                            e.code,
                            DomainErrorCode::AccountOverdraft
                                | DomainErrorCode::UnknownAccount
                                | DomainErrorCode::UnbalancedTransaction
                        ),
                        "op {i}: {}",
                        e.code
                    );
                }
            }
        }
        assert!(trial_balance(&l).balanced);
        assert_eq!(circulating_supply(&l), 900 + minted_extra - burned_total);
    }

    #[test]
    fn posts_four_leg_affiliate_settlement_with_zero_sum_invariant() {
        let mut l = new_ledger();
        l = add_account(
            &l,
            Account { id: "escrow:aff1".into(), kind: AccountKind::Escrow, label: None },
        )
        .unwrap();
        l = add_account(
            &l,
            Account { id: "user:ref".into(), kind: AccountKind::User, label: None },
        )
        .unwrap();
        l = mint_to(&l, "user:buyer", 1_000);
        // fund escrow
        l = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: Some("fund:aff1".into()),
                postings: vec![
                    Posting::new("user:buyer", -999),
                    Posting::new("escrow:aff1", 999),
                ],
                memo: None,
                at: None,
            },
        )
        .unwrap()
        .state;
        // 4-posting settlement: seller 791 + treasury 99 + referrer 109 = 999
        let out = post(
            &l,
            &PostCommand {
                id: None,
                idempotency_key: Some("release:aff1".into()),
                postings: vec![
                    Posting::new("escrow:aff1", -999),
                    Posting::new("user:seller", 791),
                    Posting::new(TREASURY_ACCOUNT_ID, 99),
                    Posting::new("user:ref", 109),
                ],
                memo: Some("4-leg affiliate release".into()),
                at: None,
            },
        )
        .unwrap();
        assert_eq!(out.tx.postings.len(), 4);
        assert_eq!(balance_of(&out.state, "escrow:aff1").unwrap(), 0);
        assert_eq!(balance_of(&out.state, "user:seller").unwrap(), 791);
        assert_eq!(balance_of(&out.state, TREASURY_ACCOUNT_ID).unwrap(), 99);
        assert_eq!(balance_of(&out.state, "user:ref").unwrap(), 109);
        assert!(trial_balance(&out.state).balanced);
        assert_eq!(circulating_supply(&out.state), 1_000);
    }
}
