/**
 * MAIRON analytics — SFT/DPO preference-pair builders and JSONL export.
 *
 * Mirrors the kernel's RLHF export contract (`export_rlhf_training_dataset_jsonl`,
 * `KIA-GROK-UX-LAB` telemetry): SFT lines are ChatML-shaped
 * `{ kind: "sft", messages: [...] }`; DPO lines are
 * `{ kind: "dpo", prompt, chosen, rejected }` pairs built from two turns of the
 * same DNA cohort where exactly one turn cleared the REA gate.
 */

import { MAIRON_DNA_COHORTS, type DnaCohortSpec } from "./dna";

/** One lab telemetry turn — the client-side mirror of `UserLabTelemetryLog`. */
export interface DnaTelemetryTurn {
  /** Cohort code — `DNA_01`..`DNA_10`. */
  dnaCode: string;
  /** Hour offset 0..23. */
  hourOffset: number;
  /** UX layer. */
  uxTier: string;
  /** Locale tag. */
  locale: string;
  /** Grok persona wire id. */
  grokPersona: string;
  /** Routed model slug. */
  modelSlug: string;
  /** Routing mode (`agent_auto_decision` | `manual_override`). */
  routingMode: string;
  /** User prompt. */
  promptText: string;
  /** The thinking trace (already JSON-encoded on the kernel wire). */
  thinkingTraceJson: string;
  /** Model response. */
  responseText: string;
  /** REA score 0..100. */
  reaScore: number;
  /** Latency in ms. */
  latencyMs: number;
  /** Training usability flag. */
  usableForTraining: boolean;
}

/** An SFT training line (ChatML-shaped). */
export interface SftLine {
  kind: "sft";
  dnaCode: string;
  grokPersona: string;
  modelSlug: string;
  reaScore: number;
  messages: [{ role: "system"; content: string }, { role: "user"; content: string }, { role: "assistant"; content: string }];
}

/** A DPO preference pair. */
export interface DpoPair {
  kind: "dpo";
  dnaCode: string;
  grokPersona: string;
  prompt: string;
  chosen: string;
  rejected: string;
  chosenReaScore: number;
  rejectedReaScore: number;
}

/** The MAIRON system preamble used for SFT lines (matches the kernel persona rule). */
export function maironSystemPrompt(persona: string, locale: string): string {
  return `You are MAIRON's lab assistant — persona ${persona}, locale ${locale}. Crisp, causal, zero filler; every answer passes the REA gate.`;
}

/**
 * Build one SFT line from a gate-clearing turn (`reaScore >= 85` and
 * `usableForTraining`). Returns null when the turn fails the gate — the
 * export never ships un-usable turns.
 */
export function buildSftLine(turn: DnaTelemetryTurn, minReaScore = 85): SftLine | null {
  if (!turn.usableForTraining || turn.reaScore < minReaScore) return null;
  return {
    kind: "sft",
    dnaCode: turn.dnaCode,
    grokPersona: turn.grokPersona,
    modelSlug: turn.modelSlug,
    reaScore: turn.reaScore,
    messages: [
      { role: "system", content: maironSystemPrompt(turn.grokPersona, turn.locale) },
      { role: "user", content: turn.promptText },
      { role: "assistant", content: turn.responseText },
    ],
  };
}

/**
 * Build a DPO pair from two turns of the same cohort+persona+prompt: the
 * turn that cleared the REA gate is `chosen`, the failed one is `rejected`.
 * Returns null when both fail or both pass.
 */
export function buildDpoPair(a: DnaTelemetryTurn, b: DnaTelemetryTurn, minReaScore = 85): DpoPair | null {
  if (a.dnaCode !== b.dnaCode || a.grokPersona !== b.grokPersona) return null;
  if (a.promptText !== b.promptText) return null;
  const aPass = a.reaScore >= minReaScore;
  const bPass = b.reaScore >= minReaScore;
  if (aPass === bPass) return null;
  const [chosen, rejected] = aPass ? [a, b] : [b, a];
  return {
    kind: "dpo",
    dnaCode: chosen.dnaCode,
    grokPersona: chosen.grokPersona,
    prompt: chosen.promptText,
    chosen: chosen.responseText,
    rejected: rejected.responseText,
    chosenReaScore: chosen.reaScore,
    rejectedReaScore: rejected.reaScore,
  };
}

/**
 * Sweep the telemetry log and emit the full SFT+DPO JSONL dataset:
 * every gate-clearing turn becomes an SFT line; every same-prompt pair with
 * one pass and one fail becomes a DPO pair.
 */
export function buildSftDpoDataset(turns: readonly DnaTelemetryTurn[], minReaScore = 85): string {
  const lines: string[] = [];
  for (const turn of turns) {
    const sft = buildSftLine(turn, minReaScore);
    if (sft) lines.push(JSON.stringify(sft));
  }
  for (let i = 0; i < turns.length; i++) {
    for (let j = i + 1; j < turns.length; j++) {
      const pair = buildDpoPair(turns[i], turns[j], minReaScore);
      if (pair) lines.push(JSON.stringify(pair));
    }
  }
  return lines.join("\n") + (lines.length > 0 ? "\n" : "");
}

/** Cohort descriptor lookup shared by the pair builders. */
export function cohortFor(turn: DnaTelemetryTurn): DnaCohortSpec | undefined {
  return MAIRON_DNA_COHORTS.find((c: { dnaCode: string }) => c.dnaCode === turn.dnaCode);
}
