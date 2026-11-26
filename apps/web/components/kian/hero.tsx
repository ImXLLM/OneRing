"use client";

import { motion } from "motion/react";
import Image from "next/image";
import { ArrowLeft, ShieldCheck, Hammer } from "lucide-react";
import { Badge, Button } from "@kian/ui";
import { HERO_STATS } from "@/lib/kian-data";

export function Hero() {
  return (
    <section id="top" className="relative overflow-hidden">
      {/* Background: network grid + royal halo */}
      <div aria-hidden className="pointer-events-none absolute inset-0">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(124,58,237,0.06)_1px,transparent_1px),linear-gradient(90deg,rgba(124,58,237,0.06)_1px,transparent_1px)] bg-[size:44px_44px] [mask-image:radial-gradient(ellipse_70%_60%_at_50%_35%,black,transparent)]" />
        <div className="absolute left-1/2 top-[-220px] h-[480px] w-[720px] -translate-x-1/2 rounded-full bg-royal-600/25 blur-[120px]" />
        <div className="absolute bottom-[-160px] right-[-100px] h-[300px] w-[300px] rounded-full bg-gold-400/10 blur-[100px]" />
      </div>

      <div className="relative mx-auto flex max-w-6xl flex-col items-center gap-8 px-4 pb-20 pt-16 text-center sm:px-6 sm:pt-24">
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
        >
          <Badge
            variant="outline"
            className="gap-2 border-royal-500/40 bg-royal-500/10 px-4 py-1.5 text-sm text-royal-400"
          >
            <span className="relative flex size-2">
              <span className="absolute inline-flex size-full animate-ping rounded-full bg-gold-400 opacity-60" />
              <span className="relative inline-flex size-2 rounded-full bg-gold-400" />
            </span>
            M5 &#x6a9;&#x627;&#x645;&#x644; &#x634;&#x62f; — &#x67e;&#x627;&#x631;&#x644;&#x645;&#x627;&#x646; HITL &#x648; &#x648;&#x644;&#x62a; &#x686;&#x647;&#x627;&#x631;&#x632;&#x646;&#x62c;&#x6cc;&#x631;&#x647; &#x632;&#x646;&#x62f;&#x647; &#x627;&#x633;&#x62a;
          </Badge>
        </motion.div>

        {/* Mascot + floating Đ coin */}
        <motion.div
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1, y: [0, -12, 0] }}
          transition={{
            opacity: { duration: 0.6 },
            scale: { duration: 0.6 },
            y: { duration: 5, repeat: Infinity, ease: "easeInOut" },
          }}
          className="relative flex items-center gap-5"
          aria-hidden
        >
          <div className="relative">
            <div className="absolute inset-0 -z-10 rounded-full bg-gold-400/25 blur-2xl" />
            <Image
              src="/mascot.png"
              alt=""
              width={128}
              height={128}
              priority
              className="size-28 rounded-[28%] object-cover ring-2 ring-gold-400/50 shadow-2xl shadow-royal-600/40 sm:size-32"
            />
          </div>
          <div className="relative">
            <div className="absolute inset-0 -z-10 rounded-full bg-gold-400/20 blur-2xl" />
            <div className="flex size-20 items-center justify-center rounded-[28%] bg-gradient-to-b from-[#241245] to-[#5b2bd6] ring-2 ring-gold-400/50 shadow-2xl shadow-royal-600/40 sm:size-24">
              <span className="bg-gradient-to-b from-gold-400 to-amber-500 bg-clip-text text-4xl font-black text-transparent drop-shadow-[0_2px_12px_rgba(245,183,61,0.45)] sm:text-5xl">
                Đ
              </span>
            </div>
          </div>
        </motion.div>

        <motion.h1
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.1 }}
          className="text-balance text-5xl font-black leading-[1.15] sm:text-7xl"
        >
          <span className="bg-gradient-to-b from-white via-royal-400 to-royal-600 bg-clip-text text-transparent">
            KIAN
          </span>
          <span className="mt-2 block text-2xl font-bold text-zinc-300 sm:text-3xl">
            &#x67e;&#x627;&#x62f;&#x634;&#x627;&#x647;&#x6cc;&#x650; &#x627;&#x6cc;&#x62c;&#x646;&#x62a;‌&#x647;&#x627;
          </span>
        </motion.h1>

        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2 }}
          className="max-w-2xl text-pretty leading-8 text-zinc-400"
        >
          &#x627;&#x6a9;&#x648;&#x633;&#x6cc;&#x633;&#x62a;&#x645; &#x62d;&#x627;&#x6a9;&#x645;&#x6cc;&#x62a;‌&#x645;&#x646;&#x62f; &#x627;&#x6cc;&#x62c;&#x646;&#x62a;‌&#x647;&#x627;&#x6cc; &#x647;&#x648;&#x634;&#x645;&#x646;&#x62f; — &#x628;&#x627; &#x627;&#x642;&#x62a;&#x635;&#x627;&#x62f; &#x627;&#x62e;&#x62a;&#x635;&#x627;&#x635;&#x6cc;{" "}
          <span className="font-bold text-gold-400">Din (Đ)</span>\u060c \u0627\u0645\u0627\u0646\u062a \u06f8 \u062d\u0627\u0644\u062a\u0647\u060c \u067e\u0627\u0631\u0644\u0645\u0627\u0646 HITL \u0648
          &#x646;&#x627;&#x648;&#x6af;&#x627;&#x646; &#x627;&#x6cc;&#x62c;&#x646;&#x62a;‌&#x647;&#x627;&#x6cc; A2A. &#x647;&#x633;&#x62a;&#x647; &#x631;&#x648;&#x6cc; <span className="text-royal-400">Rust 1.100</span>\u060c \u0631\u0648\u06cc\u0647
          &#x631;&#x648;&#x6cc; <span className="text-royal-400">Next.js 16</span> — \u0648 \u0647\u0631 \u0622\u0631\u062a\u06cc\u0641\u06a9\u062a \u0628\u0627\u06cc\u062f \u0641\u06cc\u0644\u062a\u0631\u0647\u0627\u06cc REA
          &#x631;&#x627; &#x628;&#x627; &#x635;&#x641;&#x631; &#x627;&#x641;&#x634;&#x627; &#x631;&#x62f; &#x6a9;&#x646;&#x62f;.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.3 }}
          className="flex flex-wrap items-center justify-center gap-3"
        >
          <Button
            asChild
            size="lg"
            className="h-12 bg-royal-500 px-6 text-base font-bold text-white hover:bg-royal-600 shadow-lg shadow-royal-600/30"
          >
            <a href="#subsystems">
              &#x6a9;&#x627;&#x648;&#x634; &#x632;&#x6cc;&#x631;&#x633;&#x6cc;&#x633;&#x62a;&#x645;‌&#x647;&#x627; <ArrowLeft className="size-4" />
            </a>
          </Button>
          <Button
            asChild
            size="lg"
            variant="outline"
            className="h-12 border-zinc-700 bg-zinc-900/60 px-6 text-base text-zinc-200 hover:bg-zinc-800 hover:text-gold-400"
          >
            <a href="#roadmap">
              <Hammer className="size-4" /> &#x646;&#x642;&#x634;&#x647; &#x631;&#x627;&#x647; &#x633;&#x627;&#x62e;&#x62a;
            </a>
          </Button>
        </motion.div>

        {/* Stats */}
        <motion.dl
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.45 }}
          className="mt-6 grid w-full max-w-3xl grid-cols-2 gap-3 sm:grid-cols-4"
        >
          {HERO_STATS.map((s) => (
            <div
              key={s.label}
              className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 px-4 py-5 backdrop-blur"
            >
              <dt className="order-2 mt-1 block text-xs text-zinc-500">{s.label}</dt>
              <dd className="order-1 text-3xl font-black text-gold-400">{s.value}</dd>
            </div>
          ))}
        </motion.dl>

        <motion.p
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.6 }}
          className="mt-2 flex items-center gap-2 text-xs text-zinc-500"
        >
          <ShieldCheck className="size-4 text-royal-400" />
          &#x6af;&#x6cc;&#x62a; &#x646;&#x647;&#x627;&#x6cc;&#x6cc; REA &#x6f1;&#x6f0;/&#x6f1;&#x6f0; — &#x627;&#x632; &#x6a9;&#x631;&#x646;&#x644; &#x67e;&#x648;&#x644; &#x62a;&#x627; &#x67e;&#x627;&#x631;&#x644;&#x645;&#x627;&#x646; &#x648; &#x648;&#x644;&#x62a; &#x686;&#x647;&#x627;&#x631;&#x632;&#x646;&#x62c;&#x6cc;&#x631;&#x647;
        </motion.p>
      </div>
    </section>
  );
}
