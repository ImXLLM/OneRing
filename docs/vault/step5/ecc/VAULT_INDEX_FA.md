# فهرست گنجینه `affaan-m/ECC` (*Everything Claude Code*) برای `KIAN v2` (`affaan-ecc-gems`)

- **مخزن ممیزی‌شده:** `https://github.com/affaan-m/ECC` (نسخه **`2.2.3`**، کامیت **`fdfca3e0`** + تاریخچه گیت و بررسی ۲۸ مخزن `affaan-m`)
- **بخش‌های کلیدی استخراج‌شده:** ۷۲ پرسونا/ایجنت تخصصی (`agents/`)، ۳۲۸ دایرکتوری مهارت (`skills/`، ۶۶۰ فایل)، ۹۹ دستور (`commands/`)، ۱۲۲ فایل قانون (`rules/`)، هوک‌های بومی Hermes و Node (`hooks/` و `scripts/hooks/`) و کانتکست‌های ۳-حالته (`contexts/`).

## ساختار پوشه‌ها
1. **`01_ecc_agents_and_personas/`**:
   - تمام ۷۲ ایجنت تخصصی در `agents/*.md` همراه با Prompt Defense Baseline و ماتریس ابزار/مدل (`opus`, `sonnet`, `haiku`).
   - ۳ فایل کانتکست در `contexts/` (`dev.md`, `review.md`, `research.md`) + `AGENTS.md`, `SOUL.md`, `VERSION`.
2. **`02_ecc_skills_and_instincts/`**:
   - کل درخت `skills/` (از جمله `continuous-learning-v2`, `strategic-compact`, `verification-loop`, `security-review`, `tdd-workflow`, `eval-harness`, `backend-patterns`, `frontend-patterns`) و ۹۹ دستور در `commands/`.
3. **`03_ecc_rules_and_hooks/`**:
   - تمام ۱۲۲ فایل قانون در `rules/` (`common/`, `typescript/`, `rust/`, `python/`, `golang/`, `react/` و...)، هوک‌های `hooks/` (شامل `hooks/hermes/config-protection.py` و `check-console-log.py`)، اسکریپت‌های `scripts_hooks/` (`suggest-compact.js`, `gateguard-fact-force.js`, `hookify-runtime.js`) و `the-security-guide.md`.
4. **`04_kian_v2_rust_and_db_injector/`**:
   - `inject_ecc_gems.py` — اسکریپت Idempotent تزریق مهارت‌های برگزیده (`kb:skill-ecc-<slug>`)، ۷۲ ایجنت (`kb:agent-ecc-<slug>`)، قوانین مشترک/TS/Rust (`kb:rule-ecc-*`) و کانتکست‌ها (`kb:ctx-ecc-*`) به جدول `"KbEntry"` در `seed/kian-production.db` با حفظ `trialBalanceSum == 0`.
   - `kian_ecc_harness.rs` — ماژول خالص Rust برای `crates/agents` و `crates/hermes` شامل `ContextMode`, `StrategicCompactor`, `Instinct` (`0.3 -> 0.9` + ارتقای بین‌پروژه‌ای) و `is_protected_config_path`.
5. **`MANIFEST.json`**: هش دقیق `SHA-256`، تعداد خطوط و حجم تک‌تک فایل‌ها.
