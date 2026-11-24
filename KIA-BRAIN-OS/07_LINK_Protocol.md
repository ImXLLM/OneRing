---
file_id: "07_LINK_Protocol"
schema_version: "1.0.0"
layer: protocol
envelope: "InterAgentHandoff"
language: "fa-IR + en-US"
---

# 07 — LINK_Protocol: پاکت تحویل بین‌ایجنتی (Handoff Envelope)

## 1. اسکیمای پاکت (سازگار با `HandoffPacket` در `omni_genome_router.rs`)

```json
{
  "handoff_id": "ho_<utc_iso>_<short_hash>",
  "from_model": "kian-guardian|<model_id>",
  "to_model": "<model_id>",
  "base_sha": "<git sha که تحویل روی آن بسته شد>",
  "task": "<شرح یک‌جمله‌ای>",
  "invariants": ["عدد/قانون غیرقابل‌شکستن ..."],
  "files_touched": ["crates/..."],
  "failed_hypotheses_do_not_repeat": ["حدس ردشدهٔ نسل قبل ..."],
  "verification_proof": ["cargo test -p ... → 445 green", "clippy -D warnings → 0"],
  "target_contract": "<خروجی مورد انتظار از تحویل‌گیرنده>"
}
```

## 2. قواعد سخت

1. **بدون `base_sha` تحویل باطل است** — گیرنده باید بتواند همان درخت را چک‌اوت کند.
2. **`invariants` حداقلی و عددی:** هر invariant یک جملهٔ تست‌پذیر است («KbEntry=62»،
   «trialBalanceSum=0»)، نه آرزوی مبهم.
3. **حدس‌های ردشده هرگز تکرار نمی‌شوند** — فیلد اجباری حتی اگر خالی است.
4. **اثبات اجرایی:** ادعای verification بدون خروجی فرمان = پاکت کاذب → رد.
5. پاکت فقط بین دو طرف زنجیرهٔ DNA حرکت می‌کند (`KIA-BRAIN-OS/07_LINK_Protocol.md`
   = حقیقت اسکیما؛ فایل‌های 09 ژنوم همان اسکیما را به زبان مدل‌ها می‌گویند).

## 3. چرخهٔ پذیرش

دریافت → چک `base_sha` → چک invariants روی درخت واقعی → اجرای verification
از نو → پذیرش یا بازگشت با دلیل. بازگشت بدون دلیل = تخلف پروتکل.
