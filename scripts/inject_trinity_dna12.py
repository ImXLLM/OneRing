#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_trinity_dna12.py — تزریق DNA #12 مغز خالق (Wave 3 — Trinity Studio Pipeline).

Sections:
  1. Idempotent upsert of 1 curated flagship "KbEntry" record into
     seed/kian-production.db:
       - kb:dna-12-trinity-avatar-engine (KIA-BRAIN-OS/12_TRINITY_AVATAR_ENGINE.md)
     tagged `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]` — 100% execution weight on the
     latest Trinity release.
  2. Verification: KbEntry 326 -> 327, citizens 466, referral lines 247,
     escrows 108, products 20, postings 870, trial balance SUM(amount) = 0,
     PRAGMA integrity_check = ok, LIKE search < 5 ms.
  3. Sync of seed/kian-production.db -> /home/z/my-project/download/kian-production.db.

Idempotency: upsert compares title+body first; unchanged rows are skipped.
Posting (double-entry ledger) is never touched by this script.
"""

from __future__ import annotations

import shutil
import sqlite3
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DB_PATH = ROOT / "seed" / "kian-production.db"
DOWNLOAD_DB = Path("/home/z/my-project/download/kian-production.db")

CREATED_AT = "2026-11-25T00:00:00.000Z"

DNA12_TITLE = (
    "DNA دوازدهم مغز خالق: خط لولهٔ سه‌کالبدی حاکم (Trinity Studio Pipeline) — "
    "آلفا → اتاق فرمان ۳×۹ → مایرون ۱۵۰/۱۴۵"
)

DNA12_BODY = (
    "دوازدهمین پروندهٔ مغز خالق (KIA-BRAIN-OS/12_TRINITY_AVATAR_ENGINE.md، Wave 3 کرنل 0xKIA): "
    "معماریِ امضادارِ استودیوی حاکم — سه کالبد کیا در یک آهنگریِ محصول: "
    "(۱) کالبد ۱ آلفا — شکارچی ایده و کامپایلر مستر پرامپت و استراکچر پروژه با کالیبراسیون قفل‌شده "
    "IQ=136 / EQ=122 / ترمزِ توقف 0.04 / دمای آنتروپی 0.88؛ استخراج قطعی پنج اسلات کشف "
    "(هدفِ هسته، مخاطب و UX در ۳۹۰ پیکسل، قابلیت‌های تعاملی فریمِ اول، داده و APIهای بدون احراز از "
    "۱۲۰ API عمومی، درآمدزایی با KIAN/Stars (XTR) و سهم معرف) و قفلِ مستر پرامپت (SHA-256) و "
    "استراکچر پروژه در آمادگیِ ≥ 80٪. "
    "(۲) کالبد ۲ اتاق فرمان منطقی — ارکستراتور قضایی با IQ=139 / EQ=129 / ترمز 0.62 / سختیِ قضایی "
    "0.96 و کارخانهٔ ساب‌ایجنت‌های OpenBot با الگوی ۳×۹: سه لایهٔ پرامپت کش-امن "
    "(هستهٔ ثابت قوانین حاکم + زمینهٔ نشست با hash مستر پرامپت + محولهٔ پویا با بذر آلل‌ها، جداشده با "
    "__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__) × ژنوم نه‌لایه‌ای (f01 توپولوژی و هارنس با قلاب توقف "
    "450s/1200s، f02 قیدهای سخت forbid(unsafe_code)/صفر سرریز ۳۹۰/گارد SSRF، f03 اقتصاد توکن "
    "Caveman، f04 آلل‌های تصادفی قطعی از session_seed+subagent_index با SplitMix64: "
    "surgical_density، ux_craft، edge_case_paranoia، execution_velocity، f05 قرارداد خودترمیمی "
    "۳-تلاشی، f06 بلوک XML <openbot_subagent_dna>، f07 گیت پاک‌ساز صفر todo/راز/اسلوپ، "
    "f08 پروب‌های تجربی، f09 کارت تحویل). "
    "(۳) کالبد ۳ مایرون-۱۵۰ — داور اورکلاک‌شده با IQ=150 / EQ=145 / افق نگاه ۸ / نظریهٔ ذهن "
    "مرتبهٔ ۳ / جاه‌طلبی 0.99 قفل‌شده در crates/domain/src/mairon_rea_and_reviews.rs و "
    "crates/hermes/src/trinity_pipeline.rs (منبع واحد حقیقت): داوری ۴×۲۵=۱۰۰ ستون REA از "
    "ویژگی‌های قابل‌سنجش آرتیفکت، مشاورهٔ استراتژیک هشت‌گامی و قلاب روان‌شناختی ToM-3 با قیمت "
    "Stars، حلقهٔ خودترمیمی execute_command_center_with_mairon_loop تا عبور از گیت REA ≥ 85 "
    "(بیشینهٔ ۳ فرمان ترمیم به ساب‌ایجنت مقصر)، دروازهٔ دو-کلیدی انتشار "
    "(confirm_and_publish_product) با نشان ⚒️ Mairon REA Certified: {score}/100 (Forge-Grade) و "
    "موتور ستارهٔ ۵گانهٔ بیزی (پیشین C=5.0 و m=4.20، وزن خرید تاییدشده 1.0 در برابر 0.25، "
    "CombinedRankScore = 0.60·(bayesian_stars·20)+0.40·mairon_score). "
    "سیم‌کشی زندهٔ HTTP: POST /api/v1/builder/trinity/alpha، POST /api/v1/builder/trinity/orchestrate، "
    "POST /api/v1/market/products/{id}/mairon-confirm، GET/POST /api/v1/market/products/{id}/reviews. "
    "ماندگاری: مهاجرت 0007_trinity_reviews.sql (مُهر maironScore/maironBadge روی Product و جدول "
    "ProductReview با قفل یک‌رأیی UNIQUE). قرارداد کرنل: صفر unsafe، صفر هشدار Clippy "
    "(rustc 1.100.0)، ۵۶۰ آزمون سبز. اجرای نامتقارن بدون اجازه، قطعیتِ کامل خط لوله — همان بذر، "
    "همان ناوگان، همان آرتیفکت. [EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]"
)

KB_ITEMS = [("kb:dna-12-trinity-avatar-engine", DNA12_TITLE, DNA12_BODY)]


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


def verify(db: sqlite3.Connection) -> dict:
    cur = db.cursor()
    kb = cur.execute('SELECT COUNT(*) FROM "KbEntry"').fetchone()[0]
    active = cur.execute('SELECT COUNT(*) FROM "Product" WHERE "active"=1').fetchone()[0]
    citizens = cur.execute('SELECT COUNT(*) FROM "User"').fetchone()[0]
    referrals = cur.execute('SELECT COUNT(*) FROM "ReferralLineage"').fetchone()[0]
    escrows = cur.execute('SELECT COUNT(*) FROM "Escrow"').fetchone()[0]
    products = cur.execute('SELECT COUNT(*) FROM "Product"').fetchone()[0]
    postings = cur.execute('SELECT COUNT(*) FROM "Posting"').fetchone()[0]
    trial = cur.execute("SELECT COALESCE(SUM(amount),0) FROM Posting").fetchone()[0]
    integrity = cur.execute("PRAGMA integrity_check").fetchone()[0]
    timings = []
    for q in ("Trinity Studio Pipeline", "openbot_subagent_dna", "IQ=150 / EQ=145",
              "CombinedRankScore", "Mairon REA Certified"):
        t0 = time.perf_counter()
        rows = cur.execute(
            'SELECT COUNT(*) FROM "KbEntry" WHERE "title" LIKE ? OR "body" LIKE ?',
            (f"%{q}%", f"%{q}%")).fetchone()[0]
        timings.append((q, rows, (time.perf_counter() - t0) * 1000))
    return {
        "kb": kb, "active": active, "citizens": citizens, "referrals": referrals,
        "escrows": escrows, "products": products, "postings": postings,
        "trial_sum": trial, "integrity": integrity, "timings": timings,
    }


def main() -> None:
    assert DB_PATH.exists(), f"missing {DB_PATH}"
    db = sqlite3.connect(DB_PATH)
    before = verify(db)
    print(f"[1/3] before: KbEntry={before['kb']} | citizens={before['citizens']} | "
          f"referrals={before['referrals']} | escrows={before['escrows']} | "
          f"products={before['products']} | postings={before['postings']} | "
          f"trial_sum={before['trial_sum']}")
    db.execute("BEGIN")
    stats = upsert_kb(db)
    db.commit()
    print(f"[2/3] upsert: {stats['inserted']} inserted, {stats['updated']} updated, "
          f"{stats['skipped']} skipped")
    after = verify(db)
    db.close()
    print(f"[3/3] after: KbEntry={after['kb']} (326 -> 327 expected) | "
          f"citizens={after['citizens']} | referrals={after['referrals']} | "
          f"escrows={after['escrows']} | products={after['products']} | "
          f"postings={after['postings']} | trial_sum={after['trial_sum']} | "
          f"integrity={after['integrity']}")
    for q, rows, ms in after["timings"]:
        flag = "OK" if ms < 5 else "SLOW"
        print(f"  LIKE %{q}% -> {rows} rows in {ms:.3f} ms [{flag}]")
    assert after["kb"] == 327, f"KbEntry expected 327, got {after['kb']}"
    assert (after["citizens"], after["referrals"], after["escrows"],
            after["products"], after["postings"]) == (466, 247, 108, 20, 870), \
        "ledger invariants drifted"
    assert after["trial_sum"] == 0, "trial balance must stay exactly zero"
    assert after["integrity"] == "ok"
    if DOWNLOAD_DB.parent.exists():
        shutil.copyfile(DB_PATH, DOWNLOAD_DB)
        print(f"synced -> {DOWNLOAD_DB}")


if __name__ == "__main__":
    main()
