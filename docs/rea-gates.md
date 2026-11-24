# دروازه‌های REA — هدف 10/10

هر ردیف: فیلتر REA (`audit-release-exposure`) → مقابلهٔ ما → سنجه.

| فیلتر REA | مقابله در KIAN | سنجه |
|---|---|---|
| `javascript.source_maps` / `source_map_sources` | `productionBrowserSourceMaps: false` + CI پادشاه `.map` در dist | audit → not_observed |
| `text.credential_markers` | کلیدها فقط از env؛ گیت grep روی bundle | audit → not_observed |
| `native.debug_companions` / `native.pdb_path` | `[profile.release] strip="symbols"` | audit → not_observed |
| `rust.dependency_metadata` | بدون leak متادیتا؛ LTO fat + codegen-units=1 | audit → not_observed |
| `electron.*` (دسکتاپ آینده) | فازهای Electron فلپ: OnlyLoadAppFromAsar، بدون RunAsNode/Inspect | audit → not_observed |
| `packaging.extractable_payload` | برای وب نامربوط؛ برای دسکتاپ asar + integrity | audit → not_observed/low |
| — (خارج از audit) | `compare-static-and-observed-endpoints`: اندپوینت‌های استاتیک فرانت ↔ قرارداد OpenAPI | تفاوت صفر |
| — (خارج از audit) | `analyze-web-bundle` قبل از انتشار | گزارش پاک |

**اجرای گیت:** `make rea-gate` یا `node tools/rea/release-exposure.mjs <artifact>` — خروج 3 = شکست، خروج 0 = 10/10.

## گام ۱ — اجرا شده (M1، 2026-11-20)

- credential_markers: grep روی src+public → صفر مورد ✓
- source_maps: productionBrowserSourceMaps=false + هیچ .map در سورس ✓
- هدرها: HSTS، nosniff، Referrer-Policy، Permissions-Policy ✓
- sw.js: فقط GET، فقط same-origin، بدون کش /api ✓
- تست آفلاین مرورگر: reload با network offline → پوسته کش‌شده سرو شد ✓
- audit-release-exposure کامل روی آرتیفکت build: در M1-finish (بعد از انتشار standalone)

## گام ۴ — اجرا شده (M4، 2026-11-21)

- X-KIAN-Key: گوار fail-closed سه‌وضعیتی (ADMIN_DISABLED/ADMIN_KEY_REQUIRED/INVALID_ADMIN_KEY) با
  مقایسهٔ زمان‌ثابت (subtle ct_eq) — ماتریس گارد روی سیم تست سبز ✓
- redaction: شش الگوی قطعی (private_key_hex/bot_token/pem_key/jwt/mnemonic/long_hex) — قبل از
  دیدن هر اسلات مدل؛ شواهد: ۱۴ تست kian-ai سبز ✓
- sandbox: مهارت‌های هرمس فقط هندلرهای ثبت‌شده را اجرا می‌کنند (بدون eval/exec/Command)؛ سپر
  ورودی/خروجی با حکم ALLOW/REDACTED/BLOCKED — ۱۷ تست kian-hermes سبز ✓
- هر ۱۰ فراخوانی require_admin در هندلرهای ادمین + مسیرهای /admin/* و /a2a/* و /mcp/invoke وایر ✓
- بدون literal کلید در سورس پنل ادمین (کلید فقط در sessionStorage تب) ✓
- audit-release-exposure --fail_on low: kernel=0 admin=0 web=0 — صفر ردیف exposed ✓
- گزارش: docs/rea/step4-m4-gate-report.txt — exit 0

## گام ۵ — اجرا شده (M5 نهایی، 2026-11-21)

- حاکمیت: پارلمان HITL گره‌خورده با مادهٔ ۷ — آستانه فقط از `kian-domain` می‌آید
  (یک منبع حقیقت)؛ رأی انسانیِ موافق برای مبالغ ≥ ۵۰,۰۰۰ Đ اجباری؛ نصاب/اکثریت/
  دلیل قطعی در `kian-nation` — ۱۴ تست خالص سبز ✓
- اجرا فقط از مسیر دامنهٔ خالص M2: `bill.release/bill.refund` با
  `approved_by_admin: true` یا posting دوطرفهٔ خزانه — هیچ مسیر جانبی برای پول ✓
- چهارزنجیره: آدرس‌ها فقط با چک‌سام رسمی زنجیره قبول می‌شوند — Base58Check ترون،
  ۳۲بایت سولانا، CRC16-XMODEM تون (big-endian)، EIP-55 با keccak-256 خالص
  (وکتورهای مرجع: خالی/abc/۲۰۰بایت + ۴ آدرس رسمی EIP-55) ✓
- قراردادهای کانونیک USDT چهار زنجیره از منابع رسمی (tron.network، Solscan،
  Tonscan، PancakeSwap) راستی‌آزمایی و در ثابت‌ها قفل شدند ✓
- واریز USDT: UNIQUE(chain, txHash) — «همان هش دوبار ضرب نمی‌شود»؛ ضرب از مسیر
  مادهٔ ۳ با نرخ `KIAN_USDT_TO_DIN` و فقط با ادمین ✓
- cargo test --workspace سبز (۴۳ تست سرور شامل چرخهٔ کامل بازار→لایحه→اجرا) ✓
- cargo clippy --workspace --all-targets -- -D warnings: صفر خطا (سازگاری clippy 1.100) ✓
- بدون credential در سورس M5 و فرانت‌ها ✓
- audit-release-exposure --fail_on low: kernel=0 admin=0 web=0 — صفر ردیف exposed ✓
- گزارش: docs/rea/step5-m5-final-gate-report.txt — exit 0

## گام ۶ — اجرا شده (M6 بات تلگرام، 2026-11-21)

- هستهٔ خالص: `crates/bot` — orchestrator با صفر I/O (بدون reqwest/teloxide/net)
  و سقف نرخ per-chat با ساعت تزریقی؛ هشدار یک‌بار در پنجره (بدون تقویت اسپم)؛
  برش پیام روی مرز کاراکتر (سقف تلگرام ۴۰۹۶) — ۱۵ تست آفلاین با FakePort ✓
- هویت تک‌مسیره: ضرب initData فقط با `build_bot_init_data` (contracts::security —
  نقطهٔ واحد) و تأیید فقط با `verify_telegram_init_data`؛ شِل طول‌کشی از
  `POST /auth/telegram` همان مسیر مینی‌اپ می‌رود؛ وبهوک درون‌پردازشی همان جفت
  ضرب→تأیید را طی می‌کند؛ **هیچ مسیر جانبی برای هویت** — بدون X-KIAN-Key در
  kian-bot، بدون endpoint هویت جدید ✓
- وبهوک fail-closed: `POST /bot/webhook` با هدر رسمی
  `X-Telegram-Bot-Api-Secret-Token` و مقایسهٔ زمان‌ثابت؛ بدون
  `KIAN_BOT_WEBHOOK_SECRET` → 503 `BOT_DISABLED`؛ هدر غایب/نادرست → 401
  `BOT_UNAUTHORIZED`؛ ران‌تایم فقط با هر دو راز (توکن + راز ≥ ۱۶) بالا می‌آید ✓
- پاسخ به تلگرام پشت پورت `TelegramSender` — تولید: Bot API رسمی با rustls؛
  تست: `RecordingSender` صفر شبکه؛ آپدیت غیرمتنی → 200 `handled:false` بدون
  هیچ پاسخی (به ناشناخته پاسخ نمی‌دهد) ✓
- دو شِل یک تجزیه: teloxide Update → اجزا → `BotUpdate::from_parts` — همان
  مسیر وبهوک (`from_json`)؛ شِل طول‌کشی با teloxide 0.17 rustls (بدون
  openssl-system) و توقف تمیز با Ctrl+C/StopToken ✓
- cargo test --workspace سبز (۲۴۴ تست؛ سرور ۴۸ شامل ماتریس گارد + جریان کامل
  آفلاین وبهوک) ✓
- cargo clippy --workspace --all-targets -- -D warnings: صفر خطا ✓
- بدون credential در سورس M6 و فرانت‌ها ✓
- audit-release-exposure --fail_on low: kernel=0 admin=0 web=0 ✓
- گزارش: docs/rea/step6-m6-gate-report.txt — exit 0
