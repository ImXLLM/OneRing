# ADR-0001 — بنیان KIAN

**وضعیت:** پذیرفته‌شده — 2026-11-20

## زمینه
۵ خط محصول legacy (Python نسل۱، TS نسل۲ و ۳، Rust نسل۴، پرچم‌دار 10x) با قطعات تکراری.
هدف: یک اکوسیستم واحد، امن و قابل توسعه با فرانتاند اولویت اول.

## تصمیم
1. **نام:** KIAN (کیان) — «اصل، سرچشمه، پادشاهی»؛ پیوستگی برند با KiaAgentX؛ هم‌خوان با استعارهٔ nation/parliament.
2. **معماری:** مونوریپو — کرنل Rust (۱۳ کرت) + فرانتاند TypeScript (Next.js 16.4 / React 19.3 / Tailwind 4.3).
3. **قرارداد اول:** DTOها در Rust منبع حقیقت‌اند؛ OpenAPI 3.1 → SDK خودکار → Zod mirror.
4. **دروازه REA 10/10:** هیچ آرتیفکتی افشا نمی‌کند (سورس‌مپ، کریدنشال، debug companion، متادیتای Rust) — docs/rea-gates.md.
5. **زبان‌ها:** Rust + TypeScript + SQL + Shell. Python legacy بیرون از مونوریپو می‌ماند تا parity کامل شود.

## پیامدها
- حذف سه‌گانه‌سازی Python/TS/Rust در بلندمدت؛ dual-run در `crates/observe` مهاجرت امن را تضمین می‌کند.
- فرانتاند اول: M1 = apps/web + packages/ui.
