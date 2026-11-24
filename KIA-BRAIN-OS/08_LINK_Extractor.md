---
file_id: "08_LINK_Extractor"
schema_version: "1.0.0"
layer: protocol
purpose: "forensic harvesting"
language: "fa-IR + en-US"
---

# 08 — LINK_Extractor: پروتکل استخراج گنجینه‌ها

## 1. ورودی → خروجی

| ورودی | خروجی | مقصد |
|---|---|---|
| `upload/*-gems.zip` | پوشه‌های شماره‌دار منحنی | `/tmp/stepN_gems/<vault>/` |
| ریپوی مرجع باز | الگوریتم خالص (نه boilerplate) | کرنل Rust + تست |
| مدل فرانتیر (Arena) | ژنوم ۹فایلی | `OMNI-MODEL-GENOME/models/<id>/` |
| هر کاوش | ردیف دانش | `"KbEntry"` (idempotent) |

## 2. قانون گزینش (harvest bar)

1. فقط فایل‌های **الگوریتم‌محور**: موتور رتبه‌بندی، ادیتور پچ، گارد امنیتی،
   آبشار فال‌بک — لاگ‌های زباله و UI نمونه استخراج نمی‌شود.
2. سقف مونت: **<100KB/فایل** — فایل بزرگ‌تر باید بریده یا خلاصه شود.
3. هر مونت باید مسیر پورت کرنل خود را در بدنه ذکر کند.

## 3. پروتکل کالبدشکافی مدل (Arena → 9 files)

- فایل‌های `01..09` با همان نام‌های canonical در `omni_genome_router.rs`.
- frontmatter YAML اجباری: `file_id`, `schema_version: "1.0.0"`,
  `model_self_reported_id`, `calibration_status`.
- عدد بدون شاهد ممنوع — حدس‌ها با برچسب `unverified_hypothesis` می‌آیند.
- پرچم‌های روتِ Rust که از ژنوم خوانده می‌شوند:
  `think_mode_active`, `requires_external_state_trace`, `suppress_web_canvas_hijack`,
  `max_simultaneous_rules_per_phase`.

## 4. ضدعفونی اجباری پایان استخراج

هر مسیر استخراج با اسکنر راز (`sanitize_step4_vaults.py` الگو) تمام می‌شود؛
موجودی غیر از `Bearer public` یعنی شکست — قبل از commit پاک می‌شود.
