"use client";

import { motion } from "motion/react";
import { ArrowLeft, Boxes, Layers, PackageX } from "lucide-react";
import { LEGACY_REPOS } from "@/lib/kian-data";

/** داستان تثبیت: ۱۸ مخزن legacy → ۸ قطعه ارزش → ۱۳ کرت کرنل */
export function Journey() {
  return (
    <section id="architecture" className="scroll-mt-24 border-y border-royal-500/15 bg-zinc-950/60 py-20">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        <div className="mb-12 text-center">
          <h2 className="text-3xl font-black sm:text-4xl">
            از ۱۸ مخزن به <span className="text-gold-400">یک پادشاهی</span>
          </h2>
          <p className="mx-auto mt-3 max-w-2xl leading-7 text-zinc-400">
            هر قطعه ارزشمند، بهترین نسخه خودش را در کرنل جدید پیدا می‌کند — سه زبان به دو زبان
            (Rust + TypeScript) خلاصه می‌شود.
          </p>
        </div>

        <div className="grid items-stretch gap-4 md:grid-cols-3">
          {/* گام ۱: legacy */}
          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true, margin: "-80px" }}
            transition={{ duration: 0.5 }}
            className="rounded-3xl border border-zinc-800 bg-zinc-900/60 p-6"
          >
            <div className="mb-4 flex items-center gap-2 text-zinc-500">
              <PackageX className="size-5" aria-hidden />
              <h3 className="font-bold">۱۸ مخزن legacy</h3>
            </div>
            <div className="flex max-h-56 flex-wrap gap-1.5 overflow-y-auto pb-1 [scrollbar-width:thin]" aria-label="فهرست مخزن‌های قدیمی">
              {LEGACY_REPOS.map((r) => (
                <span
                  key={r}
                  className="rounded-lg border border-zinc-800 bg-zinc-800/60 px-2 py-1 font-mono text-[11px] text-zinc-500"
                >
                  {r}
                </span>
              ))}
            </div>
            <p className="mt-4 text-xs leading-6 text-zinc-500">
              ۵ خط محصول، ۳ زبان، قطعات تکراری — ارزش واقعی فقط در ۸ قطعه بود.
            </p>
          </motion.div>

          {/* گام ۲: قطعات */}
          <motion.div
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-80px" }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="relative rounded-3xl border border-royal-500/30 bg-royal-600/10 p-6"
          >
            <div className="absolute inset-x-0 -top-3 mx-auto hidden h-6 w-6 items-center justify-center rounded-full border border-royal-500/50 bg-zinc-950 text-royal-400 md:flex">
              <ArrowLeft className="size-3.5" aria-hidden />
            </div>
            <div className="mb-4 flex items-center gap-2 text-royal-400">
              <Layers className="size-5" aria-hidden />
              <h3 className="font-bold">۸ قطعه ارزشمند</h3>
            </div>
            <ul className="space-y-2.5 text-sm leading-6 text-zinc-300">
              <li>موتور پول — Din، دفتر دوطرفه، امانت ۸ حالته</li>
              <li>کیف پول چندزنجیره‌ای USDT + خزانه</li>
              <li>گیت‌وی AI — fallback، بودجه، redaction</li>
              <li>موتور هرمس + ۱۶ مهارت</li>
              <li>پارلمان HITL — آستانه ۵۰,۰۰۰ Đ</li>
              <li>پروتکل ایجنت‌ها — A2A، MCP، ناوگان</li>
              <li>قرارداد OpenAPI 3.1 + SDK خودکار</li>
              <li>کانال‌ها — ربات، مینی‌اپ، ویجت</li>
            </ul>
          </motion.div>

          {/* گام ۳: کرنل */}
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true, margin: "-80px" }}
            transition={{ duration: 0.5, delay: 0.3 }}
            className="relative rounded-3xl border border-gold-400/40 bg-gradient-to-b from-royal-600/20 to-zinc-900/70 p-6 shadow-xl shadow-royal-600/20"
          >
            <div className="absolute inset-x-0 -top-3 mx-auto hidden h-6 w-6 items-center justify-center rounded-full border border-gold-400/50 bg-zinc-950 text-gold-400 md:flex">
              <ArrowLeft className="size-3.5" aria-hidden />
            </div>
            <div className="mb-4 flex items-center gap-2 text-gold-400">
              <Boxes className="size-5" aria-hidden />
              <h3 className="font-bold">۱۳ کرت کرنل KIAN</h3>
            </div>
            <p className="font-mono text-xs leading-6 text-zinc-400" dir="ltr">
              domain · db · contracts · server · bot · nation · agents · ai · hermes · wallet ·
              superapp · observe · tooling
            </p>
            <div className="mt-4 space-y-2 text-sm text-zinc-300">
              <p className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-gold-400" /> forbid(unsafe_code) در همه کرت‌ها
              </p>
              <p className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-gold-400" /> dual-run برای مهاجرت امن
              </p>
              <p className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-gold-400" /> گیت REA روی هر انتشار
              </p>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
