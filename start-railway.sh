#!/usr/bin/env sh
# ═══════════════════════════════════════════════════════════════════════════
#  start-railway.sh — boot KIAN v2 on Railway with a persistent SQLite volume
#
#  Seed logic (once and for all):
#    1) The database path is read from KIAN_DATABASE_URL (default /data/kian.db).
#    2) If the database file does not exist (first boot or a fresh volume), the community's
#       pre-migrated build (seed/kian-production.db — 466 citizens, 20 products,
#       334 KbEntry skills, 108 escrows, 14 migrations, 24 war-room ticks,
#       a sanitized 58,700.2 KIAN genesis) is copied into that same path.
#    3) If the database already exists (later boots) — nothing is overwritten;
#       the kernel checks the sqlx migrations and continues.
#    4) exec kian-server — the boot PID replaces the script (signal-safe).
# ═══════════════════════════════════════════════════════════════════════════
set -eu

DB_URL="${KIAN_DATABASE_URL:-file:/data/kian.db}"
# The same kernel normalization (crates/db/src/store.rs::connect): schema and ?query are stripped
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

echo "[start-railway] MAIRON Sovereign OS — 466 citizens, 20 products, 334 KbEntry skills, 14 migrations, 24 War-Room ticks"
echo "[start-railway] launching kian-server on ${KIAN_HOST:-0.0.0.0}:${KIAN_PORT:-8088}"
exec /app/kian-server
