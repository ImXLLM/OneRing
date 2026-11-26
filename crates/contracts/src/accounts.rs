//! Account/balance contracts — port of `accounts.ts`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::{Din, Id};

/// Register a user account (balance starts at 0) — `createAccountRequestSchema`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    /// Account ID (system prefixes are reserved — checked in the handler in phase 4)
    pub id: Id,
    /// Optional label (≤120)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(max_length = 120)]
    pub label: Option<String>,
}

/// Account view — `accountViewSchema`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    /// Account ID
    pub id: String,
    /// Account type (`user` etc.)
    pub kind: String,
    /// Label (nullable — explicit on the wire)
    #[schema(required = true)]
    pub label: Option<String>,
    /// Balance derived from the ledger (ADR-003) — a safe signed integer
    #[schema(minimum = -9_007_199_254_740_991_i64, maximum = 9_007_199_254_740_991_i64)]
    pub balance: i64,
    /// Creation time (ISO-8601)
    pub created_at: String,
}

/// `accountListResponseSchema`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountListResponse {
    /// Rows
    pub items: Vec<AccountView>,
}

/// `balanceResponseSchema` — public transparency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    /// Account ID
    pub account_id: String,
    /// Balance
    pub balance: Din,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn account_view_keeps_null_label_and_camel_case() {
        let json = r#"{"id":"user:tg:123","kind":"user","label":null,"balance":0,"createdAt":"2026-01-15T10:30:00.000Z"}"#;
        let v: AccountView = from_str(json).unwrap();
        assert_eq!(v.label, None);
        assert_eq!(to_string(&v).unwrap(), json.replace('}', " }").replace(" }", "}"));
    }

    #[test]
    fn create_account_request_omits_absent_label() {
        let req = CreateAccountRequest { id: Id::new("user:tg:1").unwrap(), label: None };
        assert_eq!(to_string(&req).unwrap(), r#"{"id":"user:tg:1"}"#);
        let req2 =
            CreateAccountRequest { id: Id::new("user:tg:1").unwrap(), label: Some("Ali".into()) };
        assert_eq!(to_string(&req2).unwrap(), r#"{"id":"user:tg:1","label":"Ali"}"#);
    }
}
