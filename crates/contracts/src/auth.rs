//! M3 identity contracts — Telegram initData authentication, sessions and TON deposits.
//!
//! Identity is never derived from the request body — only from the signed `Telegram.WebApp`
//! `initData` (official HMAC scheme in [`crate::security`]). A session is an
//! opaque token delivered via `Authorization: Bearer`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Din;

/// Telegram authentication request — the full raw `initData` string from `Telegram.WebApp`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TelegramAuthRequest {
    /// `window.Telegram.WebApp.initData` — without any tampering
    pub init_data: String,
}

/// Mini-app user view — a Telegram identity bound to a double-entry ledger account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserView {
    /// Stable ID (`citizen:{telegram_id}`)
    pub id: String,
    /// Telegram numeric ID (string — guard against 2^53−1)
    pub telegram_id: String,
    /// Telegram username without `@` — optional (key absent on the wire)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Ledger account bound to this identity
    pub account_id: String,
    /// Connected TON wallet address — explicit `null` until TON Connect links
    pub ton_address: Option<String>,
    /// Registration time (ISO)
    pub created_at: String,
}

/// Successful authentication response — session token + user view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// Session token (64 hex) — echoed back in `Authorization: Bearer`
    pub token: String,
    /// Token expiry (ISO)
    pub expires_at: String,
    /// Authenticated user
    pub user: UserView,
}

/// Wallet history row on the wire — signed amount (debit negative).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WalletTxView {
    /// Ledger transaction ID
    pub tx_id: String,
    /// Signed amount in Din — negative means leaving the account
    pub amount: i64,
    /// Memo — explicit `null` when not recorded
    pub memo: Option<String>,
    /// Time (ISO)
    pub at: String,
}

/// Full /me status — identity + derived balance + recent history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    /// Authenticated user
    pub user: UserView,
    /// Free Din balance (ledger SUM — ADR-003)
    pub balance: Din,
    /// Din locked in buyer escrows
    pub escrow_locked: Din,
    /// Last 10 rows of my ledger
    pub recent_txs: Vec<WalletTxView>,
}

/// Request to link a TON address to the identity — comes from TON Connect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetTonAddressRequest {
    /// TON wallet address (51-character base EQ/UQ format or friendly UQ..)
    pub address: String,
}

/// Demo/developer login request — dev mode only (`POST /auth/dev-login`).
///
/// Not a replacement for Telegram; only for locally testing the full loop (builder → publish →
/// referral → purchase → escrow → withdrawal) in a browser without Telegram. All fields except
/// `tgUserId` are optional with sensible defaults; `firstName` is display-only and
/// is not stored (identity = `citizen:{tgUserId}`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DevLoginRequest {
    /// Fake Telegram numeric ID — digits only (1..=20 digits); defaults to the builder persona
    pub tg_user_id: String,
    /// Optional fake username — without `@`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Optional display name — for display only; not stored on the identity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// Optional viral start parameter (`p_<productId>_r_<referrer>`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_param: Option<String>,
}

/// Default `DevLoginRequest` — the demo builder persona (1001).
impl Default for DevLoginRequest {
    fn default() -> Self {
        Self {
            tg_user_id: "1001".to_owned(),
            username: None,
            first_name: None,
            start_param: None,
        }
    }
}

/// Default faucet amount — 5,000 KIAN in micro.
pub const FAUCET_DEFAULT_MICRO: i64 = 5_000_000_000;

/// Per-draw faucet cap — 50,000 KIAN in micro.
pub const FAUCET_MAX_MICRO: i64 = 50_000_000_000;

/// Faucet request (`POST /wallet/faucet`) — dev mode only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FaucetRequest {
    /// Amount in micro-KIAN — default 5,000 KIAN; cap 50,000 KIAN
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_micro: Option<i64>,
}

/// TON deposit registration request — user-side proof of transfer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TonDepositCreateRequest {
    /// TON transaction hash (64 hex or base64 — length 8..=128)
    pub tx_hash: String,
    /// Sender address — the same TON Connect wallet
    pub wallet_address: String,
    /// TON amount as a decimal string (e.g. "0.50") — never a float on the wire
    pub amount_ton: String,
}

/// TON deposit view — full round-trip status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TonDepositView {
    /// ID (`dep:{uuid}`)
    pub id: String,
    /// On-chain transaction hash
    pub tx_hash: String,
    /// Sender address
    pub wallet_address: String,
    /// TON amount as a decimal string
    pub amount_ton: String,
    /// `PENDING` / `CONFIRMED` / `REJECTED`
    pub status: String,
    /// Registration time (ISO)
    pub created_at: String,
    /// Decision time (ISO) — explicit `null` until an admin resolves it
    pub confirmed_at: Option<String>,
}

/// List my deposits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TonDepositListResponse {
    /// Rows — newest first
    pub items: Vec<TonDepositView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn me_response_null_ton_address_is_explicit() {
        let me = MeResponse {
            user: UserView {
                id: "citizen:1".into(),
                telegram_id: "1".into(),
                username: None,
                account_id: "citizen:1".into(),
                ton_address: None,
                created_at: "2026-01-01T00:00:00.000Z".into(),
            },
            balance: Din::new(100).unwrap(),
            escrow_locked: Din::new(0).unwrap(),
            recent_txs: vec![],
        };
        let json = to_string(&me).unwrap();
        assert!(json.contains("\"tonAddress\":null"), "explicit null — {json}");
        assert!(!json.contains("username"), "optional absent — {json}");
    }

    #[test]
    fn dev_login_request_defaults_and_camel_case() {
        // empty body → the default builder persona
        let req: DevLoginRequest = from_str("{}").unwrap();
        assert_eq!(req.tg_user_id, "1001");
        assert_eq!(req.username, None);
        // camelCase wire shape + absent fields
        let full = DevLoginRequest {
            tg_user_id: "2002".into(),
            username: Some("buyer_demo".into()),
            first_name: Some("\u{62e}\u{631}\u{6cc}\u{62f}\u{627}\u{631} (Buyer)".into()),
            start_param: Some("p_prod1_r_3003".into()),
        };
        let json = to_string(&full).unwrap();
        assert!(json.contains("\"tgUserId\""), "{json}");
        assert!(json.contains("\"firstName\""), "{json}");
        assert!(json.contains("\"startParam\""), "{json}");
        assert!(json.contains("\"username\""), "{json}");
    }

    #[test]
    fn faucet_request_defaults_to_five_kian() {
        let req: FaucetRequest = from_str("{}").unwrap();
        assert_eq!(req.amount_micro, None);
        let parsed: FaucetRequest =
            from_str(r#"{"amountMicro":5000000000}"#).unwrap();
        assert_eq!(parsed.amount_micro, Some(5_000_000_000));
    }

    #[test]
    fn deposit_view_roundtrip_keeps_null_confirmed_at() {
        let raw = r#"{"id":"dep:1","txHash":"abc","walletAddress":"UQ","amountTon":"0.5","status":"PENDING","createdAt":"2026-01-01T00:00:00.000Z","confirmedAt":null}"#;
        let view: TonDepositView = from_str(raw).unwrap();
        assert_eq!(view.status, "PENDING");
        assert_eq!(view.confirmed_at, None);
    }
}
