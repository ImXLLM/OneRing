#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_step4_and_genome_gems.py — تزریق ۶ مهارت گام ۴ (Wave 2) به KIAN v2.

Sections:
  1. Idempotent upsert of 6 curated flagship "KbEntry" records into
     seed/kian-production.db:
       - kb:arch-decolua-9router-anti-429            (decolua/9router v0.5.123)
       - kb:arch-opencode-zen-and-fuzzy-patch        (anomalyco/opencode v2.4.1)
       - kb:registry-freellm-52-platforms            (tashfeenahmed/freellmapi)
       - kb:registry-public-apis-120-cors            (top_120 zero-auth APIs)
       - kb:dna-kia-brain-os-9file-genome            (KIA-BRAIN-OS 9-file DNA)
       - kb:arch-omni-model-genome-9file-engine      (OMNI-MODEL-GENOME engine)
  2. Verification: KbEntry 56 -> 62, active products stay 20, citizens 466,
     referral lines 247, escrows 108, trial balance sum stays 0
     (58,700,200,000 micro-KIAN untouched), LIKE search < 5 ms.
  3. Sync of seed/kian-production.db -> /home/z/my-project/download/kian-production.db.

Idempotency: every upsert compares title+body first; unchanged rows are skipped
("0 inserted, 0 updated, 6 skipped" on the second run), changed rows are replaced.
Posting (double-entry ledger) is never touched by this script.
"""

from __future__ import annotations

import shutil
import sqlite3
import sys
import time
from pathlib import Path

REPO = Path("/home/z/my-project/kian-launch/kian")
SEED_DB = REPO / "seed" / "kian-production.db"
DOWNLOAD_DB = Path("/home/z/my-project/download/kian-production.db")
CREATED_AT = "2026-11-24T02:00:00.000Z"

# ── 1. The 6 curated flagship KB entries ─────────────────────────────────────

KB_ITEMS: list[tuple[str, str, str]] = [
    (
        "kb:arch-decolua-9router-anti-429",
        "معماری مسیریابی: گاردهای ضد-429 جعلی 9Router v0.5.123 — سانتی‌سایز terminal و پاک‌ساز پرامپت Antigravity",
        "پورت decolua/9router v0.5.123 (opencodeFingerprint.js + systemPrompt.js @ bb399170) برای "
        "لِین‌های رایگان oc/* کیان: (۱) OpenCode Free هر ابزاری که نامش حاوی «terminal» باشد "
        "(غیرحساس به بزرگی حرف) را با 429 FreeUsageLimit جعلی رد می‌کند — تابع "
        "sanitize_tool_names_for_opencode نام را به shell_exec بازنویسی می‌کند (برخورد نام‌ها با "
        "پسوند _2/_3 حل می‌شود) و نقشهٔ بازگشت aliased→original را برای پاسخ برمی‌گرداند. "
        "(۲) خط هویتی «You are Hermes Agent, built by Nous Research» روی Google Antigravity "
        "429 RESOURCE_EXHAUSTED جعلی می‌گیرد — sanitize_system_prompt_for_antigravity آن را به "
        "«You are KIAN Autonomous Agent, built by KIAN Systems» بازنویسی می‌کند و ۶ خط محرکِ "
        "مستندشده (End Hermes runtime environment و دستورالعمل‌های Google model operational و "
        "…) را حذف می‌کند؛ بقیهٔ جمله دست‌نخورده می‌ماند. (۳) طبقه‌بندی شکست لِین: "
        "410/model_not_found → بازنشستگی دائمی لِین، 400 → فوری لِین بعدی بدون کول‌داون، "
        "429 → بک‌آف نمایی 2s×2^level با سقف ۵ دقیقه، 5xx → کول‌داون ترانزیت ۳۰ ثانیه — "
        "همان accountFallback.js + errorConfig.js + combo.js. (۴) مالتی‌تورن بدون "
        "reasoning_content غیرخالی روی Zen/DeepSeek/GLM خطای 400 می‌دهد — inject_reasoning_placeholders "
        "برای پیام‌های assistant یک « » تزریق می‌کند. پورت Rust کیان: crates/ai/src/free_llm_registry.rs "
        "(sanitize_tool_names_for_opencode + sanitize_system_prompt_for_antigravity + classify_http_failure "
        "+ LaneHealthTracker + inject قواعد reasoning). منبع: گنجینهٔ decolua-9router-gems (Wave 2 - Step 4).",
    ),
    (
        "kb:arch-opencode-zen-and-fuzzy-patch",
        "معماری درگاه: OpenCode Zen صفر-کارت (Bearer public + ۱۰ مدل فعال) و ویرایشگر جراحی ۵گذر",
        "دو قطعهٔ anomalyco/opencode v2.4.1 (commit 18ff071): (۱) درگاه استنتاج "
        "https://opencode.ai/inference/openai/v1 با فال‌بک legacy https://opencode.ai/zen/v1؛ "
        "وقتی هیچ کلیدی تنظیم نشده، پلاگین Opencode «Authorization: Bearer public» و "
        "«User-Agent: opencode/latest/2.4.1/opencode» تزریق می‌کند — ناشناس مطلق، صفر کارت. "
        "۱۰ مدل فعال صفر-هزینه: beam-free (1M ctx)، big-pickle (200K)، deepseek-v4.1-flash-free "
        "(200K، vision)، flint-40b-free (262K)، hy4-flash-free (1M، vision)، iceman-free (262K)، "
        "mimo-v2.6-flash-free (200K، multimodal)، nemotron-3.5-lightning-free (262K)، "
        "nemotron-3.5-super-vl-free (1M، vision)، tidepool-alpha-free (1M)؛ مدل‌های بازنشسته "
        "پیش از درگاه بلاک می‌شوند: longcat-2.5-free (410)، glm-5.3-flash-free (400)، "
        "deepseek-v4-flash-free (400)، kimi-k2.5-free، qwen3.6-plus-free و sambanova (402). "
        "(۲) ویرایشگر جراحی ۵گذر: گذر ۱ تطبیق دقیق بایت‌به‌بایت، گذر ۲ trim انتهای خط، "
        "گذر ۳ trim دو طرف، گذر ۴ نرمال‌سازی یونیکد (گیومهٔ فرفری ’“→'\"، دَش —–→-، "
        "فاصلهٔ non-breaking → فاصله)، گذر ۵ انکر بلوکی (≥۳ خط: خط اول/آخر + شباهت میانی "
        "≥۰٫۵ با LCS) و نرمال‌سازی تورفتگی — با حفظ CRLF/LF فایل؛ شکست → رأی‌گیری خطوط "
        "(missHint) با شمارهٔ خط نزدیک؛ تشخیص «از قبل اعمال‌شده»؛ چندتطبیق مبتنی بر رد. "
        "همراه سیاست اجرای کدکس: git reset --hard و rm -rf / ممنوع، git push --force و "
        "python3 -c نیازمند تأیید انسانی، cargo test/clippy/check و ls و pwd مجاز، "
        "بقیه → Prompt. پورت Rust کیان: crates/ai/src/opencode_zen.rs + "
        "crates/hermes/src/fuzzy_patch.rs (apply_surgical_patch + miss_hint + evaluate_exec_policy). "
        "منبع: گنجینهٔ anomalyco-opencode-gems (Wave 2 - Step 4).",
    ),
    (
        "kb:registry-freellm-52-platforms",
        "رجیستری: آبشار ۳لایهٔ ۵۲ پلتفرم LLM رایگان بدون کارت — Tier1 ناشناس → Tier2 بدون کارت → Tier3 مرورگر",
        "کاتالوگ tashfeenahmed/freellmapi (62526e0 + کاتالوگ زندهٔ 2026.11.23.4b4737): ۵۲ پلتفرم "
        "OpenAI-compatible با ۴۵۴ مدل چت، همه بدون کارت بانکی (حاکمیت صفر-کارت DNAme). آبشار رسمی کیان: "
        "Tier 1 ناشناس مطلق — OpenCode Zen (Bearer public)، Kilo (۲۰۰ درخواست/ساعت per-IP)، "
        "OVHcloud (Qwen3.5-397B، ۲RPM)، LLM7، Pollinations؛ Tier 2 ثبت‌نام رایگان بدون کارت — "
        "Cerebras (۱M توکن/روز)، Groq، Google Gemini، OpenRouter :free (۵۰ RPD)، Mistral، "
        "NVIDIA NIM، Zhipu Flash (۱M توکن/روز)، Cloudflare Workers AI (۱۰هزار Neuron/روز)؛ "
        "Tier 3 کاتالوگ مرورگری ۱۲۰ API عمومی CORS-دار برای fetch() زندهٔ استودیو. کوارک‌های "
        "بحرانی: Kilo پرامپت‌ها را برای آموزش لاگ می‌کند (متن حساس redact شود)، OVH ناشناس ۲RPM "
        "لِین نجات است نه گذرگاه اصلی، cheapllm به‌جای 429 صف می‌سازد (timeout 180s)، "
        "Routeway بدون User-Agent مرورگری خطای Cloudflare 1010 می‌دهد، ModelScope نیاز به "
        "real-name چین دارد (در آبشار رسمی کیان نیست). میان‌افزار تاب‌آوری: کشف آرگومان ابزارِ "
        "placeholder-مانند (<result-from-…> / {{step_…}}) با hint تلاش یک‌باره (langflow) و چک "
        "فیلد refusal صریح پیش از پارس JSON Schema (langchain OutputFixing). پورت Rust کیان: "
        "crates/ai/src/free_llm_registry.rs (FREE_LLM_PROVIDERS + zero_card_waterfall + "
        "detect_unresolved_placeholder_tool_args + check_model_refusal). منبع: گنجینهٔ "
        "freellm-and-public-apis-gems (Wave 2 - Step 4).",
    ),
    (
        "kb:registry-public-apis-120-cors",
        "رجیستری: ۱۲۰ API عمومی بدون کلید با HTTPS/CORS=Yes — جعبه‌ابزار fetch() زندهٔ استودیوی کیان",
        "کاتالوگ top_120_studio_browser_apis.json (115 از ۱۲۰ مرورگر-آماده با CORS=Yes، ۵ مورد "
        "بدون CORS فقط سروری): دسته‌ها شامل Cryptocurrency (CoinGecko، 0x، Block Lottos)، "
        "Geocoding (REST Countries، GeoJS)، Currency Exchange (Frankfurter، currency-api)، "
        "Weather (Open-Meteo، NASA POWER)، Development (Beeceptor، Quotable)، Test Data "
        "(DummyJSON، FakerAPI)، Anti-Malware (URLhaus)، Books (PoetryDB، Bible-api) و …؛ "
        "همه بدون کلید API و بدون KYC — دقیقاً روی حاکمیت صفر-کارت کیان. قواعد استودیو: "
        "فقط مدخل‌های HTTPS+CORS-دار در fetch() زندهٔ مرورگر مجازند (SSRF guard روی مسیرهای "
        "داخلی همچنان اعمال می‌شود)، URLها در const STUDIO_PUBLIC_APIS_120 کرنل کیان embed "
        "شده‌اند تا استودیو بدون شبکهٔ اضافه کاتالوگ را بلند کند. پورت Rust کیان: "
        "crates/ai/src/free_llm_registry.rs (STUDIO_PUBLIC_APIS_120 + "
        "browser_ready_public_apis → PublicApiSpec). منبع: گنجینهٔ freellm-and-public-apis-gems "
        "(Wave 2 - Step 4).",
    ),
    (
        "kb:dna-kia-brain-os-9file-genome",
        "ژنوم شناختی: KIA-BRAIN-OS ۹فایلی — DNAme/DNAwork سازنده، Aragon-4 و قوانین صفر-کارت/UX-اول در کرنل",
        "سیستم ۱ گام ۴: KIA-BRAIN-OS/ ریشهٔ مخزن — ۹ فایل DNA به‌هم‌پیوسته با frontmatter "
        "YAML (file_id، schema_version 1.0.0، layer): 01_DNAme (هویت سازنده Kia/KiaAgentX، "
        "۱۸ ساله، تهران، INFJ؛ حاکمیت ۱۰۰٪ صفر-کارت و صفر-KYC با ریل‌های TON و Telegram Stars "
        "XTR؛ اولویت #1 محصول = UX فریم‌اول تعاملی، RTL تمیز با Vazirmatn، صفر stub مرده)، "
        "02_DNAwork (معماری ۹ کریت Rust، ارقام دفتر: ۴۶۶ شهروند/۲۴۷ رفرال/۱۰۸ اسکرو/۲۰ محصول "
        "فعال/trialBalanceSum=0 روی ۵۸٬۷۰۰٬۲۰۰٬۰۰۰ micro-KIAN؛ ارزش COCOMO II: $523,750 مستقل "
        "/ ~$1.18M با اکوسیستم ۶۰ پروژه)، 03_DNAmind (پروتکل Aragon-4 Principal Architect: "
        "تجزیهٔ اصل اول، ممیزی ریاضی — عدد بدون اجرا ممنوع — صفر تملق، ماژول عمیق بر "
        "boilerplate)، 04_DNAharness (فیزیک محیط: Command Center → Z.ai + Arena + Railway؛ "
        "گیت راز: صفر توکن در گیت، تنها مجاز Bearer public)، 05_DNAskills (ماتریس مهارت‌ها)، "
        "06_LINK_Router (پل قطعی به OMNI-MODEL-GENOME)، 07_LINK_Protocol (پاکت تحویل با "
        "base_sha/invariants/verification_proof)، 08_LINK_Extractor (پروتکل استخراج گنجینه)، "
        "09_LINK_Bootloader (بوت‌لودر تک‌نوبتی نشست). تزریق کرنل: crates/hermes/src/"
        "creator_dna_contract.rs — CreatorDnaContract (sovereignty=true همیشگی، ux_priority_tier=1، "
        "persona ثابت)، build_creator_dna_system_anchor() لنگر سیستم هر ایجنت کیان، و "
        "audit_against_creator_dna() ممیزی قطعی دوزبانه: پیشنهاد API کارت‌دار، ادعای تکمیل "
        "بی‌اثبات (all tests pass بدون اجرا) و stub مردهٔ UI را پرچم می‌زند. منبع: ساخت "
        "اصلی گام ۴ (Wave 2 - Step 4).",
    ),
    (
        "kb:arch-omni-model-genome-9file-engine",
        "معماری ژنوم مدل: OMNI-MODEL-GENOME ۹فایلی + کامپایلر پرامپت gpt-6-standard-free (بدون Think) و روتینگ ۲۰تسکی",
        "سیستم ۲ گام ۴: OMNI-MODEL-GENOME/ ریشهٔ مخزن — ورک‌اسپیس plug-and-play مهندسی معکوس "
        "مدل‌ها: 00_ECOSYSTEM_INDEX.md (جدول مسیریابی ۲۰ تسک کلان، هدف #01 = gpt-6-standard-free "
        "وب رسمی با Think Mode OFF + ۱۱ اسلات آینده: gpt-6-thinking، claude-opus-thinking، "
        "claude-sonnet-standard، gemini-pro-longcontext، deepseek-r1-reasoner، grok-frontier، "
        "zai-glm-harness، qwen-coder-agent، openai-codex-agent، llama4-cerebras-fast، "
        "opencode-zen-flagship) و برای هر مدل ۹ فایل ژنوم 01_TOPOLOGY_AND_HARNESS تا "
        "09_ROUTING_AND_HANDOFF_CARD + اسکریپت‌های compile_for_gpt6_standard.py و "
        "sanitize_gpt6_standard_output.py. قانون طلایی مدل بدون Think: «Token Order = Thought "
        "Order» — پرسیدن کد/حکم اول باعث Premature Commitment در توکن‌های 1..30 می‌شود؛ "
        "درمان: جدول پیش‌محاسبهٔ <pre_computation_state_trace> (ثبت invariant‌ها + edge case‌ها + "
        "حالت‌های شکستِ ردشده) قبل از هر کد — بازسازی ~۸۸٪ دقت Think Mode با ~۴× سرعت؛ "
        "گارد ضد-Canvas/ضد-وب‌سرچ (emit inline fenced Markdown، سنتز قطعی آفلاین)؛ گیت‌های "
        "فازی ≤۶ قاعده؛ قرارداد ضد-elision (ممنوعیت «// … existing code»، ترجیح بلوک "
        "SEARCH/REPLACE با ≥۳ خط انکر). اسکورکارد ۲۰تسکی gpt-6-standard-free: TechnicalGenomeSynthesis "
        "0.91، HardAlgorithmicDeduction 0.88، SystemArchitecture 0.87، SpecComplianceReview 0.86، "
        "AdversarialTestGeneration 0.85، StrictJsonExtraction 0.84، SqliteLedgerDesign 0.83، "
        "XAlgorithmViralGrowth 0.43، LongHorizonAutonomousLoop 0.39 (Do-Not-Route). تزریق کرنل: "
        "crates/ai/src/omni_genome_router.rs — GenomeLayerId با نام‌های canonical، "
        "EngineeringWorkload ۲۰تایی، load_model_genome_from_dir (اعتبارسنجی ۹ فایل + frontmatter + "
        "پارس اسکورکارد؛ امتیاز نامعلوم هرگز خودسازی نمی‌شود)، builtin_gpt6_standard_profile و "
        "builtin_ecosystem_profiles (صفر-IO)، compile_prompt_for_model (تزریق state-trace/Canvas/فاز/"
        "anti-elision) و HandoffPacket هم‌اسکیما با 07_LINK_Protocol. منبع: گنجینهٔ "
        "omni-model-genome-gpt6-standard (Wave 2 - Step 4).",
    ),
]

# ── 2. Idempotent upsert ─────────────────────────────────────────────────────


def upsert_kb(db: sqlite3.Connection) -> dict:
    """Compare-then-write: unchanged rows are skipped, changed rows replaced."""
    stats = {"inserted": 0, "updated": 0, "skipped": 0}
    cur = db.cursor()
    for kb_id, title, body in KB_ITEMS:
        row = cur.execute('SELECT "title","body" FROM "KbEntry" WHERE "id"=?', (kb_id,)).fetchone()
        if row is not None and row[0] == title and row[1] == body:
            stats["skipped"] += 1
            continue
        cur.execute(
            'INSERT INTO "KbEntry" ("id","title","body","createdAt","updatedAt") VALUES (?,?,?,?,?)'
            ' ON CONFLICT("id") DO UPDATE SET "title"=excluded."title", "body"=excluded."body",'
            ' "updatedAt"=excluded."updatedAt"',
            (kb_id, title, body, CREATED_AT, CREATED_AT))
        if row is None:
            stats["inserted"] += 1
        else:
            stats["updated"] += 1
    return stats


# ── 3. Verification ──────────────────────────────────────────────────────────


def verify(db: sqlite3.Connection) -> dict:
    cur = db.cursor()
    kb = cur.execute('SELECT COUNT(*) FROM "KbEntry"').fetchone()[0]
    active = cur.execute('SELECT COUNT(*) FROM "Product" WHERE "active"=1').fetchone()[0]
    citizens = cur.execute('SELECT COUNT(*) FROM "User"').fetchone()[0]
    referrals = cur.execute('SELECT COUNT(*) FROM "ReferralLineage"').fetchone()[0]
    escrows = cur.execute('SELECT COUNT(*) FROM "Escrow"').fetchone()[0]
    trial = cur.execute("SELECT COALESCE(SUM(amount),0) FROM Posting").fetchone()[0]
    timings = []
    for q in ("429 جعلی", "shell_exec", "Bearer public", "KIA-BRAIN-OS", "pre_computation_state_trace"):
        t0 = time.perf_counter()
        rows = cur.execute(
            'SELECT COUNT(*) FROM "KbEntry" WHERE "title" LIKE ? OR "body" LIKE ?',
            (f"%{q}%", f"%{q}%")).fetchone()[0]
        timings.append((q, rows, (time.perf_counter() - t0) * 1000))
    return {
        "kb": kb, "active": active, "trial_sum": trial,
        "citizens": citizens, "referrals": referrals, "escrows": escrows,
        "search": timings,
    }


def main() -> int:
    assert REPO.is_dir() and SEED_DB.is_file(), "repo/seed db missing"

    before = verify(sqlite3.connect(SEED_DB))
    print(f"[1/3] before: KbEntry={before['kb']} | active products={before['active']} | "
          f"citizens={before['citizens']} | referrals={before['referrals']} | "
          f"escrows={before['escrows']} | trial balance sum={before['trial_sum']}")

    db = sqlite3.connect(SEED_DB)
    db.row_factory = sqlite3.Row
    try:
        ks = upsert_kb(db)
        db.commit()
    finally:
        db.close()
    print(f"[2/3] kb upsert: +{ks['inserted']} new, {ks['updated']} updated, {ks['skipped']} skipped")

    after = verify(sqlite3.connect(SEED_DB))
    print(f"[3/3] after: KbEntry={after['kb']} (56 -> 62 expected) | active products={after['active']} | "
          f"citizens={after['citizens']} | referrals={after['referrals']} | escrows={after['escrows']} | "
          f"trial balance sum={after['trial_sum']}")
    for q, rows, ms in after["search"]:
        flag = "OK" if ms < 5.0 else "SLOW"
        print(f"      search '{q}': {rows} hits in {ms:.2f} ms [{flag}]")

    # sync to download deliverable
    DOWNLOAD_DB.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(SEED_DB, DOWNLOAD_DB)
    print(f"      synced seed db -> {DOWNLOAD_DB}")

    ok = (
        after["kb"] == 62 and after["active"] == 20 and after["trial_sum"] == 0
        and after["citizens"] == 466 and after["referrals"] == 247 and after["escrows"] == 108
        and ks["updated"] == 0 and (ks["inserted"] == 6 or ks["skipped"] == 6)
        and all(ms < 5.0 for _, _, ms in after["search"])
    )
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
