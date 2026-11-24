#!/usr/bin/env python3
"""Idempotent KIAN v2 SQLite Seeder for Agent Patch/Sandbox & Retry/HITL Guardrails.
Synthesized from openai/codex, anthropics/claude-code, langflow-ai/langflow, langchain-ai/langchain.
"""
from __future__ import annotations

import argparse
import sqlite3
from datetime import datetime, timezone
from pathlib import Path

SKILL_CODEX_PATCH_SANDBOX = """# Skill: Codex-Grade Patching, ExecPolicy & Container Egress Hardening

## 1. 4-Pass Fuzzy Patch Matching (from `openai/codex/apply-patch`)
Never reject an LLM code patch solely on trailing whitespace or smart quotes:
1. Exact line slice match
2. `trim_end()` line match
3. `trim()` line match
4. Unicode normalization (`U+2010..U+2015, U+2212 -> '-'`, curly quotes -> ASCII `'/"`, NBSP -> `' '`)

## 2. Declarative ExecPolicy & Multi-Layer Sandboxing
- Evaluate command prefixes deterministically (`Allow | Prompt | Forbidden`) before spawning shells.
- On Linux: combine Bubblewrap filesystem isolation + `PR_SET_NO_NEW_PRIVS` + Seccomp BPF blocking VM/WSL2 sockets.
- On macOS: enforce closed-by-default `(deny default)` Seatbelt profiles.
- In CI/containers (`anthropics/claude-code`): enforce `iptables` + `ipset` default-DROP egress firewalls verified with negative (`example.com` fails) and positive (`api.github.com/zen` succeeds) probes.
"""

SKILL_AGENT_RETRY_HITL = """# Skill: Agent Self-Healing Retry, Placeholder Guard & Checkpointed HITL

## 1. Placeholder Tool-Arg Correction (from `langflow` `WatsonXPlaceholderMiddleware`)
- Detect `<result-from-...>` or `<previous-output>` placeholders in `tool_calls[].args`.
- Perform a single-shot immutable corrective re-invoke (`override(messages=...)`) with a hint containing the actual prior `ToolMessage` output; never loop more than once.

## 2. Single-Tool Clamp & Nonce-Keyed HITL (`langflow` `SingleToolCallMiddleware` & `ToolApprovalMixin`)
- Clamp multi-tool calls immutably (`tool_calls[:1]`) when sequential determinism is required.
- Key every HITL `approve | edit | reject | respond` pause by a fresh per-pause `interrupt_id` nonce so stale approvals cannot satisfy future tool pauses.

## 3. OutputFixing & ToolRetry (from `langchain` `RetryWithError` & `ToolRetryMiddleware`)
- Guard structured parsing with `raise_if_refused` before JSON/XML parsing.
- On `OutputParserException`, feed `{instructions}`, `{completion}`, and `{error}` back for bounded retries.
- Wrap external tools with `ToolRetryMiddleware` (`retry_on` predicate + exponential backoff + graceful `on_failure` error string).
"""

ENTRIES = [
    (
        "kb:skill-codex-patch-and-sandbox",
        "skill",
        "Codex-Grade Patching, ExecPolicy & Container Egress Hardening",
        SKILL_CODEX_PATCH_SANDBOX,
        "openai/codex,anthropics/claude-code,rust,sandbox,apply-patch",
    ),
    (
        "kb:skill-agent-retry-and-hitl-guardrails",
        "skill",
        "Agent Self-Healing Retry, Placeholder Guard & Checkpointed HITL",
        SKILL_AGENT_RETRY_HITL,
        "langflow,langchain,retry,hitl,structured-output",
    ),
]


def ensure_schema(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS "KbEntry" (
            "id" TEXT PRIMARY KEY,
            "kind" TEXT NOT NULL,
            "title" TEXT NOT NULL,
            "content" TEXT NOT NULL,
            "tags" TEXT NOT NULL DEFAULT '',
            "updatedAt" TEXT NOT NULL
        )
        """
    )


def seed(db_path: Path) -> dict[str, str]:
    now = datetime.now(timezone.utc).isoformat()
    results: dict[str, str] = {}
    with sqlite3.connect(db_path) as conn:
        ensure_schema(conn)
        for entry_id, kind, title, content, tags in ENTRIES:
            existing = conn.execute('SELECT id FROM "KbEntry" WHERE id = ?', (entry_id,)).fetchone()
            if existing:
                conn.execute(
                    'UPDATE "KbEntry" SET kind = ?, title = ?, content = ?, tags = ?, updatedAt = ? WHERE id = ?',
                    (kind, title, content, tags, now, entry_id),
                )
                results[entry_id] = "updated"
            else:
                conn.execute(
                    'INSERT INTO "KbEntry" (id, kind, title, content, tags, updatedAt) VALUES (?, ?, ?, ?, ?, ?)',
                    (entry_id, kind, title, content, tags, now),
                )
                results[entry_id] = "inserted"
        conn.commit()
    return results


def main() -> None:
    parser = argparse.ArgumentParser(description="Inject KIAN v2 agent resilience skills into SQLite KbEntry.")
    parser.add_argument("--db", default="seed/kian-production.db", help="Path to SQLite database")
    args = parser.parse_args()
    db_path = Path(args.db)
    db_path.parent.mkdir(parents=True, exist_ok=True)
    res = seed(db_path)
    for k, v in res.items():
        print(f"{k}: {v}")


if __name__ == "__main__":
    main()
