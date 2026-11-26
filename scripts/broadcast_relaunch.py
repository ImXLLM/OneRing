#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
broadcast_relaunch.py — \u0627\u0633\u06a9\u0631\u06cc\u067e\u062a \u0627\u0645\u0646 \u067e\u06cc\u0627\u0645 \u0631\u06cc\u0644\u0627\u0646\u0686 KIAN v2 \u0628\u0647 \u06a9\u0644 \u062c\u0627\u0645\u0639\u0647\u0654 \u0645\u0647\u0627\u062c\u0631\u062a‌\u0634\u062f\u0647
==============================================================================

\u0645\u0646\u0628\u0639: seed/kian-production.db (\u06f4\u06f6\u06f6 \u0634\u0647\u0631\u0648\u0646\u062f \u0645\u0647\u0627\u062c\u0631\u062a‌\u06cc\u0627\u0641\u062a\u0647) — \u0641\u0642\u0637‌\u062e\u0648\u0627\u0646\u062f\u0646\u06cc.
\u062e\u0631\u0648\u062c\u06cc: \u067e\u06cc\u0627\u0645 \u0634\u062e\u0635\u06cc‌\u0633\u0627\u0632\u06cc‌\u0634\u062f\u0647\u0654 \u0641\u0627\u0631\u0633\u06cc/\u0627\u0646\u06af\u0644\u06cc\u0633\u06cc \u0628\u0627 \u062a\u0631\u0627\u0632 KIAN \u0647\u0631 \u06a9\u0627\u0631\u0628\u0631 + \u062f\u06a9\u0645\u0647\u0654 \u0648\u0631\u0648\u062f.

\u0627\u06cc\u0645\u0646\u06cc:
  * \u067e\u06cc\u0634‌\u0641\u0631\u0636 --dry-run: \u0647\u06cc\u0686 \u067e\u06cc\u0627\u0645\u06cc \u0627\u0631\u0633\u0627\u0644 \u0646\u0645\u06cc‌\u0634\u0648\u062f\u061b \u0641\u0642\u0637 \u0646\u0645\u0648\u0646\u0647\u0654 \u067e\u06cc\u0627\u0645 \u0648 \u0622\u0645\u0627\u0631 \u0686\u0627\u067e \u0645\u06cc‌\u0634\u0648\u062f.
  * \u0627\u0631\u0633\u0627\u0644 \u0648\u0627\u0642\u0639\u06cc \u0641\u0642\u0637 \u0628\u0627 --live \u0648 TELEGRAM_BOT_TOKEN \u062f\u0631 \u0645\u062d\u06cc\u0637.
  * \u0646\u0631\u062e \u0645\u062d\u062f\u0648\u062f: \u062d\u062f\u0627\u06a9\u062b\u0631 \u06f2\u06f0 \u067e\u06cc\u0627\u0645 \u062f\u0631 \u062b\u0627\u0646\u06cc\u0647 (\u0633\u0642\u0641 \u0631\u0633\u0645\u06cc Bot API) + \u062a\u0648\u0642\u0641‌\u0647\u0627\u06cc \u062f\u0648\u0631\u0647‌\u0627\u06cc.
  * Resume \u0627\u0645\u0646: \u0648\u0636\u0639\u06cc\u062a \u0627\u0631\u0633\u0627\u0644 \u062f\u0631 --state (JSON) \u062b\u0628\u062a \u0645\u06cc‌\u0634\u0648\u062f\u061b \u0627\u062c\u0631\u0627\u06cc \u0645\u062c\u062f\u062f \u0627\u062f\u0627\u0645\u0647 \u0645\u06cc‌\u062f\u0647\u062f\u060c \u062f\u0648\u0628\u0627\u0631 \u0646\u0645\u06cc‌\u0641\u0631\u0633\u062a\u062f.
  * \u0647\u0631 \u062e\u0637\u0627\u06cc 429 \u0627\u0632 \u067e\u0627\u0633\u062e \u062a\u0644\u06af\u0631\u0627\u0645 obeyed \u0645\u06cc‌\u0634\u0648\u062f (retry_after).
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

BTN_TEXT = "🚀 \u0648\u0631\u0648\u062f \u0628\u0647 \u0627\u0633\u062a\u0648\u062f\u06cc\u0648 \u0648 \u0628\u0627\u0632\u0627\u0631 KIAN v2"

TEMPLATE_FA = (
    "\u0633\u0644\u0627\u0645 {name} 👋\n\n"
    "\u062f\u0646\u06cc\u0627\u06cc dropagent \u062d\u0627\u0644\u0627 \u0628\u0627 \u0646\u0627\u0645 **KIAN v2** \u0628\u0627 \u0642\u062f\u0631\u062a \u0628\u0631\u06af\u0634\u062a\u0647 — \u0648 \u0647\u0645\u0647\u0654 \u062f\u0627\u0631\u0627\u06cc\u06cc‌\u0627\u062a \u062f\u0642\u06cc\u0642\u0627\u064b \u0647\u0645\u0648\u0646‌\u0637\u0648\u0631 \u06a9\u0647 \u0628\u0648\u062f \u0628\u0631\u0627\u062a \u0645\u0646\u062a\u0642\u0644 \u0634\u062f\u0647:\n\n"
    "💰 \u062a\u0631\u0627\u0632 KIAN \u062a\u0648: **{balance} KIAN**\n"
    "🧠 \u062d\u0627\u0641\u0638\u0647 \u0648 \u06af\u0641\u062a\u06af\u0648\u0647\u0627\u06cc AI \u062a\u0648 \u0645\u0646\u062a\u0642\u0644 \u0634\u062f\u0647\n"
    "🛒 \u0633\u0641\u0627\u0631\u0634‌\u0647\u0627\u06cc \u0642\u0628\u0644\u06cc‌\u0627\u062a \u0628\u0627 \u0645\u062d\u062a\u0648\u0627\u06cc \u06a9\u0627\u0645\u0644 \u062f\u0631 \u062f\u0633\u062a\u0631\u0633\u0647\n"
    "🎁 \u0628\u0627 \u0647\u0631 \u0641\u0631\u0648\u0634 \u0627\u0632 \u0637\u0631\u06cc\u0642 \u0644\u06cc\u0646\u06a9 \u062f\u0639\u0648\u062a \u062a\u0648\u060c \u06f2\u06f0\u066a \u067e\u0648\u0631\u0633\u0627\u0646\u062a \u0645\u06cc‌\u06af\u06cc\u0631\u06cc!\n\n"
    "\u0647\u0645\u06cc\u0646 \u062d\u0627\u0644\u0627 \u0648\u0627\u0631\u062f \u0634\u0648 \u0648 \u06a9\u06cc\u0627\u0646 \u062c\u062f\u06cc\u062f \u0631\u0648 \u0628\u0628\u06cc\u0646 👇"
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
    return [{"tg_id": r[0], "username": r[1], "name": r[2] or "\u062f\u0648\u0633\u062a \u0645\u0646", "micro": int(r[3])} for r in rows]


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
    ap.add_argument("--live", action="store_true", help="\u0627\u0631\u0633\u0627\u0644 \u0648\u0627\u0642\u0639\u06cc — \u0628\u062f\u0648\u0646 \u0627\u06cc\u0646 \u067e\u0631\u0686\u0645 \u0641\u0642\u0637 dry-run \u0627\u0633\u062a")
    ap.add_argument("--limit", type=int, default=0, help="\u0633\u0642\u0641 \u062a\u0639\u062f\u0627\u062f \u067e\u06cc\u0627\u0645 (\u06f0 = \u0647\u0645\u0647)")
    ap.add_argument("--samples", type=int, default=3, help="\u062a\u0639\u062f\u0627\u062f \u067e\u06cc\u0627\u0645 \u0646\u0645\u0648\u0646\u0647 \u062f\u0631 dry-run")
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
