# فهرست گنجینه `obra/superpowers` برای `KIAN v2` (`obra-superpowers-gems`)

- **مخزن ممیزی‌شده:** `https://github.com/obra/superpowers` (نسخه **`7.0.2`**، کامیت **`58e107b`**) + بررسی مخازن همراه خانواده `obra` (`superpowers-marketplace`, `superpowers-lab`, `superpowers-chrome`, `superpowers-developing-for-claude-code`, `superpowers-skills`, `episodic-memory`, `muse-skills`)
- **تعداد مهارت‌ها:** **۲۲ مهارت** (**۱۵ مهارت فعال** در `main` + **۷ مهارت تاریخی بازیابی‌شده از گیت**: `proving-it-works-with-a-movie`، `when-stuck`، `collision-zone-thinking`، `inversion-exercise`، `meta-pattern-recognition`، `scale-game`، `simplification-cascades` به همراه راهنماهای `spec-reviewer-prompt.md`، `code-quality-reviewer-prompt.md` و `testing-anti-patterns.md`)

## ساختار پوشه‌ها
1. **`01_complete_superpowers_skills/`**: کل درخت `skills/` (۱۵ مهارت فعال + تمام مراجع `root-cause-tracing.md`, `defense-in-depth.md`, `condition-based-waiting.md`, `anthropic-best-practices.md`, `persuasion-principles.md`, `hermes-tools.md` و پرامپت‌ها/اسکریپت‌های `subagent-driven-development/` و `diagnosing-superpowers/` و `brainstorming/`)، بعلاوه `hooks/` (`SessionStart` و `session-start`)، `.hermes-plugin/`، `.claude-plugin/`، `AGENTS.md` و `_historical/`.
2. **`02_iron_laws_and_workflows/`**:
   - `iron_laws_and_anti_rationalization.json` — پایگاه داده ماشین‌خوان تمام قوانین آهنین، جداول ضد-بهانه‌تراشی و ماشین وضعیت بازبینی دومرحله‌ای.
   - `IRON_LAWS_AND_WORKFLOWS.md` — خلاصه خوانا برای مطالعه و تزریق به کانتکست.
3. **`03_kian_v2_rust_and_db_injector/`**:
   - `inject_superpowers_skills.py` — اسکریپت Idempotent برای درج همه ۲۲ مهارت با شناسه‌های `kb:skill-sp-<slug>` در `"KbEntry"` داخل `seed/kian-production.db` و کنترل `trialBalanceSum == 0`.
   - `kian_superpowers_guard.rs` — گارد خالص Rust (`crates/hermes`) شامل `VerificationBeforeCompletion` (`verify_completion_claim`), `IronLaw` و ماشین وضعیت `TwoStageTaskReview`.
4. **`MANIFEST.json`**: فهرست تمام فایل‌ها با حجم، تعداد خط و هش `SHA-256`.
