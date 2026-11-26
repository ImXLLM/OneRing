"use client";

/**
 * Tab 2 — Vibe-Trading & CCXT Pocket Desk.
 *
 * The 8 keyless exchange venues, a causal backtest preview through
 * `POST /api/v1/quant/vibe-backtest` (strictly `&candles[0..=t]`), the
 * Sharpe >= 1.20 live-deployment gate badge, and a WebCrypto SHA-256 PnL
 * claim verifier — every number traceable to the kernel's claims ledger.
 */

import { useCallback, useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { describeError, useMaironMiniApp } from "@/lib/mairon-state";

/** The live-deployment Sharpe gate (mirrors `MAIRON_MIN_SHARPE`). */
const MIN_SHARPE = 1.2;

interface VenueRow {
  id: string;
  displayName: string;
  venueKind: string;
  auth: string;
}

interface BacktestSummary {
  sharpeRatio?: number;
  maxDrawdownPercent?: number;
  tradeCount?: number;
  finalEquity?: number;
  claimVerificationHash?: string;
  maironApproved?: boolean;
  canonicalClaimString?: string;
}

export function TabQuantDesk() {
  const { state, sdk, patchDraft, registerJob, setError } = useMaironMiniApp();
  const { drafts } = state;
  const [venues, setVenues] = useState<VenueRow[]>([]);
  const [running, setRunning] = useState(false);
  const [report, setReport] = useState<BacktestSummary | null>(null);
  const [verifying, setVerifying] = useState(false);
  const [verification, setVerification] = useState<{ verified: boolean; sha256: string } | null>(null);

  const loadVenues = useCallback(async () => {
    try {
      const res = await sdk.quant.listVenues();
      setVenues(
        (res.venues as unknown as VenueRow[]).map((v) => ({
          id: v.id,
          displayName: v.displayName,
          venueKind: v.venueKind,
          auth: v.auth,
        })),
      );
    } catch (err) {
      setError(describeError(err));
    }
  }, [sdk, setError]);

  useEffect(() => {
    void loadVenues();
  }, [loadVenues]);

  async function runBacktest(): Promise<void> {
    setRunning(true);
    setError(null);
    setVerification(null);
    const jobId = `backtest-${Date.now()}`;
    registerJob({
      jobId,
      perspective: "desk",
      label: `Causal backtest ${drafts.deskSymbol}`,
      status: "running",
      startedAtIso: new Date().toISOString(),
      detail: drafts.deskVibePrompt.slice(0, 120),
    });
    try {
      const res = await sdk.quant.runCausalBacktest({
        vibePrompt: drafts.deskVibePrompt,
        symbol: drafts.deskSymbol,
        bars: 400,
      });
      const summary = (res as { summary?: BacktestSummary }).summary ?? (res as unknown as BacktestSummary);
      setReport(summary);
      registerJob({
        jobId,
        perspective: "desk",
        label: `Causal backtest ${drafts.deskSymbol}`,
        status: "succeeded",
        startedAtIso: new Date().toISOString(),
        detail: typeof summary.sharpeRatio === "number" ? `Sharpe ${summary.sharpeRatio.toFixed(2)}` : "finished",
      });
    } catch (err) {
      setError(describeError(err));
      registerJob({
        jobId,
        perspective: "desk",
        label: `Causal backtest ${drafts.deskSymbol}`,
        status: "failed",
        startedAtIso: new Date().toISOString(),
        detail: describeError(err).slice(0, 200),
      });
    } finally {
      setRunning(false);
    }
  }

  async function verifyClaim(): Promise<void> {
    if (!report?.claimVerificationHash || !report?.canonicalClaimString) {
      setError("Run a backtest first — the claim hash comes from the kernel ledger.");
      return;
    }
    setVerifying(true);
    setError(null);
    try {
      const res = await sdk.quant.verifyClaimHash(report.claimVerificationHash, report.canonicalClaimString);
      setVerification({ verified: res.verified, sha256: res.recomputedSha256 });
    } catch (err) {
      setError(describeError(err));
    } finally {
      setVerifying(false);
    }
  }

  const sharpeOk = typeof report?.sharpeRatio === "number" && report.sharpeRatio >= MIN_SHARPE;

  return (
    <section aria-label="Vibe-trading and CCXT pocket desk" className="flex flex-col gap-4">
      <div className="rounded-2xl border border-zinc-700/80 bg-zinc-900/60 p-3">
        <div className="flex items-center justify-between">
          <span className="text-[11px] font-semibold uppercase tracking-wider text-zinc-500">
            Keyless venues ({venues.length || 8})
          </span>
          <button
            type="button"
            onClick={() => void loadVenues()}
            className="rounded-lg border border-zinc-700 px-2 py-1 text-[11px] text-zinc-300"
          >
            Refresh
          </button>
        </div>
        <div className="mt-2 flex flex-wrap gap-1">
          {venues.map((v) => (
            <span
              key={v.id}
              className="rounded-full border border-zinc-700 bg-zinc-800/60 px-2 py-0.5 font-mono text-[10px] text-zinc-300"
              title={`${v.venueKind} · ${v.auth}`}
            >
              {v.id}
            </span>
          ))}
        </div>
        <p className="mt-2 text-[10px] text-zinc-500">Zero API keys, zero KYC — public market data only.</p>
      </div>

      <label className="text-[11px] font-semibold uppercase tracking-wider text-zinc-500" htmlFor="desk-symbol">
        Symbol
      </label>
      <input
        id="desk-symbol"
        value={drafts.deskSymbol}
        onChange={(e) => patchDraft({ deskSymbol: e.target.value })}
        aria-label="Trading pair symbol for the causal backtest"
        className="w-full rounded-xl border border-zinc-700 bg-zinc-900/80 px-3 py-2 font-mono text-sm text-zinc-100 outline-none focus:border-sky-400/60"
      />

      <label className="text-[11px] font-semibold uppercase tracking-wider text-zinc-500" htmlFor="desk-vibe">
        Vibe prompt
      </label>
      <textarea
        id="desk-vibe"
        value={drafts.deskVibePrompt}
        onChange={(e) => patchDraft({ deskVibePrompt: e.target.value })}
        aria-label="Describe the trading strategy in plain words"
        rows={2}
        className="w-full rounded-xl border border-zinc-700 bg-zinc-900/80 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-sky-400/60"
      />

      <button
        type="button"
        onClick={() => void runBacktest()}
        disabled={running}
        className="flex h-12 items-center justify-center gap-2 rounded-2xl bg-sky-400/15 text-sm font-bold text-sky-200 shadow-[inset_0_0_0_1px_rgba(56,189,248,0.35)] transition active:scale-[0.98] disabled:opacity-60"
      >
        {running ? <Loader2 className="size-4 animate-spin" aria-hidden /> : "📈"} Run causal backtest
      </button>

      {report ? (
        <div className="flex flex-col gap-2 rounded-2xl border border-zinc-700/80 bg-zinc-900/60 p-3">
          <div className="flex items-center justify-between">
            <span className="font-mono text-sm text-zinc-200">
              Sharpe {typeof report.sharpeRatio === "number" ? report.sharpeRatio.toFixed(2) : "—"}
            </span>
            <span
              className={`rounded-full px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider ${
                sharpeOk ? "bg-emerald-500/15 text-emerald-300" : "bg-zinc-700/60 text-zinc-400"
              }`}
            >
              {sharpeOk ? "Sharpe >= 1.20 — live gate open" : "below the 1.20 live gate"}
            </span>
          </div>
          <p className="text-[11px] text-zinc-400">
            Drawdown {typeof report.maxDrawdownPercent === "number" ? `${report.maxDrawdownPercent.toFixed(1)}%` : "—"} ·
            trades {typeof report.tradeCount === "number" ? report.tradeCount : "—"} · strictly causal fills
          </p>
          {report.claimVerificationHash ? (
            <>
              <p className="break-all font-mono text-[10px] text-zinc-500">
                claim {report.claimVerificationHash.slice(0, 24)}…
              </p>
              <button
                type="button"
                onClick={() => void verifyClaim()}
                disabled={verifying}
                className="rounded-xl border border-zinc-700 px-3 py-2 text-xs font-semibold text-zinc-200 disabled:opacity-60"
              >
                {verifying ? <Loader2 className="size-3.5 animate-spin" aria-hidden /> : "SHA-256"} Verify PnL claim
              </button>
              {verification ? (
                <p className={`text-[11px] ${verification.verified ? "text-emerald-300" : "text-red-300"}`}>
                  {verification.verified ? "VERIFIED" : "MISMATCH"} · {verification.sha256.slice(0, 32)}…
                </p>
              ) : null}
            </>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}
