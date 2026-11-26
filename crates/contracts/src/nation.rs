//! M5 parliament wire contracts — bill, vote, close and execute (HITL governance).
//!
//! camelCase wire shape with explicit nulls; ids follow the escrow pattern (`bill:…`).
//! Transition logic lives in `kian-nation` — only DTOs with utoipa derives here.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Din;

/// Bill type on the wire — `escrowRelease` / `treasuryMove`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum BillKind {
    /// Escrow dispute resolution — execution means release/refund with `approvedByAdmin: true`.
    EscrowRelease,
    /// Treasury move — execution means a two-sided posting treasury → destination.
    TreasuryMove,
}

/// Proposed outcome of a dispute bill — `release` / `refund`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum EscrowResolution {
    /// 90/10 settlement in the seller’s favor (Article 2).
    Release,
    /// Full refund to the buyer, tax-free.
    Refund,
}

/// Bill-opening request — `createBillRequest`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBillRequest {
    /// bill type
    pub kind: BillKind,
    /// subject: escrow id or destination account (1..=128)
    #[schema(max_length = 128)]
    pub subject: String,
    /// amount in Din — required for `EscrowRelease`; required and > 0 for `TreasuryMove`
    #[serde(default)]
    #[schema(nullable = false)]
    pub amount_din: Option<Din>,
    /// proposed outcome — required only for `EscrowRelease`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<EscrowResolution>,
    /// bill title (1..=200)
    #[schema(max_length = 200)]
    pub title: String,
}

/// Vote request — the voter comes from the bearer session (identity never from the body).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CastVoteRequest {
    /// in favor?
    pub approve: bool,
}

/// Bill view on the wire — `billView`; counts are transparent (Article 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BillView {
    /// bill id (`bill:…`)
    pub id: String,
    /// bill type
    pub kind: BillKind,
    /// subject (escrow/account)
    pub subject: String,
    /// amount in Din (0 if amountless)
    pub amount_din: i64,
    /// proposed outcome — dispute resolution only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<EscrowResolution>,
    /// title
    pub title: String,
    /// status: `OPEN` / `PASSED` / `REJECTED` / `EXECUTED`
    pub status: String,
    /// deterministic reason after closing: `passed` / `quorum_not_met` / `majority_no` / `human_approval_missing`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reason: Option<String>,
    /// opened at
    pub opened_at: String,
    /// closed at — explicit null until adjudication
    pub closed_at: Option<String>,
    /// executed at — explicit null until execution
    pub executed_at: Option<String>,
    /// votes in favor
    pub votes_yes: i64,
    /// votes against
    pub votes_no: i64,
    /// human votes in favor (Article 7 gate)
    pub human_votes_yes: i64,
    /// does Article 7 make this bill human-required? (amount ≥ 50,000 Đ)
    pub hitl_required: bool,
}

/// Bill list response — `billListResponse`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BillListResponse {
    /// bills in chronological order (newest first)
    pub items: Vec<BillView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn bill_wire_shape_is_camel_case() {
        let json = r#"{"kind":"escrowRelease","subject":"esc:1","amountDin":60000,"resolution":"release","title":"Dispute resolution"}"#;
        let req: CreateBillRequest = from_str(json).unwrap();
        assert_eq!(req.kind, BillKind::EscrowRelease);
        assert_eq!(req.resolution, Some(EscrowResolution::Release));
        assert_eq!(req.amount_din.map(Din::get), Some(60_000));
        assert_eq!(to_string(&req).unwrap(), json);
    }

    #[test]
    fn bill_view_omits_absent_optionals() {
        let view = BillView {
            id: "bill:1".into(),
            kind: BillKind::TreasuryMove,
            subject: "citizen:7".into(),
            amount_din: 10_000,
            resolution: None,
            title: "\u{62c}\u{627}\u{628}\u{647}‌\u{62c}\u{627}\u{6cc}\u{6cc} \u{62e}\u{632}\u{627}\u{646}\u{647}".into(),
            status: "OPEN".into(),
            decision_reason: None,
            opened_at: "2026-11-21T00:00:00.000Z".into(),
            closed_at: None,
            executed_at: None,
            votes_yes: 2,
            votes_no: 1,
            human_votes_yes: 1,
            hitl_required: false,
        };
        let s = to_string(&view).unwrap();
        assert!(!s.contains("resolution"), "resolution absent, not null");
        assert!(s.contains("\"closedAt\":null"), "closedAt explicit null");
        assert!(!s.contains("\"decisionReason\""));
    }
}
