//! Economy transparency contracts — charter article 6 — port of `economy.ts`.
//! Read-only and public: the nation can always audit its own money.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Din;

/// Currency unit — literal on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Currency {
    /// `"DNX"`
    #[serde(rename = "DNX")]
    Dnx,
}

/// Currency symbol — literal on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum DinSymbol {
    /// `"Đ"`
    #[serde(rename = "Đ")]
    Dinar,
}

/// `economySummarySchema` — the national economy summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EconomySummary {
    /// Currency unit (always `"DNX"`)
    pub currency: Currency,
    /// Symbol (always `"Đ"`)
    pub symbol: DinSymbol,
    /// Circulating supply
    pub circulating_supply: Din,
    /// Treasury balance
    pub treasury_balance: Din,
    /// Locked in escrows
    pub escrow_locked: Din,
    /// Total minted since genesis
    pub minted_all_time: Din,
    /// Total burned since genesis
    pub burned_all_time: Din,
    /// Minted this year
    pub minted_this_year: Din,
    /// Annual mint cap (article 3)
    pub yearly_mint_cap: Din,
    /// Genesis supply (ADR-002)
    pub genesis_supply: Din,
    /// Trial balance total (must be 0)
    #[schema(minimum = -9_007_199_254_740_991_i64, maximum = 9_007_199_254_740_991_i64)]
    pub trial_balance_sum: i64,
    /// Is the ledger balanced?
    pub balanced: bool,
    /// Account count
    #[schema(minimum = -9_007_199_254_740_991_i64, maximum = 9_007_199_254_740_991_i64)]
    pub account_count: i64,
    /// Open escrows
    #[schema(minimum = -9_007_199_254_740_991_i64, maximum = 9_007_199_254_740_991_i64)]
    pub open_escrows: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn literals_render_exactly_like_zod() {
        let json = r#"{"currency":"DNX","symbol":"Đ","circulatingSupply":187672,"treasuryBalance":127672,"escrowLocked":60000,"mintedAllTime":247672,"burnedAllTime":60000,"mintedThisYear":247672,"yearlyMintCap":10000000,"genesisSupply":187672,"trialBalanceSum":0,"balanced":true,"accountCount":468,"openEscrows":2}"#;
        let v: EconomySummary = from_str(json).unwrap();
        assert_eq!(v.currency, Currency::Dnx);
        assert_eq!(v.symbol, DinSymbol::Dinar);
        assert_eq!(to_string(&v).unwrap(), json);
    }
}
