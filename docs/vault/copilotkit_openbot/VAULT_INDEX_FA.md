# فهرست گنجینهٔ استخراج‌شده از سازمان `CopilotKit` و مخزن `CopilotKit/OpenBot`

این آرشیو شامل باارزش‌ترین کدهای تولیدی، پروتکل‌ها، اسکیماها، پرامپت‌ها، کامپوننت‌های Generative UI و بلوپرینت‌های مهندسی استخراج‌شده از سازمان `CopilotKit` (۱۰۷ مخزن) با تمرکز خط‌به‌خط بر `CopilotKit/OpenBot` (۱۳٬۲۹۵ ستاره) و ۸ مخزن خواهر کلیدی آن (`ag-ui`, `channels-sdk`, `OpenIntelligentUI`, `shadify`, `open-multi-agent-canvas`, `open-mcp-client`, `open-research-ANA`, `generative-ui`) است.

## ساختار ۵ پوشهٔ گنجینه

1. **`01_openbot_core/`** — هستهٔ تمیز و بدون زوائد `CopilotKit/OpenBot`:
   - `shared/`: قواعد پرامپت، ارکستراسیون (`delegate`/`follow_up`/`stop`)، شبیه‌ساز سیاست `what-if`، مسکینگ PII خطی، کف امنیتی Ed25519، روتین‌ها و اهداف، و مدیریت نتایج دوزبانه (مدل در برابر انسان).
   - `app_copilot/`: یکپارچه‌سازی `@copilotkit/react-core/v2`، A2UI 1.0، `repair-history.ts`، `settled-args.ts`، `useDecisionTool` چندتماسی، `SandboxedTools` و `server-owned-interrupts.ts`.
   - `components_gallery/`, `components_approvals/`, `components_what_if/`, `components_settings_tools/`, `components_needs_you/`, `components_decisions/`, `skins/` (`opendots`, `openduet`, `openfuse`).
   - `management_agent/`, `agent_langgraph_agui/`, `agent_claude_sdk/`, `desktop_tauri_rust/` (`vault.rs`, `security_floor.rs`, ...) و `extension_shared/` + `docs/`.
2. **`02_openbot_prompts_and_skills/`** — پرامپت‌های مهندسی‌شده، `prompt.txt`, `bot-prompt.ts`, `GENERATIVE_UI_DESIGN_SKILL`, مهارت‌های `.claude/skills/`, قالب‌های YAML (`examples/personal`, `examples/fintech`) و اسکیل‌های MCP از `OpenIntelligentUI`.
3. **`03_ag_ui_protocol_and_schemas/`** — مشخصات پروتکل **AG-UI 1.0**:
   - کریت‌های رسمی **Rust** (`ag-ui-core` و `ag-ui-client`) آمادهٔ تزریق به `crates/hermes` در `KIAN v2`!
   - پکیج‌های TypeScript (`@ag-ui/core`, `@ag-ui/a2ui-toolkit` شامل حلقهٔ خودترمیمی `recovery.ts`) و `openbot_asyncapi.yaml`.
4. **`04_generative_ui_and_canvas_components/`** — بهترین کامپوننت‌های UI از مخازن خواهر:
   - `OpenIntelligentUI` (استریمینگ بدون پرش با `Idiomorph` و `processPartialHtml` + `CSP_META_TAG`).
   - `shadify` (تاب‌آوری استریم و OOM graceful drain).
   - `open-multi-agent-canvas` (بوم چندایجنتی و مدیریت MCP).
   - `open-research-ANA` و `open-mcp-client` (کارت‌های HITL پیشنهادی و پیش‌نمایش تغییر جدول).
   - `channels-sdk` (مهارت ساخت ایجنت برای Slack/Teams/Discord/Telegram).
5. **`05_kian_v2_integration_blueprints/`** — ۵ نقشهٔ اجرایی دقیق برای تزریق مستقیم این گنجینه‌ها به `crates/hermes`, `crates/ai`, `crates/nation`, `crates/security`, `apps/web/app/builder` و `apps/miniapp` در **KIAN v2**.
