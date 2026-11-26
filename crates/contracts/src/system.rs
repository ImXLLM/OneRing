//! System meta contracts (health and backup) — port of `system.ts`.
//!
//! The LLM fields (ADR-006) only reveal "configuration exists" — never a key,
//! never a base URL. A null model means the AI cycle is disabled.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// `status` literal — always `"ok"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum HealthStatus {
    /// `"ok"`
    #[serde(rename = "ok")]
    Ok,
}

/// `service` literal — always `"kian-kernel-server"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum HealthService {
    /// `"kian-kernel-server"`
    #[serde(rename = "kian-kernel-server")]
    KianKernelServer,
}

/// `healthResponseSchema` — deep liveness: a server that cannot reach its ledger is not healthy.
/// The dashboard chip fields (settings page) are additive and optional so
/// older clients can still validate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    /// always `"ok"`
    pub status: HealthStatus,
    /// always `"kian-kernel-server"`
    pub service: HealthService,
    /// Server version
    pub version: String,
    /// Is an LLM configured? (existence only — never details)
    pub llm_configured: bool,
    /// Model name or null (null = AI disabled — explicit on the wire)
    #[schema(required = true)]
    pub llm_model: Option<String>,
    /// DB chip — `"ok"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db: Option<String>,
    /// Bot chip — `"ok"` or `"disabled"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<String>,
    /// AI chip — `"ok"` or `"disabled"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai: Option<String>,
    /// Process uptime (seconds, float)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: Option<f64>,
}

/// `backupCreatedSchema` — the `POST /system/backup` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BackupCreated {
    /// Absolute snapshot path inside the volume (db/backups/…)
    pub file: String,
    /// Size in bytes
    #[schema(minimum = 0_i64, maximum = 9_007_199_254_740_991_i64)]
    pub size_bytes: i64,
    /// Number of snapshots retained after pruning
    #[schema(minimum = 0_i64, maximum = 9_007_199_254_740_991_i64)]
    pub kept: i64,
    /// Creation time
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn health_literals_and_optional_chips() {
        let json = r#"{"status":"ok","service":"kian-kernel-server","version":"4.4.3","llmConfigured":true,"llmModel":"gpt-4o-mini","db":"ok","bot":"ok","ai":"ok","uptime":1234.5}"#;
        let v: HealthResponse = from_str(json).unwrap();
        assert_eq!(v.status, HealthStatus::Ok);
        assert_eq!(v.service, HealthService::KianKernelServer);
        assert_eq!(to_string(&v).unwrap(), json);
    }

    #[test]
    fn health_without_optional_chips_still_parses_and_serializes_minimally() {
        let json = r#"{"status":"ok","service":"kian-kernel-server","version":"x","llmConfigured":false,"llmModel":null}"#;
        let v: HealthResponse = from_str(json).unwrap();
        assert_eq!(v.llm_model, None);
        assert_eq!(to_string(&v).unwrap(), json);
    }

    #[test]
    fn health_missing_required_fields_is_rejected() {
        // RESPONSE_MISMATCH test — an incomplete body must be rejected (like client.test.ts)
        let bad: Result<HealthResponse, _> = serde_json::from_str(r#"{"status":"ok"}"#);
        assert!(bad.is_err());
    }
}
