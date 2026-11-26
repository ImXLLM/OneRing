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
            From 18 repos to <span className="text-gold-400">one kingdom</span>
          </h2>
          <p className="mx-auto mt-3 max-w-2xl leading-7 text-zinc-400">
            Every valuable piece finds its best version in the new kernel — three languages condensed into two
            (Rust + TypeScript).
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
              <h3 className="font-bold">18 legacy repos</h3>
            </div>
            <div className="flex max-h-56 flex-wrap gap-1.5 overflow-y-auto pb-1 [scrollbar-width:thin]" aria-label="the list of legacy repos">
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
              5 product lines, 3 languages, duplicated pieces — the real value sat in only 8 pieces.
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
              <h3 className="font-bold">8 valuable pieces</h3>
            </div>
            <ul className="space-y-2.5 text-sm leading-6 text-zinc-300">
              <li>The money engine — Din, the double-entry ledger, the 8-state escrow</li>
              <li>The multichain USDT wallet + treasury</li>
              <li>The AI gateway — fallback, budget, redaction</li>
              <li>The Hermes engine + 16 skills</li>
              <li>The HITL parliament — the 50,000 Đ threshold</li>
              <li>The agent protocol — A2A, MCP, the fleet</li>
              <li>The OpenAPI 3.1 contract + the automatic SDK</li>
              <li>The channels — bot, mini-app, widget</li>
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
              <h3 className="font-bold">13 MAIRON kernel crates</h3>
            </div>
            <p className="font-mono text-xs leading-6 text-zinc-400" dir="ltr">
              domain · db · contracts · server · bot · nation · agents · ai · hermes · wallet ·
              superapp · observe · tooling
            </p>
            <div className="mt-4 space-y-2 text-sm text-zinc-300">
              <p className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-gold-400" /> forbid(unsafe_code) in every crate
              </p>
              <p className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-gold-400" /> dual-run for a safe migration
              </p>
              <p className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-gold-400" /> the REA gate on every release
              </p>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
