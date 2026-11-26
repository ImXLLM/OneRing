#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_copilotkit_gems.py — inject the six selected CopilotKit/OpenBot vault gems into MAIRON (phase 2, wave of 2026-11).

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
        "Studio skill: jump-free HTML streaming with the sandbox guard and CSP (OpenIntelligentUI)",
        "The live studio stream must never throw away the preview iframe. Three defense layers ported from OpenIntelligentUI: "
        "(1) processPartialHtml — a pure DOM-free function that strips an open tag at the end of the document (</...>), an open <script> block with all of its content "
        "so incomplete JS never raises a syntax error, an incomplete trailing entity, and injects the responsive guard style (box-sizing: border-box, "
        "max-width: 100vw, overflow-x: hidden) only when the document lacks it; (2) settledArgs/is_settled_stream_arg — until a stream call completes, hex colors with partial lengths (4/7/9), "
        "unfinished URLs (ending in \"/\" or \".\" or shorter than 12) and numbers ending in \".\" or \"-\" never reach the UI so the component never crashes; (3) the iframe sandbox uses sandbox=\"allow-scripts\" only — no allow-same-origin, "
        "so product code never reaches the session or cookies, plus a tight CSP on the document header. MAIRON port: apps/miniapp/lib/studio-html-guard.ts "
        "(wired to the live studio preview) and crates/ai/src/history_repair.rs. Golden rule: the downloadable/published version is always the untouched complete "
        "document; the guard applies to the live preview only.",
        "The live studio stream must never throw away the preview iframe. Three defense layers ported from OpenIntelligentUI: "
        "(1) processPartialHtml — a pure DOM-free function that strips an open tag at the end of the document (</...>), an open <script> block with all of its content "
        "so incomplete JS never raises a syntax error, an incomplete trailing entity, and injects the responsive guard style (box-sizing: border-box, "
        "max-width: 100vw, overflow-x: hidden) only when the document lacks it; (2) settledArgs/is_settled_stream_arg — until a stream call completes, hex colors with partial lengths (4/7/9), "
        "unfinished URLs (ending in a slash or dot, or shorter than 12) and numbers ending in a dot or dash never reach the UI so the component never crashes; (3) the iframe sandbox uses sandbox=\"allow-scripts\" only — no allow-same-origin, "
        "so product code never reaches the session or cookies, plus a tight CSP on the document header. MAIRON port: apps/miniapp/lib/studio-html-guard.ts "
        "(wired to the live studio preview) and crates/ai/src/history_repair.rs. Golden rule: the downloadable/published version is always the untouched complete "
        "document; the guard applies to the live preview only.",
    ),
    (
        "kb:skill-generative-ui-zero-chroma",
        "Studio skill: the generative-UI design system — the zero-chroma #0a0a0a/#fafafa palette with #00bba7/#ff6467 accents and valueAsNumber-safe math",
        "The generative-UI design system gives model-produced output two deterministic neutral themes: a dark theme with the #0a0a0a background and #fafafa text, "
        "and the inverse light theme; both hold AAA contrast. Only two accent colors are allowed: success green #00bba7 and destructive red #ff6467 — "
        "no loud colors or flashing gradients belong in studio output; brand colors come only from theme variables, never from the model. Layout rules: "
        "spacing on the 4px scale, 8-12px radii, a 16px baseline grid, and logical-direction layout (padding-inline instead of left). "
        "Math safety rule: every money/numeric form-input computation uses valueAsNumber — never parseFloat/parseInt on "
        "innerHTML or an input's raw string; NaN and the empty string map explicitly to 0 or a form error, and no string is ever passed straight to eval or "
        "innerHTML. Charts always draw with an axis safety margin and no neck-breaking floats (max 2 decimal places). "
        "These rules govern the studio design prompt and the MAIRON components at once so every model's output looks like a real product.",
        "The generative-UI design system gives model-produced output two deterministic neutral themes: a dark theme with the #0a0a0a background and #fafafa text, "
        "and the inverse light theme; both hold AAA contrast. Only two accent colors are allowed: success green #00bba7 and destructive red #ff6467 — "
        "no loud colors or flashing gradients belong in studio output; brand colors come only from theme variables, never from the model. Layout rules: "
        "spacing on the 4px scale, 8-12px radii, a 16px baseline grid, and logical-direction layout (padding-inline instead of left). "
        "Math safety rule: every money/numeric form-input computation uses valueAsNumber — never parseFloat/parseInt on "
        "innerHTML or an input's raw string; NaN and the empty string map explicitly to 0 or a form error, and no string is ever passed straight to eval or "
        "innerHTML. Charts always draw with an axis safety margin and no neck-breaking floats (max 2 decimal places). "
        "These rules govern the studio design prompt and the MAIRON components at once so every model's output looks like a real product.",
    ),
    (
        "kb:skill-ag-ui-protocol-v1",
        "Studio skill: the AG-UI 1.0 standard protocol — agent-to-UI streaming with the 30-event standard family",
        "AG-UI 1.0 is the shared agent-to-UI streaming contract; instead of ad-hoc SSEs, every event is a packet with a \"type\" from the "
        "standard family: TEXT_MESSAGE_START/CONTENT/END for model text, TOOL_CALL_START/ARGS/END/RESULT for tool calls, "
        "STATE_SNAPSHOT and STATE_DELTA (JSON patches) for shared state, THINKING_* for reasoning, STEP_*/MESSAGES_SNAPSHOT/Raw/Custom and "
        "RUN_STARTED/FINISHED/ERROR for the run lifecycle. Serialization rules: type names SCREAMING_SNAKE_CASE, fields camelCase (messageId, "
        "toolCallId, parentMessageId), and a null field is never serialized. MAIRON adds three extension events — ACTIVITY_SNAPSHOT/ACTIVITY_DELTA and "
        "SUBAGENT_SPAWN — on the same contract so studio progress and subagent births stay standard. Port: "
        "crates/hermes/src/ag_ui.rs with ready-made stream builders (text_message_content, tool_call_args, state_snapshot, ...) and a full "
        "14-event flow test. Benefit: web, mini-app and the audit log share one language, and the renderer reads every agent's output with no translator.",
        "AG-UI 1.0 is the shared agent-to-UI streaming contract; instead of ad-hoc SSEs, every event is a packet with a type from the "
        "standard family: TEXT_MESSAGE_START/CONTENT/END for model text, TOOL_CALL_START/ARGS/END/RESULT for tool calls, "
        "STATE_SNAPSHOT and STATE_DELTA (JSON patches) for shared state, THINKING_* for reasoning, STEP_*/MESSAGES_SNAPSHOT/Raw/Custom and "
        "RUN_STARTED/FINISHED/ERROR for the run lifecycle. Serialization rules: type names SCREAMING_SNAKE_CASE, fields camelCase (messageId, "
        "toolCallId, parentMessageId), and a null field is never serialized. MAIRON adds three extension events — ACTIVITY_SNAPSHOT/ACTIVITY_DELTA and "
        "SUBAGENT_SPAWN — on the same contract so studio progress and subagent births stay standard. Port: "
        "crates/hermes/src/ag_ui.rs with ready-made stream builders (text_message_content, tool_call_args, state_snapshot, ...) and a full "
        "14-event flow test. Benefit: web, mini-app and the audit log share one language, and the renderer reads every agent's output with no translator.",
    ),
    (
        "kb:skill-openbot-hitl-governance",
        "Governance skill: the 6-category human-approval gate with a dual voice and the secret-box pattern (OpenBot HITL)",
        "Six action categories always get an approval card: messaging others, moving or spending money, deleting or mutating data, publishing public content, "
        "handling credentials or access, and submitting forms/requests. Every decision carries a dual voice: one sentence for the model (outcome_for_model) stating "
        "who decided and what to do — on a human rejection it explicitly says \"Do not retry, rephrase, or work around it\" — and one for the person "
        "(outcome_for_person) that is blunt and plain (\"Your request was rejected.\"). Changing one never bypasses the other because both are defined and tested together. "
        "Secret-box pattern: password/token/credential handling is never delegated to the bot; the card says the task belongs to the human alone "
        "(\"I'll do it myself\") and no reply can sit in for them. The what-if simulator runs the same layer order for \"what would have happened\" so the "
        "simulator and the real gate can never disagree; in MAIRON the money gate comes from the charter: below 50,000 Din the escrow is automatic, above the threshold a human approves. "
        "Port: crates/hermes/src/approvals.rs (ApprovalCategory, ApprovalOutcome, ...).",
        "Six action categories always get an approval card: messaging others, moving or spending money, deleting or mutating data, publishing public content, "
        "handling credentials or access, and submitting forms/requests. Every decision carries a dual voice: one sentence for the model (outcome_for_model) stating "
        "who decided and what to do — on a human rejection it explicitly says \"Do not retry, rephrase, or work around it\" — and one for the person "
        "(outcome_for_person) that is blunt and plain (\"Your request was rejected.\"). Changing one never bypasses the other because both are defined and tested together. "
        "Secret-box pattern: password/token/credential handling is never delegated to the bot; the card says the task belongs to the human alone "
        "(\"I will do it myself\") and no reply can sit in for them. The what-if simulator runs the same layer order for \"what would have happened\" so the "
        "simulator and the real gate can never disagree; in MAIRON the money gate comes from the charter: below 50,000 Din the escrow is automatic, above the threshold a human approves. "
        "Port: crates/hermes/src/approvals.rs (ApprovalCategory, ApprovalOutcome, ...).",
    ),
    (
        "kb:skill-openbot-routine-fatigue",
        "Resilience skill: routine anti-spam — a 15-minute floor, a 20-cap, a 10-failure circuit breaker and deterministic UUIDv8 subagents",
        "Routines and goals are long-lived; without a guard they would fire en masse after every restart. Four fatigue-guard rules: "
        "(1) a 15-minute floor (900 seconds) between two fires of one routine — earlier calls are suppressed; (2) a 20-cap on concurrent routines — the twenty-first "
        "waits for a seat; (3) the circuit breaker: after 10 consecutive failures a routine turns itself off and one success re-closes the circuit; (4) skip "
        "missed cron windows after a restart — the routine waits for the next window and never fires every missed window at once. "
        "Subagents get a derived thread, not a multiplied one: SHA-256 over the formatted \"subagent\\n{parent}\\n{tool_call}\" rendered as "
        "UUIDv8 (version 8 — the bits are not random; the first 6 bytes are inherited from the parent) — both sides know the same derivation, so the client "
        "finds the transcript before the result arrives and a restart cannot re-birth it. The guard is fully deterministic: time is injected "
        "from outside and tests run without sleeps or chance. Port: crates/hermes/src/approvals.rs (RoutineFatigueGuard, ...).",
        "Routines and goals are long-lived; without a guard they would fire en masse after every restart. Four fatigue-guard rules: "
        "(1) a 15-minute floor (900 seconds) between two fires of one routine — earlier calls are suppressed; (2) a 20-cap on concurrent routines — the twenty-first "
        "waits for a seat; (3) the circuit breaker: after 10 consecutive failures a routine turns itself off and one success re-closes the circuit; (4) skip "
        "missed cron windows after a restart — the routine waits for the next window and never fires every missed window at once. "
        "Subagents get a derived thread, not a multiplied one: SHA-256 over the formatted subagent-parent-tool_call triple rendered as "
        "UUIDv8 (version 8 — the bits are not random; the first 6 bytes are inherited from the parent) — both sides know the same derivation, so the client "
        "finds the transcript before the result arrives and a restart cannot re-birth it. The guard is fully deterministic: time is injected "
        "from outside and tests run without sleeps or chance. Port: crates/hermes/src/approvals.rs (RoutineFatigueGuard, ...).",
    ),
    (
        "kb:skill-a2ui-self-healing-loop",
        "Studio skill: the generative-UI self-healing loop and chart provenance (basis: read | given | own-knowledge)",
        "Studio output is checked before render with the same validator the render gate uses — so the tool retry decision and the render suppress decision "
        "can never disagree. A healthy packet means: a complete <!DOCTYPE html>...</html> envelope, balanced script and style tags, and no markdown fence leak "
        "(the triple backtick). If a defect exists and attempts remain, structured feedback is built for the model naming exactly "
        "which tag is missing ([missing_doctype], [unclosed_script], [markdown_fence_leak]) — the next turn the model fixes only that; after the "
        "attempt cap (default 3) the a2ui_recovery_exhausted packet returns and the safe fallback renders. The loop state streams on the activity channel "
        "(activity_type: a2ui_recovery) so the user sees \"rebuilding...\". Chart provenance rule: every displayed number "
        "carries a declared origin — basis: read (read from a real source), given (user data) or own-knowledge (model knowledge; it must be labeled explicitly "
        "and is forbidden in invoices/financial reports); a chart without a provenance label never renders. Port: crates/ai/src/history_repair.rs "
        "(validate_and_build_self_healing_feedback, studio_defects) with tests for the named defects and the EXHAUSTED packet.",
        "Studio output is checked before render with the same validator the render gate uses — so the tool retry decision and the render suppress decision "
        "can never disagree. A healthy packet means: a complete <!DOCTYPE html>...</html> envelope, balanced script and style tags, and no markdown fence leak "
        "(the triple backtick). If a defect exists and attempts remain, structured feedback is built for the model naming exactly "
        "which tag is missing ([missing_doctype], [unclosed_script], [markdown_fence_leak]) — the next turn the model fixes only that; after the "
        "attempt cap (default 3) the a2ui_recovery_exhausted packet returns and the safe fallback renders. The loop state streams on the activity channel "
        "(activity_type: a2ui_recovery) so the user sees the rebuilding state. Chart provenance rule: every displayed number "
        "carries a declared origin — basis: read (read from a real source), given (user data) or own-knowledge (model knowledge; it must be labeled explicitly "
        "and is forbidden in invoices/financial reports); a chart without a provenance label never renders. Port: crates/ai/src/history_repair.rs "
        "(validate_and_build_self_healing_feedback, studio_defects) with tests for the named defects and the EXHAUSTED packet.",
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
    for q in ("AG-UI", "jump-free", "secret box", "UUIDv8", "self-healing"):
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
