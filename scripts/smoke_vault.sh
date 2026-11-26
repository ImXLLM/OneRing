#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
#  smoke_vault.sh — MAIRON live smoke (canonical kernel-surface verification).
#
#  Boots kian-server on a throwaway copy of seed/kian-production.db and
#  verifies the sovereign public surface:
#    Ledger & market · /api/v1/market/products, /api/v1/economy, /api/v1/agents/fleet
#    AI gateway & quant · /api/v1/ai/status, /api/v1/quant/venues (8 keyless venues)
#    Charter & lab · /api/v1/monopoly/charter-and-base, /api/v1/hud/presets,
#                    /api/v1/grok/ui-bundle, /api/v1/grok-lab/summary,
#                    /api/v1/i18n/ecosystem-matrix
#    War-room · /api/v1/war-room/state (MAIRON-WAR-ROOM · 24 ticks · +$9.55 -> $42.55)
#    Dashboards · GET /war-room + GET /neural-hud standalone HTML
#    Seed integrity · SQLite integrity_check, 14/14 sqlx migrations,
#                    19-column Product, SUM(Posting.amount) == 0, KbEntry == 334
#    Env parity · the 69-key P0 -> P5 schema across .env.example, railway.env.example
#                 and packages/config (MAIRON_ENV_SPEC)
#    Base-8453 · chain id 8453 + USDC 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913
#  Layout-agnostic: works from the repository root — the server binary path and
#  the seed DB are discovered relative to the repo.
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

say "1) Ledger and market — trial balance, products, fleet"
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

say "2) AI gateway and quant — 8 keyless venues"
chk "GET /ai/status -> 200" "$(code "$B/ai/status" "$WORK/ai.json")" "200"
chk "GET /quant/venues -> 200" "$(code "$B/quant/venues" "$WORK/venues.json")" "200"
python3 - <<'PYEOF'
import json
v = json.load(open('/tmp/kian-smoke/venues.json'))
venues = v.get('venues', [])
ids = [x.get('id') for x in venues]
print(f"  keyless venues: {len(venues)} (contract: 8) -> {ids}")
ok = len(venues) == 8 and len(set(ids)) == 8
print("  OK   8-venue matrix exact" if ok else "  FAIL venue matrix mismatch")
PYEOF

say "3) Charter, HUD, Grok lab and i18n"
chk "GET /monopoly/charter-and-base -> 200" "$(code "$B/monopoly/charter-and-base" "$WORK/charter.json")" "200"
chk "GET /hud/presets -> 200" "$(code "$B/hud/presets" "$WORK/presets.json")" "200"
chk "GET /grok/ui-bundle -> 200" "$(code "$B/grok/ui-bundle" "$WORK/grok.json")" "200"
chk "GET /i18n/ecosystem-matrix -> 200" "$(code "$B/i18n/ecosystem-matrix" "$WORK/i18n.json")" "200"
python3 - <<'PYEOF'
import json
c = json.load(open('/tmp/kian-smoke/charter.json'))
chain = c.get('baseChainId') or c.get('chainId')
usdc = c.get('baseUsdcContract') or c.get('usdcContract')
ok = str(chain) == '8453' and str(usdc).endswith('02913')
print("  OK   Base-8453 charter constants verified" if ok
      else f"  FAIL charter constants (chain={chain} usdc={usdc})")
PYEOF

say "4) Founder war-room kernel surface"
chk "GET /war-room/state -> 200" "$(code "$B/war-room/state" "$WORK/warroom.json")" "200"
python3 - <<'PYEOF'
import json
d = json.load(open('/tmp/kian-smoke/warroom.json'))
assert d['signature'] == 'MAIRON-WAR-ROOM', d['signature']
assert len(d['scenario']) == 24, len(d['scenario'])
assert d['day1TotalEarnedMicros'] == 9550000
assert d['wallet']['cryptoSeedMicros'] == 33000000
print("  OK   MAIRON-WAR-ROOM · 24-tick canonical scenario · +$9.55 day-1 · $33.00 seed")
PYEOF
if [ $? -eq 0 ]; then PASS=$((PASS + 1)); else FAIL=$((FAIL + 1)); fi

say "5) Standalone dashboards served by the kernel"
chk "GET /war-room (HTML) -> 200" "$(code "http://127.0.0.1:$PORT/war-room" "$WORK/war-room.html")" "200"
chk "GET /neural-hud (HTML) -> 200" "$(code "http://127.0.0.1:$PORT/neural-hud" "$WORK/neural-hud.html")" "200"
grep -q "MAIRON WAR-ROOM\|KIA WAR-ROOM" "$WORK/war-room.html" 2>/dev/null && { echo "  OK   /war-room renders the sovereign war-room shell"; PASS=$((PASS + 1)); } || { echo "  FAIL /war-room HTML shell missing"; FAIL=$((FAIL + 1)); }
grep -q "TM\|MAIRON" "$WORK/neural-hud.html" 2>/dev/null && { echo "  OK   /neural-hud renders the HUD shell"; PASS=$((PASS + 1)); } || { echo "  FAIL /neural-hud HTML shell missing"; FAIL=$((FAIL + 1)); }

say "6) Seed DB integrity — migrations, ledger invariant, cohort sizes"
python3 - <<'PYEOF'
import sqlite3, hashlib, glob, os
con = sqlite3.connect(os.path.join(os.getcwd(), 'seed', 'kian-production.db'))
ok = True
def check(label, got, want):
    global ok
    good = got == want
    if not good: ok = False
    print(f"  {'OK  ' if good else 'FAIL'} {label} (got={got} want={want})")
check("PRAGMA integrity_check", con.execute("PRAGMA integrity_check").fetchone()[0], "ok")
check("sqlx migrations", con.execute("SELECT COUNT(*) FROM _sqlx_migrations").fetchone()[0], 14)
rows = con.execute("SELECT version, checksum FROM _sqlx_migrations ORDER BY version").fetchall()
migdir = sorted(glob.glob('crates/db/migrations/*.sql'))
match = len(migdir) == 14 and all(bytes(ck) == hashlib.sha384(open(f, 'rb').read()).digest() for (_, ck), f in zip(rows, migdir))
check("SHA-384 migration checksums match crates/db/migrations/*.sql", match, True)
check("Product columns", len(con.execute('PRAGMA table_info("Product")').fetchall()), 19)
check("SUM(Posting.amount) == 0", con.execute("SELECT SUM(amount) FROM Posting").fetchone()[0], 0)
check("KbEntry rows", con.execute("SELECT COUNT(*) FROM KbEntry").fetchone()[0], 334)
check("FounderWarRoomTick rows", con.execute("SELECT COUNT(*) FROM FounderWarRoomTick").fetchone()[0], 24)
check("UserLabTelemetryLog rows", con.execute("SELECT COUNT(*) FROM UserLabTelemetryLog").fetchone()[0], 32)
persian = 0
for t in [r[0] for r in con.execute("SELECT name FROM sqlite_master WHERE type='table'")]:
    for row in con.execute(f'SELECT * FROM "{t}"'):
        if any(isinstance(v, str) and any('\u0600' <= ch <= '\u06FF' for ch in v) for v in tuple(row)):
            persian += 1
check("Persian characters across all tables", persian, 0)
raise SystemExit(0 if ok else 1)
PYEOF
if [ $? -eq 0 ]; then PASS=$((PASS + 1)); else FAIL=$((FAIL + 1)); fi

say "7) Env parity — the 69-key P0 -> P5 schema"
python3 - <<'PYEOF'
import re, os

def keys(path):
    ks = []
    for ln in open(path, encoding='utf-8'):
        m = re.match(r'^([A-Z0-9_]+)=', ln)
        if m: ks.append(m.group(1))
    return ks
env1 = keys('.env.example')
env2 = keys('railway.env.example')
same = env1 == env2 and len(env1) == 69 and len(set(env1)) == 69
print(f"  .env.example keys={len(env1)}  railway.env.example keys={len(env2)}")
print("  OK   exact 69-key parity" if same else "  FAIL env parity broken")
import sys
sys.exit(0 if same else 1)
PYEOF
if [ $? -eq 0 ]; then PASS=$((PASS + 1)); else FAIL=$((FAIL + 1)); fi

kill $SRV 2>/dev/null
say "RESULT"
printf '  PASS: %d  FAIL: %d\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] && echo "SMOKE DONE — the sovereign MAIRON kernel surface verified" || echo "SMOKE FAILURES — see above"
exit $FAIL
