# DNA شانزدهم مغز خالق: ماتریسِ ژنومِ چهارمدلی، خزانهٔ چهارجریانیِ روزِ اول روی Base و قراردادهایِ Solidity

> `KIA-BRAIN-OS/16_QUAD_MODEL_GENOME_AND_BASE_TREASURY.md` — Wave 7 (Step 4/4 — FINAL WAVE) کرنل `0xKIA`
> برچسب اجرایی: `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]`
> موتورها: `OMNI-MODEL-GENOME/` · `crates/wallet/src/base_treasury.rs` · `crates/hermes/src/sovereign_monopoly_engine.rs`
> قراردادها: `contracts/base/ZeroXKiaToken.sol` · `contracts/base/KiaBaseEscrowTreasury.sol`
> مهاجرت: `0011_sovereign_monopoly_genome_and_base_treasury.sql` · سیم: `GET/POST /api/v1/monopoly/*`

---

## ۱. چکیدهٔ اجرایی

DNA شانزدهم — موجِ پایانی — سه لایهٔ حاکم را همزمان قفل می‌کند:

1. **ماتریسِ ژنومِ `OMNI-MODEL-GENOME` v4 (چهارمدلی × ۹فایلی)** — ۳۶ فایلِ
   canonical برای `GPT 6-Web`، `GEMINI 4 ARAGON HIGH`،
   `DeepSeek-3.2-WEB-Think-mode` و `Cluade-sonnet-5.5-extra` (با aliasِ
   `Claude-sonnet-5.5-extra`)؛ هر مدل دقیقاً ۹ فایلِ
   `01_IDENTITY…` تا `09_DISTILLED_CLONE_SYSTEM_PROMPT.md` دارد + ماتریسِ
   مقایسهٔ ۲۰تسکی + README؛ ژنوم‌هایِ v1/v2 زیرِ `legacy/` زنده‌اند.
2. **خزانهٔ حاکمِ روزِ اول روی Ethereum Base (`Chain ID: 8453`)** —
   چهار جریانِ درآمد با تقسیمِ اتمیِ صفرجمعِ `Precise`: مالیاتِ منشورِ ۱۰٪،
   پایشِ وابستهٔ ۱۰٪–۵۰٪ با قانونِ «فروشِ مستقیمِ بی‌معرف» (۲۰۰۰ bps پلتفرم /
   ۸۰۰۰ bps سازنده)، بوست‌هایِ مارژینِ ۱۰۰٪ و اسپردِ برداشتِ ۱.۵٪.
3. **دو قراردادِ Solidity (`^0.8.24`)** — `ZeroXKiaToken` (عرضهٔ ثابتِ ۱
   میلیارد + EIP-2612 permit + burn، معماریِ تمیزِ ۱۰۰/۱۰۰ GoPlus/TokenSniffer)
   و `KiaBaseEscrowTreasury` (امانتِ غیرحضانتی + تقسیم‌گرِ اتمی + قفلِ
   non-reentrant).

قرارداد کرنل حفظ شده: صفر `unsafe`، صفر هشدارِ `Clippy` (`-D warnings`)، قطعیتِ
کامل و صفر شناور روی پول — همهٔ پاها با موتورِ `Precise` Wave 6 اثبات می‌شوند.

## ۲. ماتریسِ ژنوم — نقشهٔ کاملِ `omni-genome-v4`

| # | مدل | پوشه | تخصصِ حاکم | آواتارِ ترامینیتی |
| --- | --- | --- | --- | --- |
| 01 | `GPT 6-Web` | `models/01-gpt-6-web/` | سنتزِ وبِ لحظه‌ای، آرتیفکتِ فریم‌۱ (۳۹۰px صفر-overflow)، محصولِ تمام‌پشتهٔ سریع | `Alpha` |
| 02 | `GEMINI 4 ARAGON HIGH` | `models/02-gemini-4-aragon-high/` | سنتزِ میان‌مخزنیِ ۱M-توکنی، اورکستراتورِ ۹-مدالیتی، تأییدِ معماریِ کلان | `Logic-Prime` |
| 03 | `DeepSeek-3.2-WEB-Think-mode` | `models/03-deepseek-3.2-web-think-mode/` | ریاضیاتِ کمّیِ `<think>`، اثباتِ آلفایِ علّیِ CCXT/Vibe، راستی‌آزماییِ رسمی | `Mairon` |
| 04 | `Cluade-sonnet-5.5-extra` | `models/04-claude-sonnet-5.5-extra/` | جراحیِ Rust، صفر-هشدارِ pedantic، ممیزیِ قرارداد، کپیِ دوزبانه | `Logic-Prime` |

قواعدِ مسیریابی (`QuadTaskDomain → champion`): `web-synthesis`/`ui-ux-artifact`
→ 01 · `cross-repo-context` → 02 · `quant-proof` → 03 ·
`rust-systems-audit`/`bilingual-copywriting` → 04. حلِ alias صادقانه است:
`Cluade-sonnet-5.5-extra` و `Claude-sonnet-5.5-extra` هر دو به مدلِ ۰۴ می‌روند و
`gpt-6-standard-free` به ۰۱ (سازگاری با ژنومِ موروثی).

## ۳. منشورِ روزِ اول — «جنگ اول به از صلح آخر»

| جریان | نرخ | مقصد | قانون |
| --- | --- | --- | --- |
| ۱ — مالیاتِ منشور | `1000 bps` (۱۰٪) | `system:kia_sovereign_treasury` | روی هر فروشِ بازارخانه |
| ۲ — پایشِ وابسته | `1000..=5000 bps` | `user:{ref_id}` | پایشِ پیش‌فرض `1000`؛ بدونِ معرف: پایشِ ۱۰٪ **مستقیم به خزانهٔ حاکم** → پلتفرم `2000 bps`، سازنده `8000 bps` |
| ۳ — بوستِ لحظه‌ای | `10000 bps` (۱۰۰٪) | `system:kia_sovereign_treasury` | `FeaturedSlot24h` (۲۵۰ XTR / ۵$ USDC / ۵M µKIAN) · `MaironFastTrackAudit` (۱۰۰ XTR / ۲$ / ۲M) · `ViralSwarmAmplifier` (۵۰۰ XTR / ۱۰$ / ۱۰M) |
| ۴ — اسپردِ برداشت | `150 bps` (۱.۵٪) | `system:kia_sovereign_treasury` | تسویهٔ لحظه‌ای و برداشتِ بین‌زنجیره‌ای |

`compute_day_one_atomic_split` چهار پایِ دوطرفه می‌سازد
(`buyer_debit + kia_treasury + affiliate + creator == 0`) — باقیماندهٔ اتمی روی
سازنده می‌بندد و اثباتِ صفرجمع با `Precise` بازچک می‌شود؛ تقسیمِ نامتوازن هرگز
بیرون نمی‌آید و ردیفِ نامتوازن در لایهٔ db رد می‌شود (`DbError::Corrupt`).

## ۴. قراردادهایِ Base `8453`

* **`ZeroXKiaToken.sol`** — ERC-20 با عرضهٔ ثابتِ `1,000,000,000 × 10^18` که
  یک‌بار در constructor به deployer (خزانهٔ حاکم) ضرب می‌شود؛ بدونِ mint
  پسِ‌constructor، blacklist، pause و مالیاتِ mutable؛ EIP-2612 `permit` با
  `DOMAIN_SEPARATOR` و `burn` کاربر — هدفِ ۱۰۰/۱۰۰ GoPlus و TokenSniffer.
* **`KiaBaseEscrowTreasury.sol`** — whitelistِ بستهٔ دارایی (ETH بومی،
  USDC بومیِ `0x8335…2913`، `$0xKIA`)، `settleMarketplaceSale` (کشِ گروس از
  خریدار → پاهای اتمی → ledger داخلی)، `purchaseInstantBoost` (۱۰۰٪ به خزانه)
  و `withdrawWithSpread` (اسپردِ ۱۵۰ bps)؛ همه با قفلِ non-reentrant،
  effects-then-interactions و خزانهٔ immutable؛ `receive()` مستقیم رد می‌شود.
* **EIP-681** — `build_base_eip681_payment_uri` برای لینکِ یک‌کلیکیِ
  Trust/Coinbase/MetaMask با `@8453` اجباری؛ دارایی‌های غیر on-chain
  (`XTR`, `µKIAN`) خطای `NotOnChainSettleable` می‌دهند — هیچ URI جعلی نیست.

## ۵. اورکستراتورِ انحصار و سیمِ `/api/v1/monopoly/*`

`SovereignMonopolyOrchestrator` روترِ ژنوم + منشور + descriptor هایِ Base +
کیتِ لانچ (`ProductHuntCmcLaunchKit`) را در یک وضعیتِ واحد نگه می‌دارد. سه
مسیرِ additive:

* `GET /monopoly/genome-matrix` — ۴ مدل با وضعیتِ تأییدِ ۹فایلی (هدفِ ۳۶) +
  ۶ قاعدهٔ مسیریابی؛ تأییدِ دیسکی fail-closed با fallback به builtin.
* `GET /monopoly/charter-and-base` — چهار جریانِ منشور + پیکربندیِ Base +
  ممیزیِ هر دو قرارداد.
* `POST /monopoly/simulate-split` — اجرایِ تقسیم، ماندگاری در
  `sovereign_treasury_splits` (مهاجرتِ `0011`) و اثباتِ صفرجمع + EIP-681.

## ۶. گیتِ پذیرشِ موجِ هفتم

| گیت | وضعیت |
| --- | --- |
| ۳۶ فایلِ ژنوم کامل و بالایِ آستانه | ✅ `models/{01..04}/` |
| منشور: فروشِ معرف‌دار ۱۰/۱۰/۸۰ و بی‌معرف ۲۰/۰/۸۰ (µUSDC دقیق) | ✅ تستِ قطعی |
| بوست‌ها ۱۰۰٪ و اسپردِ ۱۵۰ bps بی‌اتلاف | ✅ |
| EIP-681 با `@8453` + ردِ XTR/µKIAN | ✅ |
| ردیفِ نامتوازن در db رد می‌شود | ✅ |
| قراردادها: بدونِ mint بعدی/blacklist/pause/مالیاتِ mutable | ✅ |
| صفر `unsafe` + صفر هشدارِ clippy | ✅ |
| `KbEntry` → **۳۳۱** (`kb:dna-16-quad-genome-base-treasury`) | ✅ |

*Wave 7 پایانِ خطِ چهارموجی است: هر DNA از ۱۲ تا ۱۶ برچسبِ
`[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]` دارد و هیچ موجی بدونِ گیتِ مایرون
بسته نشده است.*
