"use client";

/**
 * KIAN M3 — صفحهٔ ولت مینی‌اپ.
 *
 * تراز مشتق‌شده از دفتر دوطرفه + قفل اسکرو + آخرین ردیف‌های دفتر؛
 * TON Connect برای اتصال ولت و ثبت آدرس روی هویت؛ واریز TON با اثبات
 * کاربر (PENDING) تا تأیید ادمین و ضرب مادهٔ ۳ در کرنل.
 */

import dynamic from "next/dynamic";
import { useCallback, useEffect, useState } from "react";
import { Badge } from "@kian/ui";
import { toast } from "@kian/ui";
import { Coins, Landmark, Link2, Loader2, ReceiptText } from "lucide-react";
import { useSession } from "@/components/kian/session";
import { ApiError, api, type TonDepositView } from "@/lib/api";
import { DEPOSIT_STATUS, formatDin, formatWhen } from "@/lib/format";

// TON Connect فقط روی کلاینت — کتابخانه در سطح ماژول به مرورگر وابسته است
const TonConnectSection = dynamic(
  () => import("@/components/kian/ton-connect"),
  { ssr: false, loading: () => <div className="h-16 animate-pulse rounded-xl bg-zinc-900/60" /> },
);

function SessionGate() {
  return (
    <div className="flex flex-col items-center gap-4 rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-10 text-center">
      <Landmark className="size-10 text-gold-400" aria-hidden />
      <h1 className="text-xl font-black">ولت KIAN</h1>
      <p className="max-w-md text-sm leading-6 text-zinc-400">
        این مینی‌اپ باید از داخل تلگرام باز شود (یا با لینک رسمی مینی‌اپ که
        <code className="mx-1 rounded bg-zinc-800 px-1.5 py-0.5 text-xs" dir="ltr">tgWebAppData</code>
        دارد). هویت هرگز از بدنهٔ درخواست ساخته نمی‌شود — فقط با امضای HMAC کرنل.
      </p>
      <button
        type="button"
        onClick={() => window.location.reload()}
        className="rounded-xl bg-royal-500/20 px-4 py-2 text-sm font-semibold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
      >
        تلاش دوباره
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
      /* لیست غیرحیاتی — بی‌سروصدا */
    }
  }, []);

  useEffect(() => {
    if (state.phase === "ready") void loadDeposits(state.token);
  }, [state, loadDeposits]);

  if (state.phase === "loading") {
    return (
      <div className="flex items-center justify-center gap-3 py-24 text-zinc-500">
        <Loader2 className="size-5 animate-spin" aria-hidden />
        <span className="text-sm">برقراری نشست با کرنل…</span>
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
      toast({ title: "آدرس TON ثبت شد", variant: "success" });
      await refresh();
    } catch (err) {
      toast({
        title: "ثبت آدرس ناموفق",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "خطای شبکه",
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
      await Promise.all([loadDeposits(token), refresh()]);
    } catch (err) {
      toast({
        title: "ثبت واریز ناموفق",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "خطای شبکه",
        variant: "destructive",
      });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col gap-6">
      {/* هویت و تراز */}
      <section className="grid gap-4 sm:grid-cols-2">
        <div className="rounded-2xl border border-royal-500/20 bg-gradient-to-b from-royal-500/10 to-transparent p-5">
          <div className="flex items-center gap-2 text-sm text-zinc-400">
            <Coins className="size-4 text-gold-400" aria-hidden />
            تراز مشتق‌شده از دفتر دوطرفه
          </div>
          <p className="mt-3 text-3xl font-black text-gold-400">{formatDin(me.balance)}</p>
          <p className="mt-2 text-xs text-zinc-500">
            قفل‌شده در اسکرو خریدها: <span className="font-semibold text-zinc-300">{formatDin(me.escrowLocked)}</span>
          </p>
        </div>
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
          <div className="text-sm text-zinc-400">هویت تلگرام</div>
          <p className="mt-2 font-bold">
            {me.user.username ? `@${me.user.username}` : `کاربر ${me.user.telegramId}`}
          </p>
          <p className="mt-1 font-mono text-xs text-zinc-500" dir="ltr">
            {me.user.accountId}
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
        </div>
      </section>

      {/* TON Connect */}
      <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
        <h2 className="mb-3 flex items-center gap-2 font-bold">
          <Link2 className="size-4 text-royal-400" aria-hidden />
          اتصال ولت TON
        </h2>
        <TonConnectSection onRegisterAddress={(addr) => void registerAddress(addr)} registering={registering} />
      </section>

      {/* واریز TON */}
      <section className="rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5">
        <h2 className="mb-1 font-bold">واریز TON — ثبت اثبات</h2>
        <p className="mb-4 text-xs leading-5 text-zinc-500">
          مبلغ را به TON وارد کنید (حداکثر ۶ رقم اعشار). پس از تأیید ادمین،
          دیین با نرخ تبدیل کرنل مطابق مادهٔ ۳ ضرب می‌شود. هش تراکنش یکتاست —
          اثبات تکراری ۴۲۲ می‌گیرد.
        </p>
        <form onSubmit={(e) => void submitDeposit(e)} className="grid gap-3 sm:grid-cols-3">
          <label className="text-xs text-zinc-400 sm:col-span-1">
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
          <label className="text-xs text-zinc-400 sm:col-span-2">
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
          <label className="text-xs text-zinc-400 sm:col-span-3">
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
            className="sm:col-span-3 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2.5 text-sm font-bold text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 transition-opacity hover:opacity-90 disabled:opacity-50"
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
  );
}
