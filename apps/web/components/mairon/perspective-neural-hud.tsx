"use client";

/**
 * Perspective 4 — 3D Neural-HUD & Grok 10-DNA Lab.
 *
 * An interactive 180-node dual-shell Fibonacci topology viewer (cortex shell
 * 120 + telemetry shell 60) rendered on canvas with the golden-angle spiral,
 * the HUD preset catalog with kernel-side synthesis, the four Grok voice
 * modes, and the 10-DNA telemetry inspector. The standalone `/neural-hud`
 * dashboard stays one click away.
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import {
  FIBONACCI_SHELL_INNER_NODES,
  FIBONACCI_SHELL_OUTER_NODES,
  FIBONACCI_SPHERE_TOTAL_NODES,
  GROK_VOICE_MODES,
  GROK_VOICE_MODE_WIRE_IDS,
} from "@kian/contracts";
import { useMaironWorld } from "@/lib/mairon/world-state";

type SphereNode = { x: number; y: number; z: number; shell: "cortex" | "telemetry"; index: number };

/** Deterministic golden-angle Fibonacci sphere — 120 cortex + 60 telemetry. */
function buildSphere(): SphereNode[] {
  const nodes: SphereNode[] = [];
  const goldenAngle = Math.PI * (3 - Math.sqrt(5));
  for (let i = 0; i < FIBONACCI_SPHERE_TOTAL_NODES; i += 1) {
    const t = i / FIBONACCI_SPHERE_TOTAL_NODES;
    const inclination = Math.acos(1 - 2 * t);
    const azimuth = goldenAngle * i;
    const shell: SphereNode["shell"] = i < FIBONACCI_SHELL_INNER_NODES ? "cortex" : "telemetry";
    nodes.push({
      x: Math.sin(inclination) * Math.cos(azimuth),
      y: Math.cos(inclination),
      z: Math.sin(inclination) * Math.sin(azimuth),
      shell,
      index: i,
    });
  }
  return nodes;
}

function NeuralTopologyCanvas() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const nodes = useMemo(buildSphere, []);
  const rotationRef = useRef(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    let raf = 0;
    const render = () => {
      rotationRef.current += 0.004;
      const rotation = rotationRef.current;
      const width = canvas.width;
      const height = canvas.height;
      const radius = Math.min(width, height) * 0.36;
      const centerX = width / 2;
      const centerY = height / 2;
      ctx.clearRect(0, 0, width, height);
      const projected = nodes.map((node) => {
        const cosR = Math.cos(rotation);
        const sinR = Math.sin(rotation);
        const x = node.x * cosR - node.z * sinR;
        const z = node.x * sinR + node.z * cosR;
        const depth = (z + 1) / 2;
        return {
          px: centerX + x * radius,
          py: centerY + node.y * radius,
          depth,
          shell: node.shell,
          index: node.index,
        };
      });
      // Sparse deterministic edges — each cortex node links to +7 and +23.
      ctx.strokeStyle = "rgba(245, 158, 11, 0.14)";
      for (const node of projected) {
        for (const offset of [7, 23]) {
          const target = projected[(node.index + offset) % projected.length];
          if (!target) continue;
          ctx.beginPath();
          ctx.moveTo(node.px, node.py);
          ctx.lineTo(target.px, target.py);
          ctx.stroke();
        }
      }
      for (const node of projected) {
        const size = node.shell === "cortex" ? 2.4 : 1.6;
        const alpha = 0.25 + node.depth * 0.75;
        ctx.fillStyle =
          node.shell === "cortex"
            ? `rgba(245, 158, 11, ${alpha.toFixed(3)})`
            : `rgba(56, 189, 248, ${alpha.toFixed(3)})`;
        ctx.beginPath();
        ctx.arc(node.px, node.py, size, 0, Math.PI * 2);
        ctx.fill();
      }
      raf = requestAnimationFrame(render);
    };
    raf = requestAnimationFrame(render);
    return () => cancelAnimationFrame(raf);
  }, [nodes]);

  return (
    <canvas
      ref={canvasRef}
      width={720}
      height={480}
      className="w-full rounded-lg border border-zinc-800 bg-zinc-950"
      aria-label="180-node dual-shell neural topology"
    />
  );
}

export function PerspectiveNeuralHud() {
  const { state, dispatch, sdk, telemetry, startJob, finishJob } = useMaironWorld();
  const hud = state.neuralHud;
  const [persona, setPersona] = useState<(typeof GROK_VOICE_MODE_WIRE_IDS)[number]>("mairon_sovereign");
  const [uxTier, setUxTier] = useState("pro_terminal");
  const [simPrompt, setSimPrompt] = useState("Explain the treasury streams like I am five");
  const [simResult, setSimResult] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const synthesize = useCallback(async () => {
    const slug = hud.selectedPresetSlug ?? hud.presets?.presets[0]?.slug;
    if (!slug) return;
    setBusy("render");
    setError(null);
    const jobId = startJob("hud-synth", "neural-hud", `preset ${slug}`);
    try {
      const response = await sdk.neuralHud.renderPreset({ repoName: slug });
      dispatch({ type: "SET_HUD_RENDER", render: response as Record<string, unknown> });
      telemetry("neural-hud", "info", `HUD preset "${slug}" synthesized with the safety limiter`);
      finishJob(jobId, "succeeded");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("neural-hud", "error", message);
      finishJob(jobId, "failed", message);
    } finally {
      setBusy(null);
    }
  }, [dispatch, finishJob, hud.presets, hud.selectedPresetSlug, sdk, startJob, telemetry]);

  const runGrok = useCallback(async () => {
    setBusy("grok");
    setError(null);
    const jobId = startJob("grok-sim", "neural-hud", `persona ${persona}`);
    try {
      const response = await sdk.grokLab.runSimulation({
        prompt: simPrompt,
        persona,
        uxTier: uxTier as "toyspark" | "simple" | "creator_studio" | "pro_terminal",
        locale: "en",
        think: true,
      });
      setSimResult(response.responseText);
      telemetry(
        "neural-hud",
        "info",
        `Grok turn routed to ${response.modelSlug} (${response.persona}, ${response.thinkingSteps.length} thinking steps)`,
      );
      finishJob(jobId, "succeeded");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      telemetry("neural-hud", "error", message);
      finishJob(jobId, "failed", message);
    } finally {
      setBusy(null);
    }
  }, [dispatch, finishJob, persona, sdk, simPrompt, startJob, telemetry, uxTier]);

  return (
    <section className="grid gap-4 lg:grid-cols-3">
      <div className="lg:col-span-2 grid gap-4">
        <div className="rounded-xl border border-amber-500/30 bg-zinc-900/60 p-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
              Neural topology — {FIBONACCI_SPHERE_TOTAL_NODES} nodes, dual shell
            </h3>
            <span className="text-xs text-zinc-400">
              cortex {FIBONACCI_SHELL_INNER_NODES} + telemetry {FIBONACCI_SHELL_OUTER_NODES}
            </span>
          </div>
          <div className="mt-3">
            <NeuralTopologyCanvas />
          </div>
        </div>

        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            HUD preset catalog
          </h3>
          {hud.presets ? (
            <div className="mt-2 flex flex-wrap gap-2">
              {hud.presets.presets.map((preset) => (
                <button
                  key={preset.slug}
                  type="button"
                  onClick={() => dispatch({ type: "SET_HUD_SELECTED", slug: preset.slug })}
                  className={`rounded-md px-3 py-1.5 text-xs font-medium transition ${
                    hud.selectedPresetSlug === preset.slug
                      ? "bg-amber-500 text-zinc-950"
                      : "border border-zinc-700 text-zinc-300 hover:border-amber-500/60"
                  }`}
                >
                  {preset.title} · {preset.source}
                </button>
              ))}
              <button
                type="button"
                onClick={() => void synthesize()}
                disabled={busy !== null}
                className="rounded-lg bg-amber-500 px-4 py-1.5 text-xs font-semibold text-zinc-950 disabled:opacity-50"
              >
                {busy === "render" ? "Synthesizing..." : "Synthesize preset"}
              </button>
            </div>
          ) : (
            <p className="mt-2 text-xs text-zinc-500">Sync the world state to load the preset catalog.</p>
          )}
          {hud.lastRender ? (
            <pre className="mt-2 max-h-32 overflow-auto rounded-lg border border-zinc-800 bg-zinc-950/80 p-2 text-[11px] text-zinc-300">
              {JSON.stringify(hud.lastRender, null, 2)}
            </pre>
          ) : null}
        </div>

        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            Grok 10-DNA lab — voice switcher
          </h3>
          <div className="mt-2 flex flex-wrap gap-2">
            {GROK_VOICE_MODES.map((mode, index) => (
              <button
                key={mode}
                type="button"
                onClick={() => setPersona(GROK_VOICE_MODE_WIRE_IDS[index])}
                className={`rounded-full px-3 py-1 text-xs font-medium transition ${
                  persona === GROK_VOICE_MODE_WIRE_IDS[index]
                    ? "bg-zinc-100 text-zinc-900"
                    : "border border-zinc-700 text-zinc-300 hover:border-amber-500/60"
                }`}
              >
                {mode}
              </button>
            ))}
          </div>
          <textarea
            aria-label="Grok lab prompt"
            value={simPrompt}
            onChange={(event) => setSimPrompt(event.target.value)}
            rows={2}
            className="mt-3 w-full rounded-lg border border-zinc-700 bg-zinc-950/80 p-3 text-sm text-zinc-100 outline-none focus:border-amber-500/60"
          />
          <button
            type="button"
            onClick={() => void runGrok()}
            disabled={busy !== null}
            className="mt-2 rounded-lg bg-amber-500 px-4 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
          >
            {busy === "grok" ? "Simulating..." : "Run lab turn"}
          </button>
          {error ? <p className="mt-2 text-xs text-red-400">{error}</p> : null}
          {simResult ? (
            <p className="mt-2 rounded-lg border border-zinc-800 bg-zinc-950/80 p-3 text-sm text-zinc-200">
              {simResult}
            </p>
          ) : null}
        </div>
      </div>

      <aside className="grid gap-4">
        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4 text-xs text-zinc-400">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">HUD posture</h3>
          <p className="mt-2">Posture: {state.postures["neural-hud"].posture}</p>
          <p>Preset held: {hud.selectedPresetSlug ?? "none"}</p>
          <a
            href={sdk.neuralHud.dashboardUrl()}
            target="_blank"
            rel="noreferrer"
            className="mt-2 inline-block rounded-lg border border-amber-500/60 px-3 py-1.5 font-semibold text-amber-300"
          >
            Open standalone /neural-hud dashboard
          </a>
        </div>
        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4 text-xs text-zinc-400">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">Safety envelope</h3>
          <p className="mt-2 font-mono text-zinc-200">alimiter=limit=0.95</p>
          <p className="mt-1">
            Every HUD synthesis path ends in the master limiter — the same ceiling the WebAudio voices honor here.
          </p>
        </div>
      </aside>
    </section>
  );
}
