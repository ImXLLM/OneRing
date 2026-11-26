"use client";

/**
 * Perspective 3 — Base-8453 Treasury & 45-Model Genome Router.
 *
 * The live four-stream revenue calculator (charter bps from the kernel), the
 * Base USDC `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` + Telegram Stars XTR
 * settlement feed, the capital reserve partition, and the 5 x 9 = 45
 * Omni-Model Genome switcher in both `ManualOverride` and
 * `AgentAutoDecision` modes.
 */

import { useCallback, useState } from "react";

import {
  BASE_CHAIN_ID,
  BASE_USDC_CONTRACT,
  GENOME_CATEGORIES,
  KIA_TOKEN_SYMBOL,
  TELEGRAM_STARS_CURRENCY,
} from "@kian/contracts";
import { useMaironWorld } from "@/lib/mairon/world-state";

const microsToUsd = (micros: number) => `$${(micros / 1_000_000).toFixed(2)}`;

export function PerspectiveTreasury() {
  const { state, dispatch, sdk, telemetry, startJob, finishJob } = useMaironWorld();
  const treasury = state.treasury;
  const [scenario, setScenario] = useState<"marketplace_sale" | "instant_boost">("marketplace_sale");
  const [asset, setAsset] = useState("telegram-stars-xtr");
  const [gross, setGross] = useState("2500000");
  const [referred, setReferred] = useState(false);
  const [overrideModel, setOverrideModel] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const runSplit = useCallback(async () => {
    setBusy("split");
    setError(null);
    const jobId = startJob("split-sim", "treasury", `${scenario} via ${asset}`);
    try {
      const response = await sdk.treasury.calculateSplit({
        scenario,
        settlementAsset: asset as "telegram-stars-xtr" | "base-native-eth" | "base-native-usdc" | "zero-x-kia-base-token" | "internal-micro-kian",
        grossAmountMicros: Number.parseInt(gross, 10) || 0,
        buyerUserId: "web-os-buyer",
        creatorUserId: "web-os-creator",
        referrerUserId: referred ? "web-os-affiliate" : null,
        affiliateBps: 1_000,
        boostTier: scenario === "instant_boost" ? "featured-slot-24h" : null,
        recipientAddress: null,
      });
      dispatch({ type: "SET_LAST_SPLIT", split: response });
      telemetry("treasury", "info", `Split ${response.splitId}: zero-sum proof ${response.zeroSumProof}`);
      finishJob(jobId, "succeeded");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("treasury", "error", message);
      finishJob(jobId, "failed", message);
    } finally {
      setBusy(null);
    }
  }, [asset, dispatch, finishJob, gross, referred, scenario, sdk, startJob, telemetry]);

  const routeModel = useCallback(async () => {
    setBusy("route");
    setError(null);
    const jobId = startJob("trinity-build", "treasury", "genome routing");
    try {
      const response = await sdk.treasury.setGenomeRouting({
        taskPrompt: overrideModel.length > 0 ? `Manual routing: ${overrideModel}` : "Autonomous routing from the Web OS",
        agentMode: overrideModel.trim().length === 0,
        manualModelOverride: overrideModel.trim().length > 0 ? overrideModel.trim() : null,
      });
      dispatch({ type: "SET_LAST_ROUTING", routing: response });
      telemetry(
        "treasury",
        "info",
        `Genome routed to ${response.selectedModel} (${response.selectionMode}, confidence ${response.confidenceScore.toFixed(2)})`,
      );
      finishJob(jobId, "succeeded");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("treasury", "error", message);
      finishJob(jobId, "failed", message);
    } finally {
      setBusy(null);
    }
  }, [dispatch, finishJob, overrideModel, sdk, startJob, telemetry]);

  const charter = treasury.charter;
  const genome = treasury.genome;

  return (
    <section className="grid gap-4 lg:grid-cols-3">
      <div className="lg:col-span-2 grid gap-4">
        <div className="rounded-xl border border-amber-500/30 bg-zinc-900/60 p-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
              Four-stream Day-1 charter
            </h3>
            <span className="text-xs text-zinc-400">
              {KIA_TOKEN_SYMBOL} · {TELEGRAM_STARS_CURRENCY} · chain {BASE_CHAIN_ID}
            </span>
          </div>
          {charter ? (
            <ul className="mt-3 grid gap-2 md:grid-cols-2">
              {charter.streams.map((stream) => (
                <li key={stream.stream} className="rounded-lg border border-zinc-700/60 bg-zinc-950/70 p-3 text-xs">
                  <p className="font-semibold text-zinc-100">
                    Stream {stream.stream} — {stream.name} <span className="text-amber-300">({stream.bps} bps)</span>
                  </p>
                  <p className="mt-1 text-zinc-400">{stream.note}</p>
                  <p className="mt-1 font-mono text-zinc-500">{stream.destinationAccount}</p>
                </li>
              ))}
            </ul>
          ) : (
            <p className="mt-2 text-xs text-zinc-500">Sync the world state to load the charter.</p>
          )}
          <p className="mt-2 font-mono text-xs text-zinc-500">USDC {BASE_USDC_CONTRACT}</p>
        </div>

        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Revenue split calculator
          </h3>
          <div className="mt-3 flex flex-wrap items-center gap-2 text-xs">
            <select
              aria-label="Scenario"
              value={scenario}
              onChange={(event) => setScenario(event.target.value as "marketplace_sale" | "instant_boost")}
              className="rounded-lg border border-zinc-700 bg-zinc-950/80 px-2 py-1.5 text-zinc-100"
            >
              <option value="marketplace_sale">marketplace_sale</option>
              <option value="instant_boost">instant_boost</option>
            </select>
            <select
              aria-label="Settlement asset"
              value={asset}
              onChange={(event) => setAsset(event.target.value)}
              className="rounded-lg border border-zinc-700 bg-zinc-950/80 px-2 py-1.5 text-zinc-100"
            >
              <option value="telegram-stars-xtr">telegram-stars-xtr</option>
              <option value="base-native-usdc">base-native-usdc</option>
              <option value="base-native-eth">base-native-eth</option>
              <option value="zero-x-kia-base-token">zero-x-kia-base-token</option>
              <option value="internal-micro-kian">internal-micro-kian</option>
            </select>
            <input
              aria-label="Gross micros"
              value={gross}
              onChange={(event) => setGross(event.target.value)}
              className="w-40 rounded-lg border border-zinc-700 bg-zinc-950/80 px-2 py-1.5 font-mono text-zinc-100"
            />
            <label className="flex items-center gap-1.5 text-zinc-300">
              <input type="checkbox" checked={referred} onChange={(event) => setReferred(event.target.checked)} />
              referred
            </label>
            <button
              type="button"
              onClick={() => void runSplit()}
              disabled={busy !== null}
              className="rounded-lg bg-amber-500 px-4 py-1.5 font-semibold text-zinc-950 disabled:opacity-50"
            >
              {busy === "split" ? "Settling..." : "Run atomic split"}
            </button>
          </div>
          {error ? <p className="mt-2 text-xs text-red-400">{error}</p> : null}
          {treasury.lastSplit ? (
            <div className="mt-3 rounded-lg border border-zinc-800 bg-zinc-950/80 p-3 text-xs">
              <p className="text-zinc-300">
                {treasury.lastSplit.scenario} · effective platform {treasury.lastSplit.effectivePlatformBps} bps ·
                treasury {microsToUsd(treasury.lastSplit.kiaTreasuryMicros)} · creator{" "}
                {microsToUsd(treasury.lastSplit.creatorNetMicros)} · referrer {microsToUsd(treasury.lastSplit.referrerMicros)}
              </p>
              <p className="mt-1 text-emerald-400">{treasury.lastSplit.zeroSumProof}</p>
              {treasury.lastSplit.eip681PaymentUri ? (
                <p className="mt-1 font-mono text-zinc-400">{treasury.lastSplit.eip681PaymentUri}</p>
              ) : null}
            </div>
          ) : null}
        </div>

        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
              Omni-Model Genome router — 5 x 9 = 45
            </h3>
            <span className="text-xs text-zinc-400">
              {genome ? `${genome.totalFilesVerified}/${genome.expectedFileCount} genome files verified` : "matrix not synced"}
            </span>
          </div>
          <div className="mt-3 grid gap-2 md:grid-cols-5">
            {GENOME_CATEGORIES.map((category) => (
              <div key={category} className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-2">
                <p className="text-[10px] uppercase tracking-wide text-zinc-500">{category}</p>
                <p className="mt-1 text-xs text-zinc-300">9 slots</p>
              </div>
            ))}
          </div>
          <div className="mt-3 flex flex-wrap items-center gap-2 text-xs">
            <input
              aria-label="Manual model override"
              value={overrideModel}
              onChange={(event) => setOverrideModel(event.target.value)}
              className="w-56 rounded-lg border border-zinc-700 bg-zinc-950/80 px-2 py-1.5 text-zinc-100"
            />
            <button
              type="button"
              onClick={() => void routeModel()}
              disabled={busy !== null}
              className="rounded-lg border border-amber-500/60 px-4 py-1.5 font-semibold text-amber-300 disabled:opacity-50"
            >
              {busy === "route" ? "Routing..." : "Route task"}
            </button>
            <span className="text-zinc-500">
              empty override = AgentAutoDecision · filled = ManualOverride
            </span>
          </div>
          {treasury.lastRouting ? (
            <pre className="mt-3 max-h-40 overflow-auto rounded-lg border border-zinc-800 bg-zinc-950/80 p-3 text-[11px] text-zinc-300">
              {JSON.stringify(
                {
                  selectedModel: treasury.lastRouting.selectedModel,
                  selectionMode: treasury.lastRouting.selectionMode,
                  routedDomain: treasury.lastRouting.routedDomain,
                  trinityAvatar: treasury.lastRouting.trinityAvatar,
                  filesVerified: treasury.lastRouting.filesVerified,
                },
                null,
                2,
              )}
            </pre>
          ) : null}
        </div>
      </div>

      <aside className="grid gap-4">
        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4 text-xs text-zinc-400">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Capital reserve partition
          </h3>
          <ul className="mt-2 space-y-1.5">
            <li className="flex justify-between"><span>Crypto seed (operational)</span><span className="font-mono text-zinc-100">$33.00</span></li>
            <li className="flex justify-between"><span>Cash seed</span><span className="font-mono text-zinc-100">$15.00</span></li>
            <li className="flex justify-between"><span>Locked cold reserve</span><span className="font-mono text-zinc-100">$350.00</span></li>
          </ul>
          <p className="mt-2 text-[11px] text-zinc-500">
            Earned revenue travels the four streams only — the reserve partition is untouchable by design.
          </p>
        </div>
        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4 text-xs text-zinc-400">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">Treasury posture</h3>
          <p className="mt-2">Posture: {state.postures["treasury"].posture}</p>
          <p>Genome: {genome ? `${genome.models.length} models registered` : "not loaded"}</p>
          <p>Selection modes: {genome ? genome.selectionModes.join(" · ") : "—"}</p>
        </div>
      </aside>
    </section>
  );
}
