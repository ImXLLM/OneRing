//! Grok Lab wire DTOs — the `/api/v1/grok-lab/*` surface.
//!
//! Mirror of `packages/analytics/src/index.ts` (`GrokLabTelemetryPayload`) so
//! the TS analytics client and the kernel share one contract: a telemetry
//! row, a combined SFT/DPO export and a cohort-health summary.

use serde::{Deserialize, Serialize};

/// `POST /grok-lab/telemetry` — one 10-DNA lab telemetry row.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokLabTelemetryRequest {
    /// Deterministic client event id (idempotency key) — `evt:fp:…`.
    pub id: String,
    /// Cohort code — `DNA_01`..`DNA_10`.
    pub dna_code: String,
    /// Cohort user handle.
    #[serde(default)]
    pub user_handle: String,
    /// Hour offset 0..23.
    #[serde(default)]
    pub hour_offset: i64,
    /// UX tier.
    pub ux_tier: String,
    /// Locale tag.
    #[serde(default = "default_locale")]
    pub locale: String,
    /// Grok persona wire id.
    pub grok_persona: String,
    /// Routed model slug.
    #[serde(default)]
    pub model_slug: String,
    /// Routing mode — `manual_override` | `agent_auto_decision`.
    #[serde(default = "default_routing_mode")]
    pub routing_mode: String,
    /// User prompt.
    pub prompt_text: String,
    /// Thinking trace (JSON-encoded on the wire).
    #[serde(default)]
    pub thinking_trace_json: String,
    /// Model response.
    pub response_text: String,
    /// REA score 0..100.
    pub rea_score: i64,
    /// Latency in ms.
    #[serde(default)]
    pub latency_ms: i64,
    /// Training usability flag.
    #[serde(default)]
    pub usable_for_training: bool,
}

fn default_locale() -> String {
    "en".to_owned()
}

fn default_routing_mode() -> String {
    "agent_auto_decision".to_owned()
}

/// `GET /grok-lab/summary` — cohort-health aggregate for lab dashboards.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokLabSummaryResponse {
    /// Wire signature — `MAIRON-GROK-LAB`.
    pub signature: String,
    /// Total telemetry rows currently stored (or the deterministic 24h sim fallback).
    pub total_entries: usize,
    /// Mean REA score across the rows.
    pub avg_rea_score: f64,
    /// The DNA cohort count — exactly 10.
    pub dna_cohorts: usize,
    /// Rows marked usable for training.
    pub usable_for_training: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The telemetry request parses the exact wire shape the TS client sends.
    #[test]
    fn telemetry_request_parses_client_payload() {
        let raw = serde_json::json!({
            "id": "evt:fp:ab12cd34:0007:9e2d1a5c",
            "dnaCode": "DNA_04_PRO_QUANT_TRADER_28YO",
            "userHandle": "quant:pro-trader-28yo",
            "hourOffset": 7,
            "uxTier": "quant-desk",
            "locale": "en",
            "grokPersona": "mairon_sovereign",
            "modelSlug": "05-glm-5.3-flash-max",
            "routingMode": "agent_auto_decision",
            "promptText": "backtest the causal strategy",
            "thinkingTraceJson": "{\"steps\":[]}",
            "responseText": "claim hash attached",
            "reaScore": 91,
            "latencyMs": 1240,
            "usableForTraining": true
        });
        let req: GrokLabTelemetryRequest = serde_json::from_value(raw).expect("parses");
        assert_eq!(req.dna_code, "DNA_04_PRO_QUANT_TRADER_28YO");
        assert_eq!(req.hour_offset, 7);
        assert_eq!(req.rea_score, 91);
        assert!(req.usable_for_training);
    }

    /// Defaults fill in when the client omits optional fields.
    #[test]
    fn telemetry_request_defaults_fill_in() {
        let raw = serde_json::json!({
            "id": "evt:fp:ab12cd34:0001:1a2b3c4d",
            "dnaCode": "DNA_01_CHILD_9YO_TOYSPARK",
            "uxTier": "toyspark",
            "grokPersona": "fun_witty",
            "promptText": "rainbow game",
            "responseText": "one crisp take",
            "reaScore": 95
        });
        let req: GrokLabTelemetryRequest = serde_json::from_value(raw).expect("parses");
        assert_eq!(req.locale, "en");
        assert_eq!(req.routing_mode, "agent_auto_decision");
        assert_eq!(req.hour_offset, 0);
        assert!(!req.usable_for_training);
    }

    /// The summary response carries the locked signature and cohort count.
    #[test]
    fn summary_signature_is_locked() {
        let res = GrokLabSummaryResponse {
            signature: "MAIRON-GROK-LAB".to_owned(),
            total_entries: 32,
            avg_rea_score: 92.5,
            dna_cohorts: 10,
            usable_for_training: 30,
        };
        assert_eq!(res.signature, "MAIRON-GROK-LAB");
        assert_eq!(res.dna_cohorts, 10);
        let encoded = serde_json::to_string(&res).expect("serializes");
        assert!(encoded.contains("MAIRON-GROK-LAB"));
    }
}
