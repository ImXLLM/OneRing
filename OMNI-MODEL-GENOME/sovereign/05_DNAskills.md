# 🛡️ `05_DNAskills.md` — DISTILLED ENGINEERING LAWS, INSTINCTS & ANTI-BOT TRAPS
> **Connected To:** `02_DNAwork.md` · `03_DNAmind.md` · `07_LINK_Protocol.md`

---

## 1. THE 4 IRON LAWS OF EXECUTION (FROM `obra/superpowers` & `morluto/rea`)
1. **No Completion Claims Without Fresh Evidence:** Never claim a build/test passed without running the command in the current turn and verifying `exit_code == 0`.
2. **Two-Stage Verification:** Verify **Spec Compliance** (nothing missing, nothing over-engineered) BEFORE checking **Code Quality** (`complexity <= 20`, `max-depth <= 4`, `0` `TODO/FIXME`).
3. **Config Protection Law (`ECC`):** Never weaken linter/clippy rules or delete failing tests to make a build green; fix the underlying source code.
4. **Deep Modules (`Matt Pocock`):** Prefer small interfaces hiding deep, pure-Rust logic (`1 adapter = hypothetical seam; 2 adapters = real seam`).

---

## 2. CRITICAL PRODUCTION TRAPS & BYPASSES
1. **Zero-Card Free LLM Routing (`OpenCode v2.4.1` + `9router` + `freellmapi`):**
   - Primary keyless endpoint: `https://opencode.ai/inference/openai/v1` with `Authorization: Bearer public` and `User-Agent: opencode/latest/2.4.1/opencode`.
   - Active free models (`cost = 0`): `beam-free` (1M), `deepseek-v4.1-flash-free` (200K), `mimo-v2.6-flash-free`, `nemotron-3.5-super-vl-free` (1M), `nemotron-3.5-lightning-free`, `flint-40b-free`, `hy4-flash-free`, `iceman-free`, `tidepool-alpha-free`, `big-pickle`.
   - **Trap 1:** Never call retired models (`longcat-2.5-free`, `glm-5.3-flash-free`, `deepseek-v4-flash-free`).
   - **Trap 2:** Never name any tool `"terminal"` (causes fake `429 FreeUsageLimitError` on OpenCode; use `run_command`).
   - **Trap 3:** Never use the literal `"You are Hermes Agent, built by Nous Research"` prompt header on Antigravity/9router.
2. **Telegram Mini-App & Bot API 9 Security (`tdesktop`):**
   - In `HMAC-SHA256("WebAppData", bot_token)` verification of `initData`, exclude `hash` but **KEEP `signature`**.
   - Client-side `openInvoice` `'paid'` status is UI-only; fulfill orders ONLY on server-side `successful_payment` with idempotent `telegram_payment_charge_id` + CAS version lock.
   - Call `setSwipeBehavior(false)` in Studio canvas so vertical drag does not close the Mini-App.
3. **Studio Anti-Slop Gate (`Anthropic` + `UI/UX Pro Max` + `GPT-6 Astra`):**
   - Ban `#F4F1EA + #D97757` (Claude tell), ban generic `Inter + purple gradient` cards, enforce interactive frame-1 workspace, and enforce `dir="rtl"` + `Vazirmatn` for Persian.