"use client";

/**
 * KIAN miniapp — dashboard: live balance + national economy summary + shortcuts.
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
        if (err instanceof ApiError && err.status === 429) return; // rate limit — silently
        setEconomy(null);
      });
  }, []);

  if (!economy) {
    return (
      <div className="flex items-center gap-2 rounded-2xl border border-zinc-800 bg-zinc-900/60 px-4 py-3 text-xs text-zinc-600">
        <Loader2 className="size-3.5 animate-spin" aria-hidden />
        &#x62e;&#x644;&#x627;&#x635;&#x647;&#x654; &#x627;&#x642;&#x62a;&#x635;&#x627;&#x62f; &#x645;&#x644;&#x6cc; &#x62f;&#x631; &#x62d;&#x627;&#x644; &#x628;&#x627;&#x631;&#x6af;&#x6cc;&#x631;&#x6cc;…
      </div>
    );
  }

  return (
    <section className="rounded-2xl border border-royal-500/20 bg-gradient-to-b from-royal-500/10 to-transparent p-5">
      <div className="flex items-center justify-between">
        <h2 className="flex items-center gap-2 text-sm font-bold text-zinc-300">
          <Coins className="size-4 text-gold-400" aria-hidden />
          &#x627;&#x642;&#x62a;&#x635;&#x627;&#x62f; &#x645;&#x644;&#x6cc; — &#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f6;
        </h2>
        {economy.balanced ? (
          <span className="flex items-center gap-1 rounded-full bg-emerald-500/10 px-2.5 py-1 text-[11px] font-bold text-emerald-400 ring-1 ring-emerald-400/30">
            <ShieldCheck className="size-3.5" aria-hidden />
            &#x62f;&#x641;&#x62a;&#x631; &#x645;&#x62a;&#x648;&#x627;&#x632;&#x646;
          </span>
        ) : (
          <span className="rounded-full bg-red-500/10 px-2.5 py-1 text-[11px] font-bold text-red-400 ring-1 ring-red-400/30">
            &#x646;&#x627;&#x645;&#x62a;&#x648;&#x627;&#x632;&#x646;!
          </span>
        )}
      </div>
      <div className="mt-4 grid grid-cols-2 gap-3 text-sm">
        <div className="rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800">
          <div className="text-[11px] text-zinc-500">&#x627;&#x631;&#x632; &#x62f;&#x631; &#x6af;&#x631;&#x62f;&#x634;</div>
          <div className="mt-1 font-bold text-gold-400" dir="ltr">
            {economy.circulatingSupply.toLocaleString("fa-IR")} Đ
          </div>
        </div>
        <div className="rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800">
          <div className="text-[11px] text-zinc-500">&#x62e;&#x632;&#x627;&#x646;&#x647;&#x654; &#x645;&#x644;&#x6cc;</div>
          <div className="mt-1 font-bold text-zinc-200" dir="ltr">
            {economy.treasuryBalance.toLocaleString("fa-IR")} Đ
          </div>
        </div>
        <div className="rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800">
          <div className="flex items-center gap-1 text-[11px] text-zinc-500">
            <Users className="size-3" aria-hidden />
            &#x634;&#x647;&#x631;&#x648;&#x646;&#x62f;&#x627;&#x646;
          </div>
          <div className="mt-1 font-bold text-zinc-200">
            {economy.accountCount.toLocaleString("fa-IR")}
          </div>
        </div>
        <div className="rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800">
          <div className="text-[11px] text-zinc-500">&#x627;&#x633;&#x6a9;&#x631;&#x648;&#x647;&#x627;&#x6cc; &#x628;&#x627;&#x632;</div>
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
        <div className="text-sm text-zinc-400">&#x62a;&#x631;&#x627;&#x632; &#x634;&#x647;&#x631;&#x648;&#x646;&#x62f;&#x6cc;</div>
        <p className="mt-2 text-4xl font-black text-gold-400">
          {formatDin(me.balance)}
        </p>
        <p className="mt-2 text-xs text-zinc-500">
          &#x642;&#x641;&#x644;‌&#x634;&#x62f;&#x647; &#x62f;&#x631; &#x627;&#x633;&#x6a9;&#x631;&#x648;:{" "}
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
            <span className="text-sm font-bold">&#x648;&#x644;&#x62a; &#x648; &#x648;&#x627;&#x631;&#x6cc;&#x632; TON</span>
            <ArrowUpRight
              className="size-4 text-zinc-600 transition-colors group-hover:text-gold-400"
              aria-hidden
            />
          </div>
          <p className="mt-1 text-xs text-zinc-500">
            &#x62b;&#x628;&#x62a; &#x627;&#x62b;&#x628;&#x627;&#x62a; &#x648;&#x627;&#x631;&#x6cc;&#x632; &#x648; &#x627;&#x62a;&#x635;&#x627;&#x644; &#x648;&#x644;&#x62a;
          </p>
        </Link>
        <Link
          href="/market"
          className="group rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-4 transition-colors hover:bg-zinc-900"
        >
          <div className="flex items-center justify-between">
            <span className="text-sm font-bold">&#x628;&#x627;&#x632;&#x627;&#x631;</span>
            <ArrowUpRight
              className="size-4 text-zinc-600 transition-colors group-hover:text-gold-400"
              aria-hidden
            />
          </div>
          <p className="mt-1 text-xs text-zinc-500">&#x62e;&#x631;&#x6cc;&#x62f; &#x648; &#x641;&#x631;&#x648;&#x634; &#x628;&#x627; &#x627;&#x633;&#x6a9;&#x631;&#x648; &#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f8;</p>
        </Link>
      </section>

      {me.recentTxs.length > 0 && (
        <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
          <h2 className="mb-3 text-sm font-bold">&#x622;&#x62e;&#x631;&#x6cc;&#x646; &#x631;&#x62f;&#x6cc;&#x641;‌&#x647;&#x627;&#x6cc; &#x62f;&#x641;&#x62a;&#x631;</h2>
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
          <span className="text-sm">&#x628;&#x631;&#x642;&#x631;&#x627;&#x631;&#x6cc; &#x646;&#x634;&#x633;&#x62a; &#x628;&#x627; &#x6a9;&#x631;&#x646;&#x644;…</span>
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
                <span className="text-sm font-bold">&#x628;&#x627;&#x632;&#x627;&#x631; (&#x62a;&#x645;&#x627;&#x634;&#x627;)</span>
                <Store
                  className="size-4 text-zinc-600 transition-colors group-hover:text-gold-400"
                  aria-hidden
                />
              </div>
              <p className="mt-1 text-xs text-zinc-500">
                &#x645;&#x62d;&#x635;&#x648;&#x644;&#x627;&#x62a; &#x648; &#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x632;&#x646;&#x62f;&#x647; &#x631;&#x627; &#x628;&#x628;&#x6cc;&#x646;
              </p>
            </Link>
            <Link
              href="/builder"
              className="group rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-4 transition-colors hover:bg-zinc-900"
            >
              <div className="flex items-center justify-between">
                <span className="text-sm font-bold">&#x627;&#x633;&#x62a;&#x648;&#x62f;&#x6cc;&#x648; AI</span>
                <Sparkles
                  className="size-4 text-zinc-600 transition-colors group-hover:text-gold-400"
                  aria-hidden
                />
              </div>
              <p className="mt-1 text-xs text-zinc-500">
                &#x633;&#x627;&#x62e;&#x62a; &#x628;&#x627; AI — &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x641;&#x639;&#x627;&#x644; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;
              </p>
            </Link>
          </section>
        </div>
      ) : (
        <GuestGate
          title="&#x628;&#x647; &#x67e;&#x627;&#x62f;&#x634;&#x627;&#x647;&#x6cc; KIAN &#x62e;&#x648;&#x634; &#x622;&#x645;&#x62f;&#x6cc; 👑"
          hint="&#x647;&#x648;&#x6cc;&#x62a; &#x647;&#x631;&#x6af;&#x632; &#x627;&#x632; &#x628;&#x62f;&#x646;&#x647;&#x654; &#x62f;&#x631;&#x62e;&#x648;&#x627;&#x633;&#x62a; &#x633;&#x627;&#x62e;&#x62a;&#x647; &#x646;&#x645;&#x6cc;‌&#x634;&#x648;&#x62f; — &#x641;&#x642;&#x637; &#x628;&#x627; &#x627;&#x645;&#x636;&#x627;&#x6cc; HMAC &#x6a9;&#x631;&#x646;&#x644; &#x627;&#x632; &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645;. &#x6cc;&#x627; &#x628;&#x627; «&#x648;&#x631;&#x648;&#x62f; &#x622;&#x632;&#x645;&#x627;&#x6cc;&#x634;&#x6cc;» &#x647;&#x645;&#x6cc;&#x646; &#x62d;&#x627;&#x644;&#x627; &#x628;&#x627;&#x632;&#x627;&#x631; &#x648; &#x627;&#x642;&#x62a;&#x635;&#x627;&#x62f; &#x645;&#x644;&#x6cc; &#x631;&#x627; &#x62a;&#x645;&#x627;&#x634;&#x627; &#x6a9;&#x646;."
        />
      )}
    </MiniShell>
  );
}
