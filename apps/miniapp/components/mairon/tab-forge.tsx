"use client";

/**
 * Tab 1 — ToySpark & Trinity Quick-Forge.
 *
 * 64px tactile ToySpark cards, the Trinity 3-Avatar pipeline
 * (Alpha IQ 136 -> CommandCenter IQ 139 -> Mairon IQ 150), the REA >= 85
 * gate badge, and the Arda 20-locale (PATCH_READY_SLOT) switcher — every
 * control wired to the real `/api/v1/*` routes through `MaironSdkClient`.
 */

import { useCallback, useState } from "react";
import { Loader2 } from "lucide-react";
import { TrinityIqMeter } from "@kian/ui";
import { describeError, useMaironMiniApp } from "@/lib/mairon-state";

/** The ToySpark quick-idea cards — one tap fills the forge idea. */
const TOYSPARK_IDEAS: { emoji: string; label: string; idea: string }[] = [
  { emoji: "🌈", label: "Rainbow game", idea: "A one-tap rainbow tap game with a big 64px button and a score counter" },
  { emoji: "🐸", label: "Leap counter", idea: "A frog leap counter that records every jump with a satisfying haptic tick" },
  { emoji: "🎈", label: "Balloon pop", idea: "A balloon popping arcade with colorful balloons floating up the screen" },
  { emoji: "⭐", label: "Star collector", idea: "A star collector where a mini spaceship grabs falling stars" },
];

interface ArdaLocale {
  localeCode: string;
  direction: string;
  activationState: string;
  completionBps: number;
}

export function TabForge() {
  const { state, sdk, patchDraft, registerJob, setError } = useMaironMiniApp();
  const { drafts } = state;
  const [running, setRunning] = useState(false);
  const [reaScore, setReaScore] = useState<number | null>(null);
  const [projectName, setProjectName] = useState<string | null>(null);
  const [locales, setLocales] = useState<ArdaLocale[]>([]);
  const [localesNote, setLocalesNote] = useState<string | null>(null);

  const loadLocales = useCallback(async () => {
    try {
      const catalog = await sdk.arda.listLocales();
      const rows = (catalog as { locales?: ArdaLocale[] }).locales ?? [];
      setLocales(rows);
      const patchReady = rows.filter((l) => l.activationState === "PATCH_READY_SLOT").length;
      setLocalesNote(`${rows.length} locales · ${patchReady} PATCH_READY_SLOT`);
    } catch (err) {
      setLocalesNote(null);
      setError(describeError(err));
    }
  }, [sdk, setError]);

  async function runTrinity(): Promise<void> {
    if (drafts.forgeIdea.trim().length === 0) {
      setError("Describe the idea first — pick a ToySpark card or type your own.");
      return;
    }
    setRunning(true);
    setError(null);
    const jobId = `trinity-${Date.now()}`;
    registerJob({
      jobId,
      perspective: "forge",
      label: "Trinity orchestration",
      status: "running",
      startedAtIso: new Date().toISOString(),
      detail: drafts.forgeIdea.slice(0, 120),
    });
    try {
      const response = await sdk.trinity.orchestrate({
        sessionId: drafts.forgeSessionId ?? `miniapp-${jobId}`,
        slots: {
          coreObjective: drafts.forgeIdea,
          targetAudienceAndUx: "64px ToySpark tactile audience, one-hand portrait flow",
          interactiveFeatures: drafts.forgeProductName
            ? `publish as ${drafts.forgeProductName}`
            : undefined,
          monetizationAndViralLoop: "affiliate referral loop, 25 percent commission",
        },
      });
      const score = (response as { reaScore?: number }).reaScore ?? null;
      const name = (response as { projectName?: string }).projectName ?? null;
      setReaScore(typeof score === "number" ? score : null);
      setProjectName(name);
      patchDraft({ forgeSessionId: (response as { sessionId?: string }).sessionId ?? drafts.forgeSessionId });
      registerJob({
        jobId,
        perspective: "forge",
        label: "Trinity orchestration",
        status: "succeeded",
        startedAtIso: new Date().toISOString(),
        detail: typeof score === "number" ? `REA score ${score}` : "orchestration finished",
      });
    } catch (err) {
      setError(describeError(err));
      registerJob({
        jobId,
        perspective: "forge",
        label: "Trinity orchestration",
        status: "failed",
        startedAtIso: new Date().toISOString(),
        detail: describeError(err).slice(0, 200),
      });
    } finally {
      setRunning(false);
    }
  }

  return (
    <section aria-label="ToySpark and Trinity quick-forge" className="flex flex-col gap-4">
      <TrinityIqMeter reaScore={reaScore} compact />

      <div className="grid grid-cols-2 gap-2">
        {TOYSPARK_IDEAS.map((card) => (
          <button
            key={card.label}
            type="button"
            onClick={() => patchDraft({ forgeIdea: card.idea })}
            className="flex h-16 flex-col items-center justify-center gap-0.5 rounded-2xl border border-zinc-700/80 bg-zinc-900/70 text-zinc-200 transition active:scale-95 active:bg-amber-400/10"
            aria-label={`Use the ${card.label} idea`}
          >
            <span className="text-2xl" aria-hidden>{card.emoji}</span>
            <span className="text-[11px] font-semibold">{card.label}</span>
          </button>
        ))}
      </div>

      <label className="text-[11px] font-semibold uppercase tracking-wider text-zinc-500" htmlFor="forge-idea">
        Forge idea
      </label>
      <textarea
        id="forge-idea"
        value={drafts.forgeIdea}
        onChange={(e) => patchDraft({ forgeIdea: e.target.value })}
        aria-label="Describe the product idea for the Trinity pipeline"
        rows={3}
        className="w-full rounded-xl border border-zinc-700 bg-zinc-900/80 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-amber-400/60"
      />

      <label className="text-[11px] font-semibold uppercase tracking-wider text-zinc-500" htmlFor="forge-name">
        Product name
      </label>
      <input
        id="forge-name"
        value={drafts.forgeProductName}
        onChange={(e) => patchDraft({ forgeProductName: e.target.value })}
        aria-label="Product name for the storefront"
        className="w-full rounded-xl border border-zinc-700 bg-zinc-900/80 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-amber-400/60"
      />

      <button
        type="button"
        onClick={() => void runTrinity()}
        disabled={running}
        className="flex h-12 items-center justify-center gap-2 rounded-2xl bg-amber-400/15 text-sm font-bold text-amber-200 shadow-[inset_0_0_0_1px_rgba(251,191,36,0.35)] transition active:scale-[0.98] disabled:opacity-60"
      >
        {running ? <Loader2 className="size-4 animate-spin" aria-hidden /> : "⚡"} Forge with Trinity
      </button>

      {projectName ? (
        <p className="text-xs text-zinc-400">
          Trinity session: <span className="font-mono text-zinc-200">{projectName}</span>
          {reaScore !== null ? ` · REA ${reaScore}/100` : ""}
        </p>
      ) : null}

      <div className="rounded-2xl border border-zinc-700/80 bg-zinc-900/60 p-3">
        <div className="flex items-center justify-between">
          <span className="text-[11px] font-semibold uppercase tracking-wider text-zinc-500">Arda locales</span>
          <button
            type="button"
            onClick={() => void loadLocales()}
            className="rounded-lg border border-zinc-700 px-2 py-1 text-[11px] text-zinc-300"
          >
            Load 20-locale manifest
          </button>
        </div>
        {localesNote ? <p className="mt-1 text-[11px] text-zinc-400">{localesNote}</p> : null}
        {locales.length > 0 ? (
          <div className="mt-2 flex flex-wrap gap-1">
            {locales.map((l) => (
              <button
                key={l.localeCode}
                type="button"
                onClick={() => patchDraft({ forgeLocale: l.localeCode })}
                aria-label={`Switch draft locale to ${l.localeCode}`}
                className={`rounded-full border px-2 py-0.5 font-mono text-[10px] ${
                  drafts.forgeLocale === l.localeCode
                    ? "border-amber-400/60 bg-amber-400/10 text-amber-200"
                    : "border-zinc-700 text-zinc-400"
                }`}
              >
                {l.localeCode}
                {l.activationState === "PATCH_READY_SLOT" ? " ·slot" : ""}
              </button>
            ))}
          </div>
        ) : null}
      </div>
    </section>
  );
}
