@echo off
REM ─────────────────────────────────────────────────────────────────────────────
REM KIAN — «شبِ تستِ محلی» برای ویندوز (CMD / PowerShell)
REM
REM استفاده: دوبار کلیک روی start-tonight.bat یا اجرای آن در ترمینال.
REM پیش‌نیاز: Rust (rustup) + Node.js 20+ + corepack enable + pnpm install
REM
REM پس از بالا آمدن:
REM   مینی‌اپ:   http://localhost:3001   (ورود یک‌کلیکی پرسوناهای دمو)
REM   پنل ادمین: http://localhost:3002   (کلید: dev-admin-key)
REM   کرنل:      http://127.0.0.1:8088/api/v1/health
REM
REM خروج: Ctrl+C در همین پنجره — یا بستن پنجره‌های باز‌شده.
REM ─────────────────────────────────────────────────────────────────────────────
setlocal enabledelayedexpansion

cd /d "%~dp0"

echo.
echo ======================================================
echo   KIAN - shab-e test-e mahalli (Windows)
echo ======================================================
echo.

REM ── 1) .env — agar nabood az .env.example misazim ──────────────────────────
if not exist .env (
  copy /Y .env.example .env >nul
  echo [OK] .env az .env.example sakhte shod (KIAN_DEV_MODE=true, KIAN_ADMIN_KEY=dev-admin-key)
) else (
  echo [i] .env mojood ast - haman estefade mishavad.
)

REM ── 2) env-haye demoo baraye in ejra ────────────────────────────────────────
set KIAN_DEV_MODE=true
if "%KIAN_ADMIN_KEY%"=="" set KIAN_ADMIN_KEY=dev-admin-key
if "%KIAN_DATABASE_URL%"=="" set KIAN_DATABASE_URL=data\kian-dev.sqlite
if "%KIAN_HOST%"=="" set KIAN_HOST=127.0.0.1
if "%KIAN_PORT%"=="" set KIAN_PORT=8088
if "%NEXT_PUBLIC_KIAN_API%"=="" set NEXT_PUBLIC_KIAN_API=http://127.0.0.1:8088/api/v1
if "%NEXT_PUBLIC_KERNEL_URL%"=="" set NEXT_PUBLIC_KERNEL_URL=http://127.0.0.1:8088

REM ── 3) build kernel (cargo) ─────────────────────────────────────────────────
echo [build] cargo build -p kian-server ...
cargo build -p kian-server
if errorlevel 1 (
  echo [FAIL] cargo build nakard - Rust nasb shode? https://rustup.rs
  pause
  exit /b 1
)

REM ── 4) server dar panjere joda + miniapp + admin ────────────────────────────
echo [run] kernel: http://127.0.0.1:%KIAN_PORT%
start "KIAN kernel :8088" cmd /c "target\debug\kian-server.exe & pause"

echo [run] miniapp: http://localhost:3001
start "KIAN miniapp :3001" cmd /c "pnpm --filter @kian/miniapp exec next dev -p 3001"

echo [run] admin: http://localhost:3002 (key: dev-admin-key)
start "KIAN admin :3002" cmd /c "pnpm --filter @kian/admin exec next dev -p 3002"

echo.
echo ======================================================
echo   [READY] Amade-ye test!
echo      1) MiniApp:    http://localhost:3001
echo         -> dokme-ye "ورود یک‌کلیکی: سازنده (#1001)"
echo      2) Admin:      http://localhost:3002  - key: dev-admin-key
echo      3) Full guide: TEST_TONIGHT_FA.md
echo ======================================================
echo.
echo Baraye bastan-e hame, panjere-haye "KIAN kernel/miniapp/admin" ra bebandid.
pause
