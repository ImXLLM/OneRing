#!/usr/bin/env sh
# ═══════════════════════════════════════════════════════════════════════════
#  start-railway.sh — بوت KIAN v2 روی Railway با حجم دائمی SQLite
#
#  منطق seed (یک‌بار برای همیشه):
#    1) مسیر دیتابیس از KIAN_DATABASE_URL خوانده می‌شود (پیش‌فرض /data/kian.db).
#    2) اگر فایل دیتابیس وجود نداشته باشد (بوت اول یا حجم نو)، نسخهٔ
#       پیش‌مهاجرت‌شدهٔ جامعه (seed/kian-production.db — ۴۶۶ شهروند، ۱۲ محصول،
#       ۱۰۸ اسکرو، genesis پاک‌سازی‌شدهٔ ۵۸٬۷۰۰.۲ KIAN) داخل همان مسیر کپی می‌شود.
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

mkdir -p "$(dirname "$DB_PATH")"

if [ ! -f "$DB_PATH" ]; then
    if [ -f "$SEED" ]; then
        echo "[start-railway] first boot — seeding $DB_PATH from migrated community DB"
        cp "$SEED" "$DB_PATH"
        echo "[start-railway] seeded: 466 citizens, 12 products, 108 released escrows, genesis 58,700.2 KIAN"
    else
        echo "[start-railway] no seed found — starting with a fresh empty ledger"
    fi
else
    echo "[start-railway] existing database found at $DB_PATH — seed skipped (idempotent)"
fi

echo "[start-railway] launching kian-server on ${KIAN_HOST:-0.0.0.0}:${KIAN_PORT:-8088}"
exec /app/kian-server
