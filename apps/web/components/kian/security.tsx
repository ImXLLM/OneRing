"use client";

import { motion } from "motion/react";
import { CheckCircle2, ShieldCheck, Clock } from "lucide-react";
import { REA_LADDER, REA_FILTERS } from "@/lib/kian-data";

/** REA ladder: every milestone is one step toward 10/10 */
export function SecurityREA() {
  return (
    <section
      id="security"
      className="scroll-mt-24 border-y border-royal-500/15 bg-zinc-950/60 py-20"
    >
      <div className="mx-auto max-w-5xl px-4 sm:px-6">
        <div className="mb-12 text-center">
          <h2 className="text-3xl font-black sm:text-4xl">
            &#x6af;&#x627;&#x645;‌&#x628;&#x647;‌&#x6af;&#x627;&#x645; &#x62a;&#x627; <span className="text-gold-400">10 &#x627;&#x632; 10</span> REA
          </h2>
          <p className="mx-auto mt-3 max-w-2xl leading-7 text-zinc-400">
            &#x6cc;&#x6a9;‌&#x634;&#x627;&#x62a; &#x646;&#x645;&#x6cc;‌&#x633;&#x627;&#x632;&#x6cc;&#x645;&#x61b; &#x647;&#x631; &#x645;&#x627;&#x6cc;&#x644;&#x633;&#x62a;&#x648;&#x646; &#x628;&#x627;&#x6cc;&#x62f; &#x641;&#x6cc;&#x644;&#x62a;&#x631;&#x647;&#x627;&#x6cc; &#x62a;&#x62d;&#x644;&#x6cc;&#x644; &#x627;&#x633;&#x62a;&#x627;&#x62a;&#x6cc;&#x6a9; REA &#x631;&#x627; &#x628;&#x627; &#x635;&#x641;&#x631; &#x627;&#x641;&#x634;&#x627; &#x631;&#x62f; &#x6a9;&#x646;&#x62f;.
            &#x646;&#x631;&#x62f;&#x628;&#x627;&#x646; &#x67e;&#x646;&#x62c;‌&#x645;&#x631;&#x62d;&#x644;&#x647;‌&#x627;&#x6cc;:
          </p>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          {REA_LADDER.map((s, i) => (
            <motion.div
              key={s.step}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: "-60px" }}
              transition={{ duration: 0.4, delay: i * 0.06 }}
              className={`rounded-2xl border p-5 ${
                s.status === "passed"
                  ? "border-gold-400/50 bg-gold-400/5"
                  : "border-zinc-800 bg-zinc-900/60"
              }`}
            >
              <div className="mb-2 flex items-center gap-2.5">
                {s.status === "passed" ? (
                  <CheckCircle2 className="size-5 text-gold-400" aria-hidden />
                ) : (
                  <Clock className="size-5 text-zinc-600" aria-hidden />
                )}
                <span className="font-mono text-xs text-zinc-500" dir="ltr">
                  STEP {s.step}/5
                </span>
                <span className={`text-sm font-bold ${s.status === "passed" ? "text-gold-400" : "text-zinc-300"}`}>
                  {s.title}
                </span>
              </div>
              <p className="text-sm leading-6 text-zinc-400">{s.detail}</p>
            </motion.div>
          ))}

          {/* Current filters */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-60px" }}
            transition={{ duration: 0.4, delay: 0.3 }}
            className="rounded-2xl border border-royal-500/30 bg-royal-600/10 p-5 md:col-span-2"
          >
            <div className="mb-3 flex items-center gap-2 text-royal-400">
              <ShieldCheck className="size-5" aria-hidden />
              <h3 className="text-sm font-bold">
                &#x648;&#x636;&#x639;&#x6cc;&#x62a; &#x641;&#x6cc;&#x644;&#x62a;&#x631;&#x647;&#x627;&#x6cc; REA &#x62f;&#x631; &#x6af;&#x627;&#x645; &#x6f1; — &#x647;&#x645;&#x647;{" "}
                <span className="font-mono text-gold-400" dir="ltr">
                  not_observed
                </span>
              </h3>
            </div>
            <div className="flex flex-wrap gap-2" aria-label="&#x641;&#x6cc;&#x644;&#x62a;&#x631;&#x647;&#x627;&#x6cc; REA &#x648; &#x648;&#x636;&#x639;&#x6cc;&#x62a; &#x622;&#x646;‌&#x647;&#x627;">
              {REA_FILTERS.map((f) => (
                <span
                  key={f.filter}
                  className="inline-flex items-center gap-2 rounded-lg border border-zinc-800 bg-zinc-900/80 px-2.5 py-1.5 font-mono text-[11px]"
                >
                  <span className="text-zinc-300" dir="ltr">
                    {f.filter}
                  </span>
                  <span
                    className={`rounded px-1.5 py-0.5 ${
                      f.result === "not_observed"
                        ? "bg-gold-400/15 text-gold-400"
                        : "bg-zinc-700/40 text-zinc-400"
                    }`}
                    dir="ltr"
                  >
                    {f.result}
                  </span>
                </span>
              ))}
            </div>
            <p className="mt-3 text-xs leading-6 text-zinc-500">
              &#x6af;&#x6cc;&#x62a; &#x627;&#x62c;&#x631;&#x627;&#x6cc;&#x6cc;: <span className="font-mono text-zinc-400" dir="ltr">make rea-gate</span> —
              &#x647;&#x631; &#x631;&#x62f;&#x6cc;&#x641; exposed &#x6cc;&#x639;&#x646;&#x6cc; &#x634;&#x6a9;&#x633;&#x62a; &#x628;&#x6cc;&#x644;&#x62f; (&#x62e;&#x631;&#x648;&#x62c; &#x6f3;).
            </p>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
