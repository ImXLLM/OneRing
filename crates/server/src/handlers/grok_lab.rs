//! Grok Lab handlers — the `/api/v1/grok-lab/*` telemetry surface.
//!
//! Three additive routes on the persistent `UserLabTelemetryLog` store:
//! - `POST /grok-lab/telemetry` — one 10-DNA lab telemetry row (session-gated;
//!   the deterministic client event id is the idempotency key);
//! - `GET /grok-lab/export-sft-dpo` — the combined SFT+DPO JSONL training
//!   export (admin-gated, the same engine as the admin dataset endpoint);
//! - `GET /grok-lab/summary` — cohort-health aggregate (session-gated).
//!
//! Every number and persona string comes from the store or the wire body —
//! no handler fabricates telemetry on its own (single source of truth).

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;

use kian_contracts::grok_lab::{GrokLabSummaryResponse, GrokLabTelemetryRequest};
use kian_contracts::HttpFailure;

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::hermes::load_lab_telemetry_entries;
use crate::handlers::parse_body;
use crate::middleware::{require_admin, require_session};
use crate::state::AppState;

/// The lab summary wire signature.
pub const GROK_LAB_SUMMARY_SIGNATURE: &str = "MAIRON-GROK-LAB";

/// The export REA floor — identical to the admin dataset export default.
const EXPORT_MIN_REA_SCORE: u8 = 85;

/// `POST /grok-lab/telemetry` — persist one 10-DNA telemetry row (session-gated).
///
/// # Errors
/// 401 missing/expired session; 400 malformed body or out-of-range fields; 500 db.
pub async fn insert_telemetry(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    bytes: axum::body::Bytes,
) -> ApiResult<StatusCode> {
    let _user = require_session(&state, &headers).await?;
    let body: GrokLabTelemetryRequest = parse_body(&bytes)?;
    validate_telemetry(&body)?;
    let row = kian_db::UserLabTelemetryLogRow {
        id: body.id.clone(),
        dna_code: body.dna_code.clone(),
        user_handle: body.user_handle.clone(),
        hour_offset: body.hour_offset,
        ux_tier: body.ux_tier.clone(),
        locale: body.locale.clone(),
        grok_persona: body.grok_persona.clone(),
        model_slug: if body.model_slug.trim().is_empty() {
            "05-glm-5.3-flash-max".to_owned()
        } else {
            body.model_slug.clone()
        },
        routing_mode: body.routing_mode.clone(),
        prompt_text: body.prompt_text.clone(),
        thinking_trace_json: if body.thinking_trace_json.trim().is_empty() {
            "{}".to_owned()
        } else {
            body.thinking_trace_json.clone()
        },
        response_text: body.response_text.clone(),
        rea_score: body.rea_score,
        latency_ms: body.latency_ms,
        usable_for_training: body.usable_for_training,
        created_at: kian_db::now_iso(),
    };
    state.store.insert_lab_telemetry_log(&row).await.map_err(ApiFailure::from)?;
    Ok(StatusCode::CREATED)
}

/// `GET /grok-lab/export-sft-dpo` — the combined SFT+DPO JSONL export (admin-gated).
///
/// # Errors
/// 401 missing admin key; 500 db.
pub async fn export_sft_dpo(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResult<String> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    let entries = load_lab_telemetry_entries(&state).await.map_err(ApiFailure::from)?;
    let export =
        kian_hermes::grok_ux_and_lab_engine::export_rlhf_training_dataset_jsonl(&entries, EXPORT_MIN_REA_SCORE);
    let mut combined = export.sft_jsonl;
    if !combined.is_empty() && !combined.ends_with('\n') {
        combined.push('\n');
    }
    combined.push_str(&export.dpo_jsonl);
    Ok(combined)
}

/// `GET /grok-lab/summary` — cohort-health aggregate (session-gated).
///
/// # Errors
/// 401 missing/expired session; 500 db.
pub async fn summary(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResult<Json<GrokLabSummaryResponse>> {
    let _user = require_session(&state, &headers).await?;
    let entries = load_lab_telemetry_entries(&state).await.map_err(ApiFailure::from)?;
    let total = entries.len();
    let avg = if total == 0 {
        0.0
    } else {
        let sum: u64 = entries.iter().map(|e| u64::from(e.rea_score)).sum();
        sum as f64 / total as f64
    };
    let usable = entries.iter().filter(|e| e.usable_for_training).count();
    Ok(Json(GrokLabSummaryResponse {
        signature: GROK_LAB_SUMMARY_SIGNATURE.to_owned(),
        total_entries: total,
        avg_rea_score: (avg * 100.0).round() / 100.0,
        dna_cohorts: 10,
        usable_for_training: usable,
    }))
}

/// Body validation — honest 400s before the store ever sees a row.
fn validate_telemetry(body: &GrokLabTelemetryRequest) -> Result<(), ApiFailure> {
    fn invalid(field: &str, reason: &str) -> ApiFailure {
        let mut failure = HttpFailure::invalid_request(1);
        failure.message = format!("Invalid grok-lab telemetry — {field}: {reason}");
        ApiFailure::Http(failure)
    }
    let id_ok = (body.id.starts_with("evt:") || body.id.starts_with("lab:")) && body.id.len() >= 8 && body.id.len() <= 128;
    if !id_ok {
        return Err(invalid("id", "must be `evt:fp:…` or `lab:DNA:…`, 8-128 chars"));
    }
    let dna_ok = body.dna_code.starts_with("DNA_") && body.dna_code.len() >= 6 && body.dna_code.len() <= 64;
    if !dna_ok {
        return Err(invalid("dnaCode", "must be a DNA_01..DNA_10 cohort code"));
    }
    if !(0..=23).contains(&body.hour_offset) {
        return Err(invalid("hourOffset", "must be 0..=23"));
    }
    if !(0..=100).contains(&body.rea_score) {
        return Err(invalid("reaScore", "must be 0..=100"));
    }
    if body.latency_ms < 0 {
        return Err(invalid("latencyMs", "must be non-negative"));
    }
    if body.prompt_text.trim().is_empty() || body.response_text.trim().is_empty() {
        return Err(invalid("promptText/responseText", "prompt and response text are required"));
    }
    Ok(())
}

// ── tests — offline (pure validation + identity of the signature) ────────────

#[cfg(test)]
mod tests {
    use super::*;
    use kian_contracts::grok_lab::GrokLabTelemetryRequest;

    fn sample() -> GrokLabTelemetryRequest {
        serde_json::from_value(serde_json::json!({
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
        }))
        .expect("parses")
    }

    /// A well-formed payload passes validation.
    #[test]
    fn valid_payload_passes() {
        assert!(validate_telemetry(&sample()).is_ok());
    }

    /// Unknown cohort shape → honest 400.
    #[test]
    fn bad_dna_code_is_rejected() {
        let mut bad = sample();
        bad.dna_code = "WAVE99".to_owned();
        assert!(validate_telemetry(&bad).is_err());
    }

    /// Hour offset beyond the 24-tick day → honest 400.
    #[test]
    fn hour_offset_out_of_range_is_rejected() {
        let mut bad = sample();
        bad.hour_offset = 24;
        assert!(validate_telemetry(&bad).is_err());
        bad.hour_offset = -1;
        assert!(validate_telemetry(&bad).is_err());
    }

    /// REA score outside 0..=100 → honest 400.
    #[test]
    fn rea_score_out_of_range_is_rejected() {
        let mut bad = sample();
        bad.rea_score = 101;
        assert!(validate_telemetry(&bad).is_err());
        bad.rea_score = -5;
        assert!(validate_telemetry(&bad).is_err());
    }

    /// Empty prompt or response → honest 400.
    #[test]
    fn empty_texts_are_rejected() {
        let mut bad = sample();
        bad.prompt_text = "   ".to_owned();
        assert!(validate_telemetry(&bad).is_err());
        bad.prompt_text = "ok".to_owned();
        bad.response_text = String::new();
        assert!(validate_telemetry(&bad).is_err());
    }

    /// Non-`evt:`/`lab:` ids are rejected (the idempotency contract).
    #[test]
    fn bad_event_ids_are_rejected() {
        let mut bad = sample();
        bad.id = "random-id".to_owned();
        assert!(validate_telemetry(&bad).is_err());
        bad.id = "evt:".to_owned();
        assert!(validate_telemetry(&bad).is_err());
    }

    /// The summary signature is locked and the export floor matches the admin engine.
    #[test]
    fn constants_are_locked() {
        assert_eq!(GROK_LAB_SUMMARY_SIGNATURE, "MAIRON-GROK-LAB");
        assert_eq!(EXPORT_MIN_REA_SCORE, 85);
    }
}
