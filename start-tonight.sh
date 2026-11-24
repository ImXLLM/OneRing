#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# KIAN — «شبِ تستِ محلی» (start-tonight.sh)
# یک فرمان، سه سرویس: کرنل Rust (:8088) + مینی‌اپ (:3001) + پنل ادمین (:3002)
#
# استفاده:
#   chmod +x start-tonight.sh
#   ./start-tonight.sh
#
# پس از بالا آمدن:
#   مینی‌اپ:   http://localhost:3001   (ورود یک‌کلیکی پرسوناهای دمو)
#   پنل ادمین: http://localhost:3002   (کلید: dev-admin-key)
#   کرنل:      http://127.0.0.1:8088/api/v1/health
#
# خروج: Ctrl+C — همهٔ سرویس‌ها با هم بسته می‌شوند.
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo ""
echo "══════════════════════════════════════════════════════"
echo "  KIAN — شبِ تستِ محلی  🚀"
echo "══════════════════════════════════════════════════════"
echo ""

# ── ۱) .env — اگر نبود از .env.example با پیش‌فرض‌های دمو می‌سازیم ──────────
if [ ! -f .env ]; then
  cp .env.example .env
  # اطمینان از حالت دمو برای تستِ امشب (حتی اگر در example خالی بود)
  sed -i.bak 's/^KIAN_DEV_MODE=.*/KIAN_DEV_MODE=true/' .env 2>/dev/null || true
  sed -i.bak 's/^KIAN_ADMIN_KEY=.*/KIAN_ADMIN_KEY=dev-admin-key/' .env 2>/dev/null || true
  rm -f .env.bak
  echo "✅ فایل .env از .env.example ساخته شد (KIAN_DEV_MODE=true، KIAN_ADMIN_KEY=dev-admin-key)"
else
  echo "ℹ️  .env موجود است — همان استفاده می‌شود."
fi

# export‌های الزامی دمو — حتی اگر .env قبلاً با مقادیر دیگر باشد، برای این اجرا
# حالت dev و کلید دمو تضمین می‌شود (فقط روی همین پروسه).
export KIAN_DEV_MODE=true
export KIAN_ADMIN_KEY="${KIAN_ADMIN_KEY:-dev-admin-key}"
export KIAN_DATABASE_URL="${KIAN_DATABASE_URL:-data/kian-dev.sqlite}"
export KIAN_HOST="${KIAN_HOST:-127.0.0.1}"
export KIAN_PORT="${KIAN_PORT:-8088}"
export NEXT_PUBLIC_KIAN_API="${NEXT_PUBLIC_KIAN_API:-http://127.0.0.1:8088/api/v1}"
export NEXT_PUBLIC_KERNEL_URL="${NEXT_PUBLIC_KERNEL_URL:-http://127.0.0.1:8088}"

# ── ۲) کرنل Rust — بیلد incrémental (با کشِ گرم چند ثانیه طول می‌کشد) ──────
if [ ! -x target/debug/kian-server ] && [ ! -x "$CARGO_TARGET_DIR/debug/kian-server" ]; then
  echo "🔧 بیلد کرنل (cargo build — بار اول ممکن است چند دقیقه طول بکشد)…"
  cargo build -p kian-server
else
  echo "🔧 بیلد کرنل (افزایشی)…"
  cargo build -p kian-server
fi

SERVER_BIN="$(ls -1 "$CARGO_TARGET_DIR/debug/kian-server" 2>/dev/null || ls -1 target/debug/kian-server 2>/dev/null)"
if [ -z "$SERVER_BIN" ] || [ ! -x "$SERVER_BIN" ]; then
  SERVER_BIN="target/debug/kian-server"
fi

# ── ۳) بالا آوردن سرویس‌ها با پاکسازی مطمئن (trap) ─────────────────────────
PIDS=()
cleanup() {
  echo ""
  echo "🛑 بستن همهٔ سرویس‌ها…"
  for pid in "${PIDS[@]:-}"; do
    [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
  echo "خداحافظ! 👋"
}
trap cleanup EXIT INT TERM

echo "🧠 کرنل KIAN روی http://127.0.0.1:$KIAN_PORT (SQLite: $KIAN_DATABASE_URL)…"
"$SERVER_BIN" &
PIDS+=("$!")

# چند لحظه برای مهاجرت دیتابیس
for i in $(seq 1 30); do
  if curl -sf "http://127.0.0.1:$KIAN_PORT/api/v1/health" >/dev/null 2>&1; then
    echo "✅ کرنل سالم است."
    break
  fi
  sleep 0.5
done

echo "📱 مینی‌اپ روی http://localhost:3001 …"
pnpm --filter @kian/miniapp exec next dev -p 3001 &
PIDS+=("$!")

echo "🛡️  پنل ادمین روی http://localhost:3002 (کلید: dev-admin-key)…"
pnpm --filter @kian/admin exec next dev -p 3002 &
PIDS+=("$!")

echo ""
echo "══════════════════════════════════════════════════════"
echo "  ✅ آمادهٔ تست!"
echo "     ۱) مینی‌اپ:    http://localhost:3001"
echo "        ↳ دکمهٔ «🎨 ورود یک‌کلیکی: سازنده (#1001)» را بزن."
echo "     ۲) پنل ادمین: http://localhost:3002  — کلید: dev-admin-key"
echo "     ۳) راهنمای کامل: TEST_TONIGHT_FA.md"
echo "══════════════════════════════════════════════════════"
echo ""

wait
