#!/usr/bin/env bash
# e2e_verify_migrated.sh — live verification of the kernel against the migrated database (Mission 2)
# One foreground run: boot → curl suite → kill → cleanup
set -u
export PATH="$HOME/.cargo/bin:$PATH"
REPO=/home/z/my-project/kian-launch/kian
BIN=/tmp/my-project/target/debug/kian-server
DB=/tmp/kian_verify.db
PORT=18099
BASE="http://127.0.0.1:${PORT}/api/v1"
ADMIN_KEY="e2e-migration-admin-key"
PASS=0; FAIL=0

say()  { printf '\n\033[36m== %s ==\033[0m\n' "$1"; }
ok()   { PASS=$((PASS+1)); printf '  \033[32mPASS\033[0m %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  \033[31mFAIL\033[0m %s\n' "$1"; }
chk()  { if [ "$2" = "$3" ]; then ok "$1 ($2)"; else bad "$1 — expected [$3] got [$2]"; fi; }
chkj() { if [ "$2" = "$3" ]; then ok "$1 ($2)"; else bad "$1 — expected [$3] got [$2] (body: $(printf '%s' "$4" | head -c 400))"; fi; }

cp "$REPO/seed/kian-production.db" "$DB"
rm -f "$DB-wal" "$DB-shm"

KIAN_DATABASE_URL="file:$DB" KIAN_HOST=127.0.0.1 KIAN_PORT=$PORT \
KIAN_DEV_MODE=1 KIAN_ADMIN_KEY="$ADMIN_KEY" KIAN_RATE_LIMIT=100000 \
  "$BIN" > /tmp/kian_e2e_server.log 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null; wait $SRV 2>/dev/null' EXIT

for i in $(seq 1 40); do
  curl -sf "http://127.0.0.1:${PORT}/api/v1/health" >/dev/null 2>&1 && break
  sleep 0.25
done

say "1. Health + boot log"
H=$(curl -sf "$BASE/health" || true)
chkj "GET /health status" "$(printf '%s' "$H" | python3 -c 'import json,sys;print(json.load(sys.stdin)["status"])' 2>/dev/null)" "ok" "$H"

say "2. Economy summary — kernel-verified ledger"
E=$(curl -sf "$BASE/economy" || true)
chkj "accountCount = 469 (466 citizen + 3 system)" "$(printf '%s' "$E" | python3 -c 'import json,sys;print(json.load(sys.stdin)["accountCount"])' 2>/dev/null)" "469" "$E"
chkj "balanced = true" "$(printf '%s' "$E" | python3 -c 'import json,sys;print(json.load(sys.stdin)["balanced"])' 2>/dev/null)" "True" "$E"
chkj "circulatingSupply = 58,700,200,000" "$(printf '%s' "$E" | python3 -c 'import json,sys;print(json.load(sys.stdin)["circulatingSupply"])' 2>/dev/null)" "58700200000" "$E"
chkj "mintedThisYear = 58,700,200,000" "$(printf '%s' "$E" | python3 -c 'import json,sys;print(json.load(sys.stdin)["mintedThisYear"])' 2>/dev/null)" "58700200000" "$E"
chkj "yearlyMintCap = 500,000,000,000" "$(printf '%s' "$E" | python3 -c 'import json,sys;print(json.load(sys.stdin)["yearlyMintCap"])' 2>/dev/null)" "500000000000" "$E"
chkj "genesisSupply = 58,700,200,000" "$(printf '%s' "$E" | python3 -c 'import json,sys;print(json.load(sys.stdin)["genesisSupply"])' 2>/dev/null)" "58700200000" "$E"
chkj "escrowLocked = 0 (all historical RELEASED)" "$(printf '%s' "$E" | python3 -c 'import json,sys;print(json.load(sys.stdin)["escrowLocked"])' 2>/dev/null)" "0" "$E"
chkj "trialBalanceSum = 0" "$(printf '%s' "$E" | python3 -c 'import json,sys;print(json.load(sys.stdin)["trialBalanceSum"])' 2>/dev/null)" "0" "$E"

say "3. Market products — 12 migrated, 5 creators, HTML artifacts"
P=$(curl -sf "$BASE/market/products" || true)
chkj "products count = 12" "$(printf '%s' "$P" | python3 -c 'import json,sys;print(len(json.load(sys.stdin)["items"]))' 2>/dev/null)" "12" "$P"
chkj "distinct sellers = 5" "$(printf '%s' "$P" | python3 -c 'import json,sys;print(len({i["sellerId"] for i in json.load(sys.stdin)["items"]}))' 2>/dev/null)" "5" "$P"
chkj "all have non-empty previewHtml" "$(printf '%s' "$P" | python3 -c 'import json,sys;print(all(len(i["previewHtml"])>500 and i["previewHtml"].startswith("<!DOCTYPE html>") for i in json.load(sys.stdin)["items"]))' 2>/dev/null)" "True" "$P"
chkj "all artifactKind = HTML_APP" "$(printf '%s' "$P" | python3 -c 'import json,sys;print(all(i["artifactKind"]=="HTML_APP" for i in json.load(sys.stdin)["items"]))' 2>/dev/null)" "True" "$P"
chkj "creators include ImXforevr/MHMD_DEVVV/BTC_USDT/nowhere66/Amingh30" "$(printf '%s' "$P" | python3 -c '
import json,sys
sellers={i["sellerId"] for i in json.load(sys.stdin)["items"]}
want={"citizen:8198598635","citizen:6365852305","citizen:607405766","citizen:6296339549","citizen:1864304786"}
print(want <= sellers)' 2>/dev/null)" "True" "$P"

say "4. Founder dev-login + exact migrated profile"
F=$(curl -sf -X POST "$BASE/auth/dev-login" -H 'content-type: application/json' -d '{"tgUserId":"8198598635","username":"ImXforevr"}' || true)
FT=$(printf '%s' "$F" | python3 -c 'import json,sys;print(json.load(sys.stdin)["token"])' 2>/dev/null)
chkj "founder accountId" "$(printf '%s' "$F" | python3 -c 'import json,sys;print(json.load(sys.stdin)["user"]["accountId"])' 2>/dev/null)" "citizen:8198598635" "$F"
chkj "founder username" "$(printf '%s' "$F" | python3 -c 'import json,sys;print(json.load(sys.stdin)["user"]["username"])' 2>/dev/null)" "ImXforevr" "$F"
M=$(curl -sf "$BASE/me" -H "authorization: Bearer $FT" || true)
chkj "founder balance = 5,000 KIAN" "$(printf '%s' "$M" | python3 -c 'import json,sys;print(json.load(sys.stdin)["balance"])' 2>/dev/null)" "5000000000" "$M"

say "5. Real returning buyer (135876863 @Mehrsinov) — historical RELEASED orders + delivery"
B=$(curl -sf -X POST "$BASE/auth/dev-login" -H 'content-type: application/json' -d '{"tgUserId":"135876863","username":"Mehrsinov"}' || true)
BT=$(printf '%s' "$B" | python3 -c 'import json,sys;print(json.load(sys.stdin)["token"])' 2>/dev/null)
MB=$(curl -sf "$BASE/me" -H "authorization: Bearer $BT" || true)
chkj "buyer migrated balance = 145 KIAN" "$(printf '%s' "$MB" | python3 -c 'import json,sys;print(json.load(sys.stdin)["balance"])' 2>/dev/null)" "145000000" "$MB"
MY=$(curl -sf "$BASE/market/my" -H "authorization: Bearer $BT" || true)
chkj "buyer sees 3 historical orders" "$(printf '%s' "$MY" | python3 -c 'import json,sys;print(len(json.load(sys.stdin)["orders"]))' 2>/dev/null)" "3" "$MY"
chkj "orders all RELEASED" "$(printf '%s' "$MY" | python3 -c 'import json,sys;print(all(o["status"]=="RELEASED" for o in json.load(sys.stdin)["orders"]))' 2>/dev/null)" "True" "$MY"
HESC=$(printf '%s' "$MY" | python3 -c 'import json,sys;print([o["id"] for o in json.load(sys.stdin)["orders"] if o["id"].startswith("escrow-mig-")][0])' 2>/dev/null)
D=$(curl -sf "$BASE/market/escrows/$HESC/delivery" -H "authorization: Bearer $BT" || true)
chkj "historical delivery fetch OK" "$(printf '%s' "$D" | python3 -c 'import json,sys;print(json.load(sys.stdin)["deliveryPayload"].startswith("<!DOCTYPE html>"))' 2>/dev/null)" "True" "$D"

say "6. NEW purchase on migrated ledger (V3Store::apply proof) + referral + HITL release"
NP=$(curl -sf -X POST "$BASE/market/purchase" -H "authorization: Bearer $BT" -H 'content-type: application/json' \
  -d '{"productId":"prod-mig-23","referrer":"6398636666"}' || true)
NESC=$(printf '%s' "$NP" | python3 -c 'import json,sys;print(json.load(sys.stdin)["escrow"]["id"])' 2>/dev/null)
chkj "new escrow FUNDED" "$(printf '%s' "$NP" | python3 -c 'import json,sys;print(json.load(sys.stdin)["escrow"]["status"])' 2>/dev/null)" "FUNDED" "$NP"
chkj "escrow referrer captured (db check)" "$(python3 -c "
import sqlite3
con = sqlite3.connect('file:/tmp/kian_verify.db?mode=ro', uri=True)
print(con.execute('SELECT referrer FROM \"Escrow\" WHERE id=?', ('$NESC',)).fetchone()[0])" 2>/dev/null)" "citizen:6398636666" "$NP"
chkj "escrow affiliateBps = 2000 (db check)" "$(python3 -c "
import sqlite3
con = sqlite3.connect('file:/tmp/kian_verify.db?mode=ro', uri=True)
print(con.execute('SELECT affiliateBps FROM \"Escrow\" WHERE id=?', ('$NESC',)).fetchone()[0])" 2>/dev/null)" "2000" "$NP"
E2=$(curl -sf "$BASE/economy" || true)
chkj "escrowLocked = 50,000,000 after buy" "$(printf '%s' "$E2" | python3 -c 'import json,sys;print(json.load(sys.stdin)["escrowLocked"])' 2>/dev/null)" "50000000" "$E2"
chkj "buyer balance = 95,000,000 (50M locked in escrow)" "$(curl -sf "$BASE/me" -H "authorization: Bearer $BT" | python3 -c 'import json,sys;print(json.load(sys.stdin)["balance"])' 2>/dev/null)" "95000000" ""
RL=$(curl -s -o /tmp/rl.json -w '%{http_code}' -X POST "$BASE/market/escrows/$NESC/release" -H "authorization: Bearer $BT" -H 'content-type: application/json' -d '{}' || true)
chk "buyer release blocked by HITL (422)" "$RL" "422"
chkj "error code = REQUIRES_HUMAN_APPROVAL" "$(python3 -c 'import json;print(json.load(open("/tmp/rl.json"))["error"]["code"])' 2>/dev/null)" "REQUIRES_HUMAN_APPROVAL" "$(cat /tmp/rl.json)"
AD=$(curl -sf -X POST "$BASE/escrows/$NESC/deliver" -H "x-kian-key: $ADMIN_KEY" -H 'content-type: application/json' -d '{"deliveryRef":"admin:e2e"}' || true)
chkj "admin deliver -> DELIVERED" "$(printf '%s' "$AD" | python3 -c 'import json,sys;print(json.load(sys.stdin)["escrow"]["status"])' 2>/dev/null)" "DELIVERED" "$AD"
AR=$(curl -sf -X POST "$BASE/escrows/$NESC/release" -H "x-kian-key: $ADMIN_KEY" -H 'content-type: application/json' -d '{"approvedByAdmin":true}' || true)
chkj "admin release -> RELEASED" "$(printf '%s' "$AR" | python3 -c 'import json,sys;print(json.load(sys.stdin)["escrow"]["status"])' 2>/dev/null)" "RELEASED" "$AR"
E3=$(curl -sf "$BASE/economy" || true)
chkj "after release escrowLocked = 0" "$(printf '%s' "$E3" | python3 -c 'import json,sys;print(json.load(sys.stdin)["escrowLocked"])' 2>/dev/null)" "0" "$E3"
chkj "trialBalanceSum still 0" "$(printf '%s' "$E3" | python3 -c 'import json,sys;print(json.load(sys.stdin)["trialBalanceSum"])' 2>/dev/null)" "0" "$E3"
chkj "treasury += 10% tax (5,000,000)" "$(printf '%s' "$E3" | python3 -c 'import json,sys;print(json.load(sys.stdin)["treasuryBalance"])' 2>/dev/null)" "5000000" "$E3"
chkj "circulating unchanged 58,700,200,000" "$(printf '%s' "$E3" | python3 -c 'import json,sys;print(json.load(sys.stdin)["circulatingSupply"])' 2>/dev/null)" "58700200000" "$E3"
MB2=$(curl -sf "$BASE/me" -H "authorization: Bearer $BT" || true)
chkj "buyer balance after release = 95,000,000 (escrow settled)" "$(printf '%s' "$MB2" | python3 -c 'import json,sys;print(json.load(sys.stdin)["balance"])' 2>/dev/null)" "95000000" "$MB2"
FB=$(curl -sf "$BASE/me" -H "authorization: Bearer $FT" || true)
chkj "founder balance = 5,035,000,000 (+35M seller net; 20% affiliate went to referrer)" "$(printf '%s' "$FB" | python3 -c 'import json,sys;print(json.load(sys.stdin)["balance"])' 2>/dev/null)" "5035000000" "$FB"
chkj "referrer ImXsi balance = 1,010,000,000 (+10M affiliate)" "$(curl -sf "$BASE/accounts/citizen:6398636666/balance" -H "x-kian-key: $ADMIN_KEY" | python3 -c 'import json,sys;print(json.load(sys.stdin)["balance"])' 2>/dev/null)" "1010000000" ""

say "7. Faucet on migrated ledger (mint cap headroom proof)"
FA=$(curl -sf -X POST "$BASE/wallet/faucet" -H "authorization: Bearer $FT" -H 'content-type: application/json' -d '{}' || true)
chkj "faucet minted 5 KIAN (post-mint balance)" "$(printf '%s' "$FA" | python3 -c 'import json,sys;print(json.load(sys.stdin)["balance"])' 2>/dev/null)" "10035000000" "$FA"
E4=$(curl -sf "$BASE/economy" || true)
chkj "mintedThisYear = 63,700,200,000 < cap 500B" "$(printf '%s' "$E4" | python3 -c 'import json,sys;print(json.load(sys.stdin)["mintedThisYear"])' 2>/dev/null)" "63700200000" "$E4"

say "8. Wave-10 public kernel surface (war-room / hud / grok / monopoly / i18n)"
W10=$(curl -sf "$BASE/war-room/state" || true)
chkj "war-room signature MAIRON-WAR-ROOM" "$(printf '%s' "$W10" | python3 -c 'import json,sys;print(json.load(sys.stdin)["signature"])' 2>/dev/null)" "MAIRON-WAR-ROOM" "$W10"
chkj "war-room canonical 24-tick scenario" "$(printf '%s' "$W10" | python3 -c 'import json,sys;print(len(json.load(sys.stdin)["scenario"]))' 2>/dev/null)" "24" "$W10"
chkj "war-room honest day-1 earned 9.55 USD" "$(printf '%s' "$W10" | python3 -c 'import json,sys;print(json.load(sys.stdin)["day1TotalEarnedMicros"])' 2>/dev/null)" "9550000" "$W10"
chk "GET /hud/presets -> 200" "$(curl -s -o /dev/null -w '%{http_code}' "$BASE/hud/presets" || true)" "200"
chk "GET /grok/ui-bundle -> 200" "$(curl -s -o /dev/null -w '%{http_code}' "$BASE/grok/ui-bundle" || true)" "200"
chk "GET /monopoly/charter-and-base -> 200" "$(curl -s -o /dev/null -w '%{http_code}' "$BASE/monopoly/charter-and-base" || true)" "200"
chk "GET /i18n/ecosystem-matrix -> 200" "$(curl -s -o /dev/null -w '%{http_code}' "$BASE/i18n/ecosystem-matrix" || true)" "200"
chk "GET /war-room (HTML dashboard) -> 200" "$(curl -s -o /dev/null -w '%{http_code}' "${BASE%/api/v1}/war-room" || true)" "200"
chk "GET /neural-hud (HTML dashboard) -> 200" "$(curl -s -o /dev/null -w '%{http_code}' "${BASE%/api/v1}/neural-hud" || true)" "200"

say "RESULT"
printf '  PASS: %d  FAIL: %d\n' "$PASS" "$FAIL"
if [ "$FAIL" -eq 0 ]; then printf '\033[32mALL E2E CHECKS PASSED — migrated DB is kernel-verified\033[0m\n'; else printf '\033[31mE2E FAILURES — see above\033[0m\n'; fi
kill $SRV 2>/dev/null; wait $SRV 2>/dev/null
rm -f "$DB" "$DB-wal" "$DB-shm" /tmp/rl.json
exit $FAIL
