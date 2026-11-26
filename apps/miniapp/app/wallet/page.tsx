"use client";

/**
 * KIAN miniapp — wallet: double-entry ledger balance, TON connect, deposit proof registration, history.
 */

import dynamic from "next/dynamic";
import { useCallback, useEffect, useState } from "react";
import { Badge, toast } from "@kian/ui";
import { ArrowUpRight, Coins, Link2, Loader2, ReceiptText } from "lucide-react";
import { MiniShell } from "@/components/kian/mini-shell";
import { GuestGate, PreviewBanner } from "@/components/kian/guest-gate";
import { useSession } from "@/components/kian/session";
import { friendlyError } from "@/lib/friendly";
import {
  api,
  type TonDepositView,
  type WithdrawalAsset,
  type WithdrawalChain,
  type WithdrawalRequestView,
} from "@kian/sdk";
import { DEPOSIT_STATUS, formatDin, formatWhen } from "@/lib/format";

const WITHDRAWAL_STATUS: Record<
  WithdrawalRequestView["status"],
  { fa: string; className: string }
> = {
  PENDING: {
    fa: "\u0642\u0641\u0644‌\u0634\u062f\u0647 · \u062f\u0631 \u0635\u0641 \u067e\u0631\u062f\u0627\u062e\u062a",
    className: "bg-gold-400/15 text-gold-300 ring-gold-400/30",
  },
  CONFIRMED: {
    fa: "\u0648\u0627\u0631\u06cc\u0632 \u0634\u062f",
    className: "bg-emerald-500/15 text-emerald-300 ring-emerald-400/30",
  },
  REJECTED: {
    fa: "\u0631\u062f \u0634\u062f · \u0648\u062c\u0647 \u0628\u0631\u06af\u0634\u062a \u062e\u0648\u0631\u062f",
    className: "bg-rose-500/15 text-rose-300 ring-rose-400/30",
  },
};

const WITHDRAWAL_CHAINS: { chain: WithdrawalChain; label: string; placeholder: string }[] = [
  { chain: "TON", label: "TON (TON / USDT)", placeholder: "UQ… \u06cc\u0627 EQ…" },
  { chain: "TRX", label: "TRON · TRC-20 (USDT)", placeholder: "T…" },
  { chain: "SOL", label: "Solana · SPL (USDT)", placeholder: "\u0622\u062f\u0631\u0633 Base58 \u0633\u0648\u0644\u0627\u0646\u0627" },
  { chain: "BSC", label: "BNB Chain · BEP-20 (USDT)", placeholder: "0x…" },
];

// TON Connect on the client only — the library is browser-dependent at module level
const TonConnectSection = dynamic(
  () => import("@/components/kian/ton-connect"),
  {
    ssr: false,
    loading: () => <div className="h-16 animate-pulse rounded-xl bg-zinc-900/60" />,
  },
);

export default function WalletPage() {
  const { state, refresh, activePersona } = useSession();
  const [deposits, setDeposits] = useState<TonDepositView[]>([]);
  const [withdrawals, setWithdrawals] = useState<WithdrawalRequestView[]>([]);
  const [registering, setRegistering] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [withdrawing, setWithdrawing] = useState(false);
  // trial-credit faucet — shown only for demo personas
  const [claimingFaucet, setClaimingFaucet] = useState(false);
  const [txHash, setTxHash] = useState("");
  const [walletAddress, setWalletAddress] = useState("");
  const [amountTon, setAmountTon] = useState("");
  const [wChain, setWChain] = useState<WithdrawalChain>("TON");
  const [wAsset, setWAsset] = useState<WithdrawalAsset>("USDT");
  const [wAddress, setWAddress] = useState("");
  const [wAmount, setWAmount] = useState("");

  const loadHistory = useCallback(async (token: string) => {
    try {
      const [depRes, wdrRes] = await Promise.all([
        api.myDeposits(token),
        api.myWithdrawals(token),
      ]);
      setDeposits(depRes.items);
      setWithdrawals(wdrRes.items);
    } catch {
      /* Non-critical list — silently */
    }
  }, []);

  useEffect(() => {
    if (state.phase === "ready") void loadHistory(state.token);
  }, [state, loadHistory]);

  if (state.phase === "loading") {
    return (
      <MiniShell>
        <div className="flex items-center justify-center gap-3 py-24 text-zinc-500">
          <Loader2 className="size-5 animate-spin" aria-hidden />
          <span className="text-sm">&#x628;&#x631;&#x642;&#x631;&#x627;&#x631;&#x6cc; &#x646;&#x634;&#x633;&#x62a; &#x628;&#x627; &#x6a9;&#x631;&#x646;&#x644;…</span>
        </div>
      </MiniShell>
    );
  }

  if (state.phase !== "ready") {
    return (
      <MiniShell>
        <div className="flex flex-col gap-4">
          {state.phase === "preview" && <PreviewBanner />}
          <GuestGate
            title="&#x648;&#x644;&#x62a; &#x627;&#x645;&#x646; KIAN &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x628;&#x627;&#x632; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;"
            hint="&#x627;&#x62a;&#x635;&#x627;&#x644; &#x648;&#x644;&#x62a; TON&#x60c; &#x62b;&#x628;&#x62a; &#x648;&#x627;&#x631;&#x6cc;&#x632; &#x648; &#x628;&#x631;&#x62f;&#x627;&#x634;&#x62a; &#x6a9;&#x631;&#x6cc;&#x67e;&#x62a;&#x648; &#x647;&#x645;&#x647; &#x627;&#x632; &#x646;&#x634;&#x633;&#x62a; &#x627;&#x62d;&#x631;&#x627;&#x632;&#x634;&#x62f;&#x647;&#x654; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x634;&#x645;&#x627; &#x645;&#x6cc;‌&#x6af;&#x630;&#x631;&#x62f; &#x62a;&#x627; &#x62f;&#x627;&#x631;&#x627;&#x6cc;&#x6cc;‌&#x62a;&#x627;&#x646; &#x647;&#x645;&#x6cc;&#x634;&#x647; &#x627;&#x645;&#x646; &#x628;&#x645;&#x627;&#x646;&#x62f;."
            whatWorksInPreview={state.phase !== "preview"}
          />
        </div>
      </MiniShell>
    );
  }

  const { me, token } = state;

  // trial-credit faucet — 5,000 free KIAN; only in the server's dev mode
  const claimFaucet = async () => {
    setClaimingFaucet(true);
    try {
      const res = await api.claimDemoFaucet(token);
      toast({
        title: "🎁 \u0627\u0639\u062a\u0628\u0627\u0631 \u062a\u0633\u062a \u0627\u0636\u0627\u0641\u0647 \u0634\u062f",
        description: `${formatDin(res.balance)} \u062f\u0631 \u06a9\u06cc\u0641‌\u067e\u0648\u0644 \u062a\u0648 \u0622\u0645\u0627\u062f\u0647\u0654 \u062e\u0631\u06cc\u062f \u0648 \u062a\u0633\u062a \u0627\u0633\u062a.`,
        variant: "success",
      });
      await refresh();
    } catch (err) {
      toast({
        title: "\u062f\u0631\u06cc\u0627\u0641\u062a \u0627\u0639\u062a\u0628\u0627\u0631 \u062a\u0633\u062a \u0646\u0627\u0645\u0648\u0641\u0642",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setClaimingFaucet(false);
    }
  };

  const registerAddress = async (address: string) => {
    setRegistering(true);
    try {
      await api.setTonAddress(token, address);
      toast({ title: "\u0622\u062f\u0631\u0633 TON \u062b\u0628\u062a \u0634\u062f", variant: "success" });
      await refresh();
    } catch (err) {
      toast({
        title: "\u062b\u0628\u062a \u0622\u062f\u0631\u0633 \u0646\u0627\u0645\u0648\u0641\u0642",
        description: friendlyError(err),
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
      await Promise.all([loadHistory(token), refresh()]);
    } catch (err) {
      toast({
        title: "\u062b\u0628\u062a \u0648\u0627\u0631\u06cc\u0632 \u0646\u0627\u0645\u0648\u0641\u0642",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setSubmitting(false);
    }
  };

  const selectChain = (next: WithdrawalChain) => {
    setWChain(next);
    if (next !== "TON" && wAsset === "TON") {
      setWAsset("USDT");
    }
  };

  const submitWithdrawal = async (e: React.FormEvent) => {
    e.preventDefault();
    const amountDin = Number.parseInt(wAmount.trim(), 10);
    if (!Number.isFinite(amountDin) || amountDin < 1) {
      toast({
        title: "\u0645\u0628\u0644\u063a \u0646\u0627\u0645\u0639\u062a\u0628\u0631",
        description: "\u0645\u0628\u0644\u063a \u0628\u0631\u062f\u0627\u0634\u062a \u0628\u0627\u06cc\u062f \u0639\u062f\u062f \u0635\u062d\u06cc\u062d \u0628\u0632\u0631\u06af‌\u062a\u0631 \u0627\u0632 \u0635\u0641\u0631 \u0628\u0627\u0634\u062f.",
        variant: "destructive",
      });
      return;
    }
    setWithdrawing(true);
    try {
      const created = await api.createWithdrawal(token, {
        chain: wChain,
        asset: wChain === "TON" ? wAsset : "USDT",
        destinationAddress: wAddress.trim(),
        amountDin,
      });
      setWithdrawals((prev) => [created, ...prev]);
      setWAddress("");
      setWAmount("");
      toast({
        title: "\u062f\u0631\u062e\u0648\u0627\u0633\u062a \u0628\u0631\u062f\u0627\u0634\u062a \u062b\u0628\u062a \u0648 \u0642\u0641\u0644 \u0634\u062f",
        description: `${formatDin(created.amountDin)} \u0628\u0631\u0627\u06cc \u067e\u0631\u062f\u0627\u062e\u062a \u0631\u0648\u06cc ${created.chain} (${created.asset}) \u062f\u0631 \u0635\u0641 \u0642\u0631\u0627\u0631 \u06af\u0631\u0641\u062a.`,
        variant: "success",
      });
      await refresh();
    } catch (err) {
      toast({
        title: "\u062b\u0628\u062a \u0628\u0631\u062f\u0627\u0634\u062a \u0646\u0627\u0645\u0648\u0641\u0642",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setWithdrawing(false);
    }
  };

  return (
    <MiniShell>
      <div className="flex flex-col gap-5">
        {/* Identity and balance */}
        <section className="rounded-2xl border border-royal-500/20 bg-gradient-to-b from-royal-500/10 to-transparent p-5">
          <div className="flex items-center gap-2 text-sm text-zinc-400">
            <Coins className="size-4 text-gold-400" aria-hidden />
            &#x62a;&#x631;&#x627;&#x632; &#x645;&#x634;&#x62a;&#x642;‌&#x634;&#x62f;&#x647; &#x627;&#x632; &#x62f;&#x641;&#x62a;&#x631; &#x62f;&#x648;&#x637;&#x631;&#x641;&#x647;
          </div>
          <p className="mt-3 text-3xl font-black text-gold-400">{formatDin(me.balance)}</p>
          <p className="mt-2 text-xs text-zinc-500">
            &#x642;&#x641;&#x644;‌&#x634;&#x62f;&#x647; &#x62f;&#x631; &#x627;&#x633;&#x6a9;&#x631;&#x648; &#x62e;&#x631;&#x6cc;&#x62f;&#x647;&#x627;:{" "}
            <span className="font-semibold text-zinc-300">{formatDin(me.escrowLocked)}</span>
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

          {/* Trial-credit faucet — demo personas only (1001/2002/3003) */}
          {activePersona && (
            <div className="mt-4 flex flex-col gap-2 rounded-xl border border-emerald-400/30 bg-emerald-500/10 p-3">
              <p className="text-xs font-bold leading-5 text-emerald-300">
                &#x62d;&#x627;&#x644;&#x62a; &#x62a;&#x633;&#x62a; &#x641;&#x639;&#x627;&#x644; &#x627;&#x633;&#x62a; — &#x628;&#x631;&#x627;&#x6cc; &#x627;&#x645;&#x62a;&#x62d;&#x627;&#x646;&#x650; &#x6a9;&#x627;&#x645;&#x644;&#x650; &#x62e;&#x631;&#x6cc;&#x62f; &#x648; &#x628;&#x631;&#x62f;&#x627;&#x634;&#x62a;&#x60c; &#x627;&#x639;&#x62a;&#x628;&#x627;&#x631; &#x631;&#x627;&#x6cc;&#x6af;&#x627;&#x646; &#x628;&#x6af;&#x6cc;&#x631;:
              </p>
              <button
                type="button"
                disabled={claimingFaucet}
                onClick={() => void claimFaucet()}
                className="flex items-center justify-center gap-2 rounded-xl bg-gradient-to-b from-emerald-500 to-emerald-600 px-4 py-3 text-sm font-black text-white shadow-lg shadow-emerald-600/25 ring-1 ring-emerald-400/50 transition hover:opacity-90 disabled:opacity-50"
              >
                {claimingFaucet
                  ? "\u062f\u0631 \u062d\u0627\u0644 \u062f\u0631\u06cc\u0627\u0641\u062a…"
                  : "🎁 \u062f\u0631\u06cc\u0627\u0641\u062a \u06f5\u066c\u06f0\u06f0\u06f0 KIAN \u0627\u0639\u062a\u0628\u0627\u0631 \u062a\u0633\u062a \u0631\u0627\u06cc\u06af\u0627\u0646 (Faucet)"}
              </button>
            </div>
          )}
        </section>

        {/* TON Connect */}
        <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
          <h2 className="mb-3 flex items-center gap-2 font-bold">
            <Link2 className="size-4 text-royal-400" aria-hidden />
            &#x627;&#x62a;&#x635;&#x627;&#x644; &#x648;&#x644;&#x62a; TON
          </h2>
          <TonConnectSection
            onRegisterAddress={(addr) => void registerAddress(addr)}
            registering={registering}
          />
        </section>

        {/* TON deposit */}
        <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
          <h2 className="mb-1 font-bold">&#x648;&#x627;&#x631;&#x6cc;&#x632; TON — &#x62b;&#x628;&#x62a; &#x627;&#x62b;&#x628;&#x627;&#x62a;</h2>
          <p className="mb-4 text-xs leading-5 text-zinc-500">
            &#x645;&#x628;&#x644;&#x63a; &#x631;&#x627; &#x628;&#x647; TON &#x648;&#x627;&#x631;&#x62f; &#x6a9;&#x646;&#x6cc;&#x62f; (&#x62d;&#x62f;&#x627;&#x6a9;&#x62b;&#x631; &#x6f6; &#x631;&#x642;&#x645; &#x627;&#x639;&#x634;&#x627;&#x631;). &#x67e;&#x633; &#x627;&#x632; &#x62a;&#x623;&#x6cc;&#x6cc;&#x62f; &#x627;&#x62f;&#x645;&#x6cc;&#x646;&#x60c;
            &#x62f;&#x6cc;&#x6cc;&#x646; &#x628;&#x627; &#x646;&#x631;&#x62e; &#x62a;&#x628;&#x62f;&#x6cc;&#x644; &#x6a9;&#x631;&#x646;&#x644; &#x645;&#x637;&#x627;&#x628;&#x642; &#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f3; &#x636;&#x631;&#x628; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
          </p>
          <form onSubmit={(e) => void submitDeposit(e)} className="grid gap-3">
            <label className="text-xs text-zinc-400">
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
            <label className="text-xs text-zinc-400">
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
            <label className="text-xs text-zinc-400">
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
              className="rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2.5 text-sm font-bold text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 transition-opacity hover:opacity-90 disabled:opacity-50"
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

        {/* Phase 6 — earnings withdrawal request (TON / USDT) */}
        <section className="rounded-2xl border border-gold-400/25 bg-zinc-900/60 p-5">
          <div className="mb-1 flex items-center justify-between gap-2">
            <h2 className="flex items-center gap-2 font-bold">
              <ArrowUpRight className="size-4 text-gold-400" aria-hidden />
              📤 &#x62f;&#x631;&#x62e;&#x648;&#x627;&#x633;&#x62a;&#x650; &#x628;&#x631;&#x62f;&#x627;&#x634;&#x62a;&#x650; &#x62f;&#x631;&#x622;&#x645;&#x62f; (TON / USDT)
            </h2>
            <Badge variant="default" className="bg-gold-400/15 text-gold-300 ring-1 ring-gold-400/30">
              &#x642;&#x641;&#x644; &#x627;&#x62a;&#x645;&#x6cc;
            </Badge>
          </div>
          <p className="mb-4 text-xs leading-5 text-zinc-500">
            &#x628;&#x627; &#x62b;&#x628;&#x62a; &#x62f;&#x631;&#x62e;&#x648;&#x627;&#x633;&#x62a;&#x60c; &#x645;&#x628;&#x644;&#x63a; Đ &#x628;&#x647;‌&#x635;&#x648;&#x631;&#x62a; &#x627;&#x62a;&#x645;&#x6cc; &#x62f;&#x631; &#x62f;&#x641;&#x62a;&#x631; &#x62f;&#x648;&#x637;&#x631;&#x641;&#x647; &#x642;&#x641;&#x644; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f; &#x62a;&#x627; &#x627;&#x632; &#x62f;&#x627;&#x628;&#x644;‌&#x627;&#x633;&#x67e;&#x646;&#x62f; &#x62c;&#x644;&#x648;&#x6af;&#x6cc;&#x631;&#x6cc;
            &#x634;&#x648;&#x62f;. &#x627;&#x6af;&#x631; &#x627;&#x62f;&#x645;&#x6cc;&#x646; &#x62f;&#x631;&#x62e;&#x648;&#x627;&#x633;&#x62a; &#x631;&#x627; &#x631;&#x62f; &#x6a9;&#x646;&#x62f;&#x60c; &#x6a9;&#x644; &#x645;&#x628;&#x644;&#x63a; &#x628;&#x647; &#x645;&#x648;&#x62c;&#x648;&#x62f;&#x6cc; &#x634;&#x645;&#x627; &#x628;&#x631;&#x645;&#x6cc;‌&#x6af;&#x631;&#x62f;&#x62f;.
          </p>
          <form onSubmit={(e) => void submitWithdrawal(e)} className="grid gap-3">
            <div className="grid grid-cols-2 gap-2">
              {WITHDRAWAL_CHAINS.map((opt) => (
                <button
                  key={opt.chain}
                  type="button"
                  onClick={() => selectChain(opt.chain)}
                  className={`rounded-xl border px-3 py-2 text-right text-xs font-medium transition ${
                    wChain === opt.chain
                      ? "border-gold-400/60 bg-gold-400/10 text-gold-300"
                      : "border-zinc-800 bg-zinc-950/60 text-zinc-400 hover:border-zinc-700"
                  }`}
                >
                  {opt.label}
                </button>
              ))}
            </div>

            {wChain === "TON" && (
              <div className="grid grid-cols-2 gap-2">
                {(["USDT", "TON"] as const).map((asset) => (
                  <button
                    key={asset}
                    type="button"
                    onClick={() => setWAsset(asset)}
                    className={`rounded-lg border px-3 py-1.5 text-xs font-semibold transition ${
                      wAsset === asset
                        ? "border-gold-400/60 bg-gold-400/15 text-gold-300"
                        : "border-zinc-800 bg-zinc-950/60 text-zinc-400"
                    }`}
                  >
                    &#x62f;&#x627;&#x631;&#x627;&#x6cc;&#x6cc;: {asset}
                  </button>
                ))}
              </div>
            )}

            <label className="text-xs text-zinc-400">
              &#x622;&#x62f;&#x631;&#x633; &#x6a9;&#x6cc;&#x641; &#x67e;&#x648;&#x644; &#x645;&#x642;&#x635;&#x62f; ({wChain})
              <input
                value={wAddress}
                onChange={(e) => setWAddress(e.target.value)}
                required
                dir="ltr"
                placeholder={
                  WITHDRAWAL_CHAINS.find((c) => c.chain === wChain)?.placeholder ?? "Address"
                }
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
              />
            </label>

            <label className="text-xs text-zinc-400">
              &#x645;&#x628;&#x644;&#x63a; &#x628;&#x631;&#x62f;&#x627;&#x634;&#x62a; &#x628;&#x647; &#x62f;&#x6cc;&#x6cc;&#x646; (&#x645;&#x648;&#x62c;&#x648;&#x62f;&#x6cc; &#x622;&#x632;&#x627;&#x62f;: {formatDin(me.balance)})
              <input
                value={wAmount}
                onChange={(e) => setWAmount(e.target.value)}
                required
                inputMode="numeric"
                dir="ltr"
                placeholder="250"
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
              />
            </label>

            <button
              type="submit"
              disabled={withdrawing}
              className="rounded-xl bg-gradient-to-b from-gold-400 to-gold-600 px-4 py-2.5 text-sm font-black text-zinc-950 shadow-lg shadow-gold-500/20 transition-opacity hover:opacity-90 disabled:opacity-50"
            >
              {withdrawing
                ? "\u062f\u0631 \u062d\u0627\u0644 \u0642\u0641\u0644 \u0648 \u062b\u0628\u062a…"
                : `\u062b\u0628\u062a \u0628\u0631\u062f\u0627\u0634\u062a ${wChain === "TON" ? wAsset : "USDT"} \u0631\u0648\u06cc ${wChain}`}
            </button>
          </form>

          {withdrawals.length > 0 && (
            <div className="mt-5 border-t border-zinc-800 pt-4">
              <h3 className="mb-3 text-xs font-bold text-zinc-400">&#x648;&#x636;&#x639;&#x6cc;&#x62a; &#x628;&#x631;&#x62f;&#x627;&#x634;&#x62a;‌&#x647;&#x627;&#x6cc; &#x645;&#x646;</h3>
              <ul className="flex flex-col gap-2.5">
                {withdrawals.map((w) => {
                  const st = WITHDRAWAL_STATUS[w.status] ?? {
                    fa: w.status,
                    className: "bg-zinc-500/15 text-zinc-400 ring-zinc-400/30",
                  };
                  return (
                    <li
                      key={w.id}
                      className="flex flex-col gap-1.5 rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800"
                    >
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-mono text-sm font-bold text-gold-400">
                          {formatDin(w.amountDin)}
                        </span>
                        <span className="font-mono text-xs text-zinc-300" dir="ltr">
                          {w.asset} · {w.chain}
                        </span>
                        <Badge variant="default" className={`ring-1 ${st.className}`}>
                          {st.fa}
                        </Badge>
                        <span className="mr-auto text-xs text-zinc-600">
                          {formatWhen(new Date(w.createdAt * 1000).toISOString())}
                        </span>
                      </div>
                      <p className="truncate font-mono text-xs text-zinc-500" dir="ltr">
                        → {w.destinationAddress}
                      </p>
                      {w.txHash && (
                        <p className="truncate font-mono text-xs text-emerald-400" dir="ltr">
                          tx: {w.txHash}
                        </p>
                      )}
                    </li>
                  );
                })}
              </ul>
            </div>
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
    </MiniShell>
  );
}
