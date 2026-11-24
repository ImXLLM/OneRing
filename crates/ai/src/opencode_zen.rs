//! `opencode_zen` — درگاه صفر-کارت OpenCode Zen (`crates/ai`, Wave 2 — Step 4/6).
//!
//! تقطیر از `anomalyco/opencode` (`v2.4.1`, commit `18ff071`):
//! - `packages/core/src/plugin/provider/opencode.ts` — لِین ناشناس با
//!   `Authorization: Bearer public` وقتی هیچ کلیدی تنظیم نشده.
//! - `packages/core/src/models-dev/snapshot.txt` — مدل‌های فعال رایگان در برابر
//!   مدل‌های بازنشستهٔ deprecated.
//! - درگاه فعال `https://opencode.ai/inference/openai/v1` با فال‌بک legacy
//!   `https://opencode.ai/zen/v1`.
//!
//! این ماژول فقط پیکربندی و قواعد قطعی است — هیچ IO شبکه‌ای ندارد؛
//! dispatch واقعی توسط آبشار `free_llm_registry` انجام می‌شود.

use std::collections::BTreeMap;

/// درگاه استنتاج فعال OpenCode (پروتکل OpenAI Chat Completions).
pub const OPENCODE_OPENAI_BASE_URL: &str = "https://opencode.ai/inference/openai/v1";

/// درگاه legacy Zen — فال‌بک وقتی درگاه اصلی ۴۰۴/۵۰۲ داد.
pub const OPENCODE_LEGACY_ZEN_BASE_URL: &str = "https://opencode.ai/zen/v1";

/// کلید ناشناس عمومی — از `OpencodePlugin` وقتی `OPENCODE_API_KEY` خالی است.
/// توکنِ sentinel عمومی است، راز نیست و در گیت راز مجاز است.
pub const OPENCODE_PUBLIC_API_KEY: &str = "public";

/// UA مرجع که گیت OpenCode برای لِین ناشناس انتظار دارد.
pub const DEFAULT_USER_AGENT: &str = "opencode/latest/2.4.1/opencode";

/// پروتکل پیام‌رسانی مدل.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelProtocol {
    OpenAiChatCompletions,
    AnthropicMessages,
}

/// مشخصهٔ یک مدل رایگان فعال — از snapshot مدل‌ها.
#[derive(Debug, Clone)]
pub struct FreeModelSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub protocol: ModelProtocol,
    pub context_limit: u32,
    pub output_limit: u32,
    pub supports_images: bool,
    pub reasoning_field: Option<&'static str>,
}

/// ۱۰ مدل صفر-هزینهٔ فعال (snapshot `v2.4.1`) — همگی با `Bearer public`.
pub const ACTIVE_FREE_MODELS: &[FreeModelSpec] = &[
    FreeModelSpec {
        id: "beam-free",
        display_name: "Beam Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 1_048_576,
        output_limit: 131_072,
        supports_images: false,
        reasoning_field: Some("reasoning_content"),
    },
    FreeModelSpec {
        id: "big-pickle",
        display_name: "Big Pickle",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 200_000,
        output_limit: 32_000,
        supports_images: false,
        reasoning_field: Some("reasoning_content"),
    },
    FreeModelSpec {
        id: "deepseek-v4.1-flash-free",
        display_name: "DeepSeek V4.1 Flash Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 200_000,
        output_limit: 32_000,
        supports_images: true,
        reasoning_field: Some("reasoning_content"),
    },
    FreeModelSpec {
        id: "flint-40b-free",
        display_name: "Flint 40B Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 262_144,
        output_limit: 65_536,
        supports_images: false,
        reasoning_field: Some("reasoning_content"),
    },
    FreeModelSpec {
        id: "hy4-flash-free",
        display_name: "Hy4 Flash Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 1_048_576,
        output_limit: 65_536,
        supports_images: true,
        reasoning_field: None,
    },
    FreeModelSpec {
        id: "iceman-free",
        display_name: "Iceman Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 262_144,
        output_limit: 131_072,
        supports_images: false,
        reasoning_field: None,
    },
    FreeModelSpec {
        id: "mimo-v2.6-flash-free",
        display_name: "MiMo V2.6 Flash Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 200_000,
        output_limit: 32_000,
        supports_images: true,
        reasoning_field: Some("reasoning_content"),
    },
    FreeModelSpec {
        id: "nemotron-3.5-lightning-free",
        display_name: "Nemotron 3.5 Lightning Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 262_144,
        output_limit: 262_144,
        supports_images: false,
        reasoning_field: Some("reasoning_content"),
    },
    FreeModelSpec {
        id: "nemotron-3.5-super-vl-free",
        display_name: "Nemotron 3.5 Super VL Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 1_048_576,
        output_limit: 131_072,
        supports_images: true,
        reasoning_field: Some("reasoning_content"),
    },
    FreeModelSpec {
        id: "tidepool-alpha-free",
        display_name: "Tidepool Alpha Free",
        protocol: ModelProtocol::OpenAiChatCompletions,
        context_limit: 1_048_576,
        output_limit: 131_072,
        supports_images: false,
        reasoning_field: Some("reasoning_content"),
    },
];

/// مدل‌های بازنشسته که پیش از ضربه‌زدن به درگاه باید بلاک شوند
/// (`410 Gone` برای longcat، `400` برای glm و deepseek قدیمی،
/// `402 Payment Required` برای sambanova پس از پایان دورهٔ رایگان).
pub fn is_retired_model(model_id: &str) -> bool {
    let id = model_id.trim().trim_start_matches("oc/").to_ascii_lowercase();
    matches!(
        id.as_str(),
        "longcat-2.5-free"
            | "longcat-2.0-free"
            | "glm-5.3-flash-free"
            | "glm-4.7-free"
            | "deepseek-v4-flash-free"
            | "kimi-k2.5-free"
            | "qwen3.6-plus-free"
            | "sambanova"
    )
}

/// گاتِ بازنشسته → مدل فعال جایگزین — هرگز به درگاه نمی‌رسد.
#[must_use]
pub fn resolve_active_free_model(requested_id: &str) -> &'static str {
    match requested_id.trim().trim_start_matches("oc/").to_ascii_lowercase().as_str() {
        "longcat-2.0-free" | "longcat-2.5-free" => "beam-free",
        "glm-4.7-free" | "glm-5.3-flash-free" => "big-pickle",
        "deepseek-v4-flash-free" => "deepseek-v4.1-flash-free",
        "kimi-k2.5-free" | "qwen3.6-plus-free" => "deepseek-v4.1-flash-free",
        "sambanova" => "nemotron-3.5-lightning-free",
        other => ACTIVE_FREE_MODELS
            .iter()
            .find(|m| m.id == other)
            .map_or("beam-free", |m| m.id),
    }
}

/// پیکربندی کلاینت Zen — فال‌بک ناشناس داخلی، بدون هیچ env اجباری.
#[derive(Debug, Clone)]
pub struct ZenClientConfig {
    pub api_key: String,
    pub user_agent: String,
    pub primary_base_url: String,
}

impl Default for ZenClientConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENCODE_API_KEY")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| OPENCODE_PUBLIC_API_KEY.to_string()),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            primary_base_url: OPENCODE_OPENAI_BASE_URL.to_string(),
        }
    }
}

impl ZenClientConfig {
    /// هدرهای ناشناس: `Bearer public` + UA مرجع + JSON.
    #[must_use]
    pub fn build_headers(&self) -> BTreeMap<String, String> {
        let mut headers = BTreeMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));
        headers.insert("User-Agent".to_string(), self.user_agent.clone());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }

    /// URL درگاه فعال `…/chat/completions`.
    #[must_use]
    pub fn chat_completions_url(&self) -> String {
        format!("{}/chat/completions", self.primary_base_url.trim_end_matches('/'))
    }

    /// URL legacy `/zen/v1/chat/completions` — فال‌بک درگاه اصلی.
    #[must_use]
    pub fn legacy_chat_completions_url() -> String {
        format!("{OPENCODE_LEGACY_ZEN_BASE_URL}/chat/completions")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_active_models_registered_with_expected_anchors() {
        assert_eq!(ACTIVE_FREE_MODELS.len(), 10);
        let ids: Vec<&str> = ACTIVE_FREE_MODELS.iter().map(|m| m.id).collect();
        for required in [
            "beam-free",
            "big-pickle",
            "deepseek-v4.1-flash-free",
            "flint-40b-free",
            "hy4-flash-free",
            "iceman-free",
            "mimo-v2.6-flash-free",
            "nemotron-3.5-lightning-free",
            "nemotron-3.5-super-vl-free",
            "tidepool-alpha-free",
        ] {
            assert!(ids.contains(&required), "missing active model {required}");
        }
        let beam = ACTIVE_FREE_MODELS.iter().find(|m| m.id == "beam-free").unwrap();
        assert_eq!(beam.context_limit, 1_048_576, "beam-free is 1M ctx");
    }

    #[test]
    fn retired_models_are_blocked_including_sambanova_402() {
        for retired in [
            "longcat-2.5-free",
            "oc/glm-5.3-flash-free",
            "deepseek-v4-flash-free",
            "kimi-k2.5-free",
            "qwen3.6-plus-free",
            "sambanova",
        ] {
            assert!(is_retired_model(retired), "{retired} must be blocked");
        }
        assert!(!is_retired_model("beam-free"));
        assert!(!is_retired_model("deepseek-v4.1-flash-free"));
    }

    #[test]
    fn retired_models_route_to_active_lanes() {
        assert_eq!(resolve_active_free_model("longcat-2.5-free"), "beam-free");
        assert_eq!(resolve_active_free_model("glm-5.3-flash-free"), "big-pickle");
        assert_eq!(resolve_active_free_model("kimi-k2.5-free"), "deepseek-v4.1-flash-free");
        assert_eq!(resolve_active_free_model("sambanova"), "nemotron-3.5-lightning-free");
        assert_eq!(resolve_active_free_model("beam-free"), "beam-free");
        // مدل نامعلوم → پیش‌فرض امن 1M
        assert_eq!(resolve_active_free_model("totally-unknown"), "beam-free");
    }

    #[test]
    fn keyless_headers_use_public_bearer_and_reference_ua() {
        let cfg = ZenClientConfig::default();
        assert_eq!(cfg.api_key, OPENCODE_PUBLIC_API_KEY);
        let headers = cfg.build_headers();
        assert_eq!(headers.get("Authorization").unwrap(), "Bearer public");
        assert_eq!(headers.get("User-Agent").unwrap(), "opencode/latest/2.4.1/opencode");
        assert_eq!(cfg.chat_completions_url(), "https://opencode.ai/inference/openai/v1/chat/completions");
        assert_eq!(
            ZenClientConfig::legacy_chat_completions_url(),
            "https://opencode.ai/zen/v1/chat/completions"
        );
    }
}
