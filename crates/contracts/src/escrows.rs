//! Escrow contracts — the marketplace backbone (ADR-004, Charter Article 8) — port of `escrows.ts`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::{Din, EscrowStatus, Id, IdempotencyKey};

/// `createEscrowRequestSchema` — the server generates a missing id; duplicates are rejected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateEscrowRequest {
    /// optional id (server-generated; duplicate → error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    /// buyer
    pub buyer_id: Id,
    /// seller
    pub seller_id: Id,
    /// amount in Din
    pub amount: Din,
    /// optional product reference (≤128)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(max_length = 128)]
    pub product_ref: Option<String>,
    /// TTL in hours (1 to 720)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(minimum = 1_i64, maximum = 720_i64)]
    pub ttl_hours: Option<i64>,
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// Escrow view — `escrowViewSchema` (all nullable fields explicit on the wire).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EscrowView {
    /// id
    pub id: String,
    /// buyer
    pub buyer_id: String,
    /// seller
    pub seller_id: String,
    /// amount
    pub amount: Din,
    /// product reference (nullable — explicit on the wire)
    #[schema(required = true)]
    pub product_ref: Option<String>,
    /// status
    pub status: EscrowStatus,
    /// creation time
    pub created_at: String,
    /// fund transaction (nullable)
    #[schema(required = true)]
    pub funded_tx_id: Option<String>,
    /// settle transaction (nullable)
    #[schema(required = true)]
    pub settle_tx_id: Option<String>,
    /// dispute reason (nullable)
    #[schema(required = true)]
    pub dispute_reason: Option<String>,
    /// delivery reference (nullable)
    #[schema(required = true)]
    pub delivery_ref: Option<String>,
    /// expiry time (nullable)
    #[schema(required = true)]
    pub expires_at: Option<String>,
}

/// `escrowListResponseSchema`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EscrowListResponse {
    /// rows
    pub items: Vec<EscrowView>,
}

/// `fundEscrowRequestSchema`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FundEscrowRequest {
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// `deliverEscrowRequestSchema`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeliverEscrowRequest {
    /// optional delivery reference (≤256)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(max_length = 256)]
    pub delivery_ref: Option<String>,
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// `releaseEscrowRequestSchema` — Article 7: for amounts ≥ 50,000 Đ,
/// `approvedByAdmin: true` must be **explicit**; never implied.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseEscrowRequest {
    /// explicit admin approval (Article 7)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by_admin: Option<bool>,
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// `disputeEscrowRequestSchema`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisputeEscrowRequest {
    /// reason (1 to 500 characters)
    #[schema(min_length = 1, max_length = 500)]
    pub reason: String,
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// `refundEscrowRequestSchema`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefundEscrowRequest {
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// `cancelEscrowRequestSchema`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CancelEscrowRequest {
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// `expireEscrowRequestSchema`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExpireEscrowRequest {
    /// optional idempotency key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// `escrowActionResponseSchema` — the response of every escrow action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EscrowActionResponse {
    /// escrow view after the operation
    pub escrow: EscrowView,
    /// The transaction id (nullable — cancel/forge never moves money)
    #[schema(required = true)]
    pub tx_id: Option<String>,
    /// was this a replay?
    pub replayed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    fn sample_view() -> EscrowView {
        EscrowView {
            id: "escrow:01".into(),
            buyer_id: "user:tg:1".into(),
            seller_id: "user:tg:2".into(),
            amount: Din::new(60_000).unwrap(),
            product_ref: None,
            status: EscrowStatus::Released,
            created_at: "2026-02-01T00:00:00.000Z".into(),
            funded_tx_id: Some("tx:02".into()),
            settle_tx_id: Some("tx:03".into()),
            dispute_reason: None,
            delivery_ref: Some("ref-9".into()),
            expires_at: None,
        }
    }

    #[test]
    fn escrow_view_null_fields_stay_explicit() {
        let json = to_string(&sample_view()).unwrap();
        assert!(json.contains(r#""productRef":null"#));
        assert!(json.contains(r#""disputeReason":null"#));
        assert!(json.contains(r#""expiresAt":null"#));
        assert!(json.contains(r#""status":"RELEASED""#));
        let back: EscrowView = from_str(&json).unwrap();
        assert_eq!(back, sample_view());
    }

    #[test]
    fn empty_request_bodies_serialize_to_empty_object() {
        assert_eq!(to_string(&FundEscrowRequest::default()).unwrap(), "{}");
        assert_eq!(to_string(&CancelEscrowRequest::default()).unwrap(), "{}");
    }

    #[test]
    fn release_request_carries_explicit_admin_approval() {
        let json = r#"{"approvedByAdmin":true}"#;
        let req: ReleaseEscrowRequest = from_str(json).unwrap();
        assert_eq!(req.approved_by_admin, Some(true));
        assert_eq!(to_string(&req).unwrap(), json);
    }

    #[test]
    fn escrow_action_response_roundtrip() {
        let resp = EscrowActionResponse {
            escrow: sample_view(),
            tx_id: Some("tx:03".into()),
            replayed: false,
        };
        let json = to_string(&resp).unwrap();
        assert!(json.contains(r#""txId":"tx:03""#));
        assert!(json.contains(r#""replayed":false"#));
    }
}
