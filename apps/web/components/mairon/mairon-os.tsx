"use client";

/**
 * MAIRON OS shell — the sovereign Web OS chrome.
 *
 * IMPERIUM law 2 in the UI: the five perspectives render from one world-state
 * store, so switching is a pointer move, never a reset. Drafts, background
 * jobs, telemetry, the war-room clock, and every subsystem payload survive
 * each switch. Ctrl+K (ProTerminal command palette) is available everywhere.
 */

import { useCallback, useEffect, useMemo, useState } from "react";

import { MAIRON_PERSPECTIVES } from "@kian/contracts";
import type { MaironPerspectiveT } from "@kian/contracts";
import { useMaironWorld } from "@/lib/mairon/world-state";
import { PerspectiveSuperStudio } from "@/components/mairon/perspective-super-studio";
import { PerspectiveQuant } from "@/components/mairon/perspective-quant";
import { PerspectiveTreasury } from "@/components/mairon/perspective-treasury";
import { PerspectiveNeuralHud } from "@/components/mairon/perspective-neural-hud";
import { PerspectiveWarRoom } from "@/components/mairon/perspective-war-room";

const PERSPECTIVE_TITLES: Record<MaironPerspectiveT, string> = {
  "super-studio": "Perspective 1 — Cognitive Super-Studio",
  "quant-desk": "Perspective 2 — Quantitative Execution Desk",
  "treasury": "Perspective 3 — Base-8453 Treasury & Genome Router",
  "neural-hud": "Perspective 4 — 3D Neural-HUD & Grok Lab",
  "war-room": "Perspective 5 — Founder War-Room Command Bridge",
};

const POSTURE_STYLES: Record<string, string> = {
  live: "bg-emerald-500/15 text-emerald-300 border-emerald-500/40",
  stale: "bg-zinc-500/10 text-zinc-400 border-zinc-600/60",
  degraded: "bg-red-500/10 text-red-300 border-red-500/40",
};

function postureLabel(perspective: MaironPerspectiveT, posture: string): string {
  if (posture === "live") return "live";
  if (posture === "degraded") return "degraded";
  return perspective === "war-room" ? "warming" : "standby";
}

export function MaironOs() {
  const { state, dispatch, syncWorldState, telemetry } = useMaironWorld();
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [paletteQuery, setPaletteQuery] = useState("");

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen((open) => !open);
        return;
      }
      if (event.key === "Escape") setPaletteOpen(false);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const commands = useMemo(() => {
    const base: Array<{ id: string; label: string; run: () => void }> = MAIRON_PERSPECTIVES.map(
      (perspective) => ({
        id: `switch:${perspective}`,
        label: `Switch to ${PERSPECTIVE_TITLES[perspective]}`,
        run: () => dispatch({ type: "SWITCH_PERSPECTIVE", perspective }),
      }),
    );
    base.push(
      { id: "sync", label: "Sync world state (all subsystems)", run: () => void syncWorldState() },
      {
        id: "warroom:play",
        label: "War-Room: start autoplay clock",
        run: () => {
          dispatch({ type: "SWITCH_PERSPECTIVE", perspective: "war-room" });
          dispatch({ type: "SET_WARROOM_PLAYING", playing: true });
        },
      },
      {
        id: "warroom:pause",
        label: "War-Room: pause autoplay clock",
        run: () => dispatch({ type: "SET_WARROOM_PLAYING", playing: false }),
      },
      {
        id: "studio:morph",
        label: "Super-Studio: cycle morph mode",
        run: () => {
          const order = ["ToySpark", "CreatorStudio", "ProTerminal"] as const;
          const current = order.indexOf(state.superStudio.morphMode);
          const next = order[(current + 1) % order.length];
          dispatch({ type: "SET_MORPH_MODE", mode: next });
          dispatch({ type: "SWITCH_PERSPECTIVE", perspective: "super-studio" });
        },
      },
      {
        id: "telemetry:clear",
        label: "Clear telemetry stream",
        run: () => {
          dispatch({ type: "CLEAR_TELEMETRY" });
          telemetry("sdk", "info", "Telemetry stream cleared by operator");
        },
      },
    );
    return base;
  }, [dispatch, state.superStudio.morphMode, syncWorldState, telemetry]);

  const filteredCommands = commands.filter((command) =>
    command.label.toLowerCase().includes(paletteQuery.toLowerCase()),
  );

  const runCommand = useCallback((run: () => void) => {
    setPaletteOpen(false);
    setPaletteQuery("");
    run();
  }, []);

  const activeJobs = state.jobs.filter((job) => job.status === "running").length;
  const liveCount = Object.values(state.postures).filter((p) => p.posture === "live").length;

  return (
    <div className="flex min-h-screen flex-col bg-zinc-950 text-zinc-100">
      <header className="border-b border-amber-500/20 bg-zinc-950/90 px-4 py-3 md:px-6">
        <div className="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-3">
          <div className="flex items-baseline gap-3">
            <h1 className="text-xl font-black tracking-[0.35em] text-amber-400">MAIRON</h1>
            <p className="hidden text-xs text-zinc-400 sm:block">
              Sovereign Intelligence, Quantitative Execution &amp; Autonomous Capital OS
            </p>
          </div>
          <div className="flex items-center gap-2 text-xs">
            <span className="rounded-full border border-zinc-700 px-2.5 py-1 text-zinc-400">
              revision r{state.revision}
            </span>
            <span className="rounded-full border border-zinc-700 px-2.5 py-1 text-zinc-400">
              {liveCount}/5 live
            </span>
            <span className="rounded-full border border-zinc-700 px-2.5 py-1 text-zinc-400">
              {activeJobs} running jobs
            </span>
            <button
              type="button"
              onClick={() => void syncWorldState()}
              disabled={state.syncInProgress}
              className="rounded-lg bg-amber-500 px-3 py-1.5 font-semibold text-zinc-950 disabled:opacity-50"
            >
              {state.syncInProgress ? "Syncing..." : "Sync world state"}
            </button>
            <button
              type="button"
              onClick={() => setPaletteOpen(true)}
              className="rounded-lg border border-zinc-600 px-3 py-1.5 font-mono text-zinc-300"
            >
              Ctrl+K
            </button>
          </div>
        </div>
      </header>

      <nav className="border-b border-zinc-800 bg-zinc-950/95 px-4 py-2 md:px-6">
        <div className="mx-auto flex max-w-7xl flex-wrap gap-2">
          {MAIRON_PERSPECTIVES.map((perspective, index) => (
            <button
              key={perspective}
              type="button"
              onClick={() => dispatch({ type: "SWITCH_PERSPECTIVE", perspective })}
              className={`rounded-lg px-3 py-1.5 text-xs font-semibold transition ${
                state.perspective === perspective
                  ? "bg-amber-500 text-zinc-950"
                  : "border border-zinc-700 text-zinc-300 hover:border-amber-500/60"
              }`}
            >
              <span className="mr-1.5 text-[10px] opacity-70">P{index + 1}</span>
              {PERSPECTIVE_TITLES[perspective].split("— ")[1]}
              <span
                className={`ml-2 inline-block h-1.5 w-1.5 rounded-full ${
                  state.postures[perspective].posture === "live"
                    ? "bg-emerald-400"
                    : state.postures[perspective].posture === "degraded"
                      ? "bg-red-400"
                      : "bg-zinc-500"
                }`}
              />
            </button>
          ))}
        </div>
      </nav>

      <main className="mx-auto w-full max-w-7xl flex-1 px-4 py-5 md:px-6">
        <div className="mb-4 flex flex-wrap items-center justify-between gap-2">
          <h2 className="text-lg font-bold text-zinc-100">{PERSPECTIVE_TITLES[state.perspective]}</h2>
          <div className="flex flex-wrap gap-1.5">
            {MAIRON_PERSPECTIVES.map((perspective) => (
              <span
                key={perspective}
                className={`rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-wide ${
                  POSTURE_STYLES[state.postures[perspective].posture] ?? POSTURE_STYLES["stale"]
                }`}
              >
                {perspective}: {postureLabel(perspective, state.postures[perspective].posture)}
              </span>
            ))}
          </div>
        </div>

        {/* Perspectives mount by pointer; their state lives in the store, so a
            switch never unmounts an active job, feed, or draft. */}
        {state.perspective === "super-studio" ? <PerspectiveSuperStudio /> : null}
        {state.perspective === "quant-desk" ? <PerspectiveQuant /> : null}
        {state.perspective === "treasury" ? <PerspectiveTreasury /> : null}
        {state.perspective === "neural-hud" ? <PerspectiveNeuralHud /> : null}
        {state.perspective === "war-room" ? <PerspectiveWarRoom /> : null}
      </main>

      <footer className="border-t border-zinc-800 bg-zinc-950/95 px-4 py-2 text-[11px] text-zinc-500 md:px-6">
        <div className="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-2">
          <span>
            MAIRON OS · last sync {state.lastSyncAtIso ?? "never"} · telemetry {state.telemetry.length} events ·
            jobs {state.jobs.length}
          </span>
          <span className="font-mono">
            {state.jobs.filter((job) => job.status === "running").length > 0
              ? `running: ${state.jobs
                  .filter((job) => job.status === "running")
                  .map((job) => job.kind)
                  .join(", ")}`
              : "no background jobs"}
          </span>
        </div>
      </footer>

      {paletteOpen ? (
        <div
          className="fixed inset-0 z-50 flex items-start justify-center bg-zinc-950/80 pt-24"
          role="dialog"
          aria-label="Command palette"
          onClick={() => setPaletteOpen(false)}
        >
          <div
            className="w-full max-w-xl rounded-xl border border-amber-500/40 bg-zinc-900 shadow-2xl"
            onClick={(event) => event.stopPropagation()}
          >
            <input
              autoFocus
              value={paletteQuery}
              onChange={(event) => setPaletteQuery(event.target.value)}
              aria-label="Command palette query"
              className="w-full rounded-t-xl border-b border-zinc-700 bg-zinc-950/80 px-4 py-3 text-sm text-zinc-100 outline-none"
            />
            <ul className="max-h-80 overflow-auto p-2">
              {filteredCommands.map((command) => (
                <li key={command.id}>
                  <button
                    type="button"
                    onClick={() => runCommand(command.run)}
                    className="w-full rounded-lg px-3 py-2 text-left text-sm text-zinc-200 hover:bg-zinc-800"
                  >
                    {command.label}
                  </button>
                </li>
              ))}
              {filteredCommands.length === 0 ? (
                <li className="px-3 py-2 text-xs text-zinc-500">No command matches.</li>
              ) : null}
            </ul>
          </div>
        </div>
      ) : null}
    </div>
  );
}
