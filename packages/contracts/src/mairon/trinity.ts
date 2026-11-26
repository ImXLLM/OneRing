import { z } from "zod";

/**
 * MAIRON — Mairon Trinity Engine wire contracts.
 *
 * TypeScript mirror of `crates/contracts/src/trinity.rs` plus the institutional
 * framing the platform requires: the three-avatar ladder (Alpha IQ 136 ->
 * CommandCenter IQ 139 -> Mairon IQ 150), the REA publication gate
 * (`score >= 85`), the two-key publish ticket, and the Bayesian review summary.
 *
 * Wire rules: camelCase, explicit null for nullable fields, integers only
 * (no floats on money), and every field the Rust struct sends must exist here.
 */

// ── Institutional framing ─────────────────────────────────────────────────────

/** The three Mairon Trinity avatars, in ladder order. */
export const TRINITY_AVATAR_LADDER = [
  { id: "alpha", name: "Alpha", iq: 136, stage: "Discovery — five-slot intake" },
  { id: "logic_prime", name: "CommandCenter", iq: 139, stage: "Forging — deterministic subagent fleet" },
  { id: "mairon", name: "Mairon", iq: 150, stage: "Judgment — REA arbitration and repair" },
] as const;

export type TrinityAvatarIdT = (typeof TRINITY_AVATAR_LADDER)[number]["id"];

/** Discriminated stage descriptor for one avatar of the Trinity pipeline. */
export const TrinityAvatarStage = z.object({
  avatarId: z.enum(["alpha", "logic_prime", "mairon"]),
  avatarName: z.enum(["Alpha", "CommandCenter", "Mairon"]),
  iq: z.union([z.literal(136), z.literal(139), z.literal(150)]),
  status: z.enum(["idle", "listening", "forging", "judging", "approved", "rejected"]),
  readinessPct: z.number().int().min(0).max(100),
  startedAtIso: z.string().nullable(),
  finishedAtIso: z.string().nullable(),
});
export type TrinityAvatarStageT = z.infer<typeof TrinityAvatarStage>;

/** The single publication quality gate — a listing ships only at `score >= 85`. */
export const REA_GATE_MIN_SCORE = 85 as const;

/** Request to run the full Trinity build (mirror of `TrinityOrchestrateRequest`). */
export const TrinityBuildRequest = z.object({
  sessionId: z.string().min(1),
  sessionSeed: z.number().int().min(0).optional(),
  slots: TrinitySlotsState,
});
export type TrinityBuildRequestT = z.infer<typeof TrinityBuildRequest>;

/** REA audit report — mirror of the Mairon arbitration block. */
export const ReaAuditReport = z.object({
  finalScore: z.number().int().min(0).max(100),
  gateMinScore: z.literal(85),
  approvedForListing: z.boolean(),
  pillars: TrinityPillars,
  repairAttemptsUsed: z.number().int().min(0).max(3),
  artifactSha256: z.string().min(16).max(128),
  badge: z.string().min(1).max(64),
});
export type ReaAuditReportT = z.infer<typeof ReaAuditReport>;

/**
 * Two-key publish ticket — the wire shape for the dual-control listing flow:
 * key 1 mints the ticket from a passing REA report, key 2 (a second operator)
 * releases it to the marketplace. Both keys are required; neither alone suffices.
 */
export const TwoKeyPublishTicket = z.object({
  ticketId: z.string().min(1).max(120),
  productDraftRef: z.string().min(1).max(160),
  reaScore: z.number().int().min(0).max(100),
  mintedByKeyId: z.string().min(1).max(120),
  releasedByKeyId: z.string().nullable(),
  state: z.enum(["MINTED", "RELEASED", "REVOKED", "EXPIRED"]),
  mintedAtIso: z.string(),
  releasedAtIso: z.string().nullable(),
  expiresAtIso: z.string().nullable(),
});
export type TwoKeyPublishTicketT = z.infer<typeof TwoKeyPublishTicket>;

/** Bayesian review summary — aggregated reviewer evidence for a listing. */
export const BayesianReviewSummary = z.object({
  productRef: z.string().min(1).max(160),
  priorMean: z.number().min(0).max(1),
  posteriorMean: z.number().min(0).max(1),
  evidenceWeight: z.number().int().min(0),
  positiveVotes: z.number().int().min(0),
  negativeVotes: z.number().int().min(0),
  abstentions: z.number().int().min(0),
  confidenceIntervalLow: z.number().min(0).max(1),
  confidenceIntervalHigh: z.number().min(0).max(1),
  updatedAtIso: z.string(),
});
export type BayesianReviewSummaryT = z.infer<typeof BayesianReviewSummary>;

// ── Direct mirrors of crates/contracts/src/trinity.rs ──────────────────────────

/** Mirror of `TrinitySlotsState` — the five Alpha discovery slots. */
export const TrinitySlotsState = z.object({
  coreObjective: z.string().optional(),
  targetAudienceAndUx: z.string().optional(),
  interactiveFeatures: z.string().optional(),
  dataAndZeroAuthApis: z.string().optional(),
  monetizationAndViralLoop: z.string().optional(),
});
export type TrinitySlotsStateT = z.infer<typeof TrinitySlotsState>;

/** Mirror of `TrinityAlphaRequest`. */
export const TrinityAlphaRequest = z.object({
  sessionId: z.string().optional(),
  userMessage: z.string().min(1).max(4000),
  priorSlots: TrinitySlotsState.optional(),
});
export type TrinityAlphaRequestT = z.infer<typeof TrinityAlphaRequest>;

/** Mirror of `TrinityMasterPromptView`. */
export const TrinityMasterPromptView = z.object({
  specHash: z.string(),
  systemDirective: z.string(),
  functionalSpec: z.string(),
  ux390Constraints: z.string(),
  verificationCriteria: z.array(z.string()),
  compiledPrompt: z.string(),
  projectName: z.string(),
  priceStarsHint: z.number().int().min(0),
});
export type TrinityMasterPromptViewT = z.infer<typeof TrinityMasterPromptView>;

/** Mirror of `TrinityBlueprintModuleView`. */
export const TrinityBlueprintModuleView = z.object({
  path: z.string(),
  role: z.string(),
  dependencies: z.array(z.string()),
  specialty: z.string(),
  specialtyFa: z.string(),
});
export type TrinityBlueprintModuleViewT = z.infer<typeof TrinityBlueprintModuleView>;

/** Mirror of `TrinityBlueprintView`. */
export const TrinityBlueprintView = z.object({
  projectName: z.string(),
  projectSlug: z.string(),
  modules: z.array(TrinityBlueprintModuleView),
});
export type TrinityBlueprintViewT = z.infer<typeof TrinityBlueprintView>;

/** Mirror of `TrinityAlphaResponse`. */
export const TrinityAlphaResponse = z.object({
  status: z.enum(["needs_more_discovery", "locked_for_command_center"]),
  sessionId: z.string(),
  citizenId: z.string(),
  alphaReplyFa: z.string(),
  missingSlots: z.array(z.string()),
  readinessPct: z.number().int().min(0).max(100),
  updatedSlots: TrinitySlotsState,
  masterPrompt: TrinityMasterPromptView.nullable(),
  blueprint: TrinityBlueprintView.nullable(),
});
export type TrinityAlphaResponseT = z.infer<typeof TrinityAlphaResponse>;

/** Mirror of `TrinityAlleleView`. */
export const TrinityAlleleView = z.object({
  surgicalDensity: z.number(),
  uxCraft: z.number(),
  edgeCaseParanoia: z.number(),
  executionVelocity: z.number(),
  alleleSeed: z.number().int().min(0),
});
export type TrinityAlleleViewT = z.infer<typeof TrinityAlleleView>;

/** Mirror of `TrinitySubAgentDnaView` — the `3x9` OpenBot fleet DNA manifest. */
export const TrinitySubAgentDnaView = z.object({
  subagentIndex: z.number().int().min(0),
  specialty: z.string(),
  specialtyFa: z.string(),
  assignedModules: z.array(z.string()),
  threadId: z.string(),
  logicIq: z.number().int(),
  logicEq: z.number().int(),
  watchdogSoftSecs: z.number().int().min(0),
  watchdogHardSecs: z.number().int().min(0),
  caveman: z.enum(["off", "lite", "full", "ultra"]),
  alleles: TrinityAlleleView,
  layer1Chars: z.number().int().min(0),
  layer2Chars: z.number().int().min(0),
  layer3Chars: z.number().int().min(0),
  promptPreview: z.string(),
  xmlPromptChars: z.number().int().min(0),
  empiricalProbes: z.array(z.string()),
  handoffCard: z.string(),
  sanitizerPassed: z.boolean(),
});
export type TrinitySubAgentDnaViewT = z.infer<typeof TrinitySubAgentDnaView>;

/** Mirror of `TrinityPillarsView` — four REA pillars, `0..=25` each. */
export const TrinityPillars = z.object({
  architecture: z.number().int().min(0).max(25),
  securityHardening: z.number().int().min(0).max(25),
  uxAndAntiSlop: z.number().int().min(0).max(25),
  verificationProof: z.number().int().min(0).max(25),
});
export type TrinityPillarsT = z.infer<typeof TrinityPillars>;

/** Mirror of `TrinityRepairDirectiveView`. */
export const TrinityRepairDirectiveView = z.object({
  attempt: z.number().int().min(1).max(3),
  targetSpecialty: z.string(),
  pillar: z.string(),
  directiveEn: z.string(),
  directiveFa: z.string(),
});
export type TrinityRepairDirectiveViewT = z.infer<typeof TrinityRepairDirectiveView>;

/** Mirror of `TrinityRepairRoundView`. */
export const TrinityRepairRoundView = z.object({
  attempt: z.number().int().min(1).max(3),
  scoreBefore: z.number().int().min(0).max(100),
  scoreAfter: z.number().int().min(0).max(100),
  approved: z.boolean(),
  directives: z.array(TrinityRepairDirectiveView),
});
export type TrinityRepairRoundViewT = z.infer<typeof TrinityRepairRoundView>;

/** Mirror of `TrinityAdvisoryView` — Mairon's `IQ=150 / EQ=145` strategic advisory. */
export const TrinityAdvisoryView = z.object({
  iq: z.number().int(),
  eq: z.number().int(),
  lookaheadHorizon: z.number().int(),
  tomOrder: z.number().int(),
  ambitionGain: z.number(),
  eightStepLookaheadSummary: z.string(),
  thirdOrderTomMarketHookFa: z.string(),
});
export type TrinityAdvisoryViewT = z.infer<typeof TrinityAdvisoryView>;

/** Mirror of `TrinityOrchestrateRequest`. */
export const TrinityOrchestrateRequest = z.object({
  sessionId: z.string().min(1),
  sessionSeed: z.number().int().min(0).optional(),
  slots: TrinitySlotsState,
});
export type TrinityOrchestrateRequestT = z.infer<typeof TrinityOrchestrateRequest>;

/** Mirror of `TrinityOrchestrateResponse` — the full Trinity forging response. */
export const TrinityOrchestrateResponse = z.object({
  sessionId: z.string(),
  projectName: z.string(),
  artifactHtml: z.string(),
  artifactBytes: z.number().int().min(0),
  probeCount: z.number().int().min(0),
  integratedModules: z.array(z.string()),
  manifest: z.array(TrinitySubAgentDnaView),
  finalScore: z.number().int().min(0).max(100),
  approvedForListing: z.boolean(),
  maironBadge: z.string(),
  pillars: TrinityPillars,
  advisory: TrinityAdvisoryView,
  repairAttemptsUsed: z.number().int().min(0).max(3),
  repairHistory: z.array(TrinityRepairRoundView),
});
export type TrinityOrchestrateResponseT = z.infer<typeof TrinityOrchestrateResponse>;
