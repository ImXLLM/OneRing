#!/usr/bin/env python3
"""Idempotently inject two audited Telegram Mini App skills into SQLite KbEntry.

Usage (from the KIAN repository root):
  python3 inject_tdesktop_tma_skills.py --db seed/kian-production.db

The injector inspects KbEntry columns and supports common key/content schemas.
It never creates a missing database, guesses unknown required columns, or deletes
rows. It refuses duplicate matching keys and updates existing rows in place.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sqlite3
import sys
import uuid
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

SOURCE_COMMIT = "070dbc645c543ce8b9bbbb7b72ef842ffd7671da"


@dataclass(frozen=True)
class Skill:
    key: str
    title: str
    summary: str
    content: str
    tags: tuple[str, ...]


SKILLS = (
    Skill(
        key="kb:skill-tdesktop-tma-bridge",
        title="Telegram Mini App bridge patterns for KIAN",
        summary="TDesktop launch provenance, safe-area/haptic UX, WebApp bridge trust boundaries, and KIAN initData verification.",
        tags=("telegram", "mini-app", "tdesktop", "security"),
        content=f"""# Telegram Mini App bridge patterns for KIAN

Audit source: `telegramdesktop/tdesktop` at `{SOURCE_COMMIT}`. The native Mini App host is in `inline_bots/bot_attach_web_view.*` and `ui/chat/attach/attach_bot_webview.*`; TDesktop is a client using internal MTProto calls, not a Bot API server.

## Launch and client UX
- Preserve launch provenance (keyboard/simple WebView, main-menu, attach-menu, direct link, bot profile, etc.) as a typed source. Treat `start_param` as routing/context, never as an authorization credential.
- In the browser, use Telegram's documented WebApp API. Call `ready()` after initial UI render, sync theme and safe/content-safe insets, and gracefully handle unsupported APIs.
- Disable vertical swipe-to-close only while it conflicts with a Mini App gesture; restore swipes when that interaction ends. Haptics and device capabilities are optional UX, not app logic.
- TDesktop's native `TelegramWebviewProxy` / `TelegramGameProxy` messages are implementation details. Do not make KIAN depend on that private bridge.

## Trust boundary
- Send the raw `Telegram.WebApp.initData` string to the Rust backend. Never authenticate with `initDataUnsafe`, a browser-supplied Telegram ID, `start_param`, or a native WebView event.
- For bot-token validation, parse the query string, sort decoded `key=value` pairs, join with LF, exclude `hash`, and keep a received `signature` field in the HMAC check string. Derive `secret = HMAC-SHA256(key="WebAppData", message=bot_token)` and compare the expected SHA-256 HMAC in constant time. Enforce a short `auth_date` freshness window and small future-clock skew.
- Parse `user` only after successful signature validation. Keep the bot token server-only; issue a KIAN session for subsequent API calls if needed.
- TDesktop's origin checks, local trusted-bot prompts, write-access consent, and per-instance guards are client protections/UX—not KIAN backend authorization. Recheck KIAN permissions on every privileged request.
- Validate JSON schema, types, and size at every boundary. Do not log `initData`, bot tokens, hashes, or raw user payloads.

## Small handoff vs API calls
- Telegram `sendData` is a keyboard-button-only handoff with a 4096-byte limit and closes the Mini App. Handle `message.web_app_data` as arbitrary untrusted input, validate it, and use durable deduplication for mutations.
- Use authenticated KIAN API requests for larger data or ordinary Mini App actions. Client-side one-shot flags are not replay protection.

## KIAN helper
`kian-tma-bridge.ts` supplies safe-area CSS variables, optional haptics, a swipe preference wrapper, and a referral `startapp` URL builder. Referral links are attribution hints; verify/refuse referral claims server-side.""",
    ),
    Skill(
        key="kb:skill-telegram-stars-and-deeplinks",
        title="Telegram Stars invoices and startapp deeplinks",
        summary="KIAN startapp payload parsing, XTR invoice payloads, and authoritative Bot API payment handling.",
        tags=("telegram", "stars", "payments", "deeplinks"),
        content=f"""# Telegram Stars invoices and startapp deeplinks

Audit source: `telegramdesktop/tdesktop` at `{SOURCE_COMMIT}`. TDesktop's native checkout UI is not payment authority for KIAN; implement the public Bot API flow in `crates/bot`.

## Deep links
- KIAN start parameter shape: `p_<productId>_r_<refTgId>` (example: `p_42_r_123456789`). Parse strictly; reject zero, malformed, overflow, and trailing data.
- Build links as `https://t.me/<bot_username>?startapp=<URL-encoded-start-param>`; use Telegram's share URL only for the share sheet. Treat all parameters as untrusted until tied to verified `initData` and server-side product/referral rules.
- Derive the current Telegram user from validated `initData`, never from the `refTgId` field. Apply referral attribution once and idempotently.

## Stars and invoices
- Stars invoices use currency `XTR` and integer Star amounts. Use server-calculated prices; do not accept client-provided amount or currency.
- Create a pending order in SQLite before issuing the invoice. Put a compact, server-generated order reference in `invoice_payload` (Bot API payload limit: 128 bytes). On payment updates, look up that order and compare expected buyer/product/amount/currency against the DB record; the payload itself is not an authorization token.
- Answer every `pre_checkout_query` promptly (Telegram's Bot API deadline is 10 seconds), after checking order state, availability, expected amount/currency, and buyer.
- Grant goods/credits only from a verified `successful_payment` update. Store Telegram `update_id`, invoice/order ID, `telegram_payment_charge_id`, currency, and integer amount with uniqueness/idempotency constraints so retries or duplicate events cannot double-fulfill.
- Treat the Mini App's `openInvoice` callback (`paid`, `failed`, `pending`, `cancelled`) as display state only; reconcile entitlement/balance from the Bot API payment update.

## Mini App authentication
The Mini App sends raw `initData` to Rust. Validate the Bot-token HMAC using `WebAppData` as HMAC key; exclude only `hash` from that check string and retain `signature` when present. This differs from third-party Ed25519 validation. Enforce `auth_date` freshness before trusting signed identity/context.""",
    ),
)

KEY_CANDIDATES = ("key", "entry_key", "kb_key", "slug", "name")
CONTENT_CANDIDATES = ("content", "content_md", "markdown", "body", "text", "value", "data", "document", "instructions")
TITLE_CANDIDATES = ("title", "label", "display_name")


def quote_identifier(identifier: str) -> str:
    return '"' + identifier.replace('"', '""') + '"'


def normalized(name: str) -> str:
    return "".join(ch.lower() for ch in name if ch.isalnum())


def choose_column(columns: list[sqlite3.Row], candidates: tuple[str, ...]) -> sqlite3.Row | None:
    by_name = {str(col["name"]).lower(): col for col in columns}
    for candidate in candidates:
        found = by_name.get(candidate.lower())
        if found is not None:
            return found
    return None


def locate_table(conn: sqlite3.Connection, requested: str) -> str:
    row = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=? COLLATE NOCASE",
        (requested,),
    ).fetchone()
    if row is None:
        raise RuntimeError(f"SQLite table {requested!r} was not found; database was not modified")
    return str(row[0])


def table_columns(conn: sqlite3.Connection, table: str) -> list[sqlite3.Row]:
    return list(conn.execute(f"PRAGMA table_info({quote_identifier(table)})"))


def utc_values(declared_type: str) -> str | int:
    now = datetime.now(timezone.utc)
    if "INT" in declared_type.upper():
        return int(now.timestamp())
    return now.isoformat(timespec="seconds").replace("+00:00", "Z")


def payload_for(skill: Skill, payload_column: sqlite3.Row, payload_format: str) -> str:
    declared = str(payload_column["type"] or "").upper()
    use_json = payload_format == "json" or (payload_format == "auto" and "JSON" in declared)
    if not use_json:
        return skill.content
    return json.dumps(
        {
            "key": skill.key,
            "title": skill.title,
            "summary": skill.summary,
            "content": skill.content,
            "tags": list(skill.tags),
            "source_commit": SOURCE_COMMIT,
        },
        ensure_ascii=False,
        separators=(",", ":"),
    )


def build_values(
    skill: Skill,
    columns: list[sqlite3.Row],
    key_col: sqlite3.Row,
    content_col: sqlite3.Row,
    title_col: sqlite3.Row | None,
    payload_format: str,
    *,
    inserting: bool,
) -> dict[str, Any]:
    key_name = str(key_col["name"])
    content_name = str(content_col["name"])
    title_name = str(title_col["name"]) if title_col is not None else None
    values: dict[str, Any] = {}
    now_iso = datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")

    for col in columns:
        name = str(col["name"])
        kind = normalized(name)
        declared = str(col["type"] or "")
        value: Any = None
        mapped = True

        if name == key_name:
            value = skill.key
        elif name == content_name:
            value = payload_for(skill, content_col, payload_format)
        elif title_name is not None and name == title_name:
            value = skill.title
        elif kind in {"summary", "excerpt"}:
            value = skill.summary
        elif kind in {"category", "group", "namespace"}:
            value = "telegram"
        elif kind in {"type", "kind", "entrytype"}:
            value = "skill"
        elif kind in {"tags", "labels"}:
            value = json.dumps(list(skill.tags), ensure_ascii=False)
        elif kind in {"metadata", "meta"}:
            value = json.dumps(
                {"source_commit": SOURCE_COMMIT, "tags": list(skill.tags)},
                ensure_ascii=False,
                separators=(",", ":"),
            )
        elif kind in {"source", "sourcerepo", "repository"}:
            value = "telegramdesktop/tdesktop"
        elif kind in {"sourcecommit", "commit"}:
            value = SOURCE_COMMIT
        elif kind in {"createdat", "created", "timestamp"}:
            if inserting:
                value = utc_values(declared)
            else:
                mapped = False
        elif kind in {"updatedat", "updated", "modifiedat"}:
            # Set update timestamps only when content actually changes.
            if inserting:
                value = utc_values(declared)
            else:
                mapped = False
        elif kind in {"id", "entryid", "kbid"}:
            if not inserting:
                mapped = False
            elif int(col["pk"] or 0) > 0 and "INT" in declared.upper():
                mapped = False  # SQLite rowid/autoincrement.
            elif "INT" in declared.upper():
                value = int.from_bytes(hashlib.sha256(skill.key.encode()).digest()[:8], "big") & ((1 << 63) - 1)
            else:
                value = uuid.uuid5(uuid.NAMESPACE_URL, "kian:" + skill.key).hex
        else:
            mapped = False

        if mapped and value is not None:
            values[name] = value
        elif inserting and bool(col["notnull"]) and col["dflt_value"] is None:
            is_integer_primary_key = int(col["pk"] or 0) > 0 and "INT" in declared.upper()
            if not is_integer_primary_key:
                raise RuntimeError(
                    f"Unsupported required KbEntry column {name!r}; "
                    "add an explicit mapping before injecting"
                )

    return values


def update_timestamp_values(columns: list[sqlite3.Row]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for col in columns:
        if normalized(str(col["name"])) in {"updatedat", "updated", "modifiedat"}:
            result[str(col["name"])] = utc_values(str(col["type"] or ""))
    return result


def inject_one(
    conn: sqlite3.Connection,
    table: str,
    skill: Skill,
    columns: list[sqlite3.Row],
    key_col: sqlite3.Row,
    content_col: sqlite3.Row,
    title_col: sqlite3.Row | None,
    payload_format: str,
) -> str:
    key_name = str(key_col["name"])
    qtable = quote_identifier(table)
    qkey = quote_identifier(key_name)
    rows = list(conn.execute(f"SELECT * FROM {qtable} WHERE {qkey}=?", (skill.key,)))
    if len(rows) > 1:
        raise RuntimeError(f"{skill.key!r} occurs {len(rows)} times; refusing to choose a duplicate row")

    insert_values = build_values(
        skill, columns, key_col, content_col, title_col, payload_format, inserting=True
    )
    if not rows:
        names = list(insert_values)
        sql = (
            f"INSERT INTO {qtable} ({', '.join(quote_identifier(name) for name in names)}) "
            f"VALUES ({', '.join('?' for _ in names)})"
        )
        conn.execute(sql, tuple(insert_values[name] for name in names))
        return "inserted"

    mutable_values = build_values(
        skill, columns, key_col, content_col, title_col, payload_format, inserting=False
    )
    existing = rows[0]
    changed = {
        name: value
        for name, value in mutable_values.items()
        if name != key_name and existing[name] != value
    }
    if not changed:
        return "unchanged"
    changed.update(update_timestamp_values(columns))
    names = list(changed)
    sql = (
        f"UPDATE {qtable} SET "
        + ", ".join(f"{quote_identifier(name)}=?" for name in names)
        + f" WHERE {qkey}=?"
    )
    cursor = conn.execute(sql, tuple(changed[name] for name in names) + (skill.key,))
    if cursor.rowcount != 1:
        raise RuntimeError(f"Expected one updated row for {skill.key!r}, got {cursor.rowcount}")
    return "updated"


def inject(db_path: Path, table_requested: str = "KbEntry", payload_format: str = "auto", dry_run: bool = False) -> dict[str, int]:
    if not db_path.is_file():
        raise RuntimeError(f"Database file does not exist: {db_path}; refusing to create an empty database")

    conn = sqlite3.connect(str(db_path), timeout=10.0)
    conn.row_factory = sqlite3.Row
    counts = {"inserted": 0, "updated": 0, "unchanged": 0}
    try:
        conn.execute("PRAGMA foreign_keys=ON")
        conn.execute("BEGIN IMMEDIATE")
        table = locate_table(conn, table_requested)
        columns = table_columns(conn, table)
        if not columns:
            raise RuntimeError(f"Table {table!r} has no visible columns")
        key_col = choose_column(columns, KEY_CANDIDATES)
        content_col = choose_column(columns, CONTENT_CANDIDATES)
        title_col = choose_column(columns, TITLE_CANDIDATES)
        if key_col is None or content_col is None:
            found = ", ".join(str(col["name"]) for col in columns)
            raise RuntimeError(
                f"Could not identify key/content columns in {table!r} (found: {found}); "
                "use a KbEntry schema with a key and content/value column"
            )

        for skill in SKILLS:
            status = inject_one(
                conn, table, skill, columns, key_col, content_col, title_col, payload_format
            )
            counts[status] += 1
        if dry_run:
            conn.rollback()
        else:
            conn.commit()
        return counts
    except Exception:
        conn.rollback()
        raise
    finally:
        conn.close()


def main(argv: list[str] | None = None) -> int:
    default_db = os.environ.get("KIAN_DB_PATH", "seed/kian-production.db")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", type=Path, default=Path(default_db), help="existing KIAN SQLite file")
    parser.add_argument("--table", default="KbEntry", help="knowledge-entry table (default: KbEntry)")
    parser.add_argument(
        "--payload-format",
        choices=("auto", "text", "json"),
        default="auto",
        help="JSON columns are detected in auto mode; otherwise store Markdown text",
    )
    parser.add_argument("--dry-run", action="store_true", help="execute transaction then roll it back")
    args = parser.parse_args(argv)
    db_path = args.db.expanduser().resolve()
    try:
        counts = inject(db_path, args.table, args.payload_format, args.dry_run)
    except (OSError, sqlite3.Error, RuntimeError) as exc:
        print(f"inject failed: {exc}", file=sys.stderr)
        return 2
    action = "would apply (rolled back)" if args.dry_run else "applied"
    print(
        f"{action}: {counts['inserted']} inserted, {counts['updated']} updated, "
        f"{counts['unchanged']} unchanged in {db_path}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
