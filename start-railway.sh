#!/usr/bin/env sh
# ═══════════════════════════════════════════════════════════════════════════
#  start-railway.sh — بوت KIAN v2 روی Railway با حجم دائمی SQLite
#
#  منطق seed (یک‌بار برای همیشه):
#    1) مسیر دیتابیس از KIAN_DATABASE_URL خوانده می‌شود (پیش‌فرض /data/kian.db).
#    2) اگر فایل دیتابیس وجود نداشته باشد (بوت اول یا حجم نو)، نسخهٔ
#       پیش‌مهاجرت‌شدهٔ جامعه (seed/kian-production.db — ۴۶۶ شهروند، ۲۰ محصول،
#       ۳۳۴ مهارتِ KbEntry، ۱۰۸ اسکرو، ۱۴ مهاجرت، ۲۴ تیکِ اتاقِ جنگ،
#       genesis پاک‌سازی‌شدهٔ ۵۸٬۷۰۰.۲ KIAN) داخل همان مسیر کپی می‌شود.
#    3) اگر دیتابیس از قبل هست (بوت‌های بعدی) — هیچ‌چیز بازنویسی نمی‌شود؛
#       کرنل مهاجرت‌های sqlx را چک می‌کند و ادامه می‌دهد.
#    4) exec kian-server — PID بوت جای اسکریپت می‌نشیند (signal-safe).
# ═══════════════════════════════════════════════════════════════════════════
set -eu

DB_URL="${KIAN_DATABASE_URL:-file:/data/kian.db}"
# همان نرمال‌سازی کرنل (crates/db/src/store.rs::connect): اسکیمه و ?query حذف می‌شود
DB_PATH=$(printf '%s' "$DB_URL" | sed -e 's|^file:||' -e 's|^sqlite:||' -e 's|?.*$||')
[ -n "$DB_PATH" ] || DB_PATH="/data/kian.db"
# Seed source — overridable via the P0 env tier (KIAN_SEED_DATABASE_PATH):
SEED="${KIAN_SEED_DATABASE_PATH:-/app/seed/kian-production.db}"

mkdir -p "$(dirname "$DB_PATH")" 2>/dev/null || true

# Railway bind-mount guard — a root-owned volume would fail mkdir/cp under the
# kian user; fall back to a writable scratch dir instead of crash-looping.
if [ ! -w "$(dirname "$DB_PATH")" ]; then
    echo "[start-railway] WARNING: $(dirname "$DB_PATH") not writable (bind-mount ownership) — using /tmp/kian-data"
    DB_PATH="/tmp/kian-data/$(basename "$DB_PATH")"
    mkdir -p "$(dirname "$DB_PATH")"
fi

if [ ! -f "$DB_PATH" ]; then
    if [ -f "$SEED" ]; then
        echo "[start-railway] first boot — seeding $DB_PATH from migrated community DB"
        cp "$SEED" "$DB_PATH"
        echo "[start-railway] seeded: 466 citizens, 20 products, 334 KbEntry skills, 108 escrows, 14 migrations, 24 War-Room ticks, genesis 58,700.2 KIAN"
    else
        echo "[start-railway] no seed found — starting with a fresh empty ledger"
    fi
else
    echo "[start-railway] existing database found at $DB_PATH — seed skipped (idempotent)"
fi

# keep the kernel pointed at the exact path the seed logic decided on
export KIAN_DATABASE_URL="file:${DB_PATH}"

echo "[start-railway] Wave-10 boot banner — 466 citizens, 20 products, 334 KbEntry skills, 14 migrations, 24 War-Room ticks"
echo "[start-railway] launching kian-server on ${KIAN_HOST:-0.0.0.0}:${KIAN_PORT:-8088}"
exec /app/kian-server
