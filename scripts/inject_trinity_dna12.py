#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_trinity_dna12.py — inject creator-brain DNA #12 (the Trinity Studio Pipeline) into MAIRON.

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
    "The twelfth creator-brain DNA: the sovereign three-body pipeline (Trinity Studio Pipeline) — "
    "Alpha -> the 3x9 command center -> Mairon 150/145"
)

DNA12_BODY = (
    "The twelfth creator-brain dossier (KIA-BRAIN-OS/12_TRINITY_AVATAR_ENGINE.md): "
    "the signature architecture of the sovereign studio — three bodies of Kia in one product forge: "
    "(1) Body 1 Alpha — the idea hunter and the master prompt + project-structure compiler with the locked calibration "
    "IQ=136 / EQ=122 / stop brake 0.04 / entropy temperature 0.88; deterministic extraction of the five discovery slots "
    "(the core objective, the audience and UX in 390 pixels, first-frame interactive features, keyless data and APIs from the "
    "120 public APIs, monetization with KIAN/Stars (XTR) and the referrer share) and the master-prompt lock (SHA-256) plus the "
    "project structure at >= 80 percent readiness. "
    "(2) Body 2 the logical command center — the judicial orchestrator with IQ=139 / EQ=129 / brake 0.62 / judicial strictness "
    "0.96 and the OpenBot subagent factory with the 3x9 pattern: three cache-safe prompt layers "
    "(the fixed core of sovereign laws + the session context with the master-prompt hash + the dynamic assignment with allele seeds, separated by "
    "__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__) x the nine-layer genome (f01 topology and harness with the 450s/1200s stop hooks, "
    "f02 hard constraints forbid(unsafe_code)/zero 390-overflow/the SSRF guard, f03 Caveman token economics, "
    "f04 deterministic random alleles from session_seed+subagent_index with SplitMix64: "
    "surgical_density, ux_craft, edge_case_paranoia, execution_velocity, f05 the three-attempt self-healing contract, "
    "f06 the XML block <openbot_subagent_dna>, f07 the zero-todo/secret/slop purifier gate, "
    "f08 experimental probes, f09 the delivery card). "
    "(3) Body 3 Mairon-150 — the overclocked judge with IQ=150 / EQ=145 / lookahead horizon 8 / third-order "
    "theory of mind / ambition 0.99 locked in crates/domain/src/mairon_rea_and_reviews.rs and "
    "crates/hermes/src/trinity_pipeline.rs (the single source of truth): the 4x25=100-column REA judgment over "
    "the artifact's measurable properties, the eight-step strategic advisory and the ToM-3 psychological hook with Stars pricing, "
    "the execute_command_center_with_mairon_loop self-healing loop until the REA gate >= 85 passes "
    "(max 3 repair commands to the guilty subagent), the two-key publish gateway "
    "(confirm_and_publish_product) with the Mairon REA Certified {score}/100 (Forge-Grade) seal and "
    "the five-star Bayesian engine (prior C=5.0 and m=4.20, the verified-purchase weight 1.0 against 0.25, "
    "CombinedRankScore = 0.60*(bayesian_stars*20)+0.40*mairon_score). "
    "Live HTTP wiring: POST /api/v1/builder/trinity/alpha, POST /api/v1/builder/trinity/orchestrate, "
    "POST /api/v1/market/products/{id}/mairon-confirm, GET/POST /api/v1/market/products/{id}/reviews. "
    "Persistence: the 0007_trinity_reviews.sql migration (the maironScore/maironBadge seal on Product and the "
    "ProductReview table with the one-vote UNIQUE lock). Kernel contract: zero unsafe, zero Clippy warnings "
    "(rustc 1.100.0), 560 green tests. Asymmetric execution without permission, full pipeline determinism — the same seed, "
    "the same fleet, the same artifact. [EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]"
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
