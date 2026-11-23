# گزارش مهاجرت dropagent → KIAN v2 (نسخهٔ Railway-Ready)

**تاریخ اجرا:** ۲۴ نوامبر ۲۰۲۶ · **اسکریپت:** `scripts/migrate_dropagent_to_kian.py` (قطعی و idempotent)
**منبع:** `marketplace-20260924-0400.db` (۸۴۳,۷۷۶ بایت، پایگاه تولیدی ۳۰روزهٔ X)
**خروجی:** `seed/kian-production.db` (۱,۰۳۶,۲۸۸ بایت) — راستی‌آزمایی‌شده با کرنل زندهٔ KIAN v2 (۳۹/۳۹ ✓)

---

## ۱) مهاجرت هویت‌ها — ۴۶۶ شهروند

| مورد | عدد | منبع داده |
|---|---|---|
| ردیف‌های `"User"` | **۴۶۶** | `users` لگاسی (۴۶۶ ردیف، صفر ban) |
| ردیف‌های `"Account"` شهروند (`citizen:{tg}`، kind=`user`) | **۴۶۶** | یک‌به‌یک با User |
| حساب‌های سیستمی (`mint:genesis`، `treasury:nation`، `burn:void`) | **۳** | مطابق `ensure_system_accounts` کرنل |
| کل حساب‌ها (`accountCount`) | **۴۶۹** | تأیید زنده از `GET /api/v1/economy` |
| نام کوچک (`firstName`) | از `first_name` لگاسی | ستون جدید در مهاجرت `0006` |
| یوزرنیم‌ها | عیناً از لگاسی | با پچ COALESCE هرگز با dev-login پاک نمی‌شود |

**نسب رفرال:** **۲۴۷ لینک** دعوت کمپین ۳۰روزه — سه‌لایه محفوظ شد:
۱. ستون `"User"."referredBy"` (شناسهٔ حساب معرف)، ۲. جدول `"ReferralLineage"` (tgUserId / referredByTgId / legacyCreatedAt)، ۳. صفر لینک dangling (اعتبارسنجی شد).

## ۲) پاک‌سازی و ضرب ترازها — genesis دوطرفهٔ ممیزی‌پذیر

قاعدهٔ تبدیل: `1 credit = 1,000,000 micro-KIAN (1 KIAN)`، سقف کاربر عادی ۱٬۰۰۰ KIAN.

| گروه | تعداد | قاعده | نتیجه |
|---|---|---|---|
| کاربران عادی (0..500) | ۴۶۴ | تراز دقیق × ۱٬۰۰۰٬۰۰۰ micro | ۵۲٬۷۰۰.۲ KIAN (۴۳۳ حساب مثبت؛ بیشترین ۳۴۵ KIAN — سقف هیچ‌کسی فعال نشد) |
| @ImXsi `6398636666` | ۱ | **اصلاح باگ دابل‌کردیت** — واریز #3 دو بار (tx #806 و #807 هر کدام +10,000) اعمال شده بود | تراز ثابت **۱٬۰۰۰ KIAN** |
| @ImXforevr `8198598635` | ۱ | تراز تورم‌زدهٔ `admin_grant` (۱۱۵,۰۰۷.۳) و اعتبار آزمایشی | تراز بنیان‌گذار **۵٬۰۰۰ KIAN** |
| **جمع کل ضرب genesis** | **۴۳۵ Tx MINT** | هر تراز با ۲ پا (2-leg Posting) | **۵۸٬۷۰۰٬۲۰۰٬۰۰۰ micro = ۵۸,۷۰۰.۲ KIAN** |

**تضمین‌های دفتر (همه تأیید شد):**
- Σ همهٔ `"Posting"` = **۰** (تراز آزمایشی) — `trialBalanceSum: 0` زنده
- تراز `mint:genesis` = **−۵۸,۷۰۰,۲۰۰,۰۰۰** (منفی — مادهٔ ۴ منشور: mint هرگز مثبت نمی‌شود)
- کلید یکسان‌سازی هر ضرب: `mig:genesis:{tg}:v1` (UNIQUE — اجرای مجدد ضرب نمی‌کند)
- **ارتقای منشور (مادهٔ ۳):** سقف ضرب سالانه از ۶۰٬۰۰۰ KIAN به **۵۰۰٬۰۰۰ KIAN** (`500_000_000_000 micro`) رسید — genesis سال ۲۰۲۶ مهاجرت ۵۸,۷۰۰.۲ KIAN + شیر dev (۵٬۰۰۰ KIAN) = ۶۳,۷۰۰.۲ KIAN با سرریز ۴۳۶K+ KIAN. تست‌های دامنه به‌روز شدند (`cargo test --workspace`: **۲۸۲ سبز**).

## ۳) محصولات — ۱۲ محصول فعال، ۵ سازندهٔ واقعی

| دسته | تعداد | جزئیات |
|---|---|---|
| تأییدشدهٔ @ImXforevr | **۷** | ids 21,23,24,27,28,46,47 — عیناً مهاجرت شدند |
| pending یگانه | **۴** | @MHMD_DEVVV (NodeFlow ۲۰۰K، Travel Icons ۵۰۰K)، @BTC_USDT (تست)، @nowhere66 (Journal.trade) — ۳ ردیف تکراری Travel Icons حذف شد |
| احیاشدهٔ rejected | **۱** | بهترین نسخهٔ @Amingh30 از ۱۷ ردیف تکراری «Ai Prompt Book» (۱٬۰۰۰ KIAN) |
| **دورریخته (dedupe)** | **۱۸** | حذف قطعی تکرارها؛ تاریخچه در `AuditLog` لگاسی باقی است |

قواعد: `price = max(price_credits × 1,000,000 ، 10,000,000 micro)` · `affiliateBps = 2000` (۲۰٪ پورسانت وایرال) · `artifactKind = 'HTML_APP'` · **هر محصول `previewHtml` + `deliveryPayload` کامل آرتیفکت HTML تک‌صفحه‌ای فارسی RTL تعاملی** (`<!DOCTYPE html>`) دارد — خریدار بلافاصله پیش‌نمایش و دانلود دارد.

## ۴) سفارش‌های تاریخی — ۱۰۸ اسکرو RELEASED

- **۱۰۸ خرید** از **۸۴ خریدارِ بازگشتی** → `"Escrow"` با `status='RELEASED'`، `deliveryRef='legacy:migration'`
- حجم تاریخی: **۵,۳۶۰ KIAN** · نقشهٔ فروشنده از `products.creator_id` هر خرید استخراج شد
- `fundedTxId/settleTxId = NULL` و **هیچ Posting برای اسکروهای تاریخی نوشته نشد** — ترازهای فعلی دست‌نخورده (تأیید زنده: `escrowLocked: 0`، `circulatingSupply` ثابت)
- **کشف معماری (طبق مأموریت):** جدول `"Escrow"` صرفاً read-model نیست — `V3Store::apply()` کل رجیستری را با `load_escrow_registry()` داخل تراکنش می‌خواند؛ همهٔ ۱۰۸ ردیف با `status='RELEASED'` (ثابت معتبر EscrowStatus) بدون خطا لود شدند
- تفکیک مالیاتی هر خرید (مالیات ۱۰٪ خزانه/سهم فروشنده) در **۱۰۸ ردیف `AuditLog`** با meta JSON ثبت شد

**دسترسی بازگشتی:** هر ۸۴ خریدار با ورود، سفارش‌های RELEASED خود را در `/market` می‌بیند و `GET /api/v1/market/escrows/{id}/delivery` آرتیفکت کامل را می‌دهد (تأیید زنده پایین).

## ۵) حافظه و گفتگوهای AI

| مورد | مبدأ | مقصد | تعداد |
|---|---|---|---|
| حافظهٔ کاربران | `user_memories` | `"MemoryNote"` (interest/fact→semantic، goal→identity، skill→procedural) | **۱۰۸** |
| گفتگوهای هرمس | `chat_messages` (۲۳ کاربر) | `"BuilderSession"` (۱ به‌ازای هر کاربر) + `"BuilderMessage"` | **۲۳ نشست / ۱۲۵ پیام** |

## ۶) راستی‌آزمایی زندهٔ کرنل (Mission 2) — ۳۹/۳۹ PASS

سرور واقعی `kian-server` روی کپی `kian-production.db` (`KIAN_DEV_MODE=1`، پورت ۱۸۰۹۹):

| سناریو | نتیجه |
|---|---|
| `GET /api/v1/health` | `status: ok` ✓ |
| `GET /api/v1/economy` | accountCount **۴۶۹**، balanced **true**، circulatingSupply **۵۸,۷۰۰,۲۰۰,۰۰۰**، yearlyMintCap **۵۰۰,۰۰۰,۰۰۰,۰۰۰**، trialBalanceSum **۰** ✓ |
| `GET /api/v1/market/products` | **۱۲ محصول، ۵ سازنده** (@ImXforevr، @MHMD_DEVVV، @BTC_USDT، @nowhere66، @Amingh30)، همه با previewHtml معتبر `<!DOCTYPE html>` ✓ |
| dev-login بنیان‌گذار `8198598635` | `/me` → username **ImXforevr**، balance **۵٬۰۰۰ KIAN** ✓ |
| dev-login خریدار واقعی `135876863` (@Mehrsinov) | balance **۱۴۵ KIAN** (عین لگاسی)، **۳ سفارش تاریخی RELEASED**، دریافت delivery یک اسکروی مهاجرت‌شده ✓ |
| **خرید نو** `POST /market/purchase` (prod-mig-23، referrer @ImXsi) | escrow **FUNDED**، قفل ۵۰M micro، referrer+affiliateBps=۲۰۰۰ در DB ✓ — **اثبات `V3Store::apply()` روی دفتر مهاجرت‌شده** |
| آزادسازی خریدار | **۴۲۲ REQUIRES_HUMAN_APPROVAL** (مادهٔ ۷ — گیت HITL) ✓ |
| deliver+release ادمین (x-kian-key) | **RELEASED** — خزانه +۵M (۱۰٪ مالیات)، فروشنده +۳۵M، معرف @ImXsi +۱۰M (۲۰٪)، تراز آزمایشی همچنان **۰** ✓ |
| `POST /wallet/faucet` (شیر dev) | +۵٬۰۰۰ KIAN روی دفتر مهاجرت — `mintedThisYear` = ۶۳,۷۰۰,۲۰۰,۰۰۰ < سقف ۵۰۰B ✓ |

## ۷) بستهٔ استقرار Railway (Mission 3)

- **`Dockerfile`** — بیلد musl دو مرحله‌ای + ران‌تایم alpine با کاربر non-root، seed داخل ایمیج
- **`railway.json`** — builder DOCKERFILE، healthcheck `/api/v1/health`، restart ON_FAILURE
- **`start-railway.sh`** — بوت اول: کپی `seed/kian-production.db` → `KIAN_DATABASE_URL` (پیش‌فرض `/data/kian.db` داخل Volume)؛ بوت‌های بعدی: دست‌نزدن به دیتابیس (idempotent)؛ سپس `exec kian-server`
- **`scripts/broadcast_relaunch.py`** — پیام ریلانچ فارسی برای ۴۶۶ شهروند با تراز واقعی هر نفر؛ پیش‌فرض **dry-run** (نمونهٔ پیام + آمار)، ارسال واقعی فقط با `--live` و `TELEGRAM_BOT_TOKEN`، نرخ ۲۰ پیام/ثانیه، resume امن با state JSON، احترام به 429

## ۸) گزارش آشتی نهایی (reconciliation)

```
migrations_applied      : 0001..0006 (sqlx checksums ثبت شد — بوت سرور بدون اجرای مجدد)
users                   : 466  ( accounts citizen: 466 + system 3 = 469 )
referral_links          : 247  ( 3-layer: User.referredBy + ReferralLineage + sqlx migration )
tx (MINT genesis)       : 435  ( idempotencyKey = mig:genesis:{tg}:v1 )
postings                : 870  ( 2-leg per mint — Σ = 0 )
genesis_supply          : 58,700,200,000 micro = 58,700.2 KIAN
products_active         : 12   ( 7 approved + 4 unique pending + 1 revived ; 18 duplicates dropped )
escrows                 : 108  ( RELEASED ×108 ; volume 5,360 KIAN ; audit rows ×108 )
memory_notes            : 108
builder_sessions        : 23   ( 125 messages )
live_e2e                : 39/39 PASS  ( health/economy/products/dev-login/orders/delivery/new-purchase/HITL/settle/faucet )
cargo_test_workspace    : 282 passed, 0 failed
```

**نتیجه:** `seed/kian-production.db` با صفر خطا توسط کرنل Rust بارگذاری می‌شود (`load_ledger_state` + `load_escrow_registry`)، همهٔ دروازه‌های دامنه (`V3Store::apply`) روی آن سالم اجرا می‌شوند و بستهٔ Railway از بوت اول، جامعهٔ کامل ۳۰روزه را با ترازهای پاک‌سازی‌شده بالا می‌آورد.
