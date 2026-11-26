#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_trinity_dna12.py — \u062a\u0632\u0631\u06cc\u0642 DNA #12 \u0645\u063a\u0632 \u062e\u0627\u0644\u0642 (Wave 3 — Trinity Studio Pipeline).

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
    "DNA \u062f\u0648\u0627\u0632\u062f\u0647\u0645 \u0645\u063a\u0632 \u062e\u0627\u0644\u0642: \u062e\u0637 \u0644\u0648\u0644\u0647\u0654 \u0633\u0647‌\u06a9\u0627\u0644\u0628\u062f\u06cc \u062d\u0627\u06a9\u0645 (Trinity Studio Pipeline) — "
    "\u0622\u0644\u0641\u0627 → \u0627\u062a\u0627\u0642 \u0641\u0631\u0645\u0627\u0646 \u06f3×\u06f9 → \u0645\u0627\u06cc\u0631\u0648\u0646 \u06f1\u06f5\u06f0/\u06f1\u06f4\u06f5"
)

DNA12_BODY = (
    "\u062f\u0648\u0627\u0632\u062f\u0647\u0645\u06cc\u0646 \u067e\u0631\u0648\u0646\u062f\u0647\u0654 \u0645\u063a\u0632 \u062e\u0627\u0644\u0642 (KIA-BRAIN-OS/12_TRINITY_AVATAR_ENGINE.md\u060c Wave 3 \u06a9\u0631\u0646\u0644 0xKIA): "
    "\u0645\u0639\u0645\u0627\u0631\u06cc\u0650 \u0627\u0645\u0636\u0627\u062f\u0627\u0631\u0650 \u0627\u0633\u062a\u0648\u062f\u06cc\u0648\u06cc \u062d\u0627\u06a9\u0645 — \u0633\u0647 \u06a9\u0627\u0644\u0628\u062f \u06a9\u06cc\u0627 \u062f\u0631 \u06cc\u06a9 \u0622\u0647\u0646\u06af\u0631\u06cc\u0650 \u0645\u062d\u0635\u0648\u0644: "
    "(\u06f1) \u06a9\u0627\u0644\u0628\u062f \u06f1 \u0622\u0644\u0641\u0627 — \u0634\u06a9\u0627\u0631\u0686\u06cc \u0627\u06cc\u062f\u0647 \u0648 \u06a9\u0627\u0645\u067e\u0627\u06cc\u0644\u0631 \u0645\u0633\u062a\u0631 \u067e\u0631\u0627\u0645\u067e\u062a \u0648 \u0627\u0633\u062a\u0631\u0627\u06a9\u0686\u0631 \u067e\u0631\u0648\u0698\u0647 \u0628\u0627 \u06a9\u0627\u0644\u06cc\u0628\u0631\u0627\u0633\u06cc\u0648\u0646 \u0642\u0641\u0644‌\u0634\u062f\u0647 "
    "IQ=136 / EQ=122 / \u062a\u0631\u0645\u0632\u0650 \u062a\u0648\u0642\u0641 0.04 / \u062f\u0645\u0627\u06cc \u0622\u0646\u062a\u0631\u0648\u067e\u06cc 0.88\u061b \u0627\u0633\u062a\u062e\u0631\u0627\u062c \u0642\u0637\u0639\u06cc \u067e\u0646\u062c \u0627\u0633\u0644\u0627\u062a \u06a9\u0634\u0641 "
    "(\u0647\u062f\u0641\u0650 \u0647\u0633\u062a\u0647\u060c \u0645\u062e\u0627\u0637\u0628 \u0648 UX \u062f\u0631 \u06f3\u06f9\u06f0 \u067e\u06cc\u06a9\u0633\u0644\u060c \u0642\u0627\u0628\u0644\u06cc\u062a‌\u0647\u0627\u06cc \u062a\u0639\u0627\u0645\u0644\u06cc \u0641\u0631\u06cc\u0645\u0650 \u0627\u0648\u0644\u060c \u062f\u0627\u062f\u0647 \u0648 API\u0647\u0627\u06cc \u0628\u062f\u0648\u0646 \u0627\u062d\u0631\u0627\u0632 \u0627\u0632 "
    "\u06f1\u06f2\u06f0 API \u0639\u0645\u0648\u0645\u06cc\u060c \u062f\u0631\u0622\u0645\u062f\u0632\u0627\u06cc\u06cc \u0628\u0627 KIAN/Stars (XTR) \u0648 \u0633\u0647\u0645 \u0645\u0639\u0631\u0641) \u0648 \u0642\u0641\u0644\u0650 \u0645\u0633\u062a\u0631 \u067e\u0631\u0627\u0645\u067e\u062a (SHA-256) \u0648 "
    "\u0627\u0633\u062a\u0631\u0627\u06a9\u0686\u0631 \u067e\u0631\u0648\u0698\u0647 \u062f\u0631 \u0622\u0645\u0627\u062f\u06af\u06cc\u0650 ≥ 80\u066a. "
    "(\u06f2) \u06a9\u0627\u0644\u0628\u062f \u06f2 \u0627\u062a\u0627\u0642 \u0641\u0631\u0645\u0627\u0646 \u0645\u0646\u0637\u0642\u06cc — \u0627\u0631\u06a9\u0633\u062a\u0631\u0627\u062a\u0648\u0631 \u0642\u0636\u0627\u06cc\u06cc \u0628\u0627 IQ=139 / EQ=129 / \u062a\u0631\u0645\u0632 0.62 / \u0633\u062e\u062a\u06cc\u0650 \u0642\u0636\u0627\u06cc\u06cc "
    "0.96 \u0648 \u06a9\u0627\u0631\u062e\u0627\u0646\u0647\u0654 \u0633\u0627\u0628‌\u0627\u06cc\u062c\u0646\u062a‌\u0647\u0627\u06cc OpenBot \u0628\u0627 \u0627\u0644\u06af\u0648\u06cc \u06f3×\u06f9: \u0633\u0647 \u0644\u0627\u06cc\u0647\u0654 \u067e\u0631\u0627\u0645\u067e\u062a \u06a9\u0634-\u0627\u0645\u0646 "
    "(\u0647\u0633\u062a\u0647\u0654 \u062b\u0627\u0628\u062a \u0642\u0648\u0627\u0646\u06cc\u0646 \u062d\u0627\u06a9\u0645 + \u0632\u0645\u06cc\u0646\u0647\u0654 \u0646\u0634\u0633\u062a \u0628\u0627 hash \u0645\u0633\u062a\u0631 \u067e\u0631\u0627\u0645\u067e\u062a + \u0645\u062d\u0648\u0644\u0647\u0654 \u067e\u0648\u06cc\u0627 \u0628\u0627 \u0628\u0630\u0631 \u0622\u0644\u0644‌\u0647\u0627\u060c \u062c\u062f\u0627\u0634\u062f\u0647 \u0628\u0627 "
    "__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__) × \u0698\u0646\u0648\u0645 \u0646\u0647‌\u0644\u0627\u06cc\u0647‌\u0627\u06cc (f01 \u062a\u0648\u067e\u0648\u0644\u0648\u0698\u06cc \u0648 \u0647\u0627\u0631\u0646\u0633 \u0628\u0627 \u0642\u0644\u0627\u0628 \u062a\u0648\u0642\u0641 "
    "450s/1200s\u060c f02 \u0642\u06cc\u062f\u0647\u0627\u06cc \u0633\u062e\u062a forbid(unsafe_code)/\u0635\u0641\u0631 \u0633\u0631\u0631\u06cc\u0632 \u06f3\u06f9\u06f0/\u06af\u0627\u0631\u062f SSRF\u060c f03 \u0627\u0642\u062a\u0635\u0627\u062f \u062a\u0648\u06a9\u0646 "
    "Caveman\u060c f04 \u0622\u0644\u0644‌\u0647\u0627\u06cc \u062a\u0635\u0627\u062f\u0641\u06cc \u0642\u0637\u0639\u06cc \u0627\u0632 session_seed+subagent_index \u0628\u0627 SplitMix64: "
    "surgical_density\u060c ux_craft\u060c edge_case_paranoia\u060c execution_velocity\u060c f05 \u0642\u0631\u0627\u0631\u062f\u0627\u062f \u062e\u0648\u062f\u062a\u0631\u0645\u06cc\u0645\u06cc "
    "\u06f3-\u062a\u0644\u0627\u0634\u06cc\u060c f06 \u0628\u0644\u0648\u06a9 XML <openbot_subagent_dna>\u060c f07 \u06af\u06cc\u062a \u067e\u0627\u06a9‌\u0633\u0627\u0632 \u0635\u0641\u0631 todo/\u0631\u0627\u0632/\u0627\u0633\u0644\u0648\u067e\u060c "
    "f08 \u067e\u0631\u0648\u0628‌\u0647\u0627\u06cc \u062a\u062c\u0631\u0628\u06cc\u060c f09 \u06a9\u0627\u0631\u062a \u062a\u062d\u0648\u06cc\u0644). "
    "(\u06f3) \u06a9\u0627\u0644\u0628\u062f \u06f3 \u0645\u0627\u06cc\u0631\u0648\u0646-\u06f1\u06f5\u06f0 — \u062f\u0627\u0648\u0631 \u0627\u0648\u0631\u06a9\u0644\u0627\u06a9‌\u0634\u062f\u0647 \u0628\u0627 IQ=150 / EQ=145 / \u0627\u0641\u0642 \u0646\u06af\u0627\u0647 \u06f8 / \u0646\u0638\u0631\u06cc\u0647\u0654 \u0630\u0647\u0646 "
    "\u0645\u0631\u062a\u0628\u0647\u0654 \u06f3 / \u062c\u0627\u0647‌\u0637\u0644\u0628\u06cc 0.99 \u0642\u0641\u0644‌\u0634\u062f\u0647 \u062f\u0631 crates/domain/src/mairon_rea_and_reviews.rs \u0648 "
    "crates/hermes/src/trinity_pipeline.rs (\u0645\u0646\u0628\u0639 \u0648\u0627\u062d\u062f \u062d\u0642\u06cc\u0642\u062a): \u062f\u0627\u0648\u0631\u06cc \u06f4×\u06f2\u06f5=\u06f1\u06f0\u06f0 \u0633\u062a\u0648\u0646 REA \u0627\u0632 "
    "\u0648\u06cc\u0698\u06af\u06cc‌\u0647\u0627\u06cc \u0642\u0627\u0628\u0644‌\u0633\u0646\u062c\u0634 \u0622\u0631\u062a\u06cc\u0641\u06a9\u062a\u060c \u0645\u0634\u0627\u0648\u0631\u0647\u0654 \u0627\u0633\u062a\u0631\u0627\u062a\u0698\u06cc\u06a9 \u0647\u0634\u062a‌\u06af\u0627\u0645\u06cc \u0648 \u0642\u0644\u0627\u0628 \u0631\u0648\u0627\u0646‌\u0634\u0646\u0627\u062e\u062a\u06cc ToM-3 \u0628\u0627 \u0642\u06cc\u0645\u062a "
    "Stars\u060c \u062d\u0644\u0642\u0647\u0654 \u062e\u0648\u062f\u062a\u0631\u0645\u06cc\u0645\u06cc execute_command_center_with_mairon_loop \u062a\u0627 \u0639\u0628\u0648\u0631 \u0627\u0632 \u06af\u06cc\u062a REA ≥ 85 "
    "(\u0628\u06cc\u0634\u06cc\u0646\u0647\u0654 \u06f3 \u0641\u0631\u0645\u0627\u0646 \u062a\u0631\u0645\u06cc\u0645 \u0628\u0647 \u0633\u0627\u0628‌\u0627\u06cc\u062c\u0646\u062a \u0645\u0642\u0635\u0631)\u060c \u062f\u0631\u0648\u0627\u0632\u0647\u0654 \u062f\u0648-\u06a9\u0644\u06cc\u062f\u06cc \u0627\u0646\u062a\u0634\u0627\u0631 "
    "(confirm_and_publish_product) \u0628\u0627 \u0646\u0634\u0627\u0646 ⚒️ Mairon REA Certified: {score}/100 (Forge-Grade) \u0648 "
    "\u0645\u0648\u062a\u0648\u0631 \u0633\u062a\u0627\u0631\u0647\u0654 \u06f5\u06af\u0627\u0646\u0647\u0654 \u0628\u06cc\u0632\u06cc (\u067e\u06cc\u0634\u06cc\u0646 C=5.0 \u0648 m=4.20\u060c \u0648\u0632\u0646 \u062e\u0631\u06cc\u062f \u062a\u0627\u06cc\u06cc\u062f\u0634\u062f\u0647 1.0 \u062f\u0631 \u0628\u0631\u0627\u0628\u0631 0.25\u060c "
    "CombinedRankScore = 0.60·(bayesian_stars·20)+0.40·mairon_score). "
    "\u0633\u06cc\u0645‌\u06a9\u0634\u06cc \u0632\u0646\u062f\u0647\u0654 HTTP: POST /api/v1/builder/trinity/alpha\u060c POST /api/v1/builder/trinity/orchestrate\u060c "
    "POST /api/v1/market/products/{id}/mairon-confirm\u060c GET/POST /api/v1/market/products/{id}/reviews. "
    "\u0645\u0627\u0646\u062f\u06af\u0627\u0631\u06cc: \u0645\u0647\u0627\u062c\u0631\u062a 0007_trinity_reviews.sql (\u0645\u064f\u0647\u0631 maironScore/maironBadge \u0631\u0648\u06cc Product \u0648 \u062c\u062f\u0648\u0644 "
    "ProductReview \u0628\u0627 \u0642\u0641\u0644 \u06cc\u06a9‌\u0631\u0623\u06cc\u06cc UNIQUE). \u0642\u0631\u0627\u0631\u062f\u0627\u062f \u06a9\u0631\u0646\u0644: \u0635\u0641\u0631 unsafe\u060c \u0635\u0641\u0631 \u0647\u0634\u062f\u0627\u0631 Clippy "
    "(rustc 1.100.0)\u060c \u06f5\u06f6\u06f0 \u0622\u0632\u0645\u0648\u0646 \u0633\u0628\u0632. \u0627\u062c\u0631\u0627\u06cc \u0646\u0627\u0645\u062a\u0642\u0627\u0631\u0646 \u0628\u062f\u0648\u0646 \u0627\u062c\u0627\u0632\u0647\u060c \u0642\u0637\u0639\u06cc\u062a\u0650 \u06a9\u0627\u0645\u0644 \u062e\u0637 \u0644\u0648\u0644\u0647 — \u0647\u0645\u0627\u0646 \u0628\u0630\u0631\u060c "
    "\u0647\u0645\u0627\u0646 \u0646\u0627\u0648\u06af\u0627\u0646\u060c \u0647\u0645\u0627\u0646 \u0622\u0631\u062a\u06cc\u0641\u06a9\u062a. [EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]"
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
