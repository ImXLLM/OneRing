# ماتریس کامل ارائه‌دهندگان و پیشوندهای ۱۰۰٪ رایگان (بدون کارت بانکی) در `decolua/9router`

تاریخ راستی‌آزمایی زنده: **2026-11-23** (`decolua/9router v0.5.123`, commit `5ae63d22`)

## ۱. لایه کاملاً بدون احراز هویت (`noAuth: true`)
| پیشوند (`prefix/`) | ارائه‌دهنده | وضعیت زنده (2026-11-23) | مدل‌های فعال Chat/Code پیشنهادی | نکات کلیدی و تله‌ها |
|---|---|---|---|---|
| `oc/` | **OpenCode Free** (`opencode.js`) | ✅ زنده | `oc/deepseek-v4.1-flash-free`, `oc/mimo-v2.6-flash-free`, `oc/big-pickle`, `oc/beam-free`, `oc/iceman-free`, `oc/flint-40b-free`, `oc/nemotron-3.5-super-vl-free`, `oc/nemotron-3.5-lightning-free`, `oc/hy4-flash-free`, `oc/tidepool-alpha-free` | **تله ۱:** `oc/longcat-2.5-free` (`410`) و `oc/glm-5.3-flash-free` (`400`) از کاتالوگ زنده حذف شده‌اند.<br>**تله ۲:** `oc/jev-1.13-free` فقط `systemone` است (برای چت نیست).<br>**تله ۳:** نیاز به `wreq-js` با Cipher List مرورگر BoringSSL/Bun (`JA4 5b57614c22b0`) + هدر `opencode/latest/2.0.26/cli` + تگ `<env>` + ابزارهای `read`/`shell` و حذف واژه `terminal` از نام ابزارها! |
| `mmf/` | **MiMo Free** (`mimo-free.js`) | ❌ متوقف (`hidden: true`) | ندارد (`mimo-auto` غیرفعال) | سرویس رایگان مستقیم شیائومی بسته شده است؛ مدل جدید MiMo از طریق `oc/mimo-v2.6-flash-free` در دسترس است. |
| `dv/` | **Devin CLI** (`devin-cli.js`) | ⚠️ محلی (`hidden: true`) | `swe-1.6`, `claude-opus-4.7-*` | نیازمند باینری `devin` روی ماشین سرور (`devin://acp/stdio`) — برای Railway بدون نصب باینری قابل استفاده نیست. |

## ۲. لایه رایگان با ثبت‌نام ساده / OAuth یا API Key رایگان (بدون کارت بانکی)
| پیشوند (`prefix/`) | ارائه‌دهنده | نوع احراز هویت | مدل‌های رایگان برتر | محدودیت / تله |
|---|---|---|---|---|
| `nous/` (یا `ns/`) | **Nous Research Portal** (`nous.js`) | OAuth Device Code (`hermes-cli`) یا API Key رایگان | `nous/tencent/hy4-flash:free` (1M), `nous/upstage/solar-mini4:free` (524K), `nous/stepfun/step-3.7-flash:free` (256K), و در رجیستری: `nous/meituan/longcat-2.5-preview:free`, `nous/stepfun/step-5-preview:free` | جایگزین رایگان اصلی برای خانواده LongCat و StepFun؛ بدون نیاز به کارت بانکی. |
| `openrouter/` | **OpenRouter Free** (`openrouter.js`) | API Key رایگان | `openrouter/inclusionai/ling-3.2-flash:free` (1M), `openrouter/baidu/ernie-5.5-flash-preview:free` (1M), `openrouter/nvidia/nemotron-3.5-super-vl:free` (1M), `openrouter/nvidia/nemotron-3.5-lightning:free` (1M), `openrouter/nvidia/nemotron-3-ultra-550b-a55b:free` (1M), `openrouter/google/gemma-4-31b-it:free` (262K), `openrouter/openrouter/free` | مدل‌های `0/0` رایگان با محدودیت نرخ روزانه بدون نیاز به شارژ برای تست/پشتیبانی. |
| `nvidia/` | **NVIDIA NIM** (`nvidia.js`) | Developer API Key رایگان | `nvidia/deepseek-ai/deepseek-v4-flash`, `nvidia/nvidia/nemotron-3.5-lightning-30b-a3b`, `nvidia/nvidia/nemotron-3-ultra-550b-a55b`, `nvidia/z-ai/glm-5.2`, `nvidia/minimaxai/minimax-m3`, `nvidia/moonshotai/kimi-k2.6` | ۴۰ درخواست در دقیقه رایگان برای اعضای NVIDIA Developer؛ خطای `degraded` روی مدل قفل ۳۰ دقیقه‌ای فقط روی همان مدل می‌گذارد (`modelScoped: true`). |
| `groq/` | **Groq** (`groq.js`) | API Key رایگان | `groq/llama-3.3-70b-versatile`, `groq/meta-llama/llama-4-maverick-17b-128e-instruct`, `groq/qwen/qwen3-32b`, `groq/openai/gpt-oss-120b`, `groq/whisper-large-v3-turbo` | بسیار سریع؛ محدودیت TPM/RPM در هدرهای `x-ratelimit-*`. |
| `cloudflare-ai/` (یا `cf/`) | **Cloudflare Workers AI** (`cloudflare-ai.js`) | Account ID + Token رایگان | `cf/@cf/deepseek-ai/deepseek-v4.1-flash-1017`, `cf/@cf/zai-org/glm-5.3`, `cf/@cf/qwen/qwen3.8-27b`, `cf/@cf/meta/llama-4-scout-17b-16e-instruct` | ۱۰٬۰۰۰ نورون رایگان روزانه؛ جایگزین تمیز برای `glm-5.3` و `deepseek-v4.1-flash`. |
| `kgw/` | **Kilo Gateway** (`kilo-gateway.js`) | API Key رایگان | `kgw/kilo-auto/free`, `kgw/nvidia/nemotron-3-super-120b-a12b:free`, `kgw/nvidia/nemotron-3-ultra-550b-a55b:free` | مناسب برای کانال بک‌آپ. |
| `af/` | **API.airforce** (`api-airforce.js`) | API Key رایگان | `af/gpt-oss-120b`, `af/kimi-k2.7-code` | دارای `forceStream: true`. |
| `gemini/` | **Google AI Studio** (`gemini.js`) | API Key رایگان | `gemini/gemini-3.8-flash`, `gemini/gemini-3.1-pro-preview`, `gemini/gemini-2.5-flash` | نیازمند کلید AI Studio رایگان؛ مراقب سقف RPM. |
| `gc/`, `ag/`, `kr/` | **Gemini CLI / Antigravity / Kiro** | OAuth رایگان (`deprecated: true` / `RISK_NOTICE`) | `ag/gemini-4-hydrogen-high`, `gc/gemini-3.1-pro-preview`, `kr/claude-opus-5.5` | ریسک مسدودسازی اکانت OAuth بالا؛ برای Antigravity حتماً قوانین حذف ۷ خط پرامپت هرمس (`bb399170`) فعال باشد تا `429` جعلی ندهد! |
