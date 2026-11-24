---
file_id: "06_LINK_Router"
schema_version: "1.0.0"
layer: bridge
target: "OMNI-MODEL-GENOME/00_ECOSYSTEM_INDEX.md"
language: "fa-IR + en-US"
---

# 06 — LINK_Router: پل قطعی مغز سازنده ↔ ژنوم مدل‌ها

## 1. قرارداد مسیریابی

`KIA-BRAIN-OS` (هویت و قوانین) هرگز مستقیم مدل انتخاب نمی‌کند؛ از طریق روت
دترمینیستی زیر به ژنوم مدل‌ها می‌رسد:

```text
KIA-BRAIN-OS/06_LINK_Router.md
  → OMNI-MODEL-GENOME/00_ECOSYSTEM_INDEX.md          (جدول ۲۰ تسک کلان)
  → OMNI-MODEL-GENOME/models/<model_id>/09_ROUTING_AND_HANDOFF_CARD.md (اسکورکارد)
  → crates/ai/src/omni_genome_router.rs              (پورت Rust: profile + compile_prompt_for_model)
```

## 2. قانون انتخاب (deterministic)

1. `EngineeringWorkload` تسک را تعیین کن (۲۰ تسک استاندارد).
2. بالاترین `workload_scores` بین پروفایل‌های موجود را بردار؛ تساوی →
   `cost_speed_tier` سبک‌تر برنده است؛ باز تساوی → ترتیب الفبایی model_id.
3. امتیاز نامعلوم (پروفایل نبود) = **هرگز از خودسازی مقدار نمی‌گیرد** —
   «unknown» برمی‌گردد و مسیر فال‌بک (رجیستری صفر-کارت) گرفته می‌شود.

## 3. پل‌های ثابت

| از | به | قاعده |
|---|---|---|
| `05_DNAskills.md` | `KbEntry` | هر مهارت = یک ردیف idempotent با منبع |
| `07_LINK_Protocol.md` | `HandoffPacket` | same schema، بدون فیلد ابداعی |
| `04_DNAharness.md` | Arena | خروجی کالبدشکافی فقط در `models/<id>/` |

## 4. ممنوعیت‌های پل

- تغییر دستی فایل‌های `00..09` مدل بدون فرآیند Arena ممنوع.
- نوشتن امتیاز برای مدل بدون کارت 09 ممنوع (provisional فقط با برچسب).
- روتِ دترمینیستی جایگزین حکم انسانی معمار ارشد نمی‌شود — فقط پیشنهاد
  رتبه‌بندی‌شده می‌دهد.
