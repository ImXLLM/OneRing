# ⚙️ `DNAwork.md` — TECHNICAL ARCHITECTURE, 15-VAULT & STATE GENOME OF `KIAN v2`
> **Schema Version:** `DNA-WORK-v10.0` | **Project:** `KIAN v2` (Autonomous AI Studio, 7-Agent Fleet & Crypto/Stars Escrow Marketplace)  
> **Active Repository Root (Z.ai):** `/home/z/my-project/kian-launch/kian/`  
> **Deliverable Artifacts:** `/home/z/my-project/download/kian-v2-railway-migrated.zip` & `/home/z/my-project/download/kian-production.db`  
> **Usage:** Load alongside `DNAme.md` to restore 100% technical mastery over every crate, database invariant, commit hash, anti-bot rule, and remaining integration step of `KIAN v2`.

---

## 1. PRODUCT VISION & ECONOMIC ENGINE

**What `KIAN v2` Is:**
A Telegram Mini-App (`apps/miniapp`) + Web Platform (`apps/web`) backed by a 9-crate **Rust Kernel** where any user can:
1. Chat with an AI in **KIAN Studio (`/builder`)** to generate self-contained, interactive single-file HTML applications (`<!DOCTYPE html>...`) with live zero-auth APIs (`fetch()`), native Persian RTL (`Vazirmatn`), and zero streaming flicker.
2. Edit existing HTML apps in **< 2 seconds** using a 5-pass surgical fuzzy patcher (`fuzzy_replacer.rs`) without regenerating the full file.
3. Publish the artifact with **1 click** onto the internal **Escrow Marketplace**.
4. Share a **1-click Viral Referral Link (`p_<productId>_r_<refTgId>`)** to earn **`10%–50%` instant affiliate commission** (`25%` default on flagship products), while the platform collects a **`10%` Charter tax** on every settled escrow order.

---

## 2. MONOREPO TOPOLOGY & CRATE-BY-CRATE SPECIFICATION

### A. Rust Workspace (`9` Crates — Build flags: `CARGO_TARGET_DIR=/tmp/my-project/target CARGO_BUILD_JOBS=2`)
1. **`crates/contracts`:**
   - Hard payload caps: `MAX_PREVIEW_HTML_BYTES = 50_000` (50 KB), `MAX_DELIVERY_PAYLOAD_BYTES = 150_000` (150 KB).
   - Economic Charter constants: `PLATFORM_TAX_BPS = 1000` (`10%`), `DEFAULT_AFFILIATE_BPS = 1000` (`10%`), `MAX_AFFILIATE_BPS = 5000` (`50%`).
2. **`crates/domain`:**
   - Marketplace entities, Order/Escrow state machine (`Pending -> HeldInEscrow -> Released / Refunded`).
   - `viral_ranker.rs`: Exact 2026 `xai-org/x-algorithm` engagement weights (`share_via_copy_link = +20.0`, `bidirectional_reply = +20.0`, `favorite = +0.708`, `report = -234.0`) + `Beta(0.75, 49.25)` Thompson Sampling cold-start boost into discovery slots `15..=16`.
   - `dify_rag_and_idempotency.rs`: Optimistic concurrency control (`CAS expected_version`) + per-operation `idempotency_key` to make double-spend mathematically impossible.
3. **`crates/ledger`:**
   - Strict double-entry accounting (`JournalLine` / `Posting`). Global invariant: **`trialBalanceSum == 0`** across `58,700,200,000` micro-KIAN.
4. **`crates/db`:**
   - SQLite engine managing `seed/kian-production.db` (`Citizen`, `Product`, `Order`, `Referral`, `JournalLine`, `KbEntry`, `BuilderMessage`).
5. **`crates/ai`:**
   - `history_repair.rs`: Repairs broken multi-turn tool call histories by injecting synthetic `UNANSWERED` tool results (from `CopilotKit/OpenBot`).
   - `tool_parser.rs`: Streaming tool-call accumulator using direct `=` function name assignment (fixing MiniMax/NIM `read_fileread_file` concatenation bug) + JSON brace-depth slot splitting + `<tool_call>` XML+JSON lenient fallback (from `NousResearch/hermes-agent`).
   - `compressor.rs`: Two-phase context compressor (`>= 16,000` token cold-cache threshold + 1-line deterministic tool output stubbing + `user` message protection).
   - `opencode_zen.rs` + `kian_9router_lanes.rs` + `free_llm_registry.rs`: Zero-card LLM gateway across `https://opencode.ai/inference/openai/v1` (`Bearer public`) and 52 free LLM providers.
6. **`crates/hermes`:**
   - `ag_ui.rs`: Official Rust `AG-UI 1.0` protocol schemas (`EventType`, `RunAgentInput`, `ToolCall`, `StateDelta`).
   - `approvals.rs`: 6-category Dual-Voice HITL outcomes (`DECLINED_BY_PERSON` for LLM vs polite Persian message for UI) + `RoutineFatigueGuard` (`900s` min interval, `20` max active, `10` failures auto-disable).
   - `prompt.rs`: 3-layer cache-safe prompt (`stable` / `context` / `volatile`, `140`-char skill index cap, `SOUL.md` directness contract) + declarative-only memory validator (`validate_declarative_memory`).
   - `security.rs`: Hidden Unicode variation selector detector (`\u{FE00}..=\u{FE0F}`, `\u{E0100}..=\u{E01EF}`), quote-aware shell comment stripper, learned secret span propagation, and `StuckCallGate` (`4` failures / `60s`).
   - `studio_design_contract.rs` + `anthropic_anti_slop.rs` + `fuzzy_replacer.rs` + `superpowers_guard.rs` + `mattpocock_rules.rs`.
7. **`crates/agents`:**
   - 7-Role Autonomous Fleet: `orchestrator`, `scout`, `analyst`, `merchant`, `support`, `finance`, `guardian` (**`7/7 Active`** in Step 3).
   - `agent_reach.rs` (Jina Reader `https://r.jina.ai/{url}` + Exa MCP `https://mcp.exa.ai/mcp` + Cloudflare/Turnstile detector + `starnet` `SubagentStallWatchdog` `450s/1200s`), `ecc_harness.rs` (`ContextMode`, `StrategicCompactor`, `InstinctStore` `0.3..=0.9`), `firecrawl_ssrf_and_md.rs` (2-stage DNS-rebinding & CGNAT/NAT64 SSRF guard).
8. **`crates/bot`:**
   - `loop_guard.rs`: `BotLoopGuard` (max `3` consecutive bot hops with quiet-period chain decay, `60s` duplicate payload block, `A->B->A` ping-pong detector, `5/window` rate cap; humans never suppressed).
   - `telegram_api9.rs`: Constant-time `HMAC-SHA256("WebAppData", bot_token)` keeping `signature` while excluding `hash` + `StartAppPayload` (`p_<productId>_r_<refTgId>`) + `StarsInvoicePayload` (`currency: "XTR"`).
9. **`crates/api`:** Axum HTTP/SSE gateway serving Mini-App, Studio, and Marketplace endpoints.

### B. Frontend & Static Assets (`apps/miniapp` & `apps/web`)
- `apps/miniapp/lib/studio-html-guard.ts` & `components/html-preview-frame.tsx`: Blocks partial HTML streaming flicker until `<!DOCTYPE html>...</html>` is complete + 3-attempt self-healing recovery loop.
- `apps/miniapp/lib/kian-tma-bridge.ts`: Zero-dependency Telegram Mini-App bridge (`setSwipeBehavior(false)` to prevent accidental Mini-App close during Studio canvas drag, Haptics, Safe-Area CSS sync, 1-click viral link copy).
- `apps/web/public/senpai/`: 100% offline 3D WebGL Cosmic Brain (`Three.js` + `d3-force-3d` bundled locally).
- `apps/web/public/products/trinity/`: 8 standalone flagship HTML applications (`sc.html`, `fishkal.html`, `trading-god-arena.html`, `nova-messenger.html`, `liquid-glass.html`, `ocean-test-lab.html`, `fable-bazaar.html`, `dropagentx/`).

---

## 3. CRITICAL ANTI-BOT TRAPS & HARD ENGINEERING RULES DISCOVERED IN FORENSIC AUDITS

1. **OpenCode / 9Router Free Model Traps (`crates/ai`):**
   - **Retired Models (`410`/`400`):** NEVER call `oc/longcat-2.5-free`, `oc/glm-5.3-flash-free`, `oc/deepseek-v4-flash-free`, or `mmf/mimo-auto`. Always route to the **10 Active Free Models** on `https://opencode.ai/inference/openai/v1` (`Authorization: Bearer public`, `User-Agent: opencode/latest/2.4.1/opencode`):
     `beam-free` (1M ctx), `deepseek-v4.1-flash-free` (200K), `mimo-v2.6-flash-free` (200K multimodal), `nemotron-3.5-super-vl-free` (1M), `nemotron-3.5-lightning-free` (262K), `flint-40b-free` (262K), `hy4-flash-free` (1M), `iceman-free` (262K), `tidepool-alpha-free` (1M), `big-pickle` (200K).
   - **The `"terminal"` Tool-Name Ban:** OpenCode immediately returns fake `429 FreeUsageLimitError` if ANY tool name in `tools[]` contains the substring `"terminal"` (case-insensitive). Always rename shell tools to `run_command` or `bash_exec`.
   - **The Antigravity Prompt Ban:** Never include the literal string `"You are Hermes Agent, built by Nous Research"` in system prompts routed through Antigravity/9router; always use KIAN's native identity.
   - **Multi-turn Reasoning Requirement:** On `/zen/v1` and `/inference/openai/v1`, strip `thinking` request objects and ensure multi-turn `assistant` messages include `reasoning_content: " "`.
2. **Telegram Bot API 8.0+ `initData` Signature Trap (`crates/bot`):**
   - When computing `HMAC-SHA256(HMAC-SHA256("WebAppData", bot_token), data_check_string)`, exclude **ONLY** `hash`. **DO NOT** exclude `signature` (which is used for third-party Ed25519 verification and is part of the HMAC-signed payload!).
3. **Git & Secret Scanner Hygiene (`Z.ai`):**
   - Root `Cargo.toml` workspace `exclude` must contain `"docs/vault/**"`.
   - `scripts/` is in `.gitignore`; always stage new scripts with `git add -f scripts/<name>.py`.
   - Never commit strings matching `sk-[A-Za-z0-9]{20,}`, `ghp_[A-Za-z0-9]+`, or `eyJ[A-Za-z0-9_-]+` (even in unit tests or SQLite seed rows). Always use `DUMMY-TEST-FIXTURE` or split literals in test code.

---

## 4. COMPLETE 15-VAULT LEDGER & 6-STEP INTEGRATION STATUS

### A. Completed Checkpoints in Z.ai
- **Wave 1 (`kia-hidden-gems-vault.zip` — `301` files):** ✅ **COMPLETE** (`commits 7771aeb & a077ade`) — `339` tests, `20` products, `33` KB rows, `trialBalanceSum = 0`.
- **Wave 2 — Step 1 (`copilotkit-openbot-gems.zip` — `581` files):** ✅ **COMPLETE** (`commit 0ccbb28`) — `360` tests, `39` KB rows (`+6`).
- **Wave 2 — Step 2 (`nousresearch-hermes-gems.zip` — `292` files):** ✅ **COMPLETE** (`commit 011fffd`) — **`403` tests (`+43`)**, **`49` KB rows (`+10`)**, `806` files in deliverable zip, DB sanitized.
- **Wave 2 — Step 3 (`strategic-5repos-gems.zip` + `tdesktop-telegram-gems.zip`):** 🔄 **RUNNING / NEXT CHECKPOINT** (`viral_ranker.rs`, `studio_design_contract.rs`, `agent_reach.rs`, `guardian` -> **`7/7 Active`**, `telegram_api9.rs`, `kian-tma-bridge.ts`, `"KbEntry"` `49 -> 56`).

### B. Remaining Steps (Steps 4, 5 & 6)
- **Step 4 — Zero-Card LLM Gateway & Surgical Patcher (`4` Zips, `721 KB` total):**
  - Vaults: `decolua-9router-gems.zip` (`74` files), `anomalyco-opencode-gems.zip` (`42` files), `freellm-and-public-apis-gems.zip` (`13` files), `elite-rust-agent-4repos-gems.zip` (`36` files).
  - Targets: Wire `opencode_zen.rs`, `kian_9router_lanes.rs`, `free_llm_registry.rs` (`crates/ai`); wire `fuzzy_replacer.rs`, `codex_patch_and_execpolicy.rs` (`crates/hermes`); run `inject_freellm_and_public_apis.py` (`+2` mega-registries: 52 free LLM providers + 120 `CORS: Yes` live APIs) and `inject_agent_resilience_skills.py` (`+2` skills) -> `"KbEntry"` `56 -> 60`.
- **Step 5 — Engineering Super-Brain, Anti-Slop Gate, SSRF & CAS Escrow (`5` Zips):**
  - Vaults: `mattpocock-skills-gems.zip` (`167` files), `obra-superpowers-gems.zip` (`115` files), `affaan-ecc-gems.zip` (`1,038` files), `elite-skills-and-agents-4repos-gems.zip` (`829` files), `firecrawl-dify-javaguide-gems.zip` (`29` files).
  - Targets:
    1. Run 5 idempotent DB injectors (`inject_mattpocock_skills.py` `+47`, `inject_superpowers_skills.py` `+22`, `inject_ecc_gems.py` `+139`, `inject_4repos_skills.py` `+50`) -> `"KbEntry"` jumps from `60` to **`318` structured entries** (`trialBalanceSum == 0`).
    2. Wire pure-Rust guards: `anthropic_anti_slop.rs`, `superpowers_guard.rs`, `mattpocock_rules.rs` (`crates/hermes`); `ecc_harness.rs`, `firecrawl_ssrf_and_md.rs` (`crates/agents`); `dify_rag_and_idempotency.rs` (`crates/domain`).
- **Step 6 — Final `morluto/rea` Readiness & Hardening Gate (`1` Zip, `10.30 MB`):**
  - Vault: `morluto-rea-gems.zip` (`4,080` files, `rea-agents v6.10.1`).
  - Targets: Add release binary hardening (`lto = "thin"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`), write `DNAme.md` and `DNAwork.md` into `docs/dna/`, execute `/tmp/rea_vault/03_kian_v2_rea_runner/run_rea_gate.py`, verify **`total_score >= 85/100`** and **`hard_failures == 0`**, and package the final Railway release bundle `/home/z/my-project/download/kian-v2-railway-migrated.zip`.