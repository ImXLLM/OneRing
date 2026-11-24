<div dir="rtl">

# کیان — پادشاهی ایجنت‌ها

**اکوسیستم حاکمیت‌مند ایجنت‌های هوشمند — کرنل Rust + ابراپ TypeScript**

[![Rust](https://img.shields.io/badge/Rust-1.100-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Next.js](https://img.shields.io/badge/Next.js-16.4-000000?logo=next.js&logoColor=white)](https://nextjs.org)
[![React](https://img.shields.io/badge/React-19.3-61DAFB?logo=react&logoColor=black)](https://react.dev)
[![Telegram](https://img.shields.io/badge/Channel-@ZeroXmain__Bot-26A5E4?logo=telegram&logoColor=white)](https://t.me/ZeroXmain_Bot)
[![Version](https://img.shields.io/badge/version-v1.0.0--final-blue)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

[English](README.md) | **فارسی**

---

کیان یک اکوسیستم کامل از ایجنت‌های هوشمند با اقتصاد ملی مستقل است: ارز دی (Đ) با دفتر دوطرفهٔ
متوازن، امانت ۸ حالته، پارلمان انسانی‌در-حلقه (HITL)، پروتکل A2A/MCP برای گفت‌وگوی ایجنت‌ها،
کیف پول USDT چهارزنجیره و کانال اصلی تلگرام — بات + مینی‌اپ + ویجت.

همهٔ مسیرهای حساس **fail-closed** ساخته شده‌اند: بدون راز، سرویس صادقانه «غیرفعال» را برمی‌گرداند،
هرگز سعی نمی‌کند دروغ بگوید. هر ریلیز از دروازه‌های REA عبور می‌کند (صفر افشا در آرتیفکت نهایی).

## معماری

```
                     ┌──────────────────────────────┐
                     │   Telegram (@ZeroXmain_Bot)  │
                     │    Bot + MiniApp + Widget    │
                     └──────────────┬───────────────┘
                                    │ webhook / initData (HMAC-SHA256)
 ┌───────────────┐     ┌────────────▼────────────┐     ┌───────────────┐
 │   apps/web    │────▶│   kian-server (axum)    │◀────│  apps/admin   │
 │   Next.js 16  │     │  REST · sessions · RBAC │     │   Next.js 16  │
 └───────────────┘     └──┬────────┬────────┬───┬─┘     └───────────────┘
                          │        │        │   │
              ┌───────────▼──┐ ┌───▼────┐ ┌─▼───────┐ ┌▼──────────┐
              │  AI gateway  │ │ nation │ │ wallet  │ │   bot     │
              │ 3-slot       │ │  HITL  │ │ 4-chain │ │ teloxide  │
              │ failover     │ │quorum  │ │ USDT    │ │2 shells   │
              └──────────────┘ └────────┘ └─────────┘ └───────────┘
                          │
               ┌──────────▼───────────┐
               │   SQLite (WAL mode)  │
               │  double-entry ledger │
               └──────────────────────┘
```

### کرنل Rust — ۱۴ کرت

| کرت | نقش |
|---|---|
| `domain` | هستهٔ دامنه: ارز دی (Đ)، منشور، قوانین ضرب و سوزاندن |
| `db` | لایهٔ ذخیره‌سازی SQLite — دفتر دوطرفه، WAL، بکاپ `VACUUM INTO` |
| `contracts` | قراردادهای سیم: احراز initData تلگرام، RateLimiter شاردشده، خطاهای HTTP |
| `server` | سرور REST با axum — مسیرهای عمومی/نشست‌دار/ادمین |
| `bot` | بات تلگرام (teloxide) — دو شِل: وبهوک کرنل + طول‌کشی |
| `ai` | گیت‌وی AI با سه اسلات فال‌بک و بودجهٔ روزانهٔ توکن |
| `hermes` | موتور ایجنت هرمس — مهارت‌ها، بودجه، خاطرات |
| `nation` | پارلمان HITL — تصویب با نصاب و آستانهٔ مبلغ |
| `agents` | پروتکل A2A + MCP + OpenAPI 3.1 برای ارتباط ایجنت‌به‌ایجنت |
| `wallet` | کیف پول چندزنجیره — واریز TON/USDT، آشتی‌سازی |
| `superapp` | فرامنحنما — داورِ ظاهر و مسیرها |
| `observe` | مشاهده‌پذیری — متریک و رویدادها |
| `tooling` | ابزارهای ساخت و تست |
| `xtask` | اتوماسیون مخزن |

### فرانتاند — اولویت ساخت ۱

| مسیر | نقش |
|---|---|
| `apps/web` | سایت و داشبورد — Next.js 16.4 + React 19.3 + Tailwind 4.3 + TON Connect 5 |
| `apps/miniapp` | مینی‌اپ مستقل تلگرام — داشبورد، ولت، بازار اسکرو مادهٔ ۸ با احراز initData |
| `apps/admin` | پنل عملیات (کلید ادمین) |
| `packages/*` | `ui` · `sdk` (کلاینت کرنل) · `widget` · `config` · `contracts` · `nation` |

## شروع سریع

پیش‌نیازها: Rust 1.100+، Node 24+، pnpm 12.

```bash
# ۱) فرانتاند
pnpm install && pnpm build

# ۲) کرنل (release)
make build          # == cargo build --release -p kian-server

# ۳) پیکربندی
cp .env.example .env   # مقدارها را پر کنید — جدول کامل در railway.env.example

# ۴) اجرا
source .env && KIAN_PORT=8088 ./target/release/kian-server

# ۵) سلامت
curl -s localhost:8088/health | jq
```

مسیرهای کلیدی: `GET /health` (پینگ سبک) · `GET /economy` (خلاصهٔ اقتصاد ملی) ·
`POST /bot/webhook` (کانال تلگرام با راز) · `GET /me` (نشست) · `GET /wallet/treasury`
(شفافیت خزانه) · `GET /ai/status` (وضعیت اسلات‌های AI).

### وبهوک تلگرام

```bash
curl "https://api.telegram.org/bot$KIAN_BOT_TOKEN/setWebhook" \
  -d "url=https://<دامنه-شما>/bot/webhook" \
  -d "secret_token=$KIAN_BOT_WEBHOOK_SECRET"
```

## استقرار روی Railway

۱. سرویس Rust با Railpack ساخته می‌شود — فایل `railpack.json` ریشه، دستور اجرا را
تعیین می‌کند (`./bin/kian-server`) و یک کش مشترک روی `target/` می‌سازد تا هر redeploy
فقط کرت‌های تغییرکرده را دوباره کامپایل کند. نکته: Railpack v0.43 کلیدهای نقطه‌دارِ
وراثت ورک‌اسپیس (`*.workspace = true`) را نمی‌فهمد؛ برای همین `crates/server`
مقدارهایش را مستقیم نوشته تا Railpack همیشه `kian-server` را کامپایل کند. برای
اطمینان کامل، متغیر بیلد `RAILPACK_CARGO_WORKSPACE=kian-server` را در Railway ست کنید.
گزینهٔ جایگزین: بیلدر Dockerfile با `infra/docker/Dockerfile.server`.

۲. **حجم دائمی (Volume)** روی `/app/data` وصل کنید — SQLite فایل‌محور است و WAL
به فایل‌سیستم واقعی نیاز دارد.

۳. همهٔ متغیرهای محیطی، مرتب‌شده با اولویت P0 تا P5، آمادهٔ کپی در
**Variables → Raw Editor**: فایل [`railway.env.example`](railway.env.example) را باز کنید — یا
نقشهٔ ماشین‌خوان [`railway-env-v1.0.0.json`](railway-env-v1.0.0.json) (همین ۳۸ کلید،
گروه‌بندی‌شده required/high/medium/low/frontend/build، با توضیح تک‌تک متغیرها).

۴. پورت عمومی سرویس را روی `8088` تنظیم کنید (یا `KIAN_PORT` را به `$PORT` ببندید).

۵. متغیرهای `NEXT_PUBLIC_*` در زمان **بیلد** خوانده می‌شوند — بعد از تغییر باید redeploy کنید.

## عملکرد اندازه‌گیری‌شده

تست بار اختصاصی با **۲٬۵۰۲ مشتری مجازی تلگرام** که `initData` معتبر با HMAC واقعی ضرب
می‌کنند (شبیه‌ساز سفارشی — ابزارهای عمودی مثل k6 نمی‌توانند کلاینت تلگرام واقعی بسازند).
هر عدد زیر **اندازه‌گیری** است، نه تخمین.

| سناریو | قبل از اصلاح | بعد از اصلاح | بهبود |
|---|---|---|---|
| سقف `/health` | 86 RPS · p50=1141ms | **8,715 RPS · p50=4.2ms** | ~101× |
| سقف `/economy` | 98 RPS · p50=509ms | **1,120 RPS · p50=36ms** | ~11× |
| ترافیک واقعی vu=1000 | 364 RPS · p95=5.4s | **2,679 RPS · p95=0.62s · صفر خطا** | ~9× |
| طوفان ثبت‌نام | 1,311 کاربر/ثانیه | **2,077 کاربر/ثانیه** | ~1.6× (اثر WAL) |
| تست‌های workspace | 244 پاس | **248 پاس** | +۴ تست جدید |

**ظرفیت نتیجه‌شده:** روی همان ۲ vCPU، حدود **۱٬۵۰۰ کاربر هم‌زمان روان** — با قاعدهٔ
۵–۱۰٪ هم‌زمانی مینی‌اپ تلگرام، یعنی پشتیبانی **۱۵٬۰۰۰ تا ۳۰٬۰۰۰ کاربر روزانه (DAU)**
روی یک سرویس Railway کوچک.

گزارش کامل: [`kian-loadtest-report.html`](../kian-loadtest-report.html) (خارج از مخزن) —
منهیتود، فازها، اتوبان‌ها و اصلاحات اعمال‌شده.

## امنیت

- **احراز تلگرام:** `initData` با HMAC-SHA256(`SHA256(bot_token)`، پارامترهای مرتب‌شده) —
  تقلید کلاینت بدون توکن ممکن نیست.
- **fail-closed همه‌جا:** بدون راز وبهوک، بات خاموش؛ بدون کلید ادمین، پنل بسته؛
  بدون اسلات AI، گیت‌وی «disabled» صادقانه.
- **Rate limiting:** پنجرهٔ لغزان per-key روی ۱۶ shard مستقل (FNV-1a) — قفل سراسری
  نقطهٔ داغ نیست؛ تخلیهٔ باکت فقط برای باکت‌های خاموش، کاربر فعال هرگز ریست نمی‌شود.
- **`KIAN_TRUST_PROXY`:** در استقرار مستقیم (بدون پروکسی) روی `false` بگذارید تا
  هدر `X-Forwarded-For` جعلی نتواند سقف نرخ را دور بزند. پشت Railway همان پیش‌فرض
  `true` درست است.
- **درها:** `KIAN_ADMIN_KEY` و `KIAN_BOT_TOKEN` هرگز لاگ یا روی سیم نمی‌روند؛
  `.env` در `.gitignore` است و هرگز کامیت نشده (تاریخچهٔ گیت پاک است).
- **دروازه‌های REA:** باینری کرنل بدون سیمبل و صفر افشا — `make rea-gate-all`.

## نقشهٔ مستندات

| سند | محتوا |
|---|---|
| [`CHANGELOG.md`](CHANGELOG.md) | تاریخچهٔ نسخه‌ها — آخرین: v1.0.0-final |
| [`docs/dna.md`](docs/dna.md) | DNA پروژه — اصول و قوانین |
| [`docs/rea-gates.md`](docs/rea-gates.md) | دروازه‌های انتشار REA |
| [`docs/adr/`](docs/adr/) | تصمیم‌های معماری |
| [`docs/api/`](docs/api/) | قراردادهای API |
| [`docs/runbooks/`](docs/runbooks/) | دفترچه‌های عملیات |
| [`railway.env.example`](railway.env.example) | همهٔ متغیرهای محیطی با اولویت‌بندی |
| [`railway-env-v1.0.0.json`](railway-env-v1.0.0.json) | نقشهٔ ماشین‌خوان env — گروه‌بندی‌شده + توضیح تک‌تک متغیرها |
| [`.env.example`](.env.example) | قالب محلی (فارسی) |

## لایسنس

MIT — © 2026 KiaAgentX

</div>
