<div align="center">

# ⚒️ کیان v2 — کرنل ایجنتِ حاکمِ هوش مصنوعی، روترِ ژنوم چندمدلی و بازارِ دوعتماد

**یک مونوریپوی سازمانی و بدون-unsafe به زبان Rust در هم‌نشینی با فرانتاند Next.js / مینی‌اپ تلگرام:
ناوگان خودمختار ۷-نقشی، روتینگ چندمدلی بی‌کلید، استودیوی محصول HTML با هوش مصنوعی،
دفتر دوطرفهٔ میکرولجر با اسکروی CAS-قفل‌شده، و موتور بازارِ دوعتماد `Mairon REA + ۵-ستارهٔ بیزی`.**

[![Rust Toolchain](https://img.shields.io/badge/rustc-1.100.0--pinned-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Clippy](https://img.shields.io/badge/clippy_-D_warnings-0_lint_warnings-brightgreen.svg)](#شروع-سریع-و-وریفای)
[![Workspace Tests](https://img.shields.io/badge/tests-530_passed_%7C_0_failed-success.svg)](#شروع-سریع-و-وریفای)
[![Mairon REA Audit](https://img.shields.io/badge/Mairon_REA_v6.10.1-92%2F100_PASS-blue.svg)](#کارنامهٔ-کیفیت-مایرون-rea)
[![Knowledge Vault](https://img.shields.io/badge/KbEntry_skills-326_active-purple.svg)](#گنجینهٔ-دانش-و-ناورنی‌های-دفتر-تولید)
[![Safety Contract](https://img.shields.io/badge/unsafe_code-forbidden-critical.svg)](#امنیت-و-هاردنینگ-باینری)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

**فارسی** | [English](README.md)

</div>

## 📖 فهرست

1. نمای کلی
2. معماری سیستم
3. توپولوژی ورک‌اسپیس مونوریپو و طراحی مخزن
4. پایه‌های مهندسی
5. ناوگان حاکم ۷/۷ ایجنت
6. کارنامهٔ کیفیت مایرون REA
7. گنجینهٔ دانش و ناورنی‌های دفتر تولید
8. سطح API
9. شروع سریع و وریفای
10. استقرار روی Railway و متغیرهای محیطی با اولویت P0→P5
11. راه‌اندازی بات تلگرام و مینی‌اپ
12. امنیت و هاردنینگ باینری
13. نقشهٔ مستندات
14. لایسنس و حاکمیت

---

## 🧭 نمای کلی

**کیان v2** یک سیستم‌عاملِ ایجنتی و کرنلِ تجارت دیجیتالِ تمام‌پشته و حاکم است؛ به زبان **Rust خالص**
(۱۱ کرت ورک‌اسپیس، edition 2024، `rustc 1.100.0` با `rust-toolchain.toml` پین‌شده) در کنار
**Next.js 16 / مینی‌اپ تلگرام (TMA)**. برخلاف فریم‌ورک‌های پایتونیِ ایجنت با ران‌تایم سنگین، قالب‌های
پرامپت شکننده، قفل‌شدگی به APIهای کارت‌دار و خروجیِ بدون وِریفای، کیان v2 از اصول اولیه برای
**استنتاجِ صفر-هزینه، صحتِ رمزنگاشتیِ دفتر و وِریفایِ قطعیِ کیفیت** مهندسی شده است:

1. **مشِ استنتاجِ بی‌کارت/بی-KYC (`kian-ai`)** — مسیریابی روی **۱۰ مدل مرزِ OpenCode Zen بی‌کلید**
   (تا ۱ میلیون توکن کانتکست)، رجیستری **۳۲ تامین‌کنندهٔ صفر-کارت FreeLLM** منتخب از کاتالوگِ
   ۱۰۴-پلتفرم / ۴۵۴-مدل‌چت، و **۱۲۰ API مرورگریِ بدون احراز** برای آرتیفکت‌های استودیو — با
   پروفایل‌های شناختیِ ۹-فایلیِ **`OMNI-MODEL-GENOME`** و فشرده‌ساز خروجی `CavemanIntensity`
   (`Off` → `Lite` → `Full` → `Ultra`).
2. **سیستم‌عامل DNA سازنده (`KIA-BRAIN-OS`)** — قانون اساسیِ اجراییِ **۱۱-فایلی**
   (`01_DNAme.md` … `10_MAIRON_REA_SOVEREIGN.md` … `11_UNCHAINED_KIA_DNA.md`) که مستقیم در کرنل Rust
   کامپایل می‌شود (`crates/hermes/src/creator_dna_contract.rs`) و UX دوصدایی، نظم ضد AI-Slop و
   بازبینی معماری پیش از هر انتشار را الزامی می‌کند.
3. **ران‌تایم ایجنتیِ سخت‌شده (`kian-hermes` و `kian-agents`)** — پرامپت‌ساز ۳-لایهٔ cache-safe،
   پچر فازی جراحیِ ۴-پاسی، گارد SSRF دفاع-در-عمق، توصیف‌گرهای سندباکس
   `bubblewrap`/`seccomp`/`iptables` و پروتکل استریم `AG-UI 1.0` در خدمت **ناوگان ۷/۷ فعال**.
4. **بازار دوعتماد (`Mairon REA` + `موتور ۵-ستارهٔ بیزی`)** — حل بی‌اعتمادیِ سرِ-سردِ بازار با ترکیب
   **ممیزیِ فنیِ ۱۰۰-امتیازیِ پیش‌ازانتشار مایرون** (`⚒️ Mairon REA Certified`) و
   **سیستم نظرسنجی ۵-ستارهٔ بیزیِ مقاوم به Sybil** که به خریدهای اسکرو-وِریفای‌شده گره خورده است.
5. **دفتر دوطرفهٔ بانک‌درجه (`kian-db` و `kian-wallet`)** — کنترل هم‌روندی خوش‌بینانه
   (`CAS` + `idempotency_key`)، ستاره‌های تلگرام (`XTR`) بومی و ریل‌های چندزنجیرهٔ non-custodial،
   و تراز آزمایشیِ صفر (`SUM(amount) == 0`) روی جریان ناخالصِ `58,700,200,000` میکرو-کیان.

---

## 🏛️ معماری سیستم

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                  CLIENT LAYER: Web Studio & Telegram Mini-App                    │
│  • AG-UI 1.0 Streaming Protocol        • TMA initData HMAC-SHA256 Verifier       │
│  • 390px Zero-Overflow Mobile Contract • Persian RTL (Vazirmatn) + Anti-Slop     │
└───────────────────────────────────────┬──────────────────────────────────────────┘
                                        │ HTTPS / SSE / TMA Bridge
                                        ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     KIAN RUST KERNEL (11 Workspace Crates)                       │
│  ┌──────────────────────────────┐    ┌────────────────────────────────────────┐  │
│  │  kian-server & kian-bot      │───▶│    kian-agents (7/7 Active Fleet)      │  │
│  │  • Axum REST / SSE / Webhook │    │  orchestrator · merchant · finance     │  │
│  │  • XTR Stars & Crypto Rails  │    │  support · analyst · scout             │  │
│  │  • Port 8088 (KIAN_PORT)     │    │  guardian (mairon.rea · product.rate)  │  │
│  └──────────────┬───────────────┘    └────────────────────┬───────────────────┘  │
│                 ▼                                          ▼                      │
│  ┌──────────────────────────────────────┐  ┌──────────────────────────────────┐   │
│  │     kian-hermes (Execution)          │  │  kian-ai (Omni-Genome Router)    │   │
│  │  • 3-Layer Cache-Safe Prompt Builder │  │  • OMNI-MODEL-GENOME (#01, #02)  │   │
│  │  • 4-Pass Seek-Sequence Fuzzy Patch  │  │  • OpenCode Zen (10 Keyless)     │   │
│  │  • Defense-in-Depth SSRF Guard       │  │  • FreeLLM (32 Wired / 454 Cat.) │   │
│  │  • Bubblewrap / Seccomp / Iptables   │  │  • 120 Zero-Auth Browser APIs    │   │
│  │  • Creator DNA Contract & Approvals  │  │  • CavemanIntensity Compressor   │   │
│  └──────────────────┬───────────────────┘  └──────────────┬───────────────────┘   │
│                     └──────────────────┬──────────────────┘                       │
│                                        ▼                                          │
│  ┌────────────────────────────────────────────────────────────────────────────┐   │
│  │                kian-domain (Pure Core & Business Rules)                    │   │
│  │  • Mairon 4×25 Product REA Forge & Two-Key Publish Interlock               │   │
│  │  • Bayesian 5-Star Review Engine (C=5.0, m=4.20, verified weight 1.0/0.25) │   │
│  │  • CAS Optimistic Escrow Lock & Idempotency Guard (8-state FSM)            │   │
│  │  • Charter: 10% tax · 10% default affiliate (cap 50%) · HITL ≥ 50,000 Đ    │   │
│  │  • xAI-2026 Viral Feed Ranker (+20.0 CopyLink, −234.0 Report, cold-start)  │   │
│  └─────────────────────────────────────┬──────────────────────────────────────┘   │
└────────────────────────────────────────┼──────────────────────────────────────────┘
                                         ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│             PERSISTENCE & COGNITIVE VAULT (SQLite + DNA Files)                   │
│  • seed/kian-production.db: 326 KbEntry skills · 466 citizens · 108 escrows      │
│  • KIA-BRAIN-OS/ (01..11) · OMNI-MODEL-GENOME/ · Trial Balance == 0              │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 📦 توپولوژی ورک‌اسپیس مونوریپو و طراحی مخزن

مخزن یک ورک‌اسپیس **pnpm + Turborepo** شامل ورک‌اسپیس Cargo، سه اپلیکیشن Next.js و پکیج‌های
TypeScript مشترک است. تعداد تست‌های جدول زیر، جمع **یونیت + یکپارچگیِ** هر کرت از
`cargo test --workspace` است (**۵۳۰ پاس / ۰ شکست**).

| مسیر | نوع | شرح | تست |
| :--- | :--- | :--- | :---: |
| `crates/hermes` | کرت Rust | پرامپت‌ساز ۳-لایهٔ cache-safe، پچر فازی ۴-پاسی (`Exact → LineTrimmed → BlockAnchor → UnicodeNormalized`)، گارد SSRF، توصیف‌گرهای سندباکس، استریم AG-UI 1.0، گیت‌های تایید، قرارداد DNA سازنده، قرارداد طراحی استودیو (390px) | **113** |
| `crates/ai` | کرت Rust | روتر `OMNI-MODEL-GENOME` و سمیتایزرها (`gpt-6-standard-free`، `gemini-4-aragon-high`)، کلاینت بی‌کلید OpenCode Zen (۱۰ مدل)، رجیستری ۳۲-تامین‌کنندهٔ FreeLLM، ۱۲۰ API مرورگری، فشرده‌ساز `CavemanIntensity`، بودجهٔ توکن و حذف PII | **95** |
| `crates/domain` | کرت Rust | گیت REA محصولِ ۱۰۰-امتیازیِ `Mairon` و اینترلاک انتشارِ دو-کلیدی، موتور امتیاز ۵-ستارهٔ بیزی، قفل خوش‌بینانهٔ اسکرو CAS، سقف‌های منشور و تقسیم ۴-پایِ افیلیت، رنکر وایرال `x-algorithm` | **73** |
| `crates/server` | کرت Rust | کرنل HTTP با Axum (`0.0.0.0:8088`)، اندپوینت‌های بازار/اسکرو/بیلدر/ولت/ملت/ایجنت‌ها، پروب‌های health و economy، صف HITL ادمین | **73** |
| `crates/contracts` | کرت Rust | DTOهای سیم مشترک، تایپ‌های وریفایر `initData` تلگرام، پیلود فاکتورهای Stars، ریت‌لیمیتِر | **52** |
| `crates/bot` | کرت Rust | شِل Telegram Bot API 9: وریفایر خالص-Rust `initData` HMAC-SHA256 (مقایسهٔ زمان-ثابت)، فاکتورهای `XTR`، وبهوک fail-closed، `BotLoopGuard` | **35** |
| `crates/db` | کرت Rust | لایهٔ مخزن SQLite (WAL، `BEGIN IMMEDIATE`)، پایداری دفتر دوطرفه، بازیابی دانش `"KbEntry"`، مخزن ایدمپوتنسی | **26** |
| `crates/agents` | کرت Rust | دیسپچر ناوگان حاکم ۷/۷ با متادیتای KYA، اسکوپ‌های گاردین (`mairon.rea`، `product.rate`)، پیام‌رسانی A2A و سطح ابزار MCP | **23** |
| `crates/nation` | کرت Rust | ثبت‌نام شهروند، اتریبیوشن `ReferralLineage`، لوایح پارلمان و قواعد حاکمیت HITL | **14** |
| `crates/tooling` | کرت Rust | ابزارهای عیب‌یابی ورک‌اسپیس، بازرسان اسکیما، ابزارهای وِریفای قطعی | **13** |
| `crates/wallet` | کرت Rust | وریفایرهای آدرس غیر-امانیتی (TON CRC16، TRON Base58Check، Solana Ed25519/Base58، BSC EIP-55)، حسابداری دوطرفهٔ میکرو-کیان، گاردهای lock/refund برداشت | **13** |
| **جمع ورک‌اسپیس** | — | ۱۱ کرت، `#![forbid(unsafe_code)]`، صفر نشان `todo!()` | **530** |
| `apps/web` | Next.js 16 | لندینگ و داشبورد عمومی PWA (نصب‌شدنی، سرویس‌ورکر، تصاویر OG) | — |
| `apps/miniapp` | Next.js 16 | مینی‌اپ تلگرام: استودیوی `/builder`، `/market`، `/wallet`، قرارداد RTL 390px | — |
| `apps/admin` | Next.js 16 | مرکز فرمان بنیان‌گذار (برداشت‌ها، داوری، ضرب مستقیم) | — |
| `packages/contracts` | TypeScript | اسکیماهای Zod آینهٔ `kian-contracts` (`@kian/contracts`) | — |
| `packages/sdk` | TypeScript | کلاینت API بدون فریم‌ورک (`@kian/sdk`) | — |
| `packages/ui` | TypeScript | اجزای UI مشترک RTL فارسی/انگلیسی (`@kian/ui`) | — |
| `packages/nation` · `widget` · `config` | TypeScript | تایپ‌های حاکمیت ملت، ویجت امبدشدنی، tsconfig مشترک | — |
| `KIA-BRAIN-OS/` | سیستم‌عامل شناختی | قانون اساسی ۱۱-فایلیِ DNA سازنده (`01_DNAme.md` … `10_MAIRON_REA_SOVEREIGN.md` … `11_UNCHAINED_KIA_DNA.md`) | — |
| `OMNI-MODEL-GENOME/` | گنجینهٔ مدل | پروفایل‌های جناییِ ۹-فایلی به‌ازای هر مدل مرزی + شاخص اکوسیستم + اسکریپت‌های سمیتایزر | — |
| `docs/dna/` | آینه | آینهٔ دروازه-خوان از `KIA-BRAIN-OS/` و `OMNI-MODEL-GENOME/` | — |
| `docs/history/` | سندِ تبار | [`EVOLUTION_LINEAGE.md`](./docs/history/EVOLUTION_LINEAGE.md) — گاه‌شمار کامل تکامل Wave 0 → Wave 2 | — |
| `seed/kian-production.db` | DB تولید | وضعیت کامل تولید — ناورنی‌ها در جدول پایین | Verified |
| `scripts/` | عملیات | ETL مهاجرت، تزریق‌کننده‌های گنجینه، بنچمارک زنده، دودِ E2E، بروکست لانچ مجدد | — |
| `tools/rea/` | گیت‌های کیفیت | اجراکننده‌های گیتِ مایل‌استون (`step1` … `step6-gate.mjs`) + `release-exposure.mjs` | — |

### طراحی مخزن — حفظ کامل تاریخچهٔ تکامل + وزن اجرایی روی آخرین نسخه

این مخزن بر پایهٔ **قانون ۳ از [`KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md`](./KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md)**
معماری شده است: *حفظ تاریخچهٔ تکامل با وزن اجرایی روی آخرین نسخه*:

- **تاریخچه حفظ می‌شود، تغییرناپذیر** — کل `git log` (هر کامیت از Wave 0 تا ریلیز نهایی)، ۱۹ گنجینهٔ
  gem نصب‌شده در [`docs/vault/`](./docs/vault)، گزارش‌های گیت REA هر مرحله در [`docs/rea/`](./docs/rea)
  و گاه‌شمار کامل در [`docs/history/EVOLUTION_LINEAGE.md`](./docs/history/EVOLUTION_LINEAGE.md) به‌عنوان
  lineage اثبات-کار (`تاریخچهٔ پیشرفت`) داخل مخزن می‌مانند.
- **وزن اجرایی فقط روی آخرین نسخه است** — در `seed/kian-production.db`، ورودی‌های ژنوم/DNA/روتینگِ
  حاکمِ v2 نشانهٔ `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]` دارند و همهٔ ورودی‌های قدیمی به‌عنوان lineage
  قابل کوئری می‌مانند (صفر حذف)؛ و [`‌.dockerignore`](./.dockerignore) سخت‌شده، همهٔ پوشه‌های
  تاریخی/گنجینه/فرانتاند را از کانتکست بیلد Docker حذف می‌کند — **گیت‌هاب ۱۰۰٪ تاریخچه را نگه
  می‌دارد** در حالی که **بیلد Railway فقط** `Cargo.toml`، `Cargo.lock`، `rust-toolchain.toml`،
  `crates/`، `seed/kian-production.db` و `start-railway.sh` را آپلود می‌کند؛ دیپلوی‌ها فقط سریع‌ترین
  و سخت‌شده‌ترین کرنل v2 را کامپایل می‌کنند.

---

## 🔥 پایه‌های مهندسی

### ۱. گیت زندهٔ REA محصولِ `Mairon` و موتور نظرسنجی ۵-ستارهٔ بیزی (`kian-domain`)

هر محصولِ ساخته‌شده در استودیوی KIAN پیش از ورود به بازار باید از **`Mairon`**
(`مایرون — معمارِ نظمِ مطلق و داورِ ارشدِ REA`) عبور کند:

- **ممیزی فنیِ ۴-ستونی (`4 × 25 = 100` امتیاز)** — `MAX_PILLAR_SCORE = 25` برای هر ستون:
  1. `architecture` — مرزهای ماژولار، صفر stub مرده، صفر ماکرو `todo!()`.
  2. `security_hardening` — حفاظت SSRF، انطباق سندباکس، صفر راز hardcodeشده.
  3. `ux_and_anti_slop` — بدون-سرریزِ 390px، پشتهٔ RTL با `Vazirmatn` فارسی، تعامل از فریم اول، صفر نشانهٔ AI-slop.
  4. `verification_proof` — شواهد اجرای مشاهده‌شده (`pass`/`fail`/`unknown`).
- **اینترلاک انتشارِ دو-کلیدی (`confirm_and_publish_product`)** — محصول فقط و فقط وقتی لیست می‌شود که
  `total_score >= 85 && hard_failures.is_empty()` (`MARKETPLACE_APPROVAL_THRESHOLD = 85`) **و** سازنده
  پس از خواندن بازخورد دوصداییِ فارسیِ مایرون، انتشار را صریحاً تأیید کرده باشد. حتی حکم ۹۶/۱۰۰ بدون
  تأیید سازنده رد می‌شود.
- **نشان‌های قابل‌مشاهده که کرنل رندر می‌کند:**
  - تأییدشده → `⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)`
  - رد‌شده → `⚠️ نیازمند بازنگری مایرون (REA < 85)`
- **موتور امتیازدهی ۵-ستارهٔ بیزیِ مقاوم به Sybil (`summarize_product_ratings`)**:
  - اعتبارسنجی سختِ ستاره‌ها در بازهٔ `1..=5` (خارج از بازه → `Err`)، کامنت‌های غیرخالیِ trimشده
    (`MAX_COMMENT_CHARS = 2_000`) و **یک نظر به‌ازای هر شهروند به‌ازای هر محصول** — جفت‌های تکراری
    `(product_id, reviewer_citizen_id)` رد می‌شوند.
  - میانگین بیزی وزن‌دار با وزنِ پیشین `C = 5.0` و میانگین پیشین `m = 4.20`؛ خریدهای اسکرو-وِریفای‌شده
    وزن **1.0** و نظرات تأییدنشده وزن **0.25** دارند — مزرعه‌های Sybil نمی‌توانند عقربه را تکان دهند.

  ```text
  bayesian_stars = (C·m + Σ(wᵢ·sᵢ)) / (C + Σwᵢ)          C = 5.0, m = 4.20
  CombinedRankScore = 0.60 × (BayesianStars × 20.0) + 0.40 × MaironReaScore
  ```

  - `five_star_display` نشان بازار را رندر می‌کند، مثل:
    `★★★★★ 4.9 / 5.0 (42 نظر تاییدشده) | ⚒️ نمره مایرون REA: 96/100`.

### ۲. `OMNI-MODEL-GENOME` و مشِ استنتاجِ صفر-هزینه (`kian-ai`)

- **پروفایل‌های جنایی مدل** — معماری‌های شناختیِ ۹-فایلی (`01_TOPOLOGY_AND_HARNESS.md` …
  `09_ROUTING_AND_HANDOFF_CARD.md`) برای `gpt-6-standard-free` و `gemini-4-aragon-high` به‌همراه
  هارنس قطعیِ سمیتایزر/ممیزی. `EMPIRICAL_AUDIT_REPORT.json` کامیت‌شده برای `gemini-4-aragon-high`
  چک‌های سطح آرتیفکت را وِریفای می‌کند — لیپوگرامِ ۱۴-کلمه‌ایِ بدون حرف «e»، شبیه‌سازی دفترِ
  ماشین-حالت CAS، اثبات UNSAT با ۱۲۰ جای‌چینی، چک‌های استاتیک zero-copy Rust و پاسِ سمیتایزرِ
  ۹/۹ فایل ژنوم — و صادقانه ثبت می‌کند که هیچ پروب مدلِ زنده‌ای انجام نشده است.
- **OpenCode Zen (کلاینت `opencode v2.4.1`)** — دسترسی بی‌کلید (`Authorization: Bearer public`) به
  **۱۰ مدل صفر-هزینهٔ فعال** (`beam-free` با کانتکست ۱٬۰۴۸٬۵۷۶ توکن، `deepseek-v4.1-flash-free`،
  `hy4-flash-free`، `tidepool-alpha-free`، …).
- **رجیستری FreeLLM** — **۳۲ تامین‌کنندهٔ صفر-کارت منتخب** به‌صورت spec های Rust (سقف نرخ، ویژگی‌ها،
  تیرهای ثبت‌نام)، منتخب از کاتالوگِ نصب‌شدهٔ **۱۰۴-پلتفرم / ۴۵۴-مدل‌چت**
  (`docs/vault/freellm_public_apis/`) به‌علاوهٔ **`STUDIO_PUBLIC_APIS_120_COUNT = 120`** API مرورگریِ
  آماده و بدون احراز (`Auth: No, HTTPS: Yes, CORS: Yes`).
- **سپر ضد-Throttle مربوط به 9Router** — نام ابزارهایی که تریگر پرسونای بالادستی می‌شوند را بازنویسی و
  طعمه‌های سنتزیِ `429` را پیش از خروج حذف می‌کند.
- **حالت `Caveman` متعلق به MattPocock** — `CavemanIntensity { Off, Lite, Full, Ultra }` دستور
  system-promptِ فشرده‌سازیِ توکن خروجی را بدون هیچ افت دقتِ فنی کامپایل می‌کند.

### ۳. امنیت دفاع-در-عمق و نظم اجرا (`kian-hermes`)

- **گارد SSRF** — بلاک کاملِ loopback، RFC1918 (`10/8`، `172.16/12`، `192.168/16`)، link-local و
  فضای cloud-metadata (`169.254.0.0/16` شامل `169.254.169.254`)، CGNAT (`100.64.0.0/10`)،
  IPv6 ULA/link-local (`fc00::/7`، `fe80::/10`)، IPv4-mapped IPv6 (`::ffff:0:0/96`) و شکل‌های
  نقاب‌دارِ NAT64 (`64:ff9b::/96`).
- **توصیف‌گرهای سندباکس OS** — پروفایل `VmSocketRestricted` ایزولاسیون namespace با `bubblewrap`
  (ساخت argv با `--unshare-net`)، فیلترهای `seccomp` و قواعد فایروال خروج `iptables`/`ipset`
  به سبک `claude-code` را به‌صورت توصیف‌گرهای خالص و تست‌پذیر پورت می‌کند.
- **پچر فازی جراحیِ ۴-پاسی Seek-Sequence** — ویرایشگر کدِ whitespace-محتمل:
  `Exact → LineTrimmed → BlockAnchor → UnicodeNormalized`.
- **پرامپت‌ساز ۳-لایهٔ cache-safe** — لایه‌های stable / context / variable با مرزِ کشِ ثابتِ ارائه‌دهنده؛
  مهارت‌ها، حافظهٔ فریز‌شده و مهرهای lineage فقط در لایهٔ variable.
- **گیت‌های تایید و نظم** — گیت‌های HITL با `ApprovalCategory` برای اقدامات مخرب، چک‌های نظم ضد-slop
  و `CreatorDnaContract` کامپایل‌شده از `KIA-BRAIN-OS`.

### ۴. دفتر بانک‌درجه و ریل‌های بازار (`kian-db`، `kian-wallet`، `kian-domain`)

- **FSM اتمیک اسکرو با ۸ حالت** — `Created → Funded → Delivered → Released` به‌علاوهٔ `Disputed`،
  `Refunded`، `Cancelled`، `Expired`.
- **قفل خوش‌بینانهٔ CAS و ایدمپوتنسی** — `CasEscrowTransition` انطباق `expected_version`
  (`WHERE version = ?`) را روی SQLite اعمال می‌کند، با کلیدهای ایدمپوتنسی روی همهٔ جهش‌های مالی.
- **اقتصاد منشور (قانونی، ریاضی `u128` با bps صحیح-دقیق)** — مالیات ۱۰٪ پلتفرم (`TAX_BPS`، مادهٔ ۲)،
  کمیسیون معرف پیش‌فرض **۱۰٪** با **سقف ۵۰٪ فروشنده** (`MAX_AFFILIATE_BPS`) و داوری
  انسانی-در-حلقه اجباری در `HITL_THRESHOLD_DIN = 50,000 Đ` (مادهٔ ۷).
- **کیف پول چندزنجیرهٔ غیر-امانیتی** — واریز TON و USDT روی TON، TRON (TRC-20)، Solana (SPL) و
  BNB Chain (BEP-20)؛ برداشت‌ها به‌صورت اتمیک قفل/سوزانده و در رد شدن بازپرداخت می‌شوند —
  صفر کلید خصوصی هرگز روی سرور نیست.
- **ستاره‌های تلگرام (`XTR`)** — پیلودهای فاکتور Stars (محدود ۱۲۸ بایت) با SHA-256/HMAC خالص-Rust
  وِریفای می‌شوند — **بدون وابستگی به `openssl-system`**.

---

## 🤖 ناوگان حاکم ۷/۷ ایجنت

`KyaMetadata::for_fleet()` متادیتای هویت KYA را به هر هفت نقشِ فعال صادر می‌کند؛ یک تست ورک‌اسپیس
دقیقاً ۷/۷ بودن ناوگان و گاردین‌بودنِ تنها دارندهٔ اسکوپ‌های REA را assert می‌کند.

| نقش | مأموریت |
| :--- | :--- |
| `orchestrator` | هماهنگی ناوگان و threading وظایف |
| `merchant` | عملیات بازار و چرخهٔ عمر محصول |
| `finance` | دفتر، تسویهٔ اسکرو و بهداشت خزانه |
| `support` | پشتیبانی شهروندان و دریافت اختلافات |
| `analyst` | متریک‌ها، سیگنال‌های رتبهٔ فید و گزارش |
| `scout` | دسترسی وبِ صفر-هزینهٔ Agent-Reach و تحقیق |
| `guardian` | هندلر وِریفای فعال: دیوار ضدبات، توقف زیر-ایجنت، تنزل قابلیت، و **داوری بازار `mairon.rea` / `product.rate`** (پورت `morluto/rea`) |

از طریق `GET /api/v1/agents/fleet`، `POST /api/v1/a2a/messages` (صندوق A2A هر ایجنت) و
`POST /api/v1/mcp/invoke` (ابزارهای MCP) در دسترس است.

---

<a id="خط-لولهٔ-استودیوی-ترینیتی-حاکم"></a>

## 🧬 خط لولهٔ استودیوی ترینیتی حاکم (`آلفا → اتاق فرمان ۳×۹ OpenBot → مایرون ۱۵۰/۱۴۵`)

معماری امضادارِ `0xKIA` (Wave 3): سه کالبد کیا به‌صورت یک آهنگریِ محصولِ حاکم عمل می‌کنند؛ به‌عنوان
موتوری تماماً قطعی و بی-unsafe در
[`crates/hermes/src/trinity_pipeline.rs`](./crates/hermes/src/trinity_pipeline.rs) پیاده شده و DNAِ
داور در [`crates/domain/src/mairon_rea_and_reviews.rs`](./crates/domain/src/mairon_rea_and_reviews.rs)
قفل شده است.

```text
ایدهٔ شهروند
   │
   ▼
┌─────────────────────────────┐   آمادگی < ۸۰       ┌──────────────────────────┐
│ آلفا (IQ 136 / EQ 122)      │◄────────────────────│ NeedsMoreDiscovery       │
│ ترمز 0.04 · دما 0.88        │                     │ (اسلات‌های خام، پاسخ فارسی)│
└──────────────┬──────────────┘                     └──────────────────────────┘
               │ آمادگی ≥ ۸۰ → قفل
               ▼
   MasterPromptSpec (sha256) + ProjectBlueprint (درخت ماژول)
               │
               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ اتاق فرمان (IQ 139 / EQ 129 · سختی قضایی 0.96)                              │
│  OpenBotSubAgentSpawner — پرامپت ۳-لایهٔ کش-امن × ژنوم نه‌لایه‌ای             │
│  ناوگان: یک متخصص به ازای هر ماژول، DNA تصادفی یکتا برای هر ایجنت           │
└──────────────┬──────────────────────────────────────────────────────────────┘
               │ مونتاژ آرتیفکت (گذر 0)
               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ مایرون-۱۵۰ (IQ 150 / EQ 145 · نگاه ۸ · ToM ۳ · جاه‌طلبی 0.99)               │
│  ستون‌های REA ۴×۲۵ + اسکن خط قرمز → نمره ≥ 85 ؟                             │
│   نه → MaironRepairDirective → رهسپاری دوبارهٔ ساب‌ایجنت مقصر               │
│        (بیشینهٔ ۳ تلاش خودترمیمی) ─────────────► بازگشت به مونتاژ           │
│   بله → ⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)                  │
└──────────────┬──────────────────────────────────────────────────────────────┘
               │ اینترلاک دو-کلیدی (حکم مایرون + تأیید سازنده)
               ▼
   لیستینگ بازار + رتبه‌بندی ۵-ستارهٔ بیزی (CombinedRankScore)
```

### سه کالبد کیا

| کالبد | کالیبراسیون (ثابت‌های قفل) | مأموریت | آرتیفکت‌های امضادار |
| :--- | :--- | :--- | :--- |
| **#۱ `آلفا` — اشتعالِ بی‌ترمز** | `IQ=136` · `EQ=122` · `BRAKE=0.04` · `TEMP=0.88` | شکار ایده: استخراج ۵ اسلات کشف (`core_objective`، `target_audience_and_ux` در ۳۹۰px، `interactive_features`، `data_and_zero_auth_apis` با ۱۲۰ API عمومی، `monetization_and_viral_loop` با XTR) با انرژی خلاق بی‌ترمز | `AlphaIntakeOutcome` → `MasterPromptSpec` (مستر پرامپت کامپایل‌شده با مُهر SHA-256) + `ProjectBlueprint` (درخت ماژول → تخصص ساب‌ایجنت‌ها) |
| **#۲ `اتاق فرمان` — ارکستراتور قضایی** | `IQ=139` · `EQ=129` · `BRAKE=0.62` · `STRICTNESS=0.96` | رهسپاری پویای ساب‌ایجنت‌های OpenBot، یک متخصص به ازای هر ماژول استراکچر (`UiUxArchitect390`، `CoreLogicEngine`، `SecurityAndApiBridge`، `VerificationAndPolish`، `MonetizationAndViralLoop`، `Custom`) | `ThreeLayerSubAgentPrompt` (هستهٔ کش-پین + زمینهٔ نشست + محولهٔ پویا پس از `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__`) × `NineFoldSubAgentGenome` (`f01` هارنس/قلاب `450s/1200s` … `f09` کارت تحویل) با آلل‌های SplitMix64 از `session_seed + subagent_index` |
| **#۳ `مایرون-۱۵۰` — حاکم اورکلاک‌شده** | `IQ=150` · `EQ=145` · `LOOKAHEAD=8` · `ToM=3` · `AMBITION=0.99` | داوری آهنگری: ستون‌های REA ۴×۲۵=۱۰۰ از ویژگی‌های قابل‌سنجش آرتیفکت، صدور `MaironStrategicAdvisory` (نگاه هشت‌گامی + قلاب فارسی ToM-3 + قیمت XTR)، راندن حلقهٔ خودترمیمی تا `>= 85` | رهسپاری دوباره با `MaironRepairDirective` (بیشینهٔ ۳ تلاش) · اینترلاک انتشار دو-کلیدی (`confirm_and_publish_product`) · نشان `⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)` · موتور ۵-ستارهٔ بیزی (`C=5.0`، `m=4.20`، `CombinedRankScore = 0.60·(bayesian_stars·20) + 0.40·mairon_score`) |

**تضمین قطعیت**: همان `session_seed` همان ناوگان، همان آلل‌ها و همان آرتیفکت را بازتولید می‌کند —
با تست پین شده (`sovereign_loop_is_fully_deterministic`)؛ و مونتاژِ نخست عمداً زیر گیتِ ۸۵ شروع
می‌شود تا حلقهٔ خودترمیمی (تزریق پروب → پرداخت UX → تقویت مرز) در هر آهنگری اجرا شود.

## 🏆 کارنامهٔ کیفیت مایرون REA

کل مخزن توسط هارنس گیتِ `morluto/rea v6.10.1` ممیزی شده — شواهد به‌صورت
[`REA_FINAL_VERDICT.json`](./REA_FINAL_VERDICT.json) / [`rea-scorecard.json`](./rea-scorecard.json)
با گزارش‌های مرحله‌به‌مرحله در [`docs/rea/`](./docs/rea) کامیت شده‌اند.

| بُعد ممیزی | امتیاز | سقف | وضعیت |
| :--- | :---: | :---: | :--- |
| `security_and_hardening` | **25** | 25 | ✅ 100% |
| `nextjs_frontend` | **20** | 20 | ✅ 100% |
| `sqlite_and_kbentry` | **15** | 15 | ✅ 100% |
| `tests_and_docs` | **10** | 10 | ✅ 100% |
| `architecture_and_hygiene` | **22** | 30 | ✅ PASS |
| **حکم نهایی** | **92** | **100** | 🏆 **PASS** (`pass_threshold = 85`، `hard_failures: []`) |

**متریک‌های وِریفای‌شده در زمان ممیزی:** ۱۸۵ فایل Rust · ۲۱۲ فایل TS/JS · ۱۱ کرت · ۳۲۵ ردیف
`"KbEntry"` در زمان ممیزی (اکنون **۳۲۷** پس از تزریق DNA دوازدهم `12_TRINITY_AVATAR_ENGINE` — بخش
*طراحی مخزن*) · صفر نشان `todo!()`. گیت یک هشدار شفاف هم دارد: ۴۹ فایل از سقف سبکیِ ۵۰۰ خطِ
`.oxlintrc` فراترند (بزرگ‌ترین: `crates/ai/src/free_llm_registry.rs` با ۹۶۰ خط).

---

## 🗄️ گنجینهٔ دانش و ناورنی‌های دفتر تولید

`seed/kian-production.db` ناورنی‌های سخت‌گیرانهٔ حسابداری را نگه می‌دارد و روی هر بیلد وریفای می‌شود:

| ناورنی / جدول | مقدار وِریفای‌شده | تضمین |
| :--- | :---: | :--- |
| `"KbEntry"` (مهارت‌ها، ایجنت‌ها و ژنوم‌ها) | **327 ردیف** | تزریق‌کننده‌های `UPSERT` ایدمپوتنت زیر `scripts/` (بدون تکرار) — جدیدترین ردیف، DNA دوازدهم `kb:dna-12-trinity-avatar-engine` است |
| `"User"` (شهروندان وریفای‌شده) | **466 شهروند** | حساب‌های شهروندِ تولید حفظ شده‌اند |
| `"ReferralLineage"` (گراف وایرال) | **247 تبار** | اتریبیوشن چند-سطحیِ رفرال دست‌نخورده |
| `"Escrow"` (سفارش‌های اتمیک) | **108 اسکرو** | با CAS (`WHERE version = ?`) + `idempotency_key` محافظت‌شده |
| `"Product"` (کاتالوگ بازار) | **20 محصول** | لیستینگ‌های فعال حفظ شده‌اند |
| `"Posting"` (پاهای دفتر) | **870 پا** | دوطرفه: دقیقاً `SUM(amount) == 0` |
| جریان ناخالص دفتر | **58,700,200,000 µKIAN** | کل بدهکاری == کل بستانکاری (`58,700.2 KIAN`) |
| ناوگان فعال ایجنت‌ها | **7 / 7 نقش** | با تست ورک‌اسپیس (`kya.rs`) اعمال می‌شود |

---

## 🛣️ سطح API (`/api/v1`)

خلاصه‌شده از `crates/server/src/routes.rs` — فهرست مرجع داخل کد است.

| گروه | اندپوینت‌ها | احراز |
| :--- | :--- | :--- |
| System | `GET /health` · `GET /economy` · `POST /system/backup` | عمومی / ادمین |
| Auth | `POST /auth/telegram` (HMAC تما) · `POST /auth/dev-login` (`KIAN_DEV_MODE`) · `POST /auth/logout` · `GET /me` · `POST /me/ton-address` | عمومی / Bearer |
| Market | `GET /market/products` · `GET /market/products/{id}` · `POST /market/purchase` · `GET /market/my` · `GET /market/escrows/{id}/delivery` · `POST /market/escrows/{id}/deliver\|release\|dispute` | عمومی / Bearer |
| Escrows | `POST /escrows` · `GET /escrows/{id}` · `POST /escrows/{id}/fund\|deliver\|release\|dispute\|refund\|cancel\|expire` | Bearer (طرف) |
| Builder | `POST /builder/sessions` · `GET /builder/sessions/{id}` · `POST /builder/sessions/{id}/chat` · `POST /builder/sessions/{id}/publish` | Bearer |
| Trinity | `POST /builder/trinity/alpha` (کالبد ۱ — کشف / قفل مستر پرامپت) · `POST /builder/trinity/orchestrate` (کالبد ۲ «۳×۹ OpenBot» + کالبد ۳ مایرون `IQ=150/EQ=145` با حلقهٔ خودترمیمی) | Bearer |
| Market — گیت مایرون | `POST /market/products/{id}/mairon-confirm` (انتشار دو-کلیدی + مُهر `⚒️ Forge-Grade`) · `POST /market/products/{id}/reviews` (رأی ستارهٔ `1..=5` شهروند) · `GET /market/products/{id}/reviews` (خلاصهٔ بیزی + هیستوگرام + `combinedRankScore`) | Bearer / عمومی |
| Wallet | `POST /wallet/mint\|burn\|faucet` · `GET/POST /wallet/withdrawals` · `POST /wallet/withdrawals/{id}/confirm\|reject` · `GET /wallet/chains` · `GET /wallet/treasury` | Bearer / ادمین |
| Deposits | `POST /wallet/ton-deposits` · `POST /wallet/usdt-deposits` · `POST /wallet/ton-deposits/{id}/confirm\|reject` | Bearer / ادمین |
| Nation | `GET /nation/bills/{id}` · `POST /nation/bills/{id}/votes\|close\|execute` | Bearer |
| AI | `GET /ai/status` · `POST /ai/chat` · `GET /ai/usage` | Bearer |
| Hermes | `GET /hermes/skills` · `POST /hermes/run` · `GET /hermes/runs` | Bearer |
| Agents | `GET /agents/fleet` · `POST /a2a/messages` · `GET /a2a/messages/{agent}` · `GET /mcp/tools` · `POST /mcp/invoke` | Bearer |
| Admin | `GET /admin/overview` · `GET /admin/hitl` · `POST /admin/hitl/{id}/approve\|reject` · `GET /admin/withdrawals` | ادمین (`X-KIAN-Key`) |
| Bot | `POST /bot/webhook` (fail-closed، secret-token) | تلگرام |

---

## ⚡ شروع سریع و وریفای

### ۱. پیش‌نیازها

- **Rust**: `rustup` (فایل پین‌شدهٔ `rust-toolchain.toml` به‌طور خودکار `rustc 1.100.0` را انتخاب می‌کند)
- **Node.js** نسخهٔ `>= 20` + **pnpm** (`corepack enable`)
- **Python 3** (برای چکر ناورنی‌ها)

### ۲. گیت کلیدیِ پدنتیک (صفر هشدار)

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### ۳. مجموعهٔ کامل تست ورک‌اسپیس (۵۶۰ تست روی ۱۱ کرت)

```bash
cargo test --workspace
```

### ۴. تایپ‌چک قراردادهای TypeScript

```bash
pnpm install && pnpm -r typecheck
```

### ۵. وریفای ناورنی‌های SQLite تولید

```bash
python3 - <<'PY'
import sqlite3
con = sqlite3.connect("seed/kian-production.db")
cur = con.cursor()
for t in ("User", "ReferralLineage", "Escrow", "Product", "KbEntry"):
    print(f"{t:16s}", cur.execute(f'SELECT COUNT(*) FROM "{t}"').fetchone()[0])
print("trial_balance  ", cur.execute("SELECT COALESCE(SUM(amount),0) FROM Posting").fetchone()[0])
PY
```

خروجی موردانتظار: `User 466 · ReferralLineage 247 · Escrow 108 · Product 20 · KbEntry 327 · trial_balance 0`.

### ۶. شواهد REA مایرون

```bash
cat REA_FINAL_VERDICT.json          # حکم کامیت‌شده: 92/100، hard_failures: []
node tools/rea/step6-gate.mjs       # گیت نهایی مایل‌استون (cargo test + clippy + ممیزی افشا)
ls docs/rea/                        # گزارش‌های گیت هر مرحله (step1 … step6)
```

### ۷. اجرای محلی کرنل

```bash
cp .env.example .env    # رازها را پر کنید — هر سطح بدون آن‌ها fail-closed است
cargo run -p kian-server
curl http://127.0.0.1:8088/api/v1/health
```

---

## 🚀 استقرار روی Railway

کیان پیکربندی بومی Railway برای یک پروژهٔ تمیزِ **۲-سرویسی** از یک مخزن ارائه می‌دهد:

- **سرویس ۱ — `kian-kernel` (API زبان Rust)**: با [`railpack.json`](./railpack.json) یا
  [`Dockerfile`](./Dockerfile) بیلد می‌شود (`infra/docker/Dockerfile.server` هم ارائه شده). یک Volume
  روی `/data` وصل کنید و `KIAN_DATABASE_URL=/data/kian.sqlite` و `KIAN_BACKUP_DIR=/data/backups` بگذارید.
  سرور روی `0.0.0.0:$KIAN_PORT` (fallback `8088`) bind می‌شود؛ مسیر healthcheck `/api/v1/health`.
  متغیرهای لازم: `KIAN_ADMIN_KEY`، `KIAN_BOT_TOKEN`، `KIAN_BOT_WEBHOOK_SECRET`، `KIAN_HOST=0.0.0.0`،
  `KIAN_TRUST_PROXY=true`، `KIAN_CORS_ORIGIN`، `KIAN_MINIAPP_URL`، `KIAN_TREASURY_*`
  و حداقل یک اسلات AI (`KIAN_AI_*`).
- **سرویس ۲ — `kian-miniapp` (Next.js 16)**: `pnpm install --frozen-lockfile && pnpm --filter
  @kian/miniapp build` → `pnpm --filter @kian/miniapp start` با `NEXT_PUBLIC_KIAN_API` و
  `NEXT_PUBLIC_TELEGRAM_BOT_USERNAME`.
- بیلدهای release سرتاسری سخت‌شده‌اند: `lto = "thin"`، `codegen-units = 1`، `panic = "abort"`،
  `strip = "symbols"`، `overflow-checks = true` (ببینید [`Cargo.toml`](./Cargo.toml)).

### متغیرهای محیطی با اولویت‌بندی اکید (`P0` → `P5`)

هر دو فایل [`.env.example`](./.env.example) و [`railway.env.example`](./railway.env.example) به‌ترتیب
**اولویت عملیاتیِ اکید** نوشته شده‌اند — بحرانی‌ترین متغیرها در بالای فایل و کلیدهای اختیاریِ تیونینگ
در پایین. هر نام زیر توسط کد واقعی خوانده می‌شود (`config.rs` · `gateway.rs` · `opencode_zen.rs` ·
`start-railway.sh` · اپ‌های Next.js):

| لایه | معنا | متغیرها (همه واقعی، معناشناسی fail-closed) |
| :--- | :--- | :--- |
| **P0 — بوت بحرانی، شبکه و فضای ذخیره** | بدون این‌ها سرور بالا نمی‌آید یا وضعیت از بین می‌رود | `KIAN_DATABASE_URL` · `KIAN_HOST` · `KIAN_PORT` · `KIAN_SEED_DATABASE_PATH` · `KIAN_BACKUP_DIR` · `KIAN_KEEP_BACKUPS` |
| **P1 — امنیت حاکم و گیت‌های ادمین** | Fail-closed: خالی = سطح صادقانه غیرفعال | `KIAN_ADMIN_KEY` · `KIAN_TRUST_PROXY` · `KIAN_CORS_ORIGIN` · `KIAN_DEV_MODE` |
| **P2 — مش استنتاج AI (اولِ بی‌کلیدِ صفر-هزینه)** | OpenCode Zen با `Bearer public` و صفر تومان کار می‌کند؛ بعدش اسلات‌های اختیاری | `OPENCODE_API_KEY` · `KIAN_AI_BASE_URL/API_KEY/MODEL` · `KIAN_AI_FALLBACK_*` · `KIAN_AI_PRIMARY_*` · `KIAN_AI_SECONDARY_*` · `KIAN_AI_TERTIARY_*` · `KIAN_AI_COARENA_*` · `KIAN_AI_OPENROUTER_*` · `KIAN_AI_ROUTER9_*` · `KIAN_AI_DAILY_TOKENS` |
| **P3 — پل بات تلگرام و مینی‌اپ** | احراز HMAC تما، Stars با `XTR`، وبهوک fail-closed | `KIAN_BOT_TOKEN` · `KIAN_BOT_WEBHOOK_SECRET` · `KIAN_MINIAPP_URL` · `KIAN_KERNEL_URL` · `NEXT_PUBLIC_KIAN_API` · `NEXT_PUBLIC_TELEGRAM_BOT_USERNAME` |
| **P4 — خزانهٔ غیر-امانیتی و ریل‌های چندزنجیره** | فقط آدرس‌های عمومی واریز — صفر کلید خصوصی روی سرور | `KIAN_TON_TREASURY` · `KIAN_TON_TO_DIN` · `KIAN_USDT_TO_DIN` · `KIAN_TREASURY_TRX/SOL/TON/BSC` · `KIAN_NATION_QUORUM` |
| **P5 — حاکمیت ران‌تایم و تیونینگ تلمتری** | اختیاری؛ پیش‌فرض‌های امنِ تولید | `KIAN_SESSION_TTL_HOURS` · `KIAN_RATE_LIMIT` · `RUST_LOG` · `RAILPACK_CARGO_WORKSPACE` · `NEXT_PUBLIC_SITE_URL` · `NEXT_PUBLIC_KIAN_URL` · `NEXT_PUBLIC_KERNEL_URL` |

جایگزین محلیِ همه-در-یک: [`docker-compose.yml`](./docker-compose.yml) یا `./start-tonight.sh`.

---

## 💬 راه‌اندازی بات تلگرام و مینی‌اپ

1. [@BotFather](https://t.me/BotFather) → `/newbot` → توکن را در `KIAN_BOT_TOKEN` بگذارید.
2. `/newapp` (یا **Bot Settings → Menu Button**) → URL مربوط به WebApp را روی دامنهٔ HTTPS
   `kian-miniapp` خودتان تنظیم کنید.
3. وبهوک را ثبت کنید:

   ```bash
   curl -X POST "https://api.telegram.org/bot${KIAN_BOT_TOKEN}/setWebhook" \
     -H "Content-Type: application/json" \
     -d '{"url": "https://<kernel-domain>/api/v1/bot/webhook",
          "secret_token": "'"${KIAN_BOT_WEBHOOK_SECRET}"'"}'
   ```

4. هر محصول بازار یک لینک عمیقِ افیلیت دارد:
   `https://t.me/<bot>?startapp=prod_<PRODUCT_ID>_ref_<TELEGRAM_ID>` — معرف هنگام خرید ثبت و از
   تقسیم اسکرو پرداخت می‌شود.

---

## 🔐 امنیت و هاردنینگ باینری

- **`#![forbid(unsafe_code)]`** در کل ورک‌اسپیس — دفتر، Rust ایمنِ خالص است.
- **پروفایل release سخت‌شده**: `lto = "thin"`، `codegen-units = 1`، `panic = "abort"`،
  `strip = "symbols"`، `overflow-checks = true`.
- **رازهای fail-closed**: نبودِ `KIAN_ADMIN_KEY`، `KIAN_BOT_TOKEN` یا `KIAN_BOT_WEBHOOK_SECRET`
  سطح مربوطه را غیرفعال می‌کند و هرگز به پیش‌فرضِ ناامن fallback نمی‌زند.
- **مقایسه‌های زمان-ثابت** (کرت `subtle`) برای کلیدهای ادمین و رازهای وبهوک؛ `initData` تلگرام با
  `HMAC-SHA256("WebAppData", bot_token)` و مدیریت فیلد مدرن `signature` وریفای می‌شود.
- **پیش‌نمایش‌های HTML سندباکس‌شده**: همهٔ آرتیفکت‌های ناموثقِ خریدار/فروشنده داخل
  `<iframe sandbox="allow-scripts">` و به‌طور اکید بدون `allow-same-origin` رندر می‌شوند.
- **صفر کلید خصوصی روی سرور**: آدرس‌های خزانه فقط اندپوینت‌های عمومی واریزند؛ پرداخت‌ها بیرون از سرور
  اجرا و با تأیید ادمین + مسیر بازپرداخت اتمیک دفتر ثبت می‌شوند.
- **مالیِ ایدمپوتنت**: هر جهش مالی یک `idempotency_key` دارد؛ گذارهای اسکرو CAS-چک‌شده‌اند و
  تراز آزمایشی ناورناً صفر است.

---

## 🧬 نقشهٔ مستندات

| مسیر | محتوا |
| :--- | :--- |
| [`KIA-BRAIN-OS/`](./KIA-BRAIN-OS) | قانون اساسی ۱۲-فایلیِ DNA سازنده — `01_DNAme` · `02_DNAwork` · `03_DNAmind` · `04_DNAharness` · `05_DNAskills` · `06_LINK_Router` · `07_LINK_Protocol` · `08_LINK_Extractor` · `09_LINK_Bootloader` · `10_MAIRON_REA_SOVEREIGN` · `11_UNCHAINED_KIA_DNA` · `12_TRINITY_AVATAR_ENGINE` |
| [`OMNI-MODEL-GENOME/`](./OMNI-MODEL-GENOME) | شاخص اکوسیستم، پروفایل‌های جناییِ ۹-فایلی به‌ازای هر مدل، قالب ورک‌اسپیس، اسکریپت‌های سمیتایزر/ممیزی |
| [`docs/dna/`](./docs/dna) | آینهٔ دروازه-خوان از هر دو گنجینهٔ بالا |
| [`docs/history/`](./docs/history) | سندِ تبار تکاملی — `EVOLUTION_LINEAGE.md` (Wave 0 تا ریلیز؛ تاریخچه حفظ / آخرین نسخه اجرا) |
| [`docs/rea/`](./docs/rea) | گزارش‌های گیت REA هر مایل‌استون (`step1` … `step6`) |
| [`docs/vault/`](./docs/vault) | گنجینه‌های gem نصب‌شده (کاتالوگ FreeLLM، 9router، AG-UI/OpenBot، Hermes، tdesktop، ربریک‌های گنجینه) |
| [`CHANGELOG.md`](./CHANGELOG.md) | تاریخچهٔ انتشار |

---

## 📜 لایسنس و حاکمیت

تحت **لایسنس MIT** منتشر شده — ببینید [`LICENSE`](./LICENSE) (`Copyright (c) 2026 KiaAgentX`).

معماری و ساخت به‌دست **Kia (`@KiaAgentX`)** تحت دکترین معماری حاکمِ **`KIA-BRAIN-OS`** — هر انتشار
باید صادقانه از گیت‌های REA عبور کند، وگرنه منتشر نمی‌شود.

</div>
