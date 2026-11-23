"use client";

/**
 * KIAN miniapp — داشبورد: تراز زنده + خلاصهٔ اقتصاد ملی + میان‌برها.
 */

import Link from "next/link";
import { useEffect, useState } from "react";
import {
  ArrowUpRight,
  Coins,
  Loader2,
  ShieldCheck,
  Sparkles,
  Store,
  Users,
} from "lucide-react";
import { MiniShell } from "@/components/kian/mini-shell";
import { GuestGate, PreviewBanner } from "@/components/kian/guest-gate";
import { useSession } from "@/components/kian/session";
import { api, ApiError, type EconomySummary } from "@kian/sdk";
import { formatDin } from "@/lib/format";

function EconomyCard() {
  const [economy, setEconomy] = useState<EconomySummary | null>(null);

  useEffect(() => {
    api
      .economy()
      .then(setEconomy)
      .catch((err) => {
        if (err instanceof ApiError && err.status === 429) return; // سقف نرخ — بی‌سروصدا
        setEconomy(null);
      });
  }, []);

  if (!economy) {
    return (
      <div className="flex items-center gap-2 rounded-2xl border border-zinc-800 bg-zinc-900/60 px-4 py-3 text-xs text-zinc-600">
        <Loader2 className="size-3.5 animate-spin" aria-hidden />
        خلاصهٔ اقتصاد ملی در حال بارگیری…
      </div>
    );
  }

  return (
    <section className="rounded-2xl border border-royal-500/20 bg-gradient-to-b from-royal-500/10 to-transparent p-5">
      <div className="flex items-center justify-between">
        <h2 className="flex items-center gap-2 text-sm font-bold text-zinc-300">
          <Coins className="size-4 text-gold-400" aria-hidden />
          اقتصاد ملی — مادهٔ ۶
        </h2>
        {economy.balanced ? (
          <span className="flex items-center gap-1 rounded-full bg-emerald-500/10 px-2.5 py-1 text-[11px] font-bold text-emerald-400 ring-1 ring-emerald-400/30">
            <ShieldCheck className="size-3.5" aria-hidden />
            دفتر متوازن
          </span>
        ) : (
          <span className="rounded-full bg-red-500/10 px-2.5 py-1 text-[11px] font-bold text-red-400 ring-1 ring-red-400/30">
            نامتوازن!
          </span>
        )}
      </div>
      <div className="mt-4 grid grid-cols-2 gap-3 text-sm">
        <div className="rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800">
          <div className="text-[11px] text-zinc-500">ارز در گردش</div>
          <div className="mt-1 font-bold text-gold-400" dir="ltr">
            {economy.circulatingSupply.toLocaleString("fa-IR")} Đ
          </div>
        </div>
        <div className="rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800">
          <div className="text-[11px] text-zinc-500">خزانهٔ ملی</div>
          <div className="mt-1 font-bold text-zinc-200" dir="ltr">
            {economy.treasuryBalance.toLocaleString("fa-IR")} Đ
          </div>
        </div>
        <div className="rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800">
          <div className="flex items-center gap-1 text-[11px] text-zinc-500">
            <Users className="size-3" aria-hidden />
            شهروندان
          </div>
          <div className="mt-1 font-bold text-zinc-200">
            {economy.accountCount.toLocaleString("fa-IR")}
          </div>
        </div>
        <div className="rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800">
          <div className="text-[11px] text-zinc-500">اسکروهای باز</div>
          <div className="mt-1 font-bold text-zinc-200">
            {economy.openEscrows.toLocaleString("fa-IR")}
          </div>
        </div>
      </div>
    </section>
  );
}

function Dashboard() {
  const { state } = useSession();
  if (state.phase !== "ready") return null;
  const { me } = state;

  return (
    <div className="flex flex-col gap-5">
      <section className="rounded-2xl border border-royal-500/20 bg-gradient-to-b from-royal-500/10 to-transparent p-5">
        <div className="text-sm text-zinc-400">تراز شهروندی</div>
        <p className="mt-2 text-4xl font-black text-gold-400">
          {formatDin(me.balance)}
        </p>
        <p className="mt-2 text-xs text-zinc-500">
          قفل‌شده در اسکرو:{" "}
          <span className="font-semibold text-zinc-300">
            {formatDin(me.escrowLocked)}
          </span>
        </p>
        <p className="mt-3 font-mono text-[11px] text-zinc-600" dir="ltr">
          {me.user.username ? `@${me.user.username} · ` : ""}
          {me.user.accountId}
        </p>
      </section>

      <EconomyCard />

      <section className="grid grid-cols-2 gap-3">
        <Link
          href="/wallet"
          className="group rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-4 transition-colors hover:bg-zinc-900"
        >
          <div className="flex items-center justify-between">
            <span className="text-sm font-bold">ولت و واریز TON</span>
            <ArrowUpRight
              className="size-4 text-zinc-600 transition-colors group-hover:text-gold-400"
              aria-hidden
            />
          </div>
          <p className="mt-1 text-xs text-zinc-500">
            ثبت اثبات واریز و اتصال ولت
          </p>
        </Link>
        <Link
          href="/market"
          className="group rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-4 transition-colors hover:bg-zinc-900"
        >
          <div className="flex items-center justify-between">
            <span className="text-sm font-bold">بازار</span>
            <ArrowUpRight
              className="size-4 text-zinc-600 transition-colors group-hover:text-gold-400"
              aria-hidden
            />
          </div>
          <p className="mt-1 text-xs text-zinc-500">خرید و فروش با اسکرو مادهٔ ۸</p>
        </Link>
      </section>

      {me.recentTxs.length > 0 && (
        <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
          <h2 className="mb-3 text-sm font-bold">آخرین ردیف‌های دفتر</h2>
          <ul className="flex flex-col gap-2">
            {me.recentTxs.slice(0, 5).map((tx) => (
              <li
                key={`${tx.txId}-${tx.amount}-${tx.at}`}
                className="flex items-center gap-3 rounded-lg bg-zinc-950/60 px-3 py-2 ring-1 ring-zinc-800"
              >
                <span
                  className={`font-mono text-sm font-bold ${
                    tx.amount >= 0 ? "text-emerald-400" : "text-red-400"
                  }`}
                >
                  {tx.amount >= 0 ? "+" : "−"}
                  {formatDin(Math.abs(tx.amount))}
                </span>
                <span className="truncate text-xs text-zinc-400">
                  {tx.memo ?? tx.txId}
                </span>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}

export default function HomePage() {
  const { state } = useSession();

  return (
    <MiniShell>
      {state.phase === "loading" ? (
        <div className="flex items-center justify-center gap-3 py-24 text-zinc-500">
          <Loader2 className="size-5 animate-spin" aria-hidden />
          <span className="text-sm">برقراری نشست با کرنل…</span>
        </div>
      ) : state.phase === "ready" ? (
        <Dashboard />
      ) : state.phase === "preview" ? (
        <div className="flex flex-col gap-5">
          <PreviewBanner />
          <EconomyCard />
          <section className="grid grid-cols-2 gap-3">
            <Link
              href="/market"
              className="group rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-4 transition-colors hover:bg-zinc-900"
            >
              <div className="flex items-center justify-between">
                <span className="text-sm font-bold">بازار (تماشا)</span>
                <Store
                  className="size-4 text-zinc-600 transition-colors group-hover:text-gold-400"
                  aria-hidden
                />
              </div>
              <p className="mt-1 text-xs text-zinc-500">
                محصولات و پیش‌نمایش زنده را ببین
              </p>
            </Link>
            <Link
              href="/builder"
              className="group rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-4 transition-colors hover:bg-zinc-900"
            >
              <div className="flex items-center justify-between">
                <span className="text-sm font-bold">استودیو AI</span>
                <Sparkles
                  className="size-4 text-zinc-600 transition-colors group-hover:text-gold-400"
                  aria-hidden
                />
              </div>
              <p className="mt-1 text-xs text-zinc-500">
                ساخت با AI — داخل تلگرام فعال می‌شود
              </p>
            </Link>
          </section>
        </div>
      ) : (
        <GuestGate
          title="به پادشاهی KIAN خوش آمدی 👑"
          hint="هویت هرگز از بدنهٔ درخواست ساخته نمی‌شود — فقط با امضای HMAC کرنل از داخل تلگرام. یا با «ورود آزمایشی» همین حالا بازار و اقتصاد ملی را تماشا کن."
        />
      )}
    </MiniShell>
  );
}
