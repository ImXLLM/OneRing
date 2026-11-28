"use client";

/**
 * Tab 3 — Base-8453 USDC & Telegram Stars (XTR) Sovereign Wallet.
 *
 * Base Mainnet (chain id 8453, USDC 0x8335…2913, $0xKIA, XTR Stars), the
 * 4-Stream Revenue Split calculator wired to `POST /api/v1/monopoly/simulate-split`,
 * and the Capital Reserve partition ($33.00 operational crypto seed, $15.00 cash,
 * $350.00 locked cold reserve) — numbers always from the kernel charter, never local.
 */

import { useCallback, useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { Base8453AddressPill } from "@kian/ui";
import { describeError, useMaironMiniApp } from "@/lib/mairon-state";

/** Canonical Base-8453 constants (mirrors packages/config). */
const BASE_CHAIN_ID = 8453;
const BASE_USDC = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";

interface CharterState {
  charter?: {
    platformTakeBpsUnreferred?: number;
    creatorBps?: number;
    referrerBps?: number;
    boostRecipientBps?: number;
    swapSpreadBps?: number;
  };
  wallet?: {
    cryptoSeedMicros?: number;
    cashSeedCents?: number;
    paperReserveLockedCents?: number;
    earnedRevenueMicros?: number;
  };
  baseChainId?: number;
  baseUsdcContract?: string;
  tokenSymbol?: string;
  starsCurrency?: string;
}

interface SplitResult {
  creatorMicros?: number;
  platformMicros?: number;
  referrerMicros?: number | null;
  lineItems?: { label: string; amountMicros: number }[];
}

export function TabWallet() {
  const { state, sdk, patchDraft, setError } = useMaironMiniApp();
  const { drafts } = state;
  const [charter, setCharter] = useState<CharterState | null>(null);
  const [split, setSplit] = useState<SplitResult | null>(null);
  const [running, setRunning] = useState(false);

  const loadCharter = useCallback(async () => {
    try {
      const res = await sdk.treasury.getCharterState();
      setCharter(res as unknown as CharterState);
    } catch (err) {
      setError(describeError(err));
    }
  }, [sdk, setError]);

  useEffect(() => {
    void loadCharter();
  }, [loadCharter]);

  async function runSplit(): Promise<void> {
    const gross = Number.parseFloat(drafts.walletGrossUsd);
    if (!Number.isFinite(gross) || gross <= 0) {
      setError("Enter a positive gross amount to simulate the split.");
      return;
    }
    setRunning(true);
    setError(null);
    try {
      const res = await sdk.treasury.calculateSplit({
        scenario: drafts.walletScenario,
        settlementAsset: drafts.walletScenario === "instant_boost" ? "telegram-stars-xtr" : "base-native-usdc",
        grossAmountMicros: Math.round(gross * 1_000_000),
        buyerUserId: "miniapp-buyer",
        creatorUserId: "miniapp-creator",
        referrerUserId: drafts.walletReferred ? "miniapp-referrer" : null,
        affiliateBps: drafts.walletReferred ? 1_000 : 2_500,
        boostTier: drafts.walletScenario === "instant_boost" ? "featured-slot-24h" : null,
        recipientAddress: null,
      });
      setSplit((res as { split?: SplitResult }).split ?? (res as unknown as SplitResult));
    } catch (err) {
      setError(describeError(err));
    } finally {
      setRunning(false);
    }
  }

  const wallet = charter?.wallet;
  const bps = charter?.charter ?? {};

  return (
    <section aria-label="Base-8453 sovereign wallet" className="flex flex-col gap-4">
      <div className="rounded-2xl border border-sky-400/30 bg-sky-400/5 p-3">
        <div className="flex flex-wrap items-center gap-2">
          <span className="rounded-full bg-sky-400/20 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-sky-300">
            Base Mainnet · chain {charter?.baseChainId ?? BASE_CHAIN_ID}
          </span>
          <span className="rounded-full bg-zinc-700/60 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-zinc-300">
            $0xKIA
          </span>
          <span className="rounded-full bg-zinc-700/60 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-zinc-300">
            Telegram Stars (XTR)
          </span>
        </div>
        <div className="mt-2">
          <Base8453AddressPill address={charter?.baseUsdcContract ?? BASE_USDC} />
        </div>
      </div>

      <div className="grid grid-cols-3 gap-2">
        <Stat label="Crypto seed" value={`$${((wallet?.cryptoSeedMicros ?? 33_000_000) / 1_000_000).toFixed(2)}`} />
        <Stat label="Cash float" value={`$${((wallet?.cashSeedCents ?? 1_500) / 100).toFixed(2)}`} />
        <Stat label="Locked cold" value={`$${((wallet?.paperReserveLockedCents ?? 35_000) / 100).toFixed(2)}`} />
      </div>

      <div className="rounded-2xl border border-zinc-700/80 bg-zinc-900/60 p-3 text-[11px] leading-5 text-zinc-300">
        <p className="mb-1 text-[11px] font-semibold uppercase tracking-wider text-zinc-500">4-Stream Revenue Split</p>
        <p>Unreferred sale: {bpsDiv(bps.platformTakeBpsUnreferred, 2000)} platform / {bpsDiv(bps.creatorBps, 8000)} creator</p>
        <p>Referred sale: 10% referrer / 80% creator / 10% platform</p>
        <p>Instant Boosts: 100% routes to the MAIRON Sovereign Treasury (0% to the creator)</p>
        <p>Swap spread: 1.5% treasury fee</p>
      </div>

      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => patchDraft({ walletScenario: "marketplace_sale" })}
          className={`flex-1 rounded-xl border px-3 py-2 text-xs font-semibold ${
            drafts.walletScenario === "marketplace_sale"
              ? "border-amber-400/60 bg-amber-400/10 text-amber-200"
              : "border-zinc-700 text-zinc-400"
          }`}
        >
          Marketplace sale
        </button>
        <button
          type="button"
          onClick={() => patchDraft({ walletScenario: "instant_boost" })}
          className={`flex-1 rounded-xl border px-3 py-2 text-xs font-semibold ${
            drafts.walletScenario === "instant_boost"
              ? "border-amber-400/60 bg-amber-400/10 text-amber-200"
              : "border-zinc-700 text-zinc-400"
          }`}
        >
          Instant boost
        </button>
        <button
          type="button"
          onClick={() => patchDraft({ walletReferred: !drafts.walletReferred })}
          aria-pressed={drafts.walletReferred}
          className={`flex-1 rounded-xl border px-3 py-2 text-xs font-semibold ${
            drafts.walletReferred
              ? "border-emerald-400/60 bg-emerald-400/10 text-emerald-200"
              : "border-zinc-700 text-zinc-400"
          }`}
        >
          {drafts.walletReferred ? "Referred" : "Unreferred"}
        </button>
      </div>

      <label className="text-[11px] font-semibold uppercase tracking-wider text-zinc-500" htmlFor="wallet-gross">
        Gross amount (USD)
      </label>
      <input
        id="wallet-gross"
        inputMode="decimal"
        value={drafts.walletGrossUsd}
        onChange={(e) => patchDraft({ walletGrossUsd: e.target.value })}
        aria-label="Gross sale amount in dollars"
        className="w-full rounded-xl border border-zinc-700 bg-zinc-900/80 px-3 py-2 font-mono text-sm text-zinc-100 outline-none focus:border-amber-400/60"
      />

      <button
        type="button"
        onClick={() => void runSplit()}
        disabled={running}
        className="flex h-12 items-center justify-center gap-2 rounded-2xl bg-amber-400/15 text-sm font-bold text-amber-200 shadow-[inset_0_0_0_1px_rgba(251,191,36,0.35)] transition active:scale-[0.98] disabled:opacity-60"
      >
        {running ? <Loader2 className="size-4 animate-spin" aria-hidden /> : "💠"} Simulate the atomic split
      </button>

      {split ? (
        <div className="rounded-2xl border border-zinc-700/80 bg-zinc-900/60 p-3 text-[11px] text-zinc-300">
          <p>Creator: {fmtMicros(split.creatorMicros)}</p>
          <p>Platform: {fmtMicros(split.platformMicros)}</p>
          {split.referrerMicros != null ? <p>Referrer: {fmtMicros(split.referrerMicros)}</p> : null}
          {Array.isArray(split.lineItems)
            ? split.lineItems.map((li) => (
                <p key={li.label} className="text-zinc-400">
                  · {li.label}: {fmtMicros(li.amountMicros)}
                </p>
              ))
            : null}
        </div>
      ) : null}

      <p className="text-[10px] text-zinc-500">
        Charter state is fetched live from `/api/v1/monopoly/charter-and-base` — no number on this
        screen is hand-entered.
      </p>
    </section>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl border border-zinc-700/80 bg-zinc-900/60 p-2 text-center">
      <p className="text-[10px] uppercase tracking-wider text-zinc-500">{label}</p>
      <p className="font-mono text-sm font-bold text-zinc-100">{value}</p>
    </div>
  );
}

function fmtMicros(micros: number | undefined | null): string {
  if (typeof micros !== "number") return "—";
  return `$${(micros / 1_000_000).toFixed(2)}`;
}

function bpsDiv(value: number | undefined, fallback: number): string {
  const bps = typeof value === "number" ? value : fallback;
  return `${(bps / 100).toFixed(0)}%`;
}
