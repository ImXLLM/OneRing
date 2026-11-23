#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_copilotkit_gems.py — تزریق ۶ مهارت منتخب گنجینهٔ CopilotKit/OpenBot به KIAN v2 (Wave 2 - Step 1).

Sections:
  1. Idempotent upsert of 6 curated "KbEntry" records into seed/kian-production.db
     (Zero-Flicker Streaming, Generative UI Design System, AG-UI 1.0 protocol,
     6-Category HITL Governance, Routine Fatigue Guard, A2UI Self-Healing Loop).
  2. Verification: KbEntry 33 -> 39, active products stay 20,
     trialBalanceSum stays 0 (58,700,200,000 micro-KIAN untouched), LIKE search < 5 ms.
  3. Sync of seed/kian-production.db -> /home/z/my-project/download/kian-production.db.

Idempotency: every upsert compares title+body first; unchanged rows are skipped
("0 updated, 6 skipped" on the second run), changed rows are replaced.
Trial balance (Posting) is never touched by this script.
"""

from __future__ import annotations

import shutil
import sqlite3
import sys
import time
from pathlib import Path

REPO = Path("/home/z/my-project/kian-launch/kian")
SEED_DB = REPO / "seed" / "kian-production.db"
DOWNLOAD_DB = Path("/home/z/my-project/download/kian-production.db")
CREATED_AT = "2026-11-24T00:00:00.000Z"

# ── 1. The 6 curated KB entries ──────────────────────────────────────────────

KB_ITEMS: list[tuple[str, str, str]] = [
    (
        "kb:skill-open-intelligent-ui-streaming",
        "مهارت استودیو: استریم HTML بدون پرش با گارد سندباکس و CSP (OpenIntelligentUI)",
        "استریم زندهٔ خروجی استودیو هرگز نباید iframe پیش‌نمایش را بپراند. سه لایهٔ دفاعی پورت‌شده از OpenIntelligentUI: "
        "(۱) processPartialHtml — تابع خالص بدون DOM: تگ بازِ ته سند (</...>) حذف، بلوک <script> بازِ ته سند با کل محتوایش حذف "
        "تا JS ناقص هرگز خطای syntax نیندازد، موجودیت ناقصِ ته سند (‎&...‎) حذف، و استایل گارد واکنش‌گرا (box-sizing: border-box، "
        "max-width: 100vw، overflow-x: hidden) فقط اگر در سند نبود تزریق می‌شود؛ (۲) settledArgs/is_settled_stream_arg — تا وقتی "
        "فراخوانی استریم کامل نشده، رنگ هگز با طول ناقص (۴/۷/۹)، URL ناتمام (به «/» یا «.» ختم یا کوتاه‌تر از ۱۲) و عددِ با «.» یا «-» "
        "پایانی به UI داده نمی‌شود تا کامپوننت نمی‌پرد؛ (۳) سندباکس iframe فقط با sandbox=\"allow-scripts\" — بدون allow-same-origin "
        "تا کد محصول هرگز به نشست و کوکی دسترسی نرسد، و CSP تنگ در سرصفحهٔ سند. پورت در کیان: apps/miniapp/lib/studio-html-guard.ts "
        "(متصل به پیش‌نمایش زندهٔ استودیو در app/builder) و crates/ai/src/history_repair.rs. قانون طلایی: نسخهٔ دانلودی/منتشرشده همیشه "
        "سند کاملِ دست‌نخورده است؛ گارد فقط روی پیش‌نمایش زنده اعمال می‌شود.",
    ),
    (
        "kb:skill-generative-ui-zero-chroma",
        "مهارت استودیو: دیزاین‌سیستم جنریتیو UI — پالت بی‌کروم ‎#0a0a0a/‎#fafafa + ‎#00bba7/‎#ff6467 و ریاضی امن valueAsNumber",
        "دیزاین‌سیستم جنریتیو UI برای خروجی‌های تولیدی مدل، دو تم نوتورال قطعی دارد: تم تیره با پس‌زمینهٔ ‎#0a0a0a و متن ‎#fafafa، "
        "تم روشن با برعکس؛ هر دو با کنتراست AAA برای فارسی و RTL. تنها دو رنگ تأکیدی مجاز: سبز موفقیت ‎#00bba7 و قرمز مخرب ‎#ff6467 — "
        "هیچ رنگ جیغ یا گرادیان چشمک‌زن در خروجی استودیو جایی ندارد؛ رنگ‌های برند فقط از متغیر تم می‌آیند نه از مدل. قواعد چیدمان: "
        "فاصله‌ها از مقیاس ۴px، شعاع گرد ۸-۱۲px، خط پایهٔ ۱۶px، و جهت RTL با منطق فیزیکی (padding-inline به‌جای left). "
        "قانون امنیتی ریاضی: هر محاسبهٔ پولی/عددی روی ورودی فرم با valueAsNumber انجام می‌شود — هرگز parseFloat/parseInt روی "
        "innerHTML یا مقدار رشته‌ای input؛ NaN و رشتهٔ خالی صراحتاً به ۰ یا خطای فرم نگاشت می‌شوند و هیچ رشته‌ای مستقیم به eval یا "
        "innerHTML داده نمی‌شود. چارت‌ها همیشه با حاشیهٔ امن محور و بدون اعداد شناورِ گردنشکن (حداکثر ۲ رقم اعشار) رسم می‌شوند. "
        "این قواعد پرامپتِ دیزاین استودیو و کامپوننت‌های کیان‌شده را هم‌زمان کنترل می‌کنند تا خروجی هر مدل، شبیه یک محصول واقعی باشد.",
    ),
    (
        "kb:skill-ag-ui-protocol-v1",
        "مهارت استودیو: پروتکل استاندارد AG-UI 1.0 — استریم ایجنت‌به‌UI با خانوادهٔ ۳۰ رخداد استاندارد",
        "AG-UI 1.0 قرارداد مشترک استریم ایجنت به رابط کاربری است؛ به‌جای SSEهای ادراریِ ad-hoc، هر رخداد یک پاکت با «type» از "
        "خانوادهٔ استاندارد است: TEXT_MESSAGE_START/CONTENT/END برای متن مدل، TOOL_CALL_START/ARGS/END/RESULT برای فراخوانی ابزار، "
        "STATE_SNAPSHOT و STATE_DELTA (پچ JSON) برای state اشتراکی، THINKING_* برای تفکر، STEP_*/MESSAGES_SNAPSHOT/Raw/Custom و "
        "RUN_STARTED/FINISHED/ERROR برای چرخهٔ ران. قواعد سری‌سازی: نام تایپ SCREAMING_SNAKE_CASE، فیلدها camelCase (messageId، "
        "toolCallId، parentMessageId) و فیلد بی‌مقدار هرگز سری نمی‌شود. کیان سه رخداد پسوندی ACTIVITY_SNAPSHOT/ACTIVITY_DELTA و "
        "SUBAGENT_SPAWN را هم روی همین قرارداد اضافه کرده تا پیشرفت استودیو و تولد ساب‌ایجنت استاندارد بماند. پورت در "
        "crates/hermes/src/ag_ui.rs با سازنده‌های آمادهٔ استریم (text_message_content، tool_call_args، state_snapshot و …) و تست "
        "گردش کامل ۱۴ رخداد. فایده: وب، مینی‌اپ و لاگ ممیزی همه یک زبان مشترک دارند و رندرگر بدون ترجمه‌گر، خروجی هر ایجنت را می‌خواند.",
    ),
    (
        "kb:skill-openbot-hitl-governance",
        "مهارت حکمرانی: گیت تأیید انسانی ۶طبقه‌ای با صدای دوگانه و الگوی جعبهٔ راز (OpenBot HITL)",
        "شش طبقهٔ اقدام همیشه کارت تأیید می‌گیرند: ارسال پیام به دیگران، خرید یا جابه‌جایی پول، حذف/تغییر داده، انتشار محتوای عمومی، "
        "مدیریت اعتبارنامه یا دسترسی، و ثبت فرم/درخواست. هر تصمیم «صدای دوگانه» دارد: جمله‌ای برای مدل (outcome_for_model) که می‌گوید "
        "چه کسی تصمیم گرفت و چه کند — روی ردِ انسان صراحتاً «Do not retry, rephrase, or work around it» — و جمله‌ای برای انسان "
        "(outcome_for_person_fa) که رک و فارسی است (درخواست توسط شما رد شد.). تغییر یکی هرگز دیگری را پشت سر نمی‌گذارد چون هر دو کنار "
        "هم تعریف و تست‌اند. الگوی جعبهٔ راز: مدیریت رمز/توکن/مجوز هرگز به بات سپرده نمی‌شود؛ کارت می‌گوید کارِ خودِ انسان است "
        "(خودم انجامش می‌دهم) و هیچ پاسخی از طرف او نمی‌تواند بنشیند. شبیه‌ساز what-if همان ترتیب لایه‌ها را برای «چه می‌شد اگر…» "
        "اجرا می‌کند تا شبیه‌ساز و گیت واقعی هرگز اختلاف نظر نداشته باشند؛ در کیان، دروازهٔ پول از منشور می‌آید: زیر ۵۰٬۰۰۰ دینار "
        "اسکرو خودکار، روی آستانه تأیید انسانی. پورت در crates/hermes/src/approvals.rs (ApprovalCategory، ApprovalOutcome، "
        "simulate_approval_policy).",
    ),
    (
        "kb:skill-openbot-routine-fatigue",
        "مهارت پایداری: ضداسپم روتین‌ها — کف ۱۵ دقیقه، سقف ۲۰، مدارشکن ۱۰ شکست و ساب‌ایجنت UUIDv8 قطعی",
        "روتین‌ها و اهداف ایستاده‌اند و اگر گارد نداشته باشند، بعد از هر ری‌استارت شلیک انبوه می‌کنند. چهار قاعدهٔ گارد خستگی: "
        "(۱) کف ۱۵ دقیقه (۹۰۰ ثانیه) بین دو شلیک یک روتین — فراخوانی زودتر خفه می‌شود؛ (۲) سقف ۲۰ روتین فعال هم‌زمان — بیست‌ویکمی "
        "منتظر صندلی می‌ماند؛ (۳) مدارشکن: پس از ۱۰ شکست پیاپی روتین خودکار خاموش می‌شود و یک موفقیت مدار را می‌بندد؛ (۴) پرش "
        "پنجره‌های از دست رفتهٔ کرون بعد از ری‌استارت — روتین منتظر پنجرهٔ بعدی می‌ماند، هرگز همهٔ پنجره‌های عقب‌افتاده را یک‌جا "
        "نمی‌شلیکد. ساب‌ایجنت‌ها thread مشتق دارند نه ضرب‌شده: SHA-256 روی «subagent\\n{parent}\\n{tool_call}» قالب‌بندی‌شده به "
        "UUIDv8 (نسخه ۸ = بیت‌ها تصادفی نیستند؛ ۶ بایت اول از والد به ارث می‌رسد) — هر دو سمت همین derivation را می‌دانند، پس کلاینت "
        "رونوشت را قبل از رسیدن نتیجه پیدا می‌کند و بعد از ری‌استارت تولد دوباره ممکن نیست. گارد کاملاً قطعی است: زمان از بیرون "
        "تزریق می‌شود و تست بدون خواب/شانس است. پورت در crates/hermes/src/approvals.rs (RoutineFatigueGuard، "
        "deterministic_subagent_thread_id).",
    ),
    (
        "kb:skill-a2ui-self-healing-loop",
        "مهارت استودیو: حلقهٔ خودترمیم خروجی جنریتیو UI و اثبات مبدأ چارت (basis: read | given | own-knowledge)",
        "خروجی استودیو قبل از رندر، با همان اعتبارسنجی که گیت رندر است چک می‌شود — تا تصمیم retry ابزار و تصمیم suppress رندر هرگز "
        "اختلاف نظر نداشته باشند. پاکت سالم یعنی: پاکت کامل ‎<!DOCTYPE html>…</html>‎، تگ‌های script و style بالانس، و بدون نشت حصار "
        "markdown (‎```‎). اگر عیب بود و تلاش‌ها تمام نشده بودند، بازخورد ساخت‌یافته برای مدل ساخته می‌شود که دقیقاً نام‌برداری می‌کند "
        "کدام تگ کم است ([missing_doctype]، [unclosed_script]، [markdown_fence_leak]) — مدل نوبت بعد فقط همان را می‌سازد؛ بعد از "
        "سقف تلاش (پیش‌فرض ۳) پاکت a2ui_recovery_exhausted برمی‌گردد و فال‌بک امن رندر می‌شود. وضعیت حلقه روی کانال فعالیت "
        "(activity_type: a2ui_recovery) استریم می‌شود تا کاربر «در حال بازسازی…» را ببیند. قانون اثبات مبدأ چارت: هر عدد نمایش‌داده‌شده "
        "منشأ اعلام‌شده دارد — basis: read (خوانده از منبع واقعی)، given (دادهٔ کاربر) یا own-knowledge (دانش مدل؛ باید صریح برچسب "
        "بخورد و در کارها/گزارش‌های مالی ممنوع است)؛ چارت بدون برچسب مبدأ رندر نمی‌شود. پورت در crates/ai/src/history_repair.rs "
        "(validate_and_build_self_healing_feedback، studio_defects) با تست عیب‌های نام‌دار و پاکت EXHAUSTED.",
    ),
]

# ── 2. Idempotent upsert ─────────────────────────────────────────────────────


def upsert_kb(db: sqlite3.Connection) -> dict:
    """Compare-then-write: unchanged rows are skipped, changed rows replaced."""
    stats = {"inserted": 0, "updated": 0, "skipped": 0}
    cur = db.cursor()
    for kb_id, title, body in KB_ITEMS:
        row = cur.execute('SELECT "title","body" FROM "KbEntry" WHERE "id"=?', (kb_id,)).fetchone()
        if row is not None and row[0] == title and row[1] == body:
            stats["skipped"] += 1
            continue
        cur.execute(
            'INSERT INTO "KbEntry" ("id","title","body","createdAt","updatedAt") VALUES (?,?,?,?,?)'
            ' ON CONFLICT("id") DO UPDATE SET "title"=excluded."title", "body"=excluded."body",'
            ' "updatedAt"=excluded."updatedAt"',
            (kb_id, title, body, CREATED_AT, CREATED_AT))
        if row is None:
            stats["inserted"] += 1
        else:
            stats["updated"] += 1
    return stats


# ── 3. Verification ──────────────────────────────────────────────────────────


def verify(db: sqlite3.Connection) -> dict:
    cur = db.cursor()
    kb = cur.execute('SELECT COUNT(*) FROM "KbEntry"').fetchone()[0]
    active = cur.execute('SELECT COUNT(*) FROM "Product" WHERE "active"=1').fetchone()[0]
    trial = cur.execute("SELECT COALESCE(SUM(amount),0) FROM Posting").fetchone()[0]
    timings = []
    for q in ("AG-UI", "بدون پرش", "جعبهٔ راز", "UUIDv8", "خودترمیم"):
        t0 = time.perf_counter()
        rows = cur.execute(
            'SELECT COUNT(*) FROM "KbEntry" WHERE "title" LIKE ? OR "body" LIKE ?',
            (f"%{q}%", f"%{q}%")).fetchone()[0]
        timings.append((q, rows, (time.perf_counter() - t0) * 1000))
    return {"kb": kb, "active": active, "trial_sum": trial, "search": timings}


def main() -> int:
    assert REPO.is_dir() and SEED_DB.is_file(), "repo/seed db missing"

    before = verify(sqlite3.connect(SEED_DB))
    print(f"[1/3] before: KbEntry={before['kb']} | active products={before['active']} | "
          f"trial balance sum={before['trial_sum']}")

    db = sqlite3.connect(SEED_DB)
    db.row_factory = sqlite3.Row
    try:
        ks = upsert_kb(db)
        db.commit()
    finally:
        db.close()
    print(f"[2/3] kb upsert: +{ks['inserted']} new, {ks['updated']} updated, {ks['skipped']} skipped")

    after = verify(sqlite3.connect(SEED_DB))
    print(f"[3/3] after: KbEntry={after['kb']} (33 -> 39 expected) | active products={after['active']} | "
          f"trial balance sum={after['trial_sum']}")
    for q, rows, ms in after["search"]:
        flag = "OK" if ms < 5.0 else "SLOW"
        print(f"      search '{q}': {rows} hits in {ms:.2f} ms [{flag}]")

    # sync to download deliverable
    DOWNLOAD_DB.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(SEED_DB, DOWNLOAD_DB)
    print(f"      synced seed db -> {DOWNLOAD_DB}")

    ok = (after["kb"] == 39 and after["active"] == 20 and after["trial_sum"] == 0
          and ks["updated"] == 0 and (ks["inserted"] == 6 or ks["skipped"] == 6)
          and all(ms < 5.0 for _, _, ms in after["search"]))
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
