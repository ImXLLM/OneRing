#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_nousresearch_gems.py — inject the 10 flagship NousResearch/hermes-agent vault gems into MAIRON (phase 2, wave of 2026-11).

Sections:
  1. Idempotent upsert of 10 curated flagship "KbEntry" records into
     seed/kian-production.db (Popular Web Designs, p5.js Canvas, Architecture
     Diagrams, Baoyu Infographics, Systematic Debugging, TDD, Codebase
     Inspection, Grounded Citations, Product Price Monitor, Hermes Function
     Calling Standard).
  2. Verification: KbEntry 39 -> 49, active products stay 20,
     Posting sum stays 0 (58,700,200,000 micro-KIAN untouched), LIKE search < 5 ms.
  3. Sync of seed/kian-production.db -> /home/z/my-project/download/kian-production.db.

Idempotency: every upsert compares title+body first; unchanged rows are skipped
("0 updated, 10 skipped" on the second run), changed rows are replaced.
Posting (double-entry ledger) is never touched by this script.
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

# ── 1. The 10 curated flagship KB entries ────────────────────────────────────

KB_ITEMS: list[tuple[str, str, str]] = [
    (
        "kb:skill-nous-popular-web-designs",
        "Web skill: 54 real design systems (Stripe, Linear, Vercel) as ready HTML/CSS blocks",
        "A library of 54 real product design systems (Stripe, Linear, Vercel, GitHub, ...) shipped as copy-ready HTML/CSS reference files — instead of the model guessing colors and spacing from scratch, a natural trigger (\n        \"build like Stripe, design like Linear\") loads that brand's visual operating system. Each design system carries a palette, a typographic scale, radii, shadows and a layout pattern, and acts as the "
        "reference palace that keeps studio output on real industry standards. Usage rules: (1) first find the closest brand match in the list, never a self-invented blend; (2) the design system is "
        "a base, not a cage — user brand colors replace the reference palette while spacing and type scales are preserved; (3) never emit the full CSS — announce the chosen system path and the theme variables. "
        "In MAIRON this skill combines with the zero-chroma #0a0a0a/#fafafa design system (kb:skill-generative-ui-zero-chroma): the neutral base stays constant and this skill builds the brand-feel layer. "
        "Source: skills/creative/popular-web-designs in the NousResearch vault.",
    ),
    (
        "kb:skill-nous-p5js-creative-canvas",
        "Creative skill: interactive p5.js sketches — generative art, shaders, 3D and a live dependency-free canvas",
        "A creative-coding p5.js skill for the studio's live canvas outputs: generative art, animation, mouse/keyboard interaction, WebGL, shaders and light 3D scenes — all in one build-free single-file HTML "
        "that runs directly in the MAIRON preview sandbox. Golden rules: (1) setup and draw stay stateless with minimal state — every frame builds from short inputs so an unfinished stream never shows a black canvas; (2) a fixed random "
        "seed for reproducibility — the same seed always renders the same sketch; (3) resolution adapts to devicePixelRatio and the draw loop runs under a frame-rate cap that does not burn a phone CPU; (4) no external fetches — all assets inline so the CSP "
        "sandbox never breaks; (5) interaction operators follow physical-direction logic. In MAIRON the studio's generative outputs (live dashboards, visualizers, small games) follow these rules so the preview boots without jumps or external dependencies. "
        "Source: skills/creative/p5js in the NousResearch vault.",
    ),
    (
        "kb:skill-nous-architecture-diagram",
        "Architecture skill: dark-theme SVG system/infrastructure/cloud diagrams as self-contained HTML",
        "Presentation-grade architecture diagrams: dark-theme SVG for system architecture, cloud infrastructure, data flow and network topology — the output is one self-contained HTML with inline SVG that never depends on "
        "an external library and renders directly in MAIRON flags/docs/console panels. Design rules: (1) an 8px grid and a deterministic layered layout (user -> gateway -> service -> datastore) flowing left-to-right "
        "or top-to-bottom, never diagonal; (2) every box carries a name plus a one-line role — the diagram is not a documentation vehicle; (3) lines use directional arrows and protocol labels (HTTP, gRPC, WS) along the path; (4) a "
        "limited dark palette with four gray layers + two accent colors (green for the healthy path, red for the error path) matched to the MAIRON zero-chroma design system; (5) SVG text uses a system font-family at >= 12px for readability. "
        "In MAIRON the Article-of-architecture diagrams (M2 ledger, M4 brain, M6 bot) and migration reports reproduce with this standard. Source: skills/creative/architecture-diagram in the NousResearch vault.",
    ),
    (
        "kb:skill-nous-baoyu-infographic",
        "Infographic skill: high-density visual synthesis with 21 layouts x 21 styles for dense data summaries",
        "A high-density infographic skill (ported from baoyu-skills by Jim Liu): a report/dense data set becomes a one-page infographic with 21 standard layouts (timeline, comparison, hierarchy, flow, matrix, "
        "KPI-strip, ...) x 21 visual styles (flat, glassmorphism, blueprint, editorial, ...) — the layout is chosen by the shape of the data, not taste: temporal data -> timeline; a two-entity comparison -> split-comparison; "
        "a hierarchy -> tree; counters -> KPI-strip. Hard rules: (1) every number has a source — an origin label (basis: read | given | own-knowledge) under every data block; without it the block never renders; (2) controlled "
        "density — max 9 blocks per page, each block <= 2 lines of text, the rest moves to sub-pages; (3) hierarchical typography with three levels on a fixed baseline; (4) a single-file HTML output with inline CSS and no heavy JS — animation is CSS-only; (5) physical-direction layout. "
        "In MAIRON, studio output for at-a-glance reports recognizes this skill as the default template. Source: skills/creative/baoyu-infographic in the NousResearch vault.",
    ),
    (
        "kb:skill-nous-systematic-debugging",
        "Debugging skill: the four-phase systematic root-cause protocol — understand first, then touch (root-cause first)",
        "A four-phase debugging protocol with the central law of no fix before root cause — ported from obra/superpowers: "
        "Phase 1 REPRODUCE — the bug must reproduce deterministically; a bug that does not reproduce is a guess, and the search moves toward environmental conditions right here. Phase 2 ISOLATE — the smallest possible reproduction, "
        "bisecting inputs, dependencies and code paths until one variable remains. Phase 3 ROOT CAUSE — name the exact mechanism, prove it with evidence, and reject the first plausible story. Phase 4 FIX — patch the cause, "
        "not the symptom, then re-run the Phase 1 reproduction as the regression test. In MAIRON the studio's self-healing loop and the engineering lanes follow this protocol so a shipped fix always carries its proof. "
        "Source: skills/engineering/systematic-debugging in the NousResearch vault.",
    ),
    (
        "kb:skill-nous-test-driven-development",
        "Testing skill: strict RED-GREEN-REFACTOR TDD — the test before the code, always",
        "Strict test-driven development (ported from obra/superpowers): the iron law of no production code without a prior failing test. The three-part cycle: RED — write the next small test and run it; it must fail for the right reason (missing behavior, not a compile error), and the red failure must prove exactly the behavioral claim; GREEN — the smallest possible code that turns only this test green — no further, no better; optimization is forbidden here; REFACTOR — with green tests, improve the structure (names, duplication, dependencies) while the behavior stays identical and the tests stay green. "
        "Hard rules: never run more than one red cycle at once; never fix a test and the code at the same time (pick one first); a removed test means a removed behavior and must be stated explicitly; in Rust every bug fix opens with a test reproducing that bug. In MAIRON this is the default development flow — the current 360 kernel tests were built with this discipline. Source: skills/software-development/test-driven-development in the NousResearch vault.",
    ),
    (
        "kb:skill-nous-codebase-inspection",
        "Exploration skill: fast codebase archaeology — LOC, language ratios and the heat map before any change",
        "Fast structural archaeology of the codebase before every big change: LOC stats and language ratios (parsing by an internal logic, not pygount), heavyweight risky files, the hottest modules (most imports/dependents) "
        "and an entry map for the newly arriving agent. Three-step protocol: STEP 1 — MAP: a depth-3 directory tree split by crates/packages/apps, identifying the domain core from the shells; STEP 2 — MEASURE: LOC per crate, the biggest files (a file over 1000 lines is a review risk), the test-to-code ratio, "
        "the entry points (main.rs, lib.rs, routes); STEP 3 — REPORT: three answerable paragraphs — where to start, what is dangerous, what is stable — with numbers and paths, never vague description. Forbidden: reading every file (entry points + hot files only), guessing the architecture without seeing Cargo.toml "
        "and the module tree. In MAIRON every \"inspect this repo\" task answers with this protocol and a structured output. Source: skills/software-development/codebase-inspection in the NousResearch vault.",
    ),
    (
        "kb:skill-nous-grounded-citations",
        "Credibility skill: every claim carries a verifiable source — a claim-by-claim audit",
        "Grounded citations for answers and docs (ported from Teknium): every factual claim carries a verifiable source, and every source is actually opened — never guessed from the model's memory. Protocol: (1) claim split — the output text decomposes into atomic claims; every statistical/temporal/attribution claim must carry an address; "
        "(2) the source triad — a primary origin (official doc, code, log) outranks a secondary one (news, blog); each citation includes the address + access date + the exact quote carrying the claim; (3) reverse audit — after writing, every citation is re-opened and the quote is matched against the claim; a citation that does not support the claim is dropped or the claim is corrected; (4) the no-source label — model-internal-knowledge claims are labeled explicitly "
        "basis: own-knowledge and are forbidden in financial/legal reports (aligned with kb:skill-a2ui-self-healing-loop); (5) blunt uncertainty — \"I could not find it\" beats a guessed source. In MAIRON the M4 brain's research answers and the audit reports follow this protocol. Source: skills/research/grounded-citations in the NousResearch vault.",
    ),
    (
        "kb:skill-nous-product-price-monitor",
        "Data skill: product/flight/listing price watching with a target threshold and smart alerts",
        "Price watching for products, flights or listings with a target threshold and alerts (ported from hermes-agent): a three-part architecture — a periodic watcher (cron, not a tight loop — aligned with the 15-minute fatigue-guard floor), a price-history store (min price/last price/trend), and a threshold alerter (price <= target -> a one-time notification per run, never spam). "
        "Rules: (1) every watch has a stable id (a normalized URL or SKU) so runs attach to the same record; (2) the price reads from structure (JSON-LD, schema.org, a specific selector), never a vague regex over the whole page — a false price from reserved text is worse than no data; (3) every price change records the change rate; sub-noise wobble (say < 1 percent) never alerts; (4) scraping stays inside the sandbox and "
        "never takes user keys/cookies — public sources only; (5) a failed read records an explicit failure, never the stale last price (stale data is more dangerous than no data). MAIRON use: marketplace product price watching and trend reports for sellers. Source: skills/productivity/product-price-monitor in the NousResearch vault.",
    ),
    (
        "kb:skill-nous-hermes-function-calling",
        "Protocol skill: the Hermes <tools> XML+JSON schema and the progressive-disclosure standard",
        "The Hermes function-calling tool standard for open-weight models without native support: (1) valid strict double-quote JSON — trailing commas and single-quoted keys recovered only by a lenient parse fallback; "
        "(3) every tool schema carries a complete JSON Schema (parameter, type, description) — schema-free free models cannot invent types; (2) progressive disclosure — the tool list streams by group so a long catalog never blows the context window; (4) every call validates against the schema before dispatch, unresolved placeholder arguments are detected, and a refusal field is checked before parsing so a model refusal never masquerades as a tool result. "
        "In MAIRON this standard drives the agent fleet's MCP/A2A calls. Source: skills/protocol/function-calling in the NousResearch vault.",
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
    for q in ("design system", "p5.js", "root cause", "RED-GREEN-REFACTOR", "progressive disclosure"):
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
    print(f"[3/3] after: KbEntry={after['kb']} (39 -> 49 expected) | active products={after['active']} | "
          f"trial balance sum={after['trial_sum']}")
    for q, rows, ms in after["search"]:
        flag = "OK" if ms < 5.0 else "SLOW"
        print(f"      search '{q}': {rows} hits in {ms:.2f} ms [{flag}]")

    # sync to download deliverable
    DOWNLOAD_DB.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(SEED_DB, DOWNLOAD_DB)
    print(f"      synced seed db -> {DOWNLOAD_DB}")

    ok = (after["kb"] == 49 and after["active"] == 20 and after["trial_sum"] == 0
          and ks["updated"] == 0 and (ks["inserted"] == 10 or ks["skipped"] == 10)
          and all(ms < 5.0 for _, _, ms in after["search"]))
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
