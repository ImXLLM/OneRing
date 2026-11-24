#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_nousresearch_gems.py — تزریق ۱۰ مهارت پرچم‌دار گنجینهٔ NousResearch/hermes-agent به KIAN v2 (Wave 2 - Step 2).

Sections:
  1. Idempotent upsert of 10 curated flagship "KbEntry" records into
     seed/kian-production.db (Popular Web Designs, p5.js Canvas, Architecture
     Diagrams, Baoyu Infographics, Systematic Debugging, TDD, Codebase
     Inspection, Grounded Citations, Product Price Monitor, Hermes Function
     Calling Standard).
  2. Verification: KbEntry 39 -> 49, active products stay 20,
     Posting sum stays 0 (58,700,200,000 micro-KIAN untouched), LIKE search < 5 ms.
  3. Sync of seed/kian-production.db -> /home/z/my-project/download/kian-production.db.

Idempotency: every upsert compares title+body first; unchanged rows are skipped
("0 updated, 10 skipped" on the second run), changed rows are replaced.
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
CREATED_AT = "2026-11-24T00:00:00.000Z"

# ── 1. The 10 curated flagship KB entries ────────────────────────────────────

KB_ITEMS: list[tuple[str, str, str]] = [
    (
        "kb:skill-nous-popular-web-designs",
        "مهارت وب: ۵۴ دیزاین‌سیستم واقعی (Stripe، Linear، Vercel) به‌صورت بلوک‌های آمادهٔ HTML/CSS",
        "کتابخانهٔ ۵۴ دیزاین‌سیستمِ واقعیِ محصولات محبوب (Stripe، Linear، Vercel، GitHub و …) به‌شکل "
        "فایل‌های مرجع HTML/CSS آمادهٔ کپی — به‌جای اینکه مدل از صفر رنگ و فاصله حدس بزند، با تریگرهای "
        "طبیعی («شبیه Stripe بساز»، «طراحی مثل Linear») سیستم‌عاملِ دیداریِ همان برند بارگذاری می‌شود. "
        "هر دیزاین‌سیستم شامل پالت، مقیاس تایپوگرافی، شعاع گرد، سایه و الگوی چیدمان است و به‌عنوان "
        "«قصر مرجع» جلوی انحراف خروجی استودیو از استانداردهای واقعی صنعت را می‌گیرد. قواعد استفاده: "
        "(۱) اول دقیق‌ترین مطابقتِ برند را از فهرست پیدا کن، نه ترکیبِ چشمیِ خودساخته؛ (۲) دیزاین‌سیستم "
        "پایه است نه محدودیت — رنگ‌های برندِ کاربر جایگزین پالت مرجع می‌شوند ولی مقیاس فاصله و تایپ حفظ "
        "می‌شود؛ (۳) هیچ‌گاه CSS کامل را در پاسخ نمی‌آوری، فقط مسیرِ سیستم انتخاب‌شده و متغیرهای تِم را "
        "اعلام می‌کنی. در کیان این مهارت با دیزاین‌سیستم بی‌کروم ‎#0a0a0a/‎#fafafa (kb:skill-generative-ui-zero-chroma) "
        "ترکیب می‌شود: پایگاهِ نوتورالِ کیان ثابت می‌ماند و این مهارت لایهٔ «احساسِ برند» را می‌سازد. "
        "منبع: skills/creative/popular-web-designs در گنجینهٔ NousResearch (Wave 2).",
    ),
    (
        "kb:skill-nous-p5js-creative-canvas",
        "مهارت خلاق: اسکچ‌های تعاملی p5.js — هنر مولد، شیدر، سه‌بعدی و بوم زندهٔ بدون وابستگی",
        "مهارت خلاق‌کدنویسی با p5.js برای خروجی‌های زندهٔ بومِ (canvas) استودیو: هنر مولد (generative art)، "
        "انیمیشن، تعامل موس/کیبورد، WebGL و شیدر و صحنه‌های سه‌بعدی سبک — همه در یک فایل HTML تک‌فایلیِ "
        "بدون build که در sandbox پیش‌نمایش کیان مستقیم اجرا می‌شود. قواعد طلایی این مهارت: (۱) setup و "
        "draw با state کم و بی‌حالت (stateless) — هر فریم از ورودی‌های کوتاه می‌سازد تا استریمِ ناتمام "
        "هرگز بومِ سیاه نگیرد؛ (۲) seed تصادفی ثابت برای بازتولیدپذیری — هر ران از یک sketch با seed "
        "داده‌شده همان خروجی را می‌دهد؛ (۳) رزولوشن با devicePixelRatio تطبیق و حلقهٔ draw با "
        "frameRate سقف‌دار که CPU گوشی را نمی‌سوزاند؛ (۴) هیچ fetch بیرونی — همهٔ assetها inline تا CSP "
        "سندباکس نشکند؛ (۵) عملگرهای تعاملی با منطق فیزیکی RTL سازگارند. کاربرد در کیان: خروجی‌های "
        "جنریتیوِ استودیو (داشبورد زنده، ویژوالایزر، بازی کوچک) با همین قواعد تولید می‌شوند تا پیش‌نمایش "
        "بدون پرش و بدون وابستگی بیرونی بالا بیاید. منبع: skills/creative/p5js گنجینهٔ NousResearch.",
    ),
    (
        "kb:skill-nous-architecture-diagram",
        "مهارت معماری: دیاگرام SVG تیرهٔ سیستم/زیرساخت/کلاود به‌صورت HTML خودبسنده",
        "دیاگرام‌های معماری با کیفیت ارائه‌ای: SVG تیره‌تم (dark-themed) برای معماری سیستم، زیرساخت "
        "کلاود، جریان داده و توپولوژی شبکه — خروجی یک HTML خودبسنده با SVG درون‌خطی است که هرگز به "
        "کتابخانهٔ بیرونی وابسته نیست و در پرچم‌ها/مستندات/پنل کیان مستقیم رندر می‌شود. قواعد طراحی: "
        "(۱) گرید ۸px و چیدمانِ قطعیِ لایه‌ها (کاربر ← گیت‌وی ← سرویس ← دیتاستور) با جهت جریان چپ‌به‌راست "
        "یا بالا‌به‌پایین که هرگز قطری نمی‌شود؛ (۲) هر باکس فقط نام + یک خط نقش — دیاگرام حاملِ متنِ "
        "مستند نیست؛ (۳) خطوط با پیکانِ جهت‌دار و برچسبِ پروتکل (HTTP، gRPC، WS) روی مسیر؛ (۴) پالت "
        "محدود تیره با چهار لایهٔ خاکستری + دو رنگ تأکیدی (سبز مسیرِ سالم، قرمز مسیرِ خطا) هماهنگ با "
        "دیزاین‌سیستم بی‌کروم کیان؛ (۵) متن SVG با font-family سیستم و سایز ≥ ۱۲px برای خوانایی RTL. "
        "برای کیان: دیاگرام‌های مادهٔ معماری (M2 دفتر، M4 مغز، M6 بات) و گزارش‌های مهاجرت با همین استاندارد "
        "بازتولید می‌شوند. منبع: skills/creative/architecture-diagram گنجینهٔ NousResearch.",
    ),
    (
        "kb:skill-nous-baoyu-infographic",
        "مهارت اینفوگرافیک: سنتز بصری پرتراکم با ۲۱ چیدمان × ۲۱ سبک برای خلاصهٔ داده‌های متراکم",
        "مهارت اینفوگرافیکِ پرتراکم (پورت از baoyu-skills جیم‌لیو): تبدیل گزارش/دادهٔ متراکم به یک "
        "اینفوگرافیک تک‌صفحه‌ای با ۲۱ چیدمان استاندارد (timeline، comparison، hierarchy، flow، matrix، "
        "KPI-strip و …) × ۲۱ سبک بصری (flat، glassmorphism، blueprint، editorial و …) — انتخاب چیدمان "
        "بر اساس «شکلِ داده» نه سلیقه: دادهٔ زمانی → timeline؛ مقایسهٔ دو موجودیت → split-comparison؛ "
        "سلسله‌مراتب → tree؛ شمارنده‌ها → KPI-strip. قواعد سخت: (۱) هر عدد منبع دارد — برچسب مبدأ "
        "(basis: read | given | own-knowledge) زیر هر بلوک داده؛ بدون برچسب رندر نمی‌شود؛ (۲) تراکم "
        "کنترل‌شده — حداکثر ۹ بلوک در صفحه، هر بلوک ≤ ۲ خط متن، بقیه به زیر-صفحه؛ (۳) تایپوگرافی "
        "سلسله‌مراتب‌دار با سه سطح و خط پایهٔ ثابت؛ (۴) خروجی تک‌فایل HTML با CSS درون‌خطی و بدون JS "
        "سنگین — انیمیشن فقط CSS؛ (۵) RTL فارسی با منطق فیزیکی. در کیان، خروجی استودیو برای "
        "«گزارش‌های یک‌نگاهی» همین مهارت را به‌عنوان قالبِ پیش‌فرض می‌شناسد. منبع: skills/creative/baoyu-infographic گنجینهٔ NousResearch.",
    ),
    (
        "kb:skill-nous-systematic-debugging",
        "مهارت دیباگ: چهار فازِ ریشه‌یابی سیستماتیک — اول بفهم، بعد دست بزن (Root-Cause First)",
        "پروتکل دیباگ چهارفازی با قانون مرکزی «هیچ فیکسِ قبل از ریشه‌یابی» — پورت از obra/superpowers: "
        "فاز ۱ **بازتولید** — باگ باید قطعی بازتولید شود؛ باگی که بازتولید نمی‌شود حدس است نه باگ، و "
        "همین‌جا جست‌وجو به سمتِ شرایطِ محیطی می‌رود؛ فاز ۲ **جداسازی** — کوچک‌ترین بازتولیدِ ممکن، "
        "دوگانیِ فرضیه‌ها (باینری‌سرچ روی لایه‌ها: ورودی؟ استیت؟ شبکه؟؟)، خواندنِ پیام خطای واقعی به‌جای "
        "حدس زدن؛ فاز ۳ **ریشه‌یابی** — زنجیرهٔ علت تا نقطهٔ اولیه؛ سه پرسش: این مقدار از کجا آمد؟ چرا "
        "این مسیر انتخاب شد؟ چه فرضی غلط بود؟ اگر ریشه پیدا نشد، فیکس ممنوع؛ فاز ۴ **ترمیم + اثبات** — "
        "کوچک‌ترین تغییرِ ممکن روی ریشه، تستِ بازتولیدِ فاز ۱ به‌عنوان اثباتِ ترمیم، و جست‌وجوی همان "
        "الگو در نقاط مشابه. ممنوعیت‌های صریح: no shotgun debugging (چند تغییر هم‌زمان)، no «شاید درست "
        "شد» (بدون بازتولید سبز، تمام نیست)، no try/catch خفه‌کننده. در کیان این پروتکل جریانِ کاریِ "
        "پیش‌فرضِ هر فرمانِ «این باگ را درست کن» است. منبع: skills/software-development/systematic-debugging گنجینهٔ NousResearch.",
    ),
    (
        "kb:skill-nous-test-driven-development",
        "مهارت TDD: پروتکل سختگیرانهٔ RED-GREEN-REFACTOR — تست قبل از کد، همیشه",
        "پروتکل سختگیرانهٔ توسعهٔ تست‌محور (پورت از obra/superpowers): قانونِ آهنین «هیچ کدِ تولیدی بدون "
        "تستِ شکست‌خوردهٔ قبلی نوشته نمی‌شود». چرخهٔ سه‌گانه: **RED** — تستِ کوچکِ بعدی را می‌نویسی و "
        "اجرا می‌کنی؛ باید به‌دلیلِ «قابلیتِ موجود نه» شکست بخورد نه به‌دلیلِ خطای کامپایل — شکستِ قرمز "
        "باید دقیقاً همان ادعای رفتاری را اثبات کند؛ **GREEN** — کوچک‌ترین کدِ ممکن که فقط همین تست را "
        "سبز کند — نه جلوتر، نه بهتر؛ بهینه‌سازی اینجا ممنوع است؛ **REFACTOR** — با تست‌های سبز، "
        "بهبودِ ساختار (نام، تکرار، وابستگی) با همان رفتار؛ در هر گامِ ریفاکتور تست‌ها دوباره سبز "
        "بمانند. قواعد سختگیرانه: هرگز بیش از یک چرخهٔ قرمز هم‌زمان نداشته باش؛ هرگز تست را با کد "
        "هم‌زمان «اصلاح» نکن (اول یکی را انتخاب کن)؛ تستِ حذف‌شده = رفتارِ حذف‌شده — باید صریح اعلام "
        "شود؛ در Rust: هر bug fix با تستِ بازتولیدِ همان باگ باز می‌شود. در کیان این مهارت جریانِ "
        "پیش‌فرضِ توسعهٔ crateهاست — ۳۶۰ تستِ فعلی با همین دیسیپلین ساخته شده‌اند. منبع: "
        "skills/software-development/test-driven-development گنجینهٔ NousResearch.",
    ),
    (
        "kb:skill-nous-codebase-inspection",
        "مهارت کاوش کدبیس: باستان‌شناسی ساختاری سریع — LOC، نسبت زبان‌ها و نقشهٔ حرارتی قبل از هر تغییر",
        "باستان‌شناسیِ ساختاریِ سریعِ کدبیس پیش از هر تغییرِ بزرگ: آمارِ خطوط کد و نسبت زبان‌ها (پورت "
        "pygount به لوجیکِ داخلی)، فایل‌های حجیمِ ریسک‌دار، داغ‌ترین ماژول‌ها (بیشترین ایمپورت/وابستگی)، "
        "و نقشهٔ ورود به مخزن برای عاملِ تازه‌وارد. پروتکل سه‌گام: **گام ۱ — نقشه**: درخت دایرکتوری تا "
        "عمق ۳، تفکیک crates/packages/apps، شناسایی هستهٔ دامنه از شِل‌ها؛ **گام ۲ — اندازه‌گیری**: "
        "LOC به‌ازای هر crate، بزرگ‌ترین فایل‌ها (فایل > ۱۰۰۰ خط = ریسکِ بازبینی)، نسبت تست به کد، "
        "نقاطِ ورود (main.rs، lib.rs، routeها)؛ **گام ۳ — گزارش**: سه پاراگرافِ پاسخ‌گو — «کجا شروع "
        "کنم»، «چه چیزی خطرناک است»، «چه چیزی پایدار است» — با عدد و مسیر، نه توصیفِ مبهم. ممنوع: "
        "خواندنِ همهٔ فایل‌ها (فقط نقاط ورود + فایل‌های داغ)، حدس زدنِ معماری بدون دیدنِ Cargo.toml/ترتیب "
        "ماژول‌ها. در کیان: هر تسکِ «این مخزن را بررسی کن» با همین پروتکل و خروجیِ ساختاریافته پاسخ "
        "می‌دهد. منبع: skills/software-development/codebase-inspection گنجینهٔ NousResearch.",
    ),
    (
        "kb:skill-nous-grounded-citations",
        "مهارت استناد زمین‌گیر: هر ادعا یک منبع قابل‌راستی‌آزمایی — ممیزی ادعا به ادعا",
        "استنادِ زمین‌گیرِ پاسخ‌ها و مستندات (پورت Teknium): هر ادعای واقعی، یک منبعِ قابل‌راستی‌آزمایی "
        "دارد و هر منبع، واقعاً باز شده نه از حافظهٔ مدل حدس زده شده. پروتکل: (۱) تفکیکِ ادعا — متنِ "
        "خروجی به ادعاهای اتمی تجزیه می‌شود؛ هر ادعای آماری/زمانی/انتسابی باید نشانی داشته باشد؛ "
        "(۲) سه‌گانهٔ منبع — مبدأ اولیه (سند رسمی، کد، لاگ) بر مبدأ ثانویه (خبر، بلاگ) مقدم است؛ هر "
        "استناد شامل نشانی + تاریخِ دسترسی + نقلِ قولِ دقیقِ حاملِ ادعا؛ (۳) ممیزی معکوس — بعد از "
        "نوشتن، هر استناد دوباره باز می‌شود و نقلِ قول با ادعا تطبیق داده می‌شود؛ استنادی که ادعا را "
        "پشتیبانی نکند حذف یا ادعا اصلاح می‌شود؛ (۴) برچسبِ بی‌منبع — ادعاهای دانشِ داخلیِ مدل صریحاً "
        "«basis: own-knowledge» برچسب می‌خورند و در گزارش‌های مالی/قانونی ممنوع‌اند (هم‌راستا با "
        "kb:skill-a2ui-self-healing-loop)؛ (۵) عدمِ قطعیت رک — «یافتم نشد» بهتر از منبعِ حدسی است. "
        "در کیان: پاسخ‌های پژوهشیِ مغز M4 و گزارش‌های ممیزی با همین پروتکل تولید می‌شوند. منبع: "
        "skills/research/grounded-citations گنجینهٔ NousResearch.",
    ),
    (
        "kb:skill-nous-product-price-monitor",
        "مهارت رصد قیمت: پایش محصول/پرواز/لیستینگ با آستانهٔ هدف و هشدار هوشمند",
        "رصدِ قیمتِ محصول، پرواز یا لیستینگ با آستانهٔ هدف و هشدار (پورت hermes-agent): معماریِ سه‌جزئی — "
        "رصدگر دوره‌ای (کرون، نه حلقهٔ فشرده — هم‌راستا با کف ۱۵ دقیقهٔ گارد خستگی کیان)، ذخیره‌سازِ "
        "تاریخچهٔ قیمت (حداقل قیمت/آخرین قیمت/روند)، و هشدارِ آستانه‌ای (قیمت ≤ هدف → اعلان یک‌بار؛ نه "
        "اسپام هر ران). قواعد: (۱) هر رصد یک شناسهٔ پایدار (URL نرمال‌شده یا SKU) دارد تا ران‌های بعدی "
        "به همان رکورد می‌چسبند؛ (۲) خوانشِ قیمت از ساختار (JSON-LD، schema.org، سلکتور مشخص) نه "
        "regex مبهم روی کل صفحه — قیمتِ کاذبِ متنِ رزرو بدتر از نبودِ داده است؛ (۳) هر تغییرِ قیمت با "
        "نرخِ تغییر ثبت می‌شود؛ نوسانِ زیر نویز (مثلاً < ۱٪) هشدار نمی‌دهد؛ (۴) رزرو از درونِ سندباکس "
        "هرگز کلید/کوکی کاربر نمی‌گیرد — فقط منابع عمومی؛ (۵) شکستِ خوانش = رکوردِ «ناموفق خوانده شد» "
        "نه آخرین قیمتِ کهنه (دادهٔ کهنه خطرناک‌تر از نبودِ داده است). کاربرد کیان: پایشِ قیمتِ "
        "محصولاتِ مارکت‌پلیس و گزارشِ روند برای فروشنده‌ها. منبع: skills/productivity/product-price-monitor گنجینهٔ NousResearch.",
    ),
    (
        "kb:skill-nous-hermes-function-calling",
        "مهارت پروتکل فراخوانی: اسکیمای ‎<tools>‎/‎<tool_call>‎ XML+JSON هرمس و استاندارد افشای تدریجی",
        "استاندارد فراخوانی ابزارِ Hermes-Function-Calling برای مدل‌های وزن‌باز بدون پشتیبانی بومیِ ارائه‌دهنده: ابزارها داخل تگ ‎<tools>…</tools>‎ در پرامپت سیستم می‌آیند و مدل فراخوانی را درون ‎<tool_call>{\"name\": ..., \"arguments\": {...}}</tool_call>‎ در متنِ پاسخ می‌نویسد. قواعد سخت: "
        "(۱) JSON دوتاییِ معتبر — ویرگول انتهایی و کلیدِ یک‌نقل‌قولی فقط با فال‌بکِ آسان‌گیرِ پارسر پذیرفته می‌شود، هرگز با eval؛ (۲) یک فراخوانی در هر دور — مدل نتیجه را در ‎<tool_call>tool_response</tool_call>‎ می‌گیرد و بعد تحلیل/فراخوانی بعدی؛ حدس زدنِ نتیجهٔ ابزار قبل از رسیدنِ ‎<tool_call>tool_response</tool_call>‎ ممنوع است؛ "
        "(۳) اسکیمای هر ابزار با JSON Schema کامل (پارامتر، نوع، توضیح) — مدل‌های آزاد بدون اسکیما پارامترها را می‌سازند نه استدلال؛ (۴) پارسِ فال‌بک: بلوک‌های خراب پرت می‌شوند و بقیه ادامه می‌یابند، با منطقِ انباشتگرِ استریم (نامِ تابع = انتساب، جداسازی با عمق براکت). در کیان پورتِ Rust این استاندارد در crates/ai/src/tool_parser.rs است (<tool_call>extract_xml_tool_calls</tool_call> + ToolCallStreamAccumulator) و لایهٔ حافظه، فهرست مهارت‌ها را با «افشای تدریجی» می‌سازد — فقط «نام: توضیحِ یک‌خطی» در پرامپت، بدنه فقط با بارگذاری صریح. منبع: پوشهٔ ۰۵ گنجینهٔ NousResearch."
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
    trial = cur.execute("SELECT COALESCE(SUM(amount),0) FROM Posting").fetchone()[0]
    timings = []
    for q in ("دیزاین‌سیستم", "p5.js", "ریشه‌یابی", "RED-GREEN-REFACTOR", "افشای تدریجی"):
        t0 = time.perf_counter()
        rows = cur.execute(
            'SELECT COUNT(*) FROM "KbEntry" WHERE "title" LIKE ? OR "body" LIKE ?',
            (f"%{q}%", f"%{q}%")).fetchone()[0]
        timings.append((q, rows, (time.perf_counter() - t0) * 1000))
    return {"kb": kb, "active": active, "trial_sum": trial, "search": timings}


def main() -> int:
    assert REPO.is_dir() and SEED_DB.is_file(), "repo/seed db missing"

    before = verify(sqlite3.connect(SEED_DB))
    print(f"[1/3] before: KbEntry={before['kb']} | active products={before['active']} | "
          f"trial balance sum={before['trial_sum']}")

    db = sqlite3.connect(SEED_DB)
    db.row_factory = sqlite3.Row
    try:
        ks = upsert_kb(db)
        db.commit()
    finally:
        db.close()
    print(f"[2/3] kb upsert: +{ks['inserted']} new, {ks['updated']} updated, {ks['skipped']} skipped")

    after = verify(sqlite3.connect(SEED_DB))
    print(f"[3/3] after: KbEntry={after['kb']} (39 -> 49 expected) | active products={after['active']} | "
          f"trial balance sum={after['trial_sum']}")
    for q, rows, ms in after["search"]:
        flag = "OK" if ms < 5.0 else "SLOW"
        print(f"      search '{q}': {rows} hits in {ms:.2f} ms [{flag}]")

    # sync to download deliverable
    DOWNLOAD_DB.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(SEED_DB, DOWNLOAD_DB)
    print(f"      synced seed db -> {DOWNLOAD_DB}")

    ok = (after["kb"] == 49 and after["active"] == 20 and after["trial_sum"] == 0
          and ks["updated"] == 0 and (ks["inserted"] == 10 or ks["skipped"] == 10)
          and all(ms < 5.0 for _, _, ms in after["search"]))
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
