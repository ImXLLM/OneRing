# فهرست گنجینه `mattpocock/skills` برای KIAN v2 (`mattpocock-skills-gems`)

- **مخزن ممیزی‌شده:** `https://github.com/mattpocock/skills` (نسخه `1.3.2`، کامیت `4fa24fa` + تاریخچه کامل شاخه‌ها و کامیت‌های حذف‌شده)
- **تعداد کل مهارت‌ها:** **۴۷ مهارت** (**۳۹ مهارت فعال** در شاخه `main` + **۸ مهارت تاریخی/شاخه‌ای بازیابی‌شده از گیت**: `constrain`, `animatic`, `setup-animatic`, `spawn`, `resolving-merge-conflicts`, `caveman`, `obsidian-vault`, `edit-article`)

## ساختار پوشه‌ها
1. **`01_complete_skills_catalog/`**: نسخه کامل تمام پوشه‌های `engineering/` (۲۰)، `in-progress/` (۸)، `misc/` (۴)، `productivity/` (۷)، `deprecated/README.md`، `_repo_meta/` (`.agents/` ADRs، `.claude-plugin/`، `scripts/`) و `_historical_and_branches/` (۸ مهارت بازیابی‌شده از گیت).
2. **`02_progressive_disclosure_index/`**:
   - `skills_index.json` — کاتالوگ فشرده JSON شامل `(slug, kb_id, title, category, status, invocation_mode, short_description, triggers, tags)` برای تزریق به لایه `volatile` در سیستم پرامپت ۳-لایه `KIAN v2`.
   - `SKILLS_INDEX.md` — جدول روتر سریع برای انسان و ایجنت.
3. **`03_kian_v2_db_injector/`**:
   - `inject_mattpocock_skills.py` — اسکریپت Idempotent پایتون برای تزریق همه ۴۷ مهارت در جدول `"KbEntry"` فایل `seed/kian-production.db` با شناسه‌های `kb:skill-mp-<slug>` و بررسی سلامت تراز مالی (`trialBalanceSum == 0`).
   - `kian_mattpocock_rules.rs` — ماژول خالص Rust برای `crates/hermes` حاوی قوانین تقطیرشده `codebase-design`، `tdd`، `diagnosing-bugs`، `code-review` (۱۲ بوی کد فاولر) و `writing-for-agents`.
4. **`MANIFEST.json`**: فهرست کامل فایل‌ها با تعداد خط، حجم بایت و هش دقیق `SHA-256`.
