"use client";

import { motion } from "motion/react";
import { MonitorSmartphone, Smartphone, Apple, Terminal } from "lucide-react";
import { Card, CardContent } from "@kian/ui";
import { InstallButton } from "./install-button";
import { FUTURE_TARGETS } from "@/lib/kian-data";

export function PwaInstall() {
  return (
    <section id="install" className="scroll-mt-24 py-20">
      <div className="mx-auto max-w-5xl px-4 sm:px-6">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-80px" }}
          transition={{ duration: 0.5 }}
        >
          <Card className="overflow-hidden border-gold-400/30 bg-gradient-to-bl from-royal-600/15 via-zinc-900/80 to-zinc-900/80">
            <CardContent className="grid gap-8 p-6 sm:p-10 md:grid-cols-2 md:items-center">
              <div>
                <h2 className="text-3xl font-black">
                  &#x647;&#x645;&#x6cc;&#x646; &#x627;&#x645;&#x631;&#x648;&#x632; <span className="text-gold-400">&#x646;&#x635;&#x628;&#x634; &#x6a9;&#x646;</span>
                </h2>
                <p className="mt-3 leading-7 text-zinc-400">
                  KIAN &#x627;&#x644;&#x627;&#x646; &#x6cc;&#x6a9; PWA &#x6a9;&#x627;&#x645;&#x644; &#x627;&#x633;&#x62a;: &#x631;&#x648;&#x6cc; &#x635;&#x641;&#x62d;&#x647; &#x627;&#x635;&#x644;&#x6cc; &#x6af;&#x648;&#x634;&#x6cc; &#x645;&#x6cc;‌&#x646;&#x634;&#x6cc;&#x646;&#x62f;&#x60c; &#x622;&#x641;&#x644;&#x627;&#x6cc;&#x646; &#x628;&#x627;&#x632; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f; &#x648; &#x645;&#x62b;&#x644;
                  &#x627;&#x67e; &#x628;&#x648;&#x645;&#x6cc; &#x627;&#x62c;&#x631;&#x627; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;. &#x67e;&#x648;&#x633;&#x62a;&#x647;‌&#x647;&#x627;&#x6cc; &#x628;&#x648;&#x645;&#x6cc; (APK&#x60c; iOS&#x60c; Electron&#x60c; macOS) &#x62f;&#x631; &#x646;&#x642;&#x634;&#x647; &#x631;&#x627;&#x647;
                  &#x631;&#x648;&#x6cc; &#x647;&#x645;&#x6cc;&#x646; &#x647;&#x633;&#x62a;&#x647; &#x633;&#x648;&#x627;&#x631; &#x645;&#x6cc;‌&#x634;&#x648;&#x646;&#x62f;.
                </p>
                <div className="mt-5 flex flex-wrap items-center gap-3">
                  <InstallButton className="h-11 px-5 text-sm" />
                  <span className="text-xs text-zinc-500">
                    &#x627;&#x6af;&#x631; &#x62f;&#x6a9;&#x645;&#x647; &#x63a;&#x6cc;&#x631;&#x641;&#x639;&#x627;&#x644; &#x628;&#x648;&#x62f;&#x60c; &#x627;&#x632; &#x631;&#x627;&#x647;&#x646;&#x645;&#x627;&#x6cc; &#x631;&#x648;&#x628;&#x647;‌&#x631;&#x648; &#x646;&#x635;&#x628; &#x6a9;&#x646;
                  </span>
                </div>
                <div className="mt-6 flex flex-wrap gap-1.5" aria-label="&#x67e;&#x644;&#x62a;&#x641;&#x631;&#x645;‌&#x647;&#x627;&#x6cc; &#x647;&#x62f;&#x641;">
                  {FUTURE_TARGETS.map((t, i) => (
                    <span
                      key={t}
                      className={`rounded-lg px-2.5 py-1 text-[11px] font-medium ${
                        i === 0
                          ? "border border-gold-400/50 bg-gold-400/10 text-gold-400"
                          : "border border-zinc-800 bg-zinc-800/60 text-zinc-500"
                      }`}
                    >
                      {t}
                    </span>
                  ))}
                </div>
              </div>

              <div className="space-y-3">
                <div className="flex items-start gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
                  <Smartphone className="mt-0.5 size-5 shrink-0 text-royal-400" aria-hidden />
                  <div>
                    <h3 className="text-sm font-bold">&#x627;&#x646;&#x62f;&#x631;&#x648;&#x6cc;&#x62f; / Chrome</h3>
                    <p className="mt-1 text-sm leading-6 text-zinc-400">
                      &#x645;&#x646;&#x648;&#x6cc; &#x633;&#x647;‌&#x646;&#x642;&#x637;&#x647; ← <span className="text-zinc-200">«&#x646;&#x635;&#x628; &#x628;&#x631;&#x646;&#x627;&#x645;&#x647;» / Add to Home screen</span>
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
                  <Apple className="mt-0.5 size-5 shrink-0 text-royal-400" aria-hidden />
                  <div>
                    <h3 className="text-sm font-bold">iOS / Safari</h3>
                    <p className="mt-1 text-sm leading-6 text-zinc-400">
                      &#x62f;&#x6a9;&#x645;&#x647; &#x627;&#x634;&#x62a;&#x631;&#x627;&#x6a9; <span className="text-zinc-200" dir="ltr">Share</span> ←{" "}
                      <span className="text-zinc-200" dir="ltr">Add to Home Screen</span>
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
                  <Terminal className="mt-0.5 size-5 shrink-0 text-royal-400" aria-hidden />
                  <div>
                    <h3 className="text-sm font-bold">&#x62f;&#x633;&#x6a9;&#x62a;&#x627;&#x67e;</h3>
                    <p className="mt-1 text-sm leading-6 text-zinc-400">
                      &#x622;&#x6cc;&#x6a9;&#x648;&#x646; &#x646;&#x635;&#x628; &#x62f;&#x631; &#x646;&#x648;&#x627;&#x631; &#x622;&#x62f;&#x631;&#x633; &#x645;&#x631;&#x648;&#x631;&#x6af;&#x631; &#x6cc;&#x627; &#x62f;&#x6a9;&#x645;&#x647;{" "}
                      <span className="text-gold-400" dir="ltr">Install</span>
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
                  <MonitorSmartphone className="mt-0.5 size-5 shrink-0 text-gold-400" aria-hidden />
                  <div>
                    <h3 className="text-sm font-bold">&#x622;&#x6cc;&#x646;&#x62f;&#x647; — &#x67e;&#x648;&#x633;&#x62a;&#x647; &#x628;&#x648;&#x645;&#x6cc;</h3>
                    <p className="mt-1 text-sm leading-6 text-zinc-400">
                      APK &#x648; iOS &#x628;&#x627; &#x67e;&#x648;&#x634;&#x634; Capacitor/Tauri&#x60c; &#x62f;&#x633;&#x6a9;&#x62a;&#x627;&#x67e; &#x628;&#x627; Electron + &#x641;&#x627;&#x632;&#x647;&#x627;&#x6cc; &#x642;&#x641;&#x644;‌&#x634;&#x62f;&#x647; —
                      &#x647;&#x645;&#x647; &#x627;&#x632; &#x647;&#x645;&#x6cc;&#x646; &#x6a9;&#x62f; PWA.
                    </p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}
