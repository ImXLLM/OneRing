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
      <h1 className="text-xl font-black">MAIRON Wallet</h1>
      <p className="max-w-md text-sm leading-6 text-zinc-400">
        This mini-app must open from inside Telegram (or from the official mini-app link that
        <code className="mx-1 rounded bg-zinc-800 px-1.5 py-0.5 text-xs" dir="ltr">tgWebAppData</code>
        carries it). Identity is never built from the request body — only from the kernel HMAC signature.
      </p>
      <button
        type="button"
        onClick={() => window.location.reload()}
        className="rounded-xl bg-royal-500/20 px-4 py-2 text-sm font-semibold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
      >
        Try again
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
        <span className="text-sm">Establishing the kernel session…</span>
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
      toast({ title: "TON address registered", variant: "success" });
      await refresh();
    } catch (err) {
      toast({
        title: "Address registration failed",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "network error",
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
        title: "Deposit proof submitted",
        description: "After admin confirmation the Din mints under Article 3.",
        variant: "success",
      });
      setTxHash("");
      setAmountTon("");
      await Promise.all([loadDeposits(token), refresh()]);
    } catch (err) {
      toast({
        title: "Deposit submission failed",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "network error",
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
            Balance derived from the double-entry ledger
          </div>
          <p className="mt-3 text-3xl font-black text-gold-400">{formatDin(me.balance)}</p>
          <p className="mt-2 text-xs text-zinc-500">
            locked in purchase escrows: <span className="font-semibold text-zinc-300">{formatDin(me.escrowLocked)}</span>
          </p>
        </div>
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
          <div className="text-sm text-zinc-400">Telegram identity</div>
          <p className="mt-2 font-bold">
            {me.user.username ? `@${me.user.username}` : `member ${me.user.telegramId}`}
          </p>
          <p className="mt-1 font-mono text-xs text-zinc-500" dir="ltr">
            {me.user.accountId}
          </p>
          <p className="mt-2 text-xs text-zinc-500">
            connected wallet:{" "}
            {me.user.tonAddress ? (
              <span className="font-mono text-emerald-400" dir="ltr">
                {me.user.tonAddress.slice(0, 10)}…{me.user.tonAddress.slice(-6)}
              </span>
            ) : (
              <span className="text-zinc-600">not connected yet</span>
            )}
          </p>
        </div>
      </section>

      {/* TON Connect */}
      <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
        <h2 className="mb-3 flex items-center gap-2 font-bold">
          <Link2 className="size-4 text-royal-400" aria-hidden />
          Connect the TON wallet
        </h2>
        <TonConnectSection onRegisterAddress={(addr) => void registerAddress(addr)} registering={registering} />
      </section>

      {/* TON deposit */}
      <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
        <h2 className="mb-1 font-bold">TON deposit — proof registration</h2>
        <p className="mb-4 text-xs leading-5 text-zinc-500">
          Enter the amount in TON (up to 6 decimal places). After admin approval,
          the Din mints at the kernel conversion rate under Article 3. The transaction hash is unique —
          a duplicate proof gets a 422.
        </p>
        <form onSubmit={(e) => void submitDeposit(e)} className="grid gap-3 sm:grid-cols-3">
          <label className="text-xs text-zinc-400 sm:col-span-1">
            Amount (TON)
            <input
              value={amountTon}
              onChange={(e) => setAmountTon(e.target.value)}
              required
              inputMode="decimal"
              aria-label="Deposit amount in TON"
              dir="ltr"
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <label className="text-xs text-zinc-400 sm:col-span-2">
            On-chain transaction hash
            <input
              value={txHash}
              onChange={(e) => setTxHash(e.target.value)}
              required
              minLength={16}
              maxLength={128}
              dir="ltr"
              aria-label="On-chain transaction hash"
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <label className="text-xs text-zinc-400 sm:col-span-3">
            Sender wallet address
            <input
              value={walletAddress}
              onChange={(e) => setWalletAddress(e.target.value)}
              required
              dir="ltr"
              aria-label="Sender wallet address"
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <button
            type="submit"
            disabled={submitting}
            className="sm:col-span-3 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2.5 text-sm font-bold text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 transition-opacity hover:opacity-90 disabled:opacity-50"
          >
            {submitting ? "submitting…" : "Submit deposit proof"}
          </button>
        </form>
      </section>

      {/* Deposit history */}
      <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
        <h2 className="mb-3 flex items-center gap-2 font-bold">
          <ReceiptText className="size-4 text-royal-400" aria-hidden />
          My deposits
        </h2>
        {deposits.length === 0 ? (
          <p className="py-4 text-center text-sm text-zinc-600">No deposits registered yet.</p>
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
        <h2 className="mb-3 font-bold">Latest ledger rows</h2>
        {me.recentTxs.length === 0 ? (
          <p className="py-4 text-center text-sm text-zinc-600">The ledger for this account is empty.</p>
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
