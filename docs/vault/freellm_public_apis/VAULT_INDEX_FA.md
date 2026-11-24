# گاوصندوق زیرساخت LLM رایگان و APIهای عمومی بدون کلید (`freellmapi` · `public-apis`)

- **مخازن مرجع:**
  - `tashfeenahmed/freellmapi` (`62526e0` + کاتالوگ زندهٔ `2026.11.23.4b4737`)
  - `public-apis/public-apis` (`9fb3bdb`)

## ساختار پوشه‌ها

1. `01_freellmapi_complete_catalog/`
   - `providers_registry.json` و `FREE_LLM_PROVIDERS_COMPLETE.md`: کاتالوگ ساختاریافته تمام **۱۰۴ پلتفرم** تعریف‌شده و **۴۵۴ مدل چت رایگان زنده** (شامل Base URL، نیاز به کارت بانکی، RPM/RPD/TPM/TPD، پشتیبانی Tool-Calling و Streaming).
   - `01-supported-platforms.md`, `keys_shared.tsx`, `providers_index.ts`, `freellmapi_live_catalog_2026-11-23.json`: اسناد و کدهای خام استخراج‌شده از مخزن.
2. `02_zero_auth_public_apis_for_studio/`
   - `public_apis_all_zero_auth_https.json`: کل **۱,۱۵۰ API** دارای `Auth: No` و `HTTPS: Yes` از `public-apis` (شامل ۶۶۵ مورد `CORS: Yes`).
   - `top_120_studio_browser_apis.json` و `TOP_120_ZERO_AUTH_APIS.md`: **۱۲۰ API منتخب برتر** برای اپلیکیشن‌های تک‌فایلی `KIAN Studio (/builder)` همراه با کدهای آمادهٔ `fetch(...)`.
3. `03_kian_v2_rust_and_db_adapters/`
   - `kian_free_llm_registry.rs`: ماژول خالص Rust برای `crates/ai` شامل آرایهٔ `FREE_LLM_PROVIDERS` و توابع انتخاب زنجیرهٔ fallback رایگان.
   - `inject_freellm_and_public_apis.py`: اسکریپت Idempotent پایتون برای تزریق `kb:skill-zero-auth-live-apis` و `kb:registry-free-llm-providers` به جدول `"KbEntry"` در `seed/kian-production.db`.
