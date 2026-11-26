"use client";

import { motion } from "motion/react";
import { CheckCircle2, Circle, Clock, Hammer } from "lucide-react";
import { Badge } from "@kian/ui";
import { ROADMAP } from "@/lib/kian-data";

export function Roadmap() {
  return (
    <section id="roadmap" className="scroll-mt-24 py-20">
      <div className="mx-auto max-w-4xl px-4 sm:px-6">
        <div className="mb-12 text-center">
          <h2 className="text-3xl font-black sm:text-4xl">
            &#x646;&#x642;&#x634;&#x647; &#x631;&#x627;&#x647; — <span className="text-royal-400">&#x6f5; &#x6af;&#x627;&#x645;</span> \u062a\u0627 \u067e\u0627\u062f\u0634\u0627\u0647\u06cc \u06a9\u0627\u0645\u0644
          </h2>
          <p className="mx-auto mt-3 max-w-2xl leading-7 text-zinc-400">
            &#x647;&#x6cc;&#x686; &#x6af;&#x627;&#x645;&#x6cc; &#x6cc;&#x6a9;‌&#x634;&#x627;&#x62a; &#x633;&#x627;&#x62e;&#x62a;&#x647; &#x646;&#x645;&#x6cc;‌&#x634;&#x648;&#x62f;&#x61b; &#x647;&#x631; &#x645;&#x627;&#x6cc;&#x644;&#x633;&#x62a;&#x648;&#x646; &#x62c;&#x62f;&#x627; &#x642;&#x627;&#x628;&#x644; &#x62a;&#x62d;&#x648;&#x6cc;&#x644;&#x60c; &#x628;&#x6a9;‌&#x622;&#x67e;‌&#x67e;&#x630;&#x6cc;&#x631; &#x648; &#x639;&#x628;&#x648;&#x631;&#x6a9;&#x646;&#x646;&#x62f;&#x647; &#x627;&#x632;
            &#x6af;&#x6cc;&#x62a; &#x627;&#x645;&#x646;&#x6cc;&#x62a;&#x6cc; &#x62e;&#x648;&#x62f;&#x634; &#x627;&#x633;&#x62a;.
          </p>
        </div>

        <ol className="relative space-y-6 border-r border-royal-500/25 pr-6">
          {ROADMAP.map((m, i) => {
            const active = m.status === "active";
            const done = m.status === "done";
            return (
              <motion.li
                key={m.id}
                initial={{ opacity: 0, x: -24 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true, margin: "-60px" }}
                transition={{ duration: 0.45, delay: i * 0.06 }}
                className={`relative rounded-3xl border p-6 transition-colors ${
                  active
                    ? "border-gold-400/50 bg-gradient-to-bl from-royal-600/20 to-zinc-900/80 shadow-lg shadow-royal-600/20"
                    : done
                      ? "border-emerald-500/30 bg-zinc-900/60"
                      : "border-zinc-800 bg-zinc-900/60"
                }`}
              >
                {/* Point on the timeline */}
                <span
                  aria-hidden
                  className={`absolute -right-[31px] top-8 flex size-5 items-center justify-center rounded-full border ${
                    active
                      ? "border-gold-400 bg-gold-400 text-zinc-950"
                      : done
                        ? "border-gold-400/60 bg-zinc-950 text-gold-400"
                        : "border-royal-500/60 bg-zinc-950 text-royal-400"
                  }`}
                >
                  {active ? (
                    <Hammer className="size-3" />
                  ) : done ? (
                    <CheckCircle2 className="size-3.5" />
                  ) : (
                    <span className="size-1.5 rounded-full bg-current" />
                  )}
                </span>

                <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
                  <div className="flex items-center gap-3">
                    <span
                      className={`font-mono text-sm font-bold ${active ? "text-gold-400" : "text-royal-400"}`}
                      dir="ltr"
                    >
                      {m.id}
                    </span>
                    <h3 className="text-lg font-bold">{m.title}</h3>
                  </div>
                  {active ? (
                    <Badge className="gap-1.5 border-gold-400/50 bg-gold-400/15 text-gold-400 hover:bg-gold-400/15">
                      <span className="relative flex size-2">
                        <span className="absolute inline-flex size-full animate-ping rounded-full bg-gold-400 opacity-60" />
                        <span className="relative inline-flex size-2 rounded-full bg-gold-400" />
                      </span>
                      &#x62f;&#x631; &#x62d;&#x627;&#x644; &#x633;&#x627;&#x62e;&#x62a;
                    </Badge>
                  ) : done ? (
                    <Badge variant="success" className="gap-1 border-emerald-500/40 bg-emerald-500/10 text-emerald-400">
                      <CheckCircle2 className="size-3" /> &#x62a;&#x645;&#x627;&#x645; &#x634;&#x62f; — &#x6af;&#x6cc;&#x62a; REA &#x67e;&#x627;&#x633;
                    </Badge>
                  ) : m.status === "next" ? (
                    <Badge variant="secondary" className="gap-1.5 border-royal-500/40 bg-royal-500/10 text-royal-400">
                      <Clock className="size-3" /> &#x628;&#x639;&#x62f;&#x6cc;
                    </Badge>
                  ) : (
                    <Badge variant="secondary" className="border-zinc-700 bg-zinc-800/70 text-zinc-500">
                      &#x62f;&#x631; &#x635;&#x641;
                    </Badge>
                  )}
                </div>

                <p className="mb-3 font-mono text-xs text-zinc-500" dir="ltr">
                  {m.scope}
                </p>

                <ul className="grid gap-2 sm:grid-cols-2">
                  {m.items.map((item, j) => {
                    const itemDone = done || (active && j < 4);
                    return (
                      <li key={item} className="flex items-start gap-2 text-sm leading-6">
                        {itemDone ? (
                          <CheckCircle2 className="mt-1 size-4 shrink-0 text-gold-400" aria-label="&#x627;&#x646;&#x62c;&#x627;&#x645; &#x634;&#x62f;&#x647;" />
                        ) : (
                          <Circle className="mt-1 size-4 shrink-0 text-zinc-600" aria-label="&#x627;&#x646;&#x62c;&#x627;&#x645; &#x646;&#x634;&#x62f;&#x647;" />
                        )}
                        <span className={itemDone ? "text-zinc-200" : "text-zinc-400"}>{item}</span>
                      </li>
                    );
                  })}
                </ul>
              </motion.li>
            );
          })}
        </ol>
      </div>
    </section>
  );
}
