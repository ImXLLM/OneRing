//! Wire contracts for the AI HTML product builder studio (Phase 4).
//!
//! All routes in this module work with a citizen Bearer session:
//! `POST /api/v1/builder/sessions`, `GET /api/v1/builder/sessions`,
//! `GET /api/v1/builder/sessions/{id}`, `POST /api/v1/builder/sessions/{id}/chat`
//! and `POST /api/v1/builder/sessions/{id}/publish`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Din;
use crate::market::ProductView;

/// One message in the product builder conversation history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderMessageView {
    /// Incremental message ID
    pub id: i64,
    /// Role (`user` / `assistant`)
    pub role: String,
    /// Message text
    pub content: String,
    /// Recorded time (Unix epoch seconds)
    pub created_at: i64,
}

/// Summary of a session for the user's session list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderSessionSummary {
    /// Session ID (`bldr:{uuid}`)
    pub id: String,
    /// Stage (`discovery` / `ready` / `published`)
    pub stage: String,
    /// Proposed title
    pub title: String,
    /// Proposed sales copy
    pub description: String,
    /// Proposed price in Din
    pub suggested_price: i64,
    /// Referral commission rate (`0..=5000`)
    pub affiliate_bps: u16,
    /// Has the HTML code been generated?
    pub has_html: bool,
    /// Published product ID (if published)
    pub published_product_id: Option<String>,
    /// Creation time (Unix epoch seconds)
    pub created_at: i64,
    /// Last modified time (Unix epoch seconds)
    pub updated_at: i64,
}

/// Full view of a builder session (with message history, HTML code and preview).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderSessionView {
    /// Session ID (`bldr:{uuid}`)
    pub id: String,
    /// Stage (`discovery` / `ready` / `published`)
    pub stage: String,
    /// The proposed product title
    pub title: String,
    /// Sales copy (CAST formula)
    pub description: String,
    /// Proposed price in Din
    pub suggested_price: i64,
    /// Referral commission rate (`0..=5000`)
    pub affiliate_bps: u16,
    /// Generated full single-page HTML code
    pub generated_html: String,
    /// Safe HTML preview version (≤ 50 KB)
    pub preview_html: String,
    /// Product ID published to the market (if published)
    pub published_product_id: Option<String>,
    /// Conversation message history in chronological order
    pub messages: Vec<BuilderMessageView>,
    /// Creation time (Unix epoch seconds)
    pub created_at: i64,
    /// Last modified time (Unix epoch seconds)
    pub updated_at: i64,
}

/// List of the user's builder sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderSessionListResponse {
    /// The user's sessions — newest first
    pub items: Vec<BuilderSessionSummary>,
}

/// Request to create a new studio session (an empty `{}` body is also allowed).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBuilderSessionRequest {
    /// Optional initial title or idea
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_idea: Option<String>,
}

/// Request to send a new message in a builder session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderChatRequest {
    /// User message (1..=4000 characters)
    pub message: String,
    /// Force immediate HTML code generation without extra questions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_generate: Option<bool>,
}

/// Response of one chat turn with the product builder studio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderChatResponse {
    /// AI assistant text reply
    pub reply: String,
    /// Updated session state (including new messages and HTML code)
    pub session: BuilderSessionView,
    /// Was the internal fallback interactive generator run?
    pub used_fallback: bool,
    /// Provider slot name (when using an external LLM)
    pub provider: Option<String>,
    /// Total tokens consumed in this turn
    pub total_tokens: i64,
}

/// One-click publish request for a built product to the Kian market.
///
/// All fields are optional; when omitted, the AI-proposed values stored on the session are used.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublishBuilderRequest {
    /// Optional edit of the product title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional edit of the sales copy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional edit of the price in Din
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_din: Option<Din>,
    /// Optional edit of the referral commission rate (`0..=5000`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliate_bps: Option<u16>,
    /// Optional product category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// One-click publish response — includes the published product and the Telegram referral link parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublishBuilderResponse {
    /// Product registered on the market
    pub product: ProductView,
    /// Builder session ID
    pub session_id: String,
    /// Ready-made parameter for mini-app viral and referral links (`p_{product_id}_r_{tg_id}`)
    pub share_start_param: String,
    /// If the session was already published, `true` and the same product are returned
    pub already_published: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn builder_chat_and_publish_dtos_round_trip() {
        let req: BuilderChatRequest =
            from_str(r#"{"message":"Build a calculator","forceGenerate":true}"#).unwrap();
        assert_eq!(req.force_generate, Some(true));
        let pub_req: PublishBuilderRequest = from_str("{}").unwrap();
        assert_eq!(pub_req, PublishBuilderRequest::default());

        let summary = BuilderSessionSummary {
            id: "bldr:1".into(),
            stage: "ready".into(),
            title: "\u{645}\u{627}\u{634}\u{6cc}\u{646}‌\u{62d}\u{633}\u{627}\u{628}".into(),
            description: "\u{62a}\u{648}\u{636}\u{6cc}\u{62d}".into(),
            suggested_price: 250,
            affiliate_bps: 1_500,
            has_html: true,
            published_product_id: None,
            created_at: 1_700_000_000,
            updated_at: 1_700_000_100,
        };
        let json = to_string(&summary).unwrap();
        assert!(json.contains("\"hasHtml\":true"));
        assert!(json.contains("\"affiliateBps\":1500"));
        assert!(json.contains("\"publishedProductId\":null"));
    }
}
