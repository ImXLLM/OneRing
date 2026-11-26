"use client";

/**
 * KIAN M3 — the mini-app wallet page.
 *
 * Balance derived from the double-entry ledger + escrow locks + the latest ledger rows;
 * TON Connect to link a wallet and record the address on identity; TON deposits with user
 * proof (PENDING) until admin approval and the article 3 mint in the kernel.
 */

import dynamic from "next/dynamic";
import { useCallback, useEffect, useState } from "react";
import { Badge } from "@kian/ui";
import { toast } from "@kian/ui";
import { Coins, Landmark, Link2, Loader2, ReceiptText } from "lucide-react";
import { useSession } from "@/components/kian/session";
import { ApiError, api, type TonDepositView } from "@/lib/api";
import { DEPOSIT_STATUS, formatDin, formatWhen } from "@/lib/format";

// TON Connect on the client only — the library is browser-dependent at module level
const TonConnectSection = dynamic(
  () => import("@/components/kian/ton-connect"),
  { ssr: false, loading: () => <div className="h-16 animate-pulse rounded-xl bg-zinc-900/60" /> },
);

function SessionGate() {
  return (
    <div className="flex flex-col items-center gap-4 rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-10 text-center">
      <Landmark className="size-10 text-gold-400" aria-hidden />
      <h1 className="text-xl font-black">&#x648;&#x644;&#x62a; KIAN</h1>
      <p className="max-w-md text-sm leading-6 text-zinc-400">
        &#x627;&#x6cc;&#x646; &#x645;&#x6cc;&#x646;&#x6cc;‌&#x627;&#x67e; &#x628;&#x627;&#x6cc;&#x62f; &#x627;&#x632; &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x628;&#x627;&#x632; &#x634;&#x648;&#x62f; (&#x6cc;&#x627; &#x628;&#x627; &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x633;&#x645;&#x6cc; &#x645;&#x6cc;&#x646;&#x6cc;‌&#x627;&#x67e; &#x6a9;&#x647;
        <code className="mx-1 rounded bg-zinc-800 px-1.5 py-0.5 text-xs" dir="ltr">tgWebAppData</code>
        &#x62f;&#x627;&#x631;&#x62f;). &#x647;&#x648;&#x6cc;&#x62a; &#x647;&#x631;&#x6af;&#x632; &#x627;&#x632; &#x628;&#x62f;&#x646;&#x647;&#x654; &#x62f;&#x631;&#x62e;&#x648;&#x627;&#x633;&#x62a; &#x633;&#x627;&#x62e;&#x62a;&#x647; &#x646;&#x645;&#x6cc;‌&#x634;&#x648;&#x62f; — &#x641;&#x642;&#x637; &#x628;&#x627; &#x627;&#x645;&#x636;&#x627;&#x6cc; HMAC &#x6a9;&#x631;&#x646;&#x644;.
      </p>
      <button
        type="button"
        onClick={() => window.location.reload()}
        className="rounded-xl bg-royal-500/20 px-4 py-2 text-sm font-semibold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
      >
        &#x62a;&#x644;&#x627;&#x634; &#x62f;&#x648;&#x628;&#x627;&#x631;&#x647;
      </button>
    </div>
  );
}

export default function WalletPage() {
  const { state, refresh } = useSession();
  const [deposits, setDeposits] = useState<TonDepositView[]>([]);
  const [registering, setRegistering] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [txHash, setTxHash] = useState("");
  const [walletAddress, setWalletAddress] = useState("");
  const [amountTon, setAmountTon] = useState("");

  const loadDeposits = useCallback(async (token: string) => {
    try {
      const res = await api.myDeposits(token);
      setDeposits(res.items);
    } catch {
      /* Non-critical list — silently */
    }
  }, []);

  useEffect(() => {
    if (state.phase === "ready") void loadDeposits(state.token);
  }, [state, loadDeposits]);

  if (state.phase === "loading") {
    return (
      <div className="flex items-center justify-center gap-3 py-24 text-zinc-500">
        <Loader2 className="size-5 animate-spin" aria-hidden />
        <span className="text-sm">&#x628;&#x631;&#x642;&#x631;&#x627;&#x631;&#x6cc; &#x646;&#x634;&#x633;&#x62a; &#x628;&#x627; &#x6a9;&#x631;&#x646;&#x644;…</span>
      </div>
    );
  }

  if (state.phase !== "ready") {
    return <SessionGate />;
  }

  const { me, token } = state;

  const registerAddress = async (address: string) => {
    setRegistering(true);
    try {
      await api.setTonAddress(token, address);
      toast({ title: "\u0622\u062f\u0631\u0633 TON \u062b\u0628\u062a \u0634\u062f", variant: "success" });
      await refresh();
    } catch (err) {
      toast({
        title: "\u062b\u0628\u062a \u0622\u062f\u0631\u0633 \u0646\u0627\u0645\u0648\u0641\u0642",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "\u062e\u0637\u0627\u06cc \u0634\u0628\u06a9\u0647",
        variant: "destructive",
      });
    } finally {
      setRegistering(false);
    }
  };

  const submitDeposit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createDeposit(token, {
        txHash: txHash.trim(),
        walletAddress: walletAddress.trim(),
        amountTon: amountTon.trim(),
      });
      toast({
        title: "\u0627\u062b\u0628\u0627\u062a \u0648\u0627\u0631\u06cc\u0632 \u062b\u0628\u062a \u0634\u062f",
        description: "\u067e\u0633 \u0627\u0632 \u062a\u0623\u06cc\u06cc\u062f \u0627\u062f\u0645\u06cc\u0646\u060c \u062f\u06cc\u06cc\u0646 \u0645\u0637\u0627\u0628\u0642 \u0645\u0627\u062f\u0647\u0654 \u06f3 \u0636\u0631\u0628 \u0645\u06cc‌\u0634\u0648\u062f.",
        variant: "success",
      });
      setTxHash("");
      setAmountTon("");
      await Promise.all([loadDeposits(token), refresh()]);
    } catch (err) {
      toast({
        title: "\u062b\u0628\u062a \u0648\u0627\u0631\u06cc\u0632 \u0646\u0627\u0645\u0648\u0641\u0642",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "\u062e\u0637\u0627\u06cc \u0634\u0628\u06a9\u0647",
        variant: "destructive",
      });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col gap-6">
      {/* Identity and balance */}
      <section className="grid gap-4 sm:grid-cols-2">
        <div className="rounded-2xl border border-royal-500/20 bg-gradient-to-b from-royal-500/10 to-transparent p-5">
          <div className="flex items-center gap-2 text-sm text-zinc-400">
            <Coins className="size-4 text-gold-400" aria-hidden />
            &#x62a;&#x631;&#x627;&#x632; &#x645;&#x634;&#x62a;&#x642;‌&#x634;&#x62f;&#x647; &#x627;&#x632; &#x62f;&#x641;&#x62a;&#x631; &#x62f;&#x648;&#x637;&#x631;&#x641;&#x647;
          </div>
          <p className="mt-3 text-3xl font-black text-gold-400">{formatDin(me.balance)}</p>
          <p className="mt-2 text-xs text-zinc-500">
            &#x642;&#x641;&#x644;‌&#x634;&#x62f;&#x647; &#x62f;&#x631; &#x627;&#x633;&#x6a9;&#x631;&#x648; &#x62e;&#x631;&#x6cc;&#x62f;&#x647;&#x627;: <span className="font-semibold text-zinc-300">{formatDin(me.escrowLocked)}</span>
          </p>
        </div>
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
          <div className="text-sm text-zinc-400">&#x647;&#x648;&#x6cc;&#x62a; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645;</div>
          <p className="mt-2 font-bold">
            {me.user.username ? `@${me.user.username}` : `\u06a9\u0627\u0631\u0628\u0631 ${me.user.telegramId}`}
          </p>
          <p className="mt-1 font-mono text-xs text-zinc-500" dir="ltr">
            {me.user.accountId}
          </p>
          <p className="mt-2 text-xs text-zinc-500">
            &#x648;&#x644;&#x62a; &#x645;&#x62a;&#x635;&#x644;:{" "}
            {me.user.tonAddress ? (
              <span className="font-mono text-emerald-400" dir="ltr">
                {me.user.tonAddress.slice(0, 10)}…{me.user.tonAddress.slice(-6)}
              </span>
            ) : (
              <span className="text-zinc-600">&#x647;&#x646;&#x648;&#x632; &#x648;&#x635;&#x644; &#x646;&#x634;&#x62f;&#x647;</span>
            )}
          </p>
        </div>
      </section>

      {/* TON Connect */}
      <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
        <h2 className="mb-3 flex items-center gap-2 font-bold">
          <Link2 className="size-4 text-royal-400" aria-hidden />
          &#x627;&#x62a;&#x635;&#x627;&#x644; &#x648;&#x644;&#x62a; TON
        </h2>
        <TonConnectSection onRegisterAddress={(addr) => void registerAddress(addr)} registering={registering} />
      </section>

      {/* TON deposit */}
      <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
        <h2 className="mb-1 font-bold">&#x648;&#x627;&#x631;&#x6cc;&#x632; TON — &#x62b;&#x628;&#x62a; &#x627;&#x62b;&#x628;&#x627;&#x62a;</h2>
        <p className="mb-4 text-xs leading-5 text-zinc-500">
          &#x645;&#x628;&#x644;&#x63a; &#x631;&#x627; &#x628;&#x647; TON &#x648;&#x627;&#x631;&#x62f; &#x6a9;&#x646;&#x6cc;&#x62f; (&#x62d;&#x62f;&#x627;&#x6a9;&#x62b;&#x631; &#x6f6; &#x631;&#x642;&#x645; &#x627;&#x639;&#x634;&#x627;&#x631;). &#x67e;&#x633; &#x627;&#x632; &#x62a;&#x623;&#x6cc;&#x6cc;&#x62f; &#x627;&#x62f;&#x645;&#x6cc;&#x646;&#x60c;
          &#x62f;&#x6cc;&#x6cc;&#x646; &#x628;&#x627; &#x646;&#x631;&#x62e; &#x62a;&#x628;&#x62f;&#x6cc;&#x644; &#x6a9;&#x631;&#x646;&#x644; &#x645;&#x637;&#x627;&#x628;&#x642; &#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f3; &#x636;&#x631;&#x628; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;. &#x647;&#x634; &#x62a;&#x631;&#x627;&#x6a9;&#x646;&#x634; &#x6cc;&#x6a9;&#x62a;&#x627;&#x633;&#x62a; —
          &#x627;&#x62b;&#x628;&#x627;&#x62a; &#x62a;&#x6a9;&#x631;&#x627;&#x631;&#x6cc; &#x6f4;&#x6f2;&#x6f2; &#x645;&#x6cc;‌&#x6af;&#x6cc;&#x631;&#x62f;.
        </p>
        <form onSubmit={(e) => void submitDeposit(e)} className="grid gap-3 sm:grid-cols-3">
          <label className="text-xs text-zinc-400 sm:col-span-1">
            &#x645;&#x628;&#x644;&#x63a; (TON)
            <input
              value={amountTon}
              onChange={(e) => setAmountTon(e.target.value)}
              required
              inputMode="decimal"
              placeholder="0.5"
              dir="ltr"
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <label className="text-xs text-zinc-400 sm:col-span-2">
            &#x647;&#x634; &#x62a;&#x631;&#x627;&#x6a9;&#x646;&#x634; &#x632;&#x646;&#x62c;&#x6cc;&#x631;&#x647;
            <input
              value={txHash}
              onChange={(e) => setTxHash(e.target.value)}
              required
              minLength={16}
              maxLength={128}
              dir="ltr"
              placeholder="0x…"
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <label className="text-xs text-zinc-400 sm:col-span-3">
            &#x622;&#x62f;&#x631;&#x633; &#x648;&#x644;&#x62a; &#x641;&#x631;&#x633;&#x62a;&#x646;&#x62f;&#x647;
            <input
              value={walletAddress}
              onChange={(e) => setWalletAddress(e.target.value)}
              required
              dir="ltr"
              placeholder="UQ…"
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <button
            type="submit"
            disabled={submitting}
            className="sm:col-span-3 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2.5 text-sm font-bold text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 transition-opacity hover:opacity-90 disabled:opacity-50"
          >
            {submitting ? "\u062f\u0631 \u062d\u0627\u0644 \u062b\u0628\u062a…" : "\u062b\u0628\u062a \u0627\u062b\u0628\u0627\u062a \u0648\u0627\u0631\u06cc\u0632"}
          </button>
        </form>
      </section>

      {/* Deposit history */}
      <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
        <h2 className="mb-3 flex items-center gap-2 font-bold">
          <ReceiptText className="size-4 text-royal-400" aria-hidden />
          &#x648;&#x627;&#x631;&#x6cc;&#x632;&#x647;&#x627;&#x6cc; &#x645;&#x646;
        </h2>
        {deposits.length === 0 ? (
          <p className="py-4 text-center text-sm text-zinc-600">&#x647;&#x646;&#x648;&#x632; &#x648;&#x627;&#x631;&#x6cc;&#x632;&#x6cc; &#x62b;&#x628;&#x62a; &#x646;&#x634;&#x62f;&#x647; &#x627;&#x633;&#x62a;.</p>
        ) : (
          <ul className="flex flex-col gap-3">
            {deposits.map((d) => {
              const st = DEPOSIT_STATUS[d.status] ?? {
                fa: d.status,
                className: "bg-zinc-500/15 text-zinc-400 ring-zinc-400/30",
              };
              return (
                <li
                  key={d.id}
                  className="flex flex-wrap items-center gap-x-3 gap-y-1 rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800"
                >
                  <span className="font-mono text-sm font-semibold text-gold-400" dir="ltr">
                    {d.amountTon} TON
                  </span>
                  <Badge variant="default" className={`ring-1 ${st.className}`}>
                    {st.fa}
                  </Badge>
                  <span className="font-mono text-xs text-zinc-500" dir="ltr">
                    {d.txHash.slice(0, 14)}…{d.txHash.slice(-8)}
                  </span>
                  <span className="mr-auto text-xs text-zinc-600">{formatWhen(d.createdAt)}</span>
                </li>
              );
            })}
          </ul>
        )}
      </section>

      {/* Latest ledger rows */}
      <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
        <h2 className="mb-3 font-bold">&#x622;&#x62e;&#x631;&#x6cc;&#x646; &#x631;&#x62f;&#x6cc;&#x641;‌&#x647;&#x627;&#x6cc; &#x62f;&#x641;&#x62a;&#x631;</h2>
        {me.recentTxs.length === 0 ? (
          <p className="py-4 text-center text-sm text-zinc-600">&#x62f;&#x641;&#x62a;&#x631; &#x628;&#x631;&#x627;&#x6cc; &#x627;&#x6cc;&#x646; &#x62d;&#x633;&#x627;&#x628; &#x62e;&#x627;&#x644;&#x6cc; &#x627;&#x633;&#x62a;.</p>
        ) : (
          <ul className="flex flex-col gap-2">
            {me.recentTxs.map((tx) => (
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
                <span className="truncate text-xs text-zinc-400">{tx.memo ?? tx.txId}</span>
                <span className="mr-auto text-xs text-zinc-600">{formatWhen(tx.at)}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
