# فهرست گنجینهٔ استخراج‌شده از سازمان `NousResearch` و مخزن `NousResearch/hermes-agent`

این آرشیو شامل باارزش‌ترین کدهای تولیدی، سیستم پرامپت سه‌لایه، فشرده‌ساز دو مرحله‌ای کانتکست، **۴۴ مهارت آمادهٔ `SKILL.md`**، موتور امنیت و مسکینگ، پارسرهای `Hermes-Function-Calling`، ارزیاب `hermes-compression-eval`، بهینه‌ساز تکاملی `hermes-agent-self-evolution` و نقشه‌های انتقال به **`KIAN v2`** است.

## ساختار ۶ پوشهٔ گنجینه

1. **`01_hermes_agent_core_loop/`** — حلقهٔ اصلی `run_agent.py`، پرامپت سه‌لایهٔ سازگار با کش (`system_prompt.py`, `prompt_builder.py`, `SOUL.md`)، انباشتگر استریم ابزار (`chat_completion_helpers_tool_calls.py`)، فشرده‌ساز کانتکست و هرس خروجی ابزار (`context_compressor*.py`, `micro_compaction.py`)، ساب‌ایجنت (`delegate_tool*.py`) + `hermes-compression-eval` و `hermes-agent-self-evolution`.
2. **`02_hermes_builtin_skills_catalog/`** — تمام **۴۴ فایل `SKILL.md`** دسته‌بندی‌شده از `skills/` به همراه موتور کشف/بارگذاری/قفل/اسکن امنیتی مهارت‌ها (`skill_utils.py`, `skill_manager_tool.py`, `skill_manager_scan.py`, `skills_guard.py`, `skills_ast_audit.py`).
3. **`03_memory_and_session_fts/`** — مدیریت حافظه (`memory_manager.py`, `memory_tool.py`)، جستجوی ترکیبی FTS5 + Trigram و واکشی پنجرهٔ پیام (`persistence/sqlite/conversation/_store/search.py`)، و تعمیر تراکنش تاریخچه (`transcript_repair.py`).
4. **`04_gateway_telegram_and_cron/`** — آداپتور کامل تلگرام و A2A (`plugins/platforms/{telegram,a2a}`), گیت‌وی و گارد حلقهٔ بات‌ها (`bot_loop_guard.py`, `delivery_ledger.py`)، زمان‌بند کرون (`cron/{scheduler,missed_runs,prompt_scan}.py`)، موتور مسکینگ اسرار (`agent/redact*.py`) و دروازهٔ تأیید هوشمند دستورات شل (`tools/approval*.py`, `threat_patterns.py`).
5. **`05_hermes_function_calling_schemas/`** — کل ابزارها، قالب‌های Jinja، پرامپت‌های `<tools>` / `<tool_call>` و ولیدیتورهای JSON/XML از مخزن `NousResearch/Hermes-Function-Calling`.
6. **`06_kian_v2_rust_port_blueprints/`** — ۵ نقشهٔ اجرایی پورت به Rust و SQLite برای `crates/hermes`، `crates/ai`، `crates/bot` و جدول `"KbEntry"` در `seed/kian-production.db`.
