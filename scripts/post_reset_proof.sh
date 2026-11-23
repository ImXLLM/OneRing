#!/usr/bin/env bash
# Post-reset live proof for the OpenRouter 3-slot KIAN gateway.
#
# 1. Polls GET /auth/key (costs zero model requests) until the free-model
#    daily counter resets (resets at 00:00 UTC — used 56/50 today).
# 2. Rebuilds the server binary if /tmp was recycled between sessions.
# 3. Runs scripts/e2e_openrouter_live.sh (preflight → boot kernel on a copy
#    of seed/kian-production.db → dev-login founder → builder chat
#    forceGenerate:true → assert usedFallback:false + provider=primary +
#    Persian RTL <!DOCTYPE html> → publish), retrying up to 3 times.
# 4. Fills section ۷ of OPENROUTER_BENCHMARK_REPORT_FA.md with real numbers.
# 5. Refreshes download/kian-v2-railway-migrated.zip from the committed tree
#    (git archive — .env.local is gitignored, so no secret ever ships).
#
# Secrets come only from gitignored .env.local and are never printed.
set -uo pipefail

REPO=/home/z/my-project/kian-launch/kian
OUT=/home/z/my-project/download
LOG=/tmp/post_reset_e2e.log
E2E_DIR=/tmp/kian-e2e-or

KEY=$(grep '^KIAN_OPENROUTER_API_KEY=' "$REPO/.env.local" | cut -d= -f2)
BASE=$(grep '^KIAN_OPENROUTER_BASE_URL=' "$REPO/.env.local" | cut -d= -f2)
BIN=/tmp/my-project/target/debug/kian-server

export PATH="$HOME/.cargo/bin:$PATH" CARGO_TARGET_DIR=/tmp/my-project/target CARGO_BUILD_JOBS=2

# ── 0) binary freshness (sandbox /tmp may be recycled between sessions) ───
if [ ! -x "$BIN" ]; then
  echo "rebuilding kian-server (target dir was recycled)..."
  (cd "$REPO" && cargo build -p kian-server) || { echo "BUILD FAILED"; exit 1; }
fi

# ── 1) wait for the free-model daily counter reset ────────────────────────
echo "waiting for the OpenRouter free-model daily counter reset (00:00 UTC)..."
REMAIN=-1
for i in $(seq 1 90); do
  REMAIN=$(python3 - "$KEY" "$BASE" <<'PY'
import sys
import httpx
key, base = sys.argv[1], sys.argv[2]
try:
    r = httpx.get(f"{base.rstrip('/')}/auth/key", headers={"Authorization": f"Bearer {key}"}, timeout=20)
    d = r.json()["data"].get("free_model_daily_requests") or {}
    print(d.get("remaining", 0))
except Exception as e:
    print(0)
PY
)
  echo "  poll $i  $(date -u '+%H:%M:%S') UTC  remaining=$REMAIN"
  if [ "${REMAIN:-0}" -gt 0 ] 2>/dev/null; then
    echo "QUOTA RESET DETECTED — running the live E2E"
    break
  fi
  sleep 120
done
if [ "${REMAIN:-0}" -le 0 ] 2>/dev/null; then
  echo "GAVE UP: quota never reset (90 polls / 3h). See log: $LOG"; exit 2
fi

# ── 2) live E2E — up to 3 attempts (each: preflight ping + builder turn) ──
PASS=0
for attempt in 1 2 3; do
  echo "=== E2E attempt $attempt ==="
  if bash "$REPO/scripts/e2e_openrouter_live.sh" 2>&1 | tee "$LOG"; then
    PASS=1
    break
  fi
  echo "attempt $attempt failed — cooling down 60s before retry"
  sleep 60
done
if [ "$PASS" != "1" ]; then
  echo "E2E FAILED after 3 attempts — last 40 lines:"; tail -40 "$LOG"; exit 3
fi

# ── 3) fill report section ۷ with the real numbers ────────────────────────
python3 - <<'PY'
import json, pathlib, datetime
res = json.load(open("/tmp/kian-e2e-or/e2e_result.json", encoding="utf-8"))
c = res["chat"]
now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
slots = "، ".join(f"{s['name']}=`{s['model']}`" for s in res["ai_slots"] if s["enabled"])
fa_bool = lambda b: "✅ بله" if b else "❌ خیر"
section = f"""اجرای زنده در {now} — اسکریپت `scripts/e2e_openrouter_live.sh` روی کپی پایگاه‌دادهٔ مهاجرت‌شده (`seed/kian-production.db`):

| سنجه | مقدار اندازه‌گیری‌شده |
|---|---|
| اسلات‌های فعال | {slots} |
| `usedFallback` | **{fa_bool(not c['usedFallback'])} — `{str(c['usedFallback']).lower()}`** (یعنی پاسخ واقعی LLM، نه سنتز محلی) |
| `provider` | **`{c['provider']}`** (اسلات ۱ برنده شد) |
| کل نوبت سازنده | {res['chat_ms']:,} ms |
| توکن مصرفی نوبت | {c['totalTokens']:,} |
| مرحلهٔ سشن | `{c['stage']}` |
| عنوان محصول | {c['title']} |
| آرتیفکت HTML | {c['html_bytes']:,} بایت — doctype: {fa_bool(c['html_doctype'])}، RTL فارسی: {fa_bool(c['html_rtl'])}، بستهٔ کامل: {fa_bool(c['html_close'])} |
| انتشار در بازار | محصول `{res['published']['productId']}` — «{res['published']['title']}» با قیمت {res['published']['price']} دیین، فعال: {fa_bool(res['published']['active'])} |

**نتیجه: Slot 1 (`openrouter/free`) با HTTP 200 پاسخ داد، `usedFallback: false` بود و یک سند HTML فارسی RTL تولیدی LLM واقعی ساخته و در بازار کیان منتشر شد.** ابتدای پاسخ مدل: «{c['replyHead']}…»

نسخهٔ نهایی آرتیفکت برای بازبینی آفلاین: `/tmp/kian-e2e-or/generated.html` (پس از انتشار در محصول شمارهٔ {res['published']['productId']} داخل پایگاه‌دادهٔ زنده هم موجود است)."""
p = pathlib.Path("/home/z/my-project/download/OPENROUTER_BENCHMARK_REPORT_FA.md")
t = p.read_text(encoding="utf-8")
marker = "(بخش پس از اجرا تکمیل می‌شود.)"
assert marker in t, "report section 7 marker missing"
p.write_text(t.replace(marker, section), encoding="utf-8")
print("report section 7 filled with live numbers")
PY

# ── 4) refresh the delivery zip from the committed tree (secret-free) ─────
cd /home/z/my-project
HASH=$(git rev-parse HEAD)
git archive --format=zip -o /tmp/kian_zip_head.zip HEAD
python3 - "$HASH" <<'PY'
import sys, zipfile
h = sys.argv[1]
src = zipfile.ZipFile("/tmp/kian_zip_head.zip")
dst = zipfile.ZipFile("/home/z/my-project/download/kian-v2-railway-migrated.zip", "w", zipfile.ZIP_DEFLATED)
for it in src.infolist():
    dst.writestr(it, src.read(it.filename))
dst.comment = h.encode()
dst.close()
src.close()
PY
echo "delivery zip refreshed from commit $HASH (no .env.local inside — gitignored)"
echo "POST-RESET PROOF: ALL DONE"
