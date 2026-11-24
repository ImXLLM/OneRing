"use client";

import { motion } from "motion/react";
import { CheckCircle2, ShieldCheck, Clock } from "lucide-react";
import { REA_LADDER, REA_FILTERS } from "@/lib/kian-data";

/** نردبان REA: هر مایلستون یک گام به 10/10 */
export function SecurityREA() {
  return (
    <section
      id="security"
      className="scroll-mt-24 border-y border-royal-500/15 bg-zinc-950/60 py-20"
    >
      <div className="mx-auto max-w-5xl px-4 sm:px-6">
        <div className="mb-12 text-center">
          <h2 className="text-3xl font-black sm:text-4xl">
            گام‌به‌گام تا <span className="text-gold-400">10 از 10</span> REA
          </h2>
          <p className="mx-auto mt-3 max-w-2xl leading-7 text-zinc-400">
            یک‌شات نمی‌سازیم؛ هر مایلستون باید فیلترهای تحلیل استاتیک REA را با صفر افشا رد کند.
            نردبان پنج‌مرحله‌ای:
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

          {/* فیلترهای فعلی */}
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
                وضعیت فیلترهای REA در گام ۱ — همه{" "}
                <span className="font-mono text-gold-400" dir="ltr">
                  not_observed
                </span>
              </h3>
            </div>
            <div className="flex flex-wrap gap-2" aria-label="فیلترهای REA و وضعیت آن‌ها">
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
              گیت اجرایی: <span className="font-mono text-zinc-400" dir="ltr">make rea-gate</span> —
              هر ردیف exposed یعنی شکست بیلد (خروج ۳).
            </p>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
