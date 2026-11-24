---
file_id: "05_DNAskills"
schema_version: "1.0.0"
layer: skills
kb_table: "KbEntry"
language: "fa-IR + en-US"
---

# 05 — DNAskills: ماتریس مهارت‌های کیان

## 1. طبقه‌بندی اصلی (خانواده‌ها)

| خانواده | نمونه‌های پرچم‌دار | پورت کرنل |
|---|---|---|
| زبان و بند Rust | async runtime، lock-free/CAS، `#![forbid(unsafe_code)]` | همهٔ کریت‌ها |
| پروتکل ایجنت | AG-UI 1.0، HITL دوبل‌صدا، history repair | `crates/hermes/src/ag_ui.rs` |
| تلگرام | Bot API 9.0، TMA bridge، Deep link `p_<id>_r_<tg>`، Stars XTR | `crates/bot/src/telegram_api9.rs` |
| رشد ویروسی | x-algorithm 2026 (کپی‌لینک +20.0، report −234.0)، cold-start Beta(0.75,49.25) | `crates/domain/src/viral_ranker.rs` |
| طراحی | UI/UX Pro Max سه‌اهرم (variance/motion/density)، RTL خودکار، دروازهٔ Astra | `crates/hermes/src/studio_design_contract.rs` |
| امنیت وب | Firecrawl گارد SSRF، دیوار چالش antibot، Jina/Exa صفر-هزینه | `crates/agents/src/agent_reach.rs` |
| فشرده‌سازی | starnet verbatim ۵٪ بودجه، watchdog 450s/1200s | `crates/hermes` + `crates/agents` |
| هرمس | پرامپت ۳لایه، کمپرسور دومرحله‌ای، redaction آموخته | `crates/ai/src/*` |
| گیت‌وی صفر-کارت | OpenCode Zen `Bearer public`، 9Router ضد-429 جعلی، آبشار ۳لایهٔ ۵۲ پلتفرم | `crates/ai/src/opencode_zen.rs` + `free_llm_registry.rs` |
| ویرایش جراحی | پچر ۵گذر fuzzy، exec policy کدکس | `crates/hermes/src/fuzzy_patch.rs` |
| ژنوم شناختی | KIA-BRAIN-OS DNA + کامپایلر پرامپت ۹فایلی مدل‌ها | `crates/hermes/src/creator_dna_contract.rs` + `crates/ai/src/omni_genome_router.rs` |

## 2. دفتر مهارت‌ها (SQLite `KbEntry`)

- جدول حقیقت: `seed/kian-production.db` → `"KbEntry"` (Wave 2 Step 4: **۶۲ ردیف**).
- شناسه‌گذاری: `kb:skill-*`، `kb:arch-*`، `kb:registry-*`، `kb:dna-*`.
- تزریق فقط idempotent؛ تراز آزمایشی و ۲۰ محصول فعال دست‌نخورده.

## 3. زبان‌های Tier-1

Rust (بند مرجع)، TypeScript/React (استودیو و TMA)، Python (ابزار و ممیزی)،
SQL/SQLite (دفتر)، HTML5/CSS تک‌فایل (خروجی Astra).

## 4. الگوی Deep Module برای هر مهارت جدید

1. منبع: ریپوی مرجع + فایل دقیق؛ 2. استخراج: فقط الگوریتم، نه boilerplate؛
3. پورت: Rust خالص + تست قطعی؛ 4. ثبت: ردیف `KbEntry` با منبع و مسیر پورت؛
5. ضدعفونی: صفر راز قبل از commit.
