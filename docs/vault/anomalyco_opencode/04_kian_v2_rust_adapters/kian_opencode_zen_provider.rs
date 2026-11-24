//! KIAN v2 OpenCode Zen / Inference Provider Adapter (`crates/ai` & `crates/hermes`)
//!
//! Distilled from `https://github.com/anomalyco/opencode` (`v2.4.1`, commit `18ff071`):
//! - `packages/core/src/plugin/provider/opencode.ts` (`apiKey: "public"` zero-card lane)
//! - `packages/core/src/models-dev/snapshot.txt` (active vs deprecated free models)
//! - `packages/core/src/plugin/optimize.ts` & `session/system-prompt.ts`

use std::collections::BTreeMap;

pub const OPENCODE_OPENAI_BASE_URL: &str = "https://opencode.ai/inference/openai/v1";
pub const OPENCODE_ANTHROPIC_BASE_URL: &str = "https://opencode.ai/inference/anthropic/v1";
pub const OPENCODE_SYSTEMONE_BASE_URL: &str = "https://opencode.ai/inference/systemone/v1";
pub const OPENCODE_LEGACY_ZEN_BASE_URL: &str = "https://opencode.ai/zen/v1";
pub const OPENCODE_CONSOLE_URL: &str = "https://opencode.ai/console";

/// Anonymous public bearer key injected by `OpencodePlugin` when no `OPENCODE_API_KEY` is configured.
pub const OPENCODE_PUBLIC_API_KEY: &str = "public";
pub const DEFAULT_USER_AGENT: &str = "opencode/latest/2.4.1/opencode";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelProtocol {
    OpenAiChatCompletions,
    AnthropicMessages,
    SystemOneDecisions,
}

#[derive(Debug, Clone)]
pub struct FreeModelSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub protocol: ModelProtocol,
    pub context_limit: u32,
    pub output_limit: u32,
    pub supports_images: bool,
    pub supports_audio_video: bool,
    pub reasoning_field: Option<&'static str>,
    pub strict_tools: bool,
}

/// Active zero-cost models verified in `packages/core/src/models-dev/snapshot.txt` (`v2.4.1`).
pub const ACTIVE_FREE_MODELS: &[FreeModelSpec] = &[
    FreeModelSpec {
        id: "beam-free",
        display_name: "Beam Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 1_048_576,
        output_limit: 131_072,
        supports_images: false,
        supports_audio_video: false,
        reasoning_field: Some("reasoning_content"),
        strict_tools: true,
    },
    FreeModelSpec {
        id: "big-pickle",
        display_name: "Big Pickle",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 200_000,
        output_limit: 32_000,
        supports_images: false,
        supports_audio_video: false,
        reasoning_field: Some("reasoning_content"),
        strict_tools: false,
    },
    FreeModelSpec {
        id: "deepseek-v4.1-flash-free",
        display_name: "DeepSeek V4.1 Flash Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 200_000,
        output_limit: 32_000,
        supports_images: true,
        supports_audio_video: false,
        reasoning_field: Some("reasoning_content"),
        strict_tools: false,
    },
    FreeModelSpec {
        id: "flint-40b-free",
        display_name: "Flint 40B Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 262_144,
        output_limit: 65_536,
        supports_images: false,
        supports_audio_video: false,
        reasoning_field: Some("reasoning_content"),
        strict_tools: false,
    },
    FreeModelSpec {
        id: "hy4-flash-free",
        display_name: "Hy4 flash Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 1_048_576,
        output_limit: 65_536,
        supports_images: true,
        supports_audio_video: false,
        reasoning_field: None,
        strict_tools: false,
    },
    FreeModelSpec {
        id: "iceman-free",
        display_name: "Iceman Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 262_144,
        output_limit: 131_072,
        supports_images: false,
        supports_audio_video: false,
        reasoning_field: None,
        strict_tools: false,
    },
    FreeModelSpec {
        id: "mimo-v2.6-flash-free",
        display_name: "MiMo-V2.6-Flash Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 200_000,
        output_limit: 32_000,
        supports_images: true,
        supports_audio_video: true,
        reasoning_field: Some("reasoning_content"),
        strict_tools: true,
    },
    FreeModelSpec {
        id: "nemotron-3.5-lightning-free",
        display_name: "Nemotron 3.5 Lightning Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 262_144,
        output_limit: 262_144,
        supports_images: false,
        supports_audio_video: false,
        reasoning_field: Some("reasoning_content"),
        strict_tools: false,
    },
    FreeModelSpec {
        id: "nemotron-3.5-super-vl-free",
        display_name: "Nemotron 3.5 Super VL Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 1_048_576,
        output_limit: 131_072,
        supports_images: true,
        supports_audio_video: false,
        reasoning_field: Some("reasoning_content"),
        strict_tools: false,
    },
    FreeModelSpec {
        id: "tidepool-alpha-free",
        display_name: "Tidepool Alpha Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 1_048_576,
        output_limit: 131_072,
        supports_images: false,
        supports_audio_video: false,
        reasoning_field: Some("reasoning_content"),
        strict_tools: false,
    },
];

/// Retired free IDs marked `status: "deprecated"` in `models-dev/snapshot.txt` and replacement routing.
pub fn resolve_active_free_model(requested_id: &str) -> &'static str {
    match requested_id.trim().to_ascii_lowercase().as_str() {
        "longcat-2.0-free" | "longcat-2.5-free" | "longcat-2.5-preview-free" => "beam-free",
        "glm-4.7-free" | "glm-5-free" | "glm-5.3-flash-free" => "big-pickle",
        "deepseek-v4-flash-free" => "deepseek-v4.1-flash-free",
        "mimo-v2-flash-free" | "mimo-v2-omni-free" | "mimo-v2-pro-free" | "mimo-v2.5-free" => {
            "mimo-v2.6-flash-free"
        }
        "nemotron-3-super-free" | "nemotron-3-ultra-free" => "nemotron-3.5-lightning-free",
        "kimi-k2.5-free" | "qwen3.6-plus-free" | "minimax-m3.1-flash-free"
        | "trinity-large-preview-free" | "step-5-preview-free" | "x-preview-f-free" => {
            "deepseek-v4.1-flash-free"
        }
        other => ACTIVE_FREE_MODELS
            .iter()
            .find(|m| m.id == other)
            .map(|m| m.id)
            .unwrap_or("beam-free"),
    }
}

#[derive(Debug, Clone)]
pub struct ZenClientConfig {
    pub api_key: String,
    pub user_agent: String,
    pub org_id: Option<String>,
    pub enable_systemone_images: bool,
}

impl Default for ZenClientConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENCODE_API_KEY")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| OPENCODE_PUBLIC_API_KEY.to_string()),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            org_id: None,
            enable_systemone_images: false,
        }
    }
}

impl ZenClientConfig {
    pub fn build_headers(&self) -> BTreeMap<String, String> {
        let mut h = BTreeMap::new();
        h.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));
        h.insert("User-Agent".to_string(), self.user_agent.clone());
        h.insert("Content-Type".to_string(), "application/json".to_string());
        if let Some(org) = &self.org_id {
            h.insert("x-org-id".to_string(), org.clone());
        }
        if self.enable_systemone_images {
            h.insert("TypeSafe-Danger-Zone-Images".to_string(), "true".to_string());
        }
        h
    }

    pub fn chat_completions_url(&self) -> String {
        format!("{OPENCODE_OPENAI_BASE_URL}/chat/completions")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFamily {
    Claude,
    Gpt,
    GptAstra,
    Kimi,
    Trinity,
    MetaMuse,
    Gemini,
    Qwen,
    DeepSeek,
    OpenWeightDefault,
}

impl ModelFamily {
    pub fn from_model_id(model_id: &str) -> Self {
        let id = model_id.to_ascii_lowercase();
        if id.contains("claude") {
            Self::Claude
        } else if id.contains("gpt") && id.contains("astra") {
            Self::GptAstra
        } else if id.contains("gpt") {
            Self::Gpt
        } else if id.contains("kimi") {
            Self::Kimi
        } else if id.contains("trinity") {
            Self::Trinity
        } else if id.contains("muse") {
            Self::MetaMuse
        } else if id.contains("gemini") {
            Self::Gemini
        } else if id.contains("qwen") {
            Self::Qwen
        } else if id.contains("deepseek") {
            Self::DeepSeek
        } else {
            Self::OpenWeightDefault
        }
    }

    /// Mirrors `packages/core/src/plugin/optimize.ts` + `session/system-prompt.ts`.
    pub fn prompt_directive(&self, tools: &[&str]) -> String {
        let mut tool_guidance = Vec::new();
        if tools.contains(&"shell") {
            tool_guidance.push("- Prefer dedicated tools over shell commands; fall back to shell only when needed.");
        }
        if tools.contains(&"write") {
            tool_guidance.push("- Use write to create/overwrite files; prefer edit for targeted changes.");
        }
        if tools.contains(&"edit") {
            tool_guidance.push("- Use edit for surgical text replacements with unique surrounding context.");
        }
        let guidance = tool_guidance.join("\n");
        match self {
            Self::Claude => "Match surrounding code comment density; add comments sparingly only for non-obvious behavior.".to_string(),
            Self::Gpt | Self::GptAstra => format!(
                "You are an AI coding agent. Parallelize independent tool calls. Keep responses concise; do not revert changes you did not make.\n{guidance}"
            ),
            Self::Kimi | Self::Trinity => format!(
                "Default to taking action with tools. Parallelize non-interfering tool calls. Run lint and typecheck after finishing; never commit unless asked.\n{guidance}"
            ),
            Self::MetaMuse => format!(
                "Be concise and objective; verify every fix by execution before reporting; cite file_path:line_number.\n{guidance}"
            ),
            Self::Gemini | Self::Qwen | Self::DeepSeek | Self::OpenWeightDefault => format!(
                "You are an AI coding agent. Read surrounding files before editing, keep style consistent, and execute verification.\n{guidance}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deprecated_free_models_route_to_active_lanes() {
        assert_eq!(resolve_active_free_model("longcat-2.5-free"), "beam-free");
        assert_eq!(resolve_active_free_model("glm-5.3-flash-free"), "big-pickle");
        assert_eq!(resolve_active_free_model("deepseek-v4-flash-free"), "deepseek-v4.1-flash-free");
        assert_eq!(resolve_active_free_model("mimo-v2.6-flash-free"), "mimo-v2.6-flash-free");
    }

    #[test]
    fn anonymous_zen_headers_include_public_key_and_ua() {
        let cfg = ZenClientConfig {
            api_key: OPENCODE_PUBLIC_API_KEY.to_string(),
            ..ZenClientConfig::default()
        };
        let headers = cfg.build_headers();
        assert_eq!(headers.get("Authorization").unwrap(), "Bearer public");
        assert!(headers.get("User-Agent").unwrap().starts_with("opencode/"));
    }
}
