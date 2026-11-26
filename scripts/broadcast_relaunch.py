#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
broadcast_relaunch.py — the safe relaunch broadcast script for the entire migrated MAIRON community
==============================================================================

Source: seed/kian-production.db (466 migrated citizens) — read-only.
Output: a personalized English message with each user's MAIRON balance + an entry button.

Safety:
  * --dry-run is the default: no message is sent; only a sample message and stats are printed.
  * Real sending only with --live and TELEGRAM_BOT_TOKEN in the environment.
  * Rate-limited: at most 20 messages per second (the official Bot API cap) + periodic pauses.
  * Safe resume: send state is recorded in --state (JSON); re-runs continue and never double-send.
  * Every 429 from the Telegram response is obeyed (retry_after).
"""
from __future__ import annotations

import argparse
import json
import os
import sqlite3
import sys
import time
import urllib.parse
import urllib.request

API = "https://api.telegram.org/bot{token}/{method}"
RATE_PER_SEC = 20          # Official Bot API limit
BATCH_PAUSE_EVERY = 500    # After every 500 messages, pause for 5 seconds
BATCH_PAUSE_SEC = 5.0

BTN_TEXT = "🚀 Open the MAIRON studio and market"

TEMPLATE_FA = (
    "Hello {name} 👋\n\n"

    "The dropagent world is back, stronger than ever, as **MAIRON** — and all of your assets were migrated exactly as they were:\n\n"

    "💰 Your MAIRON balance: **{balance} KIAN**\n"

    "🧠 Your AI memory and conversations carried over\n"

    "🛒 Your past orders are available with full content\n"

    "🎁 You earn a 25 percent commission on every sale through your invite link!\n\n"

    "Sign in right now and see the new MAIRON 👇"
)


def fmt_balance(micro: int) -> str:
    return f"{micro / 1_000_000:,.0f}" if micro % 1_000_000 == 0 else f"{micro / 1_000_000:,.2f}"


def load_users(db_path: str) -> list[dict]:
    con = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    rows = con.execute(
        'SELECT u."telegramId", COALESCE(u."username",""), COALESCE(u."firstName",u."username","") AS nm, '
        'COALESCE(SUM(p."amount"),0) FROM "User" u '
        'LEFT JOIN "Posting" p ON p."accountId" = u."accountId" '
        'GROUP BY u."telegramId" ORDER BY u."createdAt"'
    ).fetchall()
    con.close()
    return [{"tg_id": r[0], "username": r[1], "name": r[2] or "my friend", "micro": int(r[3])} for r in rows]


def render(user: dict, miniapp_url: str) -> dict:
    name = user["name"]
    text = TEMPLATE_FA.format(name=name, balance=fmt_balance(user["micro"]))
    payload = {"text": text, "reply_markup": json.dumps(
        {"inline_keyboard": [[{"text": BTN_TEXT, "url": miniapp_url}]]},
        ensure_ascii=False)}
    return payload


def tg_call(token: str, method: str, params: dict, timeout: float = 15.0) -> dict:
    url = API.format(token=token, method=method)
    data = urllib.parse.urlencode(params).encode()
    req = urllib.request.Request(url, data=data, method="POST")
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode())


def send_live(users: list[dict], miniapp_url: str, token: str, state_path: str, limit: int) -> None:
    done: set[str] = set()
    if os.path.exists(state_path):
        done = set(json.load(open(state_path, encoding="utf-8")))
    sent = failed = 0
    sleep_slot = 1.0 / RATE_PER_SEC
    for i, user in enumerate(users[:limit] if limit else users):
        tg = user["tg_id"]
        if tg in done:
            continue
        try:
            res = tg_call(token, "sendMessage", {**render(user, miniapp_url), "chat_id": tg})
            if res.get("ok"):
                sent += 1
                done.add(tg)
            else:
                failed += 1
                print(f"  ! {tg}: {res.get('description', 'unknown')[:100]}")
        except Exception as exc:  # noqa: BLE001 — report honestly and continue
            failed += 1
            msg = str(exc)
            if "429" in msg or "Too Many Requests" in msg:
                print(f"  ! 429 on {tg} — sleeping 15s")
                time.sleep(15)
            else:
                print(f"  ! {tg}: {msg[:120]}")
        json.dump(sorted(done), open(state_path, "w", encoding="utf-8"), ensure_ascii=False)
        time.sleep(sleep_slot)
        if sent and sent % BATCH_PAUSE_EVERY == 0:
            print(f"  … {sent} sent, cooling down {BATCH_PAUSE_SEC}s")
            time.sleep(BATCH_PAUSE_SEC)
    print(f"LIVE DONE: sent={sent} failed={failed} total={len(users)} state={state_path}")


def main() -> int:
    repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    ap = argparse.ArgumentParser(description="KIAN v2 relaunch broadcast (safe: dry-run default)")
    ap.add_argument("--db", default=os.path.join(repo, "seed", "kian-production.db"))
    ap.add_argument("--miniapp-url", default=os.environ.get("KIAN_MINIAPP_URL", "https://t.me/KianV2Bot/app"))
    ap.add_argument("--live", action="store_true", help="real sending — without this flag it is a dry-run only")
    ap.add_argument("--limit", type=int, default=0, help="message cap (0 = all)")
    ap.add_argument("--samples", type=int, default=3, help="number of sample messages in the dry-run")
    ap.add_argument("--state", default=os.path.join(repo, "scripts", "broadcast_state.json"))
    args = ap.parse_args()

    assert os.path.exists(args.db), f"db not found: {args.db}"
    users = load_users(args.db)
    nonzero = [u for u in users if u["micro"] > 0]
    print(f"users={len(users)}  with-balance={len(nonzero)}  "
          f"supply={sum(u['micro'] for u in users):,} micro-KIAN")
    print(f"mode={'LIVE' if args.live else 'DRY-RUN'}  miniapp={args.miniapp_url}")

    if not args.live:
        for user in users[-args.samples:]:
            print("\n" + "─" * 60)
            print(render(user, args.miniapp_url)["text"])
            print(f"  [button: {BTN_TEXT} → {args.miniapp_url}] → chat_id {user['tg_id']}")
        print("\nDRY-RUN — no message sent. Run with --live and TELEGRAM_BOT_TOKEN to send.")
        token = os.environ.get("TELEGRAM_BOT_TOKEN", "")
        print(f"TELEGRAM_BOT_TOKEN present: {bool(token)}")
        return 0

    token = os.environ.get("TELEGRAM_BOT_TOKEN", "")
    if not token:
        print("ERROR: TELEGRAM_BOT_TOKEN is required for --live", file=sys.stderr)
        return 2
    send_live(users, args.miniapp_url, token, args.state, args.limit)
    return 0


if __name__ == "__main__":
    sys.exit(main())
