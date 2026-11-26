"use client";

import { motion } from "motion/react";
import { ArrowLeft, Boxes, Layers, PackageX } from "lucide-react";
import { LEGACY_REPOS } from "@/lib/kian-data";

/** Consolidation story: 18 legacy repos → 8 value pieces → 13 core crates */
export function Journey() {
  return (
    <section id="architecture" className="scroll-mt-24 border-y border-royal-500/15 bg-zinc-950/60 py-20">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        <div className="mb-12 text-center">
          <h2 className="text-3xl font-black sm:text-4xl">
            &#x627;&#x632; &#x6f1;&#x6f8; &#x645;&#x62e;&#x632;&#x646; &#x628;&#x647; <span className="text-gold-400">&#x6cc;&#x6a9; &#x67e;&#x627;&#x62f;&#x634;&#x627;&#x647;&#x6cc;</span>
          </h2>
          <p className="mx-auto mt-3 max-w-2xl leading-7 text-zinc-400">
            &#x647;&#x631; &#x642;&#x637;&#x639;&#x647; &#x627;&#x631;&#x632;&#x634;&#x645;&#x646;&#x62f;&#x60c; &#x628;&#x647;&#x62a;&#x631;&#x6cc;&#x646; &#x646;&#x633;&#x62e;&#x647; &#x62e;&#x648;&#x62f;&#x634; &#x631;&#x627; &#x62f;&#x631; &#x6a9;&#x631;&#x646;&#x644; &#x62c;&#x62f;&#x6cc;&#x62f; &#x67e;&#x6cc;&#x62f;&#x627; &#x645;&#x6cc;‌&#x6a9;&#x646;&#x62f; — &#x633;&#x647; &#x632;&#x628;&#x627;&#x646; &#x628;&#x647; &#x62f;&#x648; &#x632;&#x628;&#x627;&#x646;
            (Rust + TypeScript) &#x62e;&#x644;&#x627;&#x635;&#x647; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
          </p>
        </div>

        <div className="grid items-stretch gap-4 md:grid-cols-3">
          {/* Step 1: legacy */}
          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true, margin: "-80px" }}
            transition={{ duration: 0.5 }}
            className="rounded-3xl border border-zinc-800 bg-zinc-900/60 p-6"
          >
            <div className="mb-4 flex items-center gap-2 text-zinc-500">
              <PackageX className="size-5" aria-hidden />
              <h3 className="font-bold">&#x6f1;&#x6f8; &#x645;&#x62e;&#x632;&#x646; legacy</h3>
            </div>
            <div className="flex max-h-56 flex-wrap gap-1.5 overflow-y-auto pb-1 [scrollbar-width:thin]" aria-label="&#x641;&#x647;&#x631;&#x633;&#x62a; &#x645;&#x62e;&#x632;&#x646;‌&#x647;&#x627;&#x6cc; &#x642;&#x62f;&#x6cc;&#x645;&#x6cc;">
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
              &#x6f5; &#x62e;&#x637; &#x645;&#x62d;&#x635;&#x648;&#x644;&#x60c; &#x6f3; &#x632;&#x628;&#x627;&#x646;&#x60c; &#x642;&#x637;&#x639;&#x627;&#x62a; &#x62a;&#x6a9;&#x631;&#x627;&#x631;&#x6cc; — &#x627;&#x631;&#x632;&#x634; &#x648;&#x627;&#x642;&#x639;&#x6cc; &#x641;&#x642;&#x637; &#x62f;&#x631; &#x6f8; &#x642;&#x637;&#x639;&#x647; &#x628;&#x648;&#x62f;.
            </p>
          </motion.div>

          {/* Step 2: pieces */}
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
              <h3 className="font-bold">&#x6f8; &#x642;&#x637;&#x639;&#x647; &#x627;&#x631;&#x632;&#x634;&#x645;&#x646;&#x62f;</h3>
            </div>
            <ul className="space-y-2.5 text-sm leading-6 text-zinc-300">
              <li>&#x645;&#x648;&#x62a;&#x648;&#x631; &#x67e;&#x648;&#x644; — Din&#x60c; &#x62f;&#x641;&#x62a;&#x631; &#x62f;&#x648;&#x637;&#x631;&#x641;&#x647;&#x60c; &#x627;&#x645;&#x627;&#x646;&#x62a; &#x6f8; &#x62d;&#x627;&#x644;&#x62a;&#x647;</li>
              <li>&#x6a9;&#x6cc;&#x641; &#x67e;&#x648;&#x644; &#x686;&#x646;&#x62f;&#x632;&#x646;&#x62c;&#x6cc;&#x631;&#x647;‌&#x627;&#x6cc; USDT + &#x62e;&#x632;&#x627;&#x646;&#x647;</li>
              <li>&#x6af;&#x6cc;&#x62a;‌&#x648;&#x6cc; AI — fallback&#x60c; &#x628;&#x648;&#x62f;&#x62c;&#x647;&#x60c; redaction</li>
              <li>&#x645;&#x648;&#x62a;&#x648;&#x631; &#x647;&#x631;&#x645;&#x633; + &#x6f1;&#x6f6; &#x645;&#x647;&#x627;&#x631;&#x62a;</li>
              <li>&#x67e;&#x627;&#x631;&#x644;&#x645;&#x627;&#x646; HITL — &#x622;&#x633;&#x62a;&#x627;&#x646;&#x647; &#x6f5;&#x6f0;,&#x6f0;&#x6f0;&#x6f0; Đ</li>
              <li>&#x67e;&#x631;&#x648;&#x62a;&#x6a9;&#x644; &#x627;&#x6cc;&#x62c;&#x646;&#x62a;‌&#x647;&#x627; — A2A&#x60c; MCP&#x60c; &#x646;&#x627;&#x648;&#x6af;&#x627;&#x646;</li>
              <li>&#x642;&#x631;&#x627;&#x631;&#x62f;&#x627;&#x62f; OpenAPI 3.1 + SDK &#x62e;&#x648;&#x62f;&#x6a9;&#x627;&#x631;</li>
              <li>&#x6a9;&#x627;&#x646;&#x627;&#x644;‌&#x647;&#x627; — &#x631;&#x628;&#x627;&#x62a;&#x60c; &#x645;&#x6cc;&#x646;&#x6cc;‌&#x627;&#x67e;&#x60c; &#x648;&#x6cc;&#x62c;&#x62a;</li>
            </ul>
          </motion.div>

          {/* Step 3: core */}
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
              <h3 className="font-bold">&#x6f1;&#x6f3; &#x6a9;&#x631;&#x62a; &#x6a9;&#x631;&#x646;&#x644; KIAN</h3>
            </div>
            <p className="font-mono text-xs leading-6 text-zinc-400" dir="ltr">
              domain · db · contracts · server · bot · nation · agents · ai · hermes · wallet ·
              superapp · observe · tooling
            </p>
            <div className="mt-4 space-y-2 text-sm text-zinc-300">
              <p className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-gold-400" /> forbid(unsafe_code) &#x62f;&#x631; &#x647;&#x645;&#x647; &#x6a9;&#x631;&#x62a;‌&#x647;&#x627;
              </p>
              <p className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-gold-400" /> dual-run &#x628;&#x631;&#x627;&#x6cc; &#x645;&#x647;&#x627;&#x62c;&#x631;&#x62a; &#x627;&#x645;&#x646;
              </p>
              <p className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-gold-400" /> &#x6af;&#x6cc;&#x62a; REA &#x631;&#x648;&#x6cc; &#x647;&#x631; &#x627;&#x646;&#x62a;&#x634;&#x627;&#x631;
              </p>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
