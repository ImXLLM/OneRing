"use client";

/**
 * Perspective 1 — Cognitive Super-Studio.
 *
 * Live Trinity 3-avatar pipeline (Alpha IQ 136 -> CommandCenter IQ 139 ->
 * Mairon IQ 150), the `REA >= 85` quality gate through the supremacy
 * verification loop, the nine zero-cost modalities, the surgical diff
 * inspector entry, and the dual Ocean/Cosmos WebAudio synthesizer.
 */

import { useCallback, useRef, useState } from "react";

import {
  STUDIO_MODALITIES,
  TRINITY_AVATAR_LADDER,
  type StudioModalityIdT,
} from "@kian/contracts";
import type { UxMorphModeT } from "@kian/contracts";
import { useMaironWorld } from "@/lib/mairon/world-state";

const MORPH_MODES: readonly UxMorphModeT[] = ["ToySpark", "CreatorStudio", "ProTerminal"];

/** Dual-tone synthesizer — Ocean (low tritone drift) and Cosmos (shimmer fifth). */
function useDualSynthesizer() {
  const ctxRef = useRef<AudioContext | null>(null);
  const stop = useCallback(() => {
    void ctxRef.current?.close();
    ctxRef.current = null;
  }, []);
  const play = useCallback((kind: "ocean" | "cosmos") => {
    stop();
    const AudioCtor =
      window.AudioContext ??
      (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AudioCtor) return;
    const ctx = new AudioCtor();
    ctxRef.current = ctx;
    const master = ctx.createGain();
    master.gain.value = 0.18;
    // Safety ceiling — mirrors the kernel's `alimiter=limit=0.95` discipline.
    const limiter = ctx.createDynamicsCompressor();
    limiter.threshold.value = -6;
    limiter.ratio.value = 12;
    master.connect(limiter).connect(ctx.destination);
    const voices =
      kind === "ocean"
        ? [
            { freq: 55, type: "sine" as OscillatorType, drift: 0.8 },
            { freq: 82.5, type: "sine" as OscillatorType, drift: 1.1 },
          ]
        : [
            { freq: 220, type: "triangle" as OscillatorType, drift: 0.4 },
            { freq: 330, type: "sine" as OscillatorType, drift: 0.6 },
            { freq: 660, type: "sine" as OscillatorType, drift: 0.9 },
          ];
    for (const voice of voices) {
      const osc = ctx.createOscillator();
      osc.type = voice.type;
      osc.frequency.value = voice.freq;
      const lfo = ctx.createOscillator();
      lfo.frequency.value = voice.drift;
      const lfoGain = ctx.createGain();
      lfoGain.gain.value = voice.freq * 0.01;
      lfo.connect(lfoGain).connect(osc.frequency);
      const gain = ctx.createGain();
      gain.gain.setValueAtTime(0.0001, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.5, ctx.currentTime + 0.4);
      gain.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + 2.4);
      osc.connect(gain).connect(master);
      osc.start();
      lfo.start();
      osc.stop(ctx.currentTime + 2.6);
      lfo.stop(ctx.currentTime + 2.6);
    }
  }, [stop]);
  return { play, stop };
}

export function PerspectiveSuperStudio() {
  const { state, dispatch, sdk, telemetry, startJob, finishJob } = useMaironWorld();
  const studio = state.superStudio;
  const synth = useDualSynthesizer();
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const runModality = useCallback(async () => {
    setBusy("generate");
    setError(null);
    const jobId = startJob("trinity-build", "super-studio", `modality=${studio.selectedModality}`);
    try {
      const response = await sdk.superStudio.executeModality({
        prompt: studio.draftPrompt.length > 0 ? studio.draftPrompt : "Sovereign demo artifact",
        modality: studio.selectedModality as StudioModalityIdT,
        cognitiveTier: studio.morphMode,
      });
      dispatch({ type: "SET_STUDIO_GENERATION", generation: response as Record<string, unknown> });
      telemetry("super-studio", "info", `Modality "${studio.selectedModality}" generated an artifact`);
      finishJob(jobId, "succeeded");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("super-studio", "error", message);
      finishJob(jobId, "failed", message);
    } finally {
      setBusy(null);
    }
  }, [dispatch, finishJob, sdk, startJob, studio.draftPrompt, studio.morphMode, studio.selectedModality, telemetry]);

  const verifySeal = useCallback(async () => {
    setBusy("seal");
    setError(null);
    const jobId = startJob("sandbox-patch", "super-studio", "REA seal verification");
    try {
      const artifact =
        studio.lastGeneration && typeof studio.lastGeneration.artifactHtml === "string"
          ? studio.lastGeneration.artifactHtml
          : "<!doctype html><html><body><h1>MAIRON seal probe</h1></body></html>";
      const response = await sdk.supremacy.verifySeal({
        planSummary: "Super-Studio artifact verification",
        language: "html",
        artifact,
      });
      dispatch({ type: "SET_STUDIO_SEAL", seal: response });
      telemetry(
        "super-studio",
        response.verified ? "info" : "warn",
        response.seal
          ? `MaironVerifiedSeal minted: ${response.seal.sealId} (REA ${response.reaScore})`
          : `Verification finished without a seal (REA ${response.reaScore}, gate 85)`,
      );
      finishJob(jobId, "succeeded");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("super-studio", "error", message);
      finishJob(jobId, "failed", message);
    } finally {
      setBusy(null);
    }
  }, [dispatch, finishJob, sdk, startJob, studio.lastGeneration, telemetry]);

  return (
    <section className="grid gap-4 lg:grid-cols-3">
      <div className="lg:col-span-2 grid gap-4">
        <div className="rounded-xl border border-amber-500/30 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Trinity pipeline
          </h3>
          <div className="mt-3 grid gap-2 md:grid-cols-3">
            {TRINITY_AVATAR_LADDER.map((avatar, index) => (
              <div key={avatar.id} className="rounded-lg border border-zinc-700/60 bg-zinc-950/70 p-3">
                <p className="text-xs text-zinc-400">Stage {index + 1}</p>
                <p className="text-lg font-semibold text-zinc-100">{avatar.name}</p>
                <p className="text-xs text-amber-300">IQ {avatar.iq}</p>
                <p className="mt-1 text-xs text-zinc-500">{avatar.stage}</p>
              </div>
            ))}
          </div>
        </div>

        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-xs uppercase tracking-widest text-zinc-400">Morph mode</span>
            {MORPH_MODES.map((mode) => (
              <button
                key={mode}
                type="button"
                onClick={() => dispatch({ type: "SET_MORPH_MODE", mode })}
                className={`rounded-full px-3 py-1 text-xs font-medium transition ${
                  studio.morphMode === mode
                    ? "bg-amber-500 text-zinc-950"
                    : "border border-zinc-600 text-zinc-300 hover:border-amber-500/60"
                }`}
              >
                {mode}
                {mode === "ProTerminal" ? " (Ctrl+K)" : ""}
              </button>
            ))}
          </div>
          <div className="mt-3 flex flex-wrap gap-2">
            {STUDIO_MODALITIES.map((modality) => (
              <button
                key={modality}
                type="button"
                onClick={() => dispatch({ type: "SET_STUDIO_MODALITY", modality })}
                className={`rounded-md px-3 py-1.5 text-xs font-medium transition ${
                  studio.selectedModality === modality
                    ? "bg-zinc-100 text-zinc-900"
                    : "border border-zinc-700 text-zinc-300 hover:border-amber-500/60"
                }`}
              >
                {modality}
              </button>
            ))}
          </div>
          <textarea
            aria-label="Studio generation prompt"
            value={studio.draftPrompt}
            onChange={(event) => dispatch({ type: "SET_STUDIO_DRAFT", prompt: event.target.value })}
            rows={3}
            className="mt-3 w-full rounded-lg border border-zinc-700 bg-zinc-950/80 p-3 text-sm text-zinc-100 outline-none focus:border-amber-500/60"
          />
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={() => void runModality()}
              disabled={busy !== null}
              className="rounded-lg bg-amber-500 px-4 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
            >
              {busy === "generate" ? "Forging..." : "Execute modality"}
            </button>
            <button
              type="button"
              onClick={() => void verifySeal()}
              disabled={busy !== null}
              className="rounded-lg border border-amber-500/60 px-4 py-2 text-sm font-semibold text-amber-300 disabled:opacity-50"
            >
              {busy === "seal" ? "Verifying..." : "REA seal gate (>= 85)"}
            </button>
            <button
              type="button"
              onClick={() => synth.play("ocean")}
              className="rounded-lg border border-sky-500/50 px-3 py-2 text-xs font-medium text-sky-300"
            >
              Play Ocean tone
            </button>
            <button
              type="button"
              onClick={() => synth.play("cosmos")}
              className="rounded-lg border border-fuchsia-500/50 px-3 py-2 text-xs font-medium text-fuchsia-300"
            >
              Play Cosmos tone
            </button>
            <button
              type="button"
              onClick={synth.stop}
              className="rounded-lg border border-zinc-600 px-3 py-2 text-xs font-medium text-zinc-300"
            >
              Stop audio
            </button>
          </div>
          {error ? <p className="mt-2 text-xs text-red-400">{error}</p> : null}
        </div>

        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Surgical inspector
          </h3>
          {studio.lastGeneration ? (
            <pre className="mt-2 max-h-48 overflow-auto rounded-lg border border-zinc-800 bg-zinc-950/80 p-3 text-xs text-zinc-300">
              {JSON.stringify(studio.lastGeneration, null, 2)}
            </pre>
          ) : (
            <p className="mt-2 text-xs text-zinc-500">
              Run a modality to inspect its artifact JSON here — drafts persist across perspective switches.
            </p>
          )}
          {studio.lastSeal ? (
            <div className="mt-3 rounded-lg border border-emerald-500/40 bg-emerald-500/5 p-3 text-xs text-emerald-300">
              {studio.lastSeal.verified
                ? `Verified in ${studio.lastSeal.attemptsUsed} attempt(s) — REA ${studio.lastSeal.reaScore}`
                : `Gates failed — REA ${studio.lastSeal.reaScore} below 85`}{" "}
              · artifact sha256 {studio.lastSeal.artifactSha256.slice(0, 16)}...
            </div>
          ) : null}
        </div>
      </div>

      <aside className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
        <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
          Live posture
        </h3>
        <p className="mt-2 text-xs text-zinc-400">
          Profiles: {studio.profiles ? `${studio.profiles.tiers.length} tiers / ${studio.profiles.modalities.length} modalities` : "not synced"}
        </p>
        <p className="text-xs text-zinc-400">Draft kept: {studio.draftPrompt.length} chars</p>
        <p className="text-xs text-zinc-400">
          Sync posture: {state.postures["super-studio"].posture}
        </p>
        <ul className="mt-3 space-y-2 text-xs text-zinc-400">
          {state.jobs
            .filter((job) => job.perspective === "super-studio")
            .slice(-4)
            .reverse()
            .map((job) => (
              <li key={job.jobId} className="rounded-md border border-zinc-800 px-2 py-1.5">
                <span className="text-zinc-300">{job.kind}</span> · {job.status}
              </li>
            ))}
        </ul>
      </aside>
    </section>
  );
}
