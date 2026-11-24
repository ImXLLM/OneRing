# قواعد ارزیابی، حدود عددی و آستانه‌های عبور/رد در `morluto/rea` (`rea-agents` v6.10.1)

## توضیح بنیادین دربارهٔ ماهیت واقعی `morluto/rea`
مخزن `morluto/rea` یک موتور تحلیل استاتیک، مهندسی معکوس و تایید شواهد است (**REA: Reverse Engineer Anything**)، نه یک نمره‌دهندهٔ دلخواه ۰ تا ۱۰۰؛ اما برای تضمین کیفیت مهندسی خود و پروژه‌های هدف (`Rust`، `TypeScript/Next.js`، `SQLite`) دو مکانیزم دقیق و قطعی دارد:
1. **آستانه‌های سخت‌گیرانهٔ CI و معماری (`scripts/verify/` و `.oxlintrc.json`)** — خروج با کد غیرصفر در صورت نقض هر قانون.
2. **دفتر کل شواهد و بازسازی (`docs/reconstruction-coverage.md`)** — وضعیت‌های سه‌گانهٔ `pass` / `fail` / `unknown` (هرگز استنتاج تحلیلی بدون شاهد `observed` به‌عنوان `pass` یا `complete` پذیرفته نمی‌شود).

## ۱. جدول آستانه‌های عددی رسمی در `morluto/rea`
| بخش / فایل منبع | معیار | حد مجاز / آستانه | اثر نقض |
|---|---|---|---|
| `.oxlintrc.json` | پیچیدگی سیکلوماتیک (`complexity`) | `<= 20` در هر تابع | Warning / Gate |
| `.oxlintrc.json` | عمق تودرتویی (`max-depth`) | `<= 4` | Warning / Gate |
| `.oxlintrc.json` | طول فایل (`max-lines` بدون کامنت/خط خالی) | `<= 500` | Warning / Gate |
| `.oxlintrc.json` | طول تابع (`max-lines-per-function`) | `<= 100` | Warning / Gate |
| `.oxlintrc.json` | تعداد پارامتر (`max-params`) | `<= 4` | Warning / Gate |
| `scripts/scan-todos.mjs` | نشانگرهای `TODO` / `FIXME` / `HACK` | `0` مورد خارج از Baseline | **Fail (`exit 1`)** |
| `scripts/verify/architecture-guards.mjs` | پیش‌فرض کردن `complete = true` یا `?? true` | `0` مورد | **Fail (`exit 1`)** |
| `scripts/verify/architecture-guards.mjs` | کلید ترکیبی با `\0` (`.join("\0")`) | `0` مورد (استفاده از `compositeKey`) | **Fail (`exit 1`)** |
| `scripts/lib/module-layers.mjs` | جهت وابستگی ۱۰ لایه (`kernel -> ... -> adapters`) | فقط رو به پایین/هم‌سطح | **Fail (`exit 1`)** |
| `src/rust/RustAuditableReader.ts` | اندازهٔ فایل باینری Rust / بخش `.dep-v0` | فایل `<= 256 MiB`، فشرده `<= 1 MiB`، بازشده `<= min(8 MiB, 64x)` | **ParseFailure** |
| `src/native/hardening/ElfHardening.ts` | سخت‌سازی باینری تولیدی | `PIE (ET_DYN)`, `NX (PT_GNU_STACK !PF_X)`, `RELRO (PT_GNU_RELRO + BIND_NOW)` | گزارش ردیف‌بندی‌شده |
| `scripts/verify/rea-numeric-limits.mjs` | خواندن `process.env.REA_*` ثبت‌نشده | `0` مورد ثبت‌نشده در `ANALYSIS_SETTINGS` / `NUMERIC_LIMIT_REGISTRY` | **Fail (`exit 1`)** |

## ۲. فرمول گیت ترکیبی برای `KIAN v2` (`run_rea_gate.py`)
گیت آمادهٔ `run_rea_gate.py` در پوشهٔ `03_kian_v2_rea_runner/` تمام قوانین استاتیک `morluto/rea` را با چک‌های Rust/Next.js/SQLite ترکیب می‌کند:
- **خطوط قرمز (Hard Fail — حتی با امتیاز بالای ۸۵ باعث `exit 1` می‌شوند):**
  1. وجود کلید/توکن واقعی لورفته (`sk-`, `ghp_`, `xoxb-`, `AIza`, یا فایل `.env` ردیابی‌شده در git)
  2. خرابی یکپارچگی دیتابیس SQLite (`PRAGMA integrity_check != "ok"`) یا نبود جدول `"KbEntry"`
  3. وجود ماکروی ناتمام `todo!()` یا `unimplemented!()` در کد غیرتستی Rust (`crates/`)
  4. وجود پیش‌فرض خطرناک `complete = true` / `?? true` (قانون مستقیم `architecture-guards.mjs`)
- **محاسبهٔ نمرهٔ ۱۰۰ امتیازی:**
  - معماری و بهداشت کد Rust + TS (`30` امتیاز): کسر بابت `TODO/FIXME/HACK`، فایل‌های `> 500` خط و نقص `cargo check`
  - امنیت و سخت‌سازی باینری (`25` امتیاز): نبود اسرار هاردکد، `.env.example`، بررسی PIE/NX/RELRO یا `Cargo.toml` release profile (`lto`, `strip`/`panic`)
  - فرانت‌اند Next.js (`20` امتیاز): وجود `package.json`، `tsconfig.json`، `next.config.*`، نبود `@ts-ignore` / `any` بی‌رویه و تست/ساختار `app/`
  - سلامت دیتابیس SQLite و دانش (`15` امتیاز): `PRAGMA integrity_check == ok`، وجود `"KbEntry"` و حداقل ردیف‌های پرشده
  - تست و مستندات (`10` امتیاز): حضور `README.md`، فایل‌های تست در `crates/` و فرانت‌اند
- **آستانهٔ قبولی:** `score >= 85` **و** صفر خط قرمز (`hard_failures == 0`).
