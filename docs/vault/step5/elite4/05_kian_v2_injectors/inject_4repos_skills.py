#!/usr/bin/env python3
"""
inject_4repos_skills.py — Idempotent KIAN v2 SQLite/`KbEntry` injector for the 4 Elite Repos.

Upserts into `seed/kian-production.db` (`"KbEntry"` table):
  - All 19 official Anthropic Skills (`kb:skill-anthropic-<slug>`)
  - Curated high-impact Agency-Agents personas for KIAN v2's 7-role fleet & @ImXforevr (`kb:agent-agency-<slug>`)
  - All 6 Ponytail YAGNI / audit / debt skills (`kb:skill-ponytail-<slug>`)

Strictly preserves financial ledger integrity (`trialBalanceSum == 0`) and is 100% idempotent.
"""

import argparse
import datetime
import glob
import json
import os
import re
import sqlite3
import sys
from typing import Any, Dict, List, Optional, Tuple


DEFAULT_DB_PATH = "seed/kian-production.db"
VAULT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

# High-impact Agency-Agents personas mapped to KIAN v2 7-role fleet + @ImXforevr growth engine
CURATED_AGENCY_PERSONAS = {
    # @ImXforevr Growth & Design Engine
    "marketing/marketing-twitter-engager.md",
    "marketing/marketing-growth-hacker.md",
    "marketing/marketing-content-creator.md",
    "marketing/marketing-social-media-strategist.md",
    "marketing/marketing-seo-specialist.md",
    "design/design-ui-designer.md",
    "design/design-whimsy-injector.md",
    "design/design-ui-finish-gate-reviewer.md",
    "design/design-brand-guardian.md",
    "design/design-ux-architect.md",
    "testing/testing-reality-checker.md",
    "testing/testing-evidence-collector.md",
    # KIAN v2 7-Role Fleet Mapping (orchestrator, merchant, finance, support, analyst, scout, guardian)
    "specialized/agents-orchestrator.md",
    "specialized/operations-manager.md",
    "finance/finance-bookkeeper-controller.md",
    "finance/finance-financial-analyst.md",
    "finance/finance-fpa-analyst.md",
    "support/support-support-responder.md",
    "support/support-analytics-reporter.md",
    "product/product-trend-researcher.md",
    "engineering/engineering-multi-agent-systems-architect.md",
    "engineering/engineering-autonomous-optimization-architect.md",
    "engineering/engineering-rust-refactoring-specialist.md",
    "engineering/engineering-code-reviewer.md",
    "engineering/engineering-minimal-change-engineer.md",
}


def parse_frontmatter(txt: str) -> Dict[str, str]:
    meta: Dict[str, str] = {}
    if txt.startswith("---"):
        parts = txt.split("---", 2)
        if len(parts) >= 3:
            for line in parts[1].splitlines():
                m = re.match(r"^([a-zA-Z0-9_-]+):\s*(.+)$", line.strip())
                if m:
                    meta[m.group(1)] = m.group(2).strip().strip('"').strip("'")
    return meta


def collect_records(vault_root: str, include_all_agency: bool = False) -> List[Dict[str, Any]]:
    records: List[Dict[str, Any]] = []

    # 1. All official Anthropic Skills
    anth_dir = os.path.join(vault_root, "01_anthropics_official_skills")
    for skill_md in sorted(glob.glob(os.path.join(anth_dir, "*", "SKILL.md"))):
        slug = os.path.basename(os.path.dirname(skill_md))
        if slug.startswith("_"):
            continue
        txt = open(skill_md, encoding="utf-8", errors="ignore").read()
        meta = parse_frontmatter(txt)
        records.append({
            "id": f"kb:skill-anthropic-{slug}",
            "slug": f"anthropic-{slug}",
            "title": f"Anthropic Official Skill: {meta.get('name', slug)}",
            "summary": meta.get("description", "")[:400],
            "category": "skill:anthropic:official",
            "tags": json.dumps(["anthropics-skills", "official", slug]),
            "content": txt,
            "source": "https://github.com/anthropics/skills",
        })

    # 2. Agency-Agents Personas
    agency_dir = os.path.join(vault_root, "02_agency_agents_personas")
    for md_path in sorted(glob.glob(os.path.join(agency_dir, "*", "*.md"))):
        rel = os.path.relpath(md_path, agency_dir).replace("\\", "/")
        if not include_all_agency and rel not in CURATED_AGENCY_PERSONAS:
            continue
        div = rel.split("/")[0]
        slug = os.path.splitext(os.path.basename(md_path))[0]
        txt = open(md_path, encoding="utf-8", errors="ignore").read()
        meta = parse_frontmatter(txt)
        records.append({
            "id": f"kb:agent-agency-{slug}",
            "slug": f"agency-{slug}",
            "title": f"Agency Persona ({div}): {meta.get('name', slug)}",
            "summary": meta.get("description", "")[:400],
            "category": f"agent:agency:{div}",
            "tags": json.dumps(["agency-agents", div, slug]),
            "content": txt,
            "source": "https://github.com/msitarzewski/agency-agents",
        })

    # 3. Ponytail Skills
    pony_dir = os.path.join(vault_root, "04_ponytail_gems", "skills")
    for skill_md in sorted(glob.glob(os.path.join(pony_dir, "*", "SKILL.md"))):
        slug = os.path.basename(os.path.dirname(skill_md))
        txt = open(skill_md, encoding="utf-8", errors="ignore").read()
        meta = parse_frontmatter(txt)
        records.append({
            "id": f"kb:skill-ponytail-{slug}",
            "slug": f"ponytail-{slug}",
            "title": f"Ponytail Skill: {meta.get('name', slug)}",
            "summary": meta.get("description", "")[:400],
            "category": "skill:ponytail:lean",
            "tags": json.dumps(["ponytail", "yagni", slug]),
            "content": txt,
            "source": "https://github.com/DietrichGebert/ponytail",
        })

    return records


def check_trial_balance(conn: sqlite3.Connection) -> Tuple[bool, Optional[float]]:
    cur = conn.cursor()
    tables = {
        r[0]
        for r in cur.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()
    }
    for tbl in ("JournalLine", "LedgerEntry", "journal_lines", "ledger_entries"):
        if tbl in tables:
            cols = {
                r[1].lower(): r[1]
                for r in cur.execute(f'PRAGMA table_info("{tbl}")').fetchall()
            }
            if "debit" in cols and "credit" in cols:
                row = cur.execute(
                    f'SELECT COALESCE(SUM("{cols["debit"]}"), 0) - COALESCE(SUM("{cols["credit"]}"), 0) FROM "{tbl}"'
                ).fetchone()
                diff = float(row[0] or 0.0)
                if abs(diff) > 1e-6:
                    raise RuntimeError(f"Ledger violation in {tbl}: trialBalanceSum={diff} != 0")
                return True, diff
            if "amount" in cols:
                row = cur.execute(
                    f'SELECT COALESCE(SUM("{cols["amount"]}"), 0) FROM "{tbl}"'
                ).fetchone()
                diff = float(row[0] or 0.0)
                if abs(diff) > 1e-6:
                    raise RuntimeError(f"Ledger violation in {tbl}: trialBalanceSum={diff} != 0")
                return True, diff
    return False, None


def init_test_kb_table(conn: sqlite3.Connection) -> None:
    conn.execute(
        '''
        CREATE TABLE IF NOT EXISTS "KbEntry" (
            "id" TEXT PRIMARY KEY,
            "title" TEXT NOT NULL,
            "category" TEXT NOT NULL,
            "tags" TEXT NOT NULL,
            "summary" TEXT,
            "content" TEXT NOT NULL,
            "source" TEXT,
            "createdAt" TEXT NOT NULL,
            "updatedAt" TEXT NOT NULL
        )
        '''
    )


def upsert_records(
    conn: sqlite3.Connection,
    records: List[Dict[str, Any]],
    cols: List[str],
    dry_run: bool = False,
) -> Tuple[int, int]:
    now_iso = datetime.datetime.now(datetime.timezone.utc).isoformat()
    col_set = set(cols)
    inserted = 0
    updated = 0

    for rec in records:
        exists = conn.execute(
            'SELECT 1 FROM "KbEntry" WHERE "id" = ?', (rec["id"],)
        ).fetchone()

        row_map: Dict[str, Any] = {"id": rec["id"]}
        for candidate, val in [
            ("title", rec["title"]),
            ("name", rec["title"]),
            ("slug", rec["slug"]),
            ("category", rec["category"]),
            ("kind", rec["category"]),
            ("tags", rec["tags"]),
            ("summary", rec["summary"]),
            ("description", rec["summary"]),
            ("content", rec["content"]),
            ("body", rec["content"]),
            ("source", rec["source"]),
            ("sourceUrl", rec["source"]),
            ("createdAt", now_iso),
            ("created_at", now_iso),
            ("updatedAt", now_iso),
            ("updated_at", now_iso),
        ]:
            if candidate in col_set and candidate not in row_map:
                row_map[candidate] = val

        if exists:
            updated += 1
            if not dry_run:
                upd_cols = [
                    k for k in row_map.keys() if k not in ("id", "createdAt", "created_at")
                ]
                set_clause = ", ".join(f'"{k}" = ?' for k in upd_cols)
                conn.execute(
                    f'UPDATE "KbEntry" SET {set_clause} WHERE "id" = ?',
                    [row_map[k] for k in upd_cols] + [rec["id"]],
                )
        else:
            inserted += 1
            if not dry_run:
                ins_cols = list(row_map.keys())
                placeholders = ", ".join("?" for _ in ins_cols)
                col_names = ", ".join(f'"{k}"' for k in ins_cols)
                conn.execute(
                    f'INSERT INTO "KbEntry" ({col_names}) VALUES ({placeholders})',
                    [row_map[k] for k in ins_cols],
                )

    return inserted, updated


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Idempotently inject 4-repo elite skills & agency personas into KIAN v2 KbEntry"
    )
    parser.add_argument("--db", default=DEFAULT_DB_PATH)
    parser.add_argument("--vault", default=VAULT_ROOT)
    parser.add_argument("--all-agency", action="store_true", help="Ingest all extracted agency personas")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--init-if-missing", action="store_true")
    args = parser.parse_args()

    records = collect_records(args.vault, include_all_agency=args.all_agency)
    if not records:
        print(f"ERROR: no records found under {args.vault}", file=sys.stderr)
        return 1

    if not os.path.exists(args.db) and not args.init_if_missing:
        print(f"ERROR: database file not found: {args.db}", file=sys.stderr)
        return 1

    conn = sqlite3.connect(args.db)
    try:
        if args.init_if_missing:
            init_test_kb_table(conn)
        cols = [r[1] for r in conn.execute('PRAGMA table_info("KbEntry")').fetchall()]
        if not cols:
            raise RuntimeError('Table "KbEntry" not found in target SQLite DB.')
        has_ledger, _ = check_trial_balance(conn)
        inserted, updated = upsert_records(conn, records, cols, dry_run=args.dry_run)
        if args.dry_run:
            conn.rollback()
        else:
            conn.commit()
        _, post_sum = check_trial_balance(conn)
    finally:
        conn.close()

    mode = "DRY-RUN" if args.dry_run else "COMMITTED"
    ledger_msg = f"trialBalanceSum={post_sum} (verified == 0)" if has_ledger else "no ledger table in DB"
    print(
        f"[{mode}] Processed {len(records)} 4-repo entries -> "
        f"inserted={inserted}, updated={updated}, ledger={ledger_msg}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
