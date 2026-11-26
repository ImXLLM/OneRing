//! `kian-ai` — the Kian AI gateway — the M4 brain.
//!
//! Three independent, testable layers:
//! 1. **redaction** — deterministic input scan: hex private keys, bot tokens, PEM, JWT,
//!    mnemonics, long hex — no secret ever reaches the external provider;
//!    only the finding category is reported, never the secret itself.
//! 2. **budget** — a hard budget envelope: today’s spend + the skill estimate ≤ the daily cap,
//!    otherwise 429. Integers, no floats, no "guess" side.
//! 3. **fallback pool** — model lanes in order (port of the Hermes `brain.py`): slots and
//!    providers open into a flat pool with the Cartesian product "every model × every key",
//!    the last winner (`last_good`) is tried first next time, the response-quality gate
//!    rejects broken/wrong-language answers, and on total failure the least-harmful available one is rescued.
//!    A broken envelope answer = lane failure.
//!
//! This crate knows nothing about the database or the server’s HTTP — the budget it reads
//! is a number the server brings from the ledger; the gateway only does math.
#![forbid(unsafe_code)]

pub mod budget;
pub mod client;
pub mod compressor;
pub mod free_llm_registry;
pub mod gateway;
pub mod history_repair;
pub mod omni_genome_router;
pub mod opencode_zen;
pub mod redaction;
pub mod tool_parser;

/// Crate id — for smoke tests and logs.
pub const NAME: &str = "kian-ai";

/// Crate version — read from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use budget::{BudgetDecision, BudgetEnvelope};
pub use compressor::{
    estimate_tokens, partition_for_micro_compaction, prune_old_tool_outputs,
    should_trigger_cold_prune, CompressorMessage, ExchangeBlock, MicroCompactionPlan, PruneOutcome,
    COLD_CACHE_MIN_RECLAIM_TOKENS, MIN_PRUNE_CHARS,
};
pub use tool_parser::{
    extract_xml_tool_calls, AccumulatedToolCall, ParsedToolCall, ToolCallDelta,
    ToolCallStreamAccumulator,
};
pub use history_repair::{
    is_settled_stream_arg, is_valid_studio_output, repair_unanswered_tool_calls, studio_defects,
    unanswered_tool_calls, validate_and_build_self_healing_feedback, MAX_A2UI_ATTEMPTS, RepairedMessage,
    StudioDefect, ToolCallDecl, ToolCallTurn, UNANSWERED,
};
pub use gateway::{
    expand_models_times_keys, looks_broken, persian_char_count, strip_fenced_code_blocks,
    AiConfig, AiGateway, AttemptFailure, ChatPacket, ChatResult, CompletionPacket, GatewayError,
    ModelClient, PacketMessage, SlotConfig, BUILDER_MAX_TOKENS, COARENA_BASE_URL,
    OPENROUTER_BASE_URL, ROUTER9_BASE_URL, SLOT_NAMES,
};
pub use redaction::{redact, RedactionFinding, RedactionReport};
pub use omni_genome_router::{
    agent_auto_decision, builtin_deepseek_32_think_profile, builtin_ecosystem_profiles,
    builtin_gemini_aragon_high_profile, builtin_glm_53_flash_max_profile,
    builtin_gpt6_standard_profile, builtin_claude_sonnet_55_extra_profile,
    builtin_sovereign_penta_profiles, compile_prompt_for_gemini_aragon_high,
    compile_prompt_for_model, load_model_genome_from_dir, load_sovereign_penta_registry,
    manual_override, resolve_sovereign_model_alias, route_with_selection_mode,
    sanitize_gemini_aragon_output, build_caveman_directive,
    CavemanIntensity, DualModeDecision, EngineeringWorkload, GenomeError, GenomeLayerId,
    HandoffPacket, ModelGenomeProfile, ModelSelectionMode, RouterModeError,
    SOVEREIGN_MODEL_DIRS, GEMINI_ARAGON_MODEL_ID,
};
pub use opencode_zen::{
    is_retired_model, resolve_active_free_model, FreeModelSpec, ModelProtocol, ZenClientConfig,
    ACTIVE_FREE_MODELS, DEFAULT_USER_AGENT, OPENCODE_LEGACY_ZEN_BASE_URL, OPENCODE_OPENAI_BASE_URL,
    OPENCODE_PUBLIC_API_KEY,
};
pub use free_llm_registry::{
    browser_ready_public_apis, check_model_refusal, classify_http_failure,
    detect_unresolved_placeholder_tool_args, get_free_provider, restore_tool_names,
    sanitize_system_prompt_for_antigravity, sanitize_tool_names_for_opencode, zero_card_waterfall,
    FreeLlmProviderSpec, LaneFailureAction, LaneHealthTracker, PublicApiSpec, SignupTier,
    ANTIGRAVITY_TRIGGER_LINES, FREE_LLM_PROVIDERS, HERMES_IDENTITY_TRIGGER, KIAN_IDENTITY_REWRITE,
    STUDIO_PUBLIC_APIS_120, STUDIO_PUBLIC_APIS_120_COUNT, TIER_ONE_KEYLESS_CHAIN,
    TIER_TWO_NO_CARD_CHAIN,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Single-gateway shortcuts: `BASE_URL`/`API_KEY`/`MODEL` + `FALLBACK_MODEL` must
    /// build the primary and secondary slots on the same gateway and keep tertiary off.
    #[test]
    fn single_gateway_aliases_configure_primary_and_secondary() {
        let mut env: HashMap<&str, String> = HashMap::new();
        env.insert("KIAN_AI_BASE_URL", "https://router.example/v1".into());
        env.insert("KIAN_AI_API_KEY", "test-key".into());
        env.insert("KIAN_AI_MODEL", "alicode/glm-5".into());
        env.insert("KIAN_AI_FALLBACK_MODEL", "oc/deepseek-v4.1-flash-free".into());
        let cfg = AiConfig::from_env_reader(|k| env.get(k).cloned());

        let p = &cfg.slots[0];
        assert_eq!(p.name, "primary");
        assert_eq!(p.url, "https://router.example/v1");
        assert_eq!(p.key, "test-key");
        assert_eq!(p.model, "alicode/glm-5");
        assert!(p.enabled());

        let s = &cfg.slots[1];
        assert_eq!(s.name, "secondary");
        // the key and URL are inherited from the main shortcut — the fallback model from FALLBACK_MODEL
        assert_eq!(s.url, "https://router.example/v1");
        assert_eq!(s.key, "test-key");
        assert_eq!(s.model, "oc/deepseek-v4.1-flash-free");
        assert!(s.enabled());

        assert!(!cfg.slots[2].enabled(), "tertiary must stay disabled");
        assert_eq!(cfg.slots.iter().filter(|s| s.enabled()).count(), 2);
    }

    /// A slot’s dedicated variable takes precedence over the single-gateway shortcut (contractual precedence).
    #[test]
    fn explicit_slot_vars_override_single_gateway_aliases() {
        let mut env: HashMap<&str, String> = HashMap::new();
        env.insert("KIAN_AI_BASE_URL", "https://router.example/v1".into());
        env.insert("KIAN_AI_API_KEY", "alias-key".into());
        env.insert("KIAN_AI_MODEL", "alias-model".into());
        env.insert("KIAN_AI_PRIMARY_URL", "https://direct.example/v1".into());
        env.insert("KIAN_AI_PRIMARY_KEY", "direct-key".into());
        let cfg = AiConfig::from_env_reader(|k| env.get(k).cloned());

        assert_eq!(cfg.slots[0].url, "https://direct.example/v1");
        assert_eq!(cfg.slots[0].key, "direct-key");
        assert_eq!(
            cfg.slots[0].model, "alias-model",
            "explicit URL/KEY must not clear alias MODEL"
        );
        assert!(!cfg.slots[1].enabled(), "no FALLBACK_MODEL → secondary stays off");
    }

    /// With no variables at all — fail-closed: all three slots off and the default cap.
    #[test]
    fn empty_env_stays_fail_closed() {
        let cfg = AiConfig::from_env_reader(|_| None);
        assert!(cfg.slots.iter().all(|s| !s.enabled()));
        assert_eq!(cfg.daily_cap_tokens, 50_000);
    }

    /// Combining the three layers: a secret gets redacted, the budget is hard, fallback is ordered —
    /// the whole gateway contract in one end-to-end scenario.
    #[tokio::test]
    async fn gateway_end_to_end_redacts_then_falls_back() {
        // first slot broken, second slot healthy — the text contains a private key
        let scripted = client::ScriptedClient::new(vec![
            Err("slot1 is down".into()), // attempt 1 → primary slot failure
            Ok(CompletionPacket {
                content: "\u{67e}\u{627}\u{633}\u{62e} \u{645}\u{63a}\u{632} \u{6a9}\u{6cc}\u{627}\u{646}".into(),
                prompt_tokens: 10,
                completion_tokens: 20,
            }),
        ]);
        let cfg = AiConfig {
            slots: vec![
                SlotConfig {
                    name: "primary".into(),
                    url: "http://slot1".into(),
                    key: "k1".into(),
                    model: "m1".into(),
                },
                SlotConfig {
                    name: "secondary".into(),
                    url: "http://slot2".into(),
                    key: "k2".into(),
                    model: "m2".into(),
                },
            ],
            daily_cap_tokens: 1_000,
        };
        let gw = AiGateway::new(cfg, std::sync::Arc::new(scripted));
        // the raw text (containing a private key) goes straight to the gateway — redaction is the
        // gateway’s own job and the findings are reported in the response, not the secret.
        let raw = "\u{6a9}\u{644}\u{6cc}\u{62f} \u{645}\u{646} 0xaaaa1111bbbb2222cccc3333dddd4444eeee5555ffff6666aaaabbbbccccdddd \u{627}\u{633}\u{62a}";
        let standalone = redact(raw);
        assert_eq!(standalone.findings.len(), 1, "private key hex must be found");
        assert_eq!(standalone.findings[0].kind, "private_key_hex");
        let packet = vec![PacketMessage { role: "user".into(), content: raw.to_owned() }];
        let res = gw.chat(packet).await.expect("slot 2 must serve");
        assert_eq!(res.provider, "secondary");
        assert_eq!(res.redactions, vec!["private_key_hex".to_owned()]);
        assert_eq!(res.total_tokens, 30);
        assert_ne!(res.content, "", "content must not be empty");
    }

    #[tokio::test]
    async fn multi_turn_chat_with_max_tokens_preserves_html_and_redacts_user() {
        let html_reply = "<!DOCTYPE html><html><head><style>body{color:#0ff}</style></head><body><script>const x=1;</script></body></html>";
        let scripted = client::ScriptedClient::new(vec![Ok(CompletionPacket {
            content: html_reply.into(),
            prompt_tokens: 120,
            completion_tokens: 380,
        })]);
        let cfg = AiConfig {
            slots: vec![SlotConfig {
                name: "primary".into(),
                url: "http://slot1".into(),
                key: "k1".into(),
                model: "m1".into(),
            }],
            daily_cap_tokens: 10_000,
        };
        let gw = AiGateway::new(cfg, std::sync::Arc::new(scripted));
        let msgs = vec![
            PacketMessage {
                role: "system".into(),
                content: "build an interactive single page html web application with clean dark neon layout".into(),
            },
            PacketMessage {
                role: "assistant".into(),
                content: html_reply.into(),
            },
            PacketMessage {
                role: "user".into(),
                content: "\u{631}\u{646}\u{6af} \u{62f}\u{6a9}\u{645}\u{647} \u{631}\u{627} \u{633}\u{628}\u{632} \u{6a9}\u{646} \u{648} \u{627}\u{6cc}\u{646} \u{62a}\u{648}\u{6a9}\u{646} 123456789:AAHdqTcvbCHB1WUqptGffZfjHlXgMbGfRcE \u{631}\u{627} \u{646}\u{6af}\u{630}\u{627}\u{631}".into(),
            },
        ];
        let res = gw
            .chat_with_max_tokens(msgs, Some(BUILDER_MAX_TOKENS))
            .await
            .expect("builder turn");
        assert_eq!(res.content, html_reply, "HTML/JS/CSS output must remain intact");
        assert_eq!(res.redactions, vec!["bot_token".to_owned()]);
        assert_eq!(res.total_tokens, 500);
    }
}
