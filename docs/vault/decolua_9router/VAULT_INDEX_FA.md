# فهرست گنجینه `decolua/9router` برای ادغام در KIAN v2 (`decolua-9router-gems`)

- **مخزن ممیزی‌شده:** `https://github.com/decolua/9router` (نسخه `v0.5.123`، کامیت `5ae63d22`، تاریخ آخرین آپدیت: `2026-11-22`) + ریپوی همراه `decolua/9remote`
- **تاریخ راستی‌آزمایی زنده:** `2026-11-23`
- **آدرس گیت‌وی Railway:** `https://9router-production-6f2225a3.up.railway.app/v1`

## ساختار پوشه‌ها و فایل‌ها

### `01_9router_core_and_routing/`
- `combo.js` — موتور Combo Router (`fallback`, `round-robin`, `smart`, `handleFusionChat` با Quorum و Judge، `emptyFailover` روی SSE و JSON، و عبور فوری روی خطاهای `requestScoped` بدون Cooldown اکانت).
- `accountFallback.js` & `errorConfig.js` — ماشین وضعیت Cooldown و Backoff نمایی (`base = 2000ms * 2^(level-1)`, سقف `5 min`) و تفکیک `requestScoped` (`400`/`410`) از خطاهای اعتبار/نرخ (`401`/`402`/`403`/`429`).
- `chatCore.js` — هسته پردازش چت: `stripContinuityFields`، `applyReaAllow`، فشرده‌سازی پرامپت (`RTK` / `Headroom` / `Caveman` / `Ponytail` / `Pxpipe`)، تشخیص ابزار کلاینت و دیسپچ به Executor.
- `appConstants.js` — قوانین `ANTIGRAVITY_PROMPT_REWRITES` شامل حذف ۷ خط پرامپت هرمس که `429` جعلی می‌دادند (کامیت `bb399170`) و لیست ابزارهای جعلی `CC_DEFAULT_TOOLS` / `AG_DEFAULT_TOOLS`.
- `capabilities.js` — جدول کامل قابلیت‌ها، پنجره کانتکست، `thinkingFormat` و پشتیبانی بینایی برای همه مدل‌های جدید.
- `Dockerfile` & `start.sh` — نحوه نصب `wreq-js` و `playwright-core` به‌عنوان Lazy Runtime Dependencies در Next Standalone (`#5245`).
- `README_9remote_summary.md` — خلاصه مخزن همراه `decolua/9remote`.

### `02_providers_and_free_translators/`
- `executors_opencode.js` — Executor اختصاصی OpenCode (`OPENCODE_UA = "opencode/latest/2.0.26/cli"`، تولید `x-opencode-session` با `~(Date.now()*0x1000 + counter)`، و ۸ بار Retry برای خطاهای `overloaded`).
- `opencodeTlsClient.js` — کلاینت `wreq-js` با Cipher List دقیق BoringSSL/Bun (`JA4: 5b57614c22b0`) برای عبور از گیت TLS سمت OpenCode.
- `opencodeFingerprint.js` — تزریق `<env>`، ابزارهای `read` و `shell` و تغییر نام خودکار ابزارهای حاوی `terminal` به `run_command` (و بازگردانی در پاسخ).
- `reasoningContentInjector.js` — تزریق خودکار `reasoning_content: " "` روی پیام‌های assistant در گفتگوی چندنوبتی برای جلوگیری از خطای `400 missing_required_parameter`.
- `claudeCloaking.js` & `systemPrompt.js` — پسوند `_ide` روی ابزارها، هدر صورت‌حساب شبیه‌سازی‌شده Claude Code و `applyReaAllow`.
- `translator_index.js`, `translator_request/`, `translator_response/` — مترجم‌های دوطرفه OpenAI ↔ Claude ↔ Gemini ↔ OpenAI Responses ↔ Ollama ↔ Kiro ↔ Antigravity.
- `free_registries/` — فایل‌های رجیستری تمام ارائه‌دهندگان رایگان / بدون نیاز به کارت (`opencode`, `nous`, `openrouter`, `nvidia`, `groq`, `cloudflare-ai`, `kilo-gateway`, `bazaarlink`, `api-airforce`, `gemini-cli`, `kiro`, `antigravity`, `iflow`, `mimo-free`, `devin-cli`, `cerebras`, `huggingface`, `ollama`, `suggested_models_filters`).

### `03_active_free_models_registry/`
- `active_free_models_2026-11-23.json` — خروجی JSON ماشین‌خوان شامل ۳۴ ارائه‌دهنده رایگان/FreeTier در `9router` + وضعیت زنده کاتالوگ `opencode.ai/zen/v1/models`، `openrouter.ai/api/v1/models` و `inference-api.nousresearch.com/v1/models`.
- `FREE_PROVIDERS_MATRIX_FA.md` — ماتریس فارسی پیشوندها، مدل‌های فعال و تله‌های احراز هویت/خطا.

### `04_kian_v2_rust_adapter/`
- `kian_9router_lanes.rs` — آداپتور آماده اتصال Rust (`reqwest` + `serde`) برای `crates/ai` در KIAN v2 همراه با Failover هوشمند (`410`/`400`/`429`/`5xx`)، تزریق `reasoning_content`، سانسور `terminal` برای OpenCode، و پاک‌سازی پرامپت هرمس برای Antigravity.
- `probe_9router_models.py` — اسکریپت تست زنده اندپوینت Railway و کاتالوگ‌های آزاد بالادستی.
