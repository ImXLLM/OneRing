# فهرست گنجینه `anomalyco/opencode` برای `KIAN v2` (`anomalyco-opencode-gems`)

- **مخزن ممیزی‌شده:** `https://github.com/anomalyco/opencode` (`v2.4.1`, کامیت `18ff071`) + بررسی مخازن اکوسیستم `anomalyco` (`models.dev`, `opentui`, `hex`, `rune`, `browse`, `opencode-android`)

## ساختار پوشه‌ها
1. **`01_opencode_providers_and_zen_free/`**:
   - رجیستری ارائه‌دهنده در `packages/ai/src/providers/opencode.ts` (`SystemOne` و `TypeSafe-Danger-Zone-Images`) و `packages/core/src/plugin/provider/opencode.ts` (تزریق خودکار `apiKey: "public"` برای دسترسی رایگان بدون کارت بانکی).
   - فایل `models_dev_opencode_zen.json` استخراج‌شده از `packages/core/src/models-dev/snapshot.txt` شامل تمام مدل‌های فعال و منقضی‌شده (`deprecated`) در `opencode`, `opencode-go`, `zenmux`، به‌همراه `free_models.rs`، `console-access.ts` و `legacy.ts`.
2. **`02_agent_prompts_and_modes/`**:
   - تمام پرامپت‌های اختصاصی خانواده مدل در `system_prompts/` (`anthropic.txt`, `gpt.txt`, `gpt-astra.txt`, `kimi.txt`, `trinity.txt`, `meta.txt`, `default_runner_system.txt`).
   - انتخابگر پویای `optimize.ts`، تزریقگر ابزار `system-prompt.ts` و تعریف ایجنت‌های `Build`، `Plan` (`plan.ts`)، `Explore`، `General`، `Compaction`، `Title` و `Summary` (`agent.ts`).
3. **`03_fuzzy_edit_and_session_engine/`**:
   - موتور کامل ویرایش و پچ (`edit.ts`, `tool_patch.ts`, `util_patch.ts`, `vcs_patch.ts`, `edit.rs`, `patch.rs`, `patch.txt`, `patch-grammar.txt`).
   - موتور اسنپ‌شات و بازگردانی (`snapshot.ts`, `revert.ts`)، فشرده‌سازی نشست (`compaction.ts`)، گیت مجوزها (`permission/`) و حلقه اجرای ایجنت (`step.ts`).
4. **`04_kian_v2_rust_adapters/`**:
   - `kian_opencode_zen_provider.rs`: کلاینت Rust برای `crates/ai` شامل آدرس‌های دقیق `/inference/openai/v1`، هدرهای `Bearer public` و `User-Agent`، نگاشت خودکار مدل‌های منقضی‌شده به ۱۰ مدل رایگان فعال، و انتخابگر پرامپت خانواده مدل.
   - `kian_fuzzy_replacer.rs`: پیاده‌سازی خالص Rust الگوریتم جایگزینی فازی ۵-مرحله‌ای برای ویرایش دقیق فایل‌های HTML/کد در KIAN Studio.
5. **`MANIFEST.json`**: هش `SHA-256`، تعداد خطوط و حجم دقیق تمام فایل‌ها.
