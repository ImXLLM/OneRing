# فهرست گنجینهٔ ادغام‌شده در KIAN v2 (`docs/vault/`) — نسخهٔ ۲۴ نوامبر ۲۰۲۶

گنجینهٔ مخفی `kia-hidden-gems-vault` (۳۰۱ فایل از ۱۹ مخزن `KiaAgentX`) یکپارچه شد.
این سند، نقشهٔ همان‌جاییِ همهٔ دارایی‌های ادغام‌شده است — هیچ فایلی گم نشده.

## ساختار این پوشه

| زیرپوشه | محتوا | مبدأ |
|---|---|---|
| `playbooks/` | `VAULT_INDEX_FA.md`، `DAY1-SALES-KIT.md`، `MARKETING-PLAYBOOK-FA.md`، `V3_UNIFIED_BLUEPRINT.md`، `SOUL.md`، `PRICING.md`، `VALUATION.md`، `projects.json`، `ADR-009-Rust-Migration-Roadmap.html` | پوشه‌های ۰۷ و ۰۹ گنجینه |
| `engines/` | `blockchain.py` (وریفای ۵زنجیره)، `identity_rl.py`، `memory2.py`، `prompt_cache.py`، `token_saver.py`، `sqlsafe.py`، `iron_egress.py`، `x402_seller.py`، `kya_verify.py`، `crm.service.ts`، `booking.service.ts` | پوشه‌های ۰۲، ۰۴، ۰۵ |
| `miniapp_30pages/` | کل ۳۰ صفحهٔ TSX مینی‌اپ (`CheckoutPage`، `EscrowPage`، `NationPage`، …) | پوشهٔ ۰۶ (`src/pages/`) |

## دارایی‌های ادغام‌شده بیرون از این پوشه

* **محصولات ترینیتی (۸ پرچم‌دار)** → `apps/web/public/products/trinity/`
  (`sc.html`، `fishkal.html`، `trading-god-arena.html`، `nova-messenger.html`،
  `liquid-glass.html`، `ocean-test-lab.html` (با CSS ابزار Tailwind درون‌خطی
  سازگار CSP + `ocean-assets/three.min.js`)، `fable-bazaar.html` +
  `fable-bazaar.assets/`، `dropagentx/`) — و ردیف‌های `"Product"` در
  `seed/kian-production.db` با شناسه‌های `prod:trinity-*` (فروشندهٔ
  `citizen:8198598635`، پورسانت ۲۵٪، آزادسازی آنی اسکرو زیر سقف HITL).
* **مغز سه‌بعدی SenPai (ELON EDITION)** → `apps/web/public/senpai/` با
  `vendor/three.min.js` آفلاین کامل (مسیرهای نسبی — صفر وابستگی CDN)، فونت
  وزیرمتن در `senpai/fonts/`، و `agent-graph.js` در `senpai/agent-graph.js`.
* **۳۳ مدخل پایگاه دانش** → جدول `"KbEntry"` در `seed/kian-production.db`:
  ۱۶ مهارت تجاری (`kb:skill:*`)، ۱۱ مهارت مهندسی (`kb:eng:*`)، `SOUL` و
  `PRICING` (`kb:eng:soul`، `kb:eng:pricing`) و ۴ بازی‌نامه
  (`kb:playbook:day1-sales-kit`، `kb:playbook:marketing-fa`،
  `kb:playbook:v3-blueprint`، `kb:playbook:valuation`).
* **پورت‌های Rust در کرنل** (همه با تست):
  * `crates/hermes/src/prompt.rs` — قفل تایپوگرافی RTL (پورت `bot.py`)؛
  * `crates/hermes/src/faq.rs` — پاسخ‌گر فوری FAQ با صفر توکن (۵+ موضوع کلیدی)؛
  * `crates/hermes/src/identity.rs` — معمارهٔ کاربر قطعی (پورت `identity_rl.py`)؛
  * `crates/hermes/src/token_saver.rs` — فشرده‌ساز با حفظ راز (پورت `token_saver.py`)؛
  * `crates/bot/src/backlog.rs` — تخلیهٔ عقب‌ماندگی (پورت `drain_backlog`)؛
  * `crates/agents/src/kya.rs` — شناسنامهٔ KYA/ERC-8004 + دروازهٔ x402 v2؛
  * `crates/agents/src/roles.rs` — ناوگان ۶+۱ فعال (پورت `AgentX`)؛
  * `crates/tooling` — سه نبودِ fuzz به‌صورت تست قطعی (پورت fuzz هدف‌های
    `fuzz_ai_chat`، `fuzz_create_escrow`، `fuzz_mint_request`)؛ اسکلت خام
    cargo-fuzz در `crates/tooling/fuzz_suite/` و جعبه‌ابزار
    dual-run/parity در `crates/tooling/vault_pro/` (مرجع — هرگز کامپایل نمی‌شوند).
* **دیتابیس مرجع** → `docs/vault/playbooks/projects.json` و `VALUATION.md`
  (~$758K ارزش‌گذاری مهندسی کیا).

## اسکریپت تزریق

`scripts/inject_vault_gems.py` — idempotent؛ اجرای مجدد فقط «skipped» می‌دهد.
تراز آزمایشی دفتر (مجموع `Posting` = صفر با اعتبار ناخالص ۵۸٬۷۰۰٬۲۰۰٬۰۰۰
micro-KIAN) هرگز دست نمی‌خورد.
