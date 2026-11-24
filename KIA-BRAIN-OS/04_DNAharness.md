---
file_id: "04_DNAharness"
schema_version: "1.0.0"
layer: environment
runtime: "Z.ai + Arena + Railway"
language: "fa-IR + en-US"
---

# 04 — DNAharness: فیزیک محیط اجرا

## 1. نقشهٔ محیط‌ها

| محیط | نقش | قواعد |
|---|---|---|
| **Command Center** (Local) | فرماندهی، پذیرش گنجینه‌ها، تصمیم معماری | مالک: Kia؛ کلیدها فقط همین‌جا |
| **Z.ai** (Sandbox) | سازندهٔ Rust/DB — کد، تست، تزریق دیتابیس | `CARGO_TARGET_DIR=/tmp/my-project/target`، `CARGO_BUILD_JOBS=2` |
| **Arena** | هاروستر جنایی مدل‌ها — کالبدشکافی رفتار مدل‌های فرانتیر | خروجی → `OMNI-MODEL-GENOME/models/<id>/` |
| **Railway** | رانتایم تولید kian-server + 9router | متغیرهای محیطی، هیچ رازی در گیت |

## 2. قوانین سخت Z.ai

- فایل‌های تحویلی فقط زیر `/home/z/my-project/download/` دیده می‌شوند.
- آپلودها در `/home/z/my-project/upload/` — فقط ورودی، هرگز in-place تغییر نمی‌کنند.
- اسکریپت‌های بلند در `/home/z/my-project/scripts/` ذخیره و از فایل اجرا می‌شوند.
- پروژهٔ کیان: `/home/z/my-project/kian-launch/kian/` — گیت اصلی.

## 3. گیت راز (Zero-Secret Git Gate)

پیش از هر commit:
1. `git diff --cached` اسکن می‌شود: `sk-`, `ghp_`, `eyJ`, `AKIA`, `xox`,
   توکن بات (`\d+:[A-Za-z0-9_-]{30,}`)، `Bearer <non-public>` → صفر مجاز؛
2. تنها توکن مجاز در مستندات: `Bearer public` (sentinel ناشناس OpenCode Zen);
3. متن جایگزین همیشه `DUMMY-TEST-FIXTURE` است.

## 4. چرخهٔ گنجینه (Vault Cycle)

`upload/*.zip` → بازگشایی در `/tmp/stepN_gems/` → گزینش (<100KB/فایل) →
مونت در `docs/vault/<name>/` → ضدعفونی → کرنل Rust/SQLite → commit.

## 5. رفتار در خرابی

- دو شکست متوالی ابزار → توقف و گزارش صادقانه، نه دور زدن سیاست.
- timeout طولانی cargo → بازهم با همان env؛ هرگز بدون
  `CARGO_TARGET_DIR` اجرا نمی‌شود (خزانگی جوی‌ایزوله).
