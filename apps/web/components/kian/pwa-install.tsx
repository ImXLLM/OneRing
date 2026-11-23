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
                  همین امروز <span className="text-gold-400">نصبش کن</span>
                </h2>
                <p className="mt-3 leading-7 text-zinc-400">
                  KIAN الان یک PWA کامل است: روی صفحه اصلی گوشی می‌نشیند، آفلاین باز می‌شود و مثل
                  اپ بومی اجرا می‌شود. پوسته‌های بومی (APK، iOS، Electron، macOS) در نقشه راه
                  روی همین هسته سوار می‌شوند.
                </p>
                <div className="mt-5 flex flex-wrap items-center gap-3">
                  <InstallButton className="h-11 px-5 text-sm" />
                  <span className="text-xs text-zinc-500">
                    اگر دکمه غیرفعال بود، از راهنمای روبه‌رو نصب کن
                  </span>
                </div>
                <div className="mt-6 flex flex-wrap gap-1.5" aria-label="پلتفرم‌های هدف">
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
                    <h3 className="text-sm font-bold">اندروید / Chrome</h3>
                    <p className="mt-1 text-sm leading-6 text-zinc-400">
                      منوی سه‌نقطه ← <span className="text-zinc-200">«نصب برنامه» / Add to Home screen</span>
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
                  <Apple className="mt-0.5 size-5 shrink-0 text-royal-400" aria-hidden />
                  <div>
                    <h3 className="text-sm font-bold">iOS / Safari</h3>
                    <p className="mt-1 text-sm leading-6 text-zinc-400">
                      دکمه اشتراک <span className="text-zinc-200" dir="ltr">Share</span> ←{" "}
                      <span className="text-zinc-200" dir="ltr">Add to Home Screen</span>
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
                  <Terminal className="mt-0.5 size-5 shrink-0 text-royal-400" aria-hidden />
                  <div>
                    <h3 className="text-sm font-bold">دسکتاپ</h3>
                    <p className="mt-1 text-sm leading-6 text-zinc-400">
                      آیکون نصب در نوار آدرس مرورگر یا دکمه{" "}
                      <span className="text-gold-400" dir="ltr">Install</span>
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
                  <MonitorSmartphone className="mt-0.5 size-5 shrink-0 text-gold-400" aria-hidden />
                  <div>
                    <h3 className="text-sm font-bold">آینده — پوسته بومی</h3>
                    <p className="mt-1 text-sm leading-6 text-zinc-400">
                      APK و iOS با پوشش Capacitor/Tauri، دسکتاپ با Electron + فازهای قفل‌شده —
                      همه از همین کد PWA.
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
