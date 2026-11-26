import { z } from "zod";

/**
 * MAIRON — 3D Neural-HUD and Grok 10-DNA Lab wire contracts.
 *
 * TypeScript mirrors of the Wave-9 HUD/Grok contracts
 * (`crates/contracts/src/wave9.rs`): the 180-node dual-shell Fibonacci sphere,
 * the WebAudio safety limiter, the lab telemetry row, the SFT/DPO export line,
 * and the four Grok voice modes.
 */

// ── Neural-HUD topology ────────────────────────────────────────────────────────

/** Inner-shell node count of the Fibonacci sphere. */
export const FIBONACCI_SHELL_INNER_NODES = 120 as const;

/** Outer-shell node count of the Fibonacci sphere. */
export const FIBONACCI_SHELL_OUTER_NODES = 60 as const;

/** Total neural topology nodes — dual-shell, 180 total. */
export const FIBONACCI_SPHERE_TOTAL_NODES = 180 as const;

/** The WebAudio master limiter fragment every synthesis path must carry. */
export const AUDIO_SAFETY_LIMITER = "alimiter=limit=0.95" as const;

/**
 * FibonacciSphereNode — one neuron of the 180-node dual-shell topology. The
 * golden-angle layout is deterministic; `shell` separates the 120-node cortex
 * shell from the 60-node telemetry shell.
 */
export const FibonacciSphereNode = z.object({
  index: z.number().int().min(0).max(179),
  shell: z.enum(["cortex", "telemetry"]),
  /** Unit-sphere coordinates from the golden-angle spiral. */
  x: z.number(),
  y: z.number(),
  z: z.number(),
  label: z.string().min(1).max(96),
  /** Edge fan-out to other node indices. */
  edges: z.array(z.number().int().min(0).max(179)),
  activation: z.number().min(0).max(1),
});
export type FibonacciSphereNodeT = z.infer<typeof FibonacciSphereNode>;

/**
 * AudioSafetyLimiterConfig — the non-negotiable synthesis safety envelope;
 * the FFmpeg audio path always ends with `alimiter=limit=0.95`.
 */
export const AudioSafetyLimiterConfig = z.object({
  limiterFilter: z.literal("alimiter=limit=0.95"),
  sampleRateHz: z.number().int(),
  /** Master gain ceiling before the limiter. */
  preGainDb: z.number(),
  /** True peak ceiling after the limiter. */
  ceilingDb: z.number(),
});
export type AudioSafetyLimiterConfigT = z.infer<typeof AudioSafetyLimiterConfig>;

// ── Grok 10-DNA Lab ────────────────────────────────────────────────────────────

/** The four Grok voice personas. */
export const GROK_VOICE_MODES = ["FunWitty", "UnhingedRoast", "RegularBalanced", "MaironSovereign"] as const;
export type GrokVoiceModeT = (typeof GROK_VOICE_MODES)[number];

/** Wire IDs of the four Grok voice modes, as the kernel expects them. */
export const GROK_VOICE_MODE_WIRE_IDS = [
  "fun_witty", "unhinged_roast", "regular_balanced", "mairon_sovereign",
] as const;

/** The ten lab DNA cohorts. */
export const LAB_DNA_COHORTS = [
  "DNA-01", "DNA-02", "DNA-03", "DNA-04", "DNA-05",
  "DNA-06", "DNA-07", "DNA-08", "DNA-09", "DNA-10",
] as const;
export type LabDnaCohortT = (typeof LAB_DNA_COHORTS)[number];

/** Seeded benchmark logs per DNA cohort. */
export const LAB_SEEDED_LOGS_PER_COHORT = 32 as const;

/**
 * UserLabTelemetryRow — one logged lab turn (1:1 with the kernel's
 * `LabTelemetryEntryView`, `user_handle` and friends on the wire).
 */
export const UserLabTelemetryRow = z.object({
  id: z.string(),
  dnaCode: z.string(),
  userHandle: z.string(),
  hourOffset: z.number().int().min(0).max(24),
  uxTier: z.enum(["toyspark", "simple", "creator_studio", "pro_terminal"]),
  locale: z.string(),
  grokPersona: z.enum(["fun_witty", "unhinged_roast", "regular_balanced", "mairon_sovereign"]),
  modelSlug: z.string(),
  routingMode: z.string(),
  promptText: z.string(),
  thinkingTraceJson: z.string(),
  responseText: z.string(),
  reaScore: z.number().int().min(0).max(100),
  latencyMs: z.number().int().min(0),
  usableForTraining: z.boolean(),
  createdAt: z.string(),
});
export type UserLabTelemetryRowT = z.infer<typeof UserLabTelemetryRow>;

/**
 * SftDpoExportLine — one JSONL line of the exportable training dataset;
 * `chosen`/`rejected` pairs for DPO, plain completions for SFT.
 */
export const SftDpoExportLine = z.object({
  id: z.string(),
  dnaCode: z.string(),
  format: z.enum(["sft", "dpo"]),
  system: z.string().nullable(),
  prompt: z.string(),
  chosen: z.string(),
  rejected: z.string().nullable(),
  reaScore: z.number().int().min(0).max(100),
  meta: z.object({
    persona: z.string(),
    modelSlug: z.string(),
    uxTier: z.string(),
    locale: z.string(),
    latencyMs: z.number().int(),
  }),
});
export type SftDpoExportLineT = z.infer<typeof SftDpoExportLine>;

// ── Direct mirrors of the Wave-9 wire contracts ────────────────────────────────

/** Mirror of `HudPresetView`. */
export const HudPresetView = z.object({
  slug: z.string(),
  title: z.string(),
  tagline: z.string(),
  accentHex: z.string(),
  badge: z.string(),
  source: z.enum(["upstream", "kia_sovereign", "synthesized"]),
  nodeLabels: z.array(z.string()),
});
export type HudPresetViewT = z.infer<typeof HudPresetView>;

/** Mirror of `HudPresetsResponse`. */
export const HudPresetsResponse = z.object({
  presets: z.array(HudPresetView),
  signature: z.string(),
});
export type HudPresetsResponseT = z.infer<typeof HudPresetsResponse>;

/** Mirror of `GrokUiBundleQuery`. */
export const GrokUiBundleQuery = z.object({
  locale: z.string().optional(),
  uxTier: z.enum(["toyspark", "simple", "creator_studio", "pro_terminal"]).optional(),
  persona: z.enum(["fun_witty", "unhinged_roast", "regular_balanced", "mairon_sovereign"]).optional(),
});
export type GrokUiBundleQueryT = z.infer<typeof GrokUiBundleQuery>;

/** Mirror of `GrokUiBundleResponse`. */
export const GrokUiBundleResponse = z.object({
  html: z.string(),
  locale: z.string(),
  direction: z.enum(["rtl", "ltr"]),
  uxTier: z.string(),
  persona: z.string(),
  chips: z.array(z.string()),
  ghostFollowUps: z.array(z.string()),
  cssSignature: z.string(),
  localeSlots: z.number().int(),
});
export type GrokUiBundleResponseT = z.infer<typeof GrokUiBundleResponse>;

/** Mirror of `GrokChatTurnRequest`. */
export const GrokChatTurnRequest = z.object({
  prompt: z.string().min(1),
  locale: z.string().optional(),
  uxTier: z.enum(["toyspark", "simple", "creator_studio", "pro_terminal"]).optional(),
  persona: z.enum(["fun_witty", "unhinged_roast", "regular_balanced", "mairon_sovereign"]).optional(),
  deepSearch: z.boolean().optional(),
  think: z.boolean().optional(),
});
export type GrokChatTurnRequestT = z.infer<typeof GrokChatTurnRequest>;

/** Mirror of `GrokChatTurnResponse`. */
export const GrokChatTurnResponse = z.object({
  thinkingSteps: z.array(z.object({ step: z.string() })),
  thinkingDurationMs: z.number().int(),
  citations: z.array(z.object({
    title: z.string(),
    domain: z.string(),
    verified: z.boolean(),
  })),
  responseText: z.string(),
  ghostFollowUps: z.array(z.string()),
  persona: z.string(),
  modelSlug: z.string(),
  locale: z.string(),
  uxTier: z.string(),
});
export type GrokChatTurnResponseT = z.infer<typeof GrokChatTurnResponse>;
