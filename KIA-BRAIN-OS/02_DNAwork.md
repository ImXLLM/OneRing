---
file_id: "02_DNAwork"
schema_version: "1.0.0"
layer: architecture
product: "KIAN v2"
valuation: "COCOMO II"
language: "fa-IR + en-US"
---

# 02 — DNAwork: معماری کاری KIAN v2 و قوانین آهنین دفتر

## 1. ارزش‌گذاری رسمی (COCOMO II — ممیزی 2026-11-24)

| قلم | مقدار |
|---|---|
| ارزش مستقل KIAN v2 | **$523,750** |
| ارزش ترکیبی با اکوسیستم ۶۰ پروژهٔ KiaAgentX | **~$1.18M** |
| روش | COCOMO II بر روی خطوط واقعی کرنل Rust + تست‌های سبز |

## 2. معماری کرنل — ۹ کریت Rust

| کریت | مسئولیت |
|---|---|
| `kian-domain` | مدل‌های دامنه، رتبه‌دهندهٔ ویروسی x-algorithm 2026 |
| `kian-contracts` | API contracts — نسخه‌بندی و اعتبارسنجی |
| `kian-db` | لایهٔ SQLite (WAL، تراکنش دوبل) |
| `kian-wallet` | دفتر کل دوبل (Posting/Tx)، TON، USDT، Stars |
| `kian-nation` | شهرک دیجیتال: شهروند، رفرال، مجلس (Bill/BillVote) |
| `kian-ai` | گیت‌وی AI: سه اسلات فال‌بک، بودجهٔ سخت، redaction، ژنوم مدل‌ها |
| `kian-hermes` | مغز تجاری: سپر، مهارت‌ها، حافظهٔ ۷لایه، DNA سازنده |
| `kian-agents` | ناوگان ۷نقشی، Agent-Reach، watchdog توقف |
| `kian-bot` + `kian-server` | Bot API 9.0 / TMA + HTTP server |

## 3. قوانین آهنین دیتابیس تولید (Production SQLite Invariants)

- شهروندان: **۴۶۶** (جدول `User`) — هرگز با تزریق دستکاری نمی‌شود.
- خطوط رفرال: **۲۴۷** (`ReferralLineage`) — سازوکار وابستگی لِین حفظ می‌شود.
- سفارش‌ها/اسکرو: **۱۰۸** (`Escrow`) — چرخهٔ سفارش بدون اسکرو وجود ندارد.
- محصولات فعال: **۲۰** (`Product.active=1`) — فقط با تراکنش واقعی تغییر می‌کند.
- تراز آزمایشی: `SUM(Posting.amount) = 0` روی **۵۸٬۷۰۰٬۲۰۰٬۰۰۰ micro-KIAN**
  — دفتر دوبل همیشه متوازن؛ هر تزریق مهارت باید این رقم را دست‌نخورده نگه دارد.
- شناسه‌ها: `KbEntry` از `kb:` prefix استفاده می‌کند و تزریق‌ها **idempotent**اند
  (مقایسهٔ title+body، ردِ تکراری).

## 4. ناوگان ایجنت‌ها — ۷/۷ فعال

`scout`، `builder`، `broker`، `envoy`، `judge`، `sage`، **`guardian`** (هندلر
امنیتی/راستی‌آزمایی). هیچ نقشی Registered باقی نمی‌ماند؛ نقش بدون هندلر واقعی
پرونده می‌شود نه مخفی.

## 5. قوانین آهنین کار

1. **بدون راز در گیت:** هیچ کلید/توکنی به مخزن نمی‌رود؛ گنجینه‌ها با
   `DUMMY-TEST-FIXTURE` ضدعفونی می‌شوند.
2. **عدد بدون اجرا ممنوع:** هر ادعای عملکردی باید خروجی واقعی
   (`cargo test`/`cargo clippy`/JSON زنده) داشته باشد.
3. **`#![forbid(unsafe_code)]`** روی همهٔ کریت‌ها؛ clippy با `-D warnings` تمیز.
4. **cargo ایزوله:** `CARGO_TARGET_DIR=/tmp/my-project/target CARGO_BUILD_JOBS=2`.
5. **Deep Module > boilerplate:** هر ماژول رابط کوچک و پیاده‌سازی عمیق دارد.
