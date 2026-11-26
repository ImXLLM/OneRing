//! M3 market contracts — products, purchase via the 8-state escrow, and my orders.
//!
//! Purchase = escrow creation + funding in «one» atomic command: the buyer's Din is locked and
//! the seller delivers; Article 8 of the charter applies the 10% tax itself in `release`.
//! The delivery payload (`deliveryPayload`) never travels on the public wire.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::{Din, EscrowStatus, Id, IdempotencyKey};
use crate::escrows::EscrowView;

/// Product delivery payload size cap (150 kilobytes for an interactive single-page HTML artifact).
pub const MAX_DELIVERY_PAYLOAD_BYTES: usize = 150_000;

/// Public HTML preview size cap before purchase (50 kilobytes).
pub const MAX_PREVIEW_HTML_BYTES: usize = 50_000;

/// Product listing request for the market.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    /// Title (1..=120 characters)
    pub title: String,
    /// Description (1..=2000 characters)
    pub description: String,
    /// Price in Din — minimum 1
    pub price_din: Din,
    /// Optional category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Safe HTML preview for the product page before purchase (max 50 KB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_html: Option<String>,
    /// Referrer commission rate in basis points (default 1000 = 10%, cap 5000 = 50%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliate_bps: Option<u16>,
    /// Single-page artifact kind (default `"html"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_kind: Option<String>,
    /// Full delivery payload (max 150 KB) — served only to the buyer of that very order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_payload: Option<String>,
}

/// Public product view — no delivery payload (`deliveryPayload`) and no session required.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductView {
    /// Id (`prod:{uuid}`)
    pub id: String,
    /// Seller account (`citizen:{tg_id}`)
    pub seller_id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Price in Din
    pub price_din: Din,
    /// Category — explicit `null` if unset
    pub category: Option<String>,
    /// Public HTML preview
    pub preview_html: String,
    /// Referrer commission rate in basis points (`0..=5000`)
    pub affiliate_bps: u16,
    /// Artifact kind (`html` and more)
    pub artifact_kind: String,
    /// Whether it is purchasable
    pub active: bool,
    /// ISO creation time
    pub created_at: String,
    /// Mairon REA score sealed on the listing (`0..=100`; `0` = not yet sealed)
    pub mairon_score: u8,
    /// Visible Mairon badge (empty = not sealed)
    pub mairon_badge: String,
}

/// List of the market's active products.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductListResponse {
    /// Products — newest first
    pub items: Vec<ProductView>,
}

/// Purchase request — escrow creation + buyer Din lock in one atomic command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequest {
    /// Target product
    pub product_id: Id,
    /// Optional referrer (numeric `tg_id` like `"12345"` or `"citizen:12345"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    /// Optional idempotency key — a retry never charges twice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// Full product content delivery response — only for the buyer or the seller of a paid escrow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EscrowDeliveryView {
    /// Escrow id
    pub escrow_id: String,
    /// Related product id
    pub product_id: String,
    /// Product title
    pub title: String,
    /// Artifact kind (`html` and more)
    pub artifact_kind: String,
    /// Current escrow status (`FUNDED` / `DELIVERED` / `RELEASED`)
    pub state: EscrowStatus,
    /// Full paid product content
    pub delivery_payload: String,
}

/// Purchase response — the escrow goes CREATED→FUNDED in the same instant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseResponse {
    /// Funded escrow (FUNDED)
    pub escrow: EscrowView,
    /// Din lock transaction — explicit `null` on a tx-less replay
    pub tx_id: Option<String>,
    /// Was this a replay? (no money was re-locked)
    pub replayed: bool,
}

/// My orders + my products — one view of the whole market presence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MyMarketResponse {
    /// Products I have sold
    pub products: Vec<ProductView>,
    /// Escrows where I am the buyer or the seller
    pub orders: Vec<EscrowView>,
}

// ── Mairon seal and citizens' 5-star votes (Wave 3 — Trinity marketplace gate) ──

/// Two-key confirm-and-publish response (`POST /market/products/{id}/mairon-confirm`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MaironConfirmResponse {
    /// Product id
    pub product_id: String,
    /// Mairon REA score (`0..=100`)
    pub mairon_rea_score: u8,
    /// Visible badge sealed on the listing
    pub mairon_visible_badge: String,
    /// Explicit builder confirmation — the second key of the two-key gate
    pub creator_confirmed: bool,
    /// Was the listing published with the seal?
    pub published: bool,
}

/// Citizen vote submission for a product (`POST /market/products/{id}/reviews`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductReviewSubmitRequest {
    /// Stars — `1..=5` only
    pub stars: u8,
    /// Validated comment (`1..=2000` characters after trim)
    pub comment: String,
}

/// One vote as seen on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductReviewView {
    /// Unique vote id (`rev:{uuid}`)
    pub review_id: String,
    /// Product id
    pub product_id: String,
    /// Voter citizen account (`citizen:{tg_id}`)
    pub reviewer_id: String,
    /// Stars (`1..=5`)
    pub stars: u8,
    /// Comment
    pub comment: String,
    /// Verified purchase (linked to a settled escrow; anti-Sybil)
    pub verified_purchase: bool,
    /// Snapshot of the Mairon REA score at vote time
    pub mairon_score_at_review: u8,
    /// ISO submission time
    pub created_at: String,
}

/// Bayesian 5-star rating summary + histogram + combined rating score
/// (`GET /market/products/{id}/reviews`) — the technical and social layers side by side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductRatingSummaryResponse {
    /// Product id
    pub product_id: String,
    /// Mairon technical score (`0..=100`)
    pub mairon_rea_score: u8,
    /// Visible Mairon badge
    pub mairon_visible_badge: String,
    /// Total vote count
    pub total_reviews: u32,
    /// Verified-purchase vote count
    pub verified_reviews: u32,
    /// Arithmetic mean of stars (`0.0..=5.0`)
    pub average_stars: f64,
    /// Bayesian mean (prior `C = 5.0` and `m = 4.20`)
    pub bayesian_stars: f64,
    /// Exact histogram `[1★..5★]`
    pub star_histogram: Vec<u32>,
    /// Bilingual five-star display
    pub five_star_display: String,
    /// Combined rating score: `0.60 * (bayesian_stars * 20) + 0.40 * mairon_score`
    pub combined_rank_score: f64,
    /// All product votes — newest first
    pub reviews: Vec<ProductReviewView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn product_view_category_null_is_explicit() {
        let p = ProductView {
            id: "prod:1".into(),
            seller_id: "citizen:1".into(),
            title: "\u{62f}\u{633}\u{62a}\u{648}\u{631} \u{67e}\u{62e}\u{62a}".into(),
            description: "\u{631}\u{627}\u{62c}\u{639} \u{628}\u{647} \u{686}\u{6cc}\u{632}\u{6cc}".into(),
            price_din: Din::new(500).unwrap(),
            category: None,
            preview_html: "<h1>\u{62f}\u{645}\u{648}</h1>".into(),
            affiliate_bps: 1_000,
            artifact_kind: "html".into(),
            active: true,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            mairon_score: 92,
            mairon_badge: "⚒️ Mairon REA Certified: 92/100 (Forge-Grade)".into(),
        };
        let json = to_string(&p).unwrap();
        assert!(json.contains("\"category\":null"), "explicit null — {json}");
        assert!(json.contains("\"affiliateBps\":1000"), "affiliateBps — {json}");
        assert!(json.contains("\"artifactKind\":\"html\""), "artifactKind — {json}");
        assert!(json.contains("\"previewHtml\":\"<h1>\u{62f}\u{645}\u{648}</h1>\""), "previewHtml — {json}");
    }

    #[test]
    fn purchase_request_rejects_bad_id() {
        assert!(from_str::<PurchaseRequest>(r#"{"productId":"bad id!"}"#).is_err());
        assert!(from_str::<PurchaseRequest>(r#"{"productId":"prod:1"}"#).is_ok());
        let with_ref: PurchaseRequest =
            from_str(r#"{"productId":"prod:1","referrer":"12345"}"#).unwrap();
        assert_eq!(with_ref.referrer.as_deref(), Some("12345"));
    }
}
