"use client";

import { cn } from "../lib/utils";

/**
 * TrinityIqMeter — the 3-avatar calibration meter.
 *
 * One row, three stages, locked kernel constants:
 * Alpha IQ 136 -> CommandCenter IQ 139 -> Mairon IQ 150,
 * with the REA >= 85 publication gate on the far end.
 */
export interface TrinityIqMeterProps {
  /** Compact mode hides the descriptions (toolbars, badges). */
  compact?: boolean;
  /** The live REA score of the current build (0-100), when known. */
  reaScore?: number | null;
  className?: string;
}

const STAGES = [
  { name: "Alpha", iq: 136, note: "master prompt + scaffold" },
  { name: "CommandCenter", iq: 139, note: "orchestrator + judiciary" },
  { name: "Mairon", iq: 150, note: "overclocked final judge" },
] as const;

export const TRINITY_IQ_TIERS = { alpha: 136, commandCenter: 139, mairon: 150 } as const;
export const REA_GATE_MIN_SCORE = 85;

export function TrinityIqMeter({ compact = false, reaScore = null, className }: TrinityIqMeterProps) {
  const gatePass = typeof reaScore === "number" && reaScore >= REA_GATE_MIN_SCORE;
  return (
    <div
      data-slot="trinity-iq-meter"
      className={cn("inline-flex items-stretch overflow-hidden rounded-lg border border-zinc-700/80 bg-zinc-900/70", className)}
      role="group"
      aria-label="Trinity three-avatar IQ meter"
    >
      {STAGES.map((s, i) => (
        <div
          key={s.name}
          className={cn(
            "flex flex-col items-center px-3 py-1.5",
            i > 0 && "border-l border-zinc-700/80",
          )}
        >
          <span className="text-[10px] uppercase tracking-widest text-zinc-500">{s.name}</span>
          <span className="font-mono text-sm font-bold text-zinc-100">IQ {s.iq}</span>
          {!compact ? <span className="text-[10px] text-zinc-500">{s.note}</span> : null}
        </div>
      ))}
      <div
        className={cn(
          "flex flex-col items-center justify-center border-l border-zinc-700/80 px-3 py-1.5",
          gatePass ? "bg-emerald-500/10 text-emerald-300" : "bg-zinc-800/60 text-zinc-400",
        )}
        aria-label={gatePass ? `REA gate passed with ${reaScore}` : `REA gate needs at least ${REA_GATE_MIN_SCORE}`}
      >
        <span className="text-[10px] uppercase tracking-widest">REA gate</span>
        <span className="font-mono text-sm font-bold">
          {typeof reaScore === "number" ? reaScore : `>= ${REA_GATE_MIN_SCORE}`}
        </span>
      </div>
    </div>
  );
}
