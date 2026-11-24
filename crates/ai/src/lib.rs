//! `kian-ai` — گیت‌وی AI کیان — مغز M4.
//!
//! سه لایهٔ مستقل و تست‌پذیر:
//! ۱. **redaction** — اسکن قطعی ورودی: کلید خصوصی هگز، توکن بات، PEM، JWT،
//!    عبارت حافظه (mnemonic)، هگز بلند — هیچ رازی به ارائه‌دهندهٔ بیرونی
//!    نمی‌رسد؛ فقط دستهٔ یافته گزارش می‌شود، نه خودِ راز.
//! ۲. **budget** — پاکت سخت بودجه: مصرف امروز + برآورد مهارت ≤ سقف روزانه،
//!    وگرنه ۴۲۹. عدد صحیح، بدون float، بدون سمتِ «حدس».
//! ۳. **استخر فال‌بک** — لِین‌های مدل به ترتیب (پورت `brain.py` هرمس): اسلات‌ها و
//!    تامین‌کننده‌ها با ضرب دکارتیِ «هر مدل × هر کلید» در استخر تخت باز می‌شوند،
//!    برندهٔ آخر (`last_good`) دفعهٔ بعد اول امتحان می‌شود، گیت کیفیت پاسخ
//!    خراب/اشتباه‌زبان را رد می‌کند و کم‌ضررترینِ موجود در همه‌جا-شکست نجات می‌یابد.
//!    پاسخ پاکتِ خراب = شکست لِین.
//!
//! این کرِیت هیچ چیز از دیتابیس و HTTP سرور نمی‌داند — بودجه‌ای که خوانده
//! می‌شود عددی است که سرور از دفتر می‌آورد؛ گیت‌وی فقط ریاضی می‌کند.
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

/// شناسه کرِیت — برای smoke test و لاگ‌ها.
pub const NAME: &str = "kian-ai";

/// نسخه کرِیت — از `Cargo.toml` خوانده می‌شود.
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
    builtin_ecosystem_profiles, builtin_gpt6_standard_profile, compile_prompt_for_model,
    load_model_genome_from_dir, EngineeringWorkload, GenomeError, GenomeLayerId, HandoffPacket,
    ModelGenomeProfile,
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

    /// میان‌برهای تک‌گیت‌وی: `BASE_URL`/`API_KEY`/`MODEL` + `FALLBACK_MODEL` باید
    /// اسلات primary و secondary را روی همان گیت‌وی بسازند و tertiary خاموش بماند.
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
        // کلید و URL از میان‌برهای اصلی به ارث می‌رسد — مدل جایگزین از FALLBACK_MODEL
        assert_eq!(s.url, "https://router.example/v1");
        assert_eq!(s.key, "test-key");
        assert_eq!(s.model, "oc/deepseek-v4.1-flash-free");
        assert!(s.enabled());

        assert!(!cfg.slots[2].enabled(), "tertiary must stay disabled");
        assert_eq!(cfg.slots.iter().filter(|s| s.enabled()).count(), 2);
    }

    /// متغیر اختصاصیِ اسلات بر میان‌بر تک‌گیت‌وی مقدم است (اولویت قراردادی).
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

    /// بدون هیچ متغیری — fail-closed: هر سه اسلات خاموش و سقف پیش‌فرض.
    #[test]
    fn empty_env_stays_fail_closed() {
        let cfg = AiConfig::from_env_reader(|_| None);
        assert!(cfg.slots.iter().all(|s| !s.enabled()));
        assert_eq!(cfg.daily_cap_tokens, 50_000);
    }

    /// ترکیب سه لایه: راز redact می‌شود، بودجه سخت است، فال‌بک ترتیبی است —
    /// کل قرارداد گیت‌وی در یک سناریوی سرتاسری.
    #[tokio::test]
    async fn gateway_end_to_end_redacts_then_falls_back() {
        // اسلات اول خراب، اسلات دوم سالم — متن حاوی کلید خصوصی
        let scripted = client::ScriptedClient::new(vec![
            Err("slot1 is down".into()), // تلاش ۱ → شکست اسلات primary
            Ok(CompletionPacket {
                content: "پاسخ مغز کیان".into(),
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
        // متن خام (حاوی کلید خصوصی) مستقیم به گیت‌وی می‌رود — redaction کار خودِ
        // گیت‌وی است و یافته‌ها در پاسخ گزارش می‌شوند، نه راز.
        let raw = "کلید من 0xaaaa1111bbbb2222cccc3333dddd4444eeee5555ffff6666aaaabbbbccccdddd است";
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
                content: "رنگ دکمه را سبز کن و این توکن 123456789:AAHdqTcvbCHB1WUqptGffZfjHlXgMbGfRcE را نگذار".into(),
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
