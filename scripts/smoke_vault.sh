#!/bin/bash
# smoke_vault.sh — بوت سرور روی کپی seed و راستی‌آزمایی زندهٔ ۴ اندپوینت مأموریت
set -u
export PATH="$HOME/.cargo/bin:$PATH"
cd /home/z/my-project/kian-launch/kian

pkill -f "target/debug/kian-server" 2>/dev/null || true
sleep 1

set -a
source .env.local 2>/dev/null
set +a
export KIAN_DATABASE_URL=/tmp/smoke/kian.db KIAN_HOST=127.0.0.1 KIAN_PORT=18088

/tmp/my-project/target/debug/kian-server > /tmp/smoke/server.log 2>&1 &
SRV=$!
for i in $(seq 1 30); do
  sleep 1
  if curl -sf "http://127.0.0.1:18088/api/v1/health" > /dev/null 2>&1; then break; fi
done
B="http://127.0.0.1:18088/api/v1"

echo "== 1) GET /market/products (فعال + ترینیتی + سرعت) =="
curl -s -w "\nHTTP %{http_code} in %{time_total}s\n" "$B/market/products?limit=100" -o /tmp/smoke/products.json
python3 - << 'PYEOF'
import json
d = json.load(open('/tmp/smoke/products.json'))
items = d.get('products') if isinstance(d, dict) else d
if isinstance(items, dict): items = items.get('items', [])
total = d.get('total') if isinstance(d, dict) else None
print('total field:', total, '| returned:', len(items))
trinity = [p for p in items if str(p['id']).startswith('prod:trinity-')]
print('trinity flagships:', len(trinity))
for p in trinity:
    print('  ', p['id'], '|', p['title'][:46], '|', p['priceDin'], '| bps', p['affiliateBps'], '| kind', p.get('artifactKind'))
mx = max((len(p.get('previewHtml') or '') for p in items), default=0)
print('max previewHtml bytes in bulk list:', mx)
assert mx < 50_000, 'پیش‌نمایش سنگین در لیست!'
PYEOF

echo "== 2) GET /economy (تراز آزمایشی) =="
curl -s -w "\nHTTP %{http_code} in %{time_total}s\n" "$B/economy" -o /tmp/smoke/economy.json
python3 -m json.tool /tmp/smoke/economy.json | head -14

echo "== 3) GET /agents/fleet (۶ فعال از ۷) =="
curl -s -w "\nHTTP %{http_code} in %{time_total}s\n" "$B/agents/fleet" -o /tmp/smoke/fleet.json
python3 - << 'PYEOF'
import json
d = json.load(open('/tmp/smoke/fleet.json'))
agents = d['agents']
active = [a for a in agents if a['status'] == 'active']
print('agents:', len(agents), '| active:', len(active))
for a in agents:
    print('  ', a['role'], '→', a['status'])
PYEOF

echo "== 4) GET /ai/status (poolSize = 7) =="
curl -s -w "\nHTTP %{http_code} in %{time_total}s\n" "$B/ai/status" -o /tmp/smoke/ai.json
python3 -m json.tool /tmp/smoke/ai.json | head -40

echo "== 5) استاتیک‌ها: /senpai + /products/trinity =="
for u in "/senpai/app.html" "/senpai/vendor/three.min.js" "/senpai/agent-graph.js" "/products/trinity/sc.html" "/products/trinity/fable-bazaar.html" "/products/trinity/ocean-test-lab.html" "/products/trinity/dropagentx/showcase3d.html"; do
  code=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:18088$u")
  echo "  $u → $code"
done

kill $SRV 2>/dev/null
echo "SMOKE DONE"
