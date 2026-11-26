import { z } from "zod";

/**
 * MAIRON — Mairon Super-Studio and Competitive Supremacy Sandbox wire contracts.
 *
 * TypeScript mirrors of `crates/contracts/src/studio.rs` and
 * `crates/contracts/src/supremacy.rs`: the `UxMorphMode` ladder
 * (`ToySpark` <-> `CreatorStudio` <-> `ProTerminal`), the nine zero-cost
 * studio modalities, surgical `SearchReplacePatch` operations, time-travel
 * checkpoints, MCP tool invocations, and the `MaironVerifiedSeal`.
 */

// ── Super-Studio ───────────────────────────────────────────────────────────────

/** The three cognitive morph modes, in ladder order. */
export const UX_MORPH_MODES = ["ToySpark", "CreatorStudio", "ProTerminal"] as const;
export type UxMorphModeT = (typeof UX_MORPH_MODES)[number];

/** Discriminated morph-mode descriptor with its ergonomic contract. */
export const UxMorphMode = z.object({
  mode: z.enum(["ToySpark", "CreatorStudio", "ProTerminal"]),
  ageBand: z.string().min(1).max(64),
  maxActionsPerScreen: z.number().int().min(1).max(64),
  minTapTargetPx: z.number().int().min(24).max(96),
  maxWordsPerLabel: z.number().int().min(1).max(32),
  voiceGuided: z.boolean(),
  zeroLossSandbox: z.boolean(),
  keyboardFirst: z.boolean(),
  layoutSummary: z.string().min(1).max(240),
});
export type UxMorphModeSpecT = z.infer<typeof UxMorphMode>;

/** The nine zero-cost studio modalities. */
export const STUDIO_MODALITIES = [
  "search", "deep", "agent", "workspace", "voice",
  "image", "video", "copy", "sfx",
] as const;
export type StudioModalityIdT = (typeof STUDIO_MODALITIES)[number];

/** Mirror of `StudioModalityView`. */
export const StudioModalityView = z.object({
  modality: z.enum([
    "search", "deep", "agent", "workspace", "voice",
    "image", "video", "copy", "sfx",
  ]),
  faLabel: z.string(),
  auth: z.literal("none"),
  engine: z.string(),
  zeroCost: z.literal(true),
});
export type StudioModalityViewT = z.infer<typeof StudioModalityView>;

/** Mirror of `CognitiveTierView`. */
export const CognitiveTierView = z.object({
  tier: z.enum(["ToySpark", "CreatorStudio", "ProTerminal"]),
  faLabel: z.string(),
  ageBand: z.string(),
  maxActionsPerScreen: z.number().int(),
  minTapTargetPx: z.number().int(),
  maxWordsPerLabel: z.number().int(),
  voiceGuided: z.boolean(),
  zeroLossSandbox: z.boolean(),
  keyboardFirst: z.boolean(),
  layoutSummary: z.string(),
});
export type CognitiveTierViewT = z.infer<typeof CognitiveTierView>;

/** Mirror of `PortfolioSafetyFixView`. */
export const PortfolioSafetyFixView = z.object({
  fixId: z.number().int().min(1).max(6),
  title: z.string(),
  status: z.literal("fixed"),
  evidence: z.string(),
});
export type PortfolioSafetyFixViewT = z.infer<typeof PortfolioSafetyFixView>;

/** Mirror of `PortfolioSafetyReportView`. */
export const PortfolioSafetyReportView = z.object({
  fixedCount: z.number().int(),
  allFixed: z.boolean(),
  fixes: z.array(PortfolioSafetyFixView),
});
export type PortfolioSafetyReportViewT = z.infer<typeof PortfolioSafetyReportView>;

/** Mirror of `StudioUxProfilesResponse`. */
export const StudioUxProfilesResponse = z.object({
  tiers: z.array(CognitiveTierView),
  modalities: z.array(StudioModalityView),
  portfolioSafety: PortfolioSafetyReportView,
  signature: z.string(),
});
export type StudioUxProfilesResponseT = z.infer<typeof StudioUxProfilesResponse>;

/** Mirror of `StudioGenerateRequest`. */
export const StudioGenerateRequest = z.object({
  prompt: z.string(),
  modality: z.enum([
    "search", "deep", "agent", "workspace", "voice",
    "image", "video", "copy", "sfx",
  ]),
  cognitiveTier: z.string().optional(),
  originLocale: z.string().optional(),
  targetLocales: z.array(z.string()).optional(),
});
export type StudioGenerateRequestT = z.infer<typeof StudioGenerateRequest>;

/** Mirror of `DualBundleView` — the Ocean/Cosmos dual synthesis bundle. */
export const DualBundleView = z.object({
  fishkalJs: z.string(),
  cosmosJs: z.string(),
  pauseLifecycleJs: z.string(),
  signature: z.string(),
  contentSha256: z.string(),
});
export type DualBundleViewT = z.infer<typeof DualBundleView>;

/** Mirror of `StudioGenerateResponse`. */
export const StudioGenerateResponse = z.object({
  creationId: z.string(),
  cognitiveTier: z.string(),
  modality: z.string(),
  title: z.string(),
  artifactHtml: z.string(),
  dualBundle: DualBundleView,
  storefrontCard: z.record(z.string(), z.union([z.string(), z.number(), z.boolean(), z.array(z.union([z.string(), z.number()]))])),
  circularFlagStripHtml: z.string(),
  originLocale: z.string(),
  supportedLocales: z.array(z.string()),
  skillMd: z.string().nullable(),
  skillJson: z.string().nullable(),
  reaScore: z.number().int().min(0).max(100),
});
export type StudioGenerateResponseT = z.infer<typeof StudioGenerateResponse>;

// ── Supremacy Sandbox ──────────────────────────────────────────────────────────

/**
 * Search/replace patch block — the canonical `<<<<<<< SEARCH` / `=======` /
 * `>>>>>>> REPLACE` surgical edit unit applied by the supremacy engine.
 */
export const SearchReplacePatch = z.object({
  search: z.string().min(1),
  replace: z.string().min(1),
  /** Zero-based occurrence index; default applies to the first match. */
  occurrence: z.number().int().min(0).optional(),
});
export type SearchReplacePatchT = z.infer<typeof SearchReplacePatch>;

/** Mirror of `SupremacySurgicalPatchRequest`. */
export const SupremacySurgicalPatchRequest = z.object({
  originalSource: z.string(),
  patchPayload: z.string(),
  locale: z.string().optional(),
  injectSandboxInspector: z.boolean(),
});
export type SupremacySurgicalPatchRequestT = z.infer<typeof SupremacySurgicalPatchRequest>;

/** Mirror of `SupremacySandboxBundleView`. */
export const SupremacySandboxBundleView = z.object({
  locale: z.string(),
  instrumentedHtml: z.string(),
  nodeCount: z.number().int().min(0),
  messageType: z.string(),
  inspectorScriptBytes: z.number().int().min(0),
});
export type SupremacySandboxBundleViewT = z.infer<typeof SupremacySandboxBundleView>;

/** Mirror of `SupremacySurgicalPatchResponse`. */
export const SupremacySurgicalPatchResponse = z.object({
  applied: z.boolean(),
  blocksTotal: z.number().int().min(0),
  blocksApplied: z.number().int().min(0),
  failedBlockIndexes: z.array(z.number().int()),
  sandboxBundle: SupremacySandboxBundleView,
  patchedSource: z.string().nullable(),
});
export type SupremacySurgicalPatchResponseT = z.infer<typeof SupremacySurgicalPatchResponse>;

/**
 * Time-travel checkpoint — a durable workflow state snapshot with a verifiable
 * SHA-256 chain, forkable to any historical point without re-execution.
 */
export const TimeTravelCheckpoint = z.object({
  checkpointId: z.string().min(1).max(160),
  workflowId: z.string().min(1).max(160),
  stepIndex: z.number().int().min(0),
  agentRole: z.string().min(1).max(64),
  stateJson: z.string().min(1),
  stateSha256: z.string().min(16).max(128),
  parentCheckpointId: z.string().nullable(),
  createdAtIso: z.string(),
});
export type TimeTravelCheckpointT = z.infer<typeof TimeTravelCheckpoint>;

/** Mirror of `SupremacyCheckpointForkRequest`. */
export const SupremacyCheckpointForkRequest = z.object({
  workflowId: z.string(),
  forkFromCheckpointId: z.string(),
  statePatchJson: z.string(),
  agentRole: z.string().optional(),
});
export type SupremacyCheckpointForkRequestT = z.infer<typeof SupremacyCheckpointForkRequest>;

/** Mirror of `SupremacyCheckpointForkResponse`. */
export const SupremacyCheckpointForkResponse = z.object({
  forkedCheckpointId: z.string(),
  parentCheckpointId: z.string(),
  workflowId: z.string(),
  stepIndex: z.number().int(),
  rewoundSteps: z.number().int(),
  stateJson: z.string(),
  stateSha256: z.string(),
  ledgerSize: z.number().int(),
  timeTravelVerified: z.boolean(),
  agentRole: z.string(),
});
export type SupremacyCheckpointForkResponseT = z.infer<typeof SupremacyCheckpointForkResponse>;

/**
 * MCP tool invocation — JSON-RPC 2.0 envelope for the sandbox tool bus
 * (`initialize` | `tools/list` | `tools/call`).
 */
export const McpToolInvocation = z.object({
  jsonrpc: z.literal("2.0"),
  id: z.union([z.string(), z.number(), z.null()]),
  method: z.enum(["initialize", "tools/list", "tools/call"]),
  params: z.unknown().optional(),
});
export type McpToolInvocationT = z.infer<typeof McpToolInvocation>;

/** Mirror of `SupremacyMcpResponse`. */
export const SupremacyMcpResponse = z.object({
  jsonrpc: z.literal("2.0"),
  id: z.union([z.string(), z.number(), z.null()]),
  result: z.unknown().nullable(),
  error: z.object({ code: z.number().int(), message: z.string() }).nullable(),
});
export type SupremacyMcpResponseT = z.infer<typeof SupremacyMcpResponse>;

/**
 * MaironVerifiedSeal — the institutional approval stamp minted only when the
 * REA score reaches `85`; carries the SHA-256 of the sealed artifact.
 */
export const MaironVerifiedSeal = z.object({
  sealId: z.string().regex(/^mairon-seal:[0-9a-f]{16}$/),
  reaScore: z.number().int().min(85).max(100),
  artifactSha256: z.string().min(16).max(128),
  language: z.enum(["rust", "typescript", "solidity", "html"]),
  mintedAtIso: z.string(),
});
export type MaironVerifiedSealT = z.infer<typeof MaironVerifiedSeal>;

/** Mirror of `SupremacySweVerifyRequest`. */
export const SupremacySweVerifyRequest = z.object({
  planSummary: z.string(),
  language: z.enum(["rust", "typescript", "solidity", "html"]),
  artifact: z.string(),
  maxRepairAttempts: z.number().int().min(1).max(9).optional(),
});
export type SupremacySweVerifyRequestT = z.infer<typeof SupremacySweVerifyRequest>;

/** Mirror of `SupremacySweCheckView`. */
export const SupremacySweCheckView = z.object({
  name: z.string(),
  passed: z.boolean(),
  detail: z.string(),
});
export type SupremacySweCheckViewT = z.infer<typeof SupremacySweCheckView>;

/** Mirror of `SupremacySweVerifyResponse`. */
export const SupremacySweVerifyResponse = z.object({
  verified: z.boolean(),
  attemptsUsed: z.number().int().min(1),
  reaScore: z.number().int().min(0).max(100),
  checks: z.array(SupremacySweCheckView),
  diagnostics: z.array(z.string()),
  repairPromptsIssued: z.number().int().min(0),
  artifactSha256: z.string(),
  seal: z.object({
    sealId: z.string(),
    reaScore: z.number().int(),
    artifactSha256: z.string(),
  }).nullable(),
});
export type SupremacySweVerifyResponseT = z.infer<typeof SupremacySweVerifyResponse>;
