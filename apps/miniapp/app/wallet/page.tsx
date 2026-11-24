"use client";

/**
 * KIAN miniapp — ولت: تراز دفتر دوطرفه، اتصال TON، ثبت اثبات واریز، تاریخچه.
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
    fa: "قفل‌شده · در صف پرداخت",
    className: "bg-gold-400/15 text-gold-300 ring-gold-400/30",
  },
  CONFIRMED: {
    fa: "واریز شد",
    className: "bg-emerald-500/15 text-emerald-300 ring-emerald-400/30",
  },
  REJECTED: {
    fa: "رد شد · وجه برگشت خورد",
    className: "bg-rose-500/15 text-rose-300 ring-rose-400/30",
  },
};

const WITHDRAWAL_CHAINS: { chain: WithdrawalChain; label: string; placeholder: string }[] = [
  { chain: "TON", label: "TON (TON / USDT)", placeholder: "UQ… یا EQ…" },
  { chain: "TRX", label: "TRON · TRC-20 (USDT)", placeholder: "T…" },
  { chain: "SOL", label: "Solana · SPL (USDT)", placeholder: "آدرس Base58 سولانا" },
  { chain: "BSC", label: "BNB Chain · BEP-20 (USDT)", placeholder: "0x…" },
];

// TON Connect فقط روی کلاینت — کتابخانه در سطح ماژول به مرورگر وابسته است
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
  // شیر اعتبار آزمایشی — فقط برای پرسوناهای دمو نمایش داده می‌شود
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
      /* لیست غیرحیاتی — بی‌سروصدا */
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
          <span className="text-sm">برقراری نشست با کرنل…</span>
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
            title="ولت امن KIAN داخل تلگرام باز می‌شود"
            hint="اتصال ولت TON، ثبت واریز و برداشت کریپتو همه از نشست احرازشدهٔ تلگرام شما می‌گذرد تا دارایی‌تان همیشه امن بماند."
            whatWorksInPreview={state.phase !== "preview"}
          />
        </div>
      </MiniShell>
    );
  }

  const { me, token } = state;

  // شیر اعتبار آزمایشی (faucet) — ۵٬۰۰۰ KIAN رایگان؛ فقط در حالت devِ سرور
  const claimFaucet = async () => {
    setClaimingFaucet(true);
    try {
      const res = await api.claimDemoFaucet(token);
      toast({
        title: "🎁 اعتبار تست اضافه شد",
        description: `${formatDin(res.balance)} در کیف‌پول تو آمادهٔ خرید و تست است.`,
        variant: "success",
      });
      await refresh();
    } catch (err) {
      toast({
        title: "دریافت اعتبار تست ناموفق",
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
      toast({ title: "آدرس TON ثبت شد", variant: "success" });
      await refresh();
    } catch (err) {
      toast({
        title: "ثبت آدرس ناموفق",
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
        title: "اثبات واریز ثبت شد",
        description: "پس از تأیید ادمین، دیین مطابق مادهٔ ۳ ضرب می‌شود.",
        variant: "success",
      });
      setTxHash("");
      setAmountTon("");
      await Promise.all([loadHistory(token), refresh()]);
    } catch (err) {
      toast({
        title: "ثبت واریز ناموفق",
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
        title: "مبلغ نامعتبر",
        description: "مبلغ برداشت باید عدد صحیح بزرگ‌تر از صفر باشد.",
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
        title: "درخواست برداشت ثبت و قفل شد",
        description: `${formatDin(created.amountDin)} برای پرداخت روی ${created.chain} (${created.asset}) در صف قرار گرفت.`,
        variant: "success",
      });
      await refresh();
    } catch (err) {
      toast({
        title: "ثبت برداشت ناموفق",
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
        {/* هویت و تراز */}
        <section className="rounded-2xl border border-royal-500/20 bg-gradient-to-b from-royal-500/10 to-transparent p-5">
          <div className="flex items-center gap-2 text-sm text-zinc-400">
            <Coins className="size-4 text-gold-400" aria-hidden />
            تراز مشتق‌شده از دفتر دوطرفه
          </div>
          <p className="mt-3 text-3xl font-black text-gold-400">{formatDin(me.balance)}</p>
          <p className="mt-2 text-xs text-zinc-500">
            قفل‌شده در اسکرو خریدها:{" "}
            <span className="font-semibold text-zinc-300">{formatDin(me.escrowLocked)}</span>
          </p>
          <p className="mt-2 text-xs text-zinc-500">
            ولت متصل:{" "}
            {me.user.tonAddress ? (
              <span className="font-mono text-emerald-400" dir="ltr">
                {me.user.tonAddress.slice(0, 10)}…{me.user.tonAddress.slice(-6)}
              </span>
            ) : (
              <span className="text-zinc-600">هنوز وصل نشده</span>
            )}
          </p>

          {/* شیر اعتبار آزمایشی — فقط پرسوناهای دمو (۱۰۰۱/۲۰۰۲/۳۰۰۳) */}
          {activePersona && (
            <div className="mt-4 flex flex-col gap-2 rounded-xl border border-emerald-400/30 bg-emerald-500/10 p-3">
              <p className="text-xs font-bold leading-5 text-emerald-300">
                حالت تست فعال است — برای امتحانِ کاملِ خرید و برداشت، اعتبار رایگان بگیر:
              </p>
              <button
                type="button"
                disabled={claimingFaucet}
                onClick={() => void claimFaucet()}
                className="flex items-center justify-center gap-2 rounded-xl bg-gradient-to-b from-emerald-500 to-emerald-600 px-4 py-3 text-sm font-black text-white shadow-lg shadow-emerald-600/25 ring-1 ring-emerald-400/50 transition hover:opacity-90 disabled:opacity-50"
              >
                {claimingFaucet
                  ? "در حال دریافت…"
                  : "🎁 دریافت ۵٬۰۰۰ KIAN اعتبار تست رایگان (Faucet)"}
              </button>
            </div>
          )}
        </section>

        {/* TON Connect */}
        <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
          <h2 className="mb-3 flex items-center gap-2 font-bold">
            <Link2 className="size-4 text-royal-400" aria-hidden />
            اتصال ولت TON
          </h2>
          <TonConnectSection
            onRegisterAddress={(addr) => void registerAddress(addr)}
            registering={registering}
          />
        </section>

        {/* واریز TON */}
        <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
          <h2 className="mb-1 font-bold">واریز TON — ثبت اثبات</h2>
          <p className="mb-4 text-xs leading-5 text-zinc-500">
            مبلغ را به TON وارد کنید (حداکثر ۶ رقم اعشار). پس از تأیید ادمین،
            دیین با نرخ تبدیل کرنل مطابق مادهٔ ۳ ضرب می‌شود.
          </p>
          <form onSubmit={(e) => void submitDeposit(e)} className="grid gap-3">
            <label className="text-xs text-zinc-400">
              مبلغ (TON)
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
              هش تراکنش زنجیره
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
              آدرس ولت فرستنده
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
              {submitting ? "در حال ثبت…" : "ثبت اثبات واریز"}
            </button>
          </form>
        </section>

        {/* تاریخچهٔ واریز */}
        <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
          <h2 className="mb-3 flex items-center gap-2 font-bold">
            <ReceiptText className="size-4 text-royal-400" aria-hidden />
            واریزهای من
          </h2>
          {deposits.length === 0 ? (
            <p className="py-4 text-center text-sm text-zinc-600">هنوز واریزی ثبت نشده است.</p>
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

        {/* فاز ۶ — درخواست برداشت درآمد (TON / USDT) */}
        <section className="rounded-2xl border border-gold-400/25 bg-zinc-900/60 p-5">
          <div className="mb-1 flex items-center justify-between gap-2">
            <h2 className="flex items-center gap-2 font-bold">
              <ArrowUpRight className="size-4 text-gold-400" aria-hidden />
              📤 درخواستِ برداشتِ درآمد (TON / USDT)
            </h2>
            <Badge variant="default" className="bg-gold-400/15 text-gold-300 ring-1 ring-gold-400/30">
              قفل اتمی
            </Badge>
          </div>
          <p className="mb-4 text-xs leading-5 text-zinc-500">
            با ثبت درخواست، مبلغ Đ به‌صورت اتمی در دفتر دوطرفه قفل می‌شود تا از دابل‌اسپند جلوگیری
            شود. اگر ادمین درخواست را رد کند، کل مبلغ به موجودی شما برمی‌گردد.
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
                    دارایی: {asset}
                  </button>
                ))}
              </div>
            )}

            <label className="text-xs text-zinc-400">
              آدرس کیف پول مقصد ({wChain})
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
              مبلغ برداشت به دیین (موجودی آزاد: {formatDin(me.balance)})
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
                ? "در حال قفل و ثبت…"
                : `ثبت برداشت ${wChain === "TON" ? wAsset : "USDT"} روی ${wChain}`}
            </button>
          </form>

          {withdrawals.length > 0 && (
            <div className="mt-5 border-t border-zinc-800 pt-4">
              <h3 className="mb-3 text-xs font-bold text-zinc-400">وضعیت برداشت‌های من</h3>
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

        {/* آخرین ردیف‌های دفتر */}
        <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
          <h2 className="mb-3 font-bold">آخرین ردیف‌های دفتر</h2>
          {me.recentTxs.length === 0 ? (
            <p className="py-4 text-center text-sm text-zinc-600">دفتر برای این حساب خالی است.</p>
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
