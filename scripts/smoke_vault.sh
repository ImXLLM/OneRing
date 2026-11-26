#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
#  smoke_vault.sh — Wave-10 live smoke (canonical kernel-surface verification).
#
#  Boots kian-server on a throwaway copy of seed/kian-production.db and
#  verifies the Waves 1–10 public surface:
#    Wave 1-3  · /api/v1/market/products, /api/v1/economy, /api/v1/agents/fleet
#    Wave 4-6  · /api/v1/ai/status, quant venues
#    Wave 7-9  · /api/v1/monopoly/charter-and-base, /api/v1/hud/presets,
#                /api/v1/grok/ui-bundle, /api/v1/i18n/ecosystem-matrix
#    Sovereign OS · /api/v1/war-room/state (MAIRON-WAR-ROOM · 24 ticks · $9.55)
#    Wave 10   · GET /war-room + GET /neural-hud standalone HTML dashboards
#  Layout-agnostic: works from the repository root (Wave-10 layout) — the
#  server binary path and the seed DB are discovered relative to the repo.
# ═══════════════════════════════════════════════════════════════════════════
set -u
export PATH="$HOME/.cargo/bin:$PATH"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

WORK="${KIAN_SMOKE_DIR:-/tmp/kian-smoke}"
PORT="${KIAN_SMOKE_PORT:-18088}"
mkdir -p "$WORK"
pkill -f "target/debug/kian-server" 2>/dev/null || true
sleep 1

set -a
source .env.local 2>/dev/null
set +a
rm -f "$WORK/kian.db" "$WORK/kian.db-wal" "$WORK/kian.db-shm"
export KIAN_DATABASE_URL="$WORK/kian.db" KIAN_HOST=127.0.0.1 KIAN_PORT="$PORT"

BIN="${KIAN_SMOKE_BIN:-$ROOT/target/debug/kian-server}"
if [ ! -x "$BIN" ]; then
    echo "[smoke] building kian-server (first run)…"
    cargo build -p kian-server --quiet || exit 1
fi
"$BIN" > "$WORK/server.log" 2>&1 &
SRV=$!
for _ in $(seq 1 30); do
    sleep 1
    if curl -sf "http://127.0.0.1:$PORT/api/v1/health" > /dev/null 2>&1; then break; fi
done
B="http://127.0.0.1:$PORT/api/v1"

say()  { echo "== $1 =="; }
code() { curl -s -o "$2" -w "%{http_code}" "$1"; }
PASS=0; FAIL=0
chk() { # chk <label> <got> <expected>
    if [ "$2" = "$3" ]; then echo "  OK   $1"; PASS=$((PASS + 1));
    else echo "  FAIL $1 (got=$2 want=$3)"; FAIL=$((FAIL + 1)); fi
}

say "1) Waves 1-3 — market / economy / fleet"
chk "GET /market/products -> 200" "$(code "$B/market/products?limit=100" "$WORK/products.json")" "200"
chk "GET /economy -> 200 (trial balance)" "$(code "$B/economy" "$WORK/economy.json")" "200"
chk "GET /agents/fleet -> 200" "$(code "$B/agents/fleet" "$WORK/fleet.json")" "200"
python3 - <<'PYEOF'
import json
d = json.load(open('/tmp/kian-smoke/products.json'))
items = d.get('products') if isinstance(d, dict) else d
if isinstance(items, dict): items = items.get('items', [])
print(f"  products listed: {len(items)} (seed: 20)")
e = json.load(open('/tmp/kian-smoke/economy.json'))
print(f"  trialBalanceSum: {e.get('trialBalanceSum')} (invariant: 0)")
PYEOF

say "2) Waves 4-6 — AI gateway / quant"
chk "GET /ai/status -> 200" "$(code "$B/ai/status" "$WORK/ai.json")" "200"
chk "GET /quant/venues -> 200" "$(code "$B/quant/venues" "$WORK/venues.json")" "200"

say "3) Waves 7-9 — monopoly / HUD / grok / i18n"
chk "GET /monopoly/charter-and-base -> 200" "$(code "$B/monopoly/charter-and-base" "$WORK/charter.json")" "200"
chk "GET /hud/presets -> 200" "$(code "$B/hud/presets" "$WORK/presets.json")" "200"
chk "GET /grok/ui-bundle -> 200" "$(code "$B/grok/ui-bundle" "$WORK/grok.json")" "200"
chk "GET /i18n/ecosystem-matrix -> 200" "$(code "$B/i18n/ecosystem-matrix" "$WORK/i18n.json")" "200"

say "4) Wave 10 — founder war-room kernel surface"
chk "GET /war-room/state -> 200" "$(code "$B/war-room/state" "$WORK/warroom.json")" "200"
python3 - <<'PYEOF'
import json
d = json.load(open('/tmp/kian-smoke/warroom.json'))
assert d['signature'] == 'MAIRON-WAR-ROOM', d['signature']
assert len(d['scenario']) == 24, len(d['scenario'])
assert d['day1TotalEarnedMicros'] == 9550000
assert d['wallet']['cryptoSeedMicros'] == 33000000
print("  OK   MAIRON-WAR-ROOM · 24-tick canonical scenario · $9.55 day-1 · $33.00 seed")
PYEOF
if [ $? -eq 0 ]; then PASS=$((PASS + 1)); else FAIL=$((FAIL + 1)); fi

say "5) Wave 10 — standalone dashboards served by the kernel"
chk "GET /war-room (HTML) -> 200" "$(code "http://127.0.0.1:$PORT/war-room" "$WORK/war-room.html")" "200"
chk "GET /neural-hud (HTML) -> 200" "$(code "http://127.0.0.1:$PORT/neural-hud" "$WORK/neural-hud.html")" "200"
grep -q "KIA WAR-ROOM" "$WORK/war-room.html" 2>/dev/null && { echo "  OK   /war-room renders the KIA WAR-ROOM shell"; PASS=$((PASS + 1)); } || { echo "  FAIL /war-room HTML shell missing"; FAIL=$((FAIL + 1)); }
grep -q "TM" "$WORK/neural-hud.html" 2>/dev/null && { echo "  OK   /neural-hud renders the HUD shell"; PASS=$((PASS + 1)); } || { echo "  FAIL /neural-hud HTML shell missing"; FAIL=$((FAIL + 1)); }

kill $SRV 2>/dev/null
say "RESULT"
printf '  PASS: %d  FAIL: %d\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] && echo "SMOKE DONE — Waves 1-10 kernel surface verified" || echo "SMOKE FAILURES — see above"
exit $FAIL
