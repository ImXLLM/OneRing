//! `free_llm_registry` — رجیستری چندتأمین‌کنندهٔ صفر-کارت کیان (`crates/ai`, Wave 2 — Step 4/6).
//!
//! سه بخش:
//! 1. **گاردهای ضد-429 جعلی 9Router (`v0.5.123`)** — سانتی‌سایز نام ابزار
//!    `terminal*` → `shell_exec` (بازگشت‌پذیر) و بازنویسی پرامپت هویتی هرمس برای
//!    Google Antigravity (پورت commit `bb399170`)؛ بدون این دو، لِین‌های رایگان
//!    `429 FreeUsageLimit` جعلی می‌گیرند.
//! 2. **کاتالوگ پلتفرم‌های رایگان** (تقطیر `tashfeenahmed/freellmapi`
//!    `62526e0` + کاتالوگ `2026.11.23.4b4737`): همهٔ ورودی‌ها OpenAI-compatible
//!    و بدون کارت بانکی — آبشار ۳لایهٔ Tier1 ناشناس → Tier2 ثبت‌نام رایگان →
//!    Tier3 کاتالوگ ۱۲۰ API عمومی CORS-دار برای `fetch()` زندهٔ استودیو.
//! 3. **میان‌افزار تاب‌آوری** (`langflow` + `langchain`): کشف آرگومان ابزارِ
//!    placeholder-مانند (`<result-from-…>`/`{{step_…}}`) با hint تلاش یک‌باره،
//!    و چک فیلد refusal پیش از پارس JSON Schema + ردیاب سلامت لِین با
//!    بک‌آف نمایی `2s × 2^level` (سقف ۵ دقیقه).
//!
//! هیچ ورودی این ماژول راز نیست؛ کلیدهای واقعی فقط از env سرور می‌آیند.

use serde_json::Value;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::time::{Duration, Instant};

// ───────────────────── گارد ۱: سانتی‌سایز نام ابزار OpenCode ─────────────────────

/// نام‌های ابزار حاوی `terminal` (غیرحساس به بزرگی حرف) به `shell_exec`
/// بازنویسی می‌شوند — `OpenCode` Free هر ابزار terminal-دار را با
/// `429 FreeUsageLimitError` جعلی رد می‌کند. نقشهٔ بازگشت برای پاسخ برمی‌گردد:
/// `aliased -> original`. برخورد نام‌ها با پسوند `_2`, `_3` … حل می‌شود.
pub fn sanitize_tool_names_for_opencode(tools: &mut [Value]) -> HashMap<String, String> {
    let mut restore_map = HashMap::new();
    let mut taken: HashMap<String, usize> = HashMap::new();
    for tool in tools.iter_mut() {
        let name_opt = tool
            .pointer("/function/name")
            .and_then(Value::as_str)
            .map(str::to_string);
        if let Some(orig) = name_opt.filter(|name| name.to_ascii_lowercase().contains("terminal")) {
            let base = "shell_exec";
            let count = taken.entry(base.to_string()).and_modify(|c| *c += 1).or_insert(0);
            let aliased = if *count == 0 {
                base.to_string()
            } else {
                format!("{base}_{}", *count + 1)
            };
            restore_map.insert(aliased.clone(), orig);
            if let Some(fn_obj) = tool.get_mut("function").and_then(Value::as_object_mut) {
                fn_obj.insert("name".to_string(), Value::String(aliased));
            }
        }
    }
    restore_map
}

/// بازگرداندن نام ابزارها در پاسخ مدل — معکوسِ قطعی `sanitize_tool_names_for_opencode`.
/// `None` یعنی نام دست‌نخورده است و همان را پاس می‌دهیم.
#[must_use]
pub fn restore_tool_names<'m, S: BuildHasher>(
    tool_name_in_response: &str,
    restore_map: &'m HashMap<String, String, S>,
) -> Option<&'m str> {
    restore_map.get(tool_name_in_response).map(String::as_str)
}

// ───────────────────── گارد ۲: پرامپت هویتی Antigravity ─────────────────────

/// هویت هرمس — خطِ محرکِ `429 RESOURCE_EXHAUSTED` جعلی روی Antigravity.
pub const HERMES_IDENTITY_TRIGGER: &str = "You are Hermes Agent, built by Nous Research";
/// بازنویسی کیانی — همان قانونِ `KIA-BRAIN-OS/03_DNAmind.md` (هویت مستقل).
pub const KIAN_IDENTITY_REWRITE: &str = "You are KIAN Autonomous Agent, built by KIAN Systems";

/// خطوط دقیق دیگری که Antigravity را به 429 جعلی می‌کشانند (پورت `bb399170`).
pub const ANTIGRAVITY_TRIGGER_LINES: &[&str] = &[
    "<!-- End Hermes runtime environment -->",
    "# Google model operational directives",
    "If a tool, install, or network call fails and blocks the real path",
    "When the user asks you to build, run, or verify something, the deliverable is a working artifact",
    "For help with Hermes Agent itself",
    "Mid-turn, the user can steer you: Hermes delivers their message",
];

/// سانتی‌سایز پرامپت سیستم برای Antigravity:
/// ۱) هویت هرمس → هویت کیان (بقیهٔ جمله دست‌نخورده)؛
/// ۲) حذف کامل خطوط محرکِ 429 جعلی.
#[must_use]
pub fn sanitize_system_prompt_for_antigravity(system_prompt: &str) -> String {
    let mut out = Vec::new();
    for line in system_prompt.lines() {
        let rewritten = line.replace(HERMES_IDENTITY_TRIGGER, KIAN_IDENTITY_REWRITE);
        let trimmed = rewritten.trim_start();
        if ANTIGRAVITY_TRIGGER_LINES.iter().any(|t| trimmed.starts_with(t)) {
            continue;
        }
        out.push(rewritten);
    }
    out.join("\n")
}

// ───────────────────── کاتالوگ پلتفرم‌های رایگان ─────────────────────

/// لایهٔ ثبت‌نام پلتفرم.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignupTier {
    /// ناشناس کار می‌کند — بدون حساب و بدون کلید.
    Keyless,
    /// ثبت‌نام رایگان با کلید — کارت بانکی ممنوع (حاکمیت `DNAme`).
    NoCardFree,
    /// نیازمند احراز هویت واقعی چین — فقط برای کامل بودن کاتالوگ.
    CnRealName,
}

/// مشخصهٔ یک پلتفرم LLM رایگان — همه OpenAI-compatible.
#[derive(Debug, Clone, Copy)]
pub struct FreeLlmProviderSpec {
    pub platform: &'static str,
    pub display_name: &'static str,
    pub base_url: &'static str,
    pub signup_tier: SignupTier,
    pub default_free_models: &'static [&'static str],
    pub rpm_cap: Option<u32>,
    pub rpd_cap: Option<u32>,
    pub tpd_cap: Option<u64>,
    pub supports_tools: bool,
    pub timeout_ms: u64,
    pub signup_url: &'static str,
    pub quirks: &'static str,
}

/// کاتالوگ گزینش‌شدهٔ ۳۲ پلتفرم صفر-کارت (از ۵۲ پلتفرم کاتالوگ کامل —
/// بقیه در `docs/vault/freellm_public_apis/` مونت‌اند).
pub const FREE_LLM_PROVIDERS: &[FreeLlmProviderSpec] = &[
    FreeLlmProviderSpec {
        platform: "kilo",
        display_name: "Kilo Gateway",
        base_url: "https://api.kilo.ai/api/gateway/v1",
        signup_tier: SignupTier::Keyless,
        default_free_models: &[
            "nvidia/nemotron-3-super-120b-a12b:free",
            "stepfun/step-3.7-flash:free",
            "poolside/laguna-m.1:free",
        ],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://app.kilo.ai",
        quirks: "Keyless anonymous :free routes (200 req/hr per IP); prompts logged for training.",
    },
    FreeLlmProviderSpec {
        platform: "ovh",
        display_name: "OVHcloud AI Endpoints",
        base_url: "https://oai.endpoints.kepler.ai.cloud.ovh.net/v1",
        signup_tier: SignupTier::Keyless,
        default_free_models: &["Qwen3.5-397B-A17B", "gpt-oss-120b", "Meta-Llama-3_3-70B-Instruct"],
        rpm_cap: Some(2),
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://endpoints.ai.cloud.ovh.net",
        quirks: "Keyless anonymous tier is 2 RPM per IP per model; great fallback, not high throughput.",
    },
    FreeLlmProviderSpec {
        platform: "llm7",
        display_name: "LLM7.io",
        base_url: "https://api.llm7.io/v1",
        signup_tier: SignupTier::Keyless,
        default_free_models: &["gpt-oss-20b", "meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo", "codestral-latest"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://llm7.io",
        quirks: "100 req/hr free tier; anonymous works for basic models.",
    },
    FreeLlmProviderSpec {
        platform: "aihorde",
        display_name: "AI Horde",
        base_url: "https://oai.aihorde.net/v1",
        signup_tier: SignupTier::Keyless,
        default_free_models: &["aphrodite/meta-llama/Llama-3.3-70B-Instruct"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: false,
        timeout_ms: 120_000,
        signup_url: "https://aihorde.net/register",
        quirks: "Queue-based volunteer workers; sentinel key 0000000000; max_tokens>=16; no streaming/tools.",
    },
    FreeLlmProviderSpec {
        platform: "pollinations",
        display_name: "Pollinations",
        base_url: "https://gen.pollinations.ai/v1",
        signup_tier: SignupTier::Keyless,
        default_free_models: &["openai-fast"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://enter.pollinations.ai",
        quirks: "1 pollen/IP/hr; validate via /account/key (GET /v1/models ignores revoked keys).",
    },
    FreeLlmProviderSpec {
        platform: "cerebras",
        display_name: "Cerebras",
        base_url: "https://api.cerebras.ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["qwen-3-coder-480b", "llama-4-maverick-17b-128e-instruct", "gpt-oss-120b"],
        rpm_cap: Some(30),
        rpd_cap: None,
        tpd_cap: Some(1_000_000),
        supports_tools: true,
        timeout_ms: 20_000,
        signup_url: "https://cloud.cerebras.ai",
        quirks: "Fastest wafer-scale inference (~30M tokens/mo free pool); 1M tokens/day cap.",
    },
    FreeLlmProviderSpec {
        platform: "groq",
        display_name: "Groq",
        base_url: "https://api.groq.com/openai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[
            "llama-3.3-70b-versatile",
            "llama-4-scout-17b-16e-instruct",
            "openai/gpt-oss-120b",
            "qwen/qwen3-32b",
        ],
        rpm_cap: Some(30),
        rpd_cap: Some(1_000),
        tpd_cap: Some(500_000),
        supports_tools: true,
        timeout_ms: 20_000,
        signup_url: "https://console.groq.com/keys",
        quirks: "Ultra-low latency LPU; strict directional TPM (6K-30K depending on model).",
    },
    FreeLlmProviderSpec {
        platform: "google",
        display_name: "Google Gemini (OpenAI-compat)",
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["gemini-2.5-flash", "gemini-2.5-flash-lite", "gemini-2.5-pro"],
        rpm_cap: Some(10),
        rpd_cap: Some(20),
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 60_000,
        signup_url: "https://aistudio.google.com/apikey",
        quirks: "1M context; hidden thinking tokens count against max_tokens.",
    },
    FreeLlmProviderSpec {
        platform: "openrouter",
        display_name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[
            "deepseek/deepseek-v3.1:free",
            "qwen/qwen3-coder:free",
            "moonshotai/kimi-k2:free",
            "z-ai/glm-4.5-air:free",
        ],
        rpm_cap: Some(20),
        rpd_cap: Some(50),
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 45_000,
        signup_url: "https://openrouter.ai/keys",
        quirks: "50 RPD shared across :free models (<10 credits) or 1,000 RPD (>=10 credits).",
    },
    FreeLlmProviderSpec {
        platform: "mistral",
        display_name: "Mistral Experiment",
        base_url: "https://api.mistral.ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["mistral-large-latest", "codestral-latest"],
        rpm_cap: Some(2),
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://console.mistral.ai/api-keys/",
        quirks: "Phone verification required; ~1B tokens/month Experiment pool at 2 RPM.",
    },
    FreeLlmProviderSpec {
        platform: "nvidia",
        display_name: "NVIDIA NIM",
        base_url: "https://integrate.api.nvidia.com/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["meta/llama-3.3-70b-instruct", "deepseek-ai/deepseek-v4-pro"],
        rpm_cap: Some(40),
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 180_000,
        signup_url: "https://build.nvidia.com/settings/api-keys",
        quirks: "Account-wide 40 RPM bucket; set parallel_tool_calls=false; 180s first-byte grace.",
    },
    FreeLlmProviderSpec {
        platform: "cloudflare",
        display_name: "Cloudflare Workers AI",
        base_url: "https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["@cf/openai/gpt-oss-120b", "@cf/meta/llama-3.1-70b-instruct"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 45_000,
        signup_url: "https://dash.cloudflare.com",
        quirks: "10,000 Neurons/day free; requires account_id in base_url.",
    },
    FreeLlmProviderSpec {
        platform: "zhipu",
        display_name: "Zhipu AI (Z.ai Global)",
        base_url: "https://api.z.ai/api/paas/v4",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["glm-4.7-flash", "glm-4.5-flash", "glm-4.6v-flash"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: Some(1_000_000),
        supports_tools: true,
        timeout_ms: 60_000,
        signup_url: "https://z.ai/manage-apikey/apikey-list",
        quirks: "60s timeout for glm-4.7-flash hidden reasoning; domestic host open.bigmodel.cn.",
    },
    FreeLlmProviderSpec {
        platform: "ollama",
        display_name: "Ollama Cloud",
        base_url: "https://ollama.com/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["qwen3-coder:480b", "deepseek-v3.2", "kimi-k2-thinking"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 120_000,
        signup_url: "https://ollama.com/settings/keys",
        quirks: "1 concurrent model, 5h GPU-time caps; reasoning returned in message.reasoning.",
    },
    FreeLlmProviderSpec {
        platform: "huggingface",
        display_name: "Hugging Face Router",
        base_url: "https://router.huggingface.co/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["deepseek-ai/DeepSeek-V4-Flash", "Qwen/Qwen3-Coder-Next"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 45_000,
        signup_url: "https://huggingface.co/settings/tokens",
        quirks: "$0.10/month recurring Inference Providers credit on free accounts.",
    },
    FreeLlmProviderSpec {
        platform: "cohere",
        display_name: "Cohere",
        base_url: "https://api.cohere.ai/compatibility/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["command-a-03-2025", "command-r-plus-08-2024"],
        rpm_cap: Some(20),
        rpd_cap: Some(33),
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://dashboard.cohere.com/api-keys",
        quirks: "Trial key allows 1,000 calls/month total (~33 RPD).",
    },
    FreeLlmProviderSpec {
        platform: "requesty",
        display_name: "Requesty",
        base_url: "https://router.requesty.ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: None,
        rpd_cap: Some(180),
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://www.requesty.ai",
        quirks: "200 RPD free tier shared across all free models (recommended router cap: 180).",
    },
    FreeLlmProviderSpec {
        platform: "bazaarlink",
        display_name: "BazaarLink",
        base_url: "https://bazaarlink.ai/api/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["auto:free"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 45_000,
        signup_url: "https://bazaarlink.ai",
        quirks: "Only `auto:free` is zero-cost; direct model IDs are paid.",
    },
    FreeLlmProviderSpec {
        platform: "orcarouter",
        display_name: "OrcaRouter",
        base_url: "https://api.orcarouter.ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["orcarouter/free"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://www.orcarouter.ai",
        quirks: "Free aliases never fall back to paid models; 429 is clean quota exhaustion.",
    },
    FreeLlmProviderSpec {
        platform: "llmkiwi",
        display_name: "LLM.kiwi",
        base_url: "https://api.llm.kiwi/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["auto", "hrllm"],
        rpm_cap: Some(20),
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://llm.kiwi/dashboard",
        quirks: "Free plan only serves `auto` & `hrllm` (others 403 pro_model_access_denied).",
    },
    FreeLlmProviderSpec {
        platform: "cheapllm",
        display_name: "cheapllm.shop",
        base_url: "https://cheapllm.shop/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["deepseek-v4.1-flash-1017"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 180_000,
        signup_url: "https://cheapllm.shop/signup",
        quirks: "5K in + 1K out tokens/hr; queues instead of returning 429; 180s timeout.",
    },
    FreeLlmProviderSpec {
        platform: "powertokens",
        display_name: "PowerTokens",
        base_url: "https://api.powertokens.ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &["glm-4.7-flash"],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 60_000,
        signup_url: "https://www.powertokens.ai",
        quirks: "glm-4.7-flash is $0/1M in+out; handles load with 429 code 1305.",
    },
    FreeLlmProviderSpec {
        platform: "navy",
        display_name: "NavyAI",
        base_url: "https://api.navy/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: Some(20),
        rpd_cap: None,
        tpd_cap: Some(150_000),
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://api.navy",
        quirks: "Requires explicit User-Agent header; Discord-backed free key.",
    },
    FreeLlmProviderSpec {
        platform: "sealion",
        display_name: "SEA-LION (AI Singapore)",
        base_url: "https://api.sea-lion.ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: Some(10),
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://sea-lion.ai",
        quirks: "Google sign-in, no card, no region wall; 10 RPM recurring free tier.",
    },
    FreeLlmProviderSpec {
        platform: "routeway",
        display_name: "Routeway",
        base_url: "https://api.routeway.ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: Some(5),
        rpd_cap: Some(200),
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://routeway.ai",
        quirks: "Requires browser-like User-Agent (Cloudflare 1010 otherwise); :free suffix.",
    },
    FreeLlmProviderSpec {
        platform: "unorouter",
        display_name: "UnoRouter",
        base_url: "https://api.unorouter.com/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: Some(1),
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://unorouter.com",
        quirks: "1 req/min per account on :free models; burst triggers multi-minute 429.",
    },
    FreeLlmProviderSpec {
        platform: "xkiro",
        display_name: "xKiro",
        base_url: "https://api.xkiro.com/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: Some(5_000_000),
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://xkiro.com",
        quirks: "5M TPD free plan; validate key via /v1/usage (since /v1/models is public).",
    },
    FreeLlmProviderSpec {
        platform: "agnes",
        display_name: "Agnes AI",
        base_url: "https://apihub.agnes-ai.com/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 60_000,
        signup_url: "https://platform.agnes-ai.com",
        quirks: "~30 concurrent reqs before 429; 60s timeout for reasoning TTFB.",
    },
    FreeLlmProviderSpec {
        platform: "inferbase",
        display_name: "Inferbase",
        base_url: "https://api.inferbase.ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: None,
        rpd_cap: Some(100),
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://inferbase.ai",
        quirks: "5,000 req/mo, 100 managed req/day, 1,024 max output tokens on free tier.",
    },
    FreeLlmProviderSpec {
        platform: "electronhub",
        display_name: "ElectronHub",
        base_url: "https://api.electronhub.ai/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 45_000,
        signup_url: "https://app.electronhub.ai",
        quirks: "$0.25 shared weekly credits; validate key via /v1/user/me.",
    },
    FreeLlmProviderSpec {
        platform: "iteracompute",
        display_name: "IteraCompute",
        base_url: "https://api.iteracompute.com/v1",
        signup_tier: SignupTier::NoCardFree,
        default_free_models: &[],
        rpm_cap: None,
        rpd_cap: None,
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 30_000,
        signup_url: "https://iteracompute.com/free/",
        quirks: "30M monthly token pool shared account-wide.",
    },
    FreeLlmProviderSpec {
        platform: "modelscope",
        display_name: "ModelScope (Alibaba)",
        base_url: "https://api-inference.modelscope.cn/v1",
        signup_tier: SignupTier::CnRealName,
        default_free_models: &[],
        rpm_cap: None,
        rpd_cap: Some(100),
        tpd_cap: None,
        supports_tools: true,
        timeout_ms: 45_000,
        signup_url: "https://modelscope.cn/my/myaccesstoken",
        quirks: "~250 magic-grain/day; requires Aliyun CN binding + real-name (listed for completeness).",
    },
];

/// پلتفرم را با نام پیدا می‌کند.
#[must_use]
pub fn get_free_provider(platform: &str) -> Option<&'static FreeLlmProviderSpec> {
    FREE_LLM_PROVIDERS.iter().find(|p| p.platform == platform)
}

/// لِین‌های Tier-1 — ناشناس مطلق، هیچ حسابی لازم نیست
/// (`opencode-zen` در [`crate::opencode_zen`] با `Bearer public` مدیریت می‌شود).
pub const TIER_ONE_KEYLESS_CHAIN: &[&str] = &["opencode-zen", "kilo", "ovh", "llm7", "pollinations"];

/// لِین‌های Tier-2 — ثبت‌نام رایگان بدون کارت (آبشار اصلی کیان).
pub const TIER_TWO_NO_CARD_CHAIN: &[&str] =
    &["cerebras", "groq", "google", "openrouter", "mistral", "nvidia", "zhipu", "cloudflare"];

/// آبشار کامل صفر-کارت: Tier1 → Tier2؛ Tier3 کاتالوگ مرورگری است، نه سروری.
#[must_use]
pub fn zero_card_waterfall() -> Vec<&'static FreeLlmProviderSpec> {
    TIER_ONE_KEYLESS_CHAIN
        .iter()
        .chain(TIER_TWO_NO_CARD_CHAIN.iter())
        .filter_map(|p| get_free_provider(p))
        .collect()
}

// ───────────────────── Tier-3: کاتالوگ ۱۲۰ API عمومی استودیو ─────────────────────

/// یک API عمومی بدون کلید — برای `fetch()` زنده در استودیوی کیان.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicApiSpec {
    pub name: &'static str,
    pub url: &'static str,
    pub category: &'static str,
    pub https: bool,
    pub cors: bool,
}

/// کاتالوگ ۱۲۰ API عمومی HTTPS/CORS (`top_120_studio_browser_apis.json`
/// مونت‌شده در `docs/vault/freellm_public_apis/`) — `(name, url, category, https, cors)`.
pub const STUDIO_PUBLIC_APIS_120: &[(&str, &str, &str, bool, bool)] = &[
    ("Cat Facts", "https://catfact.ninja", "Animals", true, true),
    ("HTTP Cat", "https://http.cat", "Animals", true, true),
    ("Art Institute of Chicago", "https://api.artic.edu/docs", "Art & Design", true, true),
    ("EmojiHub", "https://github.com/cheatsnake/emojihub", "Art & Design", true, true),
    ("Favicondl", "https://favicondl.com/documentation.html", "Art & Design", true, true),
    ("Iconify", "https://iconify.design/docs/api", "Art & Design", true, true),
    ("PoetryDB", "https://github.com/thundercomb/poetrydb#readme", "Books", true, true),
    ("HolidayExplore", "https://holidayexplore.com/api-docs", "Calendar", true, true),
    ("SalahTable", "https://salahtable.com/prayer-times-api", "Calendar", true, true),
    ("caldays", "https://caldays.com/api", "Calendar", true, true),
    ("0x", "https://docs.0x.org", "Cryptocurrency", true, true),
    ("BTCGlobe", "https://btcglobe.live/join#makers", "Cryptocurrency", true, true),
    ("Bitcoin Halving", "https://why21million.com/halving-api", "Cryptocurrency", true, true),
    ("BitcoinYield", "https://bitcoinyield.co/docs", "Cryptocurrency", true, true),
    ("BlazePhoenix", "https://blazephoenix.xyz/api", "Cryptocurrency", true, true),
    ("CoinGecko", "http://www.coingecko.com/api", "Cryptocurrency", true, true),
    ("AllRatesToday", "https://allratestoday.com/central-bank-rates-api", "Currency Exchange", true, true),
    ("Currency-api", "https://github.com/fawazahmed0/currency-api#readme", "Currency Exchange", true, true),
    ("Frankfurter", "https://frankfurter.dev", "Currency Exchange", true, true),
    ("Quotelinks Exchange Rates", "https://www.quotelinks.com/api/exchange-rates", "Currency Exchange", true, true),
    ("api-mint", "https://api-mint.hoiwan.workers.dev", "Currency Exchange", true, true),
    ("AI SENSE", "https://aisense.no/free-public-apis", "Development", true, true),
    ("APIzone", "https://apizone.io/api-docs", "Development", true, true),
    ("Agify.io", "https://agify.io", "Development", true, true),
    ("Beeceptor", "https://beeceptor.com", "Development", true, true),
    ("FreeQrly", "https://freeqrly.com/developers", "Development", true, true),
    ("Quotable", "https://github.com/lukePeavey/quotable", "Development", true, true),
    ("The Networkery", "https://thenetworkery.com/api", "Development", true, true),
    ("Random Lexicon", "https://randomlexicon.com/api-reference", "Dictionaries", true, true),
    ("Wiktionary", "https://en.wiktionary.org/w/api.php", "Dictionaries", true, true),
    ("WordSoHard", "https://wordsohard.com/api", "Dictionaries", true, true),
    ("Fun Fact", "https://api.aakhilv.me", "Entertainment", true, true),
    ("JokeAPI", "https://v2.jokeapi.dev", "Entertainment", true, true),
    ("PotterDB", "https://docs.potterdb.com", "Entertainment", true, true),
    ("Open-Meteo", "https://open-meteo.com", "Environment", true, true),
    ("SpotWave", "https://spotwave.eu/api-docs", "Environment", true, true),
    ("CardBinCheck", "https://cardbincheck.com/bin-lookup-api", "Finance", true, true),
    ("Econdb", "https://www.econdb.com/api", "Finance", true, true),
    ("EstimateTax", "https://estimatetax.net/api", "Finance", true, true),
    ("Open Brewery DB", "https://www.openbrewerydb.org", "Food & Drink", true, true),
    ("CheapShark", "https://www.cheapshark.com/api", "Games & Comics", true, true),
    ("Disney", "https://disneyapi.dev", "Games & Comics", true, true),
    ("PokéAPI (GraphQL)", "https://github.com/mazipan/graphql-pokeapi", "Games & Comics", true, true),
    ("17NAS", "https://17nas.com", "Geocoding", true, true),
    ("Country", "http://country.is", "Geocoding", true, true),
    ("GeoJS", "https://www.geojs.io", "Geocoding", true, true),
    ("GeographQL", "https://geographql.netlify.app", "Geocoding", true, true),
    ("HackMyIP", "https://hackmyip.com/api", "Geocoding", true, true),
    ("REST Countries", "https://restcountries.com", "Geocoding", true, true),
    ("ListenBrainz", "https://listenbrainz.readthedocs.io/en/latest/users/api/index.html", "Music", true, true),
    ("Radio Browser", "https://api.radio-browser.info", "Music", true, true),
    ("Noozra", "https://noozra.com/api", "News", true, true),
    ("Spaceflight News", "https://spaceflightnewsapi.net", "News", true, true),
    ("Agent Data Hub", "https://api.agentdatahub.de/docs", "Open Data", true, true),
    ("balldontlie", "https://www.balldontlie.io", "Sports & Fitness", true, true),
    ("DummyJSON", "https://dummyjson.com", "Test Data", true, true),
    ("FakerAPI", "https://fakerapi.it/en", "Test Data", true, true),
    ("CleanURI", "https://cleanuri.com/docs", "URL Shorteners", true, true),
    ("GoTiny", "https://github.com/robvanbakel/gotiny-api", "URL Shorteners", true, true),
    ("NASA POWER", "https://power.larc.nasa.gov/docs", "Weather", true, true),
    ("Open-Meteo Ensemble", "https://open-meteo.com/en/docs/ensemble-api", "Weather", true, true),
    ("Pirate Weather", "https://pirateweather.readthedocs.io/en/latest", "Weather", true, true),
    ("US Weather", "https://www.weather.gov/documentation/services-web-api", "Weather", true, true),
    ("Free Dictionary", "https://dictionaryapi.dev", "Dictionaries", true, false),
    ("Open Trivia", "https://opentdb.com/api_config.php", "Games & Comics", true, false),
    ("Wikipedia", "https://www.mediawiki.org/wiki/API:Main_page", "Open Data", true, false),
    ("SpaceX", "https://github.com/r-spacex/SpaceX-API", "Science & Math", true, false),
    ("HackerNews", "https://github.com/HackerNews/API", "Social", true, false),
    ("Dog Facts", "https://dukengn.github.io/Dog-facts-API", "Animals", true, true),
    ("AnimeFacts", "https://chandan-02.github.io/anime-facts-rest-api", "Anime", true, true),
    ("AnimeNewsNetwork", "https://www.animenewsnetwork.com/encyclopedia/api.php", "Anime", true, true),
    ("Catboy", "https://catboys.com/api", "Anime", true, true),
    ("ScanMalware", "https://scanmalware.com", "Anti-Malware", true, true),
    ("URLhaus", "https://urlhaus-api.abuse.ch", "Anti-Malware", true, true),
    ("3DTexel", "https://3dtexel.com/developers/#public-api", "Art & Design", true, true),
    ("ClearTrace", "https://cleartracedata.com/docs", "Blockchain", true, true),
    (
        "Get Started with Web3",
        "https://github.com/beihaili/Get-Started-with-Web3/blob/main/docs/api.md",
        "Blockchain",
        true,
        true,
    ),
    ("HEY Research Lab", "https://heyresearch.xyz/developers", "Blockchain", true, true),
    ("Bayan", "https://bayan.alwa7i.com/api", "Books", true, true),
    ("Bhagavad Gita telugu", "https://gita-api.vercel.app", "Books", true, true),
    ("Bible-api", "https://bible-api.com", "Books", true, true),
    ("BookRank", "https://openbookrank.com/en/open-data", "Books", true, true),
    ("Buku Indonesia", "https://github.com/khrlanamm/buku-indonesia#api", "Books", true, true),
    ("KontragentPro", "https://kontragentpro.ru/api/v2/docs", "Business", true, true),
    ("Legal Sandbox Georgia", "https://legal.ge/api/openapi.json", "Business", true, true),
    ("Logoapi", "https://logoapi.sh/docs", "Business", true, true),
    ("CountdownsHQ", "https://countdownshq.com/developers", "Calendar", true, true),
    ("Deed", "https://deed.page/docs", "Cloud Storage & File Sharing", true, true),
    ("Pantry", "https://getpantry.cloud", "Cloud Storage & File Sharing", true, true),
    ("Alpha (Mossland)", "https://alpha.moss.land/developers", "Cryptocurrency", true, true),
    ("bilbop", "https://api.bilbop.org", "Cryptocurrency", true, true),
    ("Block Lottos", "https://blocklottos.com/openapi.json", "Cryptocurrency", true, true),
    ("CoinLobster", "https://coinlobster.com/developers", "Cryptocurrency", true, true),
    ("Daleelak", "https://getdaleelak.com/en/api", "Currency Exchange", true, true),
    ("DolarKaç", "https://dolarkac.com/gelistirici-api", "Currency Exchange", true, true),
    ("24 Pull Requests", "https://24pullrequests.com/api", "Development", true, true),
    ("Agent Nexus", "https://agentnexus.app/llms.txt", "Development", true, true),
    (
        "AgentPay Domain Lookup",
        "https://agentpay-lookup.agentpay-apis.workers.dev",
        "Development",
        true,
        true,
    ),
    ("ApicAgent", "https://www.apicagent.com", "Development", true, true),
    ("BountyCheck", "https://bountycheck.vercel.app", "Development", true, true),
    ("ChartBytes", "https://chartbytes.meridian-digital.pro", "Development", true, true),
    ("CitedBy", "https://citedby.eu/ai-access-api", "Development", true, true),
    (
        "AgentPay Doc Tools",
        "https://agentpay-tools.agentpay-apis.workers.dev",
        "Documents & Productivity",
        true,
        true,
    ),
    ("CleverUtils", "https://cleverutils.com/api", "Documents & Productivity", true, true),
    ("DocStruct", "https://docstruct.pages.dev", "Documents & Productivity", true, true),
    ("AGPC Domain Check", "https://guild.tradeuniquecapital.com/api", "Email", true, true),
    ("Disify", "https://www.disify.com", "Email", true, true),
    ("Email Spam Tester", "https://email-spam-tester.com/api-docs", "Email", true, true),
    (
        "Bucketlist Dream of the Day",
        "https://bucketlist.nl/samenwerken?lang=en#droom-van-de-dag",
        "Entertainment",
        true,
        true,
    ),
    (
        "Corporate Buzz Words",
        "https://github.com/sameerkumar18/corporate-bs-generator-api",
        "Entertainment",
        true,
        true,
    ),
    ("CosmyDay Astrology", "https://cosmyday.com/api-docs", "Entertainment", true, true),
    ("justmeme.wtf", "https://justmeme.wtf/api-docs", "Entertainment", true, true),
    ("Calendrier Tempo", "https://www.calendrier-tempo.fr/api-tempo", "Environment", true, true),
    ("NeedATicket", "https://needaticket.co.uk/developers", "Events", true, true),
    ("Things To Do", "https://thingstodoapi.com/docs", "Events", true, true),
    (
        "Banking Access Index",
        "https://www.globalsolo.global/data/banking-access-index",
        "Finance",
        true,
        true,
    ),
    ("Billionaire Tracker", "https://billionairetracker.us/developers", "Finance", true, true),
    ("bluerate", "https://www.bluerate.ai/developers", "Finance", true, true),
    ("Brainy Prices", "https://prices.brainy.ae/developers.html", "Finance", true, true),
    ("contix", "https://contix.es/api/#herramientas", "Finance", true, true),
];

/// شمار کاتالوگ Tier-3 — قرارداد سخت: ۱۲۰.
pub const STUDIO_PUBLIC_APIS_120_COUNT: usize = 120;

/// APIهای مرورگر-آماده (HTTPS+CORS) به‌صورت struct — فقط همین‌ها در `fetch()`
/// زندهٔ استودیو مجازند.
pub fn browser_ready_public_apis() -> impl Iterator<Item = PublicApiSpec> {
    STUDIO_PUBLIC_APIS_120
        .iter()
        .filter(|(_, _, _, https, cors)| *https && *cors)
        .map(|(name, url, category, https, cors)| PublicApiSpec {
            name,
            url,
            category,
            https: *https,
            cors: *cors,
        })
}

// ───────────────────── میان‌افزار تاب‌آوری ─────────────────────

/// نشانگرهای جای‌نگهدارِ آرگومان ابزار — کشف قطعی (langflow `WatsonX` الگو).
const PLACEHOLDER_MARKERS: [&str; 5] =
    ["<result-from-", "<previous-output>", "{{step_", "{{result", "<insert-"];

/// کشف آرگومان ابزارِ placeholder-مانند — مدل به‌جای مقدار واقعی، علامت
/// جای‌نگهدار نتیجهٔ ابزار دیگر را گذاشته است (`langflow` `WatsonXPlaceholderMiddleware`).
/// `Some(hint)` = hint تلاش یک‌باره برای دور بعد.
#[must_use]
pub fn detect_unresolved_placeholder_tool_args(tool_args_json: &str) -> Option<String> {
    let lower = tool_args_json.to_ascii_lowercase();
    let mut hit: Option<&str> = None;
    for marker in &PLACEHOLDER_MARKERS {
        if lower.contains(*marker) {
            hit = Some(marker);
            break;
        }
    }
    let hit = hit?;
    Some(format!(
        "Tool argument contains unresolved placeholder `{hit}` instead of a real value. Retry ONCE: \
         re-emit the tool call with concrete arguments derived from previous tool outputs (or ask the \
         user). Do not reference outputs of tools that have not run."
    ))
}

/// آغازگرهای ردِ صریح مدل — الگوی `langchain` `OutputFixing`.
const REFUSAL_OPENERS: [&str; 4] =
    ["i'm sorry, but", "i am sorry, but", "i can't help with", "i cannot help with"];

/// چک فیلد refusal صریح پیش از پارس JSON Schema — الگوی `langchain` `OutputFixing`.
/// `Some(reason)` یعنی مدل رد کرده و پارس schema بی‌معنی است؛ caller باید
/// retry prompt بسازد نه اینکه خروجی خراب را schema-pars کند.
#[must_use]
pub fn check_model_refusal(completion: &Value) -> Option<String> {
    let top_refusal = completion
        .get("refusal")
        .and_then(Value::as_str)
        .filter(|r| !r.trim().is_empty());
    if let Some(r) = top_refusal {
        return Some(r.to_string());
    }
    if let Some(choice) = completion.pointer("/choices/0") {
        let choice_refusal = choice
            .pointer("/message/refusal")
            .and_then(Value::as_str)
            .filter(|r| !r.trim().is_empty());
        if let Some(r) = choice_refusal {
            return Some(r.to_string());
        }
        if choice.get("finish_reason").and_then(Value::as_str) == Some("refusal") {
            return Some("finish_reason=refusal".to_string());
        }
        let content = choice.pointer("/message/content").and_then(Value::as_str).unwrap_or("");
        let lower = content.trim_start().to_ascii_lowercase();
        if REFUSAL_OPENERS.iter().any(|op| lower.starts_with(op)) {
            return Some(content.chars().take(160).collect());
        }
    }
    if completion.get("stop_reason").and_then(Value::as_str) == Some("refusal") {
        return Some("stop_reason=refusal".to_string());
    }
    None
}

// ───────────────────── سلامت لِین و فال‌بک ─────────────────────

/// تصمیم پس از شکست HTTP لِین — پورت `9router` (`accountFallback.js` + `errorConfig.js`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneFailureAction {
    /// 410/404 مدل‌ناپیدا/بازنشسته → لِین برای کل نشست سیاه‌می‌شود.
    RetireLane,
    /// 400 خطای اعتبارسنجی درخواست → فوری لِین بعدی، بدون کول‌داون حساب.
    AdvanceNoCooldown,
    /// 429 → بک‌آف نمایی `2s × 2^level` (سقف ۵ دقیقه).
    BackoffCooldown(Duration),
    /// 5xx ترانزیت → کول‌داون کوتاه ۳۰ ثانیه.
    TransientCooldown(Duration),
}

/// طبقه‌بندی شکست HTTP لِین (قطعی و تست‌پذیر).
#[must_use]
pub fn classify_http_failure(status: u16, body_text: &str, backoff_level: u32) -> LaneFailureAction {
    let lower = body_text.to_ascii_lowercase();
    if status == 410 || lower.contains("model is unavailable") || lower.contains("model_not_found") {
        return LaneFailureAction::RetireLane;
    }
    if status == 429
        || lower.contains("rate limit")
        || lower.contains("too many requests")
        || lower.contains("quota exceeded")
        || lower.contains("overloaded")
    {
        let exp = backoff_level.saturating_sub(1).min(14);
        let secs = 2u64.saturating_mul(1u64 << exp).min(300);
        return LaneFailureAction::BackoffCooldown(Duration::from_secs(secs));
    }
    if (400..500).contains(&status) && !matches!(status, 401..=403 | 429) {
        return LaneFailureAction::AdvanceNoCooldown;
    }
    if matches!(status, 401..=404) {
        return LaneFailureAction::BackoffCooldown(Duration::from_secs(120));
    }
    LaneFailureAction::TransientCooldown(Duration::from_secs(30))
}

/// ردیاب سلامت لِین‌ها در طول نشست — بازنشسته‌ها + کول‌داون‌ها.
#[derive(Debug, Default)]
pub struct LaneHealthTracker {
    cooldowns: HashMap<String, (Instant, u32)>,
    retired: HashMap<String, String>,
}

impl LaneHealthTracker {
    /// آیا لِین الان قابل استفاده است؟
    #[must_use]
    pub fn is_available(&self, model_id: &str) -> bool {
        if self.retired.contains_key(model_id) {
            return false;
        }
        if let Some((until, _)) = self.cooldowns.get(model_id) {
            return Instant::now() >= *until;
        }
        true
    }

    /// ثبت شکست و گرفتن تصمیم.
    pub fn record_failure(&mut self, model_id: &str, status: u16, body: &str) -> LaneFailureAction {
        let prev_level = self.cooldowns.get(model_id).map_or(0, |(_, lvl)| *lvl);
        let action = classify_http_failure(status, body, prev_level + 1);
        match action {
            LaneFailureAction::RetireLane => {
                self.retired.insert(model_id.to_string(), format!("HTTP {status}"));
            }
            LaneFailureAction::AdvanceNoCooldown => {}
            LaneFailureAction::BackoffCooldown(d) | LaneFailureAction::TransientCooldown(d) => {
                self.cooldowns
                    .insert(model_id.to_string(), (Instant::now() + d, (prev_level + 1).min(15)));
            }
        }
        action
    }

    /// ثبت موفقیت — کول‌داون پاک می‌شود.
    pub fn record_success(&mut self, model_id: &str) {
        self.cooldowns.remove(model_id);
    }
}

// ───────────────────── تست‌ها ─────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn terminal_tools_are_renamed_to_shell_exec_reversibly() {
        let mut tools = vec![
            json!({"type": "function", "function": {"name": "terminal"}}),
            json!({"type": "function", "function": {"name": "MyTerminalTool"}}),
        ];
        let map = sanitize_tool_names_for_opencode(&mut tools);
        assert_eq!(tools[0]["function"]["name"], "shell_exec");
        assert_eq!(tools[1]["function"]["name"], "shell_exec_2");
        assert_eq!(map.get("shell_exec").map(String::as_str), Some("terminal"));
        assert_eq!(map.get("shell_exec_2").map(String::as_str), Some("MyTerminalTool"));
        assert_eq!(restore_tool_names("shell_exec_2", &map), Some("MyTerminalTool"));
        assert_eq!(restore_tool_names("unrelated_tool", &map), None);
        // ابزار بدون terminal دست‌نخورده می‌ماند
        let mut clean = vec![json!({"type": "function", "function": {"name": "read_file"}})];
        assert!(sanitize_tool_names_for_opencode(&mut clean).is_empty(), "clean tool must not be renamed");
        assert_eq!(clean[0]["function"]["name"], "read_file");
    }

    #[test]
    fn antigravity_rewrites_hermes_identity_and_strips_429_triggers() {
        let raw = "You are Hermes Agent, built by Nous Research. Be direct: match the persona.\n\
                   Keep me.\n<!-- End Hermes runtime environment -->\n# Google model operational directives\nTail.";
        let clean = sanitize_system_prompt_for_antigravity(raw);
        assert!(clean.contains(KIAN_IDENTITY_REWRITE), "identity rewritten: {clean}");
        assert!(clean.contains("Be direct: match the persona."), "directive suffix preserved");
        assert!(clean.contains("Keep me."));
        assert!(!clean.contains("Hermes runtime environment"));
        assert!(!clean.contains("Google model operational directives"));
        assert!(clean.contains("Tail."));
    }

    #[test]
    fn three_tier_waterfall_is_complete_and_card_free() {
        assert_eq!(TIER_ONE_KEYLESS_CHAIN.len(), 5);
        assert!(TIER_ONE_KEYLESS_CHAIN.contains(&"opencode-zen"));
        assert_eq!(TIER_TWO_NO_CARD_CHAIN.len(), 8);
        let waterfall = zero_card_waterfall();
        assert_eq!(waterfall.len(), 12, "5 keyless + 8 no-card minus opencode-zen virtual");
        for p in &waterfall {
            assert_ne!(p.signup_tier, SignupTier::CnRealName, "waterfall must stay zero-card");
        }
        // cerebras 1M TPD و zhipu 1M TPD در Tier-2 هستند
        assert_eq!(get_free_provider("cerebras").unwrap().tpd_cap, Some(1_000_000));
        assert_eq!(get_free_provider("zhipu").unwrap().tpd_cap, Some(1_000_000));
        assert!(get_free_provider("kilo").unwrap().base_url.contains("kilo.ai"));
    }

    #[test]
    fn studio_public_apis_catalog_has_120_entries_and_cors_filter() {
        assert_eq!(STUDIO_PUBLIC_APIS_120.len(), STUDIO_PUBLIC_APIS_120_COUNT);
        let browser_ready = browser_ready_public_apis().count();
        assert_eq!(browser_ready, 115, "115 of 120 are CORS=yes browser-ready");
        let gecko = STUDIO_PUBLIC_APIS_120.iter().find(|(name, _, _, _, _)| *name == "CoinGecko").unwrap();
        assert!(gecko.4 && gecko.2 == "Cryptocurrency");
        let first = browser_ready_public_apis().next().unwrap();
        assert_eq!(first.name, "Cat Facts");
        assert!(first.cors);
    }

    #[test]
    fn placeholder_tool_args_get_one_shot_retry_hint() {
        let hint = detect_unresolved_placeholder_tool_args(r#"{"path": "<result-from-read_file>"}"#)
            .expect("placeholder caught");
        assert!(hint.contains("Retry ONCE"), "hint: {hint}");
        assert!(detect_unresolved_placeholder_tool_args(r#"{"id": "{{step_3.output}}"}"#).is_some());
        assert!(detect_unresolved_placeholder_tool_args(r#"{"path": "/etc/hosts"}"#).is_none());
        assert!(detect_unresolved_placeholder_tool_args("{}").is_none());
    }

    #[test]
    fn refusals_are_caught_before_schema_parsing() {
        let with_refusal = json!({"choices": [{"message": {"content": "…", "refusal": "cannot do that"}}]});
        assert_eq!(
            check_model_refusal(&with_refusal).as_deref(),
            Some("cannot do that"),
            "explicit refusal field must win"
        );
        let finish = json!({"choices": [{"finish_reason": "refusal", "message": {"content": "x"}}]});
        assert!(check_model_refusal(&finish).is_some());
        let soft = json!({"choices": [{"message": {"content": "I'm sorry, but I can't help with that."}}]});
        assert!(check_model_refusal(&soft).is_some(), "soft refusal opener must be caught");
        let clean = json!({"choices": [{"message": {"content": "{\"a\":1}"}}]});
        assert!(check_model_refusal(&clean).is_none(), "clean JSON must pass");
        assert!(check_model_refusal(&json!({"refusal": ""})).is_none(), "empty refusal is not a refusal");
        assert!(check_model_refusal(&json!({"stop_reason": "refusal"})).is_some());
    }

    #[test]
    fn lane_failover_classification_matches_9router_semantics() {
        assert_eq!(classify_http_failure(410, "", 1), LaneFailureAction::RetireLane);
        assert_eq!(
            classify_http_failure(400, "{\"error\":\"model_not_found\"}", 1),
            LaneFailureAction::RetireLane
        );
        assert_eq!(classify_http_failure(400, "bad request", 1), LaneFailureAction::AdvanceNoCooldown);
        assert!(matches!(
            classify_http_failure(429, "rate limit exceeded", 1),
            LaneFailureAction::BackoffCooldown(_)
        ));
        // بک‌آف نمایی با سقف ۵ دقیقه
        let level1 = classify_http_failure(429, "", 1);
        assert_eq!(level1, LaneFailureAction::BackoffCooldown(Duration::from_secs(2)));
        let level8 = classify_http_failure(429, "", 8);
        assert_eq!(level8, LaneFailureAction::BackoffCooldown(Duration::from_secs(256)));
        let level20 = classify_http_failure(429, "", 20);
        assert_eq!(level20, LaneFailureAction::BackoffCooldown(Duration::from_secs(300)));
        assert!(matches!(
            classify_http_failure(503, "upstream error", 1),
            LaneFailureAction::TransientCooldown(_)
        ));
    }

    #[test]
    fn lane_tracker_retires_and_cools_down() {
        let mut tracker = LaneHealthTracker::default();
        assert!(tracker.is_available("oc/beam-free"));
        tracker.record_failure("oc/beam-free", 410, "gone");
        assert!(!tracker.is_available("oc/beam-free"), "410 must retire the lane");
        tracker.record_success("oc/deepseek-v4.1-flash-free"); // بدون کول‌داون — no-op امن
        assert!(tracker.is_available("oc/deepseek-v4.1-flash-free"));
        let action = tracker.record_failure("oc/deepseek-v4.1-flash-free", 429, "rate limit");
        assert!(matches!(action, LaneFailureAction::BackoffCooldown(_)));
        assert!(!tracker.is_available("oc/deepseek-v4.1-flash-free"));
    }

    #[test]
    fn providers_catalog_is_zero_card_by_dna() {
        // حاکمیت DNAme: هیچ پلتفرمی از آبشار رسمی کارت نمی‌خواهد؛
        // CnRealName فقط در کاتالوگ برای کامل بودن است و در آبشار نیست.
        let waterfall = zero_card_waterfall();
        assert!(waterfall.iter().all(|p| p.signup_tier != SignupTier::CnRealName));
        assert_eq!(get_free_provider("modelscope").unwrap().signup_tier, SignupTier::CnRealName);
        assert!(get_free_provider("modelscope").is_some());
        assert!(FREE_LLM_PROVIDERS.len() >= 30, "curated registry keeps the catalog depth");
    }
}
