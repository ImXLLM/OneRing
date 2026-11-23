#!/usr/bin/env bash
# Live E2E: kian-server ↔ 9router — proves the complete AI fallback chain.
# Boot the kernel against a COPY of the migrated production DB, wire both AI
# slots to the user's 9router, then run: dev-login → builder session → chat
# (primary 404 → secondary 404 → deterministic local HTML synthesizer) → publish.
set -euo pipefail

REPO=/home/z/my-project/kian-launch/kian
BIN=/tmp/my-project/target/debug/kian-server
E2E=/tmp/kian-e2e
PORT=18088
ROOT="http://127.0.0.1:$PORT"
API="$ROOT/api/v1"

set -a
# shellcheck disable=SC1091
source "$REPO/.env.local"
set +a
export KIAN_DATABASE_URL="$E2E/live.db"
export KIAN_HOST=127.0.0.1
export KIAN_PORT="$PORT"
export KIAN_ADMIN_KEY=dev-admin-key
export KIAN_AI_DAILY_TOKENS=50000
export KIAN_BACKUP_DIR="$E2E/backups"

mkdir -p "$E2E"
cp "$REPO/seed/kian-production.db" "$E2E/live.db"

"$BIN" > "$E2E/server.log" 2>&1 &
SPID=$!
trap 'kill "$SPID" 2>/dev/null || true' EXIT

READY=0
for _ in $(seq 1 60); do
  if curl -sf "$API/health" -o "$E2E/health.json"; then READY=1; break; fi
  sleep 0.5
done
if [ "$READY" != "1" ]; then
  echo "SERVER FAILED TO BOOT — last log lines:"; tail -20 "$E2E/server.log"; exit 1
fi

python3 - "$API" "$E2E" <<'PY'
import json, sys, time
import httpx

api, e2e = sys.argv[1], sys.argv[2]
out = {}

with httpx.Client(timeout=60) as c:
    h = c.get(f"{api.replace('/api/v1','')}/health")
    out["health"] = h.status_code

    s = c.get(f"{api}/ai/status").json()
    out["ai_slots"] = [
        {"slot": x["slot"], "name": x["name"], "model": x["model"], "enabled": x["enabled"]}
        for x in s.get("slots", s if isinstance(s, list) else [])
    ]

    r = c.post(f"{api}/auth/dev-login", json={"tgUserId": "8198598635"})
    r.raise_for_status()
    token = r.json()["token"]
    hdr = {"Authorization": f"Bearer {token}"}

    me = c.get(f"{api}/me", headers=hdr).json()
    out["founder"] = {
        "username": me.get("username"),
        "firstName": me.get("firstName"),
        "accountId": me.get("accountId") or me.get("account_id"),
        "balance": me.get("balance") or me.get("balanceMicro"),
    }

    sess = c.post(f"{api}/builder/sessions", json={}, headers=hdr).json()
    sid = sess["id"]
    out["session_id"] = sid

    t0 = time.perf_counter()
    chat = c.post(
        f"{api}/builder/sessions/{sid}/chat",
        headers=hdr,
        json={"message": "می‌خوام برای بازار کیان یک ماشین‌حساب ریسک ترید بسازم: موجودی، درصد ریسک، حد ضرر و حجم پوزیشن", "forceGenerate": True},
    )
    chat.raise_for_status()
    out["chat_ms"] = int((time.perf_counter() - t0) * 1000)
    cj = chat.json()
    html = (cj.get("session") or {}).get("generatedHtml") or ""
    out["chat"] = {
        "usedFallback": cj.get("usedFallback"),
        "provider": cj.get("provider"),
        "totalTokens": cj.get("totalTokens"),
        "stage": (cj.get("session") or {}).get("stage"),
        "title": (cj.get("session") or {}).get("title"),
        "replyHead": (cj.get("reply") or "")[:110],
        "html_bytes": len(html.encode()),
        "html_doctype": "<!doctype html" in html.lower(),
        "html_rtl": 'dir="rtl"' in html.lower(),
        "html_close": "</html>" in html.lower(),
    }

    pub = c.post(f"{api}/builder/sessions/{sid}/publish", json={}, headers=hdr)
    pub.raise_for_status()
    pj = pub.json()
    out["published"] = {
        "productId": pj["product"]["id"],
        "title": pj["product"]["title"],
        "price": pj["product"].get("price") or pj["product"].get("priceDin"),
        "active": pj["product"].get("active"),
        "shareStartParam": pj.get("shareStartParam", "")[:24],
    }

assert out["chat"]["usedFallback"] is True, "both 9router slots must fail → local synthesizer"
assert out["chat"]["html_doctype"] and out["chat"]["html_close"], "fallback HTML must be a full document"
import pathlib
pathlib.Path(e2e, "e2e_result.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))
print(json.dumps(out, ensure_ascii=False, indent=1))
print("\nE2E VERDICT: PASS — primary+secondary 9router slots failed → deterministic local HTML synthesizer served a full artifact, published to market")
PY

echo
echo "--- live 9router slot failures observed in server.log ---"
grep -c "AI slot failed" "$E2E/server.log" || true
grep "AI slot failed" "$E2E/server.log" | head -3
