"use client";

/**
 * Perspective 5 — Founder War-Room Command Bridge.
 *
 * Direct live synchronization with `/api/v1/war-room/{state,step,directive,
 * reset,explain-simple}`: the 24 hourly causal tick feed, the co-founder chat
 * cast (`@elonmusk`, `@cz_binance`, `@JeffBezos`), the chairman directive
 * console, the honest `+$9.55 -> $42.55` wallet ledger, the ELI5 pipeline
 * cards, and the standalone `/war-room` dashboard link. Autoplay is owned by
 * the world-state provider, so the clock keeps running across perspective
 * switches.
 */

import { useCallback, useEffect, useState } from "react";

import { useMaironWorld } from "@/lib/mairon/world-state";

const microsToUsd = (micros: number) => `$${(micros / 1_000_000).toFixed(2)}`;

export function PerspectiveWarRoom() {
  const { state, dispatch, sdk, telemetry, startJob, finishJob } = useMaironWorld();
  const warRoom = state.warRoom;
  const [directive, setDirective] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const snapshot = warRoom.snapshot;

  useEffect(() => {
    if (snapshot || state.syncInProgress) return;
    void sdk.warRoom
      .getState()
      .then((fresh) => {
        dispatch({ type: "SET_WARROOM_SNAPSHOT", snapshot: fresh });
        dispatch({ type: "APPEND_WARROOM_TICKS", ticks: fresh.scenario.slice(0, fresh.tickCursor) });
      })
      .catch((err: unknown) => {
        const message = err instanceof Error ? err.message : String(err);
        telemetry("war-room", "warn", `Initial state fetch deferred: ${message}`);
      });
  }, [dispatch, sdk, snapshot, state.syncInProgress, telemetry]);

  const loadExplain = useCallback(async () => {
    setBusy("explain");
    setError(null);
    try {
      const response = await sdk.warRoom.explainSimple();
      dispatch({ type: "SET_WARROOM_EXPLAIN", explain: response as Record<string, unknown> });
      telemetry("war-room", "info", "ELI5 pipeline cards loaded");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("war-room", "error", message);
    } finally {
      setBusy(null);
    }
  }, [dispatch, sdk, telemetry]);

  const sendDirective = useCallback(async () => {
    if (directive.trim().length === 0) return;
    setBusy("directive");
    setError(null);
    const jobId = startJob("war-room-autoplay", "war-room", "chairman directive");
    try {
      const response = await sdk.warRoom.sendDirective({
        directiveText: directive.trim(),
        locale: "en",
        triggeredTick: snapshot?.tickCursor ?? 0,
      });
      dispatch({ type: "APPEND_WARROOM_DIRECTIVE", directive: response });
      setDirective("");
      telemetry("war-room", "info", `Directive intent "${response.intentId}" — ${response.replies.length} in-persona replies`);
      finishJob(jobId, "succeeded");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("war-room", "error", message);
      finishJob(jobId, "failed", message);
    } finally {
      setBusy(null);
    }
  }, [directive, dispatch, finishJob, sdk, snapshot?.tickCursor, startJob, telemetry]);

  const resetSimulation = useCallback(async () => {
    setBusy("reset");
    setError(null);
    try {
      const response = await sdk.warRoom.reset();
      const fresh = await sdk.warRoom.getState();
      dispatch({ type: "SET_WARROOM_SNAPSHOT", snapshot: fresh });
      dispatch({ type: "SET_WARROOM_PLAYING", playing: false });
      dispatch({ type: "APPEND_WARROOM_TICKS", ticks: [] });
      telemetry(
        "war-room",
        "info",
        `Causal reset: ${response.deletedTicks} ticks and ${response.deletedDirectives} directives cleared`,
      );
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("war-room", "error", message);
    } finally {
      setBusy(null);
    }
  }, [dispatch, sdk, telemetry]);

  const stepOnce = useCallback(async () => {
    if (!snapshot) return;
    setBusy("step");
    setError(null);
    try {
      const response = await sdk.warRoom.stepTick({
        tickCursor: Math.min(snapshot.tickCursor, 24),
        speedMultiplier: warRoom.speed,
      });
      dispatch({ type: "APPEND_WARROOM_TICKS", ticks: [response.executed] });
      dispatch({
        type: "SET_WARROOM_SNAPSHOT",
        snapshot: {
          ...snapshot,
          tickCursor: response.tickCursor,
          progressPercent: response.progressPercent,
          elapsedSeconds: response.elapsedSeconds,
          wallet: response.wallet,
        },
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("war-room", "error", message);
    } finally {
      setBusy(null);
    }
  }, [dispatch, sdk, snapshot, telemetry, warRoom.speed]);

  return (
    <section className="grid gap-4 lg:grid-cols-3">
      <div className="lg:col-span-2 grid gap-4">
        <div className="rounded-xl border border-amber-500/30 bg-zinc-900/60 p-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
              Causal simulation — Day 0 to Day 1
            </h3>
            <div className="flex items-center gap-2 text-xs">
              <span className="text-zinc-400">
                tick {snapshot?.tickCursor ?? 0}/24 · {snapshot?.progressPercent ?? 0}%
              </span>
              {([1, 2, 5] as const).map((speed) => (
                <button
                  key={speed}
                  type="button"
                  onClick={() => dispatch({ type: "SET_WARROOM_SPEED", speed })}
                  className={`rounded px-2 py-0.5 font-mono ${
                    warRoom.speed === speed ? "bg-amber-500 text-zinc-950" : "border border-zinc-700 text-zinc-300"
                  }`}
                >
                  x{speed}
                </button>
              ))}
              <button
                type="button"
                onClick={() => dispatch({ type: "SET_WARROOM_PLAYING", playing: !warRoom.playing })}
                className={`rounded px-3 py-1 font-semibold ${
                  warRoom.playing ? "bg-red-500/80 text-zinc-100" : "bg-emerald-500 text-zinc-950"
                }`}
              >
                {warRoom.playing ? "Pause clock" : "Start autoplay"}
              </button>
              <button
                type="button"
                onClick={() => void stepOnce()}
                disabled={busy !== null || !snapshot}
                className="rounded border border-zinc-600 px-3 py-1 text-zinc-200 disabled:opacity-50"
              >
                {busy === "step" ? "Stepping..." : "Single step"}
              </button>
            </div>
          </div>
          <div className="mt-3 h-2 w-full overflow-hidden rounded-full bg-zinc-800">
            <div
              className="h-full rounded-full bg-amber-500 transition-all"
              style={{ width: `${snapshot?.progressPercent ?? 0}%` }}
            />
          </div>
          {error ? <p className="mt-2 text-xs text-red-400">{error}</p> : null}
        </div>

        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Organization wallet ledger
          </h3>
          {snapshot ? (
            <div className="mt-2 grid gap-1.5 text-xs md:grid-cols-2">
              <p className="flex justify-between rounded-md border border-zinc-800 px-3 py-1.5">
                <span className="text-zinc-400">Crypto seed</span>
                <span className="font-mono text-zinc-100">{snapshot.wallet.cryptoSeedUsd}</span>
              </p>
              <p className="flex justify-between rounded-md border border-zinc-800 px-3 py-1.5">
                <span className="text-zinc-400">Cash seed</span>
                <span className="font-mono text-zinc-100">{snapshot.wallet.cashSeedUsd}</span>
              </p>
              <p className="flex justify-between rounded-md border border-zinc-800 px-3 py-1.5">
                <span className="text-zinc-400">Locked cold reserve</span>
                <span className="font-mono text-zinc-100">{snapshot.wallet.paperReserveUsd}</span>
              </p>
              <p className="flex justify-between rounded-md border border-emerald-500/40 px-3 py-1.5">
                <span className="text-zinc-400">Earned revenue</span>
                <span className="font-mono text-emerald-300">
                  {snapshot.wallet.earnedRevenueUsd} · ledger total{" "}
                  {microsToUsd(snapshot.wallet.cryptoSeedMicros + snapshot.wallet.earnedRevenueMicros)}
                </span>
              </p>
              <p className="text-zinc-500 md:col-span-2">
                Funnel: {snapshot.wallet.funnelVisits} visits -> {snapshot.wallet.funnelSignups} signups ->{" "}
                {snapshot.wallet.funnelPurchases} purchases · {snapshot.wallet.activeUsers} active users
              </p>
            </div>
          ) : (
            <p className="mt-2 text-xs text-zinc-500">State not loaded yet.</p>
          )}
        </div>

        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Co-founder chat feed
          </h3>
          <div className="mt-2 max-h-72 space-y-2 overflow-auto">
            {warRoom.tickFeed.map((tick) => (
              <div key={tick.tickIndex} className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-2.5 text-xs">
                <p className="flex items-center justify-between">
                  <span className="font-semibold" style={{ color: tick.speakerAccentHex }}>
                    t{tick.tickIndex} · {tick.speakerNameEn}
                  </span>
                  <span className="text-zinc-500">hour {tick.simulationSecond}s</span>
                </p>
                <p className="mt-1 text-zinc-200">{tick.messageEn}</p>
                <p className="mt-1 flex flex-wrap gap-2 text-[11px] text-zinc-500">
                  <span>stage {tick.activePipelineStage}</span>
                  <span>stream {tick.revenueStream}</span>
                  {tick.walletDeltaUsdcMicros !== 0 ? (
                    <span className="text-emerald-400">
                      wallet {tick.walletDeltaUsdcMicros > 0 ? "+" : ""}
                      {microsToUsd(tick.walletDeltaUsdcMicros)} -> {microsToUsd(tick.walletTotalUsdcMicros)}
                    </span>
                  ) : null}
                  {tick.terminalCmd ? <code className="text-amber-300">{tick.terminalCmd}</code> : null}
                </p>
              </div>
            ))}
            {warRoom.tickFeed.length === 0 ? (
              <p className="text-xs text-zinc-500">Feed empty — start autoplay or step once.</p>
            ) : null}
            {warRoom.directiveLog.map((entry, index) => (
              <div key={`directive-${index}`} className="rounded-lg border border-sky-500/40 bg-sky-500/5 p-2.5 text-xs">
                <p className="font-semibold text-sky-300">Chairman directive · {entry.intentLabelEn}</p>
                <p className="mt-1 text-zinc-200">{entry.directiveText}</p>
                <ul className="mt-1 space-y-1">
                  {entry.replies.map((reply) => (
                    <li key={reply.speakerId}>
                      <span className="font-medium" style={{ color: reply.accentHex }}>
                        {reply.speakerNameEn}:
                      </span>{" "}
                      <span className="text-zinc-300">{reply.messageEn}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
      </div>

      <aside className="grid gap-4">
        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Chairman directive console
          </h3>
          <textarea
            aria-label="Chairman directive"
            value={directive}
            onChange={(event) => setDirective(event.target.value)}
            rows={3}
            className="mt-2 w-full rounded-lg border border-zinc-700 bg-zinc-950/80 p-3 text-sm text-zinc-100 outline-none focus:border-amber-500/60"
          />
          <div className="mt-2 flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => void sendDirective()}
              disabled={busy !== null || directive.trim().length === 0}
              className="rounded-lg bg-amber-500 px-4 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
            >
              {busy === "directive" ? "Broadcasting..." : "Send directive"}
            </button>
            <button
              type="button"
              onClick={() => void resetSimulation()}
              disabled={busy !== null}
              className="rounded-lg border border-red-500/60 px-4 py-2 text-sm font-semibold text-red-300 disabled:opacity-50"
            >
              {busy === "reset" ? "Resetting..." : "Reset to Day 0"}
            </button>
          </div>
          <a
            href={sdk.warRoom.dashboardUrl()}
            target="_blank"
            rel="noreferrer"
            className="mt-2 inline-block rounded-lg border border-amber-500/60 px-3 py-1.5 text-xs font-semibold text-amber-300"
          >
            Open standalone /war-room dashboard
          </a>
        </div>

        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            ELI5 pipeline cards
          </h3>
          <button
            type="button"
            onClick={() => void loadExplain()}
            disabled={busy !== null}
            className="mt-2 rounded-lg border border-zinc-600 px-3 py-1.5 text-xs font-semibold text-zinc-200 disabled:opacity-50"
          >
            {busy === "explain" ? "Loading..." : "Load explain-simple"}
          </button>
          {warRoom.explain ? (
            <div className="mt-2 space-y-2">
              <p className="text-xs font-semibold text-zinc-100">
                {(warRoom.explain as { headlineEn?: string }).headlineEn ?? ""}
              </p>
              <p className="text-[11px] text-zinc-400">
                {(warRoom.explain as { charterEn?: string }).charterEn ?? ""}
              </p>
              {(
                warRoom.explain as { stepIds?: string[]; stepTitlesEn?: string[]; stepBodiesEn?: string[] }
              ).stepIds?.map((stepId, index) => (
                <div key={stepId} className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-2 text-[11px]">
                  <p className="font-semibold text-amber-300">
                    {index + 1}. {(warRoom.explain as { stepTitlesEn?: string[] }).stepTitlesEn?.[index] ?? stepId}
                  </p>
                  <p className="mt-0.5 text-zinc-300">
                    {(warRoom.explain as { stepBodiesEn?: string[] }).stepBodiesEn?.[index] ?? ""}
                  </p>
                </div>
              ))}
            </div>
          ) : null}
        </div>
      </aside>
    </section>
  );
}
