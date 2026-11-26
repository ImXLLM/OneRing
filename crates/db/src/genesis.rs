//! Genesis plan — pure "net" computation of the legacy migration (no IO) + contract constants.
//!
//! Day-one numbers (genesis audit, Mehr 1405 — re-verify at any time):
//!   466 users · 1,554 transactions · Σ stored balances 187,672.8 Đ
//!   the legacy ledger implied 187,735.8 → a gap of −63.0, fully explained:
//!   9 fifty-credit purchases without transaction rows (−450) + one manual owner top-up (+387).
//!
//! v3 representation (ADR-002/003 — integer money, double-entry):
//!  - one account per legacy user: `user:tg:<id>` (all 466, banned or not)
//!  - exactly "one" genesis transaction with the dedup key `legacy:genesis:v1`:
//!    mint:genesis  −Σ(floor(credits))     (the mint is never positive)
//!    `user_i`        +`floor(credits_i)`      (an explicit floor per citizen)
//!    the genesis note is itself a transparent adjustment row (decision #5): it states exactly
//!    why the ledger starts at 187,672 and not 187,735.

use kian_domain::Din;

/// Genesis dedup key — consumed once via a UNIQUE constraint.
pub const GENESIS_KEY: &str = "legacy:genesis:v1";

/// Fixed genesis transaction ID — for reports and audit.
pub const GENESIS_TX_ID: &str = "tx_legacy_genesis";

/// Expected supply after migration — the phase-2 acceptance number (ADR-009).
pub const EXPECTED_SUPPLY: i64 = 187_672;

/// One `users` row of the legacy database (raw, no interpretation).
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUserRow {
    /// TEXT ID — legacy IDs exceed Int32 (e.g. 5010703739)
    pub user_id: String,
    /// Optional username
    pub username: Option<String>,
    /// Optional name
    pub first_name: Option<String>,
    /// Legacy fractional credits (REAL)
    pub credits: f64,
    /// 1 = banned
    pub is_banned: i64,
}

/// One `products` row of the legacy database (raw).
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProductRow {
    /// Legacy ID (TEXT)
    pub id: String,
    /// Seller (TEXT)
    pub creator_id: String,
    /// Title
    pub title: String,
    /// Optional description
    pub description: Option<String>,
    /// Legacy fractional price
    pub price_credits: f64,
    /// Optional category
    pub category: Option<String>,
    /// Delivery link — the only delivery content that survived from Railway
    pub link: Option<String>,
    /// 1 = active
    pub is_active: i64,
}

/// One citizen in the genesis plan — account + the explicit ADR-002 floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisCitizen {
    /// Account ID — `user:tg:<id>`
    pub account_id: String,
    /// Label — `username` ?? `first_name` ?? `tg:<id>`
    pub label: String,
    /// Floored amount (Din)
    pub din: i64,
}

/// Full genesis plan — the output of the pure computation; executed by `migrate_legacy`.
#[derive(Debug, Clone, PartialEq)]
pub struct GenesisPlan {
    /// All citizens (466 — even zeros; only holders get rows)
    pub citizens: Vec<GenesisCitizen>,
    /// Σ legacy fractional credits (e.g. 187,672.8)
    pub sum_credits: f64,
    /// Σ per-citizen floors = the minted supply (187,672)
    pub sum_din: i64,
    /// Number of wallets with a balance
    pub holders: usize,
    /// Number of banned users (they still get accounts — their history belongs to them)
    pub banned: usize,
    /// Genesis note — itself a transparent adjustment row (decision #5)
    pub memo: String,
}

/// Build the genesis plan from raw legacy rows — a pure function, no clock or IO.
///
/// ADR-002 rounding: `floor(credits)` per citizen; aggregate fractions (0.8 Đ)
/// are dropped at the aggregate level, not per individual — documented in the note.
#[must_use]
#[allow(clippy::cast_possible_truncation)] // floor() before cast — explicit ADR-002 rounding
pub fn plan_genesis(users: &[LegacyUserRow]) -> GenesisPlan {
    let sum_credits: f64 = users.iter().map(|u| u.credits).sum();
    let citizens: Vec<GenesisCitizen> = users
        .iter()
        .map(|u| GenesisCitizen {
            account_id: format!("user:tg:{}", u.user_id),
            label: u
                .username
                .clone()
                .or_else(|| u.first_name.clone())
                .unwrap_or_else(|| format!("tg:{}", u.user_id)),
            din: u.credits.floor() as i64,
        })
        .collect();
    let sum_din: i64 = citizens.iter().map(|c| c.din).sum();
    let holders = citizens.iter().filter(|c| c.din > 0).count();
    let banned = users.iter().filter(|u| u.is_banned == 1).count();
    let memo = format!(
        "legacy genesis — Σ held {sum_din} Đ (floor per citizen from {sum_credits:.1}) — \
         legacy tx log implied 187,735.8; stored balances 187,672.8; gap −63.0 resolved: \
         9×50 purchases without tx rows (−450) + manual owner credit (+387) — genesis audit \u{6f1}\u{6f4}\u{6f0}\u{6f5}"
    );
    GenesisPlan { citizens, sum_credits, sum_din, holders, banned, memo }
}

/// Wire-safe Din for a genesis-plan amount — a helper for tests and reports.
///
/// # Errors
/// Fires if the value crosses the safe 2^53−1 boundary (in practice impossible for the legacy genesis).
pub fn citizen_din(c: &GenesisCitizen) -> kian_domain::DomainResult<Din> {
    Din::new(c.din)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: &str, credits: f64, banned: i64) -> LegacyUserRow {
        LegacyUserRow {
            user_id: id.into(),
            username: None,
            first_name: None,
            credits,
            is_banned: banned,
        }
    }

    #[test]
    fn floors_per_citizen_and_sums_to_expected_supply() {
        // 465 users at 403.0 + one user at 277.8 = Σ 187,672.8 → Σ floor = 187,672
        let mut users: Vec<LegacyUserRow> =
            (0..465).map(|i| user(&format!("u{i}"), 403.0, 0)).collect();
        users.push(user("last", 277.8, 0));
        let plan = plan_genesis(&users);
        assert_eq!(users.len(), 466);
        assert!((plan.sum_credits - 187_672.8).abs() < 1e-9);
        assert_eq!(plan.sum_din, EXPECTED_SUPPLY);
        assert_eq!(plan.holders, 466);
        assert_eq!(plan.banned, 0);
        assert!(plan.memo.contains("Σ held 187672 Đ"));
        assert!(plan.memo.contains("187672.8"));
        assert!(plan.memo.contains("gap −63.0"));
        assert!(plan.memo.contains("\u{6f1}\u{6f4}\u{6f0}\u{6f5}"));
    }

    #[test]
    fn labels_follow_username_then_first_name_then_tg() {
        let users = vec![
            LegacyUserRow {
                user_id: "1".into(),
                username: Some("kia".into()),
                first_name: Some("\u{6a9}\u{6cc}\u{627}".into()),
                credits: 10.9,
                is_banned: 0,
            },
            LegacyUserRow {
                user_id: "2".into(),
                username: None,
                first_name: Some("\u{622}\u{62f}\u{627}".into()),
                credits: 0.0,
                is_banned: 1,
            },
            user("3", 5.0, 0),
        ];
        let plan = plan_genesis(&users);
        assert_eq!(plan.citizens[0].label, "kia");
        assert_eq!(plan.citizens[1].label, "\u{622}\u{62f}\u{627}");
        assert_eq!(plan.citizens[2].label, "tg:3");
        assert_eq!(plan.citizens[0].account_id, "user:tg:1");
        // floor(10.9) = 10 — the fraction dropped per individual
        assert_eq!(plan.citizens[0].din, 10);
        assert_eq!(plan.holders, 2); // A zero-balance citizen is not a hodlr
        assert_eq!(plan.banned, 1);
    }
}
