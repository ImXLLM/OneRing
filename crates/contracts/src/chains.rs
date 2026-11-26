//! M5 four-chain wallet wire contracts — chain metadata, the user deposit address, and
//! USDT deposits (user proof → admin approval → the Article 3 mint at the USDT rate).
//!
//! The same M3 contracts as for TON deposits — this time across four chains; address
//! validation uses official checksums in `kian-wallet` (server-side; the client is never trusted).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// One chain's view — `chainView` (public metadata; no secrets).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChainView {
    /// Chain label: TRX / SOL / TON / BSC
    pub chain: String,
    /// Persian name
    pub fa_name: String,
    /// The canonical USDT contract on this chain
    pub usdt_contract: String,
    /// On-chain USDT unit (6 on TRX/SOL/TON; 18 on BSC)
    pub usdt_onchain_decimals: u32,
}

/// Chain list response — `chainsResponse`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChainsResponse {
    /// All four chains in fixed order
    pub items: Vec<ChainView>,
}

/// Register/update a user's deposit address for one chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetDepositAddressRequest {
    /// Chain: TRX / SOL / TON / BSC
    #[schema(max_length = 4)]
    pub chain: String,
    /// Address — validated server-side with the chain's official checksum
    #[schema(max_length = 96)]
    pub address: String,
}

/// The user's registered address view — `depositAddressView`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressView {
    /// Chain
    pub chain: String,
    /// Address
    pub address: String,
    /// ISO registered-at time
    pub created_at: String,
}

/// My-addresses list response — `myAddressesResponse`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MyAddressesResponse {
    /// The user’s registered addresses (up to four chains)
    pub items: Vec<DepositAddressView>,
}

/// Register a USDT deposit — user proof until admin verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositCreateRequest {
    /// Chain: TRX / SOL / TON / BSC
    #[schema(max_length = 4)]
    pub chain: String,
    /// On-chain transaction hash — UNIQUE on (chain, txHash)
    #[schema(min_length = 8, max_length = 128)]
    pub tx_hash: String,
    /// Sender address — validated by the rules of its chain
    #[schema(max_length = 96)]
    pub from_address: String,
    /// USDT amount as a decimal string with ≤6 decimals (wire scale: micro-USDT)
    #[schema(max_length = 24)]
    pub amount_usdt: String,
}

/// The USDT deposit wire view — `usdtDepositView`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositView {
    /// ID (`udep:…`)
    pub id: String,
    /// Chain
    pub chain: String,
    /// Transaction hash
    pub tx_hash: String,
    /// Sender address
    pub from_address: String,
    /// USDT amount as a decimal string
    pub amount_usdt: String,
    /// PENDING / CONFIRMED / REJECTED
    pub status: String,
    /// ISO registered-at time
    pub created_at: String,
    /// Decision time — explicit null until adjudicated
    pub confirmed_at: Option<String>,
}

/// USDT deposit list response — `usdtDepositListResponse`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositListResponse {
    /// My deposits (the last 50)
    pub items: Vec<UsdtDepositView>,
}

/// One chain's treasury view — only the env-configured address; empty means unset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TreasuryChainView {
    /// Chain
    pub chain: String,
    /// Treasury address — an empty string means unset (transparent, not hidden)
    pub address: String,
    /// Is the address valid under the chain's rules?
    pub configured: bool,
}

/// Treasury transparency response — `treasuryResponse` (Article 6; public).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TreasuryResponse {
    /// Treasury status of all four chains
    pub items: Vec<TreasuryChainView>,
    /// Custody policy — fixed contractual text
    pub movement_policy: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn usdt_deposit_wire_shape_is_camel_case() {
        let json = r#"{"chain":"BSC","txHash":"0xabc123def4567890","fromAddress":"0x55d398326f99059fF775485246999027B3197955","amountUsdt":"12.5"}"#;
        let req: UsdtDepositCreateRequest = from_str(json).unwrap();
        assert_eq!(req.chain, "BSC");
        assert_eq!(req.amount_usdt, "12.5");
        assert_eq!(to_string(&req).unwrap(), json);
    }

    #[test]
    fn usdt_deposit_view_keeps_explicit_null() {
        let view = UsdtDepositView {
            id: "udep:1".into(),
            chain: "TON".into(),
            tx_hash: "abc12345".into(),
            from_address: "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs".into(),
            amount_usdt: "1.5".into(),
            status: "PENDING".into(),
            created_at: "2026-11-21T00:00:00.000Z".into(),
            confirmed_at: None,
        };
        let s = to_string(&view).unwrap();
        assert!(
            s.contains("\"confirmedAt\":null"),
            "explicit null until decided"
        );
    }
}
