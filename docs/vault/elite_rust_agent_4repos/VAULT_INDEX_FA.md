# فهرست گنجینه مهندسی ۴ مخزن مرجع ایجنتی (Codex / Claude Code / Langflow / LangChain)

این آرشیو شامل فایل‌های اصلی استخراج‌شده از مخازن رسمی زیر به همراه آداپتورهای آماده `KIAN v2` (Rust و SQLite Seeder) است:

1. **`01_codex_rs_rust_engine/`** (`openai/codex` @ `93752dd`)
   - موتور `apply-patch` (`seek_sequence.rs`، `parser.rs`، `lib.rs`) با تطبیق ۴مرحله‌ای فازی و نرمال‌سازی یونیکد
   - سندباکس لینوکس (`linux_run_main.rs`، `seccomp.rs`) با Bubblewrap + `PR_SET_NO_NEW_PRIVS` + Seccomp BPF
   - پروفایل Seatbelt مک (`seatbelt_base_policy.sbpl`)، قوانین `example.codexpolicy`، پرامپت Guardian (`guardian_prompt.rs`)، پرامپت اصلی `gpt-5.2-codex_prompt.md` و کلاینت بومی MCP (`rmcp_client.rs`)

2. **`02_claude_code_security_and_hooks/`** (`anthropics/claude-code` @ `070e7de`)
   - فایروال خروجی کانتینر (`init-firewall.sh`) با `iptables` + `ipset` و تست‌های مثبت/منفی اجباری
   - قوانین سخت‌سازی CI/CD (`egress-firewall.yaml`، `CLAUDE.md`)
   - هوک‌های امنیتی و حریم خصوصی سازمانی (`sec-default-register.ts`، `privacy-README.md`) و اسلش‌کامندهای رسمی (`commit-push-pr.md`، `triage-issue.md`)

3. **`03_langflow_middleware_and_flows/`** (`langflow-ai/langflow` @ `21381e5`)
   - میان‌افزارهای تاب‌آوری ایجنت: `placeholder_corrective_middleware.py`، `single_tool_call_middleware.py`، `tool_approval.py` (HITL مبتنی بر nonce `interrupt_id`)
   - رپر امنیتی `guarded_tool.py` (`ToolGuard`) و کاتالوگ کامل پرامپت‌های ۲۷ جریان آماده (`starter_projects_prompts_catalog.json`)

4. **`04_langchain_parsers_and_retry/`** (`langchain-ai/langchain` @ `fb10639`)
   - پارسرهای سخت‌گیرانه و تشخیص امتناع مدل (`format_instructions.py`، `json.py`، `xml.py`، `_refusal.py`)
   - الگوهای خودترمیمی خروجی و ابزار (`retry.py`، `fix.py`، `prompts.py`، `tool_retry.py`)
   - پرامپت‌های استاندارد `StructuredChat` و `MRKL/ReAct`

5. **`05_kian_v2_rust_and_db_adapters/`**
   - `kian_codex_patch_and_execpolicy.rs`: ماژول خالص Rust برای تطبیق ۴مرحله‌ای `seek_sequence`، ارزیاب قوانین دستورات `ExecPolicy`، تشخیص آرگومان‌های placeholder و ساخت پرامپت ترمیم خطا
   - `inject_agent_resilience_skills.py`: سیدر idempotent برای تزریق دو مهارت جامع به جدول `"KbEntry"` در `seed/kian-production.db`
