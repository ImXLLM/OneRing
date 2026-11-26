//! Wallet contracts — mint and burn (Charter Articles 3 and 4) — port of `wallet.ts`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::{Din, Id, IdempotencyKey};

/// Mint request — `mintRequestSchema` (the annual cap is enforced in Phase 4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MintRequest {
    /// destination account
    pub account_id: Id,
    /// amount in Din
    pub amount: Din,
    /// optional note (≤200)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(max_length = 200)]
    pub memo: Option<String>,
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// Burn request — `burnRequestSchema` (overdraft is impossible).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BurnRequest {
    /// source account
    pub account_id: Id,
    /// amount in Din
    pub amount: Din,
    /// optional note (≤200)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(max_length = 200)]
    pub memo: Option<String>,
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// Wallet move response — `walletMoveResponseSchema`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WalletMoveResponse {
    /// transaction id
    pub tx_id: String,
    /// account
    pub account_id: String,
    /// balance after the operation
    #[schema(minimum = -9_007_199_254_740_991_i64, maximum = 9_007_199_254_740_991_i64)]
    pub balance: i64,
    /// was this response a replay?
    pub replayed: bool,
}

/// Citizen’s crypto withdrawal registration request (`POST /api/v1/wallet/withdrawals`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateWithdrawalRequest {
    /// destination chain (`TON` / `TRX` / `SOL` / `BSC`)
    pub chain: String,
    /// destination asset (`TON` or `USDT`)
    pub asset: String,
    /// destination wallet address (checksum verified by `kian-wallet`)
    pub destination_address: String,
    /// amount withdrawn in Din (minimum 1)
    pub amount_din: Din,
}

/// Admin’s withdrawal payment confirmation request (`POST /api/v1/wallet/withdrawals/{id}/confirm`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmWithdrawalRequest {
    /// on-chain payment transaction hash (8..=128 characters)
    pub tx_hash: String,
}

/// Wire view of one withdrawal request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalRequestView {
    /// id (`wdr:{uuid}`)
    pub id: String,
    /// The citizen account (`citizen:{tg_id}`)
    pub account_id: String,
    /// chain (`TON`/`TRX`/`SOL`/`BSC`)
    pub chain: String,
    /// asset (`TON`/`USDT`)
    pub asset: String,
    /// destination address
    pub destination_address: String,
    /// amount in Din
    pub amount_din: Din,
    /// payment transaction hash — `null` until confirmed
    pub tx_hash: Option<String>,
    /// status (`PENDING` / `CONFIRMED` / `REJECTED`)
    pub status: String,
    /// creation time (Unix epoch seconds)
    pub created_at: i64,
    /// admin decision time (Unix epoch seconds)
    pub resolved_at: Option<i64>,
}

/// Withdrawal request list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalListResponse {
    /// requests — newest first
    pub items: Vec<WithdrawalRequestView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn mint_request_wire_shape_matches_ts() {
        let json = r#"{"accountId":"user:ali","amount":500,"memo":"welcome","idempotencyKey":"mint-ali-1"}"#;
        let req: MintRequest = from_str(json).unwrap();
        assert_eq!(req.amount, Din::new(500).unwrap());
        assert_eq!(
            req.idempotency_key.as_ref().map(ToString::to_string),
            Some("mint-ali-1".into())
        );
        assert_eq!(to_string(&req).unwrap(), json);
    }

    #[test]
    fn burn_request_omits_optionals() {
        let req = BurnRequest {
            account_id: Id::new("user:x").unwrap(),
            amount: Din::new(250).unwrap(),
            memo: None,
            idempotency_key: None,
        };
        assert_eq!(to_string(&req).unwrap(), r#"{"accountId":"user:x","amount":250}"#);
    }
}
