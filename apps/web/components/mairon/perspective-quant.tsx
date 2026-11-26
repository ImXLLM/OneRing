"use client";

/**
 * Perspective 2 — Quantitative Vibe-Trading & CCXT Execution Desk.
 *
 * Strategy prompt runner against the causal `&candles[0..=t]` backtester,
 * the `Sharpe >= 1.20` live-gate badge, the five-node DAG telemetry, the
 * fourteen institutional metrics, and the SHA-256 verifiable PnL claim hash.
 */

import { useCallback, useState } from "react";

import { QUANT_LIVE_SHARPE_GATE } from "@kian/contracts";
import { useMaironWorld } from "@/lib/mairon/world-state";

const METRIC_ROWS = [
  ["sharpeRatio", "Sharpe (annualized)"],
  ["sortinoRatio", "Sortino"],
  ["calmarRatio", "Calmar"],
  ["omegaRatio", "Omega"],
  ["tailRatio", "Tail ratio"],
  ["profitFactor", "Profit factor"],
  ["winRatePct", "Win rate %"],
  ["totalReturnPct", "Total return %"],
  ["cagrPct", "CAGR %"],
  ["maxDrawdownPct", "Max drawdown %"],
  ["turnoverRatio", "Turnover"],
  ["skewness", "Skewness"],
  ["kurtosis", "Excess kurtosis"],
] as const;

export function PerspectiveQuant() {
  const { state, dispatch, sdk, telemetry, startJob, finishJob } = useMaironWorld();
  const quant = state.quant;
  const [prompt, setPrompt] = useState("Buy when close crosses above the 20-bar high; exit on the 10-bar low");
  const [symbol, setSymbol] = useState("BTC-USDT");
  const [timeframe, setTimeframe] = useState("1h");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runBacktest = useCallback(async () => {
    setBusy(true);
    setError(null);
    const jobId = startJob("backtest", "quant-desk", `${symbol} ${timeframe}`);
    try {
      const response = await sdk.quant.runCausalBacktest({
        vibePrompt: prompt,
        symbol,
        timeframe,
        bars: 720,
        seed: 2026,
      });
      dispatch({ type: "SET_QUANT_BACKTEST", backtest: response });
      telemetry(
        "quant-desk",
        response.gate.approved ? "info" : "warn",
        `Backtest ${response.strategyId}: Sharpe ${response.metrics.sharpeRatio.toFixed(2)}, gate ${response.gate.approved ? "approved" : "refused"}`,
      );
      finishJob(jobId, "succeeded");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("quant-desk", "error", message);
      finishJob(jobId, "failed", message);
    } finally {
      setBusy(false);
    }
  }, [dispatch, finishJob, prompt, sdk, symbol, startJob, telemetry, timeframe]);

  const verifyClaim = useCallback(async () => {
    const backtest = quant.lastBacktest;
    if (!backtest) return;
    const canonical = `${backtest.strategyId}|${backtest.metrics.totalTrades}|${backtest.metrics.finalEquity.toFixed(8)}|${backtest.metrics.bars}`;
    const result = await sdk.quant.verifyClaimHash(backtest.metrics.claimVerificationHash, canonical);
    dispatch({
      type: "SET_QUANT_CLAIM_VERIFICATION",
      verified: result.verified,
      recomputedSha256: result.recomputedSha256,
      atIso: new Date().toISOString(),
    });
    telemetry(
      "quant-desk",
      result.verified ? "info" : "warn",
      result.verified ? "Claim hash recomputed and matched" : "Claim hash mismatch — ledger claim differs from canonical string",
    );
  }, [dispatch, quant.lastBacktest, sdk, telemetry]);

  const metrics = quant.lastBacktest?.metrics;
  const sharpe = metrics?.sharpeRatio ?? null;
  const gatePassed = sharpe !== null && sharpe >= QUANT_LIVE_SHARPE_GATE;

  return (
    <section className="grid gap-4 lg:grid-cols-3">
      <div className="lg:col-span-2 grid gap-4">
        <div className="rounded-xl border border-amber-500/30 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Strategy AST builder
          </h3>
          <div className="mt-3 grid gap-2 md:grid-cols-3">
            <input
              aria-label="Symbol"
              value={symbol}
              onChange={(event) => setSymbol(event.target.value)}
              className="rounded-lg border border-zinc-700 bg-zinc-950/80 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-amber-500/60"
            />
            <input
              aria-label="Timeframe"
              value={timeframe}
              onChange={(event) => setTimeframe(event.target.value)}
              className="rounded-lg border border-zinc-700 bg-zinc-950/80 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-amber-500/60"
            />
            <button
              type="button"
              onClick={() => void runBacktest()}
              disabled={busy}
              className="rounded-lg bg-amber-500 px-4 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
            >
              {busy ? "Running causal DAG..." : "Run causal backtest"}
            </button>
          </div>
          <textarea
            aria-label="Vibe prompt"
            value={prompt}
            onChange={(event) => setPrompt(event.target.value)}
            rows={2}
            className="mt-2 w-full rounded-lg border border-zinc-700 bg-zinc-950/80 p-3 text-sm text-zinc-100 outline-none focus:border-amber-500/60"
          />
          <p className="mt-1 text-xs text-zinc-500">
            The backtester reads a strictly causal slice — <code>&amp;candles[0..=t]</code> — so no
            future bar can leak into any decision.
          </p>
          {error ? <p className="mt-2 text-xs text-red-400">{error}</p> : null}
        </div>

        {quant.lastBacktest ? (
          <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
                Backtest report — {quant.lastBacktest.strategyId}
              </h3>
              <span
                className={`rounded-full px-3 py-1 text-xs font-bold ${
                  gatePassed ? "bg-emerald-500 text-zinc-950" : "bg-red-500/80 text-zinc-100"
                }`}
              >
                {gatePassed
                  ? `Sharpe ${sharpe?.toFixed(2)} >= ${QUANT_LIVE_SHARPE_GATE.toFixed(2)} — live gate OPEN`
                  : `Sharpe ${sharpe?.toFixed(2)} < ${QUANT_LIVE_SHARPE_GATE.toFixed(2)} — live gate CLOSED`}
              </span>
            </div>
            <div className="mt-3 grid gap-1.5 md:grid-cols-2">
              {METRIC_ROWS.map(([key, label]) => {
                const value = (metrics as unknown as Record<string, number> | undefined)?.[key];
                return (
                  <div key={key} className="flex items-center justify-between rounded-md border border-zinc-800 px-3 py-1.5 text-xs">
                    <span className="text-zinc-400">{label}</span>
                    <span className="font-mono text-zinc-100">{typeof value === "number" ? value.toFixed(4) : "—"}</span>
                  </div>
                );
              })}
            </div>
            <div className="mt-3 rounded-lg border border-zinc-800 bg-zinc-950/80 p-3">
              <p className="text-xs text-zinc-400">DAG telemetry (topological order)</p>
              <ol className="mt-1 space-y-1 text-xs text-zinc-300">
                {quant.lastBacktest.nodes.map((node) => (
                  <li key={node.index} className="flex justify-between gap-2">
                    <span>
                      {node.index}. {node.name} — {node.status}
                    </span>
                    <span className="text-zinc-500">{node.deterministicTicks} ticks</span>
                  </li>
                ))}
              </ol>
            </div>
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <code className="rounded bg-zinc-950 px-2 py-1 text-xs text-amber-300">
                claim {quant.lastBacktest.metrics.claimVerificationHash.slice(0, 24)}...
              </code>
              <button
                type="button"
                onClick={() => void verifyClaim()}
                className="rounded-lg border border-amber-500/60 px-3 py-1.5 text-xs font-semibold text-amber-300"
              >
                Verify SHA-256 claim
              </button>
              {quant.claimVerification ? (
                <span className={`text-xs ${quant.claimVerification.verified ? "text-emerald-400" : "text-red-400"}`}>
                  {quant.claimVerification.verified ? "Verified match" : "Mismatch"} · {quant.claimVerification.recomputedSha256.slice(0, 16)}...
                </span>
              ) : null}
            </div>
          </div>
        ) : (
          <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4 text-xs text-zinc-500">
            No backtest in this session yet — the desk keeps the last report alive while you switch
            perspectives.
          </div>
        )}
      </div>

      <aside className="grid gap-4">
        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Keyless venue matrix
          </h3>
          {quant.venues ? (
            <ul className="mt-2 space-y-1.5 text-xs">
              {quant.venues.venues.map((venue) => (
                <li key={venue.id} className="flex items-center justify-between rounded-md border border-zinc-800 px-2 py-1.5">
                  <span className="text-zinc-200">{venue.displayName}</span>
                  <span className="text-zinc-500">{venue.venueKind} · auth {venue.auth}</span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="mt-2 text-xs text-zinc-500">Sync the world state to load the 8-venue matrix.</p>
          )}
        </div>
        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4 text-xs text-zinc-400">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">Desk posture</h3>
          <p className="mt-2">Posture: {state.postures["quant-desk"].posture}</p>
          <p>Report held: {quant.lastBacktest ? quant.lastBacktest.strategyId : "none"}</p>
        </div>
      </aside>
    </section>
  );
}
