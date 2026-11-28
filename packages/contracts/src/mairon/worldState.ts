import { z } from "zod";

/**
 * MAIRON — the Single Authoritative World State contract.
 *
 * IMPERIUM law 1: every perspective (Super-Studio, Quant Desk, Treasury,
 * Neural-HUD, War-Room) reads from and mutates one unified, causally
 * consistent snapshot backed by `packages/sdk`. IMPERIUM law 2: switching
 * perspectives never resets, drops, or re-fetches another subsystem's state.
 *
 * This module defines the wire snapshot the SDK's `syncWorldState()` assembles
 * and the TS store in the applications persists across perspective switches.
 */

/** The five live perspectives, in switcher order. */
export const MAIRON_PERSPECTIVES = [
  "super-studio",
  "quant-desk",
  "treasury",
  "neural-hud",
  "war-room",
] as const;
export type MaironPerspectiveT = (typeof MAIRON_PERSPECTIVES)[number];

/** Human labels of the five perspectives. */
export const MAIRON_PERSPECTIVE_LABELS: Record<MaironPerspectiveT, string> = {
  "super-studio": "Cognitive Super-Studio",
  "quant-desk": "Quantitative Execution Desk",
  "treasury": "Base-8453 Treasury & Genome Router",
  "neural-hud": "3D Neural-HUD & Grok Lab",
  "war-room": "Founder War-Room Command Bridge",
};

/** Strict recursive JSON wire value — mirrors the kernel's JSON envelope. */
export type JsonT = string | number | boolean | null | JsonT[] | { [key: string]: JsonT };

/**
 * Strict JSON schema — accepts only real JSON values (strings, finite numbers,
 * booleans, null, arrays, plain objects). JavaScript functions, `undefined`,
 * class instances and other non-JSON payloads are rejected.
 */
export const JsonValue: z.ZodType<JsonT> = z.lazy(() =>
  z.union([
    z.string(),
    z.number(),
    z.boolean(),
    z.null(),
    z.array(JsonValue),
    z.record(z.string(), JsonValue),
  ]),
);

/** Strict JSON object — the concrete payload shape of a subsystem snapshot. */
export const JsonObject = z.record(z.string(), JsonValue);

/** Sync posture of one subsystem inside the world snapshot. */
export const SubsystemSyncPosture = z.object({
  subsystem: z.enum([
    "super-studio", "quant-desk", "treasury", "neural-hud", "war-room",
  ]),
  /** `live` = fresh from the kernel; `stale` = last known; `degraded` = endpoint refused. */
  posture: z.enum(["live", "stale", "degraded"]),
  syncedAtIso: z.string().nullable(),
  detail: z.string().nullable(),
});
export type SubsystemSyncPostureT = z.infer<typeof SubsystemSyncPosture>;

/** A background job registered in the world state — survives perspective switches. */
export const WorldJob = z.object({
  jobId: z.string().min(1).max(120),
  kind: z.enum([
    "trinity-build", "backtest", "split-sim", "hud-synth",
    "war-room-autoplay", "grok-sim", "sandbox-patch",
  ]),
  perspective: z.enum([
    "super-studio", "quant-desk", "treasury", "neural-hud", "war-room",
  ]),
  status: z.enum(["queued", "running", "succeeded", "failed", "cancelled"]),
  startedAtIso: z.string(),
  finishedAtIso: z.string().nullable(),
  detail: z.string().max(400),
});
export type WorldJobT = z.infer<typeof WorldJob>;

/**
 * MaironWorldState — the single authoritative snapshot. Subsystem payloads are
 * optional (`null` until first sync); a degraded subsystem keeps its last known
 * payload instead of dropping it — that is the non-destructive guarantee.
 */
export const MaironWorldState = z.object({
  /** Monotonic revision — incremented on every committed mutation. */
  revision: z.number().int().min(0),
  snapshotAtIso: z.string(),
  perspective: z.enum([
    "super-studio", "quant-desk", "treasury", "neural-hud", "war-room",
  ]),
  postures: z.array(SubsystemSyncPosture),
  jobs: z.array(WorldJob),
  /** Last known war-room snapshot (nullable until first sync). */
  warRoom: JsonObject.nullable(),
  /** Last known treasury charter snapshot (nullable until first sync). */
  treasury: JsonObject.nullable(),
  /** Last known genome matrix snapshot (nullable until first sync). */
  genome: JsonObject.nullable(),
  /** Last known quant venue matrix snapshot (nullable until first sync). */
  quant: JsonObject.nullable(),
  /** Last known HUD preset catalog snapshot (nullable until first sync). */
  neuralHud: JsonObject.nullable(),
});
export type MaironWorldStateT = z.infer<typeof MaironWorldState>;

/** A telemetry event appended to the world-state stream. */
export const WorldTelemetryEvent = z.object({
  atIso: z.string(),
  source: z.enum([
    "super-studio", "quant-desk", "treasury", "neural-hud", "war-room", "sdk",
  ]),
  level: z.enum(["info", "warn", "error"]),
  message: z.string().max(400),
});
export type WorldTelemetryEventT = z.infer<typeof WorldTelemetryEvent>;
