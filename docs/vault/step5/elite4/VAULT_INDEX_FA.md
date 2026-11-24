# فهرست گنجینه ۴ مخزن برگزیده (`elite-skills-and-agents-4repos-gems`)

1. **`01_anthropics_official_skills/`**: تمام ۱۹ مهارت رسمی Anthropic (`frontend-design`, `web-artifacts-builder`, `mcp-builder`, `webapp-testing`, `skill-creator`, `algorithmic-art`, `canvas-design`, `brand-guidelines`, `theme-factory`, `discernment-nudge`, `docx`, `pdf`, `pptx`, `xlsx` و...) به‌همراه `_spec` و `_template`.
2. **`02_agency_agents_personas/`**: ۸ بخش تخصصی از `msitarzewski/agency-agents` (`design`, `engineering`, `marketing`, `testing`, `support`, `product`, `finance`, `specialized` + `divisions.json`).
3. **`03_claw_code_harness_gems/`**: کریت‌های Rust (`runtime`, `api`, `tools`, `commands`, `plugins`, `compat-harness`) و هسته پایتون (`python_core/*.py`) از `ultraworkers/claw-code` به‌همراه `PHILOSOPHY.md` و `PARITY.md`.
4. **`04_ponytail_gems/`**: مهارت‌ها، دستورات، هوک‌های میان‌افزار ۲۰ ایجنت، نقشه‌ساز سریع کدبیس (`ponytail-map.js`) و اکستنشن Pi از `DietrichGebert/ponytail`.
5. **`05_kian_v2_injectors/`**:
   - `inject_4repos_skills.py`: تزریق‌گر Idempotent مهارت‌های Anthropic + پرسونای `agency-agents` + مهارت‌های `ponytail` به جدول `"KbEntry"` در `seed/kian-production.db` با تضمین `trialBalanceSum == 0`.
   - `kian_anthropic_anti_slop.rs`: ماژول پرامپت و اعتبارسنجی Rust برای `crates/hermes` جهت اعمال قوانین ضد ظاهر کلیشه‌ای هوش مصنوعی (`frontend-design` و `web-artifacts-builder`).
6. **`MANIFEST.json`**: هش `SHA-256`، تعداد خطوط و حجم دقیق تمام فایل‌ها.
